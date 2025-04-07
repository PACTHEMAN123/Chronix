use core::{ops::DerefMut, time::Duration};

use alloc::{boxed::Box, collections::btree_map::BTreeMap, vec,vec::Vec};
use listen_table::ListenTable;
use log::info;
use smoltcp::{iface::{Config, Interface, PollResult, SocketHandle, SocketSet}, phy::Medium, socket::{tcp::{Socket, SocketBuffer}, AnySocket}, time::Instant, wire::{EthernetAddress, HardwareAddress, IpAddress, IpCidr, IpListenEndpoint}};
use spin::{Lazy, Once};

use crate::{devices::{net::NetDeviceWrapper, NetDevice}, drivers::net::loopback::LoopbackDevice, sync::{mutex::{SpinNoIrq, SpinNoIrqLock}, UPSafeCell}, timer::{get_current_time_duration, get_current_time_us, timer::{Timer, TimerEvent, TIMER_MANAGER}}};
/// Network Address Module
pub mod addr;
/// Network Socket Module
pub mod socket;
/// TCP Module
pub mod tcp;
/// udp Module
pub mod udp;
/// A Listen Table for Server to allocte port
pub mod listen_table;
#[repr(u16)]
#[derive(Debug, Clone, Copy)]
/// socket address family, used for syscalls
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

const SOCK_RAND_SEED: u64 = 404;// for random port allocation
const CONFIG_RANDOM_SEED: u64 = 0x3A0C_1495_BC68_9A2C; // for smoltcp random seed
const PORT_START: u16 = 0xc000; // 49152
const PORT_END: u16 = 0xffff;   // 65535

const LISTEN_QUEUE_SIZE: usize = 512;
static LISTEN_TABLE: Lazy<ListenTable> = Lazy::new(ListenTable::new);

/// A wrapper for SocketSet in smoltcp
struct SocketSetWrapper<'a>(SpinNoIrqLock<SocketSet<'a>>) ; 
static SOCKET_SET: Lazy<SocketSetWrapper> = Lazy::new(SocketSetWrapper::new);

/// TCP RX and TX buffer size
pub const TCP_RX_BUF_LEN: usize = 64 * 1024;
/// TCP RX and TX buffer size
pub const TCP_TX_BUF_LEN: usize = 64 * 1024;
const UDP_RX_BUF_LEN: usize = 64 * 1024;
const UDP_TX_BUF_LEN: usize = 64 * 1024;

static ETH0: Once<InterfaceWrapper> = Once::new();
/// A wrapper for interface in smoltcp
struct InterfaceWrapper {
    /// The name of the network interface.
    name: &'static str,
    /// The Ethernet address of the network interface.
    ether_addr: EthernetAddress,
    /// The device wrapper protected by a SpinNoIrqLock to ensure thread-safe access.
    dev: SpinNoIrqLock<NetDeviceWrapper>,
    /// The network interface protected by a SpinNoIrqLock to ensure thread-safe
    /// access.
    iface: SpinNoIrqLock<Interface>,
}

impl InterfaceWrapper {
    fn new(name: &'static str, dev: Box<dyn NetDevice>, ether_addr: EthernetAddress) -> Self {
        let mut config = match dev.capabilities().medium {
            Medium::Ethernet => Config::new(HardwareAddress::Ethernet(ether_addr)),
            Medium::Ip => Config::new(HardwareAddress::Ip),
        };
        config.random_seed = CONFIG_RANDOM_SEED;
        let mut raw_dev = NetDeviceWrapper::new(dev);
        let iface = SpinNoIrqLock::new(Interface::new(config, &mut raw_dev, Self::current_time()));
        Self {
            name,
            ether_addr,
            dev:SpinNoIrqLock::new(raw_dev),
            iface,
        }
    }
    pub fn name(&self) -> &str {
        self.name
    }

