#![allow(dead_code, unused_assignments, unused_mut, non_upper_case_globals)]

use crate::libahci::*;
use crate::libata::*;
use crate::platform::*;
use crate::println;

use core::ffi::CStr;
use core::ptr::{null_mut, read_volatile, write_volatile};

fn u8slice_to_str(slice: &[u8]) -> &str {
    let cstr = CStr::from_bytes_until_nul(slice).unwrap();
    cstr.to_str().unwrap()
}

fn ahci_readl(addr: u64) -> u32 {
    let mut data: u32 = 0;
    unsafe { data = read_volatile(addr as *mut u32) };
    return data;
}

fn ahci_writel(data: u32, addr: u64) {
    unsafe { write_volatile(addr as *mut u32, data) };
}

fn ahci_ffs32(val: u32) -> u32 {
    let mut bit: u32 = 1;
    let mut i: u32 = val;

    if i == 0 {
        return 0;
    }

    while i & 1 == 0 {
        i >>= 1;
        bit += 1;
    }

    return bit;
}

// 输出ahci控制器信息
fn ahci_print_info(ahci_dev: &ahci_device) {
    let vers: u32 = ahci_dev.version;
    let cap: u32 = ahci_dev.cap;
    let cap2: u32 = ahci_dev.cap2;
    let impl_0: u32 = ahci_dev.port_map;
    let speed: u32 = (ahci_dev.cap >> 20) & 0xf;

    let scc_s= "SATA";
    let mut speed_s = "";
    if speed == 1 {
        speed_s = "1.5";
    } else if speed == 2 {
        speed_s = "3";
    } else if speed == 3 {
        speed_s = "6";
    } else {
        speed_s = "?";
    }

    println!(
        "AHCI vers {:02x}{:02x}.{:02x}{:02x}, {} slots, {} ports, {} Gbps, 0x{} impl, {} mode",
        vers >> 24 & 0xff,
        vers >> 16 & 0xff,
        vers >> 8 & 0xff,
        vers & 0xff,
        (cap >> 8 & 0x1f) + 1,
        (cap & 0x1f) + 1,
        speed_s,
        impl_0,
        scc_s
    );
    println!(
        "flags: {}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
        if cap & HOST_CAP_64 != 0 {
            "64bit "
        } else {
            ""
        },
        if cap & HOST_CAP_NCQ != 0 {
            "ncq "
        } else {
            ""
        },
        if cap & HOST_CAP_SNTF != 0 {
            "sntf "
        } else {
            ""
        },
        if cap & HOST_CAP_MPS != 0 {
            "ilck "
        } else {
            ""
        },
        if cap & HOST_CAP_SSS != 0 {
            "stag "
        } else {
            ""
        },
        if cap & HOST_CAP_ALPM != 0 {
            "pm "
        } else {
            ""
        },
        if cap & HOST_CAP_LED != 0 {
            "led "
        } else {
            ""
        },
        if cap & HOST_CAP_CLO != 0 {
            "clo "
        } else {
            ""
        },
        if cap & HOST_CAP_ONLY != 0 {
            "only "
        } else {
            ""
        },
        if cap & HOST_CAP_PMP != 0 {
            "pmp "
        } else {
            ""
        },
        if cap & HOST_CAP_FBS != 0 {
            "fbs "
        } else {
            ""
        },
        if cap & HOST_CAP_PIO_MULTI != 0 {
            "pio "
        } else {
            ""
        },
        if cap & HOST_CAP_SSC != 0 {
            "slum "
        } else {
            ""
        },
        if cap & HOST_CAP_PART != 0 {
            "part "
        } else {
            ""
        },
        if cap & HOST_CAP_CCC != 0 {
            "ccc "
        } else {
            ""
        },
        if cap & HOST_CAP_EMS != 0 {
            "ems "
        } else {
            ""
        },
        if cap & HOST_CAP_SXS != 0 {
            "sxs "
        } else {
            ""
        },
        if cap2 & HOST_CAP2_DESO != 0 {
            "deso "
        } else {
            ""
        },
        if cap2 & HOST_CAP2_SADM != 0 {
            "sadm "
        } else {
            ""
        },
        if cap2 & HOST_CAP2_SDS != 0 {
            "sds "
        } else {
            ""
        },
        if cap2 & HOST_CAP2_APST != 0 {
            "apst "
        } else {
            ""
        },
        if cap2 & HOST_CAP2_NVMHCI != 0 {
            "nvmp "
        } else {
            ""
        },
        if cap2 & HOST_CAP2_BOH != 0 {
            "boh "
        } else {
            ""
        },
    );
}

