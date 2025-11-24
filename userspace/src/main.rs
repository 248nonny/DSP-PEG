use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    println!("Hello, World!");

    let options = eframe::NativeOptions::default();

    eframe::run_native("Hello egui", options, Box::new(|_cc| Ok(Box::new(MyApp))))
}

struct MyApp;

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Hello World!");
            if ui.button("Click me!").clicked() {
                println!("Button was clicked!");
            }
        });
    }
}
