use core::{sync::atomic::{self, AtomicBool, AtomicUsize}, task::Poll};

use alloc::{boxed::Box, string::String, sync::Arc};
use async_trait::async_trait;
use fatfs::info;
use smoltcp::{socket::udp, wire::{IpAddress, IpEndpoint, IpListenEndpoint}};
use crate::{fs::{vfs::{file::PollEvents, Dentry, File, FileInner}, OpenFlags}, net::{crypto::{AlgInstance, AlgType, SockAddrAlg}, LOCAL_IPS}, sync::mutex::SpinNoIrqLock, syscall::sys_error::SysError, task::current_task, timer::ffi::TimeSpec};
use crate::syscall::net::SocketType;
use super::{addr::{SockAddr, SockAddrIn4, ZERO_IPV4_ADDR}, poll_interfaces, tcp::TcpSocket, udp::UdpSocket, SaFamily, UnixSocket};
pub type SockResult<T> = Result<T, SysError>;
/// a trait for differnt socket types
/// net poll results.
#[derive(Debug, Default, Clone, Copy)]
pub struct PollState {
    /// Object can be read now.
    pub readable: bool,
    /// Object can be writen now.
    pub writable: bool,
    /// object has been hanguped waiting for polling.
    pub hangup: bool,
}
pub enum Sock {
    TCP(TcpSocket),
    UDP(UdpSocket),
    Unix(UnixSocket),
}
impl Sock {
    /// connect method for socket connect to remote socket, for user socket
    pub async fn connect(&self, addr: IpEndpoint) -> SockResult<()>{
        match self {
            Sock::TCP(tcp) => tcp.connect(addr).await,
            Sock::UDP(udp) => udp.connect(addr),
            Sock::Unix(_) =>  Err(SysError::EAFNOSUPPORT),
        }
    }
    /// bind method for socket to tell kernel which local address to bind to, for server socket
    pub fn bind(&self, sock_fd: usize, local_addr: SockAddr) -> SockResult<()>{
        match self {
            Sock::TCP(tcp) => {
                // let family = unsafe {
                //     SaFamily::try_from(local_addr.family)?
                // };
                let local_addr = local_addr.into_listen_endpoint()?;
                let addr = if local_addr.addr.is_none(){
                   ZERO_IPV4_ADDR
                }else{
                    local_addr.addr.unwrap()
                };
                if !LOCAL_IPS.contains(&addr) {
                    return Err(SysError::EADDRNOTAVAIL);
                }
                tcp.bind(IpEndpoint::new(addr, local_addr.port))
            }
            Sock::UDP(udp) => {
                let local_endpoint = local_addr.into_listen_endpoint()?;
                log::info!("[udp::bind] local_endpoint:{:?}", local_endpoint);
                if let Some(used_fd) = udp.bind_check(sock_fd, local_endpoint) {
                    current_task().unwrap()
                    .with_mut_fd_table(|t| t.dup3_with_flags(used_fd, sock_fd))?;
                    Ok(())
                }else {
                    udp.bind(local_endpoint)
                }
            }
            // todo: suit for most cases
            Sock::Unix(_) => Err(SysError::ENOTDIR)
        }
    }
    /// listen method for socket to listen for incoming connections, for server socket
    pub fn listen(&self) -> SockResult<()>{
        match self {
            Sock::TCP(tcp) => tcp.listen(),
            Sock::UDP(udp) => Err(SysError::EOPNOTSUPP),
            Sock::Unix(_) => Err(SysError::EAFNOSUPPORT),
        }
    }
    /// set socket non-blocking, 
    pub fn set_nonblocking(&self){
        match self {
            Sock::TCP(tcp) => tcp.set_nonblocking(),
            Sock::UDP(udp) => udp.set_nonblocking(),
            Sock::Unix(_) => {},
        }
    }
    /// get the peer_addr of the socket
    pub fn peer_addr(&self) -> SockResult<SockAddr>{
        match self {
            Sock::TCP(tcp) => {
                let peer_addr = tcp.peer_addr()?;
                Ok(SockAddr::from_endpoint(peer_addr))
            },
            Sock::UDP(udp_socket) => {
                let peer_addr = udp_socket.peer_addr()?;
                Ok(SockAddr::from_endpoint(peer_addr))
            },
            Sock::Unix(_) =>  Err(SysError::EAFNOSUPPORT),
        }
    }
    /// get the local_addr of the socket
    pub fn local_addr(&self) -> SockResult<SockAddr>{
        match self {
            Sock::TCP(tcp) => {
                let local_addr = tcp.local_addr()?;
                Ok(SockAddr::from_endpoint(local_addr))
            },
            Sock::UDP(udp_socket) => {
                let local_addr = udp_socket.local_addr()?;
                Ok(SockAddr::from_endpoint(local_addr))
            },
            Sock::Unix(_) =>  Err(SysError::EAFNOSUPPORT),
        }
    }
    /// send data to the socket
    pub async fn send(&self, data: &[u8], remote_addr: Option<IpEndpoint>) -> SockResult<usize>{
        match self {
            Sock::TCP(tcp) => tcp.send(data, remote_addr).await,
            Sock::UDP(udp_socket) => {
                match remote_addr {
                    Some(addr) => udp_socket.send_to(data,addr).await,
                    None => udp_socket.send(data).await,
                }
            },
            Sock::Unix(_) =>  Err(SysError::EAFNOSUPPORT),
        }
    }
    /// recv data from the socket
    pub async fn recv(&self, data: &mut [u8]) -> SockResult<(usize, IpEndpoint)>{
        match self {
            Sock::TCP(tcp) => tcp.recv(data).await,
            Sock::UDP(udp_socket) => udp_socket.recv(data).await,
            Sock::Unix(_) =>  Err(SysError::EAFNOSUPPORT),
        }
    }
    /// shutdown a connection
    pub fn shutdown(&self, how: u8) -> SockResult<()>{
        match self {
            Sock::TCP(tcp) => tcp.shutdown(how),
            Sock::UDP(udp_socket) => udp_socket.shutdown(),
            Sock::Unix(_) =>  Err(SysError::EAFNOSUPPORT),
        }
    }
    /// poll the socket for events
    pub async fn poll(&self) -> PollState{
        match self {
            Sock::TCP(tcp) => tcp.poll().await,
            Sock::UDP(udp_socket) => udp_socket.poll().await,
            Sock::Unix(_) => todo!(),
        }
    }
    /// for tcp socket listener, accept a connection
    pub async fn accept(&self) -> SockResult<TcpSocket> {
        match self {
            Sock::TCP(tcp) => {
                        let new  = tcp.accecpt().await?;
                        Ok(new)
                    }
            Sock::UDP(udp_socket) => Err(SysError::EOPNOTSUPP),
            Sock::Unix(_) =>  Err(SysError::EAFNOSUPPORT),
        }
    }
    /// set socket reuse addr
    pub fn set_reuse_addr(&self, reuse_flag: bool) {
        match self {
            Sock::TCP(tcp) => tcp.set_reuse_addr(reuse_flag),
            Sock::UDP(udp) => udp.set_reuse_addr(reuse_flag),
            Sock::Unix(_) => {},
        }
    }

