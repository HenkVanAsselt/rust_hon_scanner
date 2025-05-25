use hidapi::{DeviceInfo, HidApi};

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

fn show_available_devices(devices: Vec<DeviceInfo>) {
    for device in devices {
        let manufacturer_string = device.manufacturer_string().unwrap();
        let product_string = device.product_string().unwrap();
        let vendor_id = device.vendor_id();
        let product_id = device.product_id();
        println!(
            "{:20} {:40} ({:04x}:{:04x})",
            product_string, manufacturer_string, vendor_id, product_id
        );
    }
}

fn main() {
    println!("All available hid devices:");
    let available_usb_devices: Vec<DeviceInfo> = enumerate_usb_devices();
    show_available_devices(available_usb_devices);

    // Interaction:

    // // Initialize the HID API
    let api = HidApi::new().unwrap();

    // This is for the Honeywell 1602g
    let vendor_id = 0x0c2e; // Example vendor ID
    let product_id = 0x0db3; // Example product ID

    // Open the device
    let device = api
        .open(vendor_id, product_id)
        .expect("Failed to open device");
    println!("Opened device: {:#?}", device.get_product_string().unwrap().unwrap());
    println!();

    // let manuf_str = device.get_manufacturer_string();
    // println!("Manufacturer string: {:?}", manuf_str);
    // let prod_str = device.get_product_string();
    // println!("Product string: {:?}", prod_str);

    // Example: Write data to the device
    let command = [0xFD, 0x03, 0x16, 0x54, 0x0d]; // Replace with your data
    let result = device.write(&command);
    match result {
        Ok(_) => println!("Command sent successfully!"),
        Err(e) => eprintln!("Failed to send command: {}", e),
    }

    // // Example: Read data from the device
    let mut buf = [0u8; 256]; // Buffer to hold the read data
    let bytes_read = device.read(&mut buf[..]).unwrap();
    let mut data_string = String::new();
    for u in &buf[..bytes_read] {
        data_string.push_str(&(u.to_string() + "\t"));
    }

    println!("{}", data_string);

    let command = [0xFD, 0x03, 0x16, 0x55, 0x0d]; // Replace with your data
    let result = device.write(&command);
    match result {
        Ok(_) => println!("Command sent successfully!"),
        Err(e) => eprintln!("Failed to send command: {}", e),
    }
}
