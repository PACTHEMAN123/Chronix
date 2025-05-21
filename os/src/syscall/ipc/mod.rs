
/// System V IPC
pub mod sysv;

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

