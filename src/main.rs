use hidapi::{DeviceInfo, HidApi};
use std::io;

use clap::Parser;
use clap_num::maybe_hex;

#[derive(Parser, Debug)]

#[command(name = "rust_hon_scanner", version = "0.1", about = "Control a Honeywell HON scanner")]
struct Cli {
    /// The name of the device to look for
    #[arg(short, long)]
    mask: Option<String>,
    /// Optional: USB Vendor identifier. This takes precedence over the option --mask
    #[arg(short, long, value_parser=maybe_hex::<u16>)]
    vid: Option<u16>,
    /// Optional: USB Product identifier. This takes precedence over the option --mask
    #[arg(short, long, value_parser=maybe_hex::<u16>)]
    pid: Option<u16>,

}

fn enumerate_usb_devices() -> Vec<DeviceInfo> {
    let mut unique_ids: Vec<String> = Vec::new();
    let mut available_usb_devices: Vec<DeviceInfo> = Vec::new();

    match HidApi::new() {
        Ok(api) => {
            for device in api.device_list() {
                let manufacturer_string = device.manufacturer_string().unwrap();
                let product_string = device.product_string().unwrap();
                let vendor_id = device.vendor_id();
                let product_id = device.product_id();

                // We cannot use the device, if it is missing basic information
                if manufacturer_string.is_empty() || product_string.is_empty() {
                    continue;
                }

                // Create an unique id, and test if it is already detected.
                // If yes, then go to the start of the loop.
                // If not, add the unique id to the vector and continue processing
                let combined_ids = format!("{}:{}", vendor_id, product_id);
                if unique_ids.contains(&combined_ids) {
                    continue;
                }
                unique_ids.push(combined_ids);

                // Show the current information
                // println!(
                //     "{:20} {:40} ({:04x}:{:04x})",
                //     product_string, manufacturer_string, vendor_id, product_id
                // );

                //Push the device on the list of availabe/valid USB devices
                available_usb_devices.push(device.clone());
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    // Return the list/vector of unique USB devices
    // println!("Available devices: {:?}", available_usb_devices);
    available_usb_devices
}

fn select_usb_device(devices: Vec<DeviceInfo>) -> usize {
    show_available_devices(devices.clone());

    // Prompt the user for an index
    println!("Please enter the index of the item you want to use:");

    // Read the user input
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    // Parse the input to an integer
    let index: usize = input.trim().parse().unwrap();
    let max_index = devices.len();
    let i = if index < max_index {
        index
    } else {
        println!("Invalid index. Please try again.");
        select_usb_device(devices)
    };
    i
}

fn show_available_devices(devices: Vec<DeviceInfo>) {
    for (index, device) in devices.iter().enumerate() {
        let manufacturer_string = device.manufacturer_string().unwrap();
        let product_string = device.product_string().unwrap();
        let vendor_id = device.vendor_id();
        let product_id = device.product_id();
        println!(
            "{}: {:20} {:40} ({:04x}:{:04x})",
            index, product_string, manufacturer_string, vendor_id, product_id
        );
    }
}

fn is_pid_and_vid_given(pid: Option<u16>, vid: Option<u16>) -> bool {
    if pid.is_none() || vid.is_none() {
        return false;
    }
    return true;
}

fn is_mask_given(mask: Option<String>) -> bool {
    if mask.is_none() {
        return false;
    }
    return true;
}

fn main() {
    
    // Handle the commandline
    let args = Cli::parse();
    println!("args: {:?}", args);

    // // This is for the Honeywell 1602g
    // let vendor_id = 0x0c2e; // Example vendor ID
    // let product_id = 0x0db3; // Example product ID
    
    let mut vendor_id: u16 = 0;
    let mut product_id: u16 = 0;
    
    if is_pid_and_vid_given(args.pid, args.vid) {
        // A pid and vid where given on the commandline. Use these.
        vendor_id = args.vid.unwrap();
        product_id = args.pid.unwrap();
    }
    else if is_mask_given(args.mask.clone()) {
        println!("Mask given: {}", args.mask.unwrap());
        // Search here for this maksk in the list of connected USB devices:
    }
    else {
        // Select the USB device from the list of connected USB devices
        println!("All available hid devices:");
        let available_usb_devices: Vec<DeviceInfo> = enumerate_usb_devices();
        let index = select_usb_device(available_usb_devices.clone());
        println!("Selected device: {:?}", available_usb_devices[index].product_string().unwrap());
        let device = &available_usb_devices[index];
        vendor_id = device.vendor_id();
        product_id = device.product_id();
    }

    // // Initialize the HID API
    let api = HidApi::new().unwrap();

    // Open the device
    let device = api
        .open(vendor_id, product_id)
        .expect("Failed to open device");
    println!("Opened device: {:#?}", device.get_product_string().unwrap().unwrap());
    println!();

    // Example: Write data to the device
    let command = [0xFD, 0x03, 0x16, 0x54, 0x0d]; // Scanner Trigger on
    let result = device.write(&command);
    match result {
        Ok(_) => println!("TRIGGER ON Command sent successfully!"),
        Err(e) => eprintln!("Failed to send command: {}", e),
    }

    // Read data from the device
    let mut buf = [0u8; 64]; // Buffer to hold the read data
    let _bytes_read = device.read(&mut buf[..]).unwrap();
    println!("Raw data: {:?}", buf);
    let aim_identifier: String = buf[2..=4].iter()
        .map(|&x| x as char) // Convert each integer to a char
        .collect(); // Collect the characters into a String
    println!("AIM identifier: {}", aim_identifier);

    let data_len: usize = buf[1] as usize;
    // println!("data_len: {}", data_len);
    // Convert the vector to a string. This skips the header AND the AIM identifier.
    let data_string: String = buf[5..=data_len+5].iter()
        .map(|&x| x as char) // Convert each integer to a char
        .collect(); // Collect the characters into a String
    println!("datastring: '{}'", data_string);

    let command = [0xFD, 0x03, 0x16, 0x55, 0x0d]; // Trigger off
    let result = device.write(&command);
    match result {
        Ok(_) => println!("TRIGGER OFF Command sent successfully!"),
        Err(e) => eprintln!("Failed to send command: {}", e),
    }

    let command = [0xFD, 0x03, 0x16, 0x07, 0x0d]; // Beep
    let result = device.write(&command);
    match result {
        Ok(_) => println!("BEEP Command sent successfully!"),
        Err(e) => eprintln!("Failed to send command: {}", e),
    }
    
    
    
 


}
