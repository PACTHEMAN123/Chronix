use crate::util::mutex::Mutex;

#[cfg(target_arch="loongarch64")]
const UART_ADDR: usize = 0x8000_0000_1fe0_01e0;

// ugly: os kernel need to map this address
#[cfg(target_arch="riscv64")]
const UART_ADDR: usize = 0xffff_ffc0_1000_0000;

static COM1: Mutex<Uart> = Mutex::new(Uart::new(UART_ADDR));

pub struct Uart {
    base_address: usize,
}

impl Uart {
    pub const fn new(base_address: usize) -> Self {
        Uart { base_address }
    }

    pub fn putchar(&mut self, c: u8) {
        let ptr = self.base_address as *mut u8;
        loop {
            unsafe {
                if ptr.add(5).read_volatile() & (1 << 5) != 0 {
                    break;
                }
            }
        }
        unsafe {
            ptr.add(0).write_volatile(c);
        }
    }

    pub fn getchar(&mut self) -> Option<u8> {
        let ptr = self.base_address as *mut u8;
        unsafe {
            if ptr.add(5).read_volatile() & 1 == 0 {
                // The DR bit is 0, meaning no data
                None
            } else {
                // The DR bit is 1, meaning data!
                Some(ptr.add(0).read_volatile())
            }
        }
    }
}

pub fn console_putchar(c: usize) {
    let c = c as u8;
    let mut locked = COM1.lock();
    if c == b'\n' {
        locked.putchar(b'\r');
    }
    locked.putchar(c)
}

pub fn console_getchar() -> usize {
    let mut locked = COM1.lock();
    loop { 
        if let Some(c) = locked.getchar() {
            break c as usize;
        }
    }
}