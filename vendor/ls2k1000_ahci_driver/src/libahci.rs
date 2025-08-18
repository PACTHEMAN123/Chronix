#![allow(dead_code, unused_assignments, unused_mut, non_upper_case_globals)]

use crate::libata::*;

pub const PORT_CMD_ICC_MASK: u32 = 0xf << 28;
pub const PORT_CMD_ICC_ACTIVE: u32 = 0x1 << 28;
pub const PORT_CMD_ICC_PARTIAL: u32 = 0x2 << 28;
pub const PORT_CMD_ICC_SLUMBER: u32 = 0x6 << 28;

pub const PORT_CMD_ASP: u32 = 0x1 << 27;
pub const PORT_CMD_ALPE: u32 = 0x1 << 26;
pub const PORT_CMD_ATAPI: u32 = 0x1 << 24;
pub const PORT_CMD_FBSCP: u32 = 0x1 << 22;
pub const PORT_CMD_ESP: u32 = 0x1 << 21;
pub const PORT_CMD_CPD: u32 = 0x1 << 20;
pub const PORT_CMD_MPSP: u32 = 0x1 << 19;
pub const PORT_CMD_HPCP: u32 = 0x1 << 18;
pub const PORT_CMD_PMP: u32 = 0x1 << 17;
pub const PORT_CMD_LIST_ON: u32 = 0x1 << 15;
pub const PORT_CMD_FIS_ON: u32 = 0x1 << 14;
pub const PORT_CMD_FIS_RX: u32 = 0x1 << 4;
pub const PORT_CMD_CLO: u32 = 0x1 << 3;
pub const PORT_CMD_POWER_ON: u32 = 0x1 << 2;
pub const PORT_CMD_SPIN_UP: u32 = 0x1 << 1;
pub const PORT_CMD_START: u32 = 0x1 << 0;

pub const PORT_CMD_CAP: u32 =
    PORT_CMD_HPCP | PORT_CMD_MPSP | PORT_CMD_CPD | PORT_CMD_ESP | PORT_CMD_FBSCP;

pub const PORT_IRQ_COLD_PRES: u32 = 0x1 << 31;
pub const PORT_IRQ_TF_ERR: u32 = 0x1 << 30;
pub const PORT_IRQ_HBUS_ERR: u32 = 0x1 << 29;
pub const PORT_IRQ_HBUS_DATA_ERR: u32 = 0x1 << 28;
pub const PORT_IRQ_IF_ERR: u32 = 0x1 << 27;
pub const PORT_IRQ_IF_NONFATAL: u32 = 0x1 << 26;
pub const PORT_IRQ_OVERFLOW: u32 = 0x1 << 24;
pub const PORT_IRQ_BAD_PMP: u32 = 0x1 << 23;
pub const PORT_IRQ_PHYRDY: u32 = 0x1 << 22;
pub const PORT_IRQ_DEV_ILCK: u32 = 0x1 << 7;
pub const PORT_IRQ_CONNECT: u32 = 0x1 << 6;
pub const PORT_IRQ_SG_DONE: u32 = 0x1 << 5;
pub const PORT_IRQ_UNK_FIS: u32 = 0x1 << 4;
pub const PORT_IRQ_SDB_FIS: u32 = 0x1 << 3;
pub const PORT_IRQ_DMAS_FIS: u32 = 0x1 << 2;
pub const PORT_IRQ_PIOS_FIS: u32 = 0x1 << 1;
pub const PORT_IRQ_D2H_REG_FIS: u32 = 0x1 << 0;
pub const PORT_IRQ_FREEZE: u32 = PORT_IRQ_HBUS_ERR
    | PORT_IRQ_IF_ERR
    | PORT_IRQ_CONNECT
    | PORT_IRQ_PHYRDY
    | PORT_IRQ_UNK_FIS
    | PORT_IRQ_BAD_PMP;
pub const PORT_IRQ_ERROR: u32 = PORT_IRQ_FREEZE | PORT_IRQ_TF_ERR | PORT_IRQ_HBUS_DATA_ERR;
pub const DEF_PORT_IRQ: u32 = PORT_IRQ_ERROR
    | PORT_IRQ_SG_DONE
    | PORT_IRQ_SDB_FIS
    | PORT_IRQ_DMAS_FIS
    | PORT_IRQ_PIOS_FIS
    | PORT_IRQ_D2H_REG_FIS;

pub const PORT_LST_ADDR: u64 = 0x00;
pub const PORT_LST_ADDR_HI: u64 = 0x04;
pub const PORT_FIS_ADDR: u64 = 0x08;
pub const PORT_FIS_ADDR_HI: u64 = 0x0c;
pub const PORT_IRQ_STAT: u64 = 0x10;
pub const PORT_IRQ_MASK: u64 = 0x14;
pub const PORT_CMD: u64 = 0x18;
pub const PORT_TFDATA: u64 = 0x20;
pub const PORT_SIG: u64 = 0x24;
pub const PORT_CMD_ISSUE: u64 = 0x38;
pub const PORT_SCR_STAT: u64 = 0x28;
pub const PORT_SCR_CTL: u64 = 0x2c;
pub const PORT_SCR_ERR: u64 = 0x30;
pub const PORT_SCR_ACT: u64 = 0x34;
pub const PORT_SCR_NTF: u64 = 0x3c;
pub const PORT_FBS: u64 = 0x40;
pub const PORT_DEVSLP: u64 = 0x44;

