use core::{fmt::UpperExp, future::Future, net::SocketAddr, sync::atomic::{AtomicBool, AtomicU8, Ordering}, time::{self, Duration}};

use crate::{ net::addr::LOCAL_IPV4, sync::{mutex::SpinNoIrqLock, UPSafeCell}, syscall::{sys_error::SysError, SysResult}, task::current_task, timer::timed_task::ksleep, utils::{get_waker, suspend_now, yield_now}};

use super::{addr::{ ZERO_IPV4_ADDR, ZERO_IPV4_ENDPOINT}, listen_table::ListenTable, socket::{PollState, Sock}, NetPollTimer, SocketSetWrapper, ETH0, LISTEN_TABLE, PORT_END, PORT_START, RCV_SHUTDOWN, SEND_SHUTDOWN, SHUTDOWN_MASK, SHUTRD, SHUTRDWR, SHUTWR, SOCKET_SET, SOCK_RAND_SEED, TCP_TX_BUF_LEN};
use alloc::vec::Vec;
use fatfs::warn;
use hal::println;
use smoltcp::{
    iface::{SocketHandle, SocketSet},
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
    nonblock_flag: AtomicBool,
    /// shutdown flag
    shutdown_flag: UPSafeCell<u8>,
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
            shutdown_flag: UPSafeCell::const_new(0),
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
            shutdown_flag: UPSafeCell::const_new(0),
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
    pub fn handle(&self) -> Option<SocketHandle> {
        unsafe{
            self.handle.get().read()
        }
    }
    /// set the socket handle
    pub fn set_handle(&self, handle: SocketHandle) {
        unsafe {
            self.handle.get().write(Some(handle));
        }
    }
    /// get the local endpoint ref
    pub fn local_endpoint(&self) -> Option<IpEndpoint> {
        unsafe{
            self.local_endpoint.get().read()
        }
    }
    /// set the local endpoint
    pub fn set_local_endpoint(&self, endpoint: IpEndpoint) {
        unsafe{
            self.local_endpoint.get().write(Some(endpoint));
        }
    }
    pub fn set_local_endpoint_with_port(&self, port: u16) {
        let inner_endpoint = self.local_endpoint.exclusive_access().clone().unwrap();
        let addr = inner_endpoint.addr;
        unsafe {
            self.local_endpoint.get().write(Some(IpEndpoint::new(addr, port)));
        }
    }
    /// get the remote endpoint ref
    pub fn remote_endpoint(&self) -> Option<IpEndpoint> {
        unsafe {
            self.remote_endpoint.get().read()
        }
    }
    /// set the remote endpoint
    pub fn set_remote_endpoint(&self, endpoint: IpEndpoint) {
        unsafe{
            self.remote_endpoint.get().write(Some(endpoint));
        }
    }
    /// set non-blocking mode
    pub fn set_nonblock(&self, nonblock: bool) {
        self.nonblock_flag.store(nonblock, Ordering::SeqCst)
    }
    /// get non-blocking mode
    pub fn nonblock(&self) -> bool {
        self.nonblock_flag.load(Ordering::SeqCst)
    }
    /// get shutdown flag
    pub fn get_shutdown(&self) -> u8 {
        self.shutdown_flag.exclusive_access().clone()
    }
    /// set shutdown flag
    pub fn set_shutdown(&self, flag: u8) {
        unsafe {
            self.shutdown_flag.get().write(flag)
        }
    }
}

