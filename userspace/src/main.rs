use std::time::{Duration, Instant};

// shared memory stuff.
use eframe::egui;

use crate::pedal_controller::PedalController;

mod init;
mod pedal_controller;
mod ui;

fn main() -> Result<(), eframe::Error> {
    println!("Hello, World!");

    let options = eframe::NativeOptions::default();

    // let screen_size = [1024.0, 600.0];
    // let options = eframe::NativeOptions {
    //     viewport: egui::ViewportBuilder::default()
    //         .with_inner_size(screen_size)
    //         .with_min_inner_size(screen_size)
    //         .with_max_inner_size(screen_size),
    //     ..Default::default()
    // };

    let pedal_controller = init::init();

    eframe::run_native(
        "Hello egui",
        options,
        Box::new(|_cc| {
            Ok(Box::new(MyApp {
                text: String::from("Hello, World!"),
                x: 0,
                last_bare_metal_poll: Instant::now(),
                pedal_controller,
            }))
        }),
    )
}

struct MyApp<PC: PedalController> {
    text: String,
    x: usize,
    last_bare_metal_poll: Instant,
    pedal_controller: PC,
}

impl<PC: PedalController> eframe::App for MyApp<PC> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.last_bare_metal_poll.elapsed().as_secs() >= 1 {
                self.last_bare_metal_poll = Instant::now();

                self.x += 2;
            }

            // while let Ok(x) = self.atomic_ring_buffer.read() {
            //     println!("{}", x);
            //     // self.text.push(x as char);
            // }

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
