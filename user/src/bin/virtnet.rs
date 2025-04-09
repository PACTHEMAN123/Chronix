#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
use core::panic;

use user_lib::*;

#[no_mangle]
pub fn main() -> ! {
    println!("[QEMU SERVER] Starting network test...");
    let host_ip = parse_ipv4(HOST_IP).expect("Failed to parse host IP");
    let sockfd = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    if sockfd < 0 {
        panic!("[QEMU SERVER] socket failed");
    }
    println!("[QEMU SERVER] Socket created (fd = {})", sockfd);
    let addr = SockaddrIn {
        sin_family: AF_INET as u16,
        sin_port: SERVER_PORT.to_be(),
        sin_addr: host_ip,
        sin_zero: [0; 8],
    };
    if bind(sockfd as usize, &addr, core::mem::size_of_val(&addr) as u32) < 0 {
        panic!("[QEMU SERVER] bind failed");
    }
    println!("[QEMU SERVER] Socket bound to port {}", SERVER_PORT);
    if listen(sockfd as usize, 5) < 0 {
        panic!("[QEMU SERVER] connect failed");
    }
    println!("[QEMU SERVER] Listening on port {}...", SERVER_PORT);
    let mut client_addr = unsafe{core::mem::zeroed()};
    let mut client_len = core::mem::size_of::<SockaddrIn>() as u32;
    let connfd = accept(sockfd as usize, &mut client_addr , &mut client_len );
    if connfd < 0 {
        panic!("Accept failed");
    }
    send_verify(connfd as i32);
    close(sockfd as usize);
    println!("[QEMU SERVER] Test finished");
    exit(0);
}

fn send_verify(sockfd: i32) {
    let mut buf = [0u8; 1024];
    loop {
        let n = read(sockfd as usize, &mut buf) ;
        println!("[DEBUG] Read returned {}", n);
        
        if n <= 0 {
            if n < 0 {
                println!("[ERROR] Read error: {}", n);
            } else {
                println!("[INFO] Connection closed by peer");
            }
            break;
        }
    
        let write_res = write(sockfd as usize, TEST_DATA, TEST_DATA.len()) ;
        println!("[DEBUG] Write returned {}", write_res);
        
        if write_res != n {
            println!("[ERROR] Partial write: expected {}, actual {}", n, write_res);
            break;
        }
    }
}

const HOST_IP: &str = "10.0.2.2"; // the IP address of the qemu host
const SERVER_PORT: u16 = 8888;
const AF_INET: i32 = 2;
const SOCK_STREAM: i32 = 1;
const IPPROTO_TCP: i32 = 6;
const TEST_DATA: &[u8] = b"QEMU TCP Test Data";