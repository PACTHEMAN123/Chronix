use smoltcp::wire::{IpEndpoint, IpListenEndpoint};
use crate::syscall::sys_error::SysError;
pub type SockResult<T> = Result<T, SysError>;
/// a trait for differnt socket types
pub trait Sock {
    /// connect method for socket connect to remote socket, for user socket
    async fn connect(&self, addr: IpEndpoint) -> SockResult<()>;
    /// bind method for socket to tell kernel which local address to bind to, for server socket
    fn bind(&self, sock_fd: usize, addr: IpListenEndpoint) -> SockResult<()>;
    /// listen method for socket to listen for incoming connections, for server socket
    fn listen(&self) -> SockResult<()>; 
    /// set socket non-blocking, 
    fn set_nonblcoking(&self);
    /// get the peer_addr of the socket
    fn peer_addr(&self) -> Option<IpEndpoint>;
    /// get the local_addr of the socket
    fn local_addr(&self) -> Option<IpEndpoint>;
    /// send data to the socket
    async fn send(&self, data: &[u8], remote_addr: IpEndpoint) -> usize;
    /// recv data from the socket
    async fn recv(&self, data: &mut [u8]) -> (usize, IpEndpoint);
}