    pub fn ethernet_address(&self) -> EthernetAddress {
        self.ether_addr
    }
    fn current_time() -> Instant {
        Instant::from_micros_const(get_current_time_us() as i64)
    }
    /// poll the interface to detect device status then poll sockets
    pub fn poll(&self, sockets: &SpinNoIrqLock<SocketSet>) -> Instant {
        let mut dev =  self.dev.lock();
        let mut iface = self.iface.lock();
        let mut sockets = sockets.lock();
        let timestamp = Self::current_time();
        let res = iface.poll(timestamp, dev.deref_mut(), &mut sockets);
        // log::warn!("[net::InterfaceWrapper::poll] does something have been changed? {res:?}");
        timestamp
    }
    /// check the interface and call poll socket_handle to detect device status then poll sockets
    pub fn check_poll(&self, timestamp: Instant, sockets: &SpinNoIrqLock<SocketSet>) {
        let mut iface = self.iface.lock();
        let mut sockets = sockets.lock();
        match iface.poll_delay(timestamp, &mut sockets)
        .map(smol_dur_to_core_cur){
            Some(Duration::ZERO) => {
                iface.poll(Self::current_time(), self.dev.lock().deref_mut(), &mut sockets);
            }
            Some(delay) => {
                // current time + delay is the deadline for the next poll
                let next_poll_deadline = delay +  Duration::from_micros(timestamp.micros() as u64);
                let current_time = get_current_time_duration();
                if next_poll_deadline < current_time {
                    iface.poll(Self::current_time(), self.dev.lock().deref_mut(), &mut sockets);
                }else {
                    let timer = Timer::new(next_poll_deadline, Box::new(NetPollTimer{}));
                    TIMER_MANAGER.add_timer(timer);
                }
            }
            // when return None means no active sockets or all the sockets are handled
            None => {
                // do nothing, just call poll interface
                let empty_timer = Timer::new(get_current_time_duration()+Duration::from_millis(5), Box::new(NetPollTimer{}));
                TIMER_MANAGER.add_timer(empty_timer);
            }
        }
    }

}

impl <'a> SocketSetWrapper<'a> {
    fn new() -> Self {
        let socket_set = SocketSet::new(vec![]);
        Self(SpinNoIrqLock::new(socket_set))
    }
    /// allocate tx buffer and rx buffer ,return a Socket struct in smoltcp
    pub fn new_tcp_socket() -> smoltcp::socket::tcp::Socket<'a> {
        let rx_buffer = SocketBuffer::new(vec![0; TCP_RX_BUF_LEN]);
        let tx_buffer = SocketBuffer::new(vec![0; TCP_TX_BUF_LEN]);
        Socket::new(rx_buffer, tx_buffer)
    }
    /// allocate a udp socket, return a Socket struct in smoltcp
    pub fn new_udp_socket() -> smoltcp::socket::udp::Socket<'a> {
        let rx_buffer = smoltcp::socket::udp::PacketBuffer::new(
            vec![smoltcp::socket::udp::PacketMetadata::EMPTY; 8],
            vec![0; UDP_RX_BUF_LEN], 
        );
        let tx_buffer = smoltcp::socket::udp::PacketBuffer::new(
            vec![smoltcp::socket::udp::PacketMetadata::EMPTY; 8],
            vec![0; UDP_TX_BUF_LEN],
        );
        smoltcp::socket::udp::Socket::new(rx_buffer, tx_buffer)
    }
    /// add a socket to the set , return a socket_handle
    pub fn add_socket<T:AnySocket<'a>>(&self, socket: T) -> SocketHandle {
        let handle = self.0.lock().add(socket);
        // info!("[SocketSetWrapper] add_socket handle {:?}" , handle);
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
    /// wrapper for eth timed poll
    pub fn poll_interfaces(&self) -> Instant {
        ETH0.get()
        .unwrap()
        .poll(&self.0)
    }
    /// wrapper for eth timed check_polled
    pub fn check_poll(&self, timestamp: Instant) {
        ETH0.get()
        .unwrap()
        .check_poll(timestamp, &self.0)
    }

    pub fn remove(&self, handle: SocketHandle) {
        self.0.lock().remove(handle);
        info!("socket {:?}: destroyed", handle);
    }
}