    /// get_reuse_addr_flag
    pub fn get_reuse_addr_flag(&self) -> bool {
        match self {
            Sock::TCP(tcp) => tcp.get_reuse_addr(),
            Sock::UDP(udp) => udp.get_reuse_addr(),
            Sock::Unix(_) => false,
        }
    }
}
/// socket for user space,Related to network protocols and communication modes
pub struct Socket {
    /// sockets inner
    pub sk: Sock,
    /// socket type
    pub sk_type: SocketType,
    /// domain
    pub domain: SaFamily,
    /// fd flags
    pub file_inner: FileInner,
    /// some socket options
    /// send_buf_size
    pub send_buf_size: AtomicUsize,
    /// recv_buf_size
    pub recv_buf_size: AtomicUsize,
    /// congestion flag
    pub congestion:  SpinNoIrqLock<String>,
    /// socketopt dout route flag
    pub dont_route: bool,
    // !member concerning af_alg
    /// whether af_alg or not 
    pub is_af_alg: AtomicBool,
    /// socket_af_alg addr
    pub socket_af_alg: SpinNoIrqLock<Option<SockAddrAlg>>,
    /// key context
    pub alg_instance: SpinNoIrqLock<Option<AlgInstance>>
}

impl Socket {
    pub fn new(domain: SaFamily, sk_type: SocketType, non_block: bool) -> Self {
        let sk = match domain {
            SaFamily::AfInet | SaFamily::AfInet6 => {
                match sk_type {
                    SocketType::STREAM => Sock::TCP(TcpSocket::new_v4_without_handle()),
                    SocketType::DGRAM => Sock::UDP(UdpSocket::new()),
                    _ => Sock::TCP(TcpSocket::new_v4_without_handle()),
                }
            },
            SaFamily::AfUnix => Sock::Unix(UnixSocket {  }),
            SaFamily::Alg => Sock::TCP(TcpSocket::new_v4_without_handle()),
            _ => Sock::TCP(TcpSocket::new_v4_without_handle()),
        };
        let fd_flags = if non_block {
            sk.set_nonblocking();
            OpenFlags::O_RDWR | OpenFlags::O_NONBLOCK
        }else {
            OpenFlags::O_RDWR
        };

        Self {
            sk_type: sk_type,
            domain: domain,
            sk: sk,
            file_inner: FileInner {
                dentry: Arc::<usize>::new_zeroed(),
                offset: AtomicUsize::new(0),
                flags: SpinNoIrqLock::new(fd_flags),
            },
            send_buf_size: AtomicUsize::new(16 * 4096),
            recv_buf_size: AtomicUsize::new(16 * 4096),
            congestion: SpinNoIrqLock::new((String::from("reno"))),
            dont_route: false,
            is_af_alg: AtomicBool::new(false),
            socket_af_alg: SpinNoIrqLock::new(None),
            alg_instance: SpinNoIrqLock::new(None),
        }
    }
    /// new a socket with a given socket 
    pub fn from_another(another: &Self, sk: Sock) -> Self {
        Self {
            sk: sk,
            sk_type: another.sk_type,
            domain: another.domain,
            file_inner: FileInner{
                dentry: Arc::<usize>::new_zeroed(),
                offset: AtomicUsize::new(0),
                flags: SpinNoIrqLock::new(OpenFlags::O_RDWR),
            },
            send_buf_size: AtomicUsize::new(16 * 4096),
            recv_buf_size: AtomicUsize::new(16 * 4096),
            congestion: SpinNoIrqLock::new((String::from("reno"))),
            dont_route: false,
            is_af_alg: AtomicBool::new(false),
            socket_af_alg: SpinNoIrqLock::new(None),        
            alg_instance: SpinNoIrqLock::new(None),
        }
    }
    /// get send buf size
    pub fn get_send_buf_size(&self) -> usize {
        self.send_buf_size.load(atomic::Ordering::Acquire)
    }
    /// set send buf size
    pub fn set_send_buf_size(&self, size: usize) {
        self.send_buf_size.store(size, atomic::Ordering::Release)
    }
    // get recv buf size
    pub fn get_recv_buf_size(&self) -> usize {
        self.recv_buf_size.load(atomic::Ordering::Acquire)
    }
    /// set recv buf size
    pub fn set_recv_buf_size(&self, size: usize) {
        self.recv_buf_size.store(size, atomic::Ordering::Release)
    }
    /// get congestion state
    pub fn get_congestion(&self) -> String {
        self.congestion.lock().clone()
    }
    /// set congestion state
    pub fn set_congestion(&self, congestion: String) {
        *self.congestion.lock() = congestion;
    }
    /// set whether is af_alg socket
    pub fn set_is_af_alg(&self, is_af_alg: bool) {
        self.is_af_alg.store(is_af_alg, atomic::Ordering::Release);
    }
    /// get whether is af_alg socket
    pub fn get_is_af_alg(&self) -> bool {
        self.is_af_alg.load(atomic::Ordering::Acquire)
    }
}

