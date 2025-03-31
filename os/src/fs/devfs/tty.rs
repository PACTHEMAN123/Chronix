//! tty
//! adapt from Phoenix
#![allow(unused)]

use async_trait::async_trait;
use alloc::{boxed::Box, vec::Vec};
use strum::FromRepr;

use crate::{drivers::serial::UART0, fs::vfs::{File, FileInner}, mm::UserBuffer, sync::mutex::SpinNoIrqLock, syscall::SysResult};

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

pub struct TtyFile {
    pub(crate) meta: SpinNoIrqLock<TtyMeta>,
}

pub struct TtyMeta {
    fg_pgid: u32,
    win_size: WinSize,
    termios: Termios,
}

#[async_trait]
impl File for TtyFile {
    fn inner(&self) ->  &FileInner {
        panic!("[tty]: tty file have no inner");
    }

    fn readable(&self) -> bool {
        true
    }

    fn writable(&self) -> bool {
        true
    }

    async fn read(&self, buf: &mut [u8]) -> usize {
        let char_dev = UART0.clone();
        let len = char_dev.read(buf).await;
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
        len
    }

    async fn write(&self, buf: &[u8]) -> usize {
        let char_dev = UART0.clone();
        let len = char_dev.write(buf).await;
        len
    }

    fn ioctl(&self, cmd: usize, arg: usize) -> SysResult {
        use TtyIoctlCmd::*;
        let Some(cmd) = TtyIoctlCmd::from_repr(cmd) else {
            log::error!("[TtyFile::ioctl] cmd {cmd} not included");
            unimplemented!()
        };
        log::info!("[TtyFile::ioctl] cmd {:?}, value {:#x}", cmd, arg);
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
                    log::info!("termios {:#x?}", self.meta.lock().termios);
                }
                Ok(0)
            }
            TIOCGPGRP => {
                let fg_pgid = self.meta.lock().fg_pgid;
                log::info!("[TtyFile::ioctl] get fg pgid {fg_pgid}");
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
                log::info!("[TtyFile::ioctl] set fg pgid {fg_pgid}");
                Ok(0)
            }
            TIOCGWINSZ => {
                let win_size = self.meta.lock().win_size;
                log::info!("[TtyFile::ioctl] get window size {win_size:?}",);
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









