//! char device: serial
//! adapt from Phoenix
#![allow(dead_code)]
pub mod uart;

use core::task::Waker;

use alloc::{boxed::Box, collections::vec_deque::VecDeque, string::ToString, sync::Arc};
use async_trait::async_trait;
use hal::constant::{Constant, ConstantsHal};
use lazy_static::lazy_static;
use uart::{Uart, UART_BAUD_RATE, UART_BUF_LEN};

use crate::{devices::{CharDevice, DevId, Device, DeviceMajor, DeviceMeta, DeviceType, DEVICE_MANAGER}, sync::{mutex::SpinNoIrqLock, UPSafeCell}, utils::{get_waker, suspend_now, RingBuffer}, with_methods};

lazy_static! {
    /// WARNING: should only be called after devices manager finish init
    pub static ref UART0: Arc<dyn CharDevice> = {
        let serial = DEVICE_MANAGER.lock()
        .find_dev_by_major(DeviceMajor::Serial)
        .into_iter()
        .map(|device| device.as_char().unwrap())
        .next()
        .unwrap();
        serial.clone()
    };
}

pub trait UartDriver: Send + Sync {
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
    pub fn new(mmio_base: usize, mmio_size: usize, irq_no: usize, driver: Box<dyn UartDriver>) -> Self {
        let meta = DeviceMeta {
            dev_id: DevId {
                major: DeviceMajor::Serial,
                minor: 0,
            },
            name: "serial".to_string(),
            need_mapping: true,
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

impl Device for Serial {
    fn meta(&self) -> &DeviceMeta {
        &self.meta
    }

    fn init(&self) {
        unsafe { &mut *self.uart.get() }.as_mut().init();
    }

    fn handle_irq(&self) {
        let uart = self.uart();
        self.with_mut_inner(|inner| {
            while uart.poll_in() {
                let byte = uart.getc();
                log::trace!(
                    "Serial interrupt handler got byte: {}, ascii: {byte}",
                    core::str::from_utf8(&[byte]).unwrap()
                );
                if inner.read_buf.enqueue(byte).is_none() {
                    break;
                }
            }
            // Round Robin
            if let Some(waiting) = inner.pollin_queue.pop_front() {
                waiting.wake();
            }
        });
    }

    fn as_char(self: Arc<Self>) -> Option<Arc<dyn CharDevice>> {
        Some(self)
    }
}
