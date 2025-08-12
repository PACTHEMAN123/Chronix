use crate::{
    net::{socket::SockResult, SocketSetWrapper, LOCAL_IP, SOCKET_SET},
    sync::UPSafeCell,
    syscall::{net::Protocol, SysError},
    task::current_task,
    utils::{get_waker, suspend_now},
};
use alloc::vec;
use alloc::vec::Vec;
use smoltcp::{
    iface::{SocketHandle, SocketSet},
    wire::{IpAddress, IpEndpoint, IpProtocol},
};
pub struct RawSocket {
    handle: UPSafeCell<Option<SocketHandle>>,
    protocol: IpProtocol,
}

impl RawSocket {
    pub fn new(protocol: IpProtocol) -> Self {
        let socket = SocketSetWrapper::new_raw_socket(protocol);
        let handle = SOCKET_SET.add_socket(socket);
        Self {
            handle: UPSafeCell::new(Some(handle)),
            protocol,
        }
    }

    pub async fn send(&self, data: &[u8], remote: Option<IpEndpoint>) -> SockResult<usize> {
        let waker = get_waker().await;
        let handle: SocketHandle = self.handle.exclusive_access().unwrap();
        let ret = self
            .block_on(|| {
                SOCKET_SET.with_socket_mut::<smoltcp::socket::raw::Socket, _, _>(handle, |socket| {
                    if socket.can_send() {
                        // 仅对ICMP且用户数据包含完整头部时重计算校验和
                        let final_buf = match self.protocol {
                            IpProtocol::Icmp if data.len() >= 8 => {
                                let mut packet = data.to_vec();
                                packet[2..4].copy_from_slice(&[0, 0]); // 清零校验和字段
                                let checksum = icmp_checksum(&packet);
                                packet[2..4].copy_from_slice(&checksum.to_be_bytes());
                                packet
                            }
                            _ => data.to_vec(),
                        };

                        // 设置目标地址（若指定）
                        if let Some(ep) = remote {
                            let dst_ip = match ep.addr {
                                IpAddress::Ipv4(ip) => ip,
                                _ => return Err(SysError::EAFNOSUPPORT),
                            };
                            let src_ip = match unsafe { LOCAL_IP } {
                                IpAddress::Ipv4(ip) => ip,
                                _ => return Err(SysError::EAFNOSUPPORT),
                            };
                            // ep 是目标 IP 地址（Endpoint）
                            // 构造 IPv4 头
                            let mut ipv4_header = [0u8; 20];
                            ipv4_header[0] = (4 << 4) | 5; // 版本(4) + 首部长度(5*4=20字节)
                            ipv4_header[1] = 0; // TOS
                            let total_len = (ipv4_header.len() + data.len()) as u16;
                            ipv4_header[2..4].copy_from_slice(&total_len.to_be_bytes());
                            ipv4_header[4..6].copy_from_slice(&0u16.to_be_bytes()); // ID
                            ipv4_header[6..8].copy_from_slice(&0u16.to_be_bytes()); // Flags+Frag offset
                            ipv4_header[8] = 64; // TTL
                            ipv4_header[9] = 1; // Protocol: ICMP
                            ipv4_header[12..16].copy_from_slice(&src_ip.octets());
                            ipv4_header[16..20].copy_from_slice(&dst_ip.octets());
                            // 计算 IPv4 头校验和
                            let checksum = ipv4_checksum(&ipv4_header);
                            ipv4_header[10..12].copy_from_slice(&checksum.to_be_bytes());

                            let final_buf = [ipv4_header.as_ref(), data].concat();
                            socket.send_slice(&final_buf).map_err(|_| SysError::EBADF);
                            return Ok(data.len());
                        }

                        socket.send_slice(&final_buf).map_err(|_| SysError::EBADF);
                        Ok(data.len())
                    } else {
                        socket.register_send_waker(&waker);
                        Err(SysError::EAGAIN)
                    }
                })
            })
            .await;
        SOCKET_SET.poll_interfaces();
        ret
    }

    pub async fn recv(&self, data: &mut [u8]) -> SockResult<usize> {
        let handle = self.handle.exclusive_access().unwrap();
        let waker = get_waker().await;
        let ret = self
            .block_on(|| {
                SOCKET_SET.with_socket_mut::<smoltcp::socket::raw::Socket, _, _>(handle, |socket| {
                    if let Ok(pkt) = socket.recv() {
                        let len = pkt.len().min(data.len());
                        data[..len].copy_from_slice(&pkt[..len]);
                        Ok(len)
                    } else {
                        socket.register_recv_waker(&waker);
                        Err(SysError::EAGAIN)
                    }
                })
            })
            .await;
        SOCKET_SET.poll_interfaces();
        ret
    }
    async fn block_on<F, R>(&self, mut f: F) -> SockResult<R>
    where
        F: FnMut() -> SockResult<R>,
    {
        loop {
            let timestamp = SOCKET_SET.poll_interfaces();
            let ret = f();
            SOCKET_SET.check_poll(timestamp);
            match ret {
                Ok(r) => return Ok(r),
                Err(SysError::EAGAIN) => {
                    log::info!("[RawSocket::block_on] handle, EAGAIN, suspend now");
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

    pub fn shutdown(&self) -> SockResult<()> {
        SOCKET_SET.remove(self.handle.exclusive_access().unwrap());
        let timestamp = SOCKET_SET.poll_interfaces();
        SOCKET_SET.check_poll(timestamp);
        Ok(())
    }
}

fn build_icmp_echo_request(id: u16, seq: u16, payload: &[u8]) -> Vec<u8> {
    use byteorder::{ByteOrder, NetworkEndian};

    let mut packet = vec![0u8; 8 + payload.len()];
    packet[0] = 8; // Type: Echo Request
    packet[1] = 0; // Code: 0
    NetworkEndian::write_u16(&mut packet[4..6], id);
    NetworkEndian::write_u16(&mut packet[6..8], seq);
    packet[8..].copy_from_slice(payload);

    // 计算 checksum
    let checksum = icmp_checksum(&packet);
    NetworkEndian::write_u16(&mut packet[2..4], checksum);

    packet
}

fn icmp_checksum(data: &[u8]) -> u16 {
    let mut sum = 0u32;
    let mut chunks = data.chunks_exact(2);
    for chunk in &mut chunks {
        sum += u16::from_be_bytes([chunk[0], chunk[1]]) as u32;
    }
    if let Some(last) = chunks.remainder().first().map(|&x| [x]) {
        sum += (last[0] as u32) << 8;
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}

fn ipv4_checksum(header: &[u8]) -> u16 {
    assert!(header.len() % 2 == 0, "ipv4 header length must be even");
    let mut sum: u32 = 0;
    let mut i = 0;
    while i < header.len() {
        let hi = (header[i] as u16) << 8;
        let lo = header[i + 1] as u16;
        sum += (hi | lo) as u32;
        i += 2;
    }
    // fold carries
    while (sum >> 16) != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !(sum as u16)
}