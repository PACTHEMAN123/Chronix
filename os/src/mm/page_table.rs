use core::{cmp::min, ptr::slice_from_raw_parts_mut};

use alloc::{string::String, vec::Vec};
use hal::{addr::{PhysAddr, PhysAddrHal, PhysPageNumHal, VirtAddr, VirtAddrHal, VirtPageNumHal}, constant::{Constant, ConstantsHal}, pagetable::{PageTableEntryHal, PageTableHal}};

use crate::mm::vm::{PageFaultAccessType, UserVmSpaceHal};

use super::{allocator::FrameAllocator, vm::UserVmSpace, PageTable};

#[deprecated = "unsafe"]
/// translate a pointer to a mutable u8 Vec through page table
pub unsafe fn translated_byte_buffer(token: usize, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
    let page_table = PageTable::from_token(token, FrameAllocator);
    let mut start = ptr as usize;
    let end = start + len;
    let mut v = Vec::new();
    while start < end {
        let start_va = VirtAddr::from(start);
        let mut vpn = start_va.floor();
        let ppn = page_table.translate_vpn(vpn).unwrap();
        vpn += 1;
        let mut end_va: VirtAddr = vpn.start_addr();
        end_va = end_va.min(VirtAddr::from(end));
        if end_va.page_offset() == 0 {
            v.push(&mut ppn.start_addr().get_mut::<[u8; 4096]>()[start_va.page_offset()..]);
        } else {
            v.push(&mut ppn.start_addr().get_mut::<[u8; 4096]>()[start_va.page_offset()..end_va.page_offset()]);
        }
        start = end_va.0;
    }
    v
}

#[deprecated = "unsafe"]
/// Translate a pointer to a mutable u8 Vec end with `\0` through page table to a `String`
pub unsafe fn translated_str(token: usize, ptr: *const u8) -> String {
    let page_table = PageTable::from_token(token, FrameAllocator);
    let mut string = String::new();
    let mut va = ptr as usize;
    loop {
        let ch: u8 = *(page_table
            .translate_va(VirtAddr::from(va))
            .unwrap()
            .get_mut());
        if ch == 0 {
            break;
        }
        string.push(ch as char);
        va += 1;
    }
    string
}


#[allow(unused)]
#[deprecated = "unsafe"]
///Translate a generic through page table and return a reference
pub unsafe fn translated_ref<T>(token: usize, ptr: *const T) -> &'static T {
    let page_table = PageTable::from_token(token, FrameAllocator);
    page_table
        .translate_va(VirtAddr::from(ptr as usize))
        .unwrap()
        .get_ref()
}

#[deprecated = "unsafe"]
///Translate a generic through page table and return a mutable reference
pub unsafe fn translated_refmut<T>(token: usize, ptr: *mut T) -> &'static mut T {
    let page_table = PageTable::from_token(token, FrameAllocator);
    let va = ptr as usize;
    page_table
        .translate_va(VirtAddr::from(va))
        .unwrap()
        .get_mut()
}

/// translate user va by user_vm_space
pub fn translate_uva_checked(user_vm_space: &mut UserVmSpace, va: VirtAddr, access_type: PageFaultAccessType) -> Option<PhysAddr> {
    match user_vm_space.get_page_table().find_pte(va.floor()) {
        Some((pte, _)) if access_type.can_access(pte.flags()) => {
            Some(pte.ppn().start_addr() + va.page_offset())
        }
        _ => {
            user_vm_space.handle_page_fault(va, access_type).ok()?;
            Some(user_vm_space.translate_va(va).unwrap())
        }
    }
}


#[allow(unused)]
#[deprecated = "UserSlice is better"]
/// copy out 
pub fn copy_out<T: Copy>(user_vm_space: &mut UserVmSpace, mut dst: VirtAddr, mut src: &[T]) {
    assert!(dst.0 < Constant::USER_ADDR_SPACE.end);
    let size = size_of::<T>();
    // size is power of 2 and less than PAGE_SIZE, dst is aligned to size
    assert!((size & (size - 1) == 0) && (size <= Constant::PAGE_SIZE) && (dst.0 & (size - 1) == 0));
    let mut bytes = src.len() * size;
    while bytes > 0 {
        let step = min(bytes, Constant::PAGE_SIZE - dst.page_offset());
        let len = step / size;
        let dst_pa = translate_uva_checked(user_vm_space, dst, PageFaultAccessType::WRITE).unwrap();
        let dst_slice = unsafe {
            &mut *slice_from_raw_parts_mut(dst_pa.get_ptr(), len)
        };
        dst_slice.copy_from_slice(&src[..len]);
        src = &src[len..];
        dst += step;
        bytes -= step;
    }
}

