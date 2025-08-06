//! Multi-Media Card Drivers
//! 

mod register;
mod dma;

use core::mem;

use alloc::sync::Arc;
use alloc::{string::ToString, vec::Vec};
use alloc::vec;
use hal::addr::PhysPageNumHal;
use hal::allocator::FrameAllocatorTrackerExt;
use hal::constant::{Constant, ConstantsHal};

use crate::config::BLOCK_SIZE;
use crate::devices::buffer_cache::BufferCache;
use crate::devices::{BlockDevice, Device, DeviceType};
use crate::drivers::block::mmc::dma::Descriptor;
use crate::drivers::block::mmc::register::{CtypeCardWidth, BLKSIZ, BMOD, BYTCNT, CDETECT, CID, CLKDIV, CLKENA, CMD, CMDARG, CTRL, CTYPE, DBADDRL, DBADDRU, IDSTS, PWREN, RESP, RINSTS, STATUS};
use crate::mm::allocator::FrameAllocator;
use crate::sync::mutex::SpinNoIrqLock;
use crate::{devices::{mmio::MmioDeviceDescripter, DevId, DeviceMajor, DeviceMeta}, mm::FrameTracker};


pub struct MMCBlock {
    meta: DeviceMeta,
    fifo_offset: SpinNoIrqLock<usize>,
    frames: SpinNoIrqLock<Vec<FrameTracker>>,
}

unsafe impl Send for MMCBlock {}
unsafe impl Sync for MMCBlock {}

impl Device for MMCBlock {
    fn meta(&self) -> &DeviceMeta {
        &self.meta
    }

    fn init(&self) {
        self.card_init();
    }

    fn handle_irq(&self) {
        // todo
    }

    fn as_blk(self: Arc<Self>) -> Option<Arc<dyn BlockDevice>> {
        Some(self)
    }
}

impl BlockDevice for MMCBlock {
    fn size(&self) -> u64 {
        16 * 1000 * 1000 * 1000
    }

    fn block_size(&self) -> usize {
        BLOCK_SIZE
    }

    fn buffer_cache(&self) -> Option<Arc<BufferCache>> {
        None
    }

    fn direct_read_block(&self, block_id: usize, buf: &mut [u8]) {
        assert!(buf.len() == BLOCK_SIZE);

        let buf_trans: &mut [usize] = unsafe {
            let len = buf.len() / mem::size_of::<usize>();
            core::slice::from_raw_parts_mut(buf.as_ptr() as *mut usize, len)
        };
        log::debug!("reading block {}", block_id);
        // Read one block
        self.set_size(512, 512);
        let cmd = CMD::data_cmd(0, 17); // TODO: card number hard coded to 0
        let cmdarg = CMDARG::from(block_id as u32);
        self.send_cmd(cmd, cmdarg, Some(buf_trans), true)
            .expect("Error sending command");
    }

    fn direct_write_block(&self, block_id: usize, buf: &[u8]) {
        assert!(buf.len() == BLOCK_SIZE);

        #[allow(mutable_transmutes)]
        let buf = unsafe { core::mem::transmute(buf) };
        log::debug!("writing block {}", block_id);
        self.set_size(512, 512);
        // CMD24 single block write
        let cmd = CMD::data_cmd(0, 24).with_read_write(true); // TODO: card number hard coded to 0

        let cmdarg = CMDARG::from(block_id as u32);
        self.send_cmd(cmd, cmdarg, Some(buf), false)
            .expect("Error sending command");
    }
}

impl MMCBlock {
    pub fn new(mmio_dev: MmioDeviceDescripter) -> Self {
        Self {
            meta: DeviceMeta { dev_id: DevId {
                major: DeviceMajor::Block,
                minor: 3,
            }, 
            name: "sda0".to_string(),
            need_mapping: true,
            mmio_ranges: vec![mmio_dev.mmio_region],
            irq_no: None, 
            dtype: DeviceType::Block 
            },
            fifo_offset: SpinNoIrqLock::new(0x600),
            frames: SpinNoIrqLock::new(Vec::new()),
        }
    }

