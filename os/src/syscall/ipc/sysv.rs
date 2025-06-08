use core::ops::Deref;

use alloc::task;
use hal::{addr::{VirtAddr, VirtAddrHal, VirtPageNumHal}, constant::{Constant, ConstantsHal}, pagetable::MapPerm};

use crate::{ipc::sysv::{self, ShmIdDs, IPC_PRIVATE}, mm::{vm::{MapFlags, UserVmFile, UserVmSpaceHal}, UserPtrRaw}, syscall::{mm::MmapFlags, SysError, SysResult}, task::current_task};

bitflags! {
    struct ShmGetFlags: i32 {
        /// Create a new segment. If this flag is not used, then shmget() will
        /// find the segment associated with key and check to see if the user
        /// has permission to access the segment.
        const IPC_CREAT = 0x200;
        /// This flag is used with IPC_CREAT to ensure that this call creates
        /// the segment.  If the segment already exists, the call fails.
        const IPC_EXCL = 0x400;
    }
}

bitflags! {
    struct ShmAtFlags: i32 {
        /// Attach the segment for read-only access.If this flag is not specified,
        /// the segment is attached for read and write access, and the process
        /// must have read and write permission for the segment.
        const SHM_RDONLY = 0x1000;
        /// round attach address to SHMLBA boundary
        const SHM_RND = 0x2000;
        /// Allow the contents of the segment to be executed.
        const SHM_EXEC = 0x8000;
    }
}

const IPC_RMID: i32 = 0;
const IPC_SET: i32 = 1;
// Copy information from the kernel data structure associated with `shmid`
// into the shmid_ds structure pointed to by buf.
const IPC_STAT: i32 = 2;

pub fn sys_shmget(key: i32, size: usize, shmflg: i32) -> SysResult {
    let task = current_task().unwrap();
    let shmflg = ShmGetFlags::from_bits_truncate(shmflg);
    log::info!("[sys_shmget] {key} {size} {:?}", shmflg);
    let rounded_up_sz = (size - 1 + Constant::PAGE_SIZE) & !(Constant::PAGE_SIZE - 1);
    if key == IPC_PRIVATE {
        let shm = sysv::SHM_MANAGER.alloc(rounded_up_sz, task.pid()).unwrap();
        return Ok(shm.get_id() as isize);
    }
    if let Some(shm) = sysv::SHM_MANAGER.get(key as usize) {
        if shmflg.contains(ShmGetFlags::IPC_CREAT | ShmGetFlags::IPC_EXCL) {
            return Err(SysError::EEXIST);
        }
        if shm.shmid_ds.lock().segsz < size {
            return Err(SysError::EINVAL);
        }
        return Ok(key as isize);
    }
    if shmflg.contains(ShmGetFlags::IPC_CREAT) {
        let shm = sysv::SHM_MANAGER.alloc_at(rounded_up_sz, task.pid(), key as usize).unwrap();
        return Ok(shm.get_id() as isize);
    } else {
        return Err(SysError::ENOENT);
    }
}

pub fn sys_shmat(shmid: i32, mut shmaddr: VirtAddr, shmflg: i32) -> SysResult {
    let shmflg = ShmAtFlags::from_bits_truncate(shmflg);
    log::info!("[sys_shmat] {shmid} {shmaddr:?} {:?}", shmflg);

    if shmaddr.page_offset() != 0 && !shmflg.contains(ShmAtFlags::SHM_RND) {
        return Err(SysError::EINVAL);
    }
    shmaddr = shmaddr.floor().start_addr();
    let mut perm = MapPerm::U | MapPerm::R | MapPerm::W;
    if shmflg.contains(ShmAtFlags::SHM_EXEC) {
        perm.insert(MapPerm::X);
    }
    if shmflg.contains(ShmAtFlags::SHM_RDONLY) {
        perm.remove(MapPerm::W);
    }
    if let Some(shm) = sysv::SHM_MANAGER.get(shmid as usize) {
        let task = current_task().unwrap();
        let mut vm = task.vm_space.lock();
        let ret = vm.alloc_anon_area(
            shmaddr, shm.shmid_ds.lock().segsz, perm, 
            MmapFlags::MAP_SHARED, 
            Some(shm.clone())
        )?;
        shm.shmid_ds.lock().attach(task.pid());
        log::info!("[sys_shmat] success: {:?}", ret);
        return Ok(ret.0 as isize)
    } else {
        return Err(SysError::EINVAL);
    }
}

pub fn sys_shmdt(shmaddr: VirtAddr) -> SysResult {
    log::info!("[sys_shmdt] {:?}", shmaddr);
    if shmaddr.page_offset() != 0 {
        // shmaddr is not aligned on a page boundary
        return Err(SysError::EINVAL);
    }
    let task = current_task().unwrap();
    let mut vm_space = task.vm_space.lock();
    if let Some(vma) = vm_space.get_area_ref(shmaddr) {
        if let UserVmFile::Shm(shm) = vma.file.clone() {
            assert!(vma.map_flags.contains(MapFlags::SHARED));
            let len = vma.range_va.clone().count();
            vm_space.unmap(shmaddr, len)?;
            shm.shmid_ds.lock().detach(task.pid());
            return Ok(0);
        } else {
            return Err(SysError::EINVAL);
        }
    } else {
        return Err(SysError::EINVAL);
    }
}

pub fn sys_shmctl(shmid: i32, op: i32, shmid_ds: UserPtrRaw<ShmIdDs>) -> SysResult {
    log::info!("[sys_shmctl] {} {} {:?}", shmid, op, shmid_ds);
    match op {
        IPC_STAT => {
            let task = current_task().unwrap();
            let shm = sysv::SHM_MANAGER.get(shmid as usize).ok_or(SysError::ENOENT)?;
            shmid_ds
                .ensure_write(&mut task.vm_space.lock())
                .ok_or(SysError::EINVAL)?
                .write(*shm.shmid_ds.lock());
            Ok(0)
        }
        IPC_RMID => {
            sysv::SHM_MANAGER.remove(shmid as usize).ok_or(SysError::ENOENT)?;
            Ok(0)
        }
        IPC_SET => {
            log::warn!("[sys_shmctl] unsupported IPC_SET");
            // todo
            Err(SysError::EINVAL)
        }
        _ => Err(SysError::EINVAL)
    }
}
