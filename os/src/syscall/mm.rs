//! memory related syscall
#![allow(missing_docs)]

use hal::{addr::VirtAddr, pagetable::MapPerm};
use log::info;

use crate::{config::PAGE_SIZE, mm::vm::UserVmSpaceHal, task::current_task};

use super::{SysError, SysResult};

bitflags! {
    // Defined in <bits/mman-linux.h>
    #[derive(Default)]
    pub struct MmapFlags: i32 {
        // Sharing types (must choose one and only one of these).
        /// Share changes.
        const MAP_SHARED = 0x01;
        /// Changes are private.
        const MAP_PRIVATE = 0x02;
        /// Share changes and validate
        const MAP_SHARED_VALIDATE = 0x03;
        const MAP_TYPE_MASK = 0x03;

        // Other flags
        /// Interpret addr exactly.
        const MAP_FIXED = 0x10;
        /// Don't use a file.
        const MAP_ANONYMOUS = 0x20;
        /// Don't check for reservations.
        const MAP_NORESERVE = 0x04000;
    }
}

bitflags! {
    // Defined in <bits/mman-linux.h>
    // NOTE: Zero bit flag is discouraged. See https://docs.rs/bitflags/latest/bitflags/#zero-bit-flags
    pub struct MmapProt: i32 {
        /// Page can be read.
        const PROT_READ = 0x1;
        /// Page can be written.
        const PROT_WRITE = 0x2;
        /// Page can be executed.
        const PROT_EXEC = 0x4;
    }
}

impl From<MmapProt> for MapPerm {
    fn from(prot: MmapProt) -> Self {
        let mut ret = Self::U;
        if prot.contains(MmapProt::PROT_READ) {
            ret |= Self::R;
        }
        if prot.contains(MmapProt::PROT_WRITE) {
            ret |= Self::W;
        }
        if prot.contains(MmapProt::PROT_EXEC) {
            ret |= Self::X;
        }
        ret
    }
}


/// syscall mmap
pub fn sys_mmap(
    addr: VirtAddr, 
    length: usize, 
    prot: i32, 
    flags: i32, 
    fd: usize, 
    offset: usize
) -> SysResult {
    let flags = MmapFlags::from_bits_truncate(flags);
    let prot = MmapProt::from_bits_truncate(prot);
    let perm = MapPerm::from(prot);
    let task = current_task().unwrap().clone();

    if length == 0 {
        return Err(SysError::EINVAL);
    } else if addr.0 == 0 && flags.contains(MmapFlags::MAP_FIXED) {
        return Err(SysError::EINVAL);
    } else if offset % PAGE_SIZE != 0 {
        return Err(SysError::EINVAL);
    }

    if flags.contains(MmapFlags::MAP_FIXED) {
        task.with_mut_vm_space(|m| m.unmap(addr, length))?;
    }

    

    match flags.intersection(MmapFlags::MAP_TYPE_MASK) {
        MmapFlags::MAP_SHARED => {
            if flags.contains(MmapFlags::MAP_ANONYMOUS) {
                // TODO: MAP_SHARED page fault should keep track of all vm areas
                log::error!("shared anonymous mapping");
                let start_va = task.with_mut_vm_space(|m| {
                    m.alloc_anon_area(addr, length, perm, flags, true)
                })?;
                Ok(start_va)
            } else {
                let file = task.with_fd_table(|t| t.get_file(fd))?;
                let start_va = task.with_mut_vm_space(|m| {
                    m.alloc_mmap_area(addr, length, perm, flags, file, offset)
                })?;
                Ok(start_va)
            }
        }
        MmapFlags::MAP_PRIVATE => {
            if flags.contains(MmapFlags::MAP_ANONYMOUS) {
                let start_va = task.with_mut_vm_space(|m| {
                    m.alloc_anon_area(addr, length, perm, flags, false)
                })?;
                Ok(start_va)
            } else {
                let file = task.with_fd_table(|t| t.get_file(fd))?;
                // TODO: private copy on write
                let start_va = task.with_mut_vm_space(|m| {
                    m.alloc_mmap_area(addr, length, perm, flags, file, offset)
                })?;
                Ok(start_va)
            }
        }
        _ => Err(SysError::EINVAL),
    }
}

/// syscall munmap
pub fn sys_munmap(addr: VirtAddr, length: usize) -> SysResult {
    // (todo) unmap the area in task's vm space
    let task = current_task().unwrap().clone();
    task.with_mut_vm_space(|m| {
        m.unmap(addr, length)
    })?;
    Ok(0)
}