    pub fn card_init(&self) {
        log::info!("[MMC] start to init sdio");
        log::info!("[MMC] Card detect: {:b}", self.card_detect());
        log::info!("[MMC] Power enable: {:b}", self.power_enable().power_enable());
        log::info!("[MMC] Clock enable: {:b}", self.clock_enable().cclk_enable());
        log::info!("[MMC] Card0 width: {:?}", self.card_width(0));
        log::info!("Control register: {:?}", self.control_reg());
        log::info!("DMA enabled: {}", self.dma_enabled());
        log::info!(
            "Descriptor base address: {:x}",
            self.descriptor_base_address()
        );

        let card_idx = 0;
        // 0xAA is check pattern, see https://elixir.bootlin.com/linux/v6.4-rc7/source/drivers/mmc/core/sd_ops.c#L162
        const TEST_PATTERN: u32 = 0xAA;

        // Read clock divider
        log::info!("Read clock divider");
        let base = self.virt_base_addr() as *mut CLKDIV;
        let clkdiv = unsafe { base.byte_add(CLKDIV::offset()).read_volatile() };
        log::info!("Clock divider: {:?}", clkdiv.clks());

        self.reset_clock();
        self.reset_fifo();
        self.set_controller_bus_width(card_idx, CtypeCardWidth::Width1);
        self.set_dma(false); // Disable DMA
        log::info!("Control register: {:?}", self.control_reg());

        let cmd = CMD::reset_cmd0(0);
        self.send_cmd(cmd, CMDARG::empty(), None, false);

        // Voltage check and SDHC 2.0 check
        log::info!("Voltage Check");
        // CMD8
        let cmd = CMD::no_data_cmd(card_idx, 8);
        let cmdarg = CMDARG::from((1 << 8) | TEST_PATTERN);
        let resp = self
            .send_cmd(cmd, cmdarg, None, false)
            .expect("Error sending command");
        if (resp.resp(0) & TEST_PATTERN) == 0 {
            log::warn!("Card {} unusable", card_idx);
        }

        // If card responses, consider it SD

        // Send ACMD41 to power up
        loop {
            // Send CMD55 before ACMD
            let cmd = CMD::no_data_cmd(card_idx, 55);
            let cmdarg = CMDARG::empty();
            self.send_cmd(cmd, cmdarg, None, false);
            let cmd = CMD::no_data_cmd_no_crc(card_idx, 41); // CRC is all 1 bit by design
            let cmdarg = CMDARG::from((1 << 30) | (1 << 24) | 0xFF8000);
            if let Some(resp) = self.send_cmd(cmd, cmdarg, None, false) {
                if resp.ocr() & (1 << 31) != 0 {
                    log::info!("Card {} powered up", card_idx);
                    if resp.ocr() & (1 << 30) != 0 {
                        log::info!("Card {} is high capacity", card_idx);
                    }
                    break;
                }
            }
            spin(100000); // Wait for card to power up
        }

        // CMD2, get CID
        let cmd = CMD::no_data_cmd_no_crc(card_idx, 2).with_response_length(true); // R2 response, no CRC
        if let Some(resp) = self.send_cmd(cmd, CMDARG::empty(), None, false) {
            let cid = CID::from(resp.resps_u128());
            log::info!("CID: {:x?}", cid);
            log::info!(
                "Card Name: {}",
                core::str::from_utf8(&cid.name().to_be_bytes()).unwrap()
            );
        }

        // CMD3, get RCA
        let cmd = CMD::no_data_cmd(card_idx, 3);
        let resp = self
            .send_cmd(cmd, CMDARG::empty(), None, false)
            .expect("Error executing CMD3");
        let rca = resp.resp(0) >> 16; // RCA[31:16]
        log::info!("Card status: {:x?}", resp.resp(0) & 0xFFFF);

        // CMD9, get CSD
        let cmd = CMD::no_data_cmd_no_crc(card_idx, 9).with_response_length(true); // R2 response, no CRC
        let cmdarg = CMDARG::from(rca << 16);
        self.send_cmd(cmd, cmdarg, None, false);

        // CMD7 select card
        let cmd = CMD::no_data_cmd(card_idx, 7);
        let cmdarg = CMDARG::from(rca << 16);
        let _resp = self
            .send_cmd(cmd, cmdarg, None, false)
            .expect("Error executing CMD7");

        log::info!("Current FIFO count: {}", self.fifo_filled_cnt());

        // ACMD51 get bus width
        // Send CMD55 before ACMD
        let cmd = CMD::no_data_cmd(card_idx, 55);
        let cmdarg = CMDARG::from(rca << 16);
        self.send_cmd(cmd, cmdarg, None, false); // RCA is required
        self.set_size(8, 8); // Set transfer size
        let cmd = CMD::data_cmd(card_idx, 51);
        let mut buffer: [usize; 64] = [0; 64]; // 512B
        self.send_cmd(cmd, CMDARG::empty(), Some(&mut buffer), true);
        log::info!("Current FIFO count: {}", self.fifo_filled_cnt());
        let resp = u64::from_be(self.read_fifo::<u64>());
        log::info!("Bus width supported: {:b}", (resp >> 48) & 0xF);

        // CMD16 set block length
        // let cmd = CMD::no_data_cmd(card_idx, 16);
        // let cmdarg = CMDARG::from(512);
        // self.send_cmd(cmd, cmdarg);

        log::info!("Current FIFO count: {}", self.fifo_filled_cnt());

        // Read one block
        self.set_size(512, 512);
        let cmd = CMD::data_cmd(card_idx, 17);
        let cmdarg = CMDARG::empty();
        let _resp = self
            .send_cmd(cmd, cmdarg, Some(&mut buffer), true)
            .expect("Error sending command");

        log::info!("Current FIFO count: {}", self.fifo_filled_cnt());

        let cmdarg = CMDARG::from(153);
        let _resp = self
            .send_cmd(cmd, cmdarg, Some(&mut buffer), true)
            .expect("Error sending command");
        log::debug!("Magic: 0x{:x}", buffer[0]);
        log::info!("Current FIFO count: {}", self.fifo_filled_cnt());

        // Try DMA

        // Allocate a page for descriptor table
        let frame = FrameAllocator.alloc_tracker(1).expect("[MMC] failed to get page");
        let descriptor_page_paddr = frame.range_ppn.start.start_addr();
        unsafe { &mut *self.frames.lock() }.push(frame);
        let descriptor_page_vaddr = descriptor_page_paddr.0 | Constant::KERNEL_ADDR_SPACE.start;
        const DESCRIPTOR_CNT: usize = 2;
        let mut buffer_page_paddr: [usize; DESCRIPTOR_CNT] = [0; DESCRIPTOR_CNT];
        for i in 0..DESCRIPTOR_CNT {
            let frame = FrameAllocator.alloc_tracker(1).expect("[MMC] failed to get page");
            buffer_page_paddr[i] = frame.range_ppn.start.start_addr().0;
            unsafe { &mut *self.frames.lock() }.push(frame);
        }
        let _descriptor_table = unsafe {
            core::slice::from_raw_parts_mut(
                descriptor_page_vaddr as *mut Descriptor,
                DESCRIPTOR_CNT,
            )
        };

        log::info!("Control register: {:?}", self.control_reg());
        let base = self.virt_base_addr() as *mut u32;
        let rinsts: RINSTS = unsafe { base.byte_add(RINSTS::offset()).read_volatile() }.into();
        // Clear interrupt by writing 1
        unsafe {
            // Just clear all for now
            base.byte_add(RINSTS::offset())
                .write_volatile(rinsts.into());
        }

        log::info!("[MMC] INT Status register: {:?}", rinsts);
    }