impl TcpSocket {
    pub async fn connect(&self, addr: IpEndpoint) ->SockResult<()>{
        // first yield now 
        yield_now().await;
        // now change the state to connecting , wait for poll connect event
        self.update_state(SocketState::Closed, SocketState::Connecting, ||{
            let handle = self.handle().unwrap_or_else(||SOCKET_SET.add_socket(SocketSetWrapper::new_tcp_socket()));
            let robust_endpoint = self.robost_port_endpoint()?;
            let (local_endpoint, remote_endpoint) = SOCKET_SET.with_socket_mut::<tcp::Socket, _, _>(handle, |socket|{
                socket.connect(ETH0.get().unwrap().iface.lock().context(),addr,robust_endpoint)
                .or_else(|e| match e {
                    ConnectError::InvalidState => {
                        log::warn!("[TcpSocket::connect] failed: InvalidState");
                        Err(SysError::EBADF)
                    }
                    ConnectError::Unaddressable => {
                        log::warn!("[TcpSocket::connect] failed: Unaddressable");
                        Err(SysError::EADDRNOTAVAIL)
                    }
                })?;
                Ok((socket.local_endpoint(), socket.remote_endpoint()))
            })?;
            self.set_local_endpoint(local_endpoint.unwrap());
            self.set_remote_endpoint(remote_endpoint.unwrap());
            self.set_handle(handle);
            // log::info!("[TCP CONNCECT], local_endpoint_port: {}, remote_endpoint_port:{}", self.local_endpoint().port,self.remote_endpoint().port);
            Ok(())
        }).unwrap_or_else(|_|{
            log::warn!("[TcpSocket::connect] failed to connect for alreay connected socket");
            Err(SysError::EEXIST)
        })?;
        
        // up to now the state is connecting, wait for poll connect event
        if self.nonblock() {
            Err(SysError::EINPROGRESS)
        }else {
            self.block_on_future(|| async {
                let connection_info = self.poll_connect().await;
                if !connection_info {
                    log::warn!("[TcpSocket::connect] try agian");
                    Err(SysError::EAGAIN)
                }else if self.state() == SocketState::Connected {
                    Ok(())
                }else {
                    log::warn!("[TcpSocket::connect] connection refused");
                    Err(SysError::ECONNREFUSED)
                }
            }).await
        }
    }
    
    pub fn bind(&self, mut new_endpoint: IpEndpoint) -> SockResult<()>  {
        // log::info!("[TcpSocket::bind] start to bind");
        self.update_state(SocketState::Closed, SocketState::Closed,||{
            // info!("new end point port {}", new_endpoint.port);
            if new_endpoint.port == 0 {
                let port = self.get_ephemeral_port().unwrap();
                new_endpoint.port = port;
                // info!("[TcpSocket::bind] local port is 0, use port {}",port);
            }
            let old = unsafe {
                self.local_endpoint.get().read().unwrap()
            };
            if old != ZERO_IPV4_ENDPOINT {
                // already bind
                return Err(SysError::EINVAL); 
            }
            if let IpAddress::Ipv6(v6) = new_endpoint.addr {
                if v6.is_unspecified() {
                    // change unspecified v6 address to v4 address
                    new_endpoint.addr = ZERO_IPV4_ADDR;
                }
            }  
            self.set_local_endpoint(new_endpoint);
            // info!("now self local endpoint port {}",unsafe {
            //     self.local_endpoint.get().read().unwrap().port
            // });
            Ok(())
        })
        .unwrap_or_else(|_|{
            info!("[TcpSocket::bind] failed to bind");
            Err(SysError::EINVAL)
        })
    }
    
    pub fn listen(&self) -> SockResult<()> {
        let binding = current_task().unwrap().clone();
        let waker = binding.waker_ref().as_ref().unwrap();
        self.update_state(SocketState::Closed, SocketState::Listening, ||{
            let inner_endpoint = self.robost_port_endpoint()?;
            self.set_local_endpoint_with_port(inner_endpoint.port);
            LISTEN_TABLE.listen(inner_endpoint, waker)?;
            // info!("[TcpSocket::listen] listening on endpoint which addr is {}, port is {}", inner_endpoint.addr.unwrap(),inner_endpoint.port);
            Ok(())
        }).unwrap_or_else(|_| {
            Ok(())
        })
    }
    
    pub fn set_nonblocking(&self) {
        self.set_nonblock(true);
    }
    