/// Poll the network stack.
///
/// It may receive packets from the NIC and process them, and transmit queued
/// packets to the NIC.
pub fn poll_interfaces() -> smoltcp::time::Instant {
    SOCKET_SET.poll_interfaces()
}
/// modify the socket first, a helper method for use smoltcp consume
pub fn modify_tcp_packet(buf: &[u8], sockets: &mut SocketSet<'_>, is_ethernet: bool) ->Result<(), smoltcp::wire::Error>{
    use smoltcp::wire::{EthernetFrame, IpProtocol, Ipv4Packet, TcpPacket};

    let ipv4_packet = if is_ethernet {
        let ether_frame = EthernetFrame::new_checked(buf)?;
        Ipv4Packet::new_checked(ether_frame.payload())?
    }else {
        Ipv4Packet::new_checked(buf)?
    };
    if ipv4_packet.next_header() == IpProtocol::Tcp {
        let tcp_packet = TcpPacket::new_checked(ipv4_packet.payload())?;
        let src_addr = (ipv4_packet.src_addr(), tcp_packet.src_port()).into();
        let dst_addr = (ipv4_packet.dst_addr(),tcp_packet.dst_port()).into();
        let first_flag = tcp_packet.syn() && !tcp_packet.ack();
        if first_flag {
            // info!("[modify tcp packet]receive tcp");
            LISTEN_TABLE.handle_coming_tcp(src_addr, dst_addr, sockets);
        }
    }
    Ok(())
}
/// a port allocator for udp socket bind
pub struct PortManager {
    port_map: SpinNoIrqLock<BTreeMap<u16, (usize, IpListenEndpoint)>>,
}
pub (crate) static PORT_MANAGER: Lazy<PortManager> = Lazy::new(PortManager::new);
impl PortManager {
    const fn new() -> Self {
        Self {
            port_map: SpinNoIrqLock::new(BTreeMap::new()),
        }
    }
    /// get fd and endpoint for a port
    pub fn get(&self, port: u16) -> Option<(usize, IpListenEndpoint)> {
        self.port_map.lock().get(&port).cloned()
    }
    /// insert a port and endpoint to the map
    pub fn insert(&self, port: u16, fd: usize, endpoint: IpListenEndpoint) {
        self.port_map.lock().insert(port, (fd, endpoint));
    }
    /// remove a port from the map
    pub fn remove(&self, port: u16) {
        self.port_map.lock().remove(&port);
    }
}
// function or struct concerning time ,from microseconds to smoltcp::time::Instant, from core::time::Duration to smoltcp::time::Duration
/// timer for network poll
struct NetPollTimer;
impl TimerEvent for NetPollTimer {
    fn callback(self: Box<Self>) -> Option<Timer> {
        SOCKET_SET.poll_interfaces();
        None
    }
}
/// from core::time::Duration to smoltcp::time::Duration
pub fn smol_dur_to_core_cur(duration: smoltcp::time::Duration) -> core::time::Duration {
    core::time::Duration::from_micros(duration.micros())
}

pub fn init_network_loopback() {
    info!("Initialize network loopback");
    let dev = LoopbackDevice::new();
    let ehter_addr = EthernetAddress(dev.mac_address().0);
    let eth0 = InterfaceWrapper::new("eth0", dev, ehter_addr);
    let gateway: IpAddress = match option_env!("GATEWAY") {
        Some(gw) => {
            gw.parse().unwrap()
        },
        None => {
            "".parse().unwrap()
        }
    };
    let ip = "127.0.0.1".parse().unwrap();
    let ip_addrs = vec![IpCidr::new(ip,8)];
    eth0.iface.lock().update_ip_addrs(|inner_ip_addrs|{
        inner_ip_addrs.extend(ip_addrs);
    });
    match gateway {
        IpAddress::Ipv4(gateway_v4) => {
            eth0.iface.lock().routes_mut().add_default_ipv4_route(gateway_v4).unwrap();
        }
        _ => {}
    }
    ETH0.call_once(|| eth0);

    info!("created net interface {:?}:", ETH0.get().unwrap().name());
    info!("  ether:    {}", ETH0.get().unwrap().ethernet_address());
    info!("  ip:       {}", ip);
    info!("  gateway:  {}", gateway);
    
}