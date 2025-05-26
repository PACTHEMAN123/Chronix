use core::ops::Range;

pub trait MmioMapperHal {
    fn map_mmio_area(&self, range: Range<usize>) -> Range<usize>;
}
