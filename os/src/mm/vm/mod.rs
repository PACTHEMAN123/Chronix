mod vm_area;
mod vm_space;

#[allow(unused)]
pub use vm_area::{KernelVmArea, KernelVmAreaType, UserVmArea, UserVmAreaType, VmArea, VmAreaCowExt, VmAreaFrameExt, VmAreaPageFaultExt, MapPerm};
#[allow(unused)]
pub use vm_space::{KERNEL_SPACE, KernelVmSpace, UserVmSpace, VmSpace, VmSpaceHeapExt, VmSpacePageFaultExt, PageFaultAccessType, remap_test};
