use alloc::vec;
use listen_table::ListenTable;
use smoltcp::{iface::{SocketSet,SocketHandle}, socket::AnySocket};
use spin::Lazy;

use crate::sync::mutex::{SpinNoIrq, SpinNoIrqLock};

/// Network Address Module
pub mod addr;
/// Network Socket Module
pub mod socket;
/// TCP Module
pub mod tcp;
/// A Listen Table for Server to allocte port
pub mod listen_table;
#[repr(u16)]
#[derive(Debug, Clone, Copy)]
/// socket address family
pub enum SaFamily {
    /// ipv4
    AfInet = 2,
    /// ipv6
    AfInet6 = 10,
}

impl TryFrom<u16> for SaFamily {
    type Error = crate::syscall::sys_error::SysError;
    fn try_from(value: u16) -> Result<Self,Self::Error> {
        match value {
            2 => Ok(Self::AfInet),
            10 => Ok(Self::AfInet6),
            _ => Err(Self::Error::EINVAL),
        }
    }
}

const SOCK_RAND_SEED: u64 = 404;
const PORT_START: u16 = 0xc000; // 49152
const PORT_END: u16 = 0xffff;   // 65535

const LISTEN_QUEUE_SIZE: usize = 512;
static LISTEN_TABLE: Lazy<ListenTable> = Lazy::new(ListenTable::new);
struct SocketSetWrapper<'a>(SpinNoIrqLock<SocketSet<'a>>) ; 
static SOCKET_SET: Lazy<SocketSetWrapper> = Lazy::new(SocketSetWrapper::new);
/// TCP RX and TX buffer size
pub const TCP_RX_BUF_LEN: usize = 64 * 1024;
/// TCP RX and TX buffer size
pub const TCP_TX_BUF_LEN: usize = 64 * 1024;

impl <'a> SocketSetWrapper<'a> {
    fn new() -> Self {
        let socket_set = SocketSet::new(vec![]);
        Self(SpinNoIrqLock::new(socket_set))
    }
    /// allocate tx buffer and rx buffer ,return a Socket struct in smoltcp
    pub fn new_tcp_socket() -> smoltcp::socket::tcp::Socket<'a> {
        let rx_buffer = smoltcp::socket::tcp::SocketBuffer::new(vec![0; TCP_RX_BUF_LEN]);
        let tx_buffer = smoltcp::socket::tcp::SocketBuffer::new(vec![0; TCP_TX_BUF_LEN]);
        smoltcp::socket::tcp::Socket::new(rx_buffer, tx_buffer)
    }
    /// add a socket to the set , return a socket_handle
    pub fn add_socket<T:AnySocket<'a>>(&self, socket: T) -> SocketHandle {
        let handle = self.0.lock().add(socket);
        handle
    }
    /// use a ref of socket and do something with it
    pub fn with_socket<T: AnySocket<'a>, R, F>(&self, handle: SocketHandle, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let set = self.0.lock();
        let socket = set.get(handle);
        f(socket)
    }
    /// use a mut ref of socket and do something with it
    pub fn with_socket_mut<T: AnySocket<'a>, R, F>(&self, handle: SocketHandle, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut set = self.0.lock();
        let socket = set.get_mut(handle);
        f(socket)
    }
}