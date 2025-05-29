use hidapi::{DeviceInfo};

use usbhid::enumerate_usb_devices;
use usbhid::find_mask_in_available_devices;
use usbhid::select_usb_device;
use usbhid::show_available_devices;
use usbhid::read_data;
use usbhid::open_device;
use usbhid::send_command;
use usbhid::send_revinfo;
use usbhid::scan_a_barcode;
use usbhid::send_beep;

use clap::Parser;
use clap_num::maybe_hex;

#[derive(Parser, Debug)]
#[command(
    name = "rust_hon_scanner",
    version = "0.1",
    about = "Control a Honeywell USBHID scanner"
)]
struct Cli {
    
    /// The name of the device to look for. (or use --vid and --pid)
    #[arg(short, long)]
    mask: Option<String>,
    
    /// USB vid (Vendor identifier). This takes precedence over the option --mask
    #[arg(short, long, value_parser=maybe_hex::<u16>)]
    vid: Option<u16>,
    
    /// USB pid (Product identifier). This takes precedence over the option --mask
    #[arg(short, long, value_parser=maybe_hex::<u16>)]
    pid: Option<u16>,
    
    /// Show a list of available devices and exit
    #[arg(short, long, default_value = "false")]
    list: bool,
    
    /// Scan a barcode
    #[arg(short, long, default_value = "false")]
    scan: bool,
    
    /// Send REVINFO.
    #[arg(short, long, default_value = "false")]
    info: bool,
    
    /// The command to send to the selected scanner
    #[arg(short, long)]
    command: Option<String>,
}



fn is_pid_and_vid_given(pid: Option<u16>, vid: Option<u16>) -> bool {
    if pid.is_none() || vid.is_none() {
        return false;
    }
    true
}

fn is_mask_given(mask: &Option<String>) -> bool {
    if mask.is_none() {
        return false;
    }
    true
}



fn main() {
    // Handle the commandline
    let args = Cli::parse();
    // println!("args: {:?}", args);

    // For the Honeywell 1602g:
    // let vendor_id = 0x0c2e; // Example vendor ID
    // let product_id = 0x0db3; // Example product ID

    if args.list {
        show_available_devices();
        return;
    }

    let vendor_id: u16;
    let product_id: u16;

    if is_pid_and_vid_given(args.pid, args.vid) {
        // A pid and vid where given on the commandline. Use these. 
        vendor_id = args.vid.unwrap();
        product_id = args.pid.unwrap();
    } else if is_mask_given(&args.mask) {
        let mask = args.mask.unwrap();
        // println!("Mask given: {}", mask);
        let available_usb_devices: Vec<DeviceInfo> = enumerate_usb_devices();
        let result = find_mask_in_available_devices(available_usb_devices, mask.clone());
        if result.is_none() {
            println!("No device found matching the given mask {}.", mask);
            return;
        }
        let device = result.unwrap();
        vendor_id = device.vendor_id();
        product_id = device.product_id();
    } else {
        // Select the USB device from the list of connected USB devices
        let available_usb_devices: Vec<DeviceInfo> = enumerate_usb_devices();
        let index = select_usb_device(&available_usb_devices);
        println!(
            "Selected device: {:?}",
            available_usb_devices[index].product_string().unwrap()
        );
        let device = &available_usb_devices[index];
        vendor_id = device.vendor_id();
        product_id = device.product_id();
    }
    
    let device = open_device(vendor_id, product_id);

    if args.command.is_some() {
        let commandstr = args.command.unwrap();
        send_command(&device, commandstr);
        return;
    }

    if args.scan {
        scan_a_barcode(&device);
        send_beep(&device);
    }

    if args.info {
        send_revinfo(&device);
        read_data(&device);
        send_beep(&device);
    }
}