    pub fn virt_base_addr(&self) -> usize {
        self.meta.mmio_ranges[0].start | Constant::KERNEL_ADDR_SPACE.start
    }

    /// Read data from FIFO
    fn read_fifo<T>(&self) -> T {
        let base = self.virt_base_addr() as *mut T;
        let result = unsafe { base.byte_add(*self.fifo_offset.lock()).read_volatile() };
        unsafe { *self.fifo_offset.lock() += size_of::<T>() };
        result
    }
    /// Write data to FIFO
    fn write_fifo<T>(&self, value: T) {
        let base = self.virt_base_addr() as *mut T;
        unsafe {
            base.byte_add(*self.fifo_offset.lock()).write_volatile(value);
            *self.fifo_offset.lock() += size_of::<T>()
        };
    }
    /// Reset FIFO offset
    fn reset_fifo_offset(&self) {
        // Hard coded
        // From Synopsys documentation
        *self.fifo_offset.lock() = 0x600;
    }

    /// Reset FIFO
    fn reset_fifo(&self) {
        let base = self.virt_base_addr() as *mut CTRL;
        let ctrl = self.control_reg().with_fifo_reset(true);
        unsafe { base.byte_add(*self.fifo_offset.lock()).write_volatile(ctrl) }
    }

    /// Set transaction size
    ///
    /// block_size: size of transfer
    /// byte_cnt: number of bytes to transfer
    fn set_size(&self, block_size: usize, byte_cnt: usize) {
        let blksiz = BLKSIZ::new().with_block_size(block_size);
        let bytcnt = BYTCNT::new().with_byte_count(byte_cnt);
        let base = self.virt_base_addr() as *mut BLKSIZ;
        unsafe { base.byte_add(BLKSIZ::offset()).write_volatile(blksiz) };
        let base = self.virt_base_addr() as *mut BYTCNT;
        unsafe { base.byte_add(BYTCNT::offset()).write_volatile(bytcnt) };
    }

