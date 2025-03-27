//! Adapted from MankorOS

#![no_std]
#![no_main]
#![feature(map_try_insert)]

use alloc::collections::BTreeMap;
use core::ops::{Add, Range};

extern crate alloc;

#[derive(Clone, Debug)]
struct Node<U, V> {
    pub end: U,
    pub value: V,
}

/// A range map that stores range as key.
///
/// # Panic
///
/// Range is clipped or range is empty.
#[derive(Clone)]
pub struct RangeMap<U: Ord + Copy + Add<usize>, V>(BTreeMap<U, Node<U, V>>);

impl<U: Ord + Copy + Add<usize, Output = U>, V> RangeMap<U, V> {
    pub const fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn try_insert(&mut self, range: Range<U>, value: V) -> Result<&mut V, V> {
        debug_assert!(!range.is_empty());
        if let Some((_xstart, Node { end: xend, .. })) = self.0.range(..range.end).next_back() {
            // if this happens:
            // xstart.......xend
            //        start.........end
            if *xend > range.start {
                log::error!("[range map] try_insert error");
                return Err(value);
            }
        }
        let start = range.start;
        let end = range.end;
        let node = self.0.try_insert(start, Node { end, value }).ok().unwrap();
        Ok(&mut node.value)
    }

    /// Find range which satisfies that `key` is in [start, end).
    pub fn get(&self, key: U) -> Option<&V> {
        let (_, Node { end, value }) = self.0.range(..=key).next_back()?;
        if *end > key {
            return Some(value);
        }
        None
    }

    /// Find range which satisfies that `key` is in [start, end).
    pub fn get_mut(&mut self, key: U) -> Option<&mut V> {
        let (_, Node { end, value }) = self.0.range_mut(..=key).next_back()?;
        if *end > key {
            return Some(value);
        }
        None
    }

    /// Find range which satisfies that `key` is in [start, end).
    pub fn get_key_value(&self, key: U) -> Option<(Range<U>, &V)> {
        let (&start, Node { end, value }) = self.0.range(..=key).next_back()?;
        if *end > key {
            return Some((start..*end, value));
        }
        None
    }

    /// Find range which satisfies that `key` is in [start, end).
    pub fn get_key_value_mut(&mut self, key: U) -> Option<(Range<U>, &mut V)> {
        let (&start, Node { end, value }) = self.0.range_mut(..=key).next_back()?;
        if *end > key {
            return Some((start..*end, value));
        }
        None
    }

    /// Find a free range in [start, end).
    pub fn find_free_range(&self, range: Range<U>, size: usize) -> Option<Range<U>> {
        debug_assert!(!range.is_empty());
        debug_assert!(size != 0);
        let mut start = range.start;
        if start + size > range.end {
            return None;
        }
        if let Some((&n_start, node)) = self.0.range(..=start).next_back() {
            if node.end > start {
                // if node satisfies n_start <= start < n_end
                start = start.min(n_start);
            }
        }
        let mut last_end = start;
        for (&n_start, node) in self.0.range(start..range.end) {
            if last_end + size <= n_start {
                break;
            }
            if node.end + size > range.end {
                return None;
            }
            last_end = node.end;
        }
        debug_assert!(last_end + size <= range.end);
        Some(last_end..(last_end + size))
    }

    /// Check whether range is free.
    pub fn is_range_free(&self, range: Range<U>) -> Result<(), ()> {
        if range.is_empty() {
            return Err(());
        }
        if let Some((_, node)) = self.0.range(..=range.start).next_back() {
            if node.end > range.start {
                return Err(());
            }
        }
        if self.0.range(range).next().is_some() {
            return Err(());
        }
        Ok(())
    }

    /// Return the value whose key is a range that contains the `range` passed
    /// in.
    pub fn range_contain(&self, range: Range<U>) -> Option<&V> {
        let (_, Node { end, value }) = self.0.range(..=range.start).next_back()?;
        if *end >= range.end {
            return Some(value);
        }
        None
    }

    /// Return the mut value whose key has a range that contains the `range`
    /// passed in.
    pub fn range_contain_mut(&mut self, range: Range<U>) -> Option<&mut V> {
        let (_, Node { end, value }) = self.0.range_mut(..=range.start).next_back()?;
        if *end >= range.end {
            return Some(value);
        }
        None
    }

    /// Return the value whose key has a range that matches the `range` passed
    /// in.
    pub fn range_match(&self, range: Range<U>) -> Option<&V> {
        let (start, Node { end, value }) = self.0.range(..=range.start).next_back()?;
        if *start == range.start && *end == range.end {
            return Some(value);
        }
        None
    }

    /// Force remove by specify a range.
    ///
    /// # Panic
    ///
    /// Panic if the range is not exactly match.
    pub fn force_remove_one(&mut self, range: Range<U>) -> V {
        let Node { end: n_end, value } = self.0.remove(&range.start).unwrap();
        assert!(n_end == range.end);
        value
    }

    /// Extend a segment from back.
    ///
    /// # Panic
    ///
    /// The segment pointed by `start` must exist.
    pub fn extend_back(&mut self, range: Range<U>) -> Result<(), ()> {
        let node = self.0.get(&range.start).unwrap();
        self.is_range_free(node.end..range.end)?;

        let node = self.0.get_mut(&range.start).unwrap();
        node.end = range.end;
        Ok(())
    }

    /// Reduce a segment backwards. Return the range reduced when success, or
    /// error when fail.
    ///
    /// Will automatically remove the range when its length becomes zero.
    ///
    /// # Panic
    ///
    /// The segment pointed by `start` must exist.
    pub fn reduce_back(&mut self, start: U, new_end: U) -> Result<(), ()> {
        let node = self.0.get_mut(&start).unwrap();
        let _node_end = node.end;
        if start <= new_end && new_end < node.end {
            if start == new_end {
                self.0.remove(&start).unwrap();
            } else {
                node.end = new_end;
            }
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (Range<U>, &V)> {
        self.0.iter().map(|(&s, n)| {
            let r = s..n.end;
            (r, &n.value)
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Range<U>, &mut V)> {
        self.0.iter_mut().map(|(&s, n)| {
            let r = s..n.end;
            (r, &mut n.value)
        })
    }

    pub fn range(&self, r: Range<U>) -> impl Iterator<Item = (Range<U>, &V)> {
        self.0.range(r).map(|(&s, n)| {
            let r = s..n.end;
            (r, &n.value)
        })
    }

    pub fn range_mut(&mut self, r: Range<U>) -> impl Iterator<Item = (Range<U>, &mut V)> {
        self.0.range_mut(r).map(|(&s, n)| {
            let r = s..n.end;
            (r, &mut n.value)
        })
    }
}
