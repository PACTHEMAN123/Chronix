use alloc::{boxed::Box, sync::Arc};
use async_trait::async_trait;
use smoltcp::wire::{IpEndpoint, IpListenEndpoint};
use crate::{fs::{vfs::{File, FileInner}, OpenFlags}, mm::UserBuffer, syscall::sys_error::SysError};
use crate::syscall::net::SocketType;
use super::{tcp::TcpSocket, SaFamily};
pub type SockResult<T> = Result<T, SysError>;
use spin::Mutex;
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
}
impl Sock {
    /// connect method for socket connect to remote socket, for user socket
    pub async fn connect(&self, addr: IpEndpoint) -> SockResult<()>{
        match self {
            Sock::TCP(tcp) => tcp.connect(addr).await
        }
    }
    /// bind method for socket to tell kernel which local address to bind to, for server socket
    pub fn bind(&self, sock_fd: usize, addr: IpListenEndpoint) -> SockResult<()>{
        match self {
            Sock::TCP(tcp) => tcp.bind(sock_fd, addr)
        }
    }
    /// listen method for socket to listen for incoming connections, for server socket
    pub fn listen(&self) -> SockResult<()>{
        match self {
            Sock::TCP(tcp) => tcp.listen()
        }
    }
    /// set socket non-blocking, 
    pub fn set_nonblocking(&self){
        match self {
            Sock::TCP(tcp) => tcp.set_nonblocking()
        }
    }
    /// get the peer_addr of the socket
    pub fn peer_addr(&self) -> Option<IpEndpoint>{
        match self {
            Sock::TCP(tcp) => tcp.peer_addr()
        }
    }
    /// get the local_addr of the socket
    pub fn local_addr(&self) -> Option<IpEndpoint>{
        match self {
            Sock::TCP(tcp) => tcp.local_addr()
        }
    }
    /// send data to the socket
    pub async fn send(&self, data: &[u8], remote_addr: IpEndpoint) -> SockResult<usize>{
        match self {
            Sock::TCP(tcp) => tcp.send(data, remote_addr).await
        }
    }
    /// recv data from the socket
    pub async fn recv(&self, data: &mut [u8]) -> SockResult<(usize, IpEndpoint)>{
        match self {
            Sock::TCP(tcp) => tcp.recv(data).await
        }
    }
    /// shutdown a connection
    pub fn shutdown(&self) -> SockResult<()>{
        match self {
            Sock::TCP(tcp) => tcp.shutdown()
        }
    }
    /// poll the socket for events
    pub async fn poll(&self) -> PollState{
        match self {
            Sock::TCP(tcp) => tcp.poll().await
        }
    }
    /// for tcp socket listener, accept a connection
    pub async fn accept(&self) -> SockResult<TcpSocket> {
        match self {
            Sock::TCP(tcp) => {
                let new  = tcp.accecpt().await?;
                Ok(new)
            }
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
    pub fd_flags: Mutex<OpenFlags>,
}

impl Socket {
    pub fn new(domain: SaFamily, sk_type: SocketType, non_block: bool) -> Self {
        let sk = match domain {
            SaFamily::AfInet | SaFamily::AfInet6 => {
                match sk_type {
                    SocketType::STREAM => Sock::TCP(TcpSocket::new_v4_without_handle()),
                    _ => unimplemented!(),
                }
            }
        };
        let fd_flags = if non_block {
            sk.set_nonblocking();
            OpenFlags::RDWR | OpenFlags::NONBLOCK
        }else {
            OpenFlags::RDWR
        };

        Self {
            sk_type: sk_type,
            sk: sk,
            fd_flags: Mutex::new(fd_flags),
        }
    }
    /// new a socket with a given socket 
    pub fn from_another(another: &Self, sk: Sock) -> Self {
        Self {
            sk: sk,
            sk_type: another.sk_type,
            fd_flags: Mutex::new(OpenFlags::RDWR),
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
        unreachable!()
    }

    #[doc = " If writable"]
    fn writable(&self) -> bool {
        unreachable!()
    }

    #[doc ="Read file to `UserBuffer`"]
    #[must_use]
    async fn read(&self, _buf: &mut [u8]) -> usize {
        todo!()
    }

    #[doc = " Write `UserBuffer` to file"]
    #[must_use]
    async fn write(& self, _buf: &[u8]) -> usize {
        todo!()
    }
}