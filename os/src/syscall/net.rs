use core::{any::Any, clone, f32::consts::E, mem, net::Ipv4Addr, option, panic, ptr::{self, copy_nonoverlapping}};

use alloc::{ffi::CString, string::String, sync::Arc, task, vec,vec::Vec};
use fatfs::{info, warn};
use hal::{addr, instruction::{Instruction, InstructionHal}, println};
use lwext4_rust::bindings::EXT4_SUPERBLOCK_FLAGS_TEST_FILESYS;
use smoltcp::{socket::dns::Socket, time::Duration, wire::{IpAddress, Ipv4Address}};

use crate::{config::PAGE_SIZE, fs::{pipefs, vfs::file::open_file, OpenFlags}, mm::{UserPtr, UserPtrRaw, UserSliceRaw}, net::{addr::{SockAddr, SockAddrIn4, SockAddrIn6, SockAddrUn}, socket::{self, Sock, SockResult}, tcp::TcpSocket, SaFamily, SOCKET_SET}, signal::SigSet, task::{current_task, fs::{FdFlags, FdInfo}, task::TaskControlBlock}, timer::ffi::TimeSpec, utils::yield_now};

use super::{IoVec, SysError, SysResult};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// Socket types
pub enum SocketType {
    /// TCP
    STREAM = 1,
    /// UDP
    DGRAM = 2,
    /// Raw IP
    RAW = 3,
    /// RDM
    RDM = 4,
    /// Seq Packet
    SEQPACKET = 5,
    /// DCCP
    DCCP = 6,
    /// Packet
    PACKET = 10,
}

impl TryFrom<i32> for SocketType {
    type Error = SysError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::STREAM),
            2 => Ok(Self::DGRAM),
            // 3 => Ok(Self::RAW),
            4 => Ok(Self::RDM),
            5 => Ok(Self::SEQPACKET),
            6 => Ok(Self::DCCP),
            10 => Ok(Self::PACKET),
            _ => Err(Self::Error::EINVAL),
        }
    }
}

/// Set O_NONBLOCK flag on the open fd
pub const SOCK_NONBLOCK: i32 = 0x800;
/// Set FD_CLOEXEC flag on the new fd
pub const SOCK_CLOEXEC: i32 = 0x80000;

/// create an endpoint for communication and returns a file decriptor refers to the endpoint
/// Since Linux 2.6.27, the type argument serves a second purpose: in
///addition to specifying a socket type, it may include the bitwise
///OR of any of the following values, to modify the behavior of
///socket():
// SOCK_NONBLOCK
//        Set the O_NONBLOCK file status flag on the open file
//        description (see open(2)) referred to by the new file
//        descriptor.  Using this flag saves extra calls to fcntl(2)
//        to achieve the same result.

// SOCK_CLOEXEC
//        Set the close-on-exec (FD_CLOEXEC) flag on the new file
//        descriptor.  See the description of the O_CLOEXEC flag in
//        open(2) for reasons why this may be useful.
pub fn sys_socket(domain: usize, types: i32, _protocol: usize) -> SysResult {
    if domain <= 0 || domain > 255 {
        return Err(SysError::EAFNOSUPPORT);
    }
    log::info!("[sys_socket] domain: {:?}, types: {:?}, protocol: {:?}", domain, types, _protocol);
    let domain = SaFamily::try_from(domain as u16)?;
    let mut types = types as i32;
    let mut nonblock = false;
    // file descriptor flags
    let mut flags = OpenFlags::empty();
    if types & SOCK_NONBLOCK != 0 {
        nonblock = true;
        types &= !SOCK_NONBLOCK;
        flags |= OpenFlags::O_NONBLOCK;
    } 
    if types & SOCK_CLOEXEC != 0 {
        types &= !SOCK_CLOEXEC;
        flags |= OpenFlags::O_CLOEXEC;
    }

    let types = SocketType::try_from(types as i32)?;
    if types != SocketType::STREAM  && types != SocketType::DGRAM {
        //todo: temp meausure for protocol check
        return Err(SysError::EPROTONOSUPPORT);
    }
    let socket = socket::Socket::new(domain,types, nonblock);
    let fd_info = FdInfo {
        file: Arc::new(socket),
        flags: flags.into(),
    };
    let task = current_task().unwrap();
    let fd = task.with_mut_fd_table(|t|t.alloc_fd())?;
    task.with_mut_fd_table(|t| {
        t.put_file(fd, fd_info).or_else(|e|Err(e))
    })?;
    log::info!("[sys_socket]socket types:{:?}, fd: {}", types,fd);
    Ok(fd as isize)
}
/// “assigning a name to a socket”
pub fn sys_bind(fd: usize, addr: usize, addr_len: usize) -> SysResult {
    log::info!("[sys_bind] fd: {}, addr: {:#?}, addr_len: {}", fd, addr, addr_len);
    if (fd as isize) < 0 {
        return Err(SysError::EBADF);
    }
    let task = current_task().unwrap();
    let local_addr = sockaddr_reader(addr, addr_len, task)?;
    log::info!("[sys_bind] local_addr's port is: {}",unsafe {
        local_addr.ipv4
    });
    let socket_file = task.with_fd_table(|table| {
        table.get_file(fd)})?
        .downcast_arc::<socket::Socket>()
        .map_err(|_| SysError::ENOTSOCK)?;
    log::info!("[sys_bind] socket_file_type {:#?}, fd_type {:#?}", socket_file.sk_type, fd);
    socket_file.sk.bind(fd, local_addr)?;
    Ok(0)
}

/// Mark the stream socket referenced by the file descriptor `sockfd` as
/// passive. This socket will be used later to accept connections from other
/// (active) sockets
pub fn sys_listen(fd: usize, _backlog: usize) -> SysResult {
    if (fd as isize) < 0 {
        return Err(SysError::EBADF);
    }
    let current_task = current_task().unwrap();
    let socket_file = current_task.with_fd_table(|table| {
        table.get_file(fd)})?
        .downcast_arc::<socket::Socket>()
        .map_err(|_| SysError::ENOTSOCK)?;
    socket_file.sk.listen()?;
    Ok(0)
}

