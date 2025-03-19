impl core::fmt::Write for crate::component::console::Stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            #[allow(deprecated)]
            sbi_rt::legacy::console_putchar(c as usize);
        }
        Ok(())
    }
}

/// use sbi call to putchar in console (qemu uart handler)
pub fn console_putchar(c: usize) {
    #[allow(deprecated)]
    sbi_rt::legacy::console_putchar(c);
}

/// use sbi call to getchar from console (qemu uart handler)
pub fn console_getchar() -> usize {
    #[allow(deprecated)]
    sbi_rt::legacy::console_getchar()
}