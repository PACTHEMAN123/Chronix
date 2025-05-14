//! memory related syscall
#![allow(missing_docs)]

use hal::{addr::{VirtAddr, VirtPageNumHal}, constant::{Constant, ConstantsHal}, pagetable::MapFlags, println};
use log::info;

use crate::{config::PAGE_SIZE, mm::vm::{UserVmArea, UserVmAreaType, UserVmFile, UserVmSpaceHal}, task::current_task};

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

bitflags! {
    /// The flags bit-mask argument may be 0, or include the following flags
    pub struct MremapFlags: i32 {
        ///  By default, if there is not sufficient space to expand a mapping at its current location, then mremap()
        ///  fails.   If  this flag is specified, then the kernel is permitted to relocate the mapping to a new vir‐
        ///  tual address, if necessary.  If the mapping is relocated, then absolute pointers into the  old  mapping
        ///  location become invalid (offsets relative to the starting address of the mapping should be employed).
        const MAYMOVE    = 1 << 0;
        /// This  flag  serves a similar purpose to the MAP_FIXED flag of mmap(2).  If this flag is specified, then
        /// mremap() accepts a fifth argument, void *new_address, which specifies a page-aligned address  to  which
        /// the  mapping  must  be  moved.   Any previous mapping at the address range specified by new_address and
        /// new_size is unmapped.
        /// 
        /// If MREMAP_FIXED is specified, then MREMAP_MAYMOVE must also be specified.
        const FIXED      = 1 << 1;
        /// This flag, which must be used in conjunction with MREMAP_MAYMOVE, remaps a mapping to a new address but
        /// does not unmap the mapping at old_address.
        /// 
        /// The MREMAP_DONTUNMAP flag can be used only with private anonymous  mappings  (see  the  description  of
        /// MAP_PRIVATE and MAP_ANONYMOUS in mmap(2)).
        /// 
        /// After  completion,  any access to the range specified by old_address and old_size will result in a page
        /// fault.  The page fault will be handled by a userfaultfd(2) handler if the address is in a range  previ‐
        /// ously registered with userfaultfd(2).  Otherwise, the kernel allocates a zero-filled page to handle the
        /// fault.
        /// 
        /// The  MREMAP_DONTUNMAP  flag  may  be used to atomically move a mapping while leaving the source mapped.
        /// See NOTES for some possible applications of MREMAP_DONTUNMAP.
        const DONTUNMAP  = 1 << 2;
    }
}