/// Connect the active socket refrenced by the file descriptor `sockfd` to the
/// address specified by `addr`. The `addr` argument is a pointer to a
/// `sockaddr` structure that contains the address of the remote socket.
/// The `addrlen` argument specifies the size of this structure.
pub async fn sys_connect(fd: usize, addr: usize, addr_len: usize) -> SysResult {
    if (fd as isize) < 0 {
        return Err(SysError::EBADF);
    }
    let task = current_task().unwrap().clone();
    let remote_addr = sockaddr_reader(addr, addr_len, &task)?;
    log::info!("[sys_connect] remote_addr is: {}",
        unsafe {
            remote_addr.ipv4
    });
    let socket_file = task.with_fd_table(|table| {
        table.get_file(fd)})?
        .downcast_arc::<socket::Socket>()
        .map_err(|_| SysError::ENOTSOCK)?;
    log::info!("[sys_connect] socket_file_type {:#?}", socket_file.sk_type);
    socket_file.sk.connect(remote_addr.into_endpoint()?).await?;
    // yield_now().await;
    Ok(0)
}

/// Accept a connection on the socket `sockfd` that is ready to be accepted.
/// The `addr` argument is a pointer to a `sockaddr` structure that will
/// hold the address of the peer socket, and `addrlen` is a pointer to
/// an integer that will hold the size of this structure.
///
/// The `sockfd` argument is a socket that has been created with the
/// `SOCK_STREAM` type, has been bound to a local address with `bind`,
/// and is listening for connections after a `listen` system call.
///
/// The `accept` system call is used on a socket that is listening for
/// incoming connections. It extracts the first connection request on
/// the queue of pending connections, creates a new socket for the
/// connection, and returns a new file descriptor referring to that
/// socket. The newly created socket is usually in the `ESTABLISHED`

pub async fn sys_accept(fd: usize, addr: usize, addr_len: usize) -> SysResult {
    if (fd as isize) < 0 {
        return Err(SysError::EBADF);
    }
    let task = current_task().unwrap();
    let socket_file = task.with_fd_table(|table| {
        table.get_file(fd)})?
        .downcast_arc::<socket::Socket>()
        .map_err(|_| SysError::ENOTSOCK)?;
    // moniter accept, allow sig_kill and sig_stop to interrupt
    task.set_interruptable();
    let old_mask = task.sig_manager.lock().blocked_sigs;
    task.set_wake_up_sigs(!old_mask);
    let accept_sk = socket_file.sk.accept().await?;
    task.set_running();
    log::info!("get accept correct");
    let peer_addr_endpoint = accept_sk.peer_addr().unwrap();
    let peer_addr = SockAddr::from_endpoint(peer_addr_endpoint);
    // log::info!("Accept a connection from {:?}", peer_addr);
    // write to pointer
   sockaddr_writer(task,addr, addr_len, peer_addr)?;

    let accept_socket = Arc::new(socket::Socket::from_another(&socket_file, Sock::TCP(accept_sk)));
    let fd_info = FdInfo {
        file: accept_socket,
        flags: OpenFlags::empty().into(),
    };
    let new_fd = task.with_mut_fd_table(|t|t.alloc_fd())?;
    task.with_mut_fd_table(|t| {
        t.put_file(new_fd, fd_info)
    })?;
    Ok(new_fd as isize)
}

