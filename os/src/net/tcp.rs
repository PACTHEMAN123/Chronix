use core::{fmt::UpperExp, future::Future, net::SocketAddr, sync::atomic::{AtomicBool, AtomicU8, Ordering}};

use crate::{ sync::{mutex::SpinNoIrqLock, UPSafeCell}, syscall::{sys_error::SysError, SysResult}, task::current_task, utils::yield_now};

use super::{addr::{SockAddr, ZERO_IPV4_ADDR, ZERO_IPV4_ENDPOINT}, listen_table::ListenTable, socket::Sock, SocketSetWrapper, LISTEN_TABLE, PORT_END, PORT_START, SOCKET_SET, SOCK_RAND_SEED};
use alloc::vec::Vec;
use smoltcp::{
    iface::SocketHandle,
    socket::tcp::{self, ConnectError, State},
    wire::{IpAddress, IpEndpoint, IpListenEndpoint},
};
use spin::Spin;
use super::socket::SockResult;
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
use rand::RngCore;
use log::info;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SocketState {
    /// Socket is not working
    Closed = 0,
    /// Socket is waiting for connection
    Busy = 1,
    /// Socket is connecting(for user socket)
    Connecting = 2,
    /// Socket is connected(for user socket)
    Connected = 3,
    /// Socket is listening(for server socket)
    Listening = 4,
}

impl From<u8> for SocketState {
    fn from(value: u8) -> Self {
        match value {
            0 => SocketState::Closed,
            1 => SocketState::Busy,
            2 => SocketState::Connecting,
            3 => SocketState::Connected,
            4 => SocketState::Listening,
            _ => panic!("Invalid SocketState value"),
        }
    }
}
/// TCP Socket
pub struct TcpSocket {
    /// socket state
    state: AtomicU8,
    /// socket handle
    handle: UPSafeCell<Option<SocketHandle>>,
    /// local endpoint
    local_endpoint: UPSafeCell<Option<IpEndpoint>>,
    /// remote endpoint
    remote_endpoint: UPSafeCell<Option<IpEndpoint>>,
    /// whether in non=blokcing mode
    nonblock_flag: AtomicBool
}

unsafe impl Send for TcpSocket {}
unsafe impl Sync for TcpSocket {}

impl TcpSocket {
    /// new a TcpSocket without a socket handle (Still not get in the SocketSet)
    pub const fn new_v4_without_handle() -> Self {
        Self {
            state: AtomicU8::new(SocketState::Closed as u8),
            handle: UPSafeCell::const_new(None),
            local_endpoint: UPSafeCell::const_new(Some(ZERO_IPV4_ENDPOINT)),
            remote_endpoint: UPSafeCell::const_new(Some(ZERO_IPV4_ENDPOINT)),
            nonblock_flag: AtomicBool::new(false),
        }
    }
    /// create a TcpSocket with a socket handle
    pub const fn new_v4_connected(handle: SocketHandle, local_endpoint: IpEndpoint, remote_endpoint: IpEndpoint) -> Self {
        Self {
            state: AtomicU8::new(SocketState::Connected as u8),
            handle: UPSafeCell::const_new(Some(handle)),
            local_endpoint: UPSafeCell::const_new(Some(local_endpoint)),
            remote_endpoint: UPSafeCell::const_new(Some(remote_endpoint)),
            nonblock_flag: AtomicBool::new(false),
        }
    }
    /// get the socket state
    pub fn state(&self) -> SocketState {
        self.state.load(Ordering::SeqCst).into()
    }
    /// set the socket state
    pub fn set_state(&self, state: u8) {
        self.state.store(state, Ordering::SeqCst)
    }
    pub fn update_state<F, T>(&self, expect_state: SocketState, new_state: SocketState, f: F) -> Result<SockResult<T>, u8>
    where 
        F: FnOnce() -> SockResult<T>,
    {
        match self.state
        .compare_exchange(expect_state as u8, SocketState::Busy as u8, Ordering::Acquire, Ordering::Acquire)
        {
            Ok(_) => {
                let res = f();
                if res.is_ok() {
                    self.set_state(new_state as u8);
                }else {
                    self.set_state(expect_state as u8);
                }
                Ok(res)
            }
            Err(actual_state) => {Err(actual_state as u8)}
        }
    }
    /// get the socket handle mut ref
    pub fn mut_handle(&self) -> Option<&mut SocketHandle> {
        self.handle.exclusive_access().as_mut()
    }
    /// get the socket handle ref
    pub fn handle(&self) -> Option<&SocketHandle> {
        self.handle.get_ref().as_ref()
    }
    /// set the socket handle
    pub fn set_handle(&self, handle: SocketHandle) {
        self.handle.exclusive_access().replace(handle);
    }
    /// get the local endpoint ref
    pub fn local_endpoint(&self) -> &IpEndpoint {
        self.local_endpoint.get_ref().as_ref().unwrap()
    }
    /// set the local endpoint
    pub fn set_local_endpoint(&self, endpoint: IpEndpoint) {
        self.local_endpoint.exclusive_access().replace(endpoint);
    }
    pub fn set_local_endpoint_with_port(&self, port: u16) {
        let inner_endpoint = self.local_endpoint.exclusive_access().clone().unwrap();
        let addr = inner_endpoint.addr;
        self.local_endpoint.exclusive_access().replace(IpEndpoint::new(addr, port));
    }
    /// get the remote endpoint ref
    pub fn remote_endpoint(&self) -> &IpEndpoint {
        self.remote_endpoint.get_ref().as_ref().unwrap()
    }
    /// set the remote endpoint
    pub fn set_remote_endpoint(&mut self, endpoint: IpEndpoint) {
        self.remote_endpoint.exclusive_access().replace(endpoint);
    }
    /// set non-blocking mode
    pub fn set_nonblock(&self, nonblock: bool) {
        self.nonblock_flag.store(nonblock, Ordering::SeqCst)
    }
    /// get non-blocking mode
    pub fn nonblock(&self) -> bool {
        self.nonblock_flag.load(Ordering::SeqCst)
    }
}

