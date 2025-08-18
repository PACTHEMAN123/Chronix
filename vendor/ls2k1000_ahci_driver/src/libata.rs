#![allow(dead_code, unused_assignments, unused_mut, non_upper_case_globals)]

pub const SETFEATURES_XFER: u8 = 0x03;
pub const XFER_UDMA_7: u8 = 0x47;
pub const XFER_UDMA_6: u8 = 0x46;
pub const XFER_UDMA_5: u8 = 0x45;
pub const XFER_UDMA_4: u8 = 0x44;
pub const XFER_UDMA_3: u8 = 0x43;
pub const XFER_UDMA_2: u8 = 0x42;
pub const XFER_UDMA_1: u8 = 0x41;
pub const XFER_UDMA_0: u8 = 0x40;
pub const XFER_MW_DMA_4: u8 = 0x24;
pub const XFER_MW_DMA_3: u8 = 0x23;
pub const XFER_MW_DMA_2: u8 = 0x22;
pub const XFER_MW_DMA_1: u8 = 0x21;
pub const XFER_MW_DMA_0: u8 = 0x20;
pub const XFER_SW_DMA_2: u8 = 0x12;
pub const XFER_SW_DMA_1: u8 = 0x11;
pub const XFER_SW_DMA_0: u8 = 0x10;
pub const XFER_PIO_6: u8 = 0x0E;
pub const XFER_PIO_5: u8 = 0x0D;
pub const XFER_PIO_4: u8 = 0x0C;
pub const XFER_PIO_3: u8 = 0x0B;
pub const XFER_PIO_2: u8 = 0x0A;
pub const XFER_PIO_1: u8 = 0x09;
pub const XFER_PIO_0: u8 = 0x08;
pub const XFER_PIO_SLOW: u8 = 0x00;