/// The system calls send(), sendto(), and sendmsg() are used to
/// transmit a message to another socket.
pub async fn sys_sendto(
    fd: usize,
    buf: usize,
    len: usize,
    _flags: usize,
    addr: usize,
    addr_len: usize,
)-> SysResult {
    if (fd as isize) < 0 {
        return Err(SysError::EBADF);
    }
    if (buf as i32) < 0 || (buf == 0 && len != 0) {
        return Err(SysError::EFAULT);
    }
    if len > 64 * 128 {
        return Err(SysError::EMSGSIZE);
    }
    log::info!("addr is {}, addr_len is {}", addr, addr_len);
    let task = current_task().unwrap().clone();
    // let buf_slice = unsafe {
    //     core::slice::from_raw_parts_mut(buf as *mut u8, len)
    // };
    let buf_slice = UserSliceRaw::new(buf as *mut u8, len)
        .ensure_read(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;
    let buf_slice = buf_slice.to_ref();
    let socket_file = task.with_fd_table(|table| {
        table.get_file(fd)})?
        .downcast_arc::<socket::Socket>()
        .map_err(|_| SysError::ENOTSOCK)?;
    task.set_interruptable();
    let bytes = match socket_file.sk_type {
        SocketType::DGRAM => {
            let remote_addr = if addr != 0 {  Some(sockaddr_reader(addr, addr_len, &task)?
            .into_endpoint()?)}else {
                None
            };
            socket_file.sk.send(&buf_slice, remote_addr).await?    
        }
        SocketType::STREAM => {
            if addr != 0 {
                return Err(SysError::EISCONN);
            }
            socket_file.sk.send(&buf_slice, None).await?
        },
        _ => todo!(),
    };
    task.set_running();
    Ok(bytes as isize)
}

/// The recvfrom() function shall receive a message from a connection-
/// mode or connectionless-mode socket. It is normally used with
/// connectionless-mode sockets because it permits the application to
/// retrieve the source address of received data.
pub async fn sys_recvfrom(
    sockfd: usize,
    buf: usize,
    len: usize,
    _flags: usize,
    addr: usize,
    addrlen: usize,
) -> SysResult {
    if (sockfd as isize) < 0 {
        return Err(SysError::EBADF);
    }
    log::info!("sys_recvfrom sockfd: {}, buf: {:#x}, len: {}, flags: {:#x}, addr: {:#x}, addrlen: {}", sockfd, buf, len, _flags, addr, addrlen);
    let task = current_task().unwrap().clone();
    let socket_file = task.with_fd_table(|table| {
        table.get_file(sockfd)})?
        .downcast_arc::<socket::Socket>()
        .map_err(|_| SysError::ENOTSOCK)?;
    let mut inner_vec = Vec::with_capacity(len);
    unsafe {
        inner_vec.set_len(len);
    }
    task.set_interruptable();
    let (bytes, remote_endpoint) = socket_file.sk.recv(&mut inner_vec).await?;
    // log::info!("first code recv:{} ",char::from(inner_vec[0]));
    // log::info!("recvfrom: bytes: {}, remote_endpoint: {:?}", bytes, remote_endpoint);
    let remote_addr = SockAddr::from_endpoint(remote_endpoint);
    task.set_running();
    sockaddr_writer(&task,addr, addrlen, remote_addr)?;
    // write to pointer
    // log::info!("now set running");
    // let buf_slice = unsafe {
    //     core::slice::from_raw_parts_mut(buf as *mut u8, bytes)
    // };
    let buf_slice = UserSliceRaw::new(buf as *mut u8, bytes)
        .ensure_write(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;
    let buf_slice = buf_slice.to_mut();
    buf_slice[..bytes].copy_from_slice(&inner_vec[..bytes]);
    // log::info!("buf_slice[0]: {}",char::from(buf_slice[0]));
    // // write to sockaddr_in
    // if addr == 0 {
    //     return Ok(bytes as isize);  
    // }
    
    
    // log::info!("now return bytes: {}",bytes);
    Ok(bytes as isize)
}
/// Returns the local address of the Socket corresponding to `sockfd`.
pub fn sys_getsockname(fd: usize, addr: usize, addr_len: usize) -> SysResult {
    if (fd as isize) < 0 {
        return Err(SysError::EBADF);
    }
    if addr == 0 {
        return Err(SysError::EFAULT);
    }
    log::info!("sys_getsockname fd: {}, addr: {:#x}, addr_len: {}", fd, addr, addr_len);
    let task = current_task().unwrap();
    let socket_file = task.with_fd_table(|table| {
        table.get_file(fd)})?
        .downcast_arc::<socket::Socket>()
        .map_err(|_| SysError::ENOTSOCK)?;
    let local_addr = socket_file.sk.local_addr()?;
    // log::info!("Get local address of socket: {:?}", local_addr);
    // write to pointer
    sockaddr_writer(&task, addr, addr_len, local_addr)?;
    Ok(0)
}

/// returns the peer address of the socket corresponding to the file descriptor `sockfd`
pub fn sys_getpeername(fd: usize, addr: usize, addr_len: usize) -> SysResult {
    if (fd as isize) < 0 {
        return Err(SysError::EBADF);
    }
    let task = current_task().unwrap();
    let socket_file = task.with_fd_table(|table| {
        table.get_file(fd)})?
        .downcast_arc::<socket::Socket>()
        .map_err(|_| SysError::ENOTSOCK)?;
    let peer_addr = socket_file.sk.peer_addr()?;
    log::info!("Get peer address of socket: {:?}", peer_addr);
    // write to pointer
    sockaddr_writer(task,addr, addr_len, peer_addr)?;
    Ok(0)
}
#[allow(missing_docs)]
#[repr(usize)]
pub enum SocketLevel {
    /// Dummy protocol for TCP
    IpprotoIp = 0,
    /// S
    SolSocket = 1,
    IpprotoTcp = 6,
    /// IPv6-in-IPv4 tunnelling
    IpprotoIpv6 = 41,
}

///为每个level建立一个配置enum
#[repr(usize)]
#[allow(non_camel_case_types)]
pub enum IpOption {
    //设置多播数据的发送出口网络接口,设置多播接口中从哪个接口发送对应数据包
    IP_MULTICAST_IF = 32,
    //设置多播数据包的生存时间（TTL），控制其传播范围
    IP_MULTICAST_TTL = 33,
    ///控制多播数据的本地环回
    /// 启用（1）：发送的多播数据会被同一主机上的接收套接字收到。
    /// 禁用（0）：发送的数据不环回，仅其他主机接收。
    IP_MULTICAST_LOOP = 34,
    ///加入一个多播组，开始接收发送到该组地址的数据
    IP_ADD_MEMBERSHIP = 35,
    IP_PKTINFO = 11,
    MCAST_JOIN_GROUP = 42,
    MCAST_LEAVE_GROUP = 45,
}

impl TryFrom<usize> for IpOption {
    type Error = SysError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            32 => Ok(IpOption::IP_MULTICAST_IF),
            33 => Ok(IpOption::IP_MULTICAST_TTL),
            34 => Ok(IpOption::IP_MULTICAST_LOOP),
            35 => Ok(IpOption::IP_ADD_MEMBERSHIP),
            11 => Ok(IpOption::IP_PKTINFO),
            42 => Ok(IpOption::MCAST_JOIN_GROUP),
            45 => Ok(IpOption::MCAST_LEAVE_GROUP),
            _ => Err(SysError::EINVAL),
        }
    }
}

impl IpOption {
    pub fn set(&self, socket: &crate::net::socket::Socket, opt: &[u8]) -> SockResult<isize> {
        match self {
            IpOption::IP_MULTICAST_IF | IpOption::MCAST_JOIN_GROUP => {
                Ok(0)
            }
            IpOption::MCAST_LEAVE_GROUP => {
                return Err(SysError::EADDRNOTAVAIL);
            }
            IpOption::IP_MULTICAST_TTL => {
                match &socket.sk {
                    Sock::UDP(udp) => {
                        let ttl = u8::from_be_bytes(<[u8; 1]>::try_from(&opt[0..1]).unwrap());
                        udp.set_ttl(ttl);
                        Ok(0)
                    }
                    _ => Err(SysError::EINVAL)
                }
            }
            IpOption::IP_MULTICAST_LOOP => Ok(0),
            IpOption::IP_ADD_MEMBERSHIP => {
                // let multicast_addr = IpAddress::Ipv4(Ipv4Address::new(opt[0], opt[1], opt[2], opt[3]));
                // let interface_addr = IpAddress::Ipv4(Ipv4Address::new(opt[4], opt[5], opt[6], opt[7]));
                // todo: add_membership
                Ok(0)
            }
            _ => {Ok(0)}
        }
    }
}
impl TryFrom<usize> for SocketLevel {
    type Error = SysError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SocketLevel::IpprotoIp),
            1 => Ok(SocketLevel::SolSocket),
            6 => Ok(SocketLevel::IpprotoTcp),
            41 => Ok(SocketLevel::IpprotoIpv6),
            _ => Err(SysError::EINVAL),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[allow(missing_docs)]
pub enum SocketOption {
    DEBUG = 1,
    REUSEADDR = 2,
    TYPE = 3,
    ERROR = 4,
    DONTROUTE = 5,
    BROADCAST = 6,
    SNDBUF = 7,
    RCVBUF = 8,
    KEEPALIVE = 9,
    OOBINLINE = 10,
    NoCheck = 11,
    PRIORITY = 12,
    LINGER = 13,
    BSDCOMPAT = 14,
    REUSEPORT = 15,
    PASSCRED = 16,
    PEERCRED = 17,
    RCVLOWAT = 18,
    SNDLOWAT = 19,
    RcvtimeoOld = 20,
    SndtimeoOld = 21,
    SecurityAuthentication = 22,
    SecurityEncryptionTransport = 23,
    SecurityEncryptionNetwork = 24,
    BINDTODEVICE = 25,
    AttachFilter = 26,
    DetachFilter = 27,
    SNDBUFFORCE = 32,
    RCVBUFFORCE = 33,
}

