use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};
use strum::FromRepr;

pub enum AtomicRingBufferErr {
    NoSpaceErr,
    NoMessagesErr,
}

#[repr(u8)]
#[derive(Clone, Copy, FromRepr)]
pub enum CoreID {
    Core1 = 0,
    Core2 = 1,
    Core3 = 2,
}

#[repr(u8)]
#[derive(FromRepr, Clone, Copy, Debug)]
pub enum CoreStatus {
    Init = 0,
    Idle = 1,
    Running = 2,
}

#[repr(u8)]
#[derive(FromRepr)]
pub enum PedalMessageType {
    Info = 0,
    Warn = 1,
    Err = 2,
}

#[repr(C, align(64))]
pub struct CacheAligned<T>(T);

impl<T> core::ops::Deref for CacheAligned<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> CacheAligned<T> {
    pub fn new(v: T) -> Self {
        Self(v)
    }
}

pub trait AtomicBufPayload: Copy + Default {}
impl<T> AtomicBufPayload for T where T: Copy + Default {}

#[repr(C, align(64))]
pub struct AtomicRingBuffer<T: AtomicBufPayload, const N: usize> {
    write_index: CacheAligned<AtomicUsize>,
    read_index: CacheAligned<AtomicUsize>,
    buffer: UnsafeCell<[T; N]>,
}

impl<T: AtomicBufPayload, const N: usize> AtomicRingBuffer<T, N> {
    pub fn new() -> Self {
        Self {
            write_index: CacheAligned(AtomicUsize::new(0)),
            read_index: CacheAligned(AtomicUsize::new(0)),
            buffer: UnsafeCell::new([T::default(); N]),
        }
    }

    pub fn push(&self, item: T) -> Result<(), AtomicRingBufferErr> {
        let write_idx = self.write_index.0.load(Ordering::Relaxed);
        let read_idx = self.read_index.0.load(Ordering::Acquire);

        if (write_idx + 1) % N == read_idx {
            return Err(AtomicRingBufferErr::NoSpaceErr);
        }

        unsafe {
            (*self.buffer.get())[write_idx] = item;
        }

        self.write_index
            .0
            .store((write_idx + 1) % N, Ordering::Release);

        Ok(())
    }

    pub fn read(&self) -> Result<T, AtomicRingBufferErr> {
        let write_idx = self.write_index.0.load(Ordering::Acquire);
        let read_idx = self.read_index.0.load(Ordering::Relaxed);

        if write_idx == read_idx {
            return Err(AtomicRingBufferErr::NoMessagesErr);
        }

        unsafe {
            let out = (*self.buffer.get())[read_idx];
            self.read_index
                .0
                .store((read_idx + 1) % N, Ordering::Release);

            Ok(out)
        }
    }
}

impl<T: AtomicBufPayload, const N: usize> core::default::Default for AtomicRingBuffer<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(C)]
#[derive(Clone, Default, Copy)]
pub struct BaremetalMessage {
    hello: u8,
}
