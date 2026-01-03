use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};

// 8KiB
const BUFFER_SIZE: usize = 8 * 1024;

pub enum LoggerErr {
    NoSpaceErr,
    NoMessagesErr,
}

#[repr(C, align(64))]
struct CacheAligned<T>(T);

#[repr(C, align(64))]
pub struct AtomicRingBuffer {
    write_index: CacheAligned<AtomicUsize>,
    read_index: CacheAligned<AtomicUsize>,
    buffer: UnsafeCell<[u8; BUFFER_SIZE]>,
}

impl AtomicRingBuffer {
    pub fn new() -> Self {
        Self {
            write_index: CacheAligned(AtomicUsize::new(0)),
            read_index: CacheAligned(AtomicUsize::new(0)),
            buffer: UnsafeCell::new([0; BUFFER_SIZE]),
        }
    }

    pub fn push(&self, item: u8) -> Result<(), LoggerErr> {
        let write_idx = self.write_index.0.load(Ordering::Relaxed);
        let read_idx = self.read_index.0.load(Ordering::Acquire);

        if (write_idx + 1) % BUFFER_SIZE == read_idx {
            return Err(LoggerErr::NoSpaceErr);
        }

        unsafe {
            (*self.buffer.get())[write_idx] = item;
        }

        self.write_index
            .0
            .store((write_idx + 1) % BUFFER_SIZE, Ordering::Release);

        Ok(())
    }

    pub fn read(&self) -> Result<u8, LoggerErr> {
        let write_idx = self.write_index.0.load(Ordering::Acquire);
        let read_idx = self.read_index.0.load(Ordering::Relaxed);

        if write_idx == read_idx {
            return Err(LoggerErr::NoMessagesErr);
        }

        unsafe {
            let out = (*self.buffer.get())[read_idx];
            self.read_index
                .0
                .store((read_idx + 1) % BUFFER_SIZE, Ordering::Release);

            Ok(out)
        }
    }
}

impl core::default::Default for AtomicRingBuffer {
    fn default() -> Self {
        Self::new()
    }
}