impl TryFrom<usize> for SocketOption {
    type Error = SysError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(SocketOption::DEBUG),
            2 => Ok(SocketOption::REUSEADDR),
            3 => Ok(SocketOption::TYPE),
            4 => Ok(SocketOption::ERROR),
            5 => Ok(SocketOption::DONTROUTE),
            6 => Ok(SocketOption::BROADCAST),
            7 => Ok(SocketOption::SNDBUF),
            8 => Ok(SocketOption::RCVBUF),
            9 => Ok(SocketOption::KEEPALIVE),
            10 => Ok(SocketOption::OOBINLINE),
            11 => Ok(SocketOption::NoCheck),
            12 => Ok(SocketOption::PRIORITY),
            13 => Ok(SocketOption::LINGER),
            14 => Ok(SocketOption::BSDCOMPAT),
            15 => Ok(SocketOption::REUSEPORT),
            16 => Ok(SocketOption::PASSCRED),
            17 => Ok(SocketOption::PEERCRED),
            18 => Ok(SocketOption::RCVLOWAT),
            19 => Ok(SocketOption::SNDLOWAT),  
            20 => Ok(Self::RcvtimeoOld),
            21 => Ok(Self::SndtimeoOld),
            22 => Ok(Self::SecurityAuthentication),
            23 => Ok(Self::SecurityEncryptionTransport),
            24 => Ok(Self::SecurityEncryptionNetwork),
            25 => Ok(Self::BINDTODEVICE),
            26 => Ok(Self::AttachFilter),
            27 => Ok(Self::DetachFilter),
            32 => Ok(Self::SNDBUFFORCE),
            33 => Ok(Self::RCVBUFFORCE), 
            opt => {
                log::warn!("[SocketOpt] unsupported option: {opt}");
                Ok(Self::DEBUG)
                // Err(Self::Error::EINVAL)
            }
        }
    }
}

#[derive(Debug,PartialEq,Eq,Clone,Copy)]
#[allow(missing_docs)]
pub enum TcpSocketOption {
    NODELAY = 1, // disable nagle algorithm and flush
    MAXSEG = 2,
    INFO = 11,
    CONGESTION = 13,
}

impl SocketOption {
    pub fn set(&self, socket: &crate::net::socket::Socket, opt: &[u8]) -> SockResult<isize> {
        match self {
            SocketOption::REUSEADDR | SocketOption:: DONTROUTE => {
                if opt.len() < 4 {
                    return Err(SysError::EINVAL)
                }
                let addr = i32::from_ne_bytes(<[u8; 4]>::try_from(&opt[0..4]).unwrap());
                // set reuse addr option
                socket.sk.set_reuse_addr(addr != 0);
                Ok(0)
            }

            SocketOption::SNDBUF => {
                if opt.len() < 4 {
                    return Err(SysError::EINVAL);
                }
                let len = i32::from_ne_bytes(<[u8; 4]>::try_from(&opt[0..4]).unwrap());
                socket.set_send_buf_size(len as usize);
                Ok(0)
            }

            SocketOption::RCVBUF => {
                if opt.len() < 4 {
                    return Err(SysError::EINVAL);
                }
                let len = i32::from_ne_bytes(<[u8; 4]>::try_from(&opt[0..4]).unwrap());
                socket.set_recv_buf_size(len as usize);
                Ok(0)
            }

            SocketOption::KEEPALIVE => {
                if opt.len() < 4 {
                    return Err(SysError::EINVAL);
                }
                let len_opt = u32::from_ne_bytes(<[u8; 4]>::try_from(&opt[0..4]).unwrap());
                let expire = if len_opt != 0 {
                    Some(Duration::from_secs(60 as u64))
                }else {
                    None
                };
                match &socket.sk {
                    Sock::TCP(tcp) => {
                        if let Some(handle) = tcp.handle(){
                            SOCKET_SET.with_socket_mut::<smoltcp::socket::tcp::Socket,_,_>(handle, |socket| {
                                socket.set_keep_alive(expire);
                            });
                        }
                    }
                    _ =>{}
                }
                socket.set_recv_buf_size(len_opt as usize);
                Ok(0)
            }

            SocketOption::RcvtimeoOld => {
                if opt.len() != size_of::<TimeSpec>() {
                    return Err(SysError::EINVAL);
                }
                let timeout = unsafe { *(opt.as_ptr() as *const TimeSpec) };
                match &socket.sk {
                    Sock::TCP(tcp) => {
                        tcp.set_timeout(if !timeout.is_valid() || timeout.tv_nsec == 0 && timeout.tv_sec == 0 {
                            None
                        }else {
                            Some(timeout)
                        });
                    }
                    _ => {}
                }
                Ok(0)
            }

            SocketOption::SNDBUFFORCE =>{
                let raw = u32::from_ne_bytes(opt[0..4].try_into().unwrap());
                let val_i32 = if raw > i32::MAX as u32 {
                    i32::MAX
                } else {
                    raw as i32
                };
                socket.set_send_buf_size(val_i32 as usize);
                Ok(0)
            },

            _ => {
                Ok(0)
            }
        }
    }

