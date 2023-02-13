use libusb::{Context, DeviceHandle, Result};

fn main() -> Result<()> {
    let context = Context::new()?;
    let mut devices = context.devices()?;

    for device in devices.iter() {
        let device_desc = device.device_descriptor()?;

        // Check if this is the device we want to communicate with
        if device_desc.vendor_id() == 0x1234 && device_desc.product_id() == 0x5678 {
            let mut handle = device.open()?;

            // Send a request to the device
            let request_type = libusb::request_type(
                libusb::Direction::Out,
                libusb::RequestType::Vendor,
                libusb::Recipient::Device,
            );
            let request = 0x01;
            let value = 0x02;
            let index = 0x03;
            let data = [0x04, 0x05, 0x06, 0x07];
            handle.control_transfer(request_type, request, value, index, &data, 1000)?;

            // Read a response from the device
            let mut data = [0u8; 8];
            let request_type = libusb::request_type(
                libusb::Direction::In,
                libusb::RequestType::Vendor,
                libusb::Recipient::Device,
            );
            handle.control_transfer(request_type, request, value, index, &mut data, 1000)?;

            println!("Received response: {:?}", data);
        }
    }

    Ok(())
}
