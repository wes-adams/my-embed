// use core::convert::TryInto;
// use usb_device::Result;
// use usb_device::{class_prelude::*, prelude::*};

// /// This should be used as `device_class` when building the `UsbDevice`.
// pub const USB_CLASS_CDC: u8 = 0x02;

// const USB_CLASS_VENDOR: u8 = 0xff;
// const MY_SUBCLASS: u8 = 0x00;
// const VENDOR_PROTOCOL_NONE: u8 = 0x00;

// const CS_INTERFACE: u8 = 0x24;
// const CDC_TYPE_HEADER: u8 = 0x00;
// const CDC_TYPE_CALL_MANAGEMENT: u8 = 0x01;
// const CDC_TYPE_ACM: u8 = 0x02;
// const CDC_TYPE_UNION: u8 = 0x06;

// const REQ_SEND_ENCAPSULATED_COMMAND: u8 = 0x00;
// #[allow(unused)]
// const REQ_GET_ENCAPSULATED_COMMAND: u8 = 0x01;
// const REQ_SET_LINE_CODING: u8 = 0x20;
// const REQ_GET_LINE_CODING: u8 = 0x21;
// const REQ_SET_CONTROL_LINE_STATE: u8 = 0x22;

// // pub struct CdcAcmClass<'a, B: UsbBus> {
// pub struct TestClass<'a, B: UsbBus> {
//     my_if: InterfaceNumber,
//     in_ep: EndpointIn<'a, B>,
//     out_ep: EndpointOut<'a, B>,
//     data_if: InterfaceNumber,
//     read_ep: EndpointOut<'a, B>,
//     write_ep: EndpointIn<'a, B>,
// }

// impl<B: UsbBus> UsbClass<B> for TestClass<'_, B> {
//     fn get_configuration_descriptors(&self, writer: &mut DescriptorWriter) -> Result<()> {
//         writer.iad(
//             self.my_if,
//             2,
//             USB_CLASS_VENDOR,
//             MY_SUBCLASS,
//             VENDOR_PROTOCOL_NONE,
//         )?;

//         writer.interface(
//             self.my_if,
//             USB_CLASS_VENDOR,
//             MY_SUBCLASS,
//             VENDOR_PROTOCOL_NONE,
//         )?;

//         writer.write(
//             CS_INTERFACE,
//             &[
//                 CDC_TYPE_HEADER, // bDescriptorSubtype
//                 0x10,
//                 0x01, // bcdCDC (1.10)
//             ],
//         )?;

//         writer.write(
//             CS_INTERFACE,
//             &[
//                 CDC_TYPE_ACM, // bDescriptorSubtype
//                 0x00,         // bmCapabilities
//             ],
//         )?;

//         writer.write(
//             CS_INTERFACE,
//             &[
//                 CDC_TYPE_UNION, // bDescriptorSubtype
//                 // self.my_if.into(),   // bControlInterface
//                 // self.data_if.into(), // bSubordinateInterface
//                 0x00,
//                 0x00,
//             ],
//         )?;

//         writer.write(
//             CS_INTERFACE,
//             &[
//                 CDC_TYPE_CALL_MANAGEMENT, // bDescriptorSubtype
//                 0x00,                     // bmCapabilities
//                 // self.data_if.into(),      // bDataInterface
//                 0x00,
//             ],
//         )?;

//         writer.endpoint(&self.in_ep)?;

//         writer.interface(self.data_if, USB_CLASS_VENDOR, 0x00, 0x00)?;

//         writer.endpoint(&self.write_ep)?;
//         writer.endpoint(&self.read_ep)?;

//         Ok(())
//     }

//     fn reset(&mut self) {
//         // self.line_coding = LineCoding::default();
//         // self.dtr = false;
//         // self.rts = false;
//     }

//     fn control_in(&mut self, xfer: ControlIn<B>) {
//         let req = xfer.request();

//         if !(req.request_type == control::RequestType::Class
//             && req.recipient == control::Recipient::Interface)
//         // && req.index == u8::from(self.my_if) as u16)
//         {
//             return;
//         }

//         match req.request {
//             // REQ_GET_ENCAPSULATED_COMMAND is not really supported - it will be rejected below.
//             // REQ_GET_LINE_CODING if req.length == 7 => {
//             //     xfer.accept(|data| {
//             //         data[0..4].copy_from_slice(&self.line_coding.data_rate.to_le_bytes());
//             //         data[4] = self.line_coding.stop_bits as u8;
//             //         data[5] = self.line_coding.parity_type as u8;
//             //         data[6] = self.line_coding.data_bits;

//             //         Ok(7)
//             //     })
//             //     .ok();
//             // }
//             // _ => {
//             //     xfer.reject().ok();
//             // }
//         }
//     }

//     fn control_out(&mut self, xfer: ControlOut<B>) {
//         let req = xfer.request();

//         // if !(req.request_type == control::RequestType::Class
//         //     && req.recipient == control::Recipient::Interface
//         //     && req.index == u8::from(self.my_if) as u16)
//         // {
//         //     return;
//         // }

//         match req.request {
//             REQ_SEND_ENCAPSULATED_COMMAND => {
//                 // We don't actually support encapsulated commands but pretend we do for standards
//                 // compatibility.
//                 xfer.accept().ok();
//             }
//             REQ_SET_LINE_CODING if xfer.data().len() >= 7 => {
//                 // self.line_coding.data_rate =
//                 //     u32::from_le_bytes(xfer.data()[0..4].try_into().unwrap());
//                 // self.line_coding.stop_bits = xfer.data()[4].into();
//                 // self.line_coding.parity_type = xfer.data()[5].into();
//                 // self.line_coding.data_bits = xfer.data()[6];

//                 // xfer.accept().ok();
//             }
//             REQ_SET_CONTROL_LINE_STATE => {
//                 // self.dtr = (req.value & 0x0001) != 0;
//                 // self.rts = (req.value & 0x0002) != 0;

//                 // xfer.accept().ok();
//             }
//             _ => {
//                 xfer.reject().ok();
//             }
//         };
//     }
// }
