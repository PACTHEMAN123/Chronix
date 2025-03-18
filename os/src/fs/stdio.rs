//!Stdin & Stdout
use async_trait::async_trait;
use hal::print;
use alloc::boxed::Box;

use crate::fs::vfs::File;
use crate::mm::UserBuffer;
use crate::sbi::console_getchar;
use crate::task::suspend_current_and_run_next;
///Standard input
pub struct Stdin;
///Standard output
pub struct Stdout;

#[async_trait]
impl File for Stdin {
    fn inner(&self) -> &super::vfs::FileInner {
        panic!("[Stdin]: dont support get inner")
    }
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        false
    }
    async fn read(&self, mut user_buf: UserBuffer) -> usize {
        assert_eq!(user_buf.len(), 1);
        // busy loop
        let mut c: usize;
        loop {
            c = console_getchar();
            if c == 0 {
                suspend_current_and_run_next();
                continue;
            } else {
                break;
            }
        }
        let ch = c as u8;
        unsafe {
            user_buf.buffers[0].as_mut_ptr().write_volatile(ch);
        }
        1
    }
    async fn write(&self, _user_buf: UserBuffer) -> usize {
        panic!("Cannot write to stdin!");
    }
}

#[async_trait]
impl File for Stdout {
    fn inner(&self) -> &super::vfs::FileInner {
        panic!("[Stdout]: dont support get inner")
    }
    fn readable(&self) -> bool {
        false
    }
    fn writable(&self) -> bool {
        true
    }
    async fn read(&self, mut _user_buf: UserBuffer) -> usize {
        panic!("Cannot read from stdout!");
    }
    async fn write(&self, user_buf: UserBuffer) -> usize {
        for buffer in user_buf.buffers.iter() {
            print!("{}", core::str::from_utf8(*buffer).unwrap());
        }
        user_buf.len()
    }
}
