#![no_std]
#![no_main]

const AF_INET: u16 = 2;
const SOCK_STREAM: u16 = 1;
const IP_ADDR: u32 = 0x0a00020f; // 10.0.2.15 的网络字节序
const PORT: u16 = 0x15b3;        // 5555 端口的网络字节序

extern crate user_lib;

use core::panic;

use user_lib::*;


#[no_mangle]
fn main() -> ! {
    // 创建服务器套接字
    let sockfd = socket(AF_INET as i32, SOCK_STREAM.into(), 0) ;
    if sockfd < 0 {
        unsafe { exit(1) };
    }
    println!("[SERVER] Socket created (fd = {})", sockfd);
    // 绑定地址和端口
    let server_addr = SockaddrIn::new(IP_ADDR, PORT);
    let bind_res = unsafe {
        bind(
            sockfd as usize,
            &server_addr as *const SockaddrIn ,
            core::mem::size_of_val(&server_addr) as u32,
        )
    };
    if bind_res < 0 {
        close(sockfd as usize);
        exit(1);
    }
    println!("[SERVER] Bound to port {}", PORT);
    // 开始监听
    let listen_res = listen(sockfd as usize, 5) ;
    if listen_res < 0 {
        close(sockfd as usize  ) ;
        exit(1) ;
    }
    println!("[SERVER] Listening on port {}...", PORT);
    // 创建子进程
    let pid = fork() ;
    if pid < 0 {
        close(sockfd as usize) ;
        exit(1) ;
    }

    if pid == 0 {
        // 子进程作为客户端
        client_process();
    } else {
        // 父进程作为服务器
        server_process(sockfd as i32, pid as i32);
    }
}

fn client_process() -> ! {
    // 创建客户端套接字
    let sockfd = socket(AF_INET as i32, SOCK_STREAM as i32, 0) ;
    if sockfd < 0 {
        exit(1) ;
    }
    println!("[CLIENT] Socket created (fd = {})", sockfd);
    // 连接服务器
    let server_addr = SockaddrIn::new(IP_ADDR, PORT);
    let connect_res = 
        connect(
            sockfd as usize,
            &server_addr as *const SockaddrIn,
            core::mem::size_of_val(&server_addr) as u32,
        );
    if connect_res < 0 {
        close(sockfd as usize) ;
        exit(1) ;
    }
    println!("[CLIENT] Connected to server");
    // 测试数据
    let data = b"test";
    
    // 发送数据
    let write_res = write(sockfd as usize, data, data.len()) ;
    if write_res != data.len() as isize {
        close(sockfd as usize) ;
        exit(1) ;
    }
    println!("[CLIENT] Sent data to server");
    // 接收响应
    let mut buf = [0u8; 4];
    let read_res = read(sockfd as usize, &mut buf ) ;
    if read_res != data.len() as isize {
        close(sockfd as usize) ;
        exit(1) ;
    }
    println!("[CLIENT] Received data from server: {}", core::str::from_utf8(&buf).unwrap());
    // 验证数据
    if &buf != data {
        close(sockfd as usize) ;
        exit(1) ;
    }

    close(sockfd as usize) ;
    exit(0) ;
}

fn server_process(sockfd: i32, pid: i32) -> ! {
    // 接受客户端连接
    let mut client_addr = SockaddrIn::new(0, 0);
    let mut addr_len = core::mem::size_of_val(&client_addr) as u32;
    let client_sockfd = accept(
            sockfd as usize,
            &mut client_addr as *mut SockaddrIn,
            &mut addr_len as *mut u32,
        );
    if client_sockfd < 0 {
        close(sockfd as usize) ;
        exit(1) ;
    }
    println!("[SERVER] Accepted connection from client (fd = {})", client_sockfd);

    // 处理数据
    let mut buf = [0u8; 4];
    let len = buf.len() as isize;
    let read_res = read(client_sockfd as usize, &mut buf);
    if read_res != buf.len() as isize {
        close(client_sockfd as usize) ;
        close(sockfd as usize) ;
        exit(1) ;
    }
    println!("[SERVER] Received data from client: {}", core::str::from_utf8(&buf).unwrap());

    // 回显数据
    let write_res = write(client_sockfd as usize, &mut buf, len as usize) ;
    if write_res != buf.len() as isize {
        close(client_sockfd as usize) ;
        close(sockfd as usize) ;
        exit(1) ;
    }

    // 清理资源
        close(client_sockfd as usize);
        close(sockfd as usize);

    // 等待子进程退出
    let mut status = 0;
    let wait_res = waitpid(pid as usize, &mut status) ;
    if wait_res < 0 {
        exit(1) ;
    }

    // 检查子进程状态
    if status != 0 {
        exit(1) ;
    }

    exit(0) ;
}