pub const ATA_CMD_DEV_RESET: u8 = 0x08;
pub const ATA_CMD_CHK_POWER: u8 = 0xE5;
pub const ATA_CMD_STANDBY: u8 = 0xE2;
pub const ATA_CMD_IDLE: u8 = 0xE3;
pub const ATA_CMD_EDD: u8 = 0x90;
pub const ATA_CMD_DOWNLOAD_MICRO: u8 = 0x92;
pub const ATA_CMD_DOWNLOAD_MICRO_DMA: u8 = 0x93;
pub const ATA_CMD_NOP: u8 = 0x00;
pub const ATA_CMD_FLUSH: u8 = 0xE7;
pub const ATA_CMD_FLUSH_EXT: u8 = 0xEA;
pub const ATA_CMD_ID_ATA: u8 = 0xEC;
pub const ATA_CMD_ID_ATAPI: u8 = 0xA1;
pub const ATA_CMD_SERVICE: u8 = 0xA2;
pub const ATA_CMD_READ: u8 = 0xC8;
pub const ATA_CMD_READ_EXT: u8 = 0x25;
pub const ATA_CMD_READ_QUEUED: u8 = 0x26;
pub const ATA_CMD_READ_STREAM_EXT: u8 = 0x2B;
pub const ATA_CMD_READ_STREAM_DMA_EXT: u8 = 0x2A;
pub const ATA_CMD_WRITE: u8 = 0xCA;
pub const ATA_CMD_WRITE_EXT: u8 = 0x35;
pub const ATA_CMD_WRITE_QUEUED: u8 = 0x36;
pub const ATA_CMD_WRITE_STREAM_EXT: u8 = 0x3B;
pub const ATA_CMD_WRITE_STREAM_DMA_EXT: u8 = 0x3A;
pub const ATA_CMD_WRITE_FUA_EXT: u8 = 0x3D;
pub const ATA_CMD_WRITE_QUEUED_FUA_EXT: u8 = 0x3E;
pub const ATA_CMD_FPDMA_READ: u8 = 0x60;
pub const ATA_CMD_FPDMA_WRITE: u8 = 0x61;
pub const ATA_CMD_NCQ_NON_DATA: u8 = 0x63;
pub const ATA_CMD_FPDMA_SEND: u8 = 0x64;
pub const ATA_CMD_FPDMA_RECV: u8 = 0x65;
pub const ATA_CMD_PIO_READ: u8 = 0x20;
pub const ATA_CMD_PIO_READ_EXT: u8 = 0x24;
pub const ATA_CMD_PIO_WRITE: u8 = 0x30;
pub const ATA_CMD_PIO_WRITE_EXT: u8 = 0x34;
pub const ATA_CMD_READ_MULTI: u8 = 0xC4;
pub const ATA_CMD_READ_MULTI_EXT: u8 = 0x29;
pub const ATA_CMD_WRITE_MULTI: u8 = 0xC5;
pub const ATA_CMD_WRITE_MULTI_EXT: u8 = 0x39;
pub const ATA_CMD_WRITE_MULTI_FUA_EXT: u8 = 0xCE;
pub const ATA_CMD_SET_FEATURES: u8 = 0xEF;
pub const ATA_CMD_SET_MULTI: u8 = 0xC6;
pub const ATA_CMD_PACKET: u8 = 0xA0;
pub const ATA_CMD_VERIFY: u8 = 0x40;
pub const ATA_CMD_VERIFY_EXT: u8 = 0x42;
pub const ATA_CMD_WRITE_UNCORR_EXT: u8 = 0x45;
pub const ATA_CMD_STANDBYNOW1: u8 = 0xE0;
pub const ATA_CMD_IDLEIMMEDIATE: u8 = 0xE1;
pub const ATA_CMD_SLEEP: u8 = 0xE6;
pub const ATA_CMD_INIT_DEV_PARAMS: u8 = 0x91;
pub const ATA_CMD_READ_NATIVE_MAX: u8 = 0xF8;
pub const ATA_CMD_READ_NATIVE_MAX_EXT: u8 = 0x27;
pub const ATA_CMD_SET_MAX: u8 = 0xF9;
pub const ATA_CMD_SET_MAX_EXT: u8 = 0x37;
pub const ATA_CMD_READ_LOG_EXT: u8 = 0x2F;
pub const ATA_CMD_WRITE_LOG_EXT: u8 = 0x3F;
pub const ATA_CMD_READ_LOG_DMA_EXT: u8 = 0x47;
pub const ATA_CMD_WRITE_LOG_DMA_EXT: u8 = 0x57;
pub const ATA_CMD_TRUSTED_NONDATA: u8 = 0x5B;
pub const ATA_CMD_TRUSTED_RCV: u8 = 0x5C;
pub const ATA_CMD_TRUSTED_RCV_DMA: u8 = 0x5D;
pub const ATA_CMD_TRUSTED_SND: u8 = 0x5E;
pub const ATA_CMD_TRUSTED_SND_DMA: u8 = 0x5F;
pub const ATA_CMD_PMP_READ: u8 = 0xE4;
pub const ATA_CMD_PMP_READ_DMA: u8 = 0xE9;
pub const ATA_CMD_PMP_WRITE: u8 = 0xE8;
pub const ATA_CMD_PMP_WRITE_DMA: u8 = 0xEB;
pub const ATA_CMD_CONF_OVERLAY: u8 = 0xB1;
pub const ATA_CMD_SEC_SET_PASS: u8 = 0xF1;
pub const ATA_CMD_SEC_UNLOCK: u8 = 0xF2;
pub const ATA_CMD_SEC_ERASE_PREP: u8 = 0xF3;
pub const ATA_CMD_SEC_ERASE_UNIT: u8 = 0xF4;
pub const ATA_CMD_SEC_FREEZE_LOCK: u8 = 0xF5;
pub const ATA_CMD_SEC_DISABLE_PASS: u8 = 0xF6;
pub const ATA_CMD_CONFIG_STREAM: u8 = 0x51;
pub const ATA_CMD_SMART: u8 = 0xB0;
pub const ATA_CMD_MEDIA_LOCK: u8 = 0xDE;
pub const ATA_CMD_MEDIA_UNLOCK: u8 = 0xDF;
pub const ATA_CMD_DSM: u8 = 0x06;
pub const ATA_CMD_CHK_MED_CRD_TYP: u8 = 0xD1;
pub const ATA_CMD_CFA_REQ_EXT_ERR: u8 = 0x03;
pub const ATA_CMD_CFA_WRITE_NE: u8 = 0x38;
pub const ATA_CMD_CFA_TRANS_SECT: u8 = 0x87;
pub const ATA_CMD_CFA_ERASE: u8 = 0xC0;
pub const ATA_CMD_CFA_WRITE_MULT_NE: u8 = 0xCD;
pub const ATA_CMD_REQ_SENSE_DATA: u8 = 0x0B;
pub const ATA_CMD_SANITIZE_DEVICE: u8 = 0xB4;
pub const ATA_CMD_ZAC_MGMT_IN: u8 = 0x4A;
pub const ATA_CMD_ZAC_MGMT_OUT: u8 = 0x9F;

