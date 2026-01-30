use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::ptr;

use common::constants::{SHARED_BASE_PHYSICAL_ADDR, SHARED_SIZE};
use common::shared_mem::SharedMemUserspace;

use std::sync::Once;

static INIT: Once = Once::new();

pub fn get_shared_mem() -> Option<SharedMemUserspace> {
    let mut out = None;

    INIT.call_once(|| unsafe {
        let mem = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_SYNC)
            .open("/dev/mem")
            .unwrap();

        let map_ptr = libc::mmap(
            ptr::null_mut(),
            SHARED_SIZE,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            mem.as_raw_fd(),
            SHARED_BASE_PHYSICAL_ADDR as libc::off_t,
        );

        if map_ptr == libc::MAP_FAILED {
            panic!("mmap failed!");
        }

        println!("Memory mapped at virtual address: {:p}", map_ptr);

        let shared_mem_ptr = map_ptr as *mut common::shared_mem::SharedMem;

        out = Some(SharedMemUserspace::from_ptr(shared_mem_ptr));
    });

    out
}
