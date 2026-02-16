pub mod types;

use core::sync::atomic::AtomicU8;
use core::sync::atomic::Ordering::{Acquire, Release};
use types::CacheAligned;

use crate::shared_mem::types::{AtomicRingBuffer, CoreID, CoreStatus};

#[repr(C, align(64))]
pub struct SharedMem {
    core_status: CacheAligned<[AtomicU8; 3]>,
    baremetal_message_buf: CacheAligned<AtomicRingBuffer<types::BaremetalMessage, 1024>>,
}

impl SharedMem {
    pub fn initial_state() -> Self {
        Self {
            core_status: CacheAligned::new(core::array::from_fn(|_| AtomicU8::new(0))),
            baremetal_message_buf: CacheAligned::new(AtomicRingBuffer::new()),
        }
    }

    pub fn read_core_status(&self, core_id: CoreID) -> Option<CoreStatus> {
        CoreStatus::from_repr(self.core_status[core_id as usize].load(Acquire))
    }
}

pub struct SharedMemUserspace {
    shared_mem: &'static SharedMem,
}

impl core::ops::Deref for SharedMemUserspace {
    type Target = SharedMem;
    fn deref(&self) -> &Self::Target {
        self.shared_mem
    }
}

pub struct SharedMemBaremetal {
    shared_mem: &'static SharedMem,
}

impl core::ops::Deref for SharedMemBaremetal {
    type Target = SharedMem;
    fn deref(&self) -> &Self::Target {
        self.shared_mem
    }
}

impl SharedMemUserspace {
    pub unsafe fn from_ptr(shared_mem_ptr: *mut SharedMem) -> Self {
        Self {
            shared_mem: unsafe { &*shared_mem_ptr },
        }
    }
}

impl SharedMemBaremetal {
    pub unsafe fn from_ptr(shared_mem_ptr: *mut SharedMem) -> Self {
        Self {
            shared_mem: unsafe { &*shared_mem_ptr },
        }
    }

    pub fn write_core_status(&self, core_id: CoreID, core_status: CoreStatus) {
        self.core_status[core_id as usize].store(core_status as u8, Release);
    }
}
