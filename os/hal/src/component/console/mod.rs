#[macro_export]
/// print string macro
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::_print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
/// println string macro
macro_rules! println {
    () => {
        $crate::console::_print("\n");
    };
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::_print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

/// every arch needs to impl core::fmt::Write for Stdout
struct Stdout;

static CONSOLE_MUTEX: AtomicBool = AtomicBool::new(false); 

pub fn _print(args: core::fmt::Arguments) {
    loop {
        if CONSOLE_MUTEX.compare_exchange(
            false, true, 
            Ordering::Acquire, Ordering::Relaxed
        ).is_ok() {
            core::fmt::Write::write_fmt(&mut Stdout, args).unwrap();
            CONSOLE_MUTEX.store(false, Ordering::Release);
            break;
        }
    }
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }
    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let color = match record.level() {
            log::Level::Error => 31, // Red
            log::Level::Warn => 93,  // BrightYellow
            log::Level::Info => 34,  // Blue
            log::Level::Debug => 32, // Green
            log::Level::Trace => 90, // BrightBlack
        };
        println!(
            "\u{1B}[{}m[{:>5}] {}\u{1B}[0m",
            color,
            record.level(),
            record.args(),
        );
    }
    fn flush(&self) {}
}

pub fn init() {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(match option_env!("LOG") {
        Some("ERROR") => log::LevelFilter::Error,
        Some("WARN") => log::LevelFilter::Warn,
        Some("INFO") => log::LevelFilter::Info,
        Some("DEBUG") => log::LevelFilter::Debug,
        Some("TRACE") => log::LevelFilter::Trace,
        _ => log::LevelFilter::Info,
    });
}


#[cfg(target_arch = "riscv64")]
mod riscv64;

use core::sync::atomic::{AtomicBool, Ordering};

#[cfg(target_arch = "riscv64")]
#[allow(unused)]
pub use riscv64::*;

#[cfg(target_arch = "loongarch64")]
mod loongarch64;

#[cfg(target_arch = "loongarch64")]
#[allow(unused)]
pub use loongarch64::*;