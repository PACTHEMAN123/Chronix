//! tty
//! adapt from Phoenix
#![allow(unused)]

use async_trait::async_trait;
use alloc::{boxed::Box, sync::{Arc, Weak}, vec::{self, Vec}};
use hal::console::console_getchar;
use spin::Once;
use strum::FromRepr;
use lazy_static::lazy_static;

use crate::{devices::CharDevice, drivers::serial::UART0, fs::{vfs::{inode::InodeMode, Dentry, DentryInner, File, FileInner, Inode, InodeInner}, Kstat, OpenFlags, StatxTimestamp, SuperBlock, Xstat, XstatMask}, mm::UserBuffer, sync::mutex::SpinNoIrqLock, syscall::{SysError, SysResult}, task::{current_task, suspend_current_and_run_next}};

/// Defined in <asm-generic/ioctls.h>
#[derive(FromRepr, Debug)]
#[repr(usize)]
enum TtyIoctlCmd {
    // For struct termios
    /// Gets the current serial port settings.
    TCGETS = 0x5401,
    /// Sets the serial port settings immediately.
    TCSETS = 0x5402,
    /// Sets the serial port settings after allowing the input and output
    /// buffers to drain/empty.
    TCSETSW = 0x5403,
    /// Sets the serial port settings after flushing the input and output
    /// buffers.
    TCSETSF = 0x5404,
    /// For struct termio
    /// Gets the current serial port settings.
    TCGETA = 0x5405,
    /// Sets the serial port settings immediately.
    #[allow(unused)]
    TCSETA = 0x5406,
    /// Sets the serial port settings after allowing the input and output
    /// buffers to drain/empty.
    #[allow(unused)]
    TCSETAW = 0x5407,
    /// Sets the serial port settings after flushing the input and output
    /// buffers.
    #[allow(unused)]
    TCSETAF = 0x5408,
    /// If the terminal is using asynchronous serial data transmission, and arg
    /// is zero, then send a break (a stream of zero bits) for between 0.25
    /// and 0.5 seconds.
    TCSBRK = 0x5409,
    /// Get the process group ID of the foreground process group on this
    /// terminal.
    TIOCGPGRP = 0x540F,
    /// Set the foreground process group ID of this terminal.
    TIOCSPGRP = 0x5410,
    /// Get window size.
    TIOCGWINSZ = 0x5413,
    /// Set window size.
    TIOCSWINSZ = 0x5414,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct WinSize {
    ws_row: u16,
    ws_col: u16,
    ws_xpixel: u16, // Unused
    ws_ypixel: u16, // Unused
}

impl WinSize {
    fn new() -> Self {
        Self {
            ws_row: 67,
            ws_col: 120,
            ws_xpixel: 0,
            ws_ypixel: 0,
        }
    }
}

/// Defined in <asm-generic/termbits.h>
#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct Termios {
    /// Input mode flags.
    pub iflag: u32,
    /// Output mode flags.
    pub oflag: u32,
    /// Control mode flags.
    pub cflag: u32,
    /// Local mode flags.
    pub lflag: u32,
    /// Line discipline.
    pub line: u8,
    /// control characters.
    pub cc: [u8; 19],
}

impl Termios {
    pub fn new() -> Self {
        Self {
            // IMAXBEL | IUTF8 | IXON | IXANY | ICRNL | BRKINT
            iflag: 0o66402,
            // OPOST | ONLCR
            oflag: 0o5,
            // HUPCL | CREAD | CSIZE | EXTB
            cflag: 0o2277,
            // IEXTEN | ECHOTCL | ECHOKE ECHO | ECHOE | ECHOK | ISIG | ICANON
            lflag: 0o105073,
            line: 0,
            cc: [
                3,   // VINTR Ctrl-C
                28,  // VQUIT
                127, // VERASE
                21,  // VKILL
                4,   // VEOF Ctrl-D
                0,   // VTIME
                1,   // VMIN
                0,   // VSWTC
                17,  // VSTART
                19,  // VSTOP
                26,  // VSUSP Ctrl-Z
                255, // VEOL
                18,  // VREPAINT
                15,  // VDISCARD
                23,  // VWERASE
                22,  // VLNEXT
                255, // VEOL2
                0, 0,
            ],
        }
    }

