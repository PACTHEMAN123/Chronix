impl core::fmt::Write for crate::component::console::Stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            #[allow(deprecated)]
            sbi_rt::legacy::console_putchar(c as usize);
        }
        Ok(())
    }
}