impl From<MmapProt> for MapFlags {
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
    let perm = MapFlags::from(prot);
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
                let start_va = task.with_mut_vm_space(|m| {
                    m.alloc_anon_area(addr, length, perm, flags, Some(0))
                })?;
                Ok(start_va.0 as _)
            } else {
                let file = task.with_fd_table(|t| t.get_file(fd))?;
                let start_va = task.with_mut_vm_space(|m| {
                    m.alloc_mmap_area(addr, length, perm, flags, file, offset)
                })?;
                Ok(start_va.0 as _)
            }
        }
        MmapFlags::MAP_PRIVATE => {
            if flags.contains(MmapFlags::MAP_ANONYMOUS) {
                let start_va = task.with_mut_vm_space(|m| {
                    m.alloc_anon_area(addr, length, perm, flags, None)
                })?;
                // log::info!("[sys_mmap] private anonymous: {:#x}", start_va);
                Ok(start_va.0 as _)
            } else {
                let file = task.with_fd_table(|t| t.get_file(fd))?;
                // TODO: private copy on write
                let start_va = task.with_mut_vm_space(|m| {
                    m.alloc_mmap_area(addr, length, perm, flags, file, offset)
                })?;
                Ok(start_va.0 as _)
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

/// syscall mprotect
pub fn sys_mprotect(addr: VirtAddr, len: usize, prot: i32) -> SysResult {
    if addr.page_offset() != 0 || len == 0 || len % Constant::PAGE_SIZE != 0 {
        return Err(SysError::EINVAL);
    }
    let prot = MmapProt::from_bits_truncate(prot);
    let perm = MapFlags::from(prot);
    // log::info!("[mprotect] {:#x} {:#x} {:?}", addr.0, len, prot);
    let task = current_task().unwrap().clone();
    task.with_mut_vm_space(|vm| -> SysResult {
        let mut vma = vm.unmap(addr, len)?;
        vma.map_flags = perm;
        vm.push_area(vma, None);
        Ok(0)
    })
}

/// syscall
pub fn sys_mremap(
    old_addr: VirtAddr, mut old_size: usize, mut new_size: usize, 
    flags: i32, new_address: usize
) -> SysResult {
    if old_addr.page_offset() != 0 {
        return Err(SysError::EINVAL);
    }
    old_size = (old_size - 1 + Constant::PAGE_SIZE) & !(Constant::PAGE_SIZE);
    new_size = (new_size - 1 + Constant::PAGE_SIZE) & !(Constant::PAGE_SIZE);
    let flags = MremapFlags::from_bits(flags).ok_or(SysError::EINVAL)?;
    if (flags.contains(MremapFlags::FIXED) | flags.contains(MremapFlags::DONTUNMAP)) && !flags.contains(MremapFlags::MAYMOVE) {
        return Err(SysError::EINVAL);
    }
    
    let task = current_task().unwrap().clone();
    let vm_space = task.vm_space.clone();
    let mut vm = vm_space.lock();
    let old_area = vm.get_area_view(old_addr).ok_or(SysError::EINVAL)?;
    if old_area.range_va.end.0 - old_addr.0 < old_size {
        return Err(SysError::EINVAL);
    }
    if old_area.vma_type != UserVmAreaType::Mmap {
        return Err(SysError::EINVAL);
    }

    if flags.contains(MremapFlags::DONTUNMAP) && !old_area.mmap_flags.contains(MmapFlags::MAP_PRIVATE | MmapFlags::MAP_ANONYMOUS) {
        return Err(SysError::EINVAL);
    }

    if flags.is_empty() || flags == MremapFlags::MAYMOVE {
        if old_size >= new_size {
            let mut old_area = vm.unmap(old_addr, old_size)?;
            old_area.shrink(old_size - new_size);
            vm.push_area(old_area, None);
            return Ok(old_size as isize);
        }
        if vm.check_free(old_addr + old_size, new_size-old_size).is_ok() {
            let mut old_area = vm.unmap(old_addr, old_size)?;
            old_area.extend(new_size - old_size);
            vm.push_area(old_area, None);
            return Ok(old_size as isize);
        }
        if flags.is_empty() {
            return Err(SysError::ENOMEM);
        }
    }

    let mut new_addr = if flags.contains(MremapFlags::FIXED) {
        let new_address = VirtAddr::from(new_address);
        vm.check_free(new_address, new_size).map_err(|_| SysError::ENOMEM)?;
        new_address
    } else {
        VirtAddr::from(0)
    };

    new_addr = if let UserVmFile::File(file) = old_area.file.clone() {
        vm.alloc_mmap_area(
            new_addr, new_size, old_area.map_perm, old_area.mmap_flags, file, 0
        )?
    } else if let UserVmFile::Shm(shm) = old_area.file.clone() {
        vm.alloc_anon_area(
            new_addr, new_size, old_area.map_perm, old_area.mmap_flags, Some(shm.get_id())
        )?
    } else {
        vm.alloc_anon_area(
            new_addr, new_size, old_area.map_perm, old_area.mmap_flags, None
        )?
    };
    
    let mut new_area = vm.unmap(new_addr, new_size).unwrap();
    let mut old_area = vm.unmap(old_addr, old_size)?;
    old_area.move_frames_to(&mut new_area);
    vm.push_area(new_area, None);
    if flags.contains(MremapFlags::DONTUNMAP) {
        vm.push_area(old_area, None);
    }

    Ok(new_addr.0 as isize)
}