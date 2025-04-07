use core::{sync::atomic::AtomicBool, time};

use alloc::vec::Vec;
use fatfs::warn;
use lwext4_rust::bindings::EEXIST;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use smoltcp::{iface::SocketHandle, socket::{dns::GetQueryResultError, udp::{BindError, SendError}}, wire::{IpEndpoint, IpListenEndpoint}};
use spin::RwLock;

use crate::{net::{LISTEN_TABLE, PORT_END, PORT_START, SOCK_RAND_SEED}, sync::mutex::SpinNoIrqLock, syscall::{SysError, SysResult}, utils::{get_waker, suspend_now, yield_now}};

use super::{addr::{to_endpoint, SockAddr, UNSPECIFIED_LISTEN_ENDPOINT}, socket::SockResult, SocketSetWrapper, PORT_MANAGER, SOCKET_SET};

pub struct UdpSocket {
    /// socket handle
    handle: SocketHandle,
    /// local endpoint
    local_endpoint: RwLock<Option<IpListenEndpoint>>,
    /// remote endpoint
    peer_endpoint: RwLock<Option<IpEndpoint>>,
    /// nonblock flag
    nonblock_flag: AtomicBool,
}

impl UdpSocket {
    /// create a new UdpSocket
    pub fn new() -> Self {
        let socket = SocketSetWrapper::new_udp_socket();
        let handle = SOCKET_SET.add_socket(socket);
        Self {
            handle,
            local_endpoint: RwLock::new(None),
            peer_endpoint: RwLock::new(None),
            nonblock_flag: AtomicBool::new(false),
        }
    }
    /// check if the nonblock flag is nonblock
    pub fn is_nonblocking(&self) -> bool {
        self.nonblock_flag.load(core::sync::atomic::Ordering::Acquire)
    }
}

/// Sock impl
impl UdpSocket {
    /// bind the socket to a local endpoint
    pub fn bind(&self, mut local_endpoint: IpListenEndpoint) -> SockResult<()> {
        let mut local_addr = self.local_endpoint.write();
        if local_endpoint.port == 0 {
            local_endpoint.port = self.get_ephemeral_port()?;
        } 
        if local_addr.is_some() {
            return Err(SysError::EINVAL);
        }
        SOCKET_SET.with_socket_mut::<smoltcp::socket::udp::Socket,_,_>(self.handle, |socket|{
            socket.bind(local_endpoint).map_err(|e| {
                log::warn!("socket bind error: {}", e);
                match e {
                    BindError::InvalidState => SysError::EEXIST,
                    BindError::Unaddressable => SysError::EINVAL,
                }
            })
        })?;
        *local_addr = Some(local_endpoint);
        Ok(())
    }
    /// for udp socket, two socket can bound to one same port, but need to check if the local addr is the same
    pub fn bind_check(&self, fd: usize, mut robust_addr: IpListenEndpoint) -> Option<usize> {
        if let Some((fd, prev_bound_addr)) = PORT_MANAGER.get(robust_addr.port) {
            if robust_addr == prev_bound_addr {
                // reuse prev socket
                return Some(fd);
            }
        }
        if robust_addr.port == 0 {
            robust_addr.port = self.get_ephemeral_port().unwrap();
        }
        PORT_MANAGER.insert(robust_addr.port, fd, robust_addr);
        None
    } 
    /// set nonblock flag ture
    pub fn set_nonblocking(&self) {
        self.nonblock_flag.store(true, core::sync::atomic::Ordering::Release);
    }
    /// connect remote endpoint
    pub fn connect(&self, addr: IpEndpoint) -> SockResult<()> {
        if self.local_endpoint.read().is_none() {
            self.bind(UNSPECIFIED_LISTEN_ENDPOINT)?;
        }
        let mut peer_addr = self.peer_endpoint.write();
        *peer_addr = Some(addr);
        Ok(())
    }
    /// get the peer endpoint
    pub fn peer_addr(&self) -> SockResult<IpEndpoint> {
        match self.peer_endpoint.try_read() {
            Some(addr) => addr.ok_or(SysError::ENOTCONN),
            None => Err(SysError::ENOTCONN),
        }
    }
    /// get the local endpoint
    pub fn local_addr(&self) -> SockResult<IpEndpoint> {
        match self.local_endpoint.try_read() {
            Some(addr) => {
                addr.ok_or(SysError::ENOTCONN).map(to_endpoint)
            }
            None => Err(SysError::ENOTCONN),
        }
    }
    /// send data to the peer
    pub async fn send(&self, data: &[u8]) -> SockResult<usize> {
        let remote_endpoint = self.peer_addr()?;
        if self.local_endpoint.read().is_none() {
            self.bind(UNSPECIFIED_LISTEN_ENDPOINT)?;
        }
        let waker =get_waker().await;
        let bytes = self.block_on(|| {
            SOCKET_SET.with_socket_mut::<smoltcp::socket::udp::Socket,_,_>(self.handle, |socket|{
                if socket.can_send() {
                    socket.send_slice(data, remote_endpoint)
                    .map_err(|e|match e {
                        SendError::BufferFull => {
                            socket.register_send_waker(&waker);
                            SysError::EAGAIN
                        }
                        SendError::Unaddressable => {
                            SysError::ECONNREFUSED
                        }
                    })?;
                    Ok(data.len())
                }else {
                    socket.register_send_waker(&waker);
                    Err(SysError::EAGAIN)
                }
            })
        }).await?;
        yield_now().await;
        return Ok(bytes);
    }
    pub async fn send_to(&self, data: &[u8], remote_endpoint: IpEndpoint) -> SockResult<usize> {
        if remote_endpoint.port == 0 || remote_endpoint.addr.is_unspecified() {
            return Err(SysError::EINVAL);
        }
        if self.local_endpoint.read().is_none() {
            self.bind(UNSPECIFIED_LISTEN_ENDPOINT)?;
        }
        let waker = get_waker().await;
        let bytes = self.block_on(|| {
            SOCKET_SET.with_socket_mut::<smoltcp::socket::udp::Socket,_,_>(self.handle, |socket| {
                if socket.can_send() {
                    socket
                    .send_slice(data, remote_endpoint)
                    .map_err(|e|match e {
                         SendError::BufferFull => {
                             socket.register_send_waker(&waker);
                             SysError::EAGAIN
                         }
                         SendError::Unaddressable => {
                             SysError::ECONNREFUSED
                         }
                     })?;
                    Ok(data.len())
                }else {
                    socket.register_send_waker(&waker);
                    Err(SysError::EAGAIN)
                }
            })
        }).await?;
        yield_now().await;
        return Ok(bytes);
    }

