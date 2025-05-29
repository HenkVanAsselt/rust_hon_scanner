use hidapi::{DeviceInfo, HidDevice};

use usbhid::enumerate_usb_devices;
use usbhid::find_mask_in_available_devices;
use usbhid::select_usb_device;
use usbhid::show_available_devices;
use usbhid::read_data;
use usbhid::open_device;

use clap::Parser;
use clap_num::maybe_hex;

#[derive(Parser, Debug)]
#[command(
    name = "rust_hon_scanner",
    version = "0.1",
    about = "Control a Honeywell USBHID scanner"
)]
struct Cli {
    /// Optional: The name of the device to look for
    #[arg(short, long)]
    mask: Option<String>,
    /// Optional: USB Vendor identifier. This takes precedence over the option --mask
    #[arg(short, long, value_parser=maybe_hex::<u16>)]
    vid: Option<u16>,
    /// Optional: USB Product identifier. This takes precedence over the option --mask
    #[arg(short, long, value_parser=maybe_hex::<u16>)]
    pid: Option<u16>,
    /// Optional: Show a list of available devices and exit
    #[arg(short, long, default_value = "false")]
    list: bool,
    /// Optional: Scan a barcode
    #[arg(short, long, default_value = "false")]
    scan: bool,
    /// Optional: Send REVINFO.
    #[arg(short, long, default_value = "false")]
    info: bool,
    /// Optional: The command to send to the selected scanner
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

fn send_trigger_on(device: &HidDevice) {
    // Example: Write data to the device
    let command = [0xFD, 0x03, 0x16, 0x54, 0x0d]; // Scanner Trigger on
    let result = device.write(&command);
    match result {
        // Ok(_) => println!("TRIGGER ON Command sent successfully!"),
        Ok(_) => (),
        Err(e) => eprintln!("Failed to send command: {}", e),
    }
}

// fn send_trigger_off(device: &HidDevice) {
//     // Example: Write data to the device
//     let command = [0xFD, 0x03, 0x16, 0x55, 0x0d]; // Scanner Trigger off
//     let result = device.write(&command);
//     match result {
//         // Ok(_) => println!("TRIGGER OFF Command sent successfully!"),
//         Ok(_) => () ,
//         Err(e) => eprintln!("Failed to send command: {}", e),
//
//     }
// }

fn send_beep(device: &HidDevice) {
    let command = [0xFD, 0x03, 0x16, 0x07, 0x0d]; // Beep
    let result = device.write(&command);
    match result {
        // Ok(_) => println!("BEEP Command sent successfully!"),
        Ok(_) => (),
        Err(e) => eprintln!("Failed to send command: {}", e),
    }
}

fn send_revinfo(device: &HidDevice) {
    let command = [
        0xFD, 0x0F, 0x16, 0x4D, 0x0D, 0x52, 0x45, 0x56, 0x49, 0x4e, 0x46, 0x2e,
    ];
    let result = device.write(&command);
    match result {
        Ok(_) => println!("REVINF. Command sent successfully!"),
        Err(e) => eprintln!("Failed to send command: {}", e),
    }
}

fn send_command(device: &HidDevice, commandstr: String) {
    println!("Sending command: {}", commandstr);

    let mut command = vec![0xFD, 0x0F, 0x16, 0x4D, 0x0D];

    let ascii_values: Vec<u8> = commandstr.chars().map(|c| c as u8).collect();

    // println!("{:?}", ascii_values);
    command.extend(ascii_values);
    // println!("Extended command{:?}", command);

    let result = device.write(&command);
    match result {
        // Ok(_) => println!("BEEP Command sent successfully!"),
        Ok(_) => (),
        Err(e) => eprintln!("Failed to send command: {}", e),
    }

    read_data(device);
}


fn scan_a_barcode(device: &HidDevice) {
    send_trigger_on(device);
    // sleep(time::Duration::from_secs(1));
    read_data(device);
    // send_trigger_off(&device);
}

fn main() {
    // Handle the commandline
    let args = Cli::parse();
    // println!("args: {:?}", args);

    // For the Honeywell 1602g:
    // let vendor_id = 0x0c2e; // Example vendor ID
    // let product_id = 0x0db3; // Example product ID

    if args.list {
        let available_usb_devices: Vec<DeviceInfo> = enumerate_usb_devices();
        show_available_devices(&available_usb_devices);
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

    // // Initialize the HID API
    // let api = HidApi::new().unwrap();
    // 
    // // Open the device
    // let device = api
    //     .open(vendor_id, product_id)
    //     .expect("Failed to open device");
    // println!(
    //     "Opened device: {:#?}",
    //     device.get_product_string().unwrap().unwrap()
    // );
    // println!();
    
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