    pub fn peer_addr(&self) -> SockResult<IpEndpoint> {
        match self.state() {
            SocketState::Connected | SocketState::Listening => {
                let remote_endpoint = self.remote_endpoint().unwrap();
                Ok(remote_endpoint)
            }
            _ => Err(SysError::ENOTCONN),
        }
    }
    
    pub fn local_addr(&self) -> SockResult<IpEndpoint> {
        match self.state() {
            SocketState::Connected | SocketState::Listening | SocketState::Closed => {
                let local_endpoint = self.local_endpoint().unwrap();
                Ok(local_endpoint)
            }
            _ => Err(SysError::ENOTCONN),
        }
    }
    
    pub async fn send(&self, data: &[u8], _remote_addr: Option<IpEndpoint>) -> SockResult<usize> {
        let shutdown = self.get_shutdown();
        if shutdown & SEND_SHUTDOWN != 0 {
            log::warn!("[TcpSocket::send] shutdown&SEND_SHUTDOWN != 0, return 0");
            return Ok(0);
        }
        if self.state() == SocketState::Connecting {
            return Err(SysError::EAGAIN);
        }else if self.state() != SocketState::Connected && shutdown == 0 {
            return Err(SysError::ENOTCONN);
        }else {
            let handle = self.handle().unwrap();
            let waker = get_waker().await;
            let ret = self.block_on(|| {
                SOCKET_SET.with_socket_mut::<tcp::Socket,_,_>( handle, |socket| {
                    if !socket.is_active() || !socket.may_send() {
                        return Err(SysError::ECONNRESET);
                    }else if socket.can_send() {
                        let len = socket.send_slice(data).map_err(|_| {
                            log::warn!("send error beacuse of EBADF");
                            SysError::EBADF
                        })?;
                        Ok(len)
                    }else {
                         // tx buffer is full
                        log::info!("[TcpSocket::send] handle{handle} send buffer is full, register waker and suspend");
                        socket.register_send_waker(&waker);
                        Err(SysError::EAGAIN)
                    }
                })
            }).await; 
            if let Ok(bytes) = ret {
                if bytes > TCP_TX_BUF_LEN / 2 {
                    ksleep(Duration::from_millis(3)).await;
                } else {
                    yield_now().await;
                }
            }
            SOCKET_SET.poll_interfaces();
            ret
        }
    }
    
    pub async fn recv(&self, data: &mut [u8]) -> SockResult<(usize, IpEndpoint)> {
        let shutdown = self.get_shutdown();
        if shutdown & RCV_SHUTDOWN != 0 {
            info!("[tcp socket] shutdown&RCV_SHUTDOWN != 0, return 0");
            let peer_addr = self.peer_addr()?;
            return Ok((0, peer_addr));
        }
        if self.state() == SocketState::Connecting {
            return Err(SysError::EAGAIN);
        }
        else if self.state() != SocketState::Connected && shutdown == 0 {
            return Err(SysError::ENOTCONN);
        }
        else {
            let peer_addr = self.peer_addr()?;
            let handle = self.handle().unwrap();
            let waker = get_waker().await;
            self.block_on(|| {
                SOCKET_SET.with_socket_mut::<tcp::Socket,_,_>(handle, |socket|{
                    if !socket.is_active() {
                        // not open 
                        log::warn!("[TcpSocket::recv] socket recv() failed because handle is not active");
                        return Err(SysError::ECONNREFUSED);
                    }else if !socket.may_recv() {
                        return Ok((0,peer_addr));
                    }else if socket.recv_queue() > 0 {
                        //data available
                        let len = socket.recv_slice(data).map_err(|_|{
                            log::warn!("socket recv failed becasue of bad state");
                            SysError::EBADF
                        })?;
                        return Ok((len, peer_addr))
                    }else {
                        // no more data
                        // log::info!("[TcpSocket::recv] handle{handle} has no data to recv, register waker and suspend");
                        socket.register_recv_waker(&waker);
                        Err(SysError::EAGAIN)
                    }
                })
            }).await
        }
        
    }