    fn set_controller_bus_width(&self, card_index: usize, width: CtypeCardWidth) {
        let ctype = CTYPE::set_card_width(card_index, width);
        let base = self.virt_base_addr() as *mut CTYPE;
        unsafe { base.byte_add(CTYPE::offset()).write_volatile(ctype) }
    }

    fn last_response_command_index(&self) -> usize {
        let base = self.virt_base_addr() as *mut STATUS;
        let status = unsafe { base.byte_add(STATUS::offset()).read_volatile() };
        status.response_index()
    }

    fn fifo_filled_cnt(&self) -> usize {
        self.status().fifo_count()
    }
    fn status(&self) -> STATUS {
        let base = self.virt_base_addr() as *mut STATUS;

        unsafe { base.byte_add(STATUS::offset()).read_volatile() }
    }

    fn card_detect(&self) -> usize {
        let base = self.virt_base_addr() as *mut CDETECT;
        let cdetect = unsafe { base.byte_add(CDETECT::offset()).read_volatile() };
        !cdetect.card_detect_n() & 0xFFFF
    }

    fn power_enable(&self) -> PWREN {
        let base = self.virt_base_addr() as *mut PWREN;
        unsafe { base.byte_add(PWREN::offset()).read_volatile() }
    }

    fn clock_enable(&self) -> CLKENA {
        let base = self.virt_base_addr() as *mut CLKENA;
        unsafe { base.byte_add(CLKENA::offset()).read_volatile() }
    }