    pub async fn recv(&self, data: &mut [u8]) -> SockResult<(usize, IpEndpoint)> {
        if self.local_endpoint.read().is_none() {
            return Err(SysError::ENOTCONN);
        }
        let waker = get_waker().await;
        let ret = self.block_on(||{
            SOCKET_SET.with_socket_mut::<smoltcp::socket::udp::Socket,_,_>(self.handle, |socket|{
                if socket.can_recv() {
                    match socket.recv_slice(data) {
                        Ok((len, meta)) => Ok((len, meta.endpoint)),
                        Err(e) => {
                            Err(SysError::EAGAIN)
                        }
                    }
                }else if !socket.is_open() {
                    Err(SysError::ENOTCONN)
                }
                else {
                    socket.register_recv_waker(&waker);
                    Err(SysError::EAGAIN)
                }
            })
        }).await?;
        yield_now().await;
        Ok(ret)
    }
    pub fn shutdown(&self) -> SockResult<()> {
        SOCKET_SET.with_socket_mut::<smoltcp::socket::udp::Socket,_,_>(self.handle, |socket| {
            socket.close();
        });
        let timestamp = SOCKET_SET.poll_interfaces();
        SOCKET_SET.check_poll(timestamp);
        Ok(())
    }
}


impl UdpSocket {
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

    async fn block_on<F, R>(&self, mut f: F) -> SockResult<R>
    where
        F: FnMut() -> SockResult<R>,
    {
        if self.is_nonblocking() {
            f()
        }else {
            loop {
                let timestamp = SOCKET_SET.poll_interfaces();
                let ret = f();
                SOCKET_SET.check_poll(timestamp);
                match ret {
                    Ok(r) => return Ok(r),
                    Err(SysError::EAGAIN) => {
                        suspend_now().await;
                    }
                    Err(e) => return Err(e),
                }
            }
        }
    }
}