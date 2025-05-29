use std::io::{self, Write};
use std::thread::sleep;
use std::time;
use hidapi::{DeviceInfo, HidApi, HidDevice};

pub fn enumerate_usb_devices() -> Vec<DeviceInfo> {
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

pub fn select_usb_device(devices: &Vec<DeviceInfo>) -> usize {
    show_available_devices();

    // Prompt the user for an index
    println!();
    print!("Please enter the index of the USB device to use: ");
    io::stdout().flush().unwrap(); // Flush the output to ensure it appears immediately

    // Read the user input
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line with the index of the USB device to use.");

    // Parse the input to an integer
    let index: usize = input.trim().parse().unwrap();
    let max_index = devices.len();
    if index < max_index {
        index
    } else {
        println!("Invalid index. Please try again.");
        select_usb_device(devices)
    }
}

pub fn show_available_devices() {
    let available_usb_devices: Vec<DeviceInfo> = enumerate_usb_devices();
    println!();
    println!("Connected USB devices:");
    for (index, device) in available_usb_devices.iter().enumerate() {
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

pub fn find_mask_in_available_devices(devices: Vec<DeviceInfo>, mask: String) -> Option<DeviceInfo> {
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

pub fn read_data(device: &HidDevice) {
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

pub fn open_device(vendor_id: u16, product_id: u16) -> HidDevice {
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
    device
}

//
// Honeywell specific scanner functions
//

pub fn send_trigger_on(device: &HidDevice) {
    // Example: Write data to the device
    let command = [0xFD, 0x03, 0x16, 0x54, 0x0d]; // Scanner Trigger on
    let result = device.write(&command);
    match result {
        // Ok(_) => println!("TRIGGER ON Command sent successfully!"),
        Ok(_) => (),
        Err(e) => eprintln!("Failed to send command: {}", e),
    }
}

// pub fn send_trigger_off(device: &HidDevice) {
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

pub fn send_beep(device: &HidDevice) {
    let command = [0xFD, 0x03, 0x16, 0x07, 0x0d]; // Beep
    let result = device.write(&command);
    match result {
        // Ok(_) => println!("BEEP Command sent successfully!"),
        Ok(_) => (),
        Err(e) => eprintln!("Failed to send command: {}", e),
    }
}

pub fn send_revinfo(device: &HidDevice) {
    let command = [
        0xFD, 0x0F, 0x16, 0x4D, 0x0D, 0x52, 0x45, 0x56, 0x49, 0x4e, 0x46, 0x2e,
    ];
    let result = device.write(&command);
    match result {
        Ok(_) => println!("REVINF. Command sent successfully!"),
        Err(e) => eprintln!("Failed to send command: {}", e),
    }
}

pub fn send_command(device: &HidDevice, commandstr: String) {
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


pub fn scan_a_barcode(device: &HidDevice) {
    send_trigger_on(device);
    // sleep(time::Duration::from_secs(1));
    read_data(device);
    // send_trigger_off(&device);
}