// 输出sata硬盘信息
fn ahci_sata_print_info(pdev: &ahci_blk_dev) {
    println!("SATA Device Info:");
    println!("S/N: {:?}", u8slice_to_str(&pdev.serial));
    println!("Product model number: {:?}", u8slice_to_str(&pdev.product));
    println!("Firmware version: {:?}", u8slice_to_str(&pdev.revision));
    println!("Capacity: {}", pdev.lba);
}

fn ahci_port_base(base: u64, port: u8) -> u64 {
    return base + 0x100 + (port as u64 * 0x80);
}

// ahci初始化
fn ahci_host_init(ahci_dev: &mut ahci_device) -> i32 {
    let mut tmp: u32 = 0;
    let mut timeout: u32 = 0;
    let mut port_mmio: u64 = 0;
    let host_mmio: u64 = ahci_dev.mmio_base;

    // reset ahci controller
    tmp = ahci_readl(host_mmio + HOST_CTL);
    if tmp & HOST_RESET == 0 {
        ahci_writel(tmp | HOST_RESET, host_mmio + HOST_CTL);
    }
    // wait for reset done
    loop {
        unsafe { ahci_mdelay(1) };
        tmp = ahci_readl(host_mmio + HOST_CTL);
        if tmp & HOST_RESET == 0 {
            break;
        }
    }

    // enable ahci
    tmp = ahci_readl(host_mmio + HOST_CTL);
    ahci_writel(tmp | HOST_AHCI_EN, host_mmio + HOST_CTL);
    unsafe { ahci_mdelay(1) };

    // init cap and pi
    // beware if no firmware initialized before
    // these bits are ready-only after write-once
    tmp = HOST_CAP_MPS | HOST_CAP_SSS;
    ahci_writel(tmp, host_mmio + HOST_CAP);
    ahci_writel(0xf, host_mmio + HOST_PORTS_IMPL);
    ahci_readl(host_mmio + HOST_PORTS_IMPL); // flush

    // get ahci info
    ahci_dev.cap = ahci_readl(host_mmio + HOST_CAP);
    ahci_dev.cap2 = ahci_readl(host_mmio + HOST_CAP2);
    ahci_dev.version = ahci_readl(host_mmio + HOST_VERSION);
    ahci_dev.port_map = ahci_readl(host_mmio + HOST_PORTS_IMPL);
    ahci_dev.n_ports = ((ahci_dev.cap & 0x1f) + 1) as u8;

    // init each port
    // for ls2kla, only 1 port available
    for i in 0..ahci_dev.n_ports {
        ahci_dev.port[i as usize].port_mmio = ahci_port_base(host_mmio, i);
        port_mmio = ahci_dev.port[i as usize].port_mmio;

        // ensure sata is in idle state
        tmp = ahci_readl(port_mmio + PORT_CMD);
        if tmp & (PORT_CMD_LIST_ON | PORT_CMD_FIS_ON | PORT_CMD_FIS_RX | PORT_CMD_START) != 0 {
            tmp &= !(PORT_CMD_START);
            ahci_writel(tmp, port_mmio + PORT_CMD);
            unsafe { ahci_mdelay(500) };
            while ahci_readl(port_mmio + PORT_CMD) & PORT_CMD_LIST_ON != 0 {}
        }

        // set spin up
        tmp = ahci_readl(port_mmio + PORT_CMD);
        ahci_writel(tmp | PORT_CMD_SPIN_UP, port_mmio + PORT_CMD);

        // wait for spin up
        timeout = 1000;
        loop {
            unsafe { ahci_mdelay(1) };
            tmp = ahci_readl(port_mmio + PORT_CMD);
            timeout -= 1;
            if !(tmp | PORT_CMD_SPIN_UP == 0 && timeout != 0) {
                break;
            }
        }
        if timeout <= 0 {
            println!("spin up cannot finish");
            return -1;
        }

        // wait for port link up
        timeout = 1000;
        loop {
            unsafe { ahci_mdelay(1) };
            tmp = ahci_readl(port_mmio + PORT_SCR_STAT);
            tmp &= 0xf;
            timeout -= 1;
            if (tmp == 0x3 || tmp == 0x1) || timeout == 0 {
                break;
            }
        }
        if timeout <= 0 {
            println!("port {} sata link timeout", i);
        } else {
            println!("port {} sata link up", i);
        }

        // clear serr
        tmp = ahci_readl(port_mmio + PORT_SCR_ERR);
        ahci_writel(tmp, port_mmio + PORT_SCR_ERR);

        // ack any pending irq events for this port
        tmp = ahci_readl(host_mmio + PORT_IRQ_STAT);
        ahci_writel(tmp, host_mmio + PORT_IRQ_STAT);

        ahci_writel(0x1 << i, host_mmio + HOST_IRQ_STAT);

        // set irq mask
        ahci_writel(DEF_PORT_IRQ, port_mmio + PORT_IRQ_MASK);

        // detect port status
        tmp = ahci_readl(port_mmio + PORT_SCR_STAT);
        if (tmp & 0xf) == 0x3 {
            ahci_dev.port_map_linkup |= 0x1 << i;
        }
    }

    // interrupt enable
    // we dont use interrupt actually
    tmp = ahci_readl(host_mmio + HOST_CTL);
    ahci_writel(tmp | HOST_IRQ_EN, host_mmio + HOST_CTL);
    tmp = ahci_readl(host_mmio + HOST_CTL);

    return 0;
}