    /// get socket option value
    pub fn get(&self, socket: &crate::net::socket::Socket, opt: &mut [u8],  opt_len: &mut u32) -> SockResult<()>{
        let buf_len = *opt_len  as usize;
        match self {
            SocketOption::REUSEADDR => {
                let value: i32 = if socket.sk.get_reuse_addr_flag() {1}else {0};
                let value = &value.to_ne_bytes();
                if buf_len < 4 {
                    return Err(SysError::EINVAL)
                } 
                let len = value.len();
                opt[..len].copy_from_slice(value);
                *opt_len = 4;
                Ok(())
            }

            SocketOption::DONTROUTE => {
                if buf_len < 4 {
                    return Err(SysError::EINVAL)
                }
                let value: i32 = if socket.dont_route {1}else {0};
                let value = &value.to_ne_bytes();
                let len = value.len();
                opt[..len].copy_from_slice(value);
                *opt_len = 4;
                Ok(())
            }

            SocketOption::SNDBUF => {
                let size = socket.get_send_buf_size().to_ne_bytes();
                let len = size.len();
                opt[..len].copy_from_slice(&size);
                *opt_len = 4;
                Ok(())
            }

            SocketOption::RCVBUF => {
                let size = socket.get_recv_buf_size().to_ne_bytes();
                let len = size.len();
                opt[..len].copy_from_slice(&size);
                *opt_len = 4;
                Ok(())
            }

            SocketOption::KEEPALIVE => {
                if buf_len < 4 {
                    return Err(SysError::EINVAL)
                }

                match &socket.sk {
                    Sock::TCP(tcp) => {
                        if let Some(handle) = tcp.handle(){
                            SOCKET_SET.with_socket::<smoltcp::socket::tcp::Socket,_,_>(handle, |socket| {
                                let value: i32 = if socket.keep_alive().is_none() {1} else {0};
                                let value = &value.to_ne_bytes();
                                let len = value.len();
                                opt[..len].copy_from_slice(value);
                                *opt_len = 4;
                            });
                        }
                    }
                    _ => {}
                }
                Ok(())
            }

            SocketOption::RcvtimeoOld => {
                if buf_len < size_of::<TimeSpec>() {
                    return Err(SysError::EINVAL);
                }
                match &socket.sk {
                    Sock::TCP(tcp) => {
                        match tcp.get_timeout() {
                            Some(timeout) => {
                                let time_u8 = TimeSpec::_as_bytes(&timeout);
                                let len = time_u8.len();
                                opt[..len].copy_from_slice(time_u8);
                                *opt_len = size_of::<TimeSpec>() as u32;
                            }
                            None => {
                                let data = &0u8.to_ne_bytes();
                                let len = data.len();
                                opt[..len].copy_from_slice(data);
                                *opt_len = size_of::<TimeSpec>() as u32;
                            }
                        }
                    }
                    _ =>{}
                }
                Ok(())
            }

            _ => {
                Ok(())
            }
        }
    }
}
impl TryFrom<usize> for TcpSocketOption {
    type Error = SysError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(TcpSocketOption::NODELAY),
            2 => Ok(TcpSocketOption::MAXSEG),
            11 => Ok(TcpSocketOption::INFO),
            13 => Ok(TcpSocketOption::CONGESTION),
            opt => {
                log::warn!("[TcpSocketOpt] unsupported option: {opt}");
                Err(Self::Error::EINVAL)
            }
        }
    }
}

impl TcpSocketOption {
    pub fn set(&self, raw_socket: &crate::net::socket::Socket, opt: &[u8]) -> SockResult<isize> {
        match &raw_socket.sk {
            Sock::TCP(tcp) => {
                if let Some(handle) = tcp.handle(){
                    let res: Result<isize, SysError> = SOCKET_SET.with_socket_mut::<smoltcp::socket::tcp::Socket,_,_>(handle, |socket| {
                        match self {
                            TcpSocketOption::NODELAY => {
                                if opt.len() < 4 {
                                    return Err(SysError::EINVAL);
                                }
                                let opt_value = u32::from_be_bytes(<[u8; 4]>::try_from(&opt[0..4]).unwrap());
                                socket.set_nagle_enabled(opt_value == 0);
                                return Ok(0);
                            }
                            TcpSocketOption::MAXSEG => todo!(),
                            TcpSocketOption::INFO => {return Ok(0);},
                            TcpSocketOption::CONGESTION => {
                                raw_socket.set_congestion(String::from_utf8(Vec::from(opt)).unwrap());
                                return Ok(0);
                            },
                        }
                    });
                    return res;
                }else {
                    Ok(0)
                }
                
            }
            _ => {Ok(0)},
        }
    }

    pub fn get(&self, rawsocket: &crate::net::socket::Socket, opt_addr: &mut [u8], opt_len:&mut u32) -> SockResult<()> {
        let buf_len = unsafe { *opt_len };
        match &rawsocket.sk {
            Sock::TCP(tcp) => {
                if let Some(handle) = tcp.handle(){
                    let res: Result<(), SysError> = SOCKET_SET.with_socket_mut::<smoltcp::socket::tcp::Socket,_,_>(handle, |socket| {
                        match self {
                            TcpSocketOption::NODELAY => {
                                if buf_len < 4 {
                                    return Err(SysError::EINVAL);
                                }
                                let value: i32 = if socket.nagle_enabled() {0} else {1};
                                let value = value.to_ne_bytes();
                                let index = value.len();
                                opt_addr[..index].copy_from_slice(&value);
                                *opt_len = 4;
                                Ok(())
                            }
                            TcpSocketOption::MAXSEG => {
                                let len = size_of::<usize>();
                                let value: usize = 1500;
                                let value = value.to_ne_bytes();
                                let index = value.len();
                                opt_addr[..index].copy_from_slice(&value);
                                *opt_len = len as u32;
                                Ok(())
                            },
                            TcpSocketOption::INFO => {Ok(())},
                            TcpSocketOption::CONGESTION => {
                                let len = rawsocket.get_congestion().as_bytes().len() as u32;
                                opt_addr.copy_from_slice(rawsocket.get_congestion().as_bytes());
                                *opt_len = len as u32;
                                Ok(())
                            }
                            _=> {
                                Ok(())
                            }
                        }
                    });
                    res
                }else {
                    Ok(())
                }
            }
            _ => Ok(())
        }
    }
}

#[repr(usize)]
#[allow(non_camel_case_types)]
pub enum Ipv6Option {
    UNICAST_HOPS = 4,
    MULTICAST_IF = 9,
    MULTICAST_HOPS = 10,
    //fake
    IPV6_DEV = 26,
    IPV6_ONLY = 27,
    PACKET_INFO = 61,
    RECV_TRAFFIC_CLASS = 66,
    TRAFFIC_CLASS = 67,
}
impl TryFrom<usize> for Ipv6Option {
    type Error = SysError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            4 => Ok(Ipv6Option::UNICAST_HOPS),
            9 => Ok(Ipv6Option::MULTICAST_IF),
            10 => Ok(Ipv6Option::MULTICAST_HOPS),
            26 => Ok(Ipv6Option::IPV6_DEV),
            27 => Ok(Ipv6Option::IPV6_ONLY),
            61 => Ok(Ipv6Option::PACKET_INFO),
            66 => Ok(Ipv6Option::RECV_TRAFFIC_CLASS),
            67 => Ok(Ipv6Option::TRAFFIC_CLASS),
            _ => {
                log::warn!("[Ipv6Option] unsupported option: {value}");
                Err(Self::Error::EINVAL)
            }
        }
    }
}

