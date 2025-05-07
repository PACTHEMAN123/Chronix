#![no_std]
#![no_main]

extern crate user_lib;

use core::{panic, ptr};

use user_lib::*;

const AF_INET: i32 = 2; // IPv4
const SOCK_DGRAM: i32 = 2; // UDP

const TEST_PORT: u16 = 0x15b3;
const TEST_ADDR: u32 = 0x7f000001; // 10.0.2.15

fn udp_server() {
    println!("udp_server: starting up");
    let sockfd = socket(AF_INET, SOCK_DGRAM, 0);
    if sockfd < 0 {
        panic!("udp_server: socket failed, wrong sockfd");
    }

    let server_addr = SockaddrIn {
        sin_family: AF_INET as u16,
        sin_port: TEST_PORT.to_be(),
        sin_addr: TEST_ADDR.to_be(),
        sin_zero: [0; 8],
    };

    if bind(sockfd as usize, &server_addr as *const SockaddrIn, core::mem::size_of::<SockaddrIn>() as u32) < 0 {
        panic!("udp_server: bind failed");
    }

    let mut buffer = [0u8; 1024];
    let mut client_addr: SockaddrIn = unsafe{core::mem::zeroed()};
    let mut addr_len = core::mem::size_of::<SockaddrIn>() as u32;
    let buffer_len = buffer.len();
    let recv_len = recvfrom(sockfd as usize, buffer.as_mut_slice(), buffer_len, 0, &mut client_addr , &mut addr_len );
    if recv_len < 0 {
        panic!("udp_server: recvfrom failed, recv_len = {}", recv_len);
    }
    println!("udp_server: received message from {}: {}", client_addr.sin_addr, core::str::from_utf8(&buffer[..recv_len as usize]).unwrap());
    let send_len = sendto(sockfd as usize, buffer.as_slice(), recv_len as usize, 0, &client_addr , addr_len);
    if send_len < 0 {
        panic!("udp_server: sendto failed, send_len = {}", send_len);
    }
    println!("udp_server: sent message to {}: {}", client_addr.sin_addr, core::str::from_utf8(&buffer[..recv_len as usize]).unwrap());
    for _ in 0..100 {
        delay();
    }
}

fn udp_client() {
    delay();
    println!("udp_client: starting up");
    let sockfd = socket(AF_INET, SOCK_DGRAM, 0);
    if sockfd < 0 {
        panic!("udp_client: socket failed");
    }

    let server_addr = SockaddrIn {
        sin_family: AF_INET as u16,
        sin_port: TEST_PORT.to_be(),
        sin_addr: TEST_ADDR.to_be(),
        sin_zero: [0; 8],
    };

    let message = b"Hello from client!";
    println!("udp_client:addr_len is {}",size_of::<SockaddrIn>() as u32);
    let send_len = sendto(sockfd as usize, message, message.len(), 0, &server_addr, size_of::<SockaddrIn>() as u32);
    if send_len < 0 {
        panic!("udp_client: sendto failed, send_len = {}",send_len);
    }
    println!("udp_client: about to recvfrom");
    let mut buffer = [0u8; 1024];
    let buffer_len = buffer.len();
    let mut client_addr = unsafe{core::mem::zeroed()};
    let mut client_len = core::mem::size_of::<SockaddrIn>() as u32;
    let recv_len = recvfrom(sockfd as usize, buffer.as_mut_slice(),buffer_len, 0, &mut client_addr, &mut client_len);
    println!("udp_client: recv_len is {}",recv_len);
    if recv_len < 0 {
        panic!("udp_client: recvfrom failed, recv_len = {}",recv_len);
    }
    println!("udp_client: received message: {}", core::str::from_utf8(&buffer[..recv_len as usize]).unwrap());
    delay();
    exit(0);
}

fn delay() {
    let mut count = 1_000_000;
    while count > 0 {
        count -= 1;
    }
}

#[no_mangle]
fn main() -> ! {
    let pid = fork();
    if pid == 0 {
        udp_client();
    } else {
        udp_server();
    }
    exit(0)
}