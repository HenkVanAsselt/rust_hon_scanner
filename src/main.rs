use hidapi::{DeviceInfo, HidApi, HidDevice};
use std::io;
use std::thread::sleep;
use std::time;

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
                // Add the ID to the list of id's found till now.
                unique_ids.push(combined_ids);
                // Push the device on the list of availabe/valid USB devices
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

fn select_usb_device(devices: &Vec<DeviceInfo>) -> usize {
    show_available_devices(devices);

    // Prompt the user for an index
    println!();
    println!("Please enter the index of the USB device to use: ");

    // Read the user input
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line with the index of the USB device to use.");

    // Parse the input to an integer
    let index: usize = input.trim().parse().unwrap();
    let max_index = devices.len();
    // let i = if index < max_index {
    //     index
    // } else {
    //     println!("Invalid index. Please try again.");
    //     select_usb_device(devices)
    // };
    // i
    if index < max_index {
        index
    } else {
        println!("Invalid index. Please try again.");
        select_usb_device(devices)
    }
}

fn show_available_devices(devices: &[DeviceInfo]) {
    println!();
    println!("Connected USB devices:");
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
    println!();
}

fn find_mask_in_available_devices(devices: Vec<DeviceInfo>, mask: String) -> Option<DeviceInfo> {
    // First see if the mask is found in the product strings (case sensitive !)
    for device in devices.iter() {
        if device.product_string().unwrap().contains(&mask) {
            return Some(device.clone());
        }
    }
    // If there was no match, see if the mask is found in the manufacturer strings (case sensitive !)
    for device in devices.iter() {
        if device.manufacturer_string().unwrap().contains(&mask) {
            return Some(device.clone());
        }
    }
    None
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
        Ok(_) => () ,
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
    let command = [0xFD, 0x0F, 0x16, 0x4D, 0x0D, 0x52, 0x45, 0x56, 0x49, 0x4e, 0x46, 0x2e];
    let result = device.write(&command);
    match result {
        Ok(_) => println!("REVINF. Command sent successfully!"),
        Err(e) => eprintln!("Failed to send command: {}", e),
    }
}

fn send_command(device: &HidDevice, commandstr: String) {
    
    println!("Sending command: {}", commandstr);
    
    let mut command = vec![0xFD, 0x0F, 0x16, 0x4D, 0x0D];

    let ascii_values: Vec<u8> = commandstr.chars()
        .map(|c| c as u8)
        .collect();

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

fn read_data(device: &HidDevice) {

    // println!("Reading data...");

    let mut full_response: Vec<String> = Vec::new();

    loop {

        // Read data from the device
        sleep(time::Duration::from_millis(50));
        let mut buf = [0u8; 64]; // Buffer to hold the read data
        let bytes_read = device.read_timeout(&mut buf[..], 300).unwrap();
        // println!("Raw data: ({}) {:?}", bytes_read, buf);
        if bytes_read == 0 {
            // println!("Done reading data.");
            // println!("Full response as vector: {:?}", full_response);
            let resp = full_response.join("");
            println!("{}", resp);
            return;
        }

        // Extract the AIM identifier
        let _aim_identifier: String = buf[2..=4]
            .iter()
            .map(|&x| x as char) // Convert each integer to a char
            .collect(); // Collect the characters into a String
        // println!("AIM identifier: {}", aim_identifier);

        // Extract the barcode data
        let data_len: usize = buf[1] as usize;
        // println!("data_len: {}", data_len);
        // Convert the vector to a string. This skips the header AND the AIM identifier.
        let data_string: String = buf[5..=data_len + 5]
            .iter()
            .map(|&x| x as char) // Convert each integer to a char
            .collect(); // Collect the characters into a String
        // println!("AIM: {}datastring: '{}'", aim_identifier,data_string);
        full_response.push(data_string);

    }
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

    // // This is for the Honeywell 1602g
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

    // Initialize the HID API
    let api = HidApi::new().unwrap();

    // Open the device
    let device = api
        .open(vendor_id, product_id)
        .expect("Failed to open device");
    println!(
        "Opened device: {:#?}",
        device.get_product_string().unwrap().unwrap()
    );
    println!();
    
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
