/// ipc private
pub const IPC_PRIVATE: i32 = 0;

bitflags! {
    /// resource get request flags
    pub struct ShmFlags: i32 {
        /// create if key is nonexistent
        const IPC_CREAT = 00001000;
        /// fail if key exists
        const IPC_EXCL  = 00002000;
    }
}