// ahci填充sgdma
fn ahci_fill_sg(ahci_dev: &ahci_device, port: u8, buf: *mut u8, mut buf_len: u32) -> u32 {
    let pp: &ahci_ioport = &ahci_dev.port[port as usize];
    let mut ahci_sg: *mut ahci_sg = pp.cmd_tbl_sg;

    let max_bytes: u32 = AHCI_MAX_BYTES_PER_SG;
    let sg_count: u32 = ((buf_len - 1) / max_bytes) + 1;

    if sg_count > AHCI_MAX_SG {
        println!("too much sg");
        return 0;
    }
    for i in 0..sg_count {
        let size: u32 = buf_len.min(max_bytes) - 1;
        let offset: isize = (i * max_bytes) as isize;

        unsafe {
            let buf_dma: u64 = ahci_virt_to_phys(buf.offset(offset) as u64);
            (*ahci_sg).addr_lo = (buf_dma & 0xffffffff) as u32;
            (*ahci_sg).addr_hi = (buf_dma >> 32) as u32;
            (*ahci_sg).flags_size = 0x3fffff & size;
            ahci_sg = ahci_sg.offset(1);
        }
        if buf_len >= max_bytes {
            buf_len -= max_bytes;
        }
    }

    return sg_count;
}

fn ahci_fill_cmd_slot(pp: &ahci_ioport, cmd_slot: u32, opts: u32) {
    let mut cmd_hdr: *mut ahci_cmd_hdr =
        unsafe { (pp.cmd_slot).offset((AHCI_CMD_SZ * cmd_slot) as isize) };

    unsafe {
        (*cmd_hdr).opts = opts;
        (*cmd_hdr).status = 0;
        (*cmd_hdr).tbl_addr_lo = (pp.cmd_tbl_dma & 0xffffffff) as u32;
        (*cmd_hdr).tbl_addr_hi = (pp.cmd_tbl_dma >> 32) as u32;
    }
}

fn ahci_get_cmd_slot(value: u32) -> u32 {
    let slot: u32 = if value != 0 { ahci_ffs32(value) } else { 0 };
    return slot;
}

// ahci命令执行
fn ahci_exec_ata_cmd(
    ahci_dev: &ahci_device,
    port: u8,
    cfis: *const sata_fis_h2d,
    buf: *mut u8,
    buf_len: u32,
    is_write: u32,
) -> u32 {
    let pp: &ahci_ioport = &ahci_dev.port[port as usize];
    let port_mmio: u64 = pp.port_mmio;
    let mut sg_count: u32 = 0;

    let cmd_slot: u32 = ahci_get_cmd_slot(ahci_readl(port_mmio + PORT_CMD_ISSUE));
    if cmd_slot == 32 {
        println!("cannot find empty command slot");
        return 0;
    }

    if buf_len > AHCI_MAX_BYTES_PER_TRANS {
        println!("max transfer length is {} bytes", AHCI_MAX_BYTES_PER_TRANS);
        return 0;
    }

    unsafe {
        (pp.cmd_tbl as *mut sata_fis_h2d).write_volatile(*cfis);
    }

    if !buf.is_null() && buf_len != 0 {
        sg_count = ahci_fill_sg(ahci_dev, port, buf, buf_len);
    }

    let opts: u32 = (size_of::<sata_fis_h2d>() as u64 >> 2
        | (sg_count << 16) as u64
        | (is_write << 6) as u64) as u32;

    ahci_fill_cmd_slot(pp, cmd_slot, opts);

    unsafe { ahci_sync_dcache() };

    ahci_writel((1 << cmd_slot) as u32, port_mmio + PORT_CMD_ISSUE);

    while ahci_readl(port_mmio + PORT_CMD_ISSUE) & (1 << cmd_slot) as u32 != 0 {}

    unsafe { ahci_sync_dcache() };

    return buf_len;
}

