//! a byte ring buffer
#![allow(missing_docs)]

use alloc::{vec, vec::Vec};
use core::cmp;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
enum RingBufferState {
    #[default]
    EMPTY,
    FULL,
    NORMAL,
}

pub struct RingBuffer {
    arr: Vec<u8>,
    // NOTE: When and only when `head` equals `tail`, `state` can only be `Full` or `Empty`.
    head: usize,
    tail: usize,
    state: RingBufferState,
}

impl RingBuffer {
    pub fn new(len: usize) -> Self {
        Self {
            arr: vec![0; len],
            head: 0,
            tail: 0,
            state: RingBufferState::EMPTY,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.state == RingBufferState::EMPTY
    }

    pub fn is_full(&self) -> bool {
        self.state == RingBufferState::FULL
    }

    /// Read as much as possible to fill `buf`.
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        if self.state == RingBufferState::EMPTY || buf.is_empty() {
            return 0;
        }

        let ret_len;
        let n = self.arr.len();
        if self.head < self.tail {
            ret_len = cmp::min(self.tail - self.head, buf.len());
            buf[..ret_len].copy_from_slice(&self.arr[self.head..self.head + ret_len]);
        } else {
            // also handles full
            ret_len = cmp::min(n - self.head + self.tail, buf.len());
            if ret_len <= (n - self.head) {
                buf[..ret_len].copy_from_slice(&self.arr[self.head..self.head + ret_len]);
            } else {
                let right_len = n - self.head;
                buf[..right_len].copy_from_slice(&self.arr[self.head..]);
                buf[right_len..ret_len].copy_from_slice(&self.arr[..(ret_len - right_len)]);
            }
        }
        self.head = (self.head + ret_len) % n;

        if self.head == self.tail {
            self.state = RingBufferState::EMPTY;
        } else {
            self.state = RingBufferState::NORMAL;
        }

        ret_len
    }

    /// Write as much as possible to fill the ring buffer.
    pub fn write(&mut self, buf: &[u8]) -> usize {
        if self.state == RingBufferState::FULL || buf.is_empty() {
            return 0;
        }

        let ret_len;
        let n = self.arr.len();
        if self.head <= self.tail {
            // also handles empty
            ret_len = cmp::min(n - (self.tail - self.head), buf.len());
            if ret_len <= (n - self.tail) {
                self.arr[self.tail..self.tail + ret_len].copy_from_slice(&buf[..ret_len]);
            } else {
                self.arr[self.tail..].copy_from_slice(&buf[..n - self.tail]);
                self.arr[..(ret_len - (n - self.tail))]
                    .copy_from_slice(&buf[n - self.tail..ret_len]);
            }
        } else {
            ret_len = cmp::min(self.head - self.tail, buf.len());
            self.arr[self.tail..self.tail + ret_len].copy_from_slice(&buf[..ret_len]);
        }
        self.tail = (self.tail + ret_len) % n;

        if self.head == self.tail {
            self.state = RingBufferState::FULL;
        } else {
            self.state = RingBufferState::NORMAL;
        }

        ret_len
    }

    pub fn dequeue(&mut self) -> Option<u8> {
        if self.is_empty() {
            return None;
        }

        let n = self.arr.len();
        let c = self.arr[self.head];
        self.head = (self.head + 1) % n;
        if self.head == self.tail {
            self.state = RingBufferState::EMPTY;
        } else {
            self.state = RingBufferState::NORMAL;
        }
        Some(c)
    }

    pub fn enqueue(&mut self, byte: u8) -> Option<()> {
        if self.is_full() {
            return None;
        }

        let n = self.arr.len();
        self.arr[self.tail] = byte;
        self.tail = (self.tail + 1) % n;
        if self.head == self.tail {
            self.state = RingBufferState::FULL;
        } else {
            self.state = RingBufferState::NORMAL;
        }
        Some(())
    }
}
