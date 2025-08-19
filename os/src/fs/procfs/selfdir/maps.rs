use alloc::string::ToString;

use crate::fs::tmpfs::inode::InodeContent;


pub struct Maps;

impl InodeContent for Maps {
    fn serialize(&self) -> alloc::string::String {
        "".to_string()
    }
}