    pub fn shutdown(&self, how: u8) -> SockResult<()> {
        let mut shutdown = self.get_shutdown();
        match how {
            SHUTRD => shutdown |= RCV_SHUTDOWN,
            SHUTWR => shutdown |= SEND_SHUTDOWN,
            SHUTRDWR => shutdown |= SHUTDOWN_MASK,
            _ => return Err(SysError::EINVAL),
        }
        self.set_shutdown(shutdown);
        // for stream socket
        self.update_state(SocketState::Connected, SocketState::Closed, ||  {
            let handle = self.handle().unwrap();
            SOCKET_SET.with_socket_mut::<tcp::Socket, _, _,>(handle, |socket| {
                // info!("tcp socket shutdown, before state is {}", socket.state());
                socket.close();
                // info!("tcp socket shutdown, after state is {}" , socket.state());
            });
            let time_instance = SOCKET_SET.poll_interfaces();
            SOCKET_SET.check_poll(time_instance);
            Ok(())
        }).unwrap_or(Ok(()))?;
        // for listener socket
        self.update_state(SocketState::Listening, SocketState::Closed, ||{
            let local_port = self.local_endpoint().unwrap().port;
            self.set_local_endpoint(ZERO_IPV4_ENDPOINT);
            LISTEN_TABLE.unlisten(local_port);
            let time_instance = SOCKET_SET.poll_interfaces();
            SOCKET_SET.check_poll(time_instance);
            Ok(())
        }).unwrap_or(Ok(()))?;
        Ok(()) 
    }
    pub async fn poll(&self) -> PollState {
        match self.state() {
            SocketState::Connecting => {
                let writable = self.poll_connect().await;
                PollState {
                    readable: false,
                    writable: writable,
                    hangup: false,
                }
            },
            SocketState::Closed => {
                let hangup = self.poll_closed();
                PollState {
                    readable: false,
                    writable: false,
                    hangup: hangup,
                }
            },
            SocketState::Busy => PollState { readable: false, writable: false, hangup: false },
            SocketState::Connected => self.poll_stream().await,
            SocketState::Listening => {
                let readable = self.poll_listener();
                PollState {
                    readable,
                    writable: false,
                    hangup: false,
                }
            },
        }
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
        let local_endpoint = self.local_endpoint().unwrap();
        let port = if local_endpoint.port == 0 {
            // info!("get a random port");
            self.get_ephemeral_port()?
        }else {
            local_endpoint.port
        };
        // info!("[robost_port_endpoint] now port is {} ",port);
        let addr = if local_endpoint.addr.is_unspecified() {
            // log::warn!("[robost_port_endpoint] local endpoint addr is unspecified, use ipv4 local addr");
            Some(LOCAL_IPV4)
        }else {
            Some(local_endpoint.addr)
        };
        // log::info!("[robost_port_endpoint] addr is {:?}, port is {}", addr, port);
        Ok(IpListenEndpoint {
            addr,
            port,
        })
    }
    /// block_on a future and wait for poll_connect to check its connection state
    async fn block_on_future<F, T, Future> (&self, mut f: F) -> SockResult<T>
    where 
        F: FnMut() -> Future,
        Future: core::future::Future<Output = SockResult<T>>,
        {
            // log::info!("in block on future");
            if self.nonblock() {
                f().await
            }else {
                loop {
                    let time_instance = SOCKET_SET.poll_interfaces();
                    let ret = f().await;
                    SOCKET_SET.check_poll(time_instance);
                    match ret {
                        Ok(res) => {
                            return Ok(res);
                        }
                        Err(SysError::EAGAIN) => {
                            log::warn!("[block_on_future] ret state:EAGAIN!");
                            suspend_now().await;
                            // TODO: check if the socket is still valid
                            continue;
                        }
                        Err(e) => {
                            return Err(e);
                    }
                }
            }
        }
    }
    async fn block_on<F, T>(&self, mut f: F) -> SockResult<T>
    where 
        F: FnMut() -> SockResult<T>,
    {
        if self.nonblock() {
            f()
        }else {
            loop {
                let time_instance = SOCKET_SET.poll_interfaces();
                let ret = f();
                SOCKET_SET.check_poll(time_instance);
                match ret {
                    Ok(res) => {
                        return Ok(res);
                    }
                    Err(SysError::EAGAIN) => {
                        suspend_now().await;
                        continue;
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
        }
    }
    /// poll the tcp connect event and return true if the socket is connected
    async fn poll_connect(&self) -> bool {
        let handle = unsafe{self.handle.get().read()}.unwrap();
        let waker = get_waker().await;
        SOCKET_SET.with_socket_mut::<tcp::Socket,_,_>(handle, |socket|{
            match socket.state() {
                State::SynSent => {
                    // this means the request is sent, but not yet received by the remote endpoint
                    info!("[TcpSocket::poll_concect]:the request is sent, but not yet received by the remote endpoint ");
                    socket.register_recv_waker(&waker);
                    false
                }
                State::Established => {
                    // this means the connection is established
                    self.set_state(SocketState::Connected as u8);
                    // info!("[TcpSocket::poll_concect] socket is connected");
                    true
                }
                _ => {
                    log::warn!("wrong state, back to zero state");
                    self.local_endpoint.exclusive_access().replace(ZERO_IPV4_ENDPOINT);
                    self.remote_endpoint.exclusive_access().replace(ZERO_IPV4_ENDPOINT);
                    self.set_state(SocketState::Closed as u8);
                    true
                }
            } 
        })
    }
    async fn poll_stream(&self) -> PollState {
        let handle = unsafe {
            self.handle.get().read().unwrap()
        };
        let waker = get_waker().await;
        SOCKET_SET.with_socket_mut::<tcp::Socket,_,_>(handle, |socket|{
            let readable = !socket.may_recv()  || socket.can_recv();
            let writable = !socket.may_send() || socket.can_send();
            if !readable {
                socket.register_recv_waker(&waker);
            }  
            if !writable {
                socket.register_send_waker(&waker);
            }
            PollState {
                readable,
                writable,
                hangup: false,
            }
        })
    }

    fn poll_listener(&self) -> bool {
        let local_addr = self.local_addr().unwrap();
        let readable = LISTEN_TABLE.can_accept(local_addr.port);
        readable
    }

    fn poll_closed(&self) -> bool {
        let handle = unsafe {
            self.handle.get().read()
        };
        if let Some(handle) = handle {
            SOCKET_SET.with_socket_mut::<tcp::Socket,_,_>(handle, |socket| {
                log::warn!(
                    "[TcpSocket::poll_closed] handle {handle} state {}",
                    socket.state()
                );
                matches!(socket.state(), State::CloseWait| State::FinWait2 | State::TimeWait)
            })
        }else {
            false
        }
    }
    /// accept method for listener socket, only for tcp socket
    pub async fn accecpt(&self) -> SockResult<TcpSocket> {
        if self.state() != SocketState::Listening {
            log::warn!("socket accept state is not listening");
            return Err(SysError::EINVAL);
        }
        let local_port = self.local_endpoint().unwrap().port;
        // log::info!("[accept]: local_port is {}", local_port);
        self.block_on(|| {
            let (handle, (local_endpoint, remote_endpoint)) = LISTEN_TABLE.accept(local_port)?;
            // info!("TCP socket accepted a new connection {}", remote_endpoint);
            Ok(TcpSocket::new_v4_connected(handle, local_endpoint, remote_endpoint))
        }).await
    }
}

impl Drop for TcpSocket {
    fn drop (&mut self) {
        log::info!("[TcpSocket::drop]");
        self.shutdown(SHUTRDWR).ok();
        if let Some(handle) = unsafe{self.handle.get().read()} {
            SOCKET_SET.remove(handle);
        }
    }
}