//! uart driver
//! adapt from Phoenix
#![allow(dead_code)]
use crate::{devices::CharDevice, utils::suspend_now};

use super::{Serial, UartDriver};

// the UART control registers.
// some have different meanings for
// read vs write.
// http://byterunner.com/16550.html
const RHR: usize = 0; // receive holding register (for input bytes)
const THR: usize = 0; // transmit holding register (for output bytes)
const IER: usize = 1; // interrupt enable register
const FCR: usize = 2; // FIFO control register
const ISR: usize = 2; // interrupt status register
const LCR: usize = 3; // line control register
const MCR: usize = 4; // modem control register
const LSR: usize = 5; // line status register

bitflags! {
    /// Interrupt enable flags
    struct IntEnFlags: u8 {
        const RECEIVED = 1;
        const SENT = 1 << 1;
        const ERRORED = 1 << 2;
        const STATUS_CHANGE = 1 << 3;
        // 4 to 7 are unused
    }
}

bitflags! {
    /// Line status flags
    struct LineStsFlags: u8 {
        const INPUT_FULL = 1;
        // 1 to 4 unknown
        const OUTPUT_EMPTY = 1 << 5;
        // 6 and 7 unknown
    }
}

/// UART driver
#[derive(Debug)]
pub struct Uart {
    /// UART MMIO base address
    mmio_base_vaddr: usize,
    clock_frequency: u32,
    baud_rate: u32,
    reg_io_width: usize,
    reg_shift: usize,
    is_snps: bool,
}

pub const UART_BUF_LEN: usize = 512;
pub const UART_BAUD_RATE: usize = 115200;

impl Uart {
    /// Creates a new UART interface on the given memory mapped address.
    ///
    /// This function is unsafe because the caller must ensure that the given
    /// base address really points to a serial port device.
    //#[rustversion::attr(since(1.61), const)]
    pub unsafe fn new(
        mmio_base_vaddr: usize,
        clock_frequency: usize,
        baud_rate: usize,
        reg_io_width: usize,
        reg_shift: usize,
        is_snps: bool,
    ) -> Self {
        Self {
            mmio_base_vaddr,
            clock_frequency: clock_frequency as u32,
            baud_rate: baud_rate as u32,
            reg_io_width,
            reg_shift,
            is_snps,
        }
    }

    /// Initializes the memory-mapped UART.
    pub fn init(&mut self) {
        match self.reg_io_width {
            1 => self.init_u8(),
            4 => self.init_u32(),
            _ => {
                panic!("Unsupported UART register width");
            }
        }
    }

    /// Sends a byte on the serial port.
    pub fn send(&mut self, c: u8) {
        match self.reg_io_width {
            1 => self.send_u8(c),
            4 => self.send_u32(c),
            _ => {
                panic!("Unsupported UART register width");
            }
        }
    }

    fn init_u8(&mut self) {
        let reg = self.mmio_base_vaddr as *mut u8;

        unsafe {
            // Disable Interrupt
            reg.byte_add(IER << self.reg_shift).write_volatile(0x00);

            // Enable DLAB
            // Enter a setting mode to set baud rate
            reg.byte_add(LCR << self.reg_shift).write_volatile(0x80);

            // Set baud rate
            let divisor = self.clock_frequency / (16 * self.baud_rate);
            reg.byte_add(0 << self.reg_shift)
                .write_volatile(divisor as u8);
            reg.byte_add(1 << self.reg_shift)
                .write_volatile((divisor >> 8) as u8);

            // Disable DLAB and set data word length to 8 bits
            // Leave setting mode
            reg.byte_add(LCR << self.reg_shift).write_volatile(0x03);

            // Enable FIFO
            reg.byte_add(FCR << self.reg_shift).write_volatile(0x00);

            // No modem control
            reg.byte_add(MCR << self.reg_shift).write_volatile(0x00);

            // Enable interrupts now
            reg.byte_add(IER << self.reg_shift).write_volatile(0x01);
        }
    }

    fn init_u32(&mut self) {
        let reg = self.mmio_base_vaddr as *mut u32;

        unsafe {
            // Disable Interrupt
            reg.byte_add(IER << self.reg_shift).write_volatile(0x00);

            // Enable DLAB
            // Enter a setting mode to set baud rate
            reg.byte_add(LCR << self.reg_shift).write_volatile(0x80);

            // Set baud rate
            let divisor = self.clock_frequency / (16 * self.baud_rate);
            reg.byte_add(0 << self.reg_shift)
                .write_volatile(divisor & 0xff);
            reg.byte_add(1 << self.reg_shift)
                .write_volatile((divisor >> 8) & 0xff);

            // Disable DLAB and set data word length to 8 bits
            // Leave setting mode
            reg.byte_add(LCR << self.reg_shift).write_volatile(0x03);

            // Enable FIFO
            reg.byte_add(FCR << self.reg_shift).write_volatile(0x01);

            // No modem control
            reg.byte_add(MCR << self.reg_shift).write_volatile(0x00);

            // Enable interrupts now
            reg.byte_add(IER << self.reg_shift).write_volatile(0x01);
        }
        log::info!("IER register: 0b{:b}", unsafe {
            reg.byte_add(IER << self.reg_shift).read_volatile()
        });
    }

