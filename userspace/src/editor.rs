use std::os::unix::fs::OpenOptionsExt;
use std::time::{Duration, Instant};

// shared memory stuff.
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;
use std::ptr;
use std::ptr::{read_volatile, write_volatile};
use std::slice;

use eframe::egui;

fn read_at_offset(slice: &[u8], offset: usize) -> u8 {
    unsafe {
        let addr = slice.as_ptr().add(offset);
        read_volatile(addr)
    }
}

fn write_at_offset(slice: &'static mut [u8], offset: usize, val: u8) {
    unsafe {
        let addr = slice.as_mut_ptr().add(offset);
        write_volatile(addr, val);
    }
}

fn get_shared_slice() -> Result<&'static mut [u8], ()> {
    const SHARED_ADDR: usize = 0x10000000;
    const SHARED_SIZE: usize = 0x00100000;

    let f = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_SYNC)
        .open("/dev/mem")
        .unwrap();

    unsafe {
        let map_ptr = libc::mmap(
            ptr::null_mut(),
            SHARED_SIZE,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            f.as_raw_fd(),
            SHARED_ADDR as libc::off_t,
        );

        if map_ptr == libc::MAP_FAILED {
            panic!("mmap failed.");
        }

        println!("Memory mapped at virtual address: {:p}", map_ptr);

        let slice = slice::from_raw_parts_mut(map_ptr as *mut u8, SHARED_SIZE);

        Ok(slice)
    }
}

fn main() -> Result<(), eframe::Error> {
    println!("Hello, World!");

    let options = eframe::NativeOptions::default();

    let shared_mem = get_shared_slice().unwrap();

    // let screen_size = [1024.0, 600.0];
    // let options = eframe::NativeOptions {
    //     viewport: egui::ViewportBuilder::default()
    //         .with_inner_size(screen_size)
    //         .with_min_inner_size(screen_size)
    //         .with_max_inner_size(screen_size),
    //     ..Default::default()
    // };

    eframe::run_native(
        "Hello egui",
        options,
        Box::new(|_cc| {
            Ok(Box::new(MyApp {
                text: String::from("Hello, World!"),
                x: 0,
                last_bare_metal_poll: Instant::now(),
                shared_mem,
            }))
        }),
    )
}

struct MyApp {
    text: String,
    x: usize,
    last_bare_metal_poll: Instant,
    shared_mem: &'static mut [u8],
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.last_bare_metal_poll.elapsed().as_secs() >= 1 {
                self.last_bare_metal_poll = Instant::now();

                self.x += 2;
                self.text
                    .push_str(&format!("\n {}", read_at_offset(self.shared_mem, 0))[..]);
            }

            // ui.label(&self.text);

            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .max_height(400.0)
                .show(ui, |ui| {
                    ui.label(&self.text);
                });

            if ui.button("Click me!").clicked() {
                println!("Button was clicked!");
            }
        });

        ctx.request_repaint_after(Duration::from_secs(1));
    }
}