    fn set_dma(&self, enable: bool) {
        let base = self.virt_base_addr() as *mut BMOD;
        let bmod = unsafe { base.byte_add(BMOD::offset()).read_volatile() };
        let bmod = bmod.with_idmac_enable(enable).with_software_reset(true);
        unsafe { base.byte_add(BMOD::offset()).write_volatile(bmod) };

        // Also reset the dma controller
        let base = self.virt_base_addr() as *mut CTRL;
        let ctrl = unsafe { base.byte_add(CTRL::offset()).read_volatile() };
        let ctrl = ctrl.with_dma_reset(true).with_use_internal_dmac(enable);
        unsafe { base.byte_add(CTRL::offset()).write_volatile(ctrl) };
    }

    fn dma_enabled(&self) -> bool {
        let base = self.virt_base_addr() as *mut BMOD;
        let bmod = unsafe { base.byte_add(BMOD::offset()).read_volatile() };
        bmod.idmac_enable()
    }

    fn dma_status(&self) -> IDSTS {
        let base = self.virt_base_addr() as *mut IDSTS;
        unsafe { base.byte_add(IDSTS::offset()).read_volatile() }
    }

    fn card_width(&self, index: usize) -> CtypeCardWidth {
        let base = self.virt_base_addr() as *mut CTYPE;
        let ctype = unsafe { base.byte_add(CTYPE::offset()).read_volatile() };
        ctype.card_width(index)
    }

    fn control_reg(&self) -> CTRL {
        let base = self.virt_base_addr() as *mut CTRL;

        unsafe { base.byte_add(CTRL::offset()).read_volatile() }
    }

    fn descriptor_base_address(&self) -> usize {
        let base = self.virt_base_addr() as *mut DBADDRL;
        let dbaddrl = unsafe { base.byte_add(DBADDRL::offset()).read_volatile() };
        let base = self.virt_base_addr() as *mut DBADDRU;
        let dbaddru = unsafe { base.byte_add(DBADDRU::offset()).read_volatile() };
        dbaddru.addr() << 32 | dbaddrl.addr()
    }

    fn set_descript_base_address(&self, addr: usize) {
        let base = self.virt_base_addr() as *mut u32;
        unsafe { base.byte_add(DBADDRL::offset()).write_volatile(addr as u32) };
        unsafe {
            base.byte_add(DBADDRU::offset())
                .write_volatile((addr >> 32) as u32)
        };
    }

    fn reset_clock(&self) {
        // Disable clock
        log::info!("Disable clock");
        let base = self.virt_base_addr() as *mut CLKENA;
        let clkena = CLKENA::new().with_cclk_enable(0);
        unsafe { base.byte_add(CLKENA::offset()).write_volatile(clkena) };
        let cmd = CMD::clock_cmd();
        self.send_cmd(cmd, CMDARG::empty(), None, false);

        // Set clock divider
        log::info!("Set clock divider");
        let base = self.virt_base_addr() as *mut CLKDIV;
        let clkdiv = CLKDIV::new().with_clk_divider0(4); // Magic, supposedly set to 400KHz
        unsafe { base.byte_add(CLKDIV::offset()).write_volatile(clkdiv) };

        // Re enable clock
        log::info!("Renable clock");
        let base = self.virt_base_addr() as *mut CLKENA;
        let clkena = CLKENA::new().with_cclk_enable(1);
        unsafe { base.byte_add(CLKENA::offset()).write_volatile(clkena) };

        let cmd = CMD::clock_cmd();
        self.send_cmd(cmd, CMDARG::empty(), None, false);
    }