    fn line_sts_u8(&self) -> LineStsFlags {
        let ptr = self.mmio_base_vaddr as *mut u8;
        unsafe {
            LineStsFlags::from_bits_truncate(ptr.byte_add(LSR << self.reg_shift).read_volatile())
        }
    }

    fn line_sts_u32(&self) -> LineStsFlags {
        let ptr = self.mmio_base_vaddr as *mut u32;
        unsafe {
            LineStsFlags::from_bits_truncate(
                ptr.byte_add(LSR << self.reg_shift).read_volatile() as u8
            )
        }
    }

    /// Sends a byte on the serial port.
    pub fn send_u8(&mut self, c: u8) {
        let ptr = self.mmio_base_vaddr as *mut u8;
        unsafe {
            match c {
                8 | 0x7F => {
                    // For backspace and delete char
                    wait_for!(self.line_sts_u8().contains(LineStsFlags::OUTPUT_EMPTY));
                    ptr.byte_add(THR << self.reg_shift).write_volatile(8);
                    wait_for!(self.line_sts_u8().contains(LineStsFlags::OUTPUT_EMPTY));
                    ptr.byte_add(THR << self.reg_shift).write_volatile(b' ');
                    wait_for!(self.line_sts_u8().contains(LineStsFlags::OUTPUT_EMPTY));
                    ptr.byte_add(THR << self.reg_shift).write_volatile(8);
                }
                _ => {
                    // Wait until previous data is flushed
                    wait_for!(self.line_sts_u8().contains(LineStsFlags::OUTPUT_EMPTY));
                    // Write data
                    ptr.byte_add(THR << self.reg_shift).write_volatile(c);
                }
            }
        }
    }

    pub fn send_u32(&mut self, c: u8) {
        let ptr = self.mmio_base_vaddr as *mut u32;
        unsafe {
            match c {
                8 | 0x7F => {
                    wait_for!(self.line_sts_u32().contains(LineStsFlags::OUTPUT_EMPTY));
                    ptr.byte_add(THR << self.reg_shift).write_volatile(8);
                    wait_for!(self.line_sts_u32().contains(LineStsFlags::OUTPUT_EMPTY));
                    ptr.byte_add(THR << self.reg_shift)
                        .write_volatile(b' '.into());
                    wait_for!(self.line_sts_u32().contains(LineStsFlags::OUTPUT_EMPTY));
                    ptr.byte_add(THR << self.reg_shift).write_volatile(8);
                }
                _ => {
                    // Wait until previous data is flushed
                    wait_for!(self.line_sts_u32().contains(LineStsFlags::OUTPUT_EMPTY));
                    // Write data
                    ptr.byte_add(THR << self.reg_shift).write_volatile(c.into());
                }
            }
        }
    }

    /// Receives a byte on the serial port.
    pub fn receive(&mut self) -> u8 {
        let ptr = self.mmio_base_vaddr as *mut u32;
        unsafe {
            wait_for!(self.line_sts_u8().contains(LineStsFlags::INPUT_FULL));
            if self.is_snps {
                // Clear busy detect interrupt
                // by reading UART status register. see Synopsys documentation
                // hard-coded register offset
                ptr.byte_add(31 << self.reg_shift).read_volatile();
            }
            ptr.byte_add(0).read_volatile() as u8
        }
    }
}

impl UartDriver for Uart {
    fn init(&mut self) {
        self.init()
    }

    fn putc(&mut self, byte: u8) {
        self.send(byte)
    }

    fn getc(&mut self) -> u8 {
        self.receive()
    }

    fn poll_in(&self) -> bool {
        match self.reg_io_width {
            1 => self.line_sts_u8().contains(LineStsFlags::INPUT_FULL),
            4 => self.line_sts_u32().contains(LineStsFlags::INPUT_FULL),
            _ => unimplemented!(),
        }
    }

    fn poll_out(&self) -> bool {
        match self.reg_io_width {
            1 => self.line_sts_u8().contains(LineStsFlags::OUTPUT_EMPTY),
            4 => self.line_sts_u32().contains(LineStsFlags::OUTPUT_EMPTY),
            _ => unimplemented!(),
        }
    }
}

macro_rules! wait_for {
    ($cond:expr) => {{
        let mut timeout = 10000000;
        while !$cond && timeout > 0 {
            core::hint::spin_loop();
            timeout -= 1;
        }
    }};
}
pub(crate) use wait_for;