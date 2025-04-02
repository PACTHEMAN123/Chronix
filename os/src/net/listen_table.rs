use alloc::{boxed::Box, collections::VecDeque, vec::Vec};
use fatfs::{info, warn};
use core::{
    ops::{Deref, DerefMut},
    task::Waker,
};
use smoltcp::{
    iface::{SocketHandle, SocketSet},
    socket::tcp::{self, State},
    wire::{IpAddress, IpEndpoint, IpListenEndpoint},
};

use crate::{net::SocketSetWrapper, sync::mutex::SpinNoIrqLock, syscall::sys_error::SysError};

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

impl Drop for ListenEntry {
    fn drop(&mut self) {
        for &handle in &self.syn_queue {
            SOCKET_SET.remove(handle);
        }
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
    /// accept a connection, check the syn queue and find the available connection
    pub fn accept(&self, port: u16) -> SockResult<(SocketHandle, (IpEndpoint, IpEndpoint))> {
        if let Some(entry) = self.inner[port as usize].lock().deref_mut() {
            let syn_queue = &mut entry.syn_queue;
            let (idx, addr_tuple) = syn_queue.iter()
            .enumerate()
            .find_map(|(idx, &handle)| {
                is_connected(handle).then(||(idx, get_addr_tuple(handle)))
            }).ok_or(SysError::EAGAIN)?;     
            
            let handle = syn_queue.swap_remove_front(idx).unwrap();
            Ok((handle,addr_tuple))
        }else {
            log::warn!("[listen table] failed: not listen");
            Err(SysError::EINVAL)
        }
    }
    pub fn can_accept(&self,port: u16) -> bool {
        if let Some(entry) = self.inner[port as usize].lock().deref(){
            entry.syn_queue.iter().any(|&handle| is_connected(handle))
        }else{
            log::info!("have been set as listening, wouldn't happen");
            false
        }    
    }
    pub fn handle_coming_tcp(&self, src: IpEndpoint, dst: IpEndpoint, sockets: &mut SocketSet<'_>) {
        if let Some(entry) = self.inner[dst.port as usize].lock().deref_mut() {
            if !entry.can_accept(dst.addr) {
                log::warn!("[LISTEN_TABLE] not listening on addr {}", dst.addr);
                return;;
            }
            if entry.syn_queue.len() >= LISTEN_QUEUE_SIZE {
                log::warn!("[LISTEN_TABLE] syn_queue overflow!");
                return;
            }
            entry.waker.wake_by_ref();
            log::info!(
                "[ListenTable::incoming_tcp_packet] wake the socket who listens port {}",
                dst.port
            );
            let mut socket = SocketSetWrapper::new_tcp_socket();
            if socket.listen(entry.listen_endpoint).is_ok() {
                let handle = sockets.add(socket);
                log::info!("TCP socket {}: prepare for connection {} -> {}", handle, src, entry.listen_endpoint);
                entry.syn_queue.push_back(handle);
            }
        }
    } 

}

fn is_connected(handle: SocketHandle) -> bool {
    SOCKET_SET.with_socket::<tcp::Socket,_,_>(handle, |socket| {
        !matches!(socket.state(), State::Listen | State::SynReceived)
    })
}

fn get_addr_tuple(handle: SocketHandle) -> (IpEndpoint, IpEndpoint) {
    SOCKET_SET.with_socket::<tcp::Socket, _, _>(handle, |socket| {
        (
            socket.local_endpoint().unwrap(),
            socket.remote_endpoint().unwrap(),
        )
    })
}