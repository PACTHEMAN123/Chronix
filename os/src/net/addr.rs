use core::panic;

use smoltcp::wire::{IpAddress, IpEndpoint, IpListenEndpoint,Ipv4Address, Ipv6Address};

use super::SaFamily;


#[derive(Debug, Clone, Copy)]
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
/// const zero IPV6 address
pub const ZERO_IPV6_ADDR: IpAddress = IpAddress::Ipv6(Ipv6Address::UNSPECIFIED);
/// const zero IpEndPoint
pub const ZERO_IPV6_ENDPOINT: IpEndpoint = IpEndpoint::new(ZERO_IPV6_ADDR, 0);

/// Socket Address Struct wrapped both ipv4 and ipv6 address
pub enum SockAddr {
    SockAddrIn4(SockAddrIn4),
    SockAddrIn6(SockAddrIn6),
}