#[allow(unused)]
/// copy out a str
pub fn copy_out_str(user_vm_space: &mut UserVmSpace, mut dst: VirtAddr, str: &str) {
    assert!(dst.0 < Constant::USER_ADDR_SPACE.end);
    let mut src = str.as_bytes();
    let mut bytes = src.len() + 1;

    loop {
        let step = min(bytes, Constant::PAGE_SIZE - dst.page_offset());
        if step == bytes {
            break;
        }
        let dst_pa = translate_uva_checked(user_vm_space, dst, PageFaultAccessType::WRITE).unwrap();
        let dst_slice = unsafe {
            &mut *slice_from_raw_parts_mut(dst_pa.get_ptr(), step)
        };
        dst_slice.copy_from_slice(&src[..step]);
        src = &src[step..];
        dst += step;
        bytes -= step;
    }

    let dst_pa = translate_uva_checked(user_vm_space, dst, PageFaultAccessType::WRITE).unwrap();
    let dst_slice = unsafe {
        &mut *slice_from_raw_parts_mut(dst_pa.get_ptr(), bytes)
    };
    dst_slice[..bytes-1].copy_from_slice(&src[..bytes-1]);
    dst_slice[bytes-1] = 0;

}

#[allow(unused)]
#[deprecated = "UserSlice is better"]
/// copy in
pub fn copy_in<T: Copy>(user_vm_space: &mut UserVmSpace, mut dst: &mut [T], mut src: VirtAddr) {
    let size = size_of::<T>();
    // size is power of 2 and less than PAGE_SIZE, dst is aligned to size
    assert!((size & (size - 1) == 0) && (size <= Constant::PAGE_SIZE) && (src.0 & (size - 1) == 0));
    let mut bytes = dst.len() * size;
    while bytes > 0 {
        let step = min(bytes, Constant::PAGE_SIZE - src.page_offset());
        let len = step / size;
        let src_pa = translate_uva_checked(user_vm_space, src, PageFaultAccessType::READ).unwrap();
        let src_slice = unsafe {
            &mut *slice_from_raw_parts_mut(src_pa.get_ptr(), len)
        };
        dst[..len].copy_from_slice(src_slice);
        dst = &mut dst[len..];
        src += step;
        bytes -= step;
    }
}

#[allow(unused, deprecated)]
/// copy in a str
pub unsafe fn copy_in_str(user_vm_space: &mut UserVmSpace, mut str: &mut str, mut src: VirtAddr) {
    let mut dst = str.as_bytes_mut();
    copy_in(user_vm_space, dst, src);
}

///Array of u8 slice that user communicate with os
#[deprecated]
pub struct UserBuffer {
    ///U8 vec
    pub buffers: Vec<&'static mut [u8]>,
}

#[allow(deprecated)]
impl UserBuffer {
    ///Create a `UserBuffer` by parameter
    pub fn new(buffers: Vec<&'static mut [u8]>) -> Self {
        Self { buffers }
    }
    ///Length of `UserBuffer`
    pub fn len(&self) -> usize {
        let mut total: usize = 0;
        for b in self.buffers.iter() {
            total += b.len();
        }
        total
    }
}

#[allow(deprecated)]
impl IntoIterator for UserBuffer {
    type Item = *mut u8;
    type IntoIter = UserBufferIterator;
    fn into_iter(self) -> Self::IntoIter {
        UserBufferIterator {
            buffers: self.buffers,
            current_buffer: 0,
            current_idx: 0,
        }
    }
}
/// Iterator of `UserBuffer`
#[deprecated]
pub struct UserBufferIterator {
    buffers: Vec<&'static mut [u8]>,
    current_buffer: usize,
    current_idx: usize,
}

#[allow(deprecated)]
impl Iterator for UserBufferIterator {
    type Item = *mut u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_buffer >= self.buffers.len() {
            None
        } else {
            let r = &mut self.buffers[self.current_buffer][self.current_idx] as *mut _;
            if self.current_idx + 1 == self.buffers[self.current_buffer].len() {
                self.current_idx = 0;
                self.current_buffer += 1;
            } else {
                self.current_idx += 1;
            }
            Some(r)
        }
    }
}