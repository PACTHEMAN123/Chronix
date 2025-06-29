use core::{fmt::Display, panic};

use alloc::{fmt, format};
use smoltcp::wire::{IpAddress, IpEndpoint, IpListenEndpoint,Ipv4Address, Ipv6Address};

use crate::syscall::SysError;

use super::SaFamily;


#[derive(Clone, Copy)]
#[repr(C)]
/// IPv4 Address
pub struct SockAddrIn4 {
    /// protocal family (AF_INET)
    pub sin_family: u16,
    /// port number
    pub sin_port: u16,
    /// IPv4 address
    pub sin_addr: Ipv4Address,
    /// padding, pd to sizeof(struct sockaddr_in)
    pub sin_zero: [u8; 8],
}

impl Into<IpEndpoint> for SockAddrIn4 {
    fn into(self) -> IpEndpoint {
        IpEndpoint::new(IpAddress::Ipv4(self.sin_addr), self.sin_port)
    }
}

impl From<IpEndpoint> for SockAddrIn4 {
    fn from (v4: IpEndpoint) -> Self {
        if let IpAddress::Ipv4(v4_addr) = v4.addr{
            Self {
                sin_family: SaFamily::AfInet as u16,
                sin_port: v4.port,
                sin_addr: v4_addr,
                sin_zero: [0; 8],
            }
        }else {
            panic!("Invalid IpEndpoint address type")
        }
        
    }
}

impl Into<IpListenEndpoint> for SockAddrIn4 {
    fn into(self) -> IpListenEndpoint {
        let inner_addr = if self.sin_addr == Ipv4Address::UNSPECIFIED {
            None
        } else {
            Some(IpAddress::Ipv4(self.sin_addr))
        };
        IpListenEndpoint{
            addr: inner_addr,
            port: self.sin_port,
        }
    }
}

impl fmt::Display for SockAddrIn4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let port = self.sin_port;
        let addr = format!(
            "{}.{}.{}.{}",
            self.sin_addr.octets()[0],
            self.sin_addr.octets()[1],
            self.sin_addr.octets()[2],
            self.sin_addr.octets()[3]
        );
        write!(f, "IPv4:{}:{}", addr, port)
    }
}

impl fmt::Debug for SockAddrIn4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// const zero IPV4 address
pub const ZERO_IPV4_ADDR: IpAddress = IpAddress::v4(0, 0, 0, 0);
/// const zero IpEndPoint
pub const ZERO_IPV4_ENDPOINT: IpEndpoint = IpEndpoint::new(ZERO_IPV4_ADDR, 0);
/// unspecified  listen endpoint
pub const UNSPECIFIED_LISTEN_ENDPOINT: IpListenEndpoint = IpListenEndpoint{
    addr: None,
    port: 0,
};
/// local ipv4 address
pub const LOCAL_IPV4: IpAddress = IpAddress::v4(127, 0, 0, 1);
/// local endpoint_ipv4:
pub const LOCAL_ENDPOINT_IPV4: IpEndpoint = IpEndpoint::new(LOCAL_IPV4, 0);

#[derive(Debug, Clone, Copy)]
#[repr(C)]
/// IPv6 Address
pub struct SockAddrIn6 {
    pub sin_family: u16,
    pub sin_port: u16,
    pub sin_flowinfo: u32,
    pub sin_addr: Ipv6Address,
    pub sin_scope_id: u32,
}
impl Into<IpEndpoint> for SockAddrIn6 {
    fn into(self) -> IpEndpoint {
        IpEndpoint::new(IpAddress::Ipv6(self.sin_addr),self.sin_port)
    }
}
impl Into<IpListenEndpoint> for SockAddrIn6 {
    fn into(self) -> IpListenEndpoint {
        let inner_addr = if self.sin_addr == Ipv6Address::UNSPECIFIED {
            None
        } else {
            Some(IpAddress::Ipv6(self.sin_addr))
        };
        IpListenEndpoint{
            addr: inner_addr,
            port: self.sin_port,
        }
    }
}
impl From<IpEndpoint> for SockAddrIn6 {
    fn from (v6: IpEndpoint) -> Self {
        if let IpAddress::Ipv6(v6_addr) = v6.addr{
            Self {
                sin_family: SaFamily::AfInet6 as u16,
                sin_port: v6.port,
                sin_flowinfo: 0,
                sin_addr: v6_addr,
                sin_scope_id: 0,
            }
        }else {
            panic!("Invalid IpEndpoint address type")
        }
        
    }
}

