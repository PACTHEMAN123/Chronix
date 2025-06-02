use core::{sync::atomic::AtomicBool, time};

use alloc::vec::Vec;
use fatfs::{info, warn};
use lwext4_rust::bindings::EEXIST;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use smoltcp::{iface::SocketHandle, socket::{dns::GetQueryResultError, udp::{BindError, SendError}}, wire::{IpEndpoint, IpListenEndpoint}};
use spin::{RwLock, Spin};

use crate::{net::{LISTEN_TABLE, PORT_END, PORT_START, SOCK_RAND_SEED}, sync::mutex::SpinNoIrqLock, syscall::{SysError, SysResult}, task::current_task, utils::{get_waker, suspend_now, yield_now}};

use super::{addr::{is_unspecified, to_endpoint, SockAddr, UNSPECIFIED_LISTEN_ENDPOINT}, socket::{PollState, SockResult}, SocketSetWrapper, PORT_MANAGER, SOCKET_SET};

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
        log::info!(
            "[Udpsocket::bind] handle {} bound on {local_endpoint}",
            self.handle
        );
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
        // log::info!("in send to");
        if remote_endpoint.port == 0 || remote_endpoint.addr.is_unspecified() {
            log::warn!("socket send_to() failed: invalid remote address");
            return Err(SysError::EINVAL);
        }
        if self.local_endpoint.read().is_none() {
            log::warn!(
                "[send_impl] UDP socket {}: not bound. Use 127.0.0.1",
                self.handle
            );
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
                            log::warn!("socket send() failed, {e:?}");
                             socket.register_send_waker(&waker);
                             SysError::EAGAIN
                         }
                         SendError::Unaddressable => {
                             log::warn!("socket send() failed, {e:?}");
                             SysError::ECONNREFUSED
                         }
                     })?;
                    Ok(data.len())
                }else {
                    log::info!(
                        "[UdpSocket::send_to] handle{} can't send now, tx buffer is full",
                        self.handle
                    );
                    socket.register_send_waker(&waker);
                    Err(SysError::EAGAIN)
                }
            })
        }).await?;
        // log::info!("[UdpSocket::send_impl] send {bytes}bytes to {remote_endpoint:?}");
        yield_now().await;
        return Ok(bytes);
    }

    pub async fn recv(&self, data: &mut [u8]) -> SockResult<(usize, IpEndpoint)> {
        if self.local_endpoint.read().is_none() {
            log::warn!("socket send failed: not bound");
            return Err(SysError::ENOTCONN);
        }
        let waker = get_waker().await;
        let ret = self.block_on(||{
            SOCKET_SET.with_socket_mut::<smoltcp::socket::udp::Socket,_,_>(self.handle, |socket|{
                if socket.can_recv() {
                    match socket.recv_slice(data) {
                        Ok((len,meta)) => {
                            Ok((len, meta.endpoint))
                        },
                        Err(e) => {
                            log::warn!("[UdpSocket::recv] socket {} recv_slice error: {}",self.handle, e);
                            return Err(SysError::EAGAIN);
                        }
                    }
                }else if !socket.is_open() {
                    log::warn!("UdpSocket {}: recv() failed, not connected", self.handle);
                    return Err(SysError::ENOTCONN);
                }else {
                    log::info!("[recv_impl] {} no more data, register waker and suspend now", self.handle);
                    socket.register_recv_waker(&waker);
                    return Err(SysError::EAGAIN);
                } 
            })
        }).await;    
        yield_now().await;
        ret   
    }
    pub fn shutdown(&self) -> SockResult<()> {
        SOCKET_SET.with_socket_mut::<smoltcp::socket::udp::Socket,_,_>(self.handle, |socket| {
            socket.close();
        });
        let timestamp = SOCKET_SET.poll_interfaces();
        SOCKET_SET.check_poll(timestamp);
        Ok(())
    }
    pub async fn poll(&self) -> PollState {
        if self.local_endpoint.read().is_none() {
            return PollState{
                readable: false,
                writable: false,
                hangup: false,
            };
        }
        let waker = get_waker().await;
        SOCKET_SET.with_socket_mut::<smoltcp::socket::udp::Socket, _, _>(self.handle, |socket|{
            let readable = socket.can_recv();
            let writable = socket.can_send();
            if !readable {
                log::info!("[UdpSocket::poll] handle{} can't recv now, rx buffer is empty", self.handle);
                socket.register_recv_waker(&waker);
            }
            if !writable {
                log::info!("[UdpSocket::poll] handle{} can't send now, tx buffer is full", self.handle);
                socket.register_send_waker(&waker);
            }
            PollState {
                readable,
                writable,
                hangup: false,
            }
        })
    }
}


impl UdpSocket {
    fn get_ephemeral_port(&self) -> SockResult<u16> {
        const PORT_START: u16 = 0xc000;
        const PORT_END: u16 = 0xffff;
        static CURR: SpinNoIrqLock<u16> = SpinNoIrqLock::new(PORT_START);
        let mut curr = CURR.lock();

        let port = *curr;
        if *curr == PORT_END {
            *curr = PORT_START;
        } else {
            *curr += 1;
        }
        Ok(port)
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
                        log::info!("[UdpSocket::block_on] handle, EAGAIN, suspend now");
                        suspend_now().await;
                        let task = current_task().unwrap();
                        let has_signal_flag = task.with_sig_manager(|sig_manager| {
                            let block_sig = sig_manager.blocked_sigs;
                            sig_manager.check_pending_flag(!block_sig)
                        });
                        if has_signal_flag {
                            log::warn!("[block_on] has signal flag, return EINTR");
                            return Err(SysError::EINTR);
                        }
                    }
                    Err(e) => return Err(e),
                }
            }
        }
    }
}

impl Drop for UdpSocket {
    fn drop(&mut self) {
        log::info!("[UdpSocket::drop] handle {} dropped", self.handle);
        self.shutdown().ok();
        SOCKET_SET.remove(self.handle);
        if let Ok(addr) = self.local_addr() {
            PORT_MANAGER.remove(addr.port);
        }
    }
}