use strum::FromRepr;

/// reference: linux/include/uapi/asm-generic/errno.h
#[derive(Clone, Copy, Debug, Eq, PartialEq, FromRepr)]
#[repr(i32)]
pub enum SysError {
    /// Operation not permitted
    EPERM = 1,
    /// No such file or directory
    ENOENT = 2,
    /// No such process
    ESRCH = 3,
    /// Interrupted system call
    EINTR = 4,
    /// I/O error
    EIO = 5,
    /// No such device or address
    ENXIO = 6,
    /// Argument list too long
    E2BIG = 7,
    /// Exec format error
    ENOEXEC = 8,
    /// Bad file number
    EBADF = 9,
    /// No child processes
    ECHILD = 10,
    /// Resource temporarily unavailable
    EAGAIN = 11,
    /// Out of memory
    ENOMEM = 12,
    /// Permission denied
    EACCES = 13,
    /// Bad address
    EFAULT = 14,
    /// Block device required
    ENOTBLK = 15,
    /// Device or resource busy
    EBUSY = 16,
    /// File exists
    EEXIST = 17,
    /// Cross-device link
    EXDEV = 18,
    /// No such device
    ENODEV = 19,
    /// Not a directory
    ENOTDIR = 20,
    /// Is a directory
    EISDIR = 21,
    /// Invalid argument
    EINVAL = 22,
    /// File table overflow
    ENFILE = 23,
    /// Too many open files
    EMFILE = 24,
    /// Not a typewriter
    ENOTTY = 25,
    /// Text file busy
    ETXTBSY = 26,
    /// File too large
    EFBIG = 27,
    /// No space left on device
    ENOSPC = 28,
    /// Illegal seek
    ESPIPE = 29,
    /// Read-only file system
    EROFS = 30,
    /// Too many links
    EMLINK = 31,
    /// Broken pipe
    EPIPE = 32,
    /// Math argument out of domain of func
    EDOM = 33,
    /// Math result not representable
    ERANGE = 34,
    /// Resource deadlock would occur
    EDEADLK = 35,
    /// File name too long
    ENAMETOOLONG = 36,
    /// No record locks available
    ENOLCK = 37,
    /// Invalid system call number
    ENOSYS = 38,
    /// Directory not empty
    ENOTEMPTY = 39,
    /// Too many symbolic links encountered
    ELOOP = 40,
    /// Timer expired   
    ETIME = 62,
    /// Socket operation on non-socket
    ENOTSOCK = 88,
    /// sendmsg bigger than biggest message
    EMSGSIZE = 90,
    /// Protocol not supported
    EPROTONOSUPPORT = 93,
    /// Unsupported
    EOPNOTSUPP = 95,
    /// unsupported protocol
    EAFNOSUPPORT = 97,
    /// Socket address is already in use
    EADDRINUSE = 98,
    /// Address not available
    EADDRNOTAVAIL = 99,
    /// Connection reset
    ECONNRESET = 104,
    /// Transport endpoint is already connected
    EISCONN = 106,
    /// The socket is not connected
    ENOTCONN = 107,
    /// time out
    ETIMEOUT = 110,
    /// Connection refused
    ECONNREFUSED = 111,
    /// The socket is nonblocking and the connection cannot be completed
    /// immediately.(connect.2)
    EINPROGRESS = 115,
    EOWNERDIED = 130,
}

impl SysError {
    /// get the error code as isize
    pub const fn code(self) -> isize {
        self as isize
    }

    /// get the error from i32
    pub fn from_i32(e: i32) -> Self {
        Self::from_repr(e).expect("unknown error")
    }
}

