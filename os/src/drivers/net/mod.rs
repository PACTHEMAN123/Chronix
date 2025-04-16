#[allow(unused)]
pub mod virtio_net;
pub mod loopback;
use core::{mem, ptr::NonNull};

use alloc::{boxed::Box, string::ToString};
use fatfs::info;
use spin::relax::Loop;
use virtio_drivers::transport::{self, mmio::{MmioTransport, VirtIOHeader}, Transport};
use hal::constant::{Constant, ConstantsHal};
use crate::{devices::NetDevice, drivers::net::virtio_net::VirtIoNetDev};
use loopback::LoopbackDevice;
const VIRTIO1: usize = 0x10002000 | Constant::KERNEL_ADDR_SPACE.start;
pub fn init_network_device() -> (Box<dyn NetDevice>,bool) {
    let _devflag = false;
    #[cfg(feature = "net")]
    let _devflag = true;
    log::info!("net device flag: {}",_devflag);
    let dev:Box<dyn NetDevice> = if _devflag {
        let header = NonNull::new(VIRTIO1 as *mut VirtIOHeader).unwrap();
        let transport = unsafe{MmioTransport::new(header, 4096).unwrap()};
        VirtIoNetDev::new(transport).unwrap()
    }else {
        LoopbackDevice::new()
    };
    (dev,_devflag)
}