fn ahci_set_feature(ahci_dev: &ahci_device, subcmd: u8, action: u8) {
    let port: u8 = ahci_dev.port_idx;
    let cfis: sata_fis_h2d = sata_fis_h2d {
        fis_type: SATA_FIS_TYPE_REGISTER_H2D,
        pm_port_c: 0x80,
        command: ATA_CMD_SET_FEATURES,
        features: subcmd,
        lba_low: 0,
        lba_mid: 0,
        lba_high: 0,
        device: 0,
        lba_low_exp: 0,
        lba_mid_exp: 0,
        lba_high_exp: 0,
        features_exp: 0,
        sector_count: action,
        sector_count_exp: 0,
        res1: 0,
        control: 0,
        res2: [0; 4],
    };

    ahci_exec_ata_cmd(ahci_dev, port, &cfis, null_mut(), 0, READ_CMD);
}

// 初始化ahci端口
fn ahci_port_start(ahci_dev: &mut ahci_device, port: u8) -> i32 {
    let pp: &mut ahci_ioport = &mut ahci_dev.port[port as usize];
    let port_mmio: u64 = pp.port_mmio;

    let port_status: u32 = ahci_readl(port_mmio + PORT_SCR_STAT);
    if (port_status & 0xf) != 0x3 {
        println!("no link on port {}", port as u32);
        return -1;
    }

    let mut mem: u64;
    unsafe {
        mem = ahci_malloc_align(AHCI_PORT_PRIV_DMA_SZ as u64, 1024);
        (mem as *mut u8).write_bytes(0, AHCI_PORT_PRIV_DMA_SZ as usize);
    }

    pp.cmd_slot = mem as *mut ahci_cmd_hdr;
    pp.cmd_slot_dma = unsafe { ahci_virt_to_phys(mem) };

    mem += AHCI_CMD_SLOT_SZ as u64;

    pp.rx_fis = mem;
    pp.rx_fis_dma = unsafe { ahci_virt_to_phys(mem) };

    mem += AHCI_RX_FIS_SZ as u64;

    pp.cmd_tbl = mem;
    pp.cmd_tbl_dma = unsafe { ahci_virt_to_phys(mem) };

    mem += AHCI_CMD_TBL_HDR_SZ as u64;

    pp.cmd_tbl_sg = mem as *mut ahci_sg;

    ahci_writel(
        (pp.cmd_slot_dma & 0xffffffff) as u32,
        port_mmio + PORT_LST_ADDR,
    );
    ahci_writel((pp.cmd_slot_dma >> 32) as u32, port_mmio + PORT_LST_ADDR_HI);
    ahci_writel(
        (pp.rx_fis_dma & 0xffffffff) as u32,
        port_mmio + PORT_FIS_ADDR,
    );
    ahci_writel((pp.rx_fis_dma >> 32) as u32, port_mmio + PORT_FIS_ADDR_HI);

    ahci_writel(
        PORT_CMD_ICC_ACTIVE
            | PORT_CMD_FIS_RX
            | PORT_CMD_POWER_ON
            | PORT_CMD_SPIN_UP
            | PORT_CMD_START,
        port_mmio + PORT_CMD,
    );

    let mut timeout: u32 = 200;
    let mut tmp: u32 = 0;
    loop {
        unsafe { ahci_mdelay(1) };
        tmp = ahci_readl(port_mmio + PORT_TFDATA);
        tmp &= (ATA_ERR | ATA_DRQ | ATA_BUSY) as u32;
        timeout -= 1;
        if tmp == 0 || timeout == 0 {
            break;
        }
    }

    if timeout <= 0 {
        println!("ahci port {} failed to start", port as u32);
        return -1;
    }
    return 0;
}

