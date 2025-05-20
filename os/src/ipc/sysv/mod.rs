mod shm;

pub use shm::{ShmObj, SHM_MANAGER, ShmIdDs};

/// ipc private
pub const IPC_PRIVATE: i32 = 0;

#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub struct IpcPerm {
    key: i32,
    uid: u32,
    gid: u32,
    cuid: u32,
    cgid: u32,
    mode: u16,
    seq: u16,
}