impl fmt::Display for SockAddrIn6 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let port = self.sin_port;
        let addr = format!(
            "{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
            self.sin_addr.as_bytes()[0],
            self.sin_addr.as_bytes()[1],
            self.sin_addr.as_bytes()[2],
            self.sin_addr.as_bytes()[3],
            self.sin_addr.as_bytes()[4],
            self.sin_addr.as_bytes()[5],
            self.sin_addr.as_bytes()[6],
            self.sin_addr.as_bytes()[7]
        );

        write!(f, "AF_INET6: [{}]:{}", addr, port)
    }
}
/// const zero IPV6 address
pub const ZERO_IPV6_ADDR: IpAddress = IpAddress::Ipv6(Ipv6Address::UNSPECIFIED);
/// const zero IpEndPoint
pub const ZERO_IPV6_ENDPOINT: IpEndpoint = IpEndpoint::new(ZERO_IPV6_ADDR, 0);

/// a superset of `SocketAddr` in `core::net` since it also
/// includes the address for socket communication between Unix processes. 
/// a user oriented program with a C language structure layout.
#[derive(Clone, Copy)]
#[repr(C)]
pub union SockAddr {
    pub family: u16,
    pub ipv4: SockAddrIn4,
    pub ipv6: SockAddrIn6,
    pub unix: SockAddrUn,
}

impl SockAddr {
    /// convert SockAddr wrapper into `IpEndpoint`
    pub fn into_endpoint(&self) -> Result<IpEndpoint,SysError> {
        unsafe {
            match SaFamily::try_from(self.family).unwrap() {
                SaFamily::AfInet => Ok(IpEndpoint::new(
                    IpAddress::Ipv4(self.ipv4.sin_addr), 
                    self.ipv4.sin_port
                )),
                SaFamily::AfInet6 => Ok(IpEndpoint::new(
                    IpAddress::Ipv6(self.ipv6.sin_addr), 
                    self.ipv6.sin_port
                )),
                _ => Err(SysError::EAFNOSUPPORT),
            }
        }   
    }
    /// SockAddr -> IpListenEndpoint
    pub fn into_listen_endpoint(&self) -> Result<IpListenEndpoint,SysError> {
        unsafe {
            match SaFamily::try_from(self.family).unwrap() {
                SaFamily::AfInet => Ok(self.ipv4.into()),
                SaFamily::AfInet6 => Ok(self.ipv6.into()),
                _ => Err(SysError::EAFNOSUPPORT),
            }
        }
    }
    /// IpEndpoint -> SockAddr
    pub fn from_endpoint(endpoint: IpEndpoint) -> Self {
        unsafe {
            match endpoint.addr {
                IpAddress::Ipv4(v4_addr) => Self {
                    ipv4: endpoint.into(),
                },
                IpAddress::Ipv6(v6_addr) => Self {
                    ipv6: endpoint.into(),
                },
            }
        }
    }
}

impl fmt::Debug for SockAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unsafe  {
            match self.family {
                2 => fmt::Display::fmt(&self.ipv4,f),
                10 => fmt::Display::fmt(&self.ipv6,f),
                _ => write!(f,"Unknown SockAddr family: {}",self.family),
            }
        }
    }
}

pub fn to_endpoint(listen_endpoint: IpListenEndpoint) -> IpEndpoint {
    let ip = match listen_endpoint.addr {
        Some(ip) => ip,
        None => ZERO_IPV4_ADDR,
    };
    IpEndpoint::new(ip, listen_endpoint.port)
}

pub fn is_unspecified(ip: IpAddress) -> bool {
    ip.as_bytes() == [0, 0, 0, 0] || ip.as_bytes() == [0, 0, 0, 0, 0, 0]
}

#[derive(Clone, Copy)]
#[repr(C)]
///
pub struct SockAddrUn {
    ///
    pub family: u16,
    ///
    pub path: [u8; 108],
}