#[async_trait]
impl File for Socket {
    #[doc ="get basic File object"]
    fn file_inner(&self) ->  &FileInner {
        &self.file_inner
    }

    #[doc = " If readable"]
    fn readable(&self) -> bool {
        true
    }

    #[doc = " If writable"]
    fn writable(&self) -> bool {
        true
    }

    #[doc ="Read file to `UserBuffer`"]
    #[must_use]
    async fn read(&self, buf: &mut [u8]) -> Result<usize, SysError> {
        log::info!("[Socket::read] buf len:{}", buf.len());
        if buf.len() == 0 {
            return Ok(0);
        }
        self.sk.recv(buf).await.map(|e|e.0)
    }

    #[doc = " Write `UserBuffer` to file"]
    #[must_use]
    async fn write(& self, buf: &[u8]) -> Result<usize, SysError> {
        if buf.len() == 0 {
            return Ok(0);
        }
        self.sk.send(buf, None).await.map(|e|e)
    }

    async fn base_poll(&self, events:PollEvents) -> PollEvents {
        let mut res = PollEvents::empty();
        poll_interfaces();
        let netstate = self.sk.poll().await;
        if events.contains(PollEvents::IN) && netstate.readable {
            res |= PollEvents::IN;
        }
        if events.contains(PollEvents::OUT) && netstate.writable {
            res |= PollEvents::OUT;
        }
        if netstate.hangup {
            log::warn!("[Socket::bask_poll] PollEvents is hangup");
            res |= PollEvents::HUP;
        }
        // log::info!("[Socket::base_poll] ret events:{res:?} {netstate:?}");
        res
    }

    fn dentry(&self) -> Option<Arc<dyn Dentry>> {
        None
    }
}

/// for alg socket
impl Socket {
    // For AF_ALG sockets, accept() behaves differently than for 
    // traditional network sockets. Instead of accepting a network 
    // connection, it creates a child socket (also called a "request 
    // socket"). This child socket is used for the actual data 
    // transmission (encryption or decryption operations). 
    // Typically, you call accept() to obtain a new child socket 
    // for each encryption or decryption operation.
    pub fn accept_alg(&self) -> SockResult<Self> {
        if self.domain != SaFamily::Alg || !self.get_is_af_alg() {
            return Err(SysError::EOPNOTSUPP);
        }
        Ok(Socket {
            sk: Sock::TCP(TcpSocket::new_v4_without_handle()),
            sk_type: self.sk_type,
            domain: self.domain.clone(),
            file_inner: FileInner{
                dentry: Arc::<usize>::new_zeroed(),
                offset: AtomicUsize::new(0),
                flags: SpinNoIrqLock::new(OpenFlags::O_RDWR),
            },
            send_buf_size: AtomicUsize::new(16 * 4096),
            recv_buf_size: AtomicUsize::new(16 * 4096),
            congestion: SpinNoIrqLock::new((String::from("reno"))),
            dont_route: false,
            is_af_alg: AtomicBool::new(true),
            socket_af_alg: SpinNoIrqLock::new(self.socket_af_alg.lock().clone()),        
            alg_instance: SpinNoIrqLock::new(self.alg_instance.lock().clone()),
        })
    }
}