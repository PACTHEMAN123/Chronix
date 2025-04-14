use core::task::Poll;

use alloc::{boxed::Box, sync::Arc};
use async_trait::async_trait;
use fatfs::info;
use smoltcp::{socket::udp, wire::{IpEndpoint, IpListenEndpoint}};
use crate::{fs::{vfs::{file::PollEvents, File, FileInner}, OpenFlags}, mm::UserBuffer, sync::mutex::SpinNoIrqLock, syscall::sys_error::SysError, task::current_task};
use crate::syscall::net::SocketType;
use super::{addr::{SockAddr, SockAddrIn4, ZERO_IPV4_ADDR}, poll_interfaces, tcp::TcpSocket, udp::UdpSocket, SaFamily};
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
    UDP(UdpSocket)
}
impl Sock {
    /// connect method for socket connect to remote socket, for user socket
    pub async fn connect(&self, addr: IpEndpoint) -> SockResult<()>{
        match self {
            Sock::TCP(tcp) => tcp.connect(addr).await,
            Sock::UDP(udp) => udp.connect(addr)
        }
    }
    /// bind method for socket to tell kernel which local address to bind to, for server socket
    pub fn bind(&self, sock_fd: usize, local_addr: SockAddr) -> SockResult<()>{
        match self {
            Sock::TCP(tcp) => {
                let local_addr = local_addr.into_listen_endpoint();
                let addr = if local_addr.addr.is_none(){
                    ZERO_IPV4_ADDR
                }else{
                    local_addr.addr.unwrap()
                };
                tcp.bind(IpEndpoint::new(addr, local_addr.port))
            }
            Sock::UDP(udp) => {
                let local_endpoint = local_addr.into_listen_endpoint();
                log::info!("[udp::bind] local_endpoint:{:?}", local_endpoint);
                if let Some(used_fd) = udp.bind_check(sock_fd, local_endpoint) {
                    current_task().unwrap()
                    .with_mut_fd_table(|t| t.dup3_with_flags(used_fd, sock_fd))?;
                    Ok(())
                }else {
                    udp.bind(local_endpoint)
                }
            }
        }
    }
    /// listen method for socket to listen for incoming connections, for server socket
    pub fn listen(&self) -> SockResult<()>{
        match self {
            Sock::TCP(tcp) => tcp.listen(),
            Sock::UDP(udp) => Err(SysError::EOPNOTSUPP)
        }
    }
    /// set socket non-blocking, 
    pub fn set_nonblocking(&self){
        match self {
            Sock::TCP(tcp) => tcp.set_nonblocking(),
            Sock::UDP(udp) => udp.set_nonblocking(),
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
        }
    }
    /// recv data from the socket
    pub async fn recv(&self, data: &mut [u8]) -> SockResult<(usize, IpEndpoint)>{
        match self {
            Sock::TCP(tcp) => tcp.recv(data).await,
            Sock::UDP(udp_socket) => udp_socket.recv(data).await,
        }
    }
    /// shutdown a connection
    pub fn shutdown(&self, how: u8) -> SockResult<()>{
        match self {
            Sock::TCP(tcp) => tcp.shutdown(how),
            Sock::UDP(udp_socket) => udp_socket.shutdown(),
        }
    }
    /// poll the socket for events
    pub async fn poll(&self) -> PollState{
        match self {
            Sock::TCP(tcp) => tcp.poll().await,
            Sock::UDP(udp_socket) => udp_socket.poll().await,
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
        }
    }
}
/// socket for user space,Related to network protocols and communication modes
pub struct Socket {
    /// sockets inner
    pub sk: Sock,
    /// socket type
    pub sk_type: SocketType,
    /// fd flags
    pub fd_flags: SpinNoIrqLock<OpenFlags>,
}

impl Socket {
    pub fn new(domain: SaFamily, sk_type: SocketType, non_block: bool) -> Self {
        let sk = match domain {
            SaFamily::AfInet | SaFamily::AfInet6 => {
                match sk_type {
                    SocketType::STREAM => Sock::TCP(TcpSocket::new_v4_without_handle()),
                    SocketType::DGRAM => Sock::UDP(UdpSocket::new()),
                    _ => unimplemented!(),
                }
            }
        };
        let fd_flags = if non_block {
            sk.set_nonblocking();
            OpenFlags::O_RDWR | OpenFlags::O_NONBLOCK
        }else {
            OpenFlags::O_RDWR
        };

        Self {
            sk_type: sk_type,
            sk: sk,
            fd_flags: SpinNoIrqLock::new(fd_flags),
        }
    }
    /// new a socket with a given socket 
    pub fn from_another(another: &Self, sk: Sock) -> Self {
        Self {
            sk: sk,
            sk_type: another.sk_type,
            fd_flags: SpinNoIrqLock::new(OpenFlags::O_RDWR),
        }
    }
}

#[async_trait]
impl File for Socket {
    #[doc ="get basic File object"]
    fn inner(&self) ->  &FileInner {
        unreachable!()
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
    async fn read(&self, buf: &mut [u8]) -> usize {
        log::info!("[Socket::read] buf len:{}", buf.len());
        if buf.len() == 0 {
            return 0;
        }
        let bytes = self.sk.recv(buf).await.map(|e|e.0).unwrap();
        bytes
    }

    #[doc = " Write `UserBuffer` to file"]
    #[must_use]
    async fn write(& self, buf: &[u8]) -> usize {
        if buf.len() == 0 {
            return 0;
        }
        let bytes = self.sk.send(buf, None).await.unwrap();
        bytes
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
}