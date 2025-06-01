#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
// #![allow(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::egui;

use hidapi::{DeviceInfo, HidDevice};

use usbhid::{enumerate_usb_devices, send_beep, send_command, send_revinfo};
// use usbhid::find_mask_in_available_devices;
// use usbhid::select_usb_device;
// use usbhid::show_available_devices;
use usbhid::read_data;
use usbhid::open_device;

/// 
/// 
/// # Arguments 
/// 
/// * `pid`: USB Vendor ID
/// * `vid`: USB Product ID
/// 
/// returns: String 
/// 
/// # Examples 
/// 
/// pid_vid_to_hexstr(0x0c2e, 0x0db3)
/// "0xc2e, 0xdb3"
/// 
/// ```
fn pid_vid_to_hexstr(pid: u16, vid:u16) -> String {
    format!("0x{:x}:0x{:x}", pid, vid)
}

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    // let _available_usb_devices: Vec<DeviceInfo> = enumerate_usb_devices();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([500.0, 400.0]),
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
    name: String,                   // From the original test. Delete this later
    age: u32,                       // From the original test. Delete this later
    devices: Vec<DeviceInfo>,       // To store all discovered USB devices.
    device: Option<HidDevice>,      // To store the selected device.
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Henk".to_owned(),
            age: 61,
            devices: enumerate_usb_devices(),
            device: None,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("HON Scanner Control");
            ui.separator();
            
            // Create a table
            egui::Grid::new("Available USB devices")
                .striped(true)
                .show(
                    ui,
                    |ui| {
                        // Add the header
                        ui.label(egui::RichText::new("Name").strong());
                        ui.label(egui::RichText::new("Vendor").strong());
                        ui.label(egui::RichText::new("vid:pid").strong());
                        ui.end_row();

                        // Add data rows, which will show information about the available devices.
                        for device in &self.devices {
                            if ui.label(device.product_string().unwrap_or("")).clicked() {
                                println!("Selected device: {}", device.product_string().unwrap_or(""));
                                self.device = Some(open_device(device.vendor_id(), device.product_id()).expect("Failed to open device"));
                            }
                            ui.label(device.product_string().unwrap_or(""));
                            ui.label(device.manufacturer_string().unwrap_or(""));
                            ui.label(pid_vid_to_hexstr(device.vendor_id(), device.product_id()));
                            ui.end_row();
                        }

                    }
                );

            ui.separator();
            if ui.button("Refresh list of connected USB devices").clicked() {
                self.devices = enumerate_usb_devices();
            }
            ui.separator();

            if ui.button("DEFOVR.").clicked() {
                println!("Sending DEFOVR.");
                let dev = self.device.as_mut().unwrap();
                send_command(dev, String::from("DEFOVR."));
            }


            if ui.button("DEFALT.").clicked() {
                println!("Sending DEFALT.");
                let dev = self.device.as_mut().unwrap();
                send_command(dev, String::from("DEFALT."));
            }
            if ui.button("REVINF.").clicked() {
                println!("Sending REVINF.");
                let dev = self.device.as_mut().unwrap();
                send_revinfo(dev);
                read_data(dev);
            }
            if ui.button("BEEP").clicked() {
                println!("Sending a beep command");
                let dev = self.device.as_mut().unwrap();
                send_beep(dev);
            }
            
            ui.separator();

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
