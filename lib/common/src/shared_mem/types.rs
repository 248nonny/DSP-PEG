use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use strum::FromRepr;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtomicRingBufferErr {
    NoSpaceErr,
    NoMessagesErr,
    AtomicConflictErr,
}

#[repr(u8)]
#[derive(Clone, Copy, FromRepr, Default)]
pub enum CoreID {
    #[default]
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
#[derive(FromRepr, Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum BaremetalMessage {
    #[default]
    Ping,
    TestSendingAU8Lol(u8),
    TestSendingAU64Lol(u64),
}

pub trait BufPayload: Copy + Default {}
impl<T> BufPayload for T where T: Copy + Default {}

pub trait FIFOBuffer<T: BufPayload, E> {
    fn new() -> Self;
    fn push(&self, item: T) -> Result<(), E>;
    fn read_one(&self) -> Result<T, E>;
}

trait CheckPow2<const N: usize> {
    // N must be a power of 2...
    const CHECK_POW_2: () = assert!(N > 0 && (N & (N - 1)) == 0,);
}

#[repr(C, align(64))]
pub struct AtomicRingBufferSPSC<T: BufPayload, const N: usize> {
    write_index: CacheAligned<AtomicUsize>,
    read_index: CacheAligned<AtomicUsize>,
    buffer: [UnsafeCell<T>; N],
}

#[repr(C)]
struct StatusElement<T> {
    element: UnsafeCell<T>,
    status: AtomicBool,
}

#[repr(C, align(64))]
pub struct AtomicRingBufferMPSC<T: BufPayload, const N: usize> {
    write_index: CacheAligned<AtomicUsize>,
    read_index: CacheAligned<AtomicUsize>,
    buffer: [CacheAligned<StatusElement<T>>; N],
}

impl<T: BufPayload, const N: usize> CheckPow2<N> for AtomicRingBufferSPSC<T, N> {
    const CHECK_POW_2: () = assert!(N > 0 && (N & (N - 1)) == 0,);
}

impl<T: BufPayload, const N: usize> AtomicRingBufferMPSC<T, N> {
    const CHECK_POW_2: () = assert!(N > 0 && (N & (N - 1)) == 0,);
}

impl<T: BufPayload, const N: usize> FIFOBuffer<T, AtomicRingBufferErr>
    for AtomicRingBufferSPSC<T, N>
{
    fn new() -> Self {
        Self::CHECK_POW_2;

        Self {
            write_index: CacheAligned(AtomicUsize::new(0)),
            read_index: CacheAligned(AtomicUsize::new(0)),
            buffer: core::array::from_fn(|_| UnsafeCell::new(T::default())),
        }
    }

    fn push(&self, item: T) -> Result<(), AtomicRingBufferErr> {
        let write_idx = self.write_index.0.load(Ordering::Relaxed);
        let read_idx = self.read_index.0.load(Ordering::Acquire);

        let next_write_idx = (write_idx + 1) & (N - 1);

        if next_write_idx == read_idx {
            return Err(AtomicRingBufferErr::NoSpaceErr);
        }

        unsafe {
            let write_ptr = self.buffer[write_idx].get();
            write_ptr.write(item);
            // (*self.buffer.get())[write_idx] = item;
        }

        self.write_index.0.store(next_write_idx, Ordering::Release);

        Ok(())
    }

    fn read_one(&self) -> Result<T, AtomicRingBufferErr> {
        let read_idx = self.read_index.0.load(Ordering::Relaxed);
        let write_idx = self.write_index.0.load(Ordering::Acquire);

        if write_idx == read_idx {
            return Err(AtomicRingBufferErr::NoMessagesErr);
        }

        unsafe {
            let read_ptr = self.buffer[read_idx].get();
            let out = read_ptr.read();
            self.read_index
                .0
                .store((read_idx + 1) & (N - 1), Ordering::Release);

            Ok(out)
        }
    }
}

impl<T: BufPayload, const N: usize> FIFOBuffer<T, AtomicRingBufferErr>
    for AtomicRingBufferMPSC<T, N>
{
    fn new() -> Self {
        Self::CHECK_POW_2;

        Self {
            write_index: CacheAligned(AtomicUsize::new(0)),
            read_index: CacheAligned(AtomicUsize::new(0)),
            buffer: core::array::from_fn(|_| {
                CacheAligned(StatusElement {
                    element: UnsafeCell::new(T::default()),
                    status: AtomicBool::new(false),
                })
            }),
        }
    }

    fn push(&self, item: T) -> Result<(), AtomicRingBufferErr> {
        let write_idx = self.write_index.0.load(Ordering::Acquire);
        let read_idx = self.read_index.0.load(Ordering::Acquire);

        let next_write_idx = (write_idx + 1) & (N - 1);

        if next_write_idx == read_idx {
            return Err(AtomicRingBufferErr::NoSpaceErr);
        }

        let update_write_idx_result = self.write_index.compare_exchange(
            write_idx,
            next_write_idx,
            Ordering::Acquire,
            Ordering::Relaxed,
        );

        match update_write_idx_result {
            Ok(_) => (),
            Err(_) => return Err(AtomicRingBufferErr::AtomicConflictErr),
        }

        unsafe {
            let element_ptr = self.buffer[write_idx].element.get();
            element_ptr.write(item);
        }

        self.buffer[write_idx].status.store(true, Ordering::Release);

        Ok(())
    }

    fn read_one(&self) -> Result<T, AtomicRingBufferErr> {
        let read_idx = self.read_index.0.load(Ordering::Relaxed);

        if !self.buffer[read_idx].status.load(Ordering::Acquire) {
            return Err(AtomicRingBufferErr::NoMessagesErr);
        }

        let out;
        unsafe {
            let element_ptr = self.buffer[read_idx].element.get();
            out = element_ptr.read();
        }

        self.buffer[read_idx].status.store(false, Ordering::Release);

        self.read_index
            .0
            .store((read_idx + 1) & (N - 1), Ordering::Release);
        Ok(out)
    }
}

impl<T: BufPayload, const N: usize> core::default::Default for AtomicRingBufferSPSC<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: BufPayload, const N: usize> core::default::Default for AtomicRingBufferMPSC<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
