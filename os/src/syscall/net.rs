use core::{any::Any, mem, option, panic};

use alloc::{sync::Arc, task, vec,vec::Vec};
use fatfs::{info, warn};
use hal::{addr, instruction::{Instruction, InstructionHal}, println};
use lwext4_rust::bindings::EXT4_SUPERBLOCK_FLAGS_TEST_FILESYS;

use crate::{config::PAGE_SIZE, fs::{pipefs, OpenFlags}, net::{addr::{SockAddr, SockAddrIn4, SockAddrIn6}, socket::{self, Sock}, tcp::TcpSocket, SaFamily}, signal::SigSet, task::{current_task, fs::{FdFlags, FdInfo}}, utils::yield_now};

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
            3 => Ok(Self::RAW),
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
pub fn sys_socket(domain: usize, types: usize, _protocol: usize) -> SysResult {
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

    let types = SocketType::try_from(types)?;
    let socket = socket::Socket::new(domain,types, nonblock);
    let fd_info = FdInfo {
        file: Arc::new(socket),
        flags: flags.into(),
    };
    let task = current_task().unwrap();
    let fd = task.with_mut_fd_table(|t|t.alloc_fd());
    task.with_mut_fd_table(|t| {
        t.put_file(fd, fd_info).or_else(|e|Err(e))
    })?;
    log::info!("[sys_socket] fd: {}", fd);
    Ok(fd as isize)
}
/// “assigning a name to a socket”
pub fn sys_bind(fd: usize, addr: usize, addr_len: usize) -> SysResult {
    let task = current_task().unwrap();
    let family = SaFamily::try_from(unsafe {
        *(addr as *const u16)
    })?;
    let local_addr = match family {
        SaFamily::AfInet => {
            if addr_len < size_of::<SockAddrIn4>() {
                return Err(SysError::EINVAL);
            }
            Ok(SockAddr{
                ipv4: unsafe { *(addr as *const _)},
            })
        }
        SaFamily::AfInet6 => {
            if addr_len < size_of::<SockAddrIn6>() {
                return Err(SysError::EINVAL);
            }
            Ok(SockAddr{
                ipv6: unsafe {
                    *(addr as *const _)
                }
            })
        },
    }?;
    log::info!("[sys_bind] local_addr's port is: {}",unsafe {
        local_addr.ipv4.sin_port
    });
    let socket_file = task.with_fd_table(|table| {
        table.get_file(fd)})?
        .downcast_arc::<socket::Socket>().unwrap_or_else(|_| {
        panic!("Failed to downcast to socket::Socket")
    });
    socket_file.sk.bind(fd, local_addr)?;
    Ok(0)
}

/// Mark the stream socket referenced by the file descriptor `sockfd` as
/// passive. This socket will be used later to accept connections from other
/// (active) sockets
pub fn sys_listen(fd: usize, _backlog: usize) -> SysResult {
    let current_task = current_task().unwrap();
    let socket_file = current_task.with_fd_table(|table| {
        table.get_file(fd)})?
        .downcast_arc::<socket::Socket>()
        .unwrap_or_else(|_| {
            panic!("Failed to downcast to socket::Socket")
        });
    socket_file.sk.listen()?;
    Ok(0)
}

