#![no_std]
#![no_main]
#![allow(missing_docs)]

use core::panic::PanicInfo;
use embedded_hal::digital::v2::OutputPin;
use rp_pico::entry;
use rp_pico::hal;
use rp_pico::hal::pac;
use rp_pico::hal::prelude::*;
// use rtt_target::rprint;
use usb_device::class_prelude::*;
use usb_device::descriptor;
use usb_device::device::{UsbDevice, UsbDeviceBuilder, UsbVidPid};
use usb_device::Result;
use usbd_serial;

use heapless::spsc::Queue;
use heapless::Vec;

use microbit as _;
use rtt_target::{rprintln, rtt_init_print};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("Panic: {:?}", info);
    loop {}
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    let sio = hal::Sio::new(pac.SIO);
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    let mut led_pin = pins.led.into_push_pull_output();

    // Set up the USB driver
    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    let mut test = TestClass::new(&usb_bus);
    let mut usb_dev = test.make_device(&usb_bus);
    let mut led_state: bool = false;
    let mut data: [u8; 64] = [0; 64];

    let mut bulk_data_queue: Queue<Vec<u8, 64>, 8> = Queue::new();
    let (mut producer, mut consumer) = bulk_data_queue.split();

    delay.delay_ms(1);

    loop {
        if usb_dev.poll(&mut [&mut test]) {
            test.poll(&mut producer);

            if consumer.peek() != None {
                let d = consumer.dequeue().unwrap();
                let mut response: Vec<u8, 64> = Vec::new();
                for x in d {
                    if x <= 0xff - 10 {
                        response.push(x + 10).unwrap();
                    } else {
                        response.push(x).unwrap();
                    }
                }
                test.write_bulk_in(&response, response.len());
            }

            match test.cdc_acm_class.read_packet(&mut data) {
                //test.cdc_acm_class.write_packet();
                Err(_e) => {
                    // Do nothing
                }
                Ok(0) => {
                    // Do nothing
                }
                Ok(_count) => {
                    if led_state {
                        led_pin.set_low().unwrap();
                        led_state = !led_state;
                    } else {
                        led_pin.set_high().unwrap();
                        led_state = !led_state;
                    }
                }
            }
        }
    }
}

#[cfg(not(feature = "test-class-high-speed"))]
mod sizes {
    pub const BUFFER: usize = 256;
    pub const CONTROL_ENDPOINT: u8 = 8;
    pub const BULK_ENDPOINT: u16 = 64;
    pub const INTERRUPT_ENDPOINT: u16 = 31;
}

/// Test USB class for testing USB driver implementations. Supports various endpoint types and
/// requests for testing USB peripheral drivers on actual hardware.
// #[derive(Debug)]
pub struct TestClass<'a, B: UsbBus> {
    custom_string: StringIndex,
    interface_string: StringIndex,
    cdc_acm_class: usbd_serial::CdcAcmClass<'a, B>,
    my_if: InterfaceNumber,
    ep_bulk_in: EndpointIn<'a, B>,
    ep_bulk_out: EndpointOut<'a, B>,
    ep_interrupt_in: EndpointIn<'a, B>,
    ep_interrupt_out: EndpointOut<'a, B>,
    control_buf: [u8; sizes::BUFFER],
    bulk_buf: [u8; sizes::BUFFER],
    interrupt_buf: [u8; sizes::BUFFER],
}

pub const VID: u16 = 0x2e8a;
pub const PID: u16 = 0xbeef;
pub const MANUFACTURER: &str = "TestClass Manufacturer";
pub const PRODUCT: &str = "virkkunen.net usb-device TestClass";
pub const SERIAL_NUMBER: &str = "TestClass Serial";
pub const CUSTOM_STRING: &str = "TestClass Custom String";
pub const INTERFACE_STRING: &str = "TestClass Interface";

pub const REQ_STORE_REQUEST: u8 = 1;
pub const REQ_READ_BUFFER: u8 = 2;
pub const REQ_WRITE_BUFFER: u8 = 3;
pub const REQ_READ_LONG_DATA: u8 = 5;
pub const REQ_LED: u8 = 6;
pub const REQ_UNKNOWN: u8 = 42;

pub const LONG_DATA: &[u8] = &[0x17; 257];

