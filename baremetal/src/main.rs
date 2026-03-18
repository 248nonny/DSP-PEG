#![no_std]
#![no_main]

use core::arch::asm;

#[cfg(not(test))]
use core::panic::PanicInfo;

use common::shared_mem::types::{AtomicRingBufferErr, CoreID, CoreStatus};
use common::shared_mem::SharedMemBaremetal;
// use dsp_core::testing;

use common::constants::SHARED_BASE_PHYSICAL_ADDR;

use crate::utils::read_register;

mod init;
mod utils;

#[unsafe(export_name = "_rust_main")]
pub extern "C" fn rust_main() {
    let shared_mem: common::shared_mem::SharedMemBaremetal;

    unsafe {
        // Set up shared memory.
        let shared_mem_ptr = SHARED_BASE_PHYSICAL_ADDR as *mut common::shared_mem::SharedMem;

        // Set up default state.
        let shared_mem_default = common::shared_mem::SharedMem::initial_state();
        core::ptr::write_volatile(shared_mem_ptr, shared_mem_default);

        // Get owned, wrapped reference to shared memory.
        shared_mem = SharedMemBaremetal::from_ptr(shared_mem_ptr);

        // let mut status = CoreStatus::Init;
        let mut status;

        shared_mem.write_core_status(CoreID::Core1, CoreStatus::Idle);

        init::tables::setup_mmu();

        let mut counter: u64 = 0;

        loop {
            counter += 1;

            let r = shared_mem.write_message(
                CoreID::Core1,
                common::shared_mem::types::BaremetalMessage::TestSendingAU64Lol(counter),
            );

            // status = match &mut status {
            //     CoreStatus::Idle => CoreStatus::Running,
            //     _ => CoreStatus::Idle,
            // };

            status = match r {
                Ok(()) => CoreStatus::Running,
                Err(_) => CoreStatus::Idle,
            };

            shared_mem.write_core_status(CoreID::Core1, status);

            for _ in 1..2500000 {
                asm!("nop");
            }
        }
    }
}

unsafe extern "C" {
    static mut __bss_start: u64;
    static mut __bss_end: u64;
}

fn zero_bss() {
    unsafe {
        let mut bss: *mut u64 = &raw mut __bss_start;
        let end: *const u64 = &raw const __bss_end;
        while (bss as *const u64) < end {
            core::ptr::write_volatile(bss, 0);
            bss = bss.add(1);
        }
    }
}

#[panic_handler]
#[cfg(not(test))]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