impl Sock for TcpSocket {
    async fn connect(&self, _addr: IpEndpoint) ->SockResult<()>{
        // todo
        Ok(())
    }
    
    fn bind(&self, _sock_fd: usize, addr: IpListenEndpoint) -> SockResult<()>  {
        let inner_addr = if addr.addr.is_some(){
            addr.addr.unwrap()
        }else {
            ZERO_IPV4_ADDR
        };
        let mut new_endpoint = IpEndpoint::new(inner_addr, addr.port);
        self.update_state(SocketState::Closed, SocketState::Closed,||{
            if new_endpoint.port == 0 {
                let port = self.get_ephemeral_port().unwrap();
                new_endpoint.port = port;
                info!("[TcpSocket::bind] local port is 0, use port {}",port);
            }
            let old = self.local_endpoint().clone();
            if old != ZERO_IPV4_ENDPOINT {
                // already bind
                return Err(SysError::EADDRINUSE); 
            }
            if let IpAddress::Ipv6(v6) = inner_addr {
                if v6.is_unspecified() {
                    // change unspecified v6 address to v4 address
                    new_endpoint.addr = ZERO_IPV4_ADDR;
                }
            }
            self.set_local_endpoint(new_endpoint);
            Ok(())
        })
        .unwrap_or_else(|_|{
            info!("[TcpSocket::bind] failed to bind");
            Err(SysError::EINVAL)
        })
    }
    
    fn listen(&self) -> SockResult<()> {
        let waker = current_task().unwrap().waker_ref().as_ref().unwrap();
        self.update_state(SocketState::Closed, SocketState::Listening, ||{
            let inner_endpoint = self.robost_port_endpoint().unwrap();
            self.set_local_endpoint_with_port(inner_endpoint.port);
            LISTEN_TABLE.listen(inner_endpoint, waker);
            info!("[TcpSocket::listen] listening on endpoint which addr is {}, port is {}", inner_endpoint.addr.unwrap(),inner_endpoint.port);
            Ok(())
        }).unwrap_or_else(|_| {
            Ok(())
        })
    }
    
    fn set_nonblcoking(&self) {
        self.set_nonblock(true);
    }
    
    fn peer_addr(&self) -> Option<IpEndpoint> {
        match self.state() {
            SocketState::Connected | SocketState::Listening => {
                let remote_endpoint = self.remote_endpoint().clone();
                Some(remote_endpoint)
            }
            _ => None,
        }
    }
    
    fn local_addr(&self) -> Option<IpEndpoint> {
        match self.state() {
            SocketState::Connected | SocketState::Listening => {
                let local_endpoint = self.local_endpoint().clone();
                Some(local_endpoint)
            }
            _ => None,
        }
    }
    
    async fn send(&self, data: &[u8], remote_addr: IpEndpoint) -> usize {
        todo!()
    }
    
    async fn recv(&self, data: &mut [u8]) -> (usize, IpEndpoint) {
        todo!()
    }
}

impl TcpSocket {
    fn get_ephemeral_port(&self) -> SockResult<u16> {
        let mut small_rng = SmallRng::seed_from_u64(SOCK_RAND_SEED);
        static CURR: SpinNoIrqLock<u16> = SpinNoIrqLock::new(PORT_START);
        // 1. quick temp random scan
        let mut attempt = 0;
        while attempt < 3 { // at most 3 attempts
            let _base = {
                let mut curr = CURR.lock();
                let base = *curr;
                // every time randomely increase the step size:（1-1023）
                *curr = curr.wrapping_add(small_rng.random::<u16>() % 1024 + 1);
                if *curr < PORT_START || *curr > PORT_END {
                    *curr = PORT_START;
                }
                base
            };

            // 2. from base randomly scam PORT_MAX_ATTEMPTS 
            const PORT_MAX_ATTEMPTS: usize = 128; // every time tries 128 ports at most
            let ports: Vec<u16> = (0..PORT_MAX_ATTEMPTS)
                .map(|_| small_rng.random_range(PORT_START..=PORT_END))
                .collect();
    
            for &port in &ports {
                if LISTEN_TABLE.can_listen(port) {
                    return Ok(port);
                }
            }
    
            attempt += 1;
        }
    
        // 3. back to the usual way
        let mut curr = CURR.lock();
        let start_port = *curr;
        let mut port = start_port;
        loop {
            port = if port == PORT_END {
                PORT_START
            } else {
                port + 1
            };
    
            if LISTEN_TABLE.can_listen(port) {
                *curr = port; 
                return Ok(port);
            }
    
            if port == start_port {
                break; 
            }
        }
        Err(SysError::EADDRINUSE)
    }
    /// read current endpoint and make it robust if it lack port or anything else
    fn  robost_port_endpoint(&self) -> SockResult<IpListenEndpoint> {
        let local_endpoint = self.local_endpoint().clone();
        let port = if local_endpoint.port == 0 {
            self.get_ephemeral_port()?
        }else {
            local_endpoint.port
        };
        let addr = if local_endpoint.addr.is_unspecified() {
            None
        }else {
            Some(local_endpoint.addr)
        };
        Ok(IpListenEndpoint {
            addr,
            port,
        })
    }
}