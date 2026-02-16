#![no_std]
#![no_main]

const SHARED_BASE: usize = 0x10000000;

const MAGIC_COUNTER: *mut u64 = (SHARED_BASE + 0x00) as *mut u64;

#[cfg(target_arch = "aarch64")]
use core::arch::asm;
use core::panic::PanicInfo;

use common::shared_mem::types::{CoreID, CoreStatus};
use common::shared_mem::SharedMemBaremetal;
// use dsp_core::testing;

use common::constants::SHARED_BASE_PHYSICAL_ADDR;

mod boot {
    use core::arch::global_asm;
    global_asm!(
        "
            .section .text._start
            .globl _start
        _start:
            ldr x0, = _stack_start_1
            mov sp, x0
            bl _rust_main
            "
    );
}

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

        let mut status = CoreStatus::Init;

        loop {
            shared_mem.write_core_status(CoreID::Core1, status);

            status = match &mut status {
                CoreStatus::Idle => CoreStatus::Running,
                _ => CoreStatus::Idle,
            };

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
        let mut bss = &raw mut __bss_start as *mut u64;
        let end = &raw const __bss_end as *const u64;
        while (bss as *const u64) < end {
            core::ptr::write_volatile(bss, 0);
            bss = bss.add(1);
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