pub const HOST_CAP_SXS: u32 = 0x1 << 5;
pub const HOST_CAP_EMS: u32 = 0x1 << 6;
pub const HOST_CAP_CCC: u32 = 0x1 << 7;
pub const HOST_CAP_PART: u32 = 0x1 << 13;
pub const HOST_CAP_SSC: u32 = 0x1 << 14;
pub const HOST_CAP_PIO_MULTI: u32 = 0x1 << 15;
pub const HOST_CAP_FBS: u32 = 0x1 << 16;
pub const HOST_CAP_PMP: u32 = 0x1 << 17;
pub const HOST_CAP_ONLY: u32 = 0x1 << 18;
pub const HOST_CAP_CLO: u32 = 0x1 << 24;
pub const HOST_CAP_LED: u32 = 0x1 << 25;
pub const HOST_CAP_ALPM: u32 = 0x1 << 26;
pub const HOST_CAP_SSS: u32 = 0x1 << 27;
pub const HOST_CAP_MPS: u32 = 0x1 << 28;
pub const HOST_CAP_SNTF: u32 = 0x1 << 29;
pub const HOST_CAP_NCQ: u32 = 0x1 << 30;
pub const HOST_CAP_64: u32 = 0x1 << 31;
pub const HOST_CAP2_BOH: u32 = 0x1 << 0;
pub const HOST_CAP2_NVMHCI: u32 = 0x1 << 1;
pub const HOST_CAP2_APST: u32 = 0x1 << 2;
pub const HOST_CAP2_SDS: u32 = 0x1 << 3;
pub const HOST_CAP2_SADM: u32 = 0x1 << 4;
pub const HOST_CAP2_DESO: u32 = 0x1 << 5;

pub const HOST_RESET: u32 = 0x1 << 0;
pub const HOST_IRQ_EN: u32 = 0x1 << 1;
pub const HOST_MRSM: u32 = 0x1 << 2;
pub const HOST_AHCI_EN: u32 = 0x1 << 31;

pub const HOST_CAP: u64 = 0x0;
pub const HOST_CTL: u64 = 0x4;
pub const HOST_IRQ_STAT: u64 = 0x8;
pub const HOST_PORTS_IMPL: u64 = 0xC;
pub const HOST_VERSION: u64 = 0x10;
pub const HOST_EM_LOC: u64 = 0x1C;
pub const HOST_EM_CTL: u64 = 0x20;
pub const HOST_CAP2: u64 = 0x24;

pub const AHCI_MAX_PORTS: u32 = 32;
pub const AHCI_MAX_SG: u32 = 56;
pub const AHCI_DMA_BOUNDARY: u32 = 0xffffffff;
pub const AHCI_MAX_CMDS: u32 = 32;
pub const AHCI_CMD_SZ: u32 = 32;
pub const AHCI_CMD_SLOT_SZ: u32 = AHCI_MAX_CMDS * AHCI_CMD_SZ;
pub const AHCI_RX_FIS_SZ: u32 = 256;
pub const AHCI_CMD_TBL_CDB: u32 = 64;
pub const AHCI_CMD_TBL_HDR_SZ: u32 = 128;
pub const AHCI_CMD_TBL_SZ: u32 = AHCI_CMD_TBL_HDR_SZ + (AHCI_MAX_SG * 16);
pub const AHCI_CMD_TBL_AR_SZ: u32 = AHCI_CMD_TBL_SZ * AHCI_MAX_CMDS;
pub const AHCI_PORT_PRIV_DMA_SZ: u32 = AHCI_CMD_SLOT_SZ + AHCI_CMD_TBL_SZ + AHCI_RX_FIS_SZ;
pub const AHCI_PORT_PRIV_FBS_DMA_SZ: u32 =
    AHCI_CMD_SLOT_SZ + AHCI_CMD_TBL_AR_SZ + (AHCI_RX_FIS_SZ * 16);
pub const AHCI_MAX_BYTES_PER_SG: u32 = 4 * 1024 * 1024; // 4 MiB
pub const AHCI_MAX_BYTES_PER_TRANS: u32 = AHCI_MAX_SG * AHCI_MAX_BYTES_PER_SG;

pub const SATA_FLAG_FLUSH_EXT: u32 = 1024;
pub const SATA_FLAG_FLUSH: u32 = 512;
pub const SATA_FLAG_WCACHE: u32 = 256;

pub const READ_CMD: u32 = 0;
pub const WRITE_CMD: u32 = 1;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ahci_cmd_hdr {
    pub opts: u32,
    pub status: u32,
    pub tbl_addr_lo: u32,
    pub tbl_addr_hi: u32,
    pub reserved: [u32; 4],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ahci_sg {
    pub addr_lo: u32,
    pub addr_hi: u32,
    pub reserved: u32,
    pub flags_size: u32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ahci_ioport {
    pub port_mmio: u64,
    pub cmd_slot: *mut ahci_cmd_hdr,
    pub cmd_slot_dma: u64,
    pub rx_fis: u64,
    pub rx_fis_dma: u64,
    pub cmd_tbl: u64,
    pub cmd_tbl_dma: u64,
    pub cmd_tbl_sg: *mut ahci_sg,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ahci_blk_dev {
    pub lba48: bool,
    pub lba: u64,
    pub blksz: u64,
    pub queue_depth: u32,
    pub product: [u8; (ATA_ID_PROD_LEN + 1) as usize],
    pub serial: [u8; (ATA_ID_SERNO_LEN + 1) as usize],
    pub revision: [u8; (ATA_ID_FW_REV_LEN + 1) as usize],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ahci_device {
    pub mmio_base: u64,

    pub flags: u32,

    pub cap: u32,
    pub cap2: u32,
    pub version: u32,
    pub port_map: u32,

    pub pio_mask: u32,
    pub udma_mask: u32,

    pub n_ports: u8, // num of ports
    pub port_map_linkup: u32,
    pub port: [ahci_ioport; 32],
    pub port_idx: u8, // the enabled port

    pub blk_dev: ahci_blk_dev,
}