impl Ipv6Option {
    pub fn set(&self, socket: &crate::net::socket::Socket, opt: &[u8]) -> SockResult<isize> {
        Ok(0)
    }
}

// ============================== 
/// socket configure interface for user
/// level: protocel level at which the option resides,
/// option name
pub fn sys_setsockopt  (
    fd: usize,
    level: usize,
    option_name: usize,
    option_value: usize,
    option_len: usize,
) -> SysResult {
    let Ok(level) = SocketLevel::try_from(level) else{
        return Err(SysError::ENOPROTOOPT);
    };
    let task = current_task().unwrap();
    let socket_file = task.with_fd_table(|table| {
        table.get_file(fd)})?
        .downcast_arc::<socket::Socket>()
        .map_err(|_| SysError::ENOTSOCK)?;

    let opt_val_r = UserSliceRaw::new(option_value as *const u8, option_len)
        .ensure_read(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;
    
    let mut kernel_opt: Vec<u8> = vec![0; option_len];
    kernel_opt.copy_from_slice(opt_val_r.to_ref());
    
    match level {
        SocketLevel::IpprotoIp => {
            let Ok(option) = IpOption::try_from(option_name) else {
                return Err(SysError::ENOPROTOOPT);
            };
            option.set(&socket_file, kernel_opt.as_slice())
        }
        SocketLevel::SolSocket => {
            let Ok(option) = SocketOption::try_from(option_name) else {
                return Err(SysError::ENOPROTOOPT);
            };
            option.set(&socket_file, kernel_opt.as_slice())
        }
        SocketLevel::IpprotoTcp => {
            let Ok(option) = TcpSocketOption::try_from(option_name) else{
                return Err(SysError::ENOPROTOOPT);
            };
            option.set(&socket_file, kernel_opt.as_slice())
        }
        SocketLevel::IpprotoIpv6 => {
            let Ok(option) = Ipv6Option::try_from(option_name) else {
                return Err(SysError::ENOPROTOOPT);
            };
            option.set(&socket_file, &kernel_opt.as_slice())
        }
        _ => {
            Ok(0)
        }
    }

}
/// get socket configure interface for user
pub fn sys_getsockopt (
    _fd: usize,
    level: usize,
    option_name: usize,
    option_value: usize,
    option_len: usize,
) -> SysResult {
    fn write_string_to_ptr(mut optval_ptr: *mut u8, str:&str) {
        let c_str = CString::new(str).expect("CString::new failed");
        let bytes = c_str.as_bytes();
        for byte in bytes {
            unsafe {
                optval_ptr.write(*byte);
                optval_ptr = optval_ptr.offset(1);
            }
        }
        unsafe {
            optval_ptr.write(0);
        }
    }
    let task = current_task().unwrap();
    let optlen_ptr = UserPtrRaw::new(option_len as *mut u32)
    .ensure_write(&mut task.get_vm_space().lock())    
    .ok_or(SysError::EFAULT)?;
    match SocketLevel::try_from(level)? {
        SocketLevel::SolSocket => {
            const SEND_BUFFER_SIZE: usize = 64 * 1024; // 64KB
            const RECV_BUFFER_SIZE: usize = 64 * 1024; // 64KB
            match SocketOption::try_from(option_name)?{
                SocketOption::SNDBUF => {
                    let optval_ptr = UserPtrRaw::new(option_value as *mut u32)
                        .ensure_write(&mut task.get_vm_space().lock())
                        .ok_or(SysError::EFAULT)?;
                    optval_ptr.write(SEND_BUFFER_SIZE as u32);
                    optlen_ptr.write(size_of::<u32>() as u32);
                },
                SocketOption::RCVBUF => {
                    let optval_ptr = UserPtrRaw::new(option_value as *mut u32)
                        .ensure_write(&mut task.get_vm_space().lock())
                        .ok_or(SysError::EFAULT)?;
                    optval_ptr.write(RECV_BUFFER_SIZE as u32);
                    optlen_ptr.write(size_of::<u32>() as u32);
                },
                SocketOption::ERROR => {
                    let optval_ptr = UserPtrRaw::new(option_value as *mut u32)
                        .ensure_write(&mut task.get_vm_space().lock())
                        .ok_or(SysError::EFAULT)?;
                    optval_ptr.write(0 as u32);
                    optlen_ptr.write(size_of::<u32>() as u32);
                }
                _ =>{
                    
                } 
            }
        },
        SocketLevel::IpprotoTcp | SocketLevel::IpprotoIp  => {
            const MAX_SEGMENT: usize = 1460; // 1460 byte susually MTU
            match TcpSocketOption::try_from(option_name)? {
                TcpSocketOption::NODELAY => {
                    let optval_ptr = UserPtrRaw::new(option_value as *mut u32)
                        .ensure_write(&mut task.get_vm_space().lock())
                        .ok_or(SysError::EFAULT)?;
                    optval_ptr.write(0 as u32);
                    optlen_ptr.write(size_of::<u32>() as u32);
                    
                },
                TcpSocketOption::MAXSEG => {
                    let optval_ptr = UserPtrRaw::new(option_value as *mut u32)
                        .ensure_write(&mut task.get_vm_space().lock())
                        .ok_or(SysError::EFAULT)?;
                        optval_ptr.write(MAX_SEGMENT as u32);
                        optlen_ptr.write(size_of::<u32>() as u32);
                },
                TcpSocketOption::INFO => {},
                TcpSocketOption::CONGESTION => {
                    log::warn!("[sys_getsockopt], TcpSocketOption::CONGESTION");
                        let optval_ptr = UserPtrRaw::new(option_value as *mut u8)
                        .ensure_write(&mut task.get_vm_space().lock())
                        .ok_or(SysError::EFAULT)?;
                        let str = "reno";
                        let optval_ptr = optval_ptr.to_mut() as *mut u8;
                        write_string_to_ptr(optval_ptr, str);
                        optlen_ptr.write(4);
                },
            }
        },
        SocketLevel::IpprotoIpv6 => {},
    }
    Ok(0)
}

/// sys_shutdown() allows a greater control over the behaviour of connection-oriented sockets.
/// todo : how used for indicate read is shut down, write is shut down, or both 
pub fn sys_shutdown(fd: usize, how: usize) -> SysResult {
    if (fd as isize) < 0 {
        return Err(SysError::EBADF);
    }
    let task = current_task().unwrap();
    let socket_file = task.with_fd_table(|table| {
        table.get_file(fd)})?
        .downcast_arc::<socket::Socket>()
        .map_err(|_| SysError::ENOTSOCK)?;
    socket_file.sk.shutdown(how as u8)?;
    log::info!("shutdown: fd: {}, how: {}", fd, how);
    Ok(0)
}
/// create a pair of connected sockets
pub fn sys_socketpair(_domain: usize, _types: usize, _protocol: usize, sv: usize) -> SysResult {
    let task = current_task().unwrap();
    let (pipe_read, pipe_write) = pipefs::make_pipe(PAGE_SIZE);
    let pipe = task.with_mut_fd_table(|table| {
        let fd_read = table.alloc_fd()?;
        let fd_info_read = FdInfo {
            file: pipe_read,
            flags: FdFlags::empty(),
        };
        table.put_file(fd_read, fd_info_read)?;
        let fd_write = table.alloc_fd()?;
        let fd_info_write = FdInfo {
            file: pipe_write,
            flags: FdFlags::empty(),    
        };
        table.put_file(fd_write, fd_info_write)?;
        Ok([fd_read as u32, fd_write as u32])
    })?;
    let sv = UserPtrRaw::new(sv as *mut [u32;2])
    .ensure_write(&mut task.get_vm_space().lock())
    .ok_or(SysError::EFAULT)?;
    sv.write(pipe);
    Ok(0)
}

/// msghdr structure for recvmsg() and sendmsg() system calls
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MsgHdr {
    /// ptr points to peer address
    pub msg_name: usize,
    /// addr len
    pub msg_namelen: u32,
    /// iovecs ptr
    pub msg_iov: usize,
    /// iovecs len
    pub msg_iovlen: u32,
    /// ancillary data ptr
    pub msg_control: usize,
    /// ancillary data len
    pub msg_controllen: u32,
    /// flags
    pub msg_flags: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
/// accillary data object information for recvmsg() and sendmsg() system calls
pub struct CmsgHdr {
    /// level
    pub cmsg_level: u32,
    /// type
    pub cmsg_type: u32,
    /// data len
    pub cmsg_len: u32,
}
/// send a message through a connection-mode or connectionless-mode socket. 
pub async fn sys_sendmsg(
    fd: usize,
    msg: usize,
    flags: usize,
)-> SysResult {
    if (fd as isize) < 0 {
        return Err(SysError::EBADF);
    }
    let task = current_task().unwrap();
    if flags != 0 {
        log::warn!("unsupported flags: {}", flags);
    }
    let socket_file = task.with_fd_table(|table| {
        table.get_file(fd)})?
        .downcast_arc::<socket::Socket>()
        .map_err(|_| SysError::ENOTSOCK)?;
    // let msg_ptr = msg as *const MsgHdr;
    // let msg = unsafe { msg_ptr.read() };
    let msg = *UserPtrRaw::new(msg as *const MsgHdr)
        .ensure_read(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?
        .to_ref();
    if msg.msg_controllen != 0 {
        log::warn!("unsupported control data");
    }
    let addr = sockaddr_reader(msg.msg_name, msg.msg_namelen as usize, task)?
        .into_endpoint()?;
    // let addr = match SaFamily::try_from(unsafe {
    //     Instruction::set_sum();
    //     *(msg.msg_name as *const u16)
    // })? {
    //     SaFamily::AfInet => {
    //         if msg.msg_namelen < mem::size_of::<SockAddrIn4>() as u32 {
    //             log::error!("[sendmsg] invalid address length: {}", msg.msg_namelen);
    //             return Err(SysError::EINVAL);
    //         }
    //         Ok(SockAddr{
    //             ipv4: unsafe { *(msg.msg_name as *const SockAddrIn4) },
    //         }.into_endpoint())
    //     },
    //     SaFamily::AfInet6 => {
    //         if msg.msg_namelen < mem::size_of::<SockAddrIn6>() as u32 {
    //             log::error!("[sendmsg] invalid address length: {}", msg.msg_namelen);
    //             return Err(SysError::EINVAL);
    //         }
    //         Ok(SockAddr{
    //             ipv6: unsafe {
    //                 *(msg.msg_name as *const SockAddrIn6)
    //             }
    //         }.into_endpoint())
    //     },
    //     SaFamily::AfUnix => {
    //         if msg.msg_namelen < mem::size_of::<SockAddrUn>() as u32 {
    //             log::error!("[sendmsg] invalid address length: {}", msg.msg_namelen);
    //             return Err(SysError::EINVAL);
    //         }
    //         Ok(SockAddr{
    //             ipv6: unsafe {
    //                 *(msg.msg_name as *const _)
    //             }
    //         }.into_endpoint())
    //     },
    //     _ => todo!()
    // }?;
    // let iovs = unsafe {
    //     Instruction::set_sum();
    //     core::slice::from_raw_parts(msg.msg_iov as *const IoVec, msg.msg_iovlen as usize)
    // };
    let iovs_slice = UserSliceRaw::new(msg.msg_iov as *const IoVec, msg.msg_iovlen as usize)
        .ensure_read(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;
    let iovs = iovs_slice.to_ref();
    let mut total_len = 0;
    for (_i, iov) in iovs.iter().enumerate() {
        if iov.len == 0 {
            continue;
        }
        let ptr = iov.base as *const u8;
        // let buf_slice = unsafe {
        //     core::slice::from_raw_parts(ptr, iov.len as usize)
        // };
        let buf_slice = UserSliceRaw::new(ptr, iov.len as usize)
            .ensure_read(&mut task.get_vm_space().lock())
            .ok_or(SysError::EFAULT)?;
        let buf_slice = buf_slice.to_ref();
        let send_len = socket_file.sk.send(buf_slice, Some(addr)).await?;
        total_len += send_len;
    }
    Ok(total_len as isize)
}

/// receive a message from a connection-mode or connectionless-mode socket.
pub async fn sys_recvmsg(
    fd: usize,
    msg: usize,
    flags: usize,
) -> SysResult {
    log::warn!("[sys_recvmsg] into sys_recvmsg");
    if (fd as isize) < 0 {
        return Err(SysError::EBADF);
    }
    if flags != 0 {
        log::warn!("unsupported flags: {}", flags);
    }
    let task = current_task().unwrap();
    let socket_file = task.with_fd_table(|table| {
        table.get_file(fd)})?
        .downcast_arc::<socket::Socket>()
        .map_err(|_| SysError::ENOTSOCK)?;
    let inner_msg = *UserPtrRaw::new(msg as *const MsgHdr)
        .ensure_read(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?
        .to_ref();
    if inner_msg.msg_controllen != 0 {
        log::warn!("unsupported control data");
    }
    // let iovs = unsafe {
    //     Instruction::set_sum();
    //     core::slice::from_raw_parts(inner_msg.msg_iov as *const IoVec, inner_msg.msg_iovlen as usize)
    // };
    let iovs_slice = UserSliceRaw::new(inner_msg.msg_iov as *const IoVec, inner_msg.msg_iovlen as usize)
        .ensure_read(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;
    let iovs = iovs_slice.to_ref();
    let mut tmp_buf = vec![0u8; 64 * 1024];
    let (recv_len,src_addr) = socket_file.sk.recv(&mut tmp_buf).await?;
    let mut copied = 0;
    let data = tmp_buf[..recv_len].to_vec();
    for iov in iovs {
        if copied >= recv_len {
            break;
        }
        let to_copy = (iov.len as usize).min(recv_len - copied);
        let dst = iov.base as *mut u8;
        unsafe  {
            core::ptr::copy_nonoverlapping(data[copied..].as_ptr(), dst, to_copy)
        };
        copied += to_copy;
    }

    if inner_msg.msg_name != 0 {
        let addr = SockAddr::from_endpoint(src_addr);
        // unsafe {
        //     match SaFamily::try_from(addr.family)? {
        //         SaFamily::AfInet => {
        //             let addr_ptr = inner_msg.msg_name as *mut SockAddrIn4;
        //             addr_ptr.write_volatile(addr.ipv4);
        //             let addr_len_ptr = inner_msg.msg_namelen as *mut u32;
        //             addr_len_ptr.write_volatile(size_of::<SockAddrIn4>() as u32);
        //         },
        //         SaFamily::AfInet6 => {
        //             let addr_ptr = inner_msg.msg_name as *mut SockAddrIn6;
        //             addr_ptr.write_volatile(addr.ipv6);
        //             let addr_len_ptr = inner_msg.msg_namelen as *mut u32;
        //             addr_len_ptr.write_volatile(size_of::<SockAddrIn6>() as u32);
        //         },
        //         SaFamily::AfUnix => {
        //             let addr_ptr = inner_msg.msg_name as *mut SockAddrUn;
        //             addr_ptr.write_volatile(addr.unix);
        //             let addr_len_ptr = inner_msg.msg_namelen as *mut u32;
        //             addr_len_ptr.write_volatile(size_of::<SockAddrUn>() as u32);
        //         },
        //         _ => todo!()
        //     }
        // }
        sockaddr_writer(task, inner_msg.msg_name, inner_msg.msg_namelen as usize, addr)?;
    }      
    Ok(copied as isize)
}

pub fn sockaddr_reader(addr: usize, addr_len: usize, task: &Arc<TaskControlBlock>) -> Result<SockAddr, SysError> {
    let addr = *(UserPtrRaw::new(addr as *const SockAddr)
    .ensure_read(&mut task.get_vm_space().lock())
    .ok_or(SysError::EFAULT)?)
    .to_ref();
    let family = unsafe {
        SaFamily::try_from(addr.family)?
    };
    log::info!("[sockaddr_reader] family: {:?}, addr_len: {}", family, addr_len);
    match family {
        SaFamily::AfInet => {
            if addr_len < size_of::<SockAddrIn4>() {
                return Err(SysError::EINVAL);
            }
            return Ok(addr);
        }
        SaFamily::AfInet6 => {
            if addr_len < size_of::<SockAddrIn6>() {
                return Err(SysError::EINVAL);
            }
            Ok(addr)
        },
        SaFamily::AfUnix => {
            if addr_len < size_of::<SockAddrUn>() {
                log::info!("in this, size of SockAddrUn: {}",size_of::<SockAddrUn>());
                return Err(SysError::EINVAL);
            }
           Ok(addr)
        },
        _ => todo!()
    }
}

pub fn sockaddr_writer(task: &Arc<TaskControlBlock>, addr: usize, addr_len: usize, sock_addr: SockAddr) -> SockResult<()>{
    if addr == 0{
        return Ok(());
    }
    let addr =  UserPtrRaw::new(addr as *const SockAddr)
        .ensure_write(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;
    let addr_len = UserPtrRaw::new(addr_len as *const u32)
        .ensure_write(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;
    unsafe {
        match SaFamily::try_from(sock_addr.family).unwrap() {
            SaFamily::AfInet => {
                addr.write(sock_addr);
                addr_len.write(size_of::<SockAddrIn4>() as u32);
            }
            SaFamily::AfInet6 => {
                addr.write(sock_addr);
                addr_len.write(size_of::<SockAddrIn6>() as u32);
            },
            SaFamily::AfUnix => {
                addr.write(sock_addr);
                addr_len.write(size_of::<SockAddrUn>() as u32);
            },
            _ => todo!()
        }
    }
    Ok(())
}

///set host name
pub async fn sys_sethostname(hostname: usize, len: usize) -> SysResult {
    if (len as isize) < 0 || (len as isize) > 64 {
        return Err(SysError::EINVAL);
    }
    let task = current_task().unwrap();
    if hostname == 0 {
        return Err(SysError::EFAULT);
    }
    if hostname == 0 && len != 0 {
        return Err(SysError::EFAULT);
    }
    let hostname = UserSliceRaw::new(hostname as *const u8, len)
        .ensure_read(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;
    let file = open_file("/etc/hostname", OpenFlags::O_RDWR).unwrap();
    file.write_at(0,hostname.to_ref()).await?;
    // log::info!("[sys_hostname] hostname will be set {}", String::from_utf8_lossy(hostname.to_ref()));
    // log::info!("[sys_sethostname] now file hostname: {}", String::from_utf8_lossy(&file.read_all()));
    Ok(0)
}

pub async fn sys_gethostname(hostname: usize, len: usize) -> SysResult {
    if (len as isize) < 0 || (len as isize) > 64 {
        return Err(SysError::EINVAL);
    }
    let task = current_task().unwrap();
    if hostname == 0 {
        return Err(SysError::EFAULT);
    }
    if hostname == 0 && len != 0 {
        return Err(SysError::EFAULT);
    }
    let hostname = UserSliceRaw::new(hostname as *mut u8, len)
        .ensure_write(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;
    let file = open_file("/etc/hostname", OpenFlags::O_WRONLY|OpenFlags::O_CLOEXEC).unwrap();
    file.read(hostname.to_mut()).await?;
    Ok(0)
}