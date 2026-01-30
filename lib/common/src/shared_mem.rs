pub mod types;

use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::{Acquire, Release};
use types::CacheAligned;

#[repr(C, align(64))]
pub struct SharedMem {
    bare_metal_status: CacheAligned<AtomicUsize>,
}

impl SharedMem {
    pub fn initial_state() -> Self {
        Self {
            bare_metal_status: CacheAligned::new(AtomicUsize::new(0)),
        }
    }

    pub fn read_bare_metal_status(&self) -> usize {
        self.bare_metal_status.load(Acquire)
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

    pub fn write_bare_metal_status(&self, val: usize) {
        self.bare_metal_status.store(val, Release);
    }
}