pub const ATA_HOB: u8 = 0x80;
pub const ATA_NIEN: u8 = 0x02;
pub const ATA_LBA: u8 = 0x40;
pub const ATA_DEV1: u8 = 0x10;
pub const ATA_DEVICE_OBS: u8 = 0xA0;
pub const ATA_DEVCTL_OBS: u8 = 0x08;
pub const ATA_BUSY: u8 = 0x80;
pub const ATA_DRDY: u8 = 0x40;
pub const ATA_DF: u8 = 0x20;
pub const ATA_DSC: u8 = 0x10;
pub const ATA_DRQ: u8 = 0x08;
pub const ATA_CORR: u8 = 0x04;
pub const ATA_SENSE: u8 = 0x02;
pub const ATA_ERR: u8 = 0x01;
pub const ATA_SRST: u8 = 0x04;
pub const ATA_ICRC: u8 = 0x80;
pub const ATA_BBK: u8 = 0x80;
pub const ATA_UNC: u8 = 0x40;
pub const ATA_MC: u8 = 0x20;
pub const ATA_IDNF: u8 = 0x10;
pub const ATA_MCR: u8 = 0x08;
pub const ATA_ABORTED: u8 = 0x04;
pub const ATA_TRK0NF: u8 = 0x02;
pub const ATA_AMNF: u8 = 0x01;
pub const ATAPI_LFS: u8 = 0xF0;
pub const ATAPI_EOM: u8 = 0x02;
pub const ATAPI_ILI: u8 = 0x01;
pub const ATAPI_IO: u8 = 0x02;
pub const ATAPI_COD: u8 = 0x01;

