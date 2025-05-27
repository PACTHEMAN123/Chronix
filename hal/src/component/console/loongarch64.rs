pub use super::uart::{console_getchar, console_putchar};

impl core::fmt::Write for super::Stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            console_putchar(c as usize);
        }
        Ok(())
    }
}