/// Connect the active socket refrenced by the file descriptor `sockfd` to the
/// address specified by `addr`. The `addr` argument is a pointer to a
/// `sockaddr` structure that contains the address of the remote socket.
/// The `addrlen` argument specifies the size of this structure.
pub async fn sys_connect(fd: usize, addr: usize, addr_len: usize) -> SysResult {
    let task = current_task().unwrap();
    let remote_addr = match SaFamily::try_from(unsafe {
        *(addr as *const u16)
    })? {
        SaFamily::AfInet => {
            if addr_len < size_of::<SockAddrIn4>() {
                return Err(SysError::EINVAL);
            }
            Ok(SockAddr{
                ipv4: unsafe { *(addr as *const _) },
            })
        }
        SaFamily::AfInet6 => {
            if addr_len < size_of::<SockAddrIn6>() {
                return Err(SysError::EINVAL);
            }
            Ok(SockAddr{
                ipv6: unsafe { *(addr as *const _) },
            })
        }
    }?;
    // log::info!("[sys_connect] remote_addr's port is: {}",
        // unsafe {
            // remote_addr.ipv4.sin_port
    // });
    let socket_file = task.with_fd_table(|table| {
        table.get_file(fd)})?
        .downcast_arc::<socket::Socket>()
        .unwrap_or_else(|_| {
            panic!("Failed to downcast to socket::Socket")
        });
    socket_file.sk.connect(remote_addr.into_endpoint()).await?;
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
    let task = current_task().unwrap();
    let socket_file = task.with_fd_table(|table| {
        table.get_file(fd)})?
        .downcast_arc::<socket::Socket>()
        .unwrap_or_else(|_| {
            panic!("Failed to downcast to socket::Socket")
        });
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
    unsafe {
        match SaFamily::try_from(peer_addr.family).unwrap() {
            SaFamily::AfInet => {
                let addr_ptr = addr as *mut SockAddrIn4;
                addr_ptr.write_volatile(peer_addr.ipv4);
                let addr_len_ptr = addr_len as *mut u32;
                addr_len_ptr.write_volatile(size_of::<SockAddrIn4>() as u32);
            }
            SaFamily::AfInet6 => {
                let addr_ptr = addr as *mut SockAddrIn6;
                addr_ptr.write_volatile(peer_addr.ipv6);
                let addr_len_ptr = addr_len as *mut u32;
                addr_len_ptr.write_volatile(size_of::<SockAddrIn6>() as u32);
            },
        }
    }

    let accept_socket = Arc::new(socket::Socket::from_another(&socket_file, Sock::TCP(accept_sk)));
    let fd_info = FdInfo {
        file: accept_socket,
        flags: OpenFlags::empty().into(),
    };
    let new_fd = task.with_mut_fd_table(|t|t.alloc_fd());
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
    // log::info!("addr is {}, addr_len is {}", addr, addr_len);
    let buf_slice = buf as *const u8 ;
    let task = current_task().unwrap();
    let buf_slice = unsafe {
        core::slice::from_raw_parts_mut(buf_slice as *mut u8, len)
    };
    let socket_file = task.with_fd_table(|table| {
        table.get_file(fd)})?
        .downcast_arc::<socket::Socket>()
        .unwrap_or_else(|_| {
            panic!("Failed to downcast to socket::Socket")
        });
    task.set_interruptable();
    let bytes = match socket_file.sk_type {
        SocketType::DGRAM => {
            let remote_addr = if addr != 0 {  Some(
                match SaFamily::try_from(unsafe {
                    *(addr as *const u16)
                })? {
                    SaFamily::AfInet => {
                        if addr_len < size_of::<SockAddrIn4>() {
                            log::warn!("sys_sendto: addr_len < size_of::<SockAddrIn4>() which is {}",size_of::<SockAddrIn4>());
                            return Err(SysError::EINVAL);
                        }
                        Ok(SockAddr{
                            ipv4: unsafe { *(addr as *const _) },
                        })
                    }
                    SaFamily::AfInet6 => {
                        if addr_len < size_of::<SockAddrIn6>() {
                            return Err(SysError::EINVAL);
                        }
                        Ok(SockAddr{
                            ipv6: unsafe { *(addr as *const _) },
                        })
                    }
                }?
            .into_endpoint())}else {
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
    let task = current_task().unwrap();
    let socket_file = task.with_fd_table(|table| {
        table.get_file(sockfd)})?
        .downcast_arc::<socket::Socket>()
        .unwrap_or_else(|_| {
            panic!("Failed to downcast to socket::Socket")
        });
    let mut inner_vec = Vec::with_capacity(len);
    unsafe {
        inner_vec.set_len(len);
    }
    task.set_interruptable();
    let (bytes, remote_endpoint) = socket_file.sk.recv(&mut inner_vec).await?;
    log::info!("recvfrom: bytes: {}, remote_endpoint: {:?}", bytes, remote_endpoint);
    let remote_addr = SockAddr::from_endpoint(remote_endpoint);
    task.set_running();
    // write to pointer
    log::info!("now set running");
    let buf_slice = unsafe {
        core::slice::from_raw_parts_mut(buf as *mut u8, bytes)
    };
    buf_slice[..bytes].copy_from_slice(&inner_vec[..bytes]);
    // write to sockaddr_in
    if addr == 0 {
        return Ok(bytes as isize);  
    }
    unsafe {
        match SaFamily::try_from(remote_addr.family).unwrap() {
            SaFamily::AfInet => {
                let addr_ptr = addr as *mut SockAddrIn4;
                addr_ptr.write_volatile(remote_addr.ipv4);
                let addr_len_ptr = addrlen as *mut u32;
                addr_len_ptr.write_volatile(size_of::<SockAddrIn4>() as u32);
            }
            SaFamily::AfInet6 => {
                let addr_ptr = addr as *mut SockAddrIn6;
                addr_ptr.write_volatile(remote_addr.ipv6);
                let addr_len_ptr = addrlen as *mut u32;
                addr_len_ptr.write_volatile(size_of::<SockAddrIn6>() as u32);
            },
        }
    }
    log::info!("now return bytes: {}",bytes);
    Ok(bytes as isize)
}
/// Returns the local address of the Socket corresponding to `sockfd`.
pub fn sys_getsockname(fd: usize, addr: usize, addr_len: usize) -> SysResult {
    log::info!("sys_getsockname fd: {}, addr: {:#x}, addr_len: {}", fd, addr, addr_len);
    let task = current_task().unwrap();
    let socket_file = task.with_fd_table(|table| {
        table.get_file(fd)
        .clone()
        .unwrap()
        .downcast_arc::<socket::Socket>()
        .unwrap_or_else(|_| {
            panic!("Failed to downcast to socket::Socket")
        })
    });
    let local_addr = socket_file.sk.local_addr()?;
    log::info!("Get local address of socket: {:?}", local_addr);
    // write to pointer
    unsafe {
        match SaFamily::try_from(local_addr.family).unwrap() {
            SaFamily::AfInet => {
                let addr_ptr = addr as *mut SockAddrIn4;
                addr_ptr.write_volatile(local_addr.ipv4);
                let addr_len_ptr = addr_len as *mut u32;
                addr_len_ptr.write_volatile(size_of::<SockAddrIn4>() as u32);
            }
            SaFamily::AfInet6 => {
                let addr_ptr = addr as *mut SockAddrIn6;
                addr_ptr.write_volatile(local_addr.ipv6);
                let addr_len_ptr = addr_len as *mut u32;
                addr_len_ptr.write_volatile(size_of::<SockAddrIn6>() as u32);
            },
        }
    }
    Ok(0)
}

/// returns the peer address of the socket corresponding to the file descriptor `sockfd`
pub fn sys_getpeername(fd: usize, addr: usize, addr_len: usize) -> SysResult {
    let task = current_task().unwrap();
    let socket_file = task.with_fd_table(|table| {
        table.get_file(fd)})?
        .downcast_arc::<socket::Socket>()
        .unwrap_or_else(|_| {
            panic!("Failed to downcast to socket::Socket")
        });
    let peer_addr = socket_file.sk.peer_addr().unwrap();
    log::info!("Get peer address of socket: {:?}", peer_addr);
    // write to pointer
    unsafe {
        match SaFamily::try_from(peer_addr.family).unwrap() {
            SaFamily::AfInet => {
                let addr_ptr = addr as *mut SockAddrIn4;
                addr_ptr.write_volatile(peer_addr.ipv4);
                let addr_len_ptr = addr_len as *mut u32;
                addr_len_ptr.write_volatile(size_of::<SockAddrIn4>() as u32);
            }
            SaFamily::AfInet6 => {
                let addr_ptr = addr as *mut SockAddrIn6;
                addr_ptr.write_volatile(peer_addr.ipv6);
                let addr_len_ptr = addr_len as *mut u32;
                addr_len_ptr.write_volatile(size_of::<SockAddrIn6>() as u32);
            },
        }
    }
    Ok(0)
}
#[allow(missing_docs)]
pub enum SocketLevel {
    /// Dummy protocol for TCP
    IpprotoIp = 0,
    /// S
    SolSocket = 1,
    IpprotoTcp = 6,
    /// IPv6-in-IPv4 tunnelling
    IpprotoIpv6 = 41,
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
// ============================== 
/// socket configure interface for user
/// level: protocel level at which the option resides,
/// option name
pub fn sys_setsockopt  (
    _fd: usize,
    _level: usize,
    _option_name: usize,
    _option_value: usize,
    _option_len: usize,
) -> SysResult {
    Ok(0)
}
/// get socket configure interface for user
pub fn sys_getsockopt (
    _fd: usize,
    level: usize,
    option_name: usize,
    option_value: usize,
    option_len: usize,
) -> SysResult {
    match SocketLevel::try_from(level)? {
        SocketLevel::SolSocket => {
            const SEND_BUFFER_SIZE: usize = 64 * 1024; // 64KB
            const RECV_BUFFER_SIZE: usize = 64 * 1024; // 64KB
            match SocketOption::try_from(option_name)?{
                SocketOption::SNDBUF => {
                    let optval_ptr = option_value as *mut u32;
                    let optlen_ptr = option_len as *mut u32;
                    unsafe {
                        optval_ptr.write_volatile(SEND_BUFFER_SIZE as u32);
                        optlen_ptr.write_volatile(size_of::<u32>() as u32);
                    }
                },
                SocketOption::RCVBUF => {
                    let optval_ptr = option_value as *mut u32;
                    let optlen_ptr = option_len as *mut u32;
                    unsafe {
                        optval_ptr.write_volatile(RECV_BUFFER_SIZE as u32);
                        optlen_ptr.write_volatile(size_of::<u32>() as u32);
                    }
                },
                _ =>{
                    todo!()
                } 
            }
        },
        SocketLevel::IpprotoTcp | SocketLevel::IpprotoIp  => {
            const MAX_SEGMENT: usize = 1460; // 1460 bytesusually MTU
            let optlen_ptr = option_len as *mut u32;
            match TcpSocketOption::try_from(option_name)? {
                TcpSocketOption::NODELAY => {
                    unsafe {
                        let optval_ptr = option_value as *mut u32;
                        optval_ptr.write_volatile(0 as u32);
                        optlen_ptr.write_volatile(size_of::<u32>() as u32);
                    }
                },
                TcpSocketOption::MAXSEG => {
                    unsafe {
                        let optval_ptr = option_value as *mut u32;
                        optval_ptr.write_volatile(MAX_SEGMENT as u32);
                        optlen_ptr.write_volatile(size_of::<u32>() as u32);
                    } 
                },
                TcpSocketOption::INFO => todo!(),
                TcpSocketOption::CONGESTION => {
                    todo!()
                },
            }
        },
        SocketLevel::IpprotoIpv6 => todo!(),
    }
    Ok(0)
}

/// sys_shutdown() allows a greater control over the behaviour of connection-oriented sockets.
/// todo : how used for indicate read is shut down, write is shut down, or both 
pub fn sys_shutdown(fd: usize, _how: usize) -> SysResult {
    let task = current_task().unwrap();
    let socket_file = task.with_fd_table(|table| {
        table.get_file(fd)})?
        .downcast_arc::<socket::Socket>()
        .unwrap_or_else(|_| {
            panic!("Failed to downcast to socket::Socket")
        });
    socket_file.sk.shutdown()?;
    Ok(0)
}
/// create a pair of connected sockets
pub fn sys_socketpair(_domain: usize, _types: usize, _protocol: usize, sv: usize) -> SysResult {
    let task = current_task().unwrap();
    let (pipe_read, pipe_write) = pipefs::make_pipe(PAGE_SIZE);
    let pipe = task.with_mut_fd_table(|table| {
        let fd_read = table.alloc_fd();
        let fd_info_read = FdInfo {
            file: pipe_read,
            flags: FdFlags::empty(),
        };
        table.put_file(fd_read, fd_info_read)?;
        let fd_write = table.alloc_fd();
        let fd_info_write = FdInfo {
            file: pipe_write,
            flags: FdFlags::empty(),    
        };
        table.put_file(fd_write, fd_info_write)?;
        Ok([fd_read as u32, fd_write as u32])
    })?;
    let sv_ptr = sv as *mut [u32; 2];
    unsafe {
        sv_ptr.write_volatile(pipe);
    }
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
    let task = current_task().unwrap();
    if flags != 0 {
        log::warn!("unsupported flags: {}", flags);
    }
    let socket_file = task.with_fd_table(|table| {
        table.get_file(fd)})?
        .downcast_arc::<socket::Socket>()
        .unwrap_or_else(|_| {
            panic!("Failed to downcast to socket::Socket")
        });
    let msg_ptr = msg as *const MsgHdr;
    let msg = unsafe { msg_ptr.read() };
    if msg.msg_controllen != 0 {
        log::warn!("unsupported control data");
    }
    let addr = match SaFamily::try_from(unsafe {
        *(msg.msg_name as *const u16)
    })? {
        SaFamily::AfInet => {
            if msg.msg_namelen < mem::size_of::<SockAddrIn4>() as u32 {
                log::error!("[sendmsg] invalid address length: {}", msg.msg_namelen);
                return Err(SysError::EINVAL);
            }
            Ok(SockAddr{
                ipv4: unsafe { *(msg.msg_name as *const _) },
            }.into_endpoint())
        },
        SaFamily::AfInet6 => {
            if msg.msg_namelen < mem::size_of::<SockAddrIn6>() as u32 {
                log::error!("[sendmsg] invalid address length: {}", msg.msg_namelen);
                return Err(SysError::EINVAL);
            }
            Ok(SockAddr{
                ipv6: unsafe {
                    *(msg.msg_name as *const _)
                }
            }.into_endpoint())
        },
    }?;
    let iovs = unsafe {
        Instruction::set_sum();
        core::slice::from_raw_parts(msg.msg_iov as *const IoVec, msg.msg_iovlen as usize)
    };
    let mut total_len = 0;
    for (_i, iov) in iovs.iter().enumerate() {
        if iov.len == 0 {
            continue;
        }
        let ptr = iov.base as *const u8;
        let buf_slice = unsafe {
            core::slice::from_raw_parts(ptr, iov.len as usize)
        };
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
    if flags != 0 {
        log::warn!("unsupported flags: {}", flags);
    }
    let task = current_task().unwrap();
    let socket_file = task.with_fd_table(|table| {
        table.get_file(fd)})?
        .downcast_arc::<socket::Socket>()
        .unwrap_or_else(|_| {
            panic!("Failed to downcast to socket::Socket")
        });
    let msg_ptr = msg as *mut MsgHdr;
    let inner_msg = unsafe { msg_ptr.read() };
    if inner_msg.msg_controllen != 0 {
        log::warn!("unsupported control data");
    }
    let iovs = unsafe {
        Instruction::set_sum();
        core::slice::from_raw_parts(inner_msg.msg_iov as *const IoVec, inner_msg.msg_iovlen as usize)
    };
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
        unsafe {
            match SaFamily::try_from(addr.family)? {
                SaFamily::AfInet => {
                    let addr_ptr = inner_msg.msg_name as *mut SockAddrIn4;
                    addr_ptr.write_volatile(addr.ipv4);
                    let addr_len_ptr = inner_msg.msg_namelen as *mut u32;
                    addr_len_ptr.write_volatile(size_of::<SockAddrIn4>() as u32);
                },
                SaFamily::AfInet6 => {
                    let addr_ptr = inner_msg.msg_name as *mut SockAddrIn6;
                    addr_ptr.write_volatile(addr.ipv6);
                    let addr_len_ptr = inner_msg.msg_namelen as *mut u32;
                    addr_len_ptr.write_volatile(size_of::<SockAddrIn6>() as u32);
                },
            }
        }
    }
                    
    Ok(copied as isize)
}

