#![no_std]
#![no_main]
#![feature(allocator_api)]
extern crate alloc;

use core::{alloc::Allocator, ops::{Add, Div, Range, Sub}};
use alloc::{alloc::Global, boxed::Box};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum RangeSetNodeStatus {
    Empty = 0, Half = 1, Full = 2
}

impl RangeSetNodeStatus {
    fn apply(&mut self, op: RangeSetOperation) {
        match op {
            RangeSetOperation::Fill => *self = Self::Full,
            RangeSetOperation::Clear => *self = Self::Empty
        }
    }

    fn combine(self, oth: Self) -> Self {
        if self != oth { Self::Half } else { oth }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RangeSetOperation {
    Fill, Clear
}

struct RangeSetNode<A: Allocator + Clone> {
    left: Option<Box<RangeSetNode<A>, A>>,
    right: Option<Box<RangeSetNode<A>, A>>,
    status: RangeSetNodeStatus,
}

impl<A: Allocator + Clone> RangeSetNode<A> {
    fn new() -> Self {
        Self { 
            left: None, 
            right: None, 
            status: RangeSetNodeStatus::Empty,
        }
    }

    fn operation<U>(&mut self, range: Range<U>, op_range: Range<U>, op: RangeSetOperation, alloc: A) 
        where U: Ord + Copy + Add<Output = U> + Sub<Output = U> + Div<usize, Output = U>
    {
        if op_range.start <= range.start && op_range.end >= range.end {
            self.status.apply(op);
            self.left = None;
            self.right = None;
            return;
        }
        
        let mid = range.start + (range.end - range.start) / 2;
        if range.start < op_range.end && mid > op_range.start {
            if self.left.is_none() {
                self.left = Some(Box::new_in(Self::new(), alloc.clone()));
            }
            self.left.as_mut().unwrap().operation(range.start..mid, op_range.clone(), op, alloc.clone());
        }
        if mid < op_range.end && range.end > op_range.start {
            if self.right.is_none() {
                self.right = Some(Box::new_in(Self::new(), alloc.clone()));
            }
            self.right.as_mut().unwrap().operation(mid..range.end, op_range.clone(), op, alloc.clone());
        }
        
        let lstatus = match &self.left {
            Some(node) => node.status,
            None => self.status
        };
        let rstatus = match &self.right {
            Some(node) => node.status,
            None => self.status
        };

        self.status = lstatus.combine(rstatus);
    }

    fn intersect<U>(&self, range: Range<U>, op_range: Range<U>) -> U 
        where U: Ord + Copy + Add<Output = U> + Sub<Output = U> + Div<usize, Output = U>
    {
        let zero = range.start - range.start;
        if self.status == RangeSetNodeStatus::Full {
            return range.end.min(op_range.end) - range.start.max(op_range.start);
        } else if self.status == RangeSetNodeStatus::Empty {
            return zero;
        }
        let mid = range.start + (range.end - range.start) / 2;
        let mut ret = zero;
        if range.start < op_range.end && mid > op_range.start {
            ret = ret + self.left.as_ref().unwrap().intersect(range.start..mid, op_range.clone());
        }
        if mid < op_range.end && range.end > op_range.start {
            ret = ret + self.right.as_ref().unwrap().intersect(mid..range.end, op_range.clone());
        }
        ret
    }
}

pub struct RangeSet<U, A = Global> 
    where U: Ord + Copy + Add<Output = U> + Sub<Output = U> + Div<usize, Output = U>, 
        A: Allocator + Clone
{
    root: RangeSetNode<A>,
    range: Range<U>,
    alloc: A
}

impl<U> RangeSet<U, Global> 
    where U: Ord + Copy + Add<Output = U> + Sub<Output = U> + Div<usize, Output = U>
{
    pub fn new(range: Range<U>) -> Self {
        Self { root: RangeSetNode::new(), range, alloc: Global }
    }
}

impl<U, A> RangeSet<U, A> 
    where U: Ord + Copy + Add<Output = U> + Sub<Output = U> + Div<usize, Output = U>, 
        A: Allocator + Clone
{
    pub fn new_in(range: Range<U>, alloc: A) -> Self {
        Self { root: RangeSetNode::new(), range, alloc }
    }

    pub fn insert(&mut self, range: Range<U>) {
        if range.is_empty() { return; }
        self.root.operation(self.range.clone(), range, RangeSetOperation::Fill, self.alloc.clone());
    }

    pub fn remove(&mut self, range: Range<U>) {
        if range.is_empty() { return; }
        self.root.operation(self.range.clone(), range, RangeSetOperation::Clear, self.alloc.clone());
    }

    pub fn contains(&self, range: Range<U>) -> bool {
        if range.is_empty() { return true; }
        let v = self.root.intersect(self.range.clone(), range.clone());
        return v == range.end - range.start
    }

    pub fn intersects(&self, range: Range<U>) -> bool {
        if range.is_empty() { return false; }
        let zero = range.start - range.start;
        let v = self.root.intersect(self.range.clone(), range.clone());
        return v != zero
    }
}
