#![no_std]
#![no_main]

extern crate user_lib;

use core::panic;

use user_lib::*;

const AF_INET: i32 = 2;
const SOCK_STREAM: i32 = 1;
const IPPROTO_TCP: i32 = 6;

const TEST_PORT: u16 = 4444;
const TEST_ADDR: u32 = 0x7f000001; // 127.0.0.1
const TEST_DATA: &[u8] = b"Hello, TCP Loopback!";

#[no_mangle]
fn main() -> ! {
    if fork() == 0{
        // child process
        println!("[Client] Child process started");
        client()
    }else {
        println!("[Server] Parent process started");
        server();
    }
}

fn server() -> ! {
    let sockfd = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP) ;
    if sockfd  < 0 {
        panic!("server: socket failed");
    }
    println!("[SERVER] Socket created (fd = {})", sockfd);
    let addr = SockaddrIn{
        sin_family: AF_INET as u16,
        sin_port: TEST_PORT.to_be(),
        sin_addr: TEST_ADDR.to_be(),
        sin_zero: [0; 8],
    };
    if bind(sockfd as usize, &addr, size_of::<SockaddrIn>() as u32) < 0 {
        close(sockfd as usize);
        panic!("server: bind failed");
    }
    println!("[Server] Bound to port {}", TEST_PORT);
    // start listen
    if listen(sockfd as usize, 1) < 0 {
        close(sockfd as usize);
        panic!("server: listen failed");
    }
    println!("[Server] Listening on port {}...", TEST_PORT);
    // accept connection
    let mut client_addr = unsafe{core::mem::zeroed()};
    let mut client_len = core::mem::size_of::<SockaddrIn>() as u32;
    let client_fd = accept(sockfd as usize, &mut client_addr , &mut client_len);
    println!("get a client_fd {}", client_fd);
    if client_fd < 0 {
        close(sockfd as usize);
        panic!("server: accept failed");
    }
    println!("[SERVER]: accpet client_fd {}",client_fd);
    let mut buf = [0u8; 1024];
    loop {
        let n = read(client_fd as usize, buf.as_mut_slice());
        if n < 0 {
            println!("[Server] Failed to read data from client, closing connection, n is {}",n);
            close(client_fd as usize);
            panic!("server: read failed");
        }
        println!("[Server] Received {} bytes, echoing back...", n);
        let write_res = write(client_fd as usize, &buf, n as usize);
        if write_res != n {
            panic!("[Server] Failed to write all bytes, {}/{}", write_res, n);
        }
        for _ in 0..100 {
            delay();
        }
        break;
    }
    println!("[Server] exiting");
    exit(0);
} 

fn client() -> ! {
    delay();
    let sockfd = socket(AF_INET,SOCK_STREAM,IPPROTO_TCP);
    if sockfd < 0 {
        panic!("client: socket failed");
    }
    println!("[Client] Socket created (fd = {})", sockfd);
    // connect the server
    let addr = SockaddrIn{
        sin_family: AF_INET as u16,
        sin_port: TEST_PORT.to_be(),
        sin_addr: TEST_ADDR.to_be(),
        sin_zero: [0; 8],
    };
    let connect_res = connect(sockfd as usize, &addr, core::mem::size_of::<SockaddrIn>() as u32);
    if connect_res < 0 {
        close(sockfd as usize);
        panic!("client: connect failed");
    }
    println!("[Client] Connected to server");
    // send data
    write(sockfd as usize, TEST_DATA, TEST_DATA.len());
    println!("[Client] Sent bytes ");
    // receive data
    let mut buf = [0u8; 1024];
    let n = read(sockfd as usize, buf.as_mut_slice());
    println!("[client ]read {} bytes",n);
    if n < 0 {
        panic!("client: read failed");
    }
    close(sockfd as usize);
    exit(0);
}


fn delay() {
    let mut count = 1_000_000;
    while count > 0 {
        count -= 1;
    }
}