fn ahci_sata_identify(ahci_dev: &ahci_device, id: &mut [u16]) {
    let port: u8 = ahci_dev.port_idx;
    let buf_len: u32 = ATA_ID_WORDS * 2;
    let cfis: sata_fis_h2d = sata_fis_h2d {
        fis_type: SATA_FIS_TYPE_REGISTER_H2D,
        pm_port_c: 0x80,
        command: ATA_CMD_ID_ATA,
        features: 0,
        lba_low: 0,
        lba_mid: 0,
        lba_high: 0,
        device: 0,
        lba_low_exp: 0,
        lba_mid_exp: 0,
        lba_high_exp: 0,
        features_exp: 0,
        sector_count: 0,
        sector_count_exp: 0,
        res1: 0,
        control: 0,
        res2: [0; 4],
    };
    ahci_exec_ata_cmd(
        ahci_dev,
        port,
        &cfis,
        id.as_mut_ptr() as *mut u8,
        buf_len,
        READ_CMD,
    );
}

fn ahci_sata_rw_cmd(
    ahci_dev: &ahci_device,
    start: u32,
    blkcnt: u32,
    buffer: *mut u8,
    is_write: u32,
) -> u32 {
    let port: u8 = ahci_dev.port_idx;
    let block: u32 = start;
    let buf_len: u32 = ATA_SECT_SIZE * blkcnt;
    let cfis: sata_fis_h2d = sata_fis_h2d {
        fis_type: SATA_FIS_TYPE_REGISTER_H2D,
        pm_port_c: 0x80,
        command: if is_write != 0 {
            ATA_CMD_WRITE
        } else {
            ATA_CMD_READ
        },
        features: 0,
        lba_low: (block & 0xff) as u8,
        lba_mid: (block >> 8 & 0xff) as u8,
        lba_high: (block >> 16 & 0xff) as u8,
        device: (block >> 24 & 0xf) as u8 | ATA_LBA,
        lba_low_exp: 0,
        lba_mid_exp: 0,
        lba_high_exp: 0,
        features_exp: 0,
        sector_count: (blkcnt & 0xff) as u8,
        sector_count_exp: 0,
        res1: 0,
        control: 0,
        res2: [0; 4],
    };

    if ahci_exec_ata_cmd(ahci_dev, port, &cfis, buffer, buf_len, is_write) > 0 {
        return blkcnt;
    } else {
        return 0;
    };
}

fn ahci_sata_flush_cache(ahci_dev: &ahci_device) {
    let port: u8 = ahci_dev.port_idx;
    let cfis: sata_fis_h2d = sata_fis_h2d {
        fis_type: SATA_FIS_TYPE_REGISTER_H2D,
        pm_port_c: 0x80,
        command: ATA_CMD_FLUSH,
        features: 0,
        lba_low: 0,
        lba_mid: 0,
        lba_high: 0,
        device: 0,
        lba_low_exp: 0,
        lba_mid_exp: 0,
        lba_high_exp: 0,
        features_exp: 0,
        sector_count: 0,
        sector_count_exp: 0,
        res1: 0,
        control: 0,
        res2: [0; 4],
    };
    ahci_exec_ata_cmd(ahci_dev, port, &cfis, null_mut(), 0, READ_CMD);
}

fn ata_low_level_rw_lba28(
    ahci_dev: &ahci_device,
    blknr: u64,
    blkcnt: u32,
    buffer: *mut u8,
    is_write: u32,
) -> u32 {
    let mut start: u32 = blknr as u32;
    let mut blks: u32 = blkcnt;
    let mut addr: *mut u8 = buffer;
    let max_blks: u32 = ATA_MAX_SECTORS;

    loop {
        if blks > max_blks {
            if max_blks != ahci_sata_rw_cmd(ahci_dev, start, max_blks, addr, is_write) {
                return 0;
            }
            start += max_blks;
            blks -= max_blks;
            addr = addr.wrapping_add((ATA_SECT_SIZE * max_blks) as usize);
        } else {
            if blks != ahci_sata_rw_cmd(ahci_dev, start, blks, addr, is_write) {
                return 0;
            }
            start += blks;
            blks = 0;
            addr = addr.wrapping_add((ATA_SECT_SIZE * blks) as usize);
        }
        if blks == 0 {
            break;
        }
    }
    return blkcnt;
}