pub const ATA_ID_WORDS: u32 = 256;
pub const ATA_ID_CONFIG: u32 = 0;
pub const ATA_ID_CYLS: u32 = 1;
pub const ATA_ID_HEADS: u32 = 3;
pub const ATA_ID_SECTORS: u32 = 6;
pub const ATA_ID_SERNO: u32 = 10;
pub const ATA_ID_BUF_SIZE: u32 = 21;
pub const ATA_ID_FW_REV: u32 = 23;
pub const ATA_ID_PROD: u32 = 27;
pub const ATA_ID_MAX_MULTSECT: u32 = 47;
pub const ATA_ID_DWORD_IO: u32 = 48;
pub const ATA_ID_TRUSTED: u32 = 48;
pub const ATA_ID_CAPABILITY: u32 = 49;
pub const ATA_ID_OLD_PIO_MODES: u32 = 51;
pub const ATA_ID_OLD_DMA_MODES: u32 = 52;
pub const ATA_ID_FIELD_VALID: u32 = 53;
pub const ATA_ID_CUR_CYLS: u32 = 54;
pub const ATA_ID_CUR_HEADS: u32 = 55;
pub const ATA_ID_CUR_SECTORS: u32 = 56;
pub const ATA_ID_MULTSECT: u32 = 59;
pub const ATA_ID_LBA_CAPACITY: u32 = 60;
pub const ATA_ID_SWDMA_MODES: u32 = 62;
pub const ATA_ID_MWDMA_MODES: u32 = 63;
pub const ATA_ID_PIO_MODES: u32 = 64;
pub const ATA_ID_EIDE_DMA_MIN: u32 = 65;
pub const ATA_ID_EIDE_DMA_TIME: u32 = 66;
pub const ATA_ID_EIDE_PIO: u32 = 67;
pub const ATA_ID_EIDE_PIO_IORDY: u32 = 68;
pub const ATA_ID_ADDITIONAL_SUPP: u32 = 69;
pub const ATA_ID_QUEUE_DEPTH: u32 = 75;
pub const ATA_ID_SATA_CAPABILITY: u32 = 76;
pub const ATA_ID_SATA_CAPABILITY_2: u32 = 77;
pub const ATA_ID_FEATURE_SUPP: u32 = 78;
pub const ATA_ID_MAJOR_VER: u32 = 80;
pub const ATA_ID_COMMAND_SET_1: u32 = 82;
pub const ATA_ID_COMMAND_SET_2: u32 = 83;
pub const ATA_ID_CFSSE: u32 = 84;
pub const ATA_ID_CFS_ENABLE_1: u32 = 85;
pub const ATA_ID_CFS_ENABLE_2: u32 = 86;
pub const ATA_ID_CSF_DEFAULT: u32 = 87;
pub const ATA_ID_UDMA_MODES: u32 = 88;
pub const ATA_ID_HW_CONFIG: u32 = 93;
pub const ATA_ID_SPG: u32 = 98;
pub const ATA_ID_LBA_CAPACITY_2: u32 = 100;
pub const ATA_ID_SECTOR_SIZE: u32 = 106;
pub const ATA_ID_WWN: u32 = 108;
pub const ATA_ID_LOGICAL_SECTOR_SIZE: u32 = 117;
pub const ATA_ID_COMMAND_SET_3: u32 = 119;
pub const ATA_ID_COMMAND_SET_4: u32 = 120;
pub const ATA_ID_LAST_LUN: u32 = 126;
pub const ATA_ID_DLF: u32 = 128;
pub const ATA_ID_CSFO: u32 = 129;
pub const ATA_ID_CFA_POWER: u32 = 160;
pub const ATA_ID_CFA_KEY_MGMT: u32 = 162;
pub const ATA_ID_CFA_MODES: u32 = 163;
pub const ATA_ID_DATA_SET_MGMT: u32 = 169;
pub const ATA_ID_SCT_CMD_XPORT: u32 = 206;
pub const ATA_ID_ROT_SPEED: u32 = 217;
pub const ATA_ID_PIO4: u32 = 2;

pub const ATA_ID_SERNO_LEN: u32 = 20;
pub const ATA_ID_FW_REV_LEN: u32 = 8;
pub const ATA_ID_PROD_LEN: u32 = 40;
pub const ATA_ID_WWN_LEN: u32 = 8;

pub const ATA_SECT_SIZE: u32 = 512;
pub const ATA_MAX_SECTORS_128: u32 = 128;
pub const ATA_MAX_SECTORS: u32 = 256;
pub const ATA_MAX_SECTORS_1024: u32 = 1024;
pub const ATA_MAX_SECTORS_LBA48: u32 = 65535;
pub const ATA_MAX_SECTORS_TAPE: u32 = 65535;

pub type sata_fis_type = u8;
pub const SATA_FIS_TYPE_SET_DEVICE_BITS_D2H: sata_fis_type = 161;
pub const SATA_FIS_TYPE_PIO_SETUP_D2H: sata_fis_type = 95;
pub const SATA_FIS_TYPE_BIST_ACT_BI: sata_fis_type = 88;
pub const SATA_FIS_TYPE_DATA_BI: sata_fis_type = 70;
pub const SATA_FIS_TYPE_DMA_SETUP_BI: sata_fis_type = 65;
pub const SATA_FIS_TYPE_DMA_ACT_D2H: sata_fis_type = 57;
pub const SATA_FIS_TYPE_REGISTER_D2H: sata_fis_type = 52;
pub const SATA_FIS_TYPE_REGISTER_H2D: sata_fis_type = 39;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct sata_fis_h2d {
    pub fis_type: u8,
    pub pm_port_c: u8,
    pub command: u8,
    pub features: u8,
    pub lba_low: u8,
    pub lba_mid: u8,
    pub lba_high: u8,
    pub device: u8,
    pub lba_low_exp: u8,
    pub lba_mid_exp: u8,
    pub lba_high_exp: u8,
    pub features_exp: u8,
    pub sector_count: u8,
    pub sector_count_exp: u8,
    pub res1: u8,
    pub control: u8,
    pub res2: [u8; 4],
}