impl<B: UsbBus> TestClass<'_, B> {
    /// Creates a new TestClass.
    pub fn new(alloc: &UsbBusAllocator<B>) -> TestClass<'_, B> {
        TestClass {
            custom_string: alloc.string(),
            interface_string: alloc.string(),
            cdc_acm_class: usbd_serial::CdcAcmClass::new(alloc, 64u16),
            ep_bulk_in: alloc.bulk(sizes::BULK_ENDPOINT),
            ep_bulk_out: alloc.bulk(sizes::BULK_ENDPOINT),
            my_if: alloc.interface(),
            ep_interrupt_in: alloc.interrupt(sizes::INTERRUPT_ENDPOINT, 1),
            ep_interrupt_out: alloc.interrupt(sizes::INTERRUPT_ENDPOINT, 1),
            control_buf: [0; sizes::BUFFER],
            bulk_buf: [0; sizes::BUFFER],
            interrupt_buf: [0; sizes::BUFFER],
        }
    }

    /// Convenience method to create a UsbDevice that is configured correctly for TestClass.
    pub fn make_device<'a, 'b>(&'a self, usb_bus: &'b UsbBusAllocator<B>) -> UsbDevice<'b, B> {
        self.make_device_builder(usb_bus).build()
    }

    /// Convenience method to create a UsbDeviceBuilder that is configured correctly for TestClass.
    ///
    /// The methods sets
    ///
    /// - manufacturer
    /// - product
    /// - serial number
    /// - max_packet_size_0
    ///
    /// on the returned builder. If you change the manufacturer, product, or serial number fields,
    /// the test host may misbehave.
    pub fn make_device_builder<'a, 'b>(
        &'a self,
        usb_bus: &'b UsbBusAllocator<B>,
    ) -> UsbDeviceBuilder<'b, B> {
        UsbDeviceBuilder::new(usb_bus, UsbVidPid(VID, PID))
            .manufacturer(MANUFACTURER)
            .product(PRODUCT)
            .serial_number(SERIAL_NUMBER)
            .max_packet_size_0(sizes::CONTROL_ENDPOINT)
    }

    /// Must be called after polling the UsbDevice.
    pub fn poll(&mut self, producer: &mut heapless::spsc::Producer<Vec<u8, 64>, 8>) {
        match self.ep_bulk_out.read(&mut self.bulk_buf[..]) {
            Ok(count) => {
                if count < self.ep_bulk_out.max_packet_size() as usize {
                    let mut vec: Vec<u8, 64> = Vec::new();
                    vec.extend_from_slice(&self.bulk_buf[..count]).unwrap();
                    producer.enqueue(vec).unwrap();
                }
            }
            Err(UsbError::WouldBlock) => {}
            Err(err) => panic!("bulk read {:?}", err),
        };

        match self.ep_interrupt_out.read(&mut self.interrupt_buf) {
            Ok(count) => {
                self.ep_interrupt_in
                    .write(&self.interrupt_buf[0..count])
                    .expect("interrupt write");
            }
            Err(UsbError::WouldBlock) => {}
            Err(err) => panic!("interrupt read {:?}", err),
        };
    }

    fn write_bulk_in(&mut self, data: &[u8], len: usize) {
        match self.ep_bulk_in.write(data) {
            Ok(count) => {
                assert_eq!(count, len);
            }
            Err(UsbError::WouldBlock) => {}
            Err(err) => {
                panic!("bulk write {:?}", err);
            }
        };
    }
}

impl<B: UsbBus> UsbClass<B> for TestClass<'_, B> {
    fn reset(&mut self) {}

    fn get_configuration_descriptors(&self, writer: &mut DescriptorWriter) -> Result<()> {
        self.cdc_acm_class.get_configuration_descriptors(writer)?;
        /////////////////////////////////////////////////////////////////
        writer.interface(self.my_if, 0xff, 0x00, 0x00)?;
        writer.endpoint(&self.ep_bulk_in)?;
        writer.endpoint(&self.ep_bulk_out)?;
        writer.endpoint(&self.ep_interrupt_in)?;
        writer.endpoint(&self.ep_interrupt_out)?;
        writer.interface_alt(self.my_if, 1, 0xff, 0x01, 0x00, Some(self.interface_string))?;

        Ok(())
    }

    fn get_string(&self, index: StringIndex, lang_id: u16) -> Option<&str> {
        if lang_id == descriptor::lang_id::ENGLISH_US {
            if index == self.custom_string {
                return Some(CUSTOM_STRING);
            } else if index == self.interface_string {
                return Some(INTERFACE_STRING);
            }
        }

        None
    }

    fn control_in(&mut self, xfer: ControlIn<B>) {
        let req = *xfer.request();

        if !(req.request_type == control::RequestType::Vendor
            && req.recipient == control::Recipient::Device)
        {
            return;
        }

        match req.request {
            REQ_READ_BUFFER if req.length as usize <= self.control_buf.len() => xfer
                .accept_with(&self.control_buf[0..req.length as usize])
                .expect("control_in REQ_READ_BUFFER failed"),
            REQ_READ_LONG_DATA => xfer
                .accept_with_static(LONG_DATA)
                .expect("control_in REQ_READ_LONG_DATA failed"),
            _ => xfer.reject().expect("control_in reject failed"),
        }
    }

    fn control_out(&mut self, xfer: ControlOut<B>) {
        let req = *xfer.request();

        if !(req.request_type == control::RequestType::Vendor
            && req.recipient == control::Recipient::Device)
        {
            return;
        }

        match req.request {
            REQ_STORE_REQUEST => {
                self.control_buf[0] =
                    (req.direction as u8) | (req.request_type as u8) << 5 | (req.recipient as u8);
                self.control_buf[1] = req.request;
                self.control_buf[2..4].copy_from_slice(&req.value.to_le_bytes());
                self.control_buf[4..6].copy_from_slice(&req.index.to_le_bytes());
                self.control_buf[6..8].copy_from_slice(&req.length.to_le_bytes());

                xfer.accept().expect("control_out REQ_STORE_REQUEST failed");
            }
            REQ_WRITE_BUFFER if xfer.data().len() as usize <= self.control_buf.len() => {
                assert_eq!(
                    xfer.data().len(),
                    req.length as usize,
                    "xfer data len == req.length"
                );

                self.control_buf[0..xfer.data().len()].copy_from_slice(xfer.data());

                xfer.accept().expect("control_out REQ_WRITE_BUFFER failed");
            }
            REQ_LED => {}
            _ => xfer.reject().expect("control_out reject failed"),
        }
    }
}