fn ahci_sata_rw_cmd_ext(
    ahci_dev: &ahci_device,
    start: u64,
    blkcnt: u32,
    buffer: *mut u8,
    is_write: u32,
) -> u32 {
    let port: u8 = ahci_dev.port_idx;
    let block: u64 = start;
    let buf_len: u32 = ATA_SECT_SIZE * blkcnt;
    let cfis: sata_fis_h2d = sata_fis_h2d {
        fis_type: SATA_FIS_TYPE_REGISTER_H2D,
        pm_port_c: 0x80,
        command: if is_write != 0 {
            ATA_CMD_WRITE_EXT
        } else {
            ATA_CMD_READ_EXT
        },
        features: 0,
        lba_low: (block & 0xff) as u8,
        lba_mid: (block >> 8 & 0xff) as u8,
        lba_high: (block >> 16 & 0xff) as u8,
        device: ATA_LBA,
        lba_low_exp: (block >> 24 & 0xff) as u8,
        lba_mid_exp: (block >> 32 & 0xff) as u8,
        lba_high_exp: (block >> 40 & 0xff) as u8,
        features_exp: 0,
        sector_count: (blkcnt & 0xff) as u8,
        sector_count_exp: (blkcnt >> 8 & 0xff) as u8,
        res1: 0,
        control: 0,
        res2: [0; 4],
    };

    if ahci_exec_ata_cmd(ahci_dev, port, &cfis, buffer, buf_len, is_write) > 0 {
        return blkcnt;
    } else {
        return 0;
    };
}

fn ahci_sata_flush_cache_ext(ahci_dev: &ahci_device) {
    let port: u8 = ahci_dev.port_idx;
    let cfis: sata_fis_h2d = sata_fis_h2d {
        fis_type: SATA_FIS_TYPE_REGISTER_H2D,
        pm_port_c: 0x80,
        command: ATA_CMD_FLUSH_EXT,
        features: 0,
        lba_low: 0,
        lba_mid: 0,
        lba_high: 0,
        device: 0,
        lba_low_exp: 0,
        lba_mid_exp: 0,
        lba_high_exp: 0,
        features_exp: 0,
        sector_count: 0,
        sector_count_exp: 0,
        res1: 0,
        control: 0,
        res2: [0; 4],
    };

    ahci_exec_ata_cmd(ahci_dev, port, &cfis, null_mut(), 0, READ_CMD);
}

fn ata_low_level_rw_lba48(
    ahci_dev: &ahci_device,
    blknr: u64,
    blkcnt: u32,
    buffer: *mut u8,
    is_write: u32,
) -> u32 {
    let mut start: u64 = blknr;
    let mut blks: u32 = blkcnt;
    let mut addr: *mut u8 = buffer;
    let max_blks: u32 = ATA_MAX_SECTORS_LBA48;

    loop {
        if blks > max_blks {
            if max_blks != ahci_sata_rw_cmd_ext(ahci_dev, start, max_blks, addr, is_write) {
                return 0;
            }
            start += max_blks as u64;
            blks -= max_blks;
            addr = addr.wrapping_add((ATA_SECT_SIZE * max_blks) as usize);
        } else {
            if blks != ahci_sata_rw_cmd_ext(ahci_dev, start, blks, addr, is_write) {
                return 0;
            }
            start += blks as u64;
            blks = 0;
            addr = addr.wrapping_add((ATA_SECT_SIZE * blks) as usize);
        }

        if blks == 0 {
            break;
        }
    }

    return blkcnt;
}

// 扫描ahci端口
fn ahci_port_scan(ahci_dev: &mut ahci_device) -> i32 {
    let linkmap: u32 = ahci_dev.port_map_linkup;
    if linkmap == 0 {
        println!("no port device detected");
        return -1;
    }

    for i in 0..ahci_dev.n_ports {
        if (linkmap >> i & 0x1) != 0 {
            if ahci_port_start(ahci_dev, i) != 0 {
                println!("cannot start port {}", i as u32);
                return -1;
            }
            ahci_dev.port_idx = i;
            break;
        }
    }

    return 0;
}

fn ahci_sata_xfer_mode(ahci_dev: &mut ahci_device, id: &[u16]) {
    ahci_dev.pio_mask = id[ATA_ID_PIO_MODES as usize] as u32;
    ahci_dev.udma_mask = id[ATA_ID_UDMA_MODES as usize] as u32;
}