    pub fn is_icrnl(&self) -> bool {
        const ICRNL: u32 = 0o0000400;
        self.iflag & ICRNL != 0
    }

    pub fn is_echo(&self) -> bool {
        const ECHO: u32 = 0o0000010;
        self.lflag & ECHO != 0
    }
}

pub static TTY: Once<Arc<TtyFile>> = Once::new();

pub struct TtyFile {
    pub(crate) meta: SpinNoIrqLock<TtyMeta>,
    inner: FileInner,
}

impl TtyFile {
    pub fn new(dentry: Arc<dyn Dentry>) -> Arc<Self> {
        let meta = SpinNoIrqLock::new(TtyMeta {
            fg_pgid: 0 as u32, // warning: shell will use this process group id
            win_size: WinSize::new(),
            termios: Termios::new(),
        });
        let inner = FileInner {
            offset: 0.into(),
            dentry,
            flags: SpinNoIrqLock::new(OpenFlags::empty()),
        };
        Arc::new(Self { meta, inner })
    }
}

pub struct TtyMeta {
    fg_pgid: u32,
    win_size: WinSize,
    termios: Termios,
}

#[async_trait]
impl File for TtyFile {
    fn file_inner(&self) ->  &FileInner {
        &self.inner
    }

    fn readable(&self) -> bool {
        true
    }

    fn writable(&self) -> bool {
        true
    }

    async fn read(&self, buf: &mut [u8]) -> Result<usize, SysError> {
        let char_dev = UART0.clone();
        log::debug!("[tty file]: reading buf len: {}", buf.len());
        //let len = char_dev.read(buf).await;
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
        let len = 1;
        unsafe {
            buf.as_mut_ptr().write_volatile(ch);
        }
        
        let termios = self.meta.lock().termios;
        if termios.is_icrnl() {
            for i in 0..len {
                if buf[i] == '\r' as u8 {
                    buf[i] = '\n' as u8;
                }
            }
        }
        if termios.is_echo() {
            self.write(buf).await;
        }
        Ok(len)
    }

    async fn write(&self, buf: &[u8]) -> Result<usize, SysError> {
        let char_dev = UART0.clone();
        let len = char_dev.write(buf).await;
        Ok(len)
    }

    fn ioctl(&self, cmd: usize, arg: usize) -> SysResult {
        use TtyIoctlCmd::*;
        let Some(cmd) = TtyIoctlCmd::from_repr(cmd) else {
            log::error!("[TtyFile::ioctl] cmd {cmd} not included");
            unimplemented!()
        };
        log::debug!("[TtyFile::ioctl] cmd {:?}, value {:#x}", cmd, arg);
        match cmd {
            TCGETS | TCGETA => {
                unsafe {
                    *(arg as *mut Termios) = self.meta.lock().termios;
                }
                Ok(0)
            }
            TCSETS | TCSETSW | TCSETSF => {
                unsafe {
                    self.meta.lock().termios = *(arg as *const Termios);
                    log::debug!("termios {:#x?}", self.meta.lock().termios);
                }
                Ok(0)
            }
            TIOCGPGRP => {
                let fg_pgid = self.meta.lock().fg_pgid;
                log::debug!("[TtyFile::ioctl] get fg pgid {fg_pgid}");
                unsafe {
                    *(arg as *mut u32) = fg_pgid;
                }
                Ok(0)
            }
            TIOCSPGRP => {
                unsafe {
                    self.meta.lock().fg_pgid = *(arg as *const u32);
                }
                let fg_pgid = self.meta.lock().fg_pgid;
                log::debug!("[TtyFile::ioctl] set fg pgid {fg_pgid}");
                Ok(0)
            }
            TIOCGWINSZ => {
                let win_size = self.meta.lock().win_size;
                log::debug!("[TtyFile::ioctl] get window size {win_size:?}",);
                unsafe {
                    *(arg as *mut WinSize) = win_size;
                }
                Ok(0)
            }
            TIOCSWINSZ => {
                unsafe {
                    self.meta.lock().win_size = *(arg as *const WinSize);
                }
                Ok(0)
            }
            TCSBRK => Ok(0),
            _ => todo!(),
        }
    }
}

pub struct TtyInode {
    inner: InodeInner,
    char_dev: Arc<dyn CharDevice>,
}

