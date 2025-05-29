#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::egui;

use hidapi::{DeviceInfo, HidDevice};

use usbhid::{enumerate_usb_devices, send_beep, send_command, send_revinfo};
use usbhid::find_mask_in_available_devices;
use usbhid::select_usb_device;
use usbhid::show_available_devices;
use usbhid::read_data;
use usbhid::open_device;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let _available_usb_devices: Vec<DeviceInfo> = enumerate_usb_devices();

    // For test purposes: the Honeywell 1602g:
    let vendor_id = 0x0c2e; // Example vendor ID
    let product_id = 0x0db3; // Example product ID

    // let device = open_device(vendor_id, product_id);
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "HON Scanner Control", // This will be the name of the window
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::<MyApp>::default())
        }),
    )
}

struct MyApp {
    name: String,
    age: u32,
    device: HidDevice,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Henk".to_owned(),
            age: 61,
            device: open_device(0x0c2e, 0x0db3),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("HON Scanner Control");
            if ui.button("DEFOVR.").clicked() {
                println!("Sending DEFOVR.");
                send_command(&self.device, String::from("DEFOVR."));
            }
            if ui.button("DEFALT.").clicked() {
                println!("Sending DEFALT.");
                send_command(&self.device, String::from("DEFALT."));
            }
            if ui.button("REVINF.").clicked() {
                println!("Sending REVINF.");
                send_revinfo(&self.device);
                read_data(&self.device);
            }
            if ui.button("BEEP").clicked() {
                println!("Sending a beep command");
                send_beep(&self.device);
            }

            // From the original example:
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Increment").clicked() {
                self.age += 1;
            }
            ui.label(format!("Hello '{}', age {}", self.name, self.age));

            // ui.image(egui::include_image!(
            //     "./ferris.png"
            // ));
        });
    }
}