fn ahci_sata_init_wcache(ahci_dev: &mut ahci_device, id: &[u16]) {
    if ata_id_has_wcache(&id) && ata_id_wcache_enabled(&id) {
        ahci_dev.flags |= SATA_FLAG_WCACHE;
    }
    if ata_id_has_flush(&id) {
        ahci_dev.flags |= SATA_FLAG_FLUSH;
    }
    if ata_id_has_flush_ext(&id) {
        ahci_dev.flags |= SATA_FLAG_FLUSH_EXT;
    }
}

// 扫描sata
fn ahci_sata_scan(ahci_dev: &mut ahci_device) {
    const id_len: usize = (ATA_ID_WORDS + 1) as usize;
    let mut id: [u16; id_len] = [0; id_len];
    
    ahci_sata_identify(ahci_dev, &mut id);

    let pdev: &mut ahci_blk_dev = &mut ahci_dev.blk_dev;

    ata_id_c_string(&id, &mut pdev.product, ATA_ID_PROD as usize);
    ata_id_c_string(&id, &mut pdev.serial, ATA_ID_SERNO as usize);
    ata_id_c_string(&id, &mut pdev.revision, ATA_ID_FW_REV as usize);

    pdev.lba = ata_id_n_sectors(&id);
    pdev.blksz = ATA_SECT_SIZE as u64;
    pdev.lba48 = ata_id_has_lba48(&id);
    pdev.queue_depth = ata_id_queue_depth(&id);

    ahci_sata_xfer_mode(ahci_dev, &id);

    ahci_sata_init_wcache(ahci_dev, &id);

    let subcmd: u8 = SETFEATURES_XFER;
    let action: u8 = (ahci_ffs32(ahci_dev.udma_mask + 1) + 0x3e) as u8;
    ahci_set_feature(ahci_dev, subcmd, action);

    ahci_sata_print_info(&ahci_dev.blk_dev);
}

// ahci sata读函数
// blknr 开始的sector/block偏移
// blkcnt 读取的sector/block总数
#[unsafe(no_mangle)]
pub extern "C" fn ahci_sata_read_common(
    ahci_dev: &ahci_device,
    blknr: u64,
    blkcnt: u32,
    buffer: *mut u8,
) -> u64 {
    let pdev: &ahci_blk_dev = &ahci_dev.blk_dev;
    let mut rc: u32 = 0;

    if pdev.lba48 {
        rc = ata_low_level_rw_lba48(ahci_dev, blknr, blkcnt, buffer, READ_CMD);
    } else {
        rc = ata_low_level_rw_lba28(ahci_dev, blknr, blkcnt, buffer, READ_CMD);
    }

    return rc as u64;
}

// ahci sata写函数
// blknr 开始的sector/block偏移
// blkcnt 写入的sector/block总数
#[unsafe(no_mangle)]
pub extern "C" fn ahci_sata_write_common(
    ahci_dev: &ahci_device,
    blknr: u64,
    blkcnt: u32,
    buffer: *mut u8,
) -> u64 {
    let pdev: &ahci_blk_dev = &ahci_dev.blk_dev;
    let mut flags: u32 = ahci_dev.flags;
    let mut rc: u32 = 0;

    if pdev.lba48 {
        rc = ata_low_level_rw_lba48(ahci_dev, blknr, blkcnt, buffer, WRITE_CMD);
        if flags & SATA_FLAG_WCACHE != 0 && flags & SATA_FLAG_FLUSH_EXT != 0 {
            ahci_sata_flush_cache_ext(ahci_dev);
        }
    } else {
        rc = ata_low_level_rw_lba28(ahci_dev, blknr, blkcnt, buffer, WRITE_CMD);
        if flags & SATA_FLAG_WCACHE != 0 && flags & SATA_FLAG_FLUSH != 0 {
            ahci_sata_flush_cache(ahci_dev);
        }
    }

    return rc as u64;
}

// ahci初始化函数
#[unsafe(no_mangle)]
pub extern "C" fn ahci_init(ahci_dev: &mut ahci_device) -> i32 {
    ahci_dev.mmio_base = unsafe { ahci_phys_to_uncached(0x400e0000) };

    let mut ret: i32 = ahci_host_init(ahci_dev);
    if ret != 0 {
        return -1;
    }

    ret = ahci_port_scan(ahci_dev);
    if ret != 0 {
        return -1;
    }

    ahci_print_info(ahci_dev);

    ahci_sata_scan(ahci_dev);

    return 0;
}
