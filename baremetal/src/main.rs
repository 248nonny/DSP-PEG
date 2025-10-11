#![no_std]
#![no_main]

const SHARED_BASE: usize = 0x10000000;

const MAGIC_COUNTER: *mut u64 = (SHARED_BASE + 0x00) as *mut u64;

use core::arch::asm;
use core::panic::PanicInfo;

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

#[export_name = "_rust_main"]
pub extern "C" fn rust_main() {
    unsafe {
        let mut magic_counter = 0xAAAA_AAAA;
        core::ptr::write_volatile(MAGIC_COUNTER, magic_counter);

        loop {
            magic_counter += 1;
            core::ptr::write_volatile(MAGIC_COUNTER, magic_counter);

            for _ in 1..1000000 {
                asm!("nop");
            }

            for _ in 1..1000000 {
                asm!("nop");
            }
        }
    }
}

extern "C" {
    static mut __bss_start: u64;
    static mut __bss_end: u64;
}

unsafe fn zero_bss() {
    let mut bss = &raw mut __bss_start as *mut u64;
    let end = &raw const __bss_end as *const u64;
    while (bss as *const u64) < end {
        core::ptr::write_volatile(bss, 0);
        bss = bss.add(1);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
