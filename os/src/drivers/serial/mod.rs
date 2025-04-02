//! char device: serial
//! adapt from Phoenix
#![allow(dead_code)]
pub mod uart;

use core::task::Waker;

use alloc::{boxed::Box, collections::vec_deque::VecDeque, string::ToString, sync::Arc};
use async_trait::async_trait;
use hal::{board::{UART_CLK_FEQ, UART_IRQ_NUM, UART_MMIO_BASE_PADDR, UART_MMIO_SIZE, UART_REG_IO_WIDTH, UART_REG_SHIFT}, constant::{Constant, ConstantsHal}};
use lazy_static::lazy_static;
use uart::{Uart, UART_BAUD_RATE, UART_BUF_LEN};

use crate::{devices::{CharDevice, DevId, DeviceMajor, DeviceMeta, DeviceType}, sync::{mutex::SpinNoIrqLock, UPSafeCell}, utils::{get_waker, suspend_now, RingBuffer}, with_methods};

lazy_static! {
    pub static ref UART0: Arc<dyn CharDevice> = {
        let size = UART_MMIO_SIZE;
        let base_paddr = UART_MMIO_BASE_PADDR;
        let base_vaddr = base_paddr | Constant::KERNEL_ADDR_SPACE.start;
        let irq_no = UART_IRQ_NUM;
        let clk_feq = UART_CLK_FEQ;
        let baud_rate = UART_BAUD_RATE;
        let reg_io_width = UART_REG_IO_WIDTH;
        let reg_shift = UART_REG_SHIFT;

        let uart = unsafe { Uart::new(
            base_vaddr,
            clk_feq,
            baud_rate,
            reg_io_width,
            reg_shift,
            false,
        )};
        log::info!("mapping uart mmio paddr {:x} to vaddr {:x}", base_paddr, base_vaddr);
        Arc::new(Serial::new(base_paddr, size, irq_no, Box::new(uart)))
    };
}

trait UartDriver: Send + Sync {
    fn init(&mut self);
    fn putc(&mut self, byte: u8);
    fn getc(&mut self) -> u8;
    fn poll_in(&self) -> bool;
    fn poll_out(&self) -> bool;
}

pub struct Serial {
    meta: DeviceMeta,
    uart: UPSafeCell<Box<dyn UartDriver>>,
    inner: SpinNoIrqLock<SerialInner>,
}

pub struct SerialInner {
    read_buf: RingBuffer,
    /// Hold wakers of pollin tasks.
    pollin_queue: VecDeque<Waker>,
}

unsafe impl Send for Serial {}
unsafe impl Sync for Serial {}

impl Serial {
    fn new(mmio_base: usize, mmio_size: usize, irq_no: usize, driver: Box<dyn UartDriver>) -> Self {
        let meta = DeviceMeta {
            dev_id: DevId {
                major: DeviceMajor::Serial,
                minor: 0,
            },
            name: "serial".to_string(),
            mmio_base,
            mmio_size,
            irq_no: Some(irq_no),
            dtype: DeviceType::Char,
        };

        Self {
            meta,
            uart: UPSafeCell::new(driver),
            inner: SpinNoIrqLock::new(SerialInner {
                read_buf: RingBuffer::new(UART_BUF_LEN),
                pollin_queue: VecDeque::new(),
            }),
        }
    }

    fn uart(&self) -> &mut Box<dyn UartDriver> {
        &mut *self.uart.exclusive_access()
    }

    with_methods!(inner: SerialInner);
}

#[async_trait]
impl CharDevice for Serial {
    async fn read(&self, buf: &mut [u8]) -> usize {
        while !self.poll_in().await {
            suspend_now().await
        }
        let mut len = 0;
        self.with_mut_inner(|inner| {
            len = inner.read_buf.read(buf);
        });
        let uart = self.uart();
        while uart.poll_in() && len < buf.len() {
            let c = uart.getc();
            buf[len] = c;
            len += 1;
        }
        len
    }

    async fn write(&self, buf: &[u8]) -> usize {
        let uart = self.uart();
        for &c in buf {
            uart.putc(c)
        }
        buf.len()
    }

    async fn poll_in(&self) -> bool {
        let uart = self.uart();
        let waker = get_waker().await;
        self.with_mut_inner(|inner| {
            if uart.poll_in() || !inner.read_buf.is_empty() {
                return true;
            }
            inner.pollin_queue.push_back(waker);
            false
        })
    }

    // TODO:
    async fn poll_out(&self) -> bool {
        true
    }
}