impl TtyInode {
    pub fn new(super_block: Weak<dyn SuperBlock>) -> Arc<Self> {
        let mut inner = InodeInner::new(Some(super_block), InodeMode::CHAR, 0);
        // todo: device manager to get device id
        let char_dev = UART0.clone();
        Arc::new(Self { inner, char_dev })
    }
}

impl Inode for TtyInode {
    fn inode_inner(&self) -> &InodeInner {
        &self.inner
    }

    fn getattr(&self) -> crate::fs::Kstat {
        let inner = self.inode_inner();
        Kstat {
            st_dev: 0,
            st_ino: inner.ino as u64,
            st_mode: inner.mode.bits() as _,
            st_nlink: inner.nlink() as u32,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            _pad0: 0,
            st_size: inner.size() as _,
            _pad1: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_atime_sec: inner.atime().tv_sec as _,
            st_atime_nsec: inner.atime().tv_nsec as _,
            st_mtime_sec: inner.mtime().tv_sec as _,
            st_mtime_nsec: inner.mtime().tv_nsec as _,
            st_ctime_sec: inner.ctime().tv_sec as _,
            st_ctime_nsec: inner.ctime().tv_nsec as _,
        }
    }

    fn getxattr(&self, mask: crate::fs::XstatMask) -> crate::fs::Xstat {
        const SUPPORTED_MASK: XstatMask = XstatMask::from_bits_truncate({
            XstatMask::STATX_BLOCKS.bits |
            XstatMask::STATX_ATIME.bits |
            XstatMask::STATX_CTIME.bits |
            XstatMask::STATX_MTIME.bits |
            XstatMask::STATX_NLINK.bits |
            XstatMask::STATX_MODE.bits |
            XstatMask::STATX_SIZE.bits |
            XstatMask::STATX_INO.bits
        });
        let mask = mask & SUPPORTED_MASK;
        let inner = self.inode_inner();
        Xstat {
            stx_mask: mask.bits,
            stx_blksize: 0,
            stx_attributes: 0,
            stx_nlink: inner.nlink() as u32,
            stx_uid: 0,
            stx_gid: 0,
            stx_mode: inner.mode.bits() as _,
            stx_ino: inner.ino as u64,
            stx_size: inner.size() as _,
            stx_blocks: 0,
            stx_attributes_mask: 0,
            stx_atime: StatxTimestamp {
                tv_sec: inner.atime().tv_sec as _,
                tv_nsec: inner.atime().tv_nsec as _,
            },
            stx_btime: StatxTimestamp {
                tv_sec: 0,
                tv_nsec: 0,
            },
            stx_ctime: StatxTimestamp {
                tv_sec: inner.ctime().tv_sec as _,
                tv_nsec: inner.ctime().tv_nsec as _,
            },
            stx_mtime: StatxTimestamp {
                tv_sec: inner.mtime().tv_sec as _,
                tv_nsec: inner.mtime().tv_nsec as _,
            },
            stx_rdev_major: 0,
            stx_rdev_minor: 0,
            stx_dev_major: 0,
            stx_dev_minor: 0,
            stx_mnt_id: 0,
            stx_dio_mem_align: 0,
            std_dio_offset_align: 0,
            stx_subvol: 0,
            stx_atomic_write_unit_min: 0,
            stx_atomic_write_unit_max: 0,
            stx_atomic_write_segments_max: 0,
            stx_dio_read_offset_align: 0,
        }
    }
}

pub struct TtyDentry {
    inner: DentryInner,
}

impl TtyDentry {
    pub fn new(
        name: &str,
        parent: Option<Arc<dyn Dentry>>
    ) -> Arc<Self> {
        Arc::new(Self {
            inner: DentryInner::new(name, parent)
        })
    }
}

unsafe impl Send for TtyDentry {}
unsafe impl Sync for TtyDentry {}

impl Dentry for TtyDentry {
    fn dentry_inner(&self) -> &DentryInner {
        &self.inner
    }

    fn new(&self,
        name: &str,
        parent: Option<Arc<dyn Dentry>>,
    ) -> Arc<dyn Dentry> {
        let dentry = Arc::new(Self {
            inner: DentryInner::new(name, parent)
        });
        dentry
    }
    
    fn open(self: Arc<Self>, flags: OpenFlags) -> Option<Arc<dyn File>> {
        Some(TtyFile::new(self.clone()))
    }
}








