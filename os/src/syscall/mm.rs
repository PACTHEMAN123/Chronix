//! memory related syscall
#![allow(missing_docs)]

use core::time::Duration;

use alloc::sync::Arc;
use hal::{
    addr::{VirtAddr, VirtAddrHal, VirtPageNumHal},
    constant::{Constant, ConstantsHal},
    pagetable::{MapPerm, PageTableEntryHal, PageTableHal},
    println,
};
use log::info;

use crate::{
    config::PAGE_SIZE,
    ipc::sysv::SHM_MANAGER,
    mm::{
        vm::{self, MapFlags, UserVmArea, UserVmAreaType, UserVmFile, UserVmSpaceHal},
        UserSliceRaw, UserVmSpace,
    },
    sync::mutex::{
        spin_mutex::{self, MutexGuard},
        SpinNoIrq,
    },
    syscall::IoVec,
    task::{current_task, manager::TASK_MANAGER, task::TaskControlBlock},
    timer::get_current_time_duration,
    utils::timer::TimerGuard,
};

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
    offset: usize,
) -> SysResult {
    let flags = MmapFlags::from_bits_truncate(flags);
    let prot = MmapProt::from_bits_truncate(prot);
    let perm = MapPerm::from(prot);
    let task = current_task().unwrap().clone();
    // info!("[sys_mmap] addr: {:#x} length: {}, prot: {:?}, flags: {:?}, fd: {}, offset: {}", addr.0, length, prot, flags, fd, offset);
    if !flags.contains(MmapFlags::MAP_ANONYMOUS) {
        task.with_fd_table(|t| t.get_file(fd))?;
    }

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
                    m.alloc_anon_area(
                        addr,
                        length,
                        perm,
                        flags,
                        SHM_MANAGER.alloc(length, task.pid()),
                    )
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
                let start_va =
                    task.with_mut_vm_space(|m| m.alloc_anon_area(addr, length, perm, flags, None))?;
                // log::info!("[sys_mmap] private anonymous: {:?}", start_va);
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
pub fn sys_munmap(addr: VirtAddr, mut length: usize) -> SysResult {
    let task = current_task().unwrap().clone();
    if length == 0 {
        return Ok(0);
    }
    length = (length - 1 + Constant::PAGE_SIZE) & !(Constant::PAGE_SIZE - 1);
    task.with_mut_vm_space(|m| {
        let end_vpn = (addr + length).ceil();
        let mut cur_vpn = addr.floor();
        while cur_vpn < end_vpn {
            if let Ok(vma) = m.unmap(cur_vpn.start_addr(), length) {
                let new_vpn = vma.range_vpn().end;
                length -= (new_vpn.0 - cur_vpn.0) << Constant::PAGE_SIZE_BITS;
                cur_vpn = new_vpn;
            } else {
                break;
            }
        }
        Ok(())
    })?;
    Ok(0)
}

/// syscall mprotect
pub fn sys_mprotect(addr: VirtAddr, mut length: usize, prot: i32) -> SysResult {
    if addr.page_offset() != 0 || length == 0 || length % Constant::PAGE_SIZE != 0 {
        return Err(SysError::EINVAL);
    }
    let prot = MmapProt::from_bits_truncate(prot);
    let perm = MapPerm::from(prot);
    // println!("[mprotect] {:#x} {:#x} {:?}", addr.0, length, prot);
    let task = current_task().unwrap().clone();
    task.with_mut_vm_space(|vm| -> SysResult {
        let end_vpn = (addr + length).ceil();
        let mut cur_vpn = addr.floor();
        while cur_vpn < end_vpn {
            if let Ok(mut vma) = vm.unmap(cur_vpn.start_addr(), length) {
                let new_vpn = vma.range_vpn().end;
                length -= (new_vpn.0 - cur_vpn.0) << Constant::PAGE_SIZE_BITS;
                cur_vpn = new_vpn;
                vma.map_perm = perm;
                vm.push_area(vma, None);
            } else {
                break;
            }
        }
        Ok(0)
    })
}

/// syscall
pub fn sys_mremap(
    old_addr: VirtAddr,
    mut old_size: usize,
    mut new_size: usize,
    flags: i32,
    new_address: usize,
) -> SysResult {
    if old_addr.page_offset() != 0 {
        return Err(SysError::EINVAL);
    }
    old_size = (old_size - 1 + Constant::PAGE_SIZE) & !(Constant::PAGE_SIZE);
    new_size = (new_size - 1 + Constant::PAGE_SIZE) & !(Constant::PAGE_SIZE);
    let flags = MremapFlags::from_bits(flags).ok_or(SysError::EINVAL)?;
    if (flags.contains(MremapFlags::FIXED) | flags.contains(MremapFlags::DONTUNMAP))
        && !flags.contains(MremapFlags::MAYMOVE)
    {
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

    if flags.contains(MremapFlags::DONTUNMAP)
        && !old_area
            .get_mmap_flags()
            .contains(MmapFlags::MAP_PRIVATE | MmapFlags::MAP_ANONYMOUS)
    {
        return Err(SysError::EINVAL);
    }

    if flags.is_empty() || flags == MremapFlags::MAYMOVE {
        if old_size >= new_size {
            let mut old_area = vm.unmap(old_addr, old_size)?;
            old_area.shrink(old_size - new_size);
            vm.push_area(old_area, None);
            return Ok(old_size as isize);
        }
        if vm
            .check_free(old_addr + old_size, new_size - old_size)
            .is_ok()
        {
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
        vm.check_free(new_address, new_size)
            .map_err(|_| SysError::ENOMEM)?;
        new_address
    } else {
        VirtAddr::from(0)
    };

    new_addr = if let UserVmFile::File(file) = old_area.file.clone() {
        vm.alloc_mmap_area(
            new_addr,
            new_size,
            old_area.map_perm,
            old_area.get_mmap_flags(),
            file,
            0,
        )?
    } else if let UserVmFile::Shm(shm) = old_area.file.clone() {
        vm.alloc_anon_area(
            new_addr,
            new_size,
            old_area.map_perm,
            old_area.get_mmap_flags(),
            Some(shm),
        )?
    } else {
        assert!(!old_area.map_flags.contains(MapFlags::SHARED));
        vm.alloc_anon_area(
            new_addr,
            new_size,
            old_area.map_perm,
            old_area.get_mmap_flags(),
            None,
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

pub fn sys_process_vm_readv(
    pid: i32, lvec: usize, liovcnt: usize, rvec: usize, riovcnt: usize, flags: usize
) -> SysResult {
    const IOV_MAX: usize = 1024;
    let mut total_read_len: usize = 0; // 类型修正为 usize
    if liovcnt > IOV_MAX || riovcnt > IOV_MAX || flags != 0 {
        return Err(SysError::EINVAL);
    }
    
    // todo 权限检查
    let caller_task = current_task().unwrap();
    let target_task = TASK_MANAGER.get_task(pid as usize).ok_or(SysError::ESRCH)?;

    // 访问并检查本地和远程的iovec数组
    let lvec_slice_raw = UserSliceRaw::new(lvec as *const IoVec, liovcnt)
        .ensure_read(&mut caller_task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;
    let rvec_slice_raw = UserSliceRaw::new(rvec as *const IoVec, riovcnt)
        .ensure_read(&mut target_task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;

    let caller_vec: &[IoVec] = lvec_slice_raw.to_ref();
    let target_vec: &[IoVec] = rvec_slice_raw.to_ref();
    
    
    for iov in caller_vec.iter() {
        if iov.len > 0 {
            let _ = UserSliceRaw::new(iov.base as *mut u8, iov.len)
                .ensure_write(&mut caller_task.get_vm_space().lock())
                .ok_or(SysError::EFAULT)?;
        }
    }
    for iov in target_vec.iter() {
        if iov.len > 0 {
            let _ = UserSliceRaw::new(iov.base as *const u8, iov.len)
                .ensure_read(&mut target_task.get_vm_space().lock())
                .ok_or(SysError::EFAULT)?;
        }
    }

    let mut caller_idx = 0;
    let mut target_idx = 0;
    let mut caller_offset: usize = 0; 
    let mut target_offset: usize = 0; 

    while caller_idx < liovcnt && target_idx < riovcnt {
        let current_caller_iov = &caller_vec[caller_idx];
        let current_target_iov = &target_vec[target_idx];

        let caller_rem_len = current_caller_iov.len.saturating_sub(caller_offset);
        let target_rem_len = current_target_iov.len.saturating_sub(target_offset);
        let copy_len = caller_rem_len.min(target_rem_len);

        if copy_len == 0 {
            if caller_rem_len == 0 { caller_idx += 1; caller_offset = 0; }
            if target_rem_len == 0 { target_idx += 1; target_offset = 0; }
            continue;
        }

        let caller_start_addr = current_caller_iov.base + caller_offset;
        let target_start_addr = current_target_iov.base + target_offset;
        
        let res = copy_from_userspace_unsafe(
            &target_task,
            &caller_task,
            VirtAddr::from(caller_start_addr),
            VirtAddr::from(target_start_addr),
            copy_len
        )?;
        
        total_read_len += res as usize;
        caller_offset += res as usize;
        target_offset += res as usize;
    }

    Ok(total_read_len as isize)
}

fn copy_from_userspace_unsafe(target: &Arc<TaskControlBlock>, caller: &Arc<TaskControlBlock>, caller_start_addr: VirtAddr, target_start_addr: VirtAddr, len: usize) -> SysResult {
    let target_vm_space = target.get_vm_space().lock();
    let target_page_table = target_vm_space.get_page_table();
    let caller_vm_space = caller.get_vm_space().lock();
    let caller_page_table = caller_vm_space.get_page_table();
    let mut current_len: usize = 0;
    
    let kernel_direct_map_offset = hal::constant::Constant::KERNEL_ENTRY_PA; 

    while current_len < len {
        let target_va = VirtAddr::from(target_start_addr.0 + current_len);
        let caller_va = VirtAddr::from(caller_start_addr.0 + current_len);

        let target_page_offset = target_va.page_offset();
        let caller_page_offset = caller_va.page_offset();

        let (target_pte, _) = target_page_table.find_pte(target_va.floor()).ok_or(SysError::EFAULT)?;
        if !target_pte.is_valid() {
            return Err(SysError::EFAULT);
        }
        // 确保远程地址可读
        if !target_pte.flags().contains(MapPerm::R) {
            return Err(SysError::EFAULT);
        }
        let target_ppn = target_pte.ppn();

        let (caller_pte, _) = caller_page_table.find_pte(caller_va.floor()).ok_or(SysError::EFAULT)?;
        if !caller_pte.is_valid() || !caller_pte.flags().contains(MapPerm::W) {
            return Err(SysError::EFAULT);
        }
        let caller_ppn = caller_pte.ppn();

        let len_in_page = (PAGE_SIZE - target_page_offset).min(PAGE_SIZE - caller_page_offset).min(len - current_len);
        

        let target_phys_addr = target_ppn.0 << 12;
        let caller_phys_addr = caller_ppn.0 << 12;

        let target_ptr = (target_phys_addr + kernel_direct_map_offset) as *const u8 ;
        let caller_ptr =  (caller_phys_addr + kernel_direct_map_offset) as *mut u8 ;

        unsafe {
            let src = target_ptr.add(target_page_offset);
            let dst = caller_ptr.add(caller_page_offset);
            core::ptr::copy_nonoverlapping(src, dst, len_in_page);
        }
        current_len += len_in_page;
    }

    Ok(current_len as isize)
}


pub fn sys_process_vm_writev(
    pid: i32,
    lvec: usize,
    liovcnt: usize,
    rvec: usize,
    riovcnt: usize,
    flags: usize,
) -> SysResult {
    const IOV_MAX: usize = 1024;
    let mut total_write_len: usize = 0;
    if liovcnt > IOV_MAX || riovcnt > IOV_MAX || flags != 0 {
        return Err(SysError::EINVAL);
    }

    let caller_task = current_task().unwrap();
    let target_task = TASK_MANAGER.get_task(pid as usize).ok_or(SysError::ESRCH)?;

    let lvec_slice_raw = UserSliceRaw::new(lvec as *const IoVec, liovcnt)
        .ensure_read(&mut caller_task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;
    let rvec_slice_raw = UserSliceRaw::new(rvec as *const IoVec, riovcnt)
        .ensure_read(&mut target_task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;

    let caller_vec: &[IoVec] = lvec_slice_raw.to_ref();
    let target_vec: &[IoVec] = rvec_slice_raw.to_ref();

    // 预先检查所有内存区域
    for iov in caller_vec.iter() {
        if iov.len > 0 {
            let _ = UserSliceRaw::new(iov.base as *const u8, iov.len)
                .ensure_read(&mut caller_task.get_vm_space().lock())
                .ok_or(SysError::EFAULT)?;
        }
    }
    for iov in target_vec.iter() {
        if iov.len > 0 {
            let _ = UserSliceRaw::new(iov.base as *mut u8, iov.len)
                .ensure_write(&mut target_task.get_vm_space().lock())
                .ok_or(SysError::EFAULT)?;
        }
    }

    let mut caller_idx = 0;
    let mut target_idx = 0;
    let mut caller_offset: usize = 0;
    let mut target_offset: usize = 0;

    while caller_idx < liovcnt && target_idx < riovcnt {
        let current_caller_iov = &caller_vec[caller_idx];
        let current_target_iov = &target_vec[target_idx];

        let caller_rem_len = current_caller_iov.len.saturating_sub(caller_offset);
        let target_rem_len = current_target_iov.len.saturating_sub(target_offset);
        let copy_len = caller_rem_len.min(target_rem_len);

        if copy_len == 0 {
            if caller_rem_len == 0 {
                caller_idx += 1;
                caller_offset = 0;
            }
            if target_rem_len == 0 {
                target_idx += 1;
                target_offset = 0;
            }
            continue;
        }

        let caller_start_addr = current_caller_iov.base + caller_offset;
        let target_start_addr = current_target_iov.base + target_offset;

        let res = copy_to_userspace_unsafe(
            &target_task,
            &caller_task,
            VirtAddr::from(caller_start_addr),
            VirtAddr::from(target_start_addr),
            copy_len,
        )?;

        total_write_len += res as usize;
        caller_offset += res as usize;
        target_offset += res as usize;
    }

    Ok(total_write_len as isize)
}

fn copy_to_userspace_unsafe(
    target: &Arc<TaskControlBlock>,
    caller: &Arc<TaskControlBlock>,
    caller_start_addr: VirtAddr,
    target_start_addr: VirtAddr,
    len: usize,
) -> SysResult {
    let target_vm_space = target.get_vm_space().lock();
    let target_page_table = target_vm_space.get_page_table();
    let caller_vm_space = caller.get_vm_space().lock();
    let caller_page_table = caller_vm_space.get_page_table();
    let mut current_len: usize = 0;

    let kernel_direct_map_offset = hal::constant::Constant::KERNEL_ENTRY_PA;

    while current_len < len {
        let target_va = VirtAddr::from(target_start_addr.0 + current_len);
        let caller_va = VirtAddr::from(caller_start_addr.0 + current_len);

        let target_page_offset = target_va.page_offset();
        let caller_page_offset = caller_va.page_offset();

        let (target_pte, _) = target_page_table
            .find_pte(target_va.floor())
            .ok_or(SysError::EFAULT)?;
        if !target_pte.is_valid() || !target_pte.flags().contains(MapPerm::W) {
            return Err(SysError::EFAULT);
        }
        let target_ppn = target_pte.ppn();

        let (caller_pte, _) = caller_page_table
            .find_pte(caller_va.floor())
            .ok_or(SysError::EFAULT)?;
        if !caller_pte.is_valid() || !caller_pte.flags().contains(MapPerm::R) {
            return Err(SysError::EFAULT);
        }
        let caller_ppn = caller_pte.ppn();

        let len_in_page = (PAGE_SIZE - target_page_offset)
            .min(PAGE_SIZE - caller_page_offset)
            .min(len - current_len);

        let target_phys_addr = target_ppn.0 << 12;
        let caller_phys_addr = caller_ppn.0 << 12;

        let target_ptr = (target_phys_addr + kernel_direct_map_offset) as *mut u8;
        let caller_ptr = (caller_phys_addr + kernel_direct_map_offset) as *const u8;

        unsafe {
            let src = caller_ptr.add(caller_page_offset);
            let dst = target_ptr.add(target_page_offset);
            core::ptr::copy_nonoverlapping(src, dst, len_in_page);
        }
        current_len += len_in_page;
    }

    Ok(current_len as isize)
}