pub fn ata_id_has_lba(id: &[u16]) -> bool {
    return (id[ATA_ID_CAPABILITY as usize] & (1 << 9)) != 0;
}

pub fn ata_id_queue_depth(id: &[u16]) -> u32 {
    return (id[ATA_ID_QUEUE_DEPTH as usize] & 0x1f + 1) as u32;
}

pub fn ata_id_u32(id: &[u16], n: u32) -> u32 {
    return (id[(n + 1) as usize] as u32) << 16 | (id[n as usize] as u32);
}

pub fn ata_id_u64(id: &[u16], n: u32) -> u64 {
    let mut val: u64 = 0;
    val |= (id[(n + 3) as usize] as u64) << 48;
    val |= (id[(n + 2) as usize] as u64) << 32;
    val |= (id[(n + 1) as usize] as u64) << 16;
    val |= (id[(n + 0) as usize] as u64) << 0;
    return val;
}

pub fn ata_id_has_flush(id: &[u16]) -> bool {
    if (id[ATA_ID_COMMAND_SET_2 as usize] & 0xc000) != 0x4000 {
        return false;
    }
    return (id[ATA_ID_COMMAND_SET_2 as usize] & (1 << 12)) != 0;
}

pub fn ata_id_has_flush_ext(id: &[u16]) -> bool {
    if (id[ATA_ID_COMMAND_SET_2 as usize] & 0xc000) != 0x4000 {
        return false;
    }
    return (id[ATA_ID_COMMAND_SET_2 as usize] & (1 << 13)) != 0;
}

pub fn ata_id_has_lba48(id: &[u16]) -> bool {
    if (id[ATA_ID_COMMAND_SET_2 as usize] & 0xc000) != 0x4000 {
        return false;
    }
    if ata_id_u64(id, ATA_ID_LBA_CAPACITY_2) == 0 {
        return false;
    }
    return (id[ATA_ID_COMMAND_SET_2 as usize] & (1 << 10)) != 0;
}

pub fn ata_id_has_wcache(id: &[u16]) -> bool {
    if (id[ATA_ID_COMMAND_SET_2 as usize] & 0xc000) != 0x4000 {
        return false;
    }
    return (id[ATA_ID_COMMAND_SET_1 as usize] & (1 << 5)) != 0;
}

pub fn ata_id_wcache_enabled(id: &[u16]) -> bool {
    if (id[ATA_ID_CSF_DEFAULT as usize] & 0xc000) != 0x4000 {
        return false;
    }
    return (id[ATA_ID_CFS_ENABLE_1 as usize] & (1 << 5)) != 0;
}

pub fn ata_id_n_sectors(id: &[u16]) -> u64 {
    if ata_id_has_lba(id) {
        if ata_id_has_lba48(id) {
            return ata_id_u64(id, ATA_ID_LBA_CAPACITY_2);
        } else {
            return ata_id_u32(id, ATA_ID_LBA_CAPACITY) as u64;
        }
    } else {
        return 0;
    }
}

pub fn ata_id_c_string(id: &[u16], s: &mut [u8], ofs: usize) {
    let len: usize = s.len() - 1;
    let len_id: usize = (len + 1) / 2;

    let src: &[u16] = id.get(ofs..ofs + len_id).unwrap();
    let dst: &mut [u8] = s.get_mut(..len).unwrap();

    for (i, &word) in src.iter().enumerate() {
        let bytes = word.to_be_bytes();
        let idx = i * 2;

        dst[idx..idx + 2].copy_from_slice(&bytes);
    }

    let last = dst.iter().rposition(|&byte| byte != b' ');

    let new_len = match last {
        Some(pos) => pos + 1,
        None => 0,
    };

    s[new_len] = b'\0';
}