    fn send_cmd(
        &self,
        cmd: CMD,
        arg: CMDARG,
        buffer: Option<&mut [usize]>,
        is_read: bool,
    ) -> Option<RESP> {
        let base = self.virt_base_addr() as *mut u32;

        // Sanity check
        if cmd.data_expected() && !self.dma_enabled() {
            debug_assert!(buffer.is_some())
        }

        let mut buffer_offset = 0;

        // Wait for can send cmd
        wait_for!({
            let cmd: CMD = unsafe { base.byte_add(CMD::offset()).read_volatile() }.into();
            cmd.can_send_cmd()
        });
        // Wait for card not busy if data is required
        if cmd.data_expected() {
            wait_for!({
                let status: STATUS =
                    unsafe { base.byte_add(STATUS::offset()).read_volatile() }.into();
                !status.data_busy()
            })
        }
        unsafe {
            // Set CMARG
            base.byte_add(CMDARG::offset()).write_volatile(arg.into());
            // Send CMD
            base.byte_add(CMD::offset()).write_volatile(cmd.into());
        }

        // Wait for cmd accepted
        wait_for!({
            let cmd: CMD = unsafe { base.byte_add(CMD::offset()).read_volatile() }.into();
            cmd.can_send_cmd()
        });

        // Wait for command done if response expected
        if cmd.response_expected() {
            wait_for!({
                let rinsts: RINSTS =
                    unsafe { base.byte_add(RINSTS::offset()).read_volatile() }.into();
                rinsts.command_done()
            });
        }

        // Wait for data transfer complete if data expected
        if cmd.data_expected() {
            let buffer = // TODO: dirty
                buffer.unwrap_or(unsafe { core::slice::from_raw_parts_mut(core::ptr::NonNull::dangling().as_ptr(), 64) });
            assert!(buffer_offset == 0);
            if is_read {
                wait_for!({
                    let rinsts: RINSTS =
                        unsafe { base.byte_add(RINSTS::offset()).read_volatile() }.into();
                    if rinsts.receive_data_request() && !self.dma_enabled() {
                        while self.fifo_filled_cnt() >= 2 {
                            if buffer_offset >= 64 {
                                break;
                            }
                            buffer[buffer_offset] = self.read_fifo::<usize>();
                            buffer_offset += 1;
                        }
                    }
                    rinsts.data_transfer_over() || !rinsts.no_error()
                });
            } else {
                wait_for!({
                    let rinsts: RINSTS =
                        unsafe { base.byte_add(RINSTS::offset()).read_volatile() }.into();
                    if rinsts.transmit_data_request() && !self.dma_enabled() {
                        // Hard coded FIFO depth
                        while self.fifo_filled_cnt() < 120 {
                            if buffer_offset >= 64 {
                                break;
                            }
                            self.write_fifo::<usize>(buffer[buffer_offset]);
                            buffer_offset += 1;
                        }
                    }
                    rinsts.data_transfer_over() || !rinsts.no_error()
                });
            }
            log::debug!("transmit {:?} bytes", (buffer_offset) * 8);
            log::debug!("Current oFIFO count: {}", self.fifo_filled_cnt());
            self.reset_fifo_offset();
        }

        // Check for error
        let rinsts: RINSTS = unsafe { base.byte_add(RINSTS::offset()).read_volatile() }.into();
        // Clear interrupt by writing 1
        unsafe {
            // Just clear all for now
            base.byte_add(RINSTS::offset())
                .write_volatile(rinsts.into());
        }

        // Check response
        let base = self.virt_base_addr() as *mut RESP;
        let resp = unsafe { base.byte_add(RESP::offset()).read_volatile() };
        if rinsts.no_error() && !rinsts.command_conflict() {
            // No return for clock command
            if cmd.update_clock_register_only() {
                log::info!("Clock cmd done");
                return None;
            }
            log::debug!(
                "CMD{} done: {:?}, dma: {:?}",
                cmd.cmd_index(),
                rinsts.status(),
                self.dma_enabled()
            );
            Some(resp)
        } else {
            log::warn!("CMD{} error: {:?}", cmd.cmd_index(), rinsts.status());
            log::warn!("Dumping response");
            log::warn!("Response: {:x?}", resp);
            log::warn!("dma: {:?}", self.dma_enabled());
            None
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

#[inline(never)]
pub fn spin(cycle: usize) {
    for _ in 0..cycle {
        core::hint::spin_loop();
    }
}