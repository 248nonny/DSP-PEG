pub mod types;

use core::sync::atomic::AtomicU8;
use core::sync::atomic::Ordering::{Acquire, Release};
use types::CacheAligned;

use log::info;

use crate::shared_mem::types::{
    AtomicRingBufferSPSC, BaremetalMessage, CoreID, CoreStatus, FIFOBuffer,
};

#[repr(C, align(64))]
pub struct SharedMem {
    core_status: CacheAligned<[AtomicU8; 3]>,
    message_bufs: [CacheAligned<AtomicRingBufferSPSC<BaremetalMessage, 1024>>; 3],
}

impl SharedMem {
    pub fn initial_state() -> Self {
        Self {
            core_status: CacheAligned::new(core::array::from_fn(|_| AtomicU8::new(0))),
            message_bufs: core::array::from_fn(|_| CacheAligned::new(AtomicRingBufferSPSC::new())),
        }
    }

    pub fn read_core_status(&self, core_id: CoreID) -> Option<CoreStatus> {
        CoreStatus::from_repr(self.core_status[core_id as usize].load(Acquire))
    }

    pub fn write_message(
        &self,
        from_core: CoreID,
        message: BaremetalMessage,
    ) -> Result<(), types::AtomicRingBufferErr> {
        info!("Sending message...");
        self.message_bufs[from_core as usize].push(message)
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

    pub fn read_message(
        &self,
        core: CoreID,
    ) -> Result<BaremetalMessage, types::AtomicRingBufferErr> {
        self.message_bufs[core as usize].read_one()
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
