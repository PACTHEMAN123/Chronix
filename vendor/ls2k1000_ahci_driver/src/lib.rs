#![no_std]
#![allow(dead_code, unused_assignments, unused_mut)]

pub mod drv_ahci;
pub mod libahci;
pub mod libata;
pub mod platform;

// use core::panic::PanicInfo;

// #[panic_handler]
// fn panic(_info: &PanicInfo) -> ! {
//     loop {}
// }

#[macro_export]
/// print string macro
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::platform::ahci_print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
/// println string macro
macro_rules! println {
    () => {
        $crate::platform::agci_print("\n");
    };
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::platform::ahci_print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}
