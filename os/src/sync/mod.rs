//! Synchronization and interior mutability primitives
pub mod up;

pub use up::UPSafeCell;

extern crate alloc;
/// safe cell type for uniprocessor systems
pub mod mutex;
