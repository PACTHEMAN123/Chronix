//! develop for finals
//! /proc/interrupts

use alloc::{collections::btree_map::BTreeMap, string::ToString};

use crate::{fs::tmpfs::inode::InodeContent, sync::mutex::SpinNoIrqLock};
use lazy_static::*;

lazy_static! {
    pub static ref IRQ_COUNTER: SpinNoIrqLock<IrqCounter> = SpinNoIrqLock::new(IrqCounter::new());
}

pub struct IrqCounter {
    counter: BTreeMap<usize, usize>,
}

impl IrqCounter {
    pub fn new() -> Self {
        Self {
            counter: BTreeMap::new()
        }
    }

    pub fn add_irq(&mut self, irq_no: usize) {
        if let Some(count) = self.counter.get_mut(&irq_no) {
            *count = (*count).saturating_add(1);
        } else {
            self.counter.insert(irq_no, 0);
        }
    }
}

pub struct Interrupts;

impl Interrupts {
    pub fn new() -> Self {
        Self {}
    }
}

impl InodeContent for Interrupts {
    fn serialize(&self) -> alloc::string::String {
        let mut res = "".to_string();
        for (&irq_no, &count) in IRQ_COUNTER.lock().counter.iter() {
            res += &irq_no.to_string();
            res += &": ".to_string();
            res += &count.to_string();
            res += &"\n".to_string();
        }
        res
    }
}

