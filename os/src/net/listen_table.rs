use alloc::{boxed::Box, collections::VecDeque, vec::Vec};
use core::{
    ops::{Deref, DerefMut},
    task::Waker,
};
use smoltcp::{
    iface::{SocketHandle, SocketSet},
    socket::tcp::{self, State},
    wire::{IpAddress, IpEndpoint, IpListenEndpoint},
};

use crate::{sync::mutex::SpinNoIrqLock, syscall::sys_error::SysError};

use super::{socket::SockResult, LISTEN_QUEUE_SIZE,SOCKET_SET};
/// u16 num 
const PORT_NUM: usize = 65536;
/// entry for listen table
struct ListenEntry{
    /// ip endpoint that listen on
    listen_endpoint: IpListenEndpoint,
    /// temporary holding area for half-open connections
    /// —that is, connection requests that have received a SYN from a client, 
    /// but have not yet completed the three-way handshake.
    syn_queue: VecDeque<SocketHandle>,
    /// waker for waiting for incoming connection
    waker: Waker,
}

impl ListenEntry {
    pub fn new(listen_endpoint: IpListenEndpoint, waker: &Waker) -> Self {
        Self {
            listen_endpoint,
            syn_queue: VecDeque::with_capacity(LISTEN_QUEUE_SIZE),
            waker: waker.clone(),
        }
    }
    fn can_accept(&self, dst: IpAddress) -> bool {
        match self.listen_endpoint.addr {
            Some(addr) => addr == dst,
            None => true,
        }
    }
    /// get self waker wake
    pub fn wake(self) {
        self.waker.wake_by_ref()
    }
}

/// A table for managing TCP listen ports.
/// Each index corresponds to a specific port number.
pub struct ListenTable {
    inner: Box<[SpinNoIrqLock<Option<Box<ListenEntry>>>]>,
}

impl ListenTable {
    /// Create a new empty `ListenTable`.
    pub fn new() -> Self {
        let inner = unsafe {
            let mut buf = Box::new_uninit_slice(PORT_NUM);
            for i in 0..PORT_NUM {
                buf[i].write(SpinNoIrqLock::new(None));
            }
            buf.assume_init()
        };
        Self { inner }
    }
    /// check if a port can listen
    pub fn can_listen(&self, port: u16) -> bool {
        self.inner[port as usize].lock().is_none()
    }
    /// set a port listen
    pub fn listen(&self, listen_endpoint: IpListenEndpoint, waker: &Waker)-> SockResult<()> {
        let port = listen_endpoint.port;
        let mut entry = self.inner[port as usize].lock();
        if entry.is_none() {
            *entry = Some(Box::new(ListenEntry::new(listen_endpoint, waker)));
            Ok(())
        }
        else {
            Err(SysError::EADDRINUSE)
        }
    }
    /// unlisten a port
    pub fn unlisten(&self, port: u16) {
        if let Some(entry) = self.inner[port as usize].lock().take() {
            entry.wake()
        }
    }
    /// check wheter a port can accept connection
    pub fn can_accept(&self, port: u16) -> SockResult<bool> {
        if let Some(entry) = self.inner[port as usize].lock().deref(){
            Ok(entry.syn_queue.iter().any(|&handle| is_connected(handle)))
        }else {
            Err(SysError::EINVAL)
        }
    }
}

fn is_connected(handle: SocketHandle) -> bool {
    true
}