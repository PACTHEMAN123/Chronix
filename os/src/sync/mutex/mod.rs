use crate::sync::mutex::spin_rw_mutex::SpinRwMutex;

use self::spin_mutex::SpinMutex;
use hal::instruction::{Instruction, InstructionHal};
use hal::util::sie_guard::SieGuard;
/// spin_mutex
pub mod spin_mutex;
pub mod spin_rw_mutex;

/// SpinLock
pub type SpinLock<T> = SpinMutex<T, Spin>;
/// SpinNoIrqLock(Cannot be interrupted)
pub type SpinNoIrqLock<T> = SpinMutex<T, SpinNoIrq>;

/// SpinLock
pub type SpinRwLock<T> = SpinRwMutex<T, Spin>;
/// SpinNoIrqLock(Cannot be interrupted)
pub type SpinNoIrqRwLock<T> = SpinRwMutex<T, SpinNoIrq>;


/// Low-level support for mutex(spinlock, sleeplock, etc)
pub trait MutexSupport: {
    /// Guard data
    type GuardData;
    /// Called before lock() & try_lock()
    fn before_lock() -> Self::GuardData;
    /// Called when MutexGuard dropping
    fn after_unlock(_: &mut Self::GuardData);
    /// Clone 
    fn clone(_: &Self::GuardData) -> Self::GuardData;
}

/// Spin MutexSupport
pub struct Spin;

impl MutexSupport for Spin {
    type GuardData = ();
    #[inline(always)]
    fn before_lock() -> Self::GuardData {}
    #[inline(always)]
    fn after_unlock(_: &mut Self::GuardData) {}
    #[inline(always)]
    fn clone(_: &Self::GuardData) -> Self::GuardData {}
}

/// SpinNoIrq MutexSupport
pub struct SpinNoIrq;

impl MutexSupport for SpinNoIrq {
    type GuardData = SieGuard;
    #[inline(always)]
    fn before_lock() -> Self::GuardData {
        SieGuard::new()
    }
    #[inline(always)]
    fn after_unlock(_: &mut Self::GuardData) {}
    #[inline(always)]
    fn clone(a: &Self::GuardData) -> Self::GuardData {
        unsafe { core::mem::transmute_copy(a) }
    }
}
