#![allow(missing_docs, unused, reason = "Module might be in use later")]

use crate::{DeserFromBuf, ReceivedMessage};

pub type I2cBusList = RawDeviceInfoList<I2cSmbusInterface>;
pub type HidDeviceList = RawDeviceInfoList<HidDeviceData>;
pub type UsbDeviceList = RawDeviceInfoList<UsbDeviceData>;
pub type SerialPortList = RawDeviceInfoList<SerialPortData>;

/// Holds a list of "raw" I2C, HID or USB devices
#[derive(Debug)]
pub struct RawDeviceInfoList<T> {
    devices: Vec<T>,
}

impl<T: DeserFromBuf> DeserFromBuf for RawDeviceInfoList<T> {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> crate::OpenRgbResult<Self> {
        let _data_size = buf.read_value::<u32>()?;
        // for some reason length here is with a u32 instead of a u16
        let devices = buf.read_value::<Vec<T>>()?;
        Ok(Self { devices })
    }
}

#[derive(Debug)]
pub struct I2cSmbusInterface {
    device_name: String,
    port_id: i32,
    pci_device: i32,
    pci_vendor: i32,
    pci_subsystem_device: i32,
    pci_subsystem_vendor: i32,
    bus_id: i32,
}

impl DeserFromBuf for I2cSmbusInterface {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> crate::OpenRgbResult<Self> {
        let name_raw: [u8; 512] = buf.read_value()?;
        let device_name = String::from_utf8_lossy(&name_raw)
            .trim_end_matches('\0')
            .to_owned();
        let port_id = buf.read_value()?;
        let pci_device = buf.read_value()?;
        let pci_vendor = buf.read_value()?;
        let pci_subsystem_device = buf.read_value()?;
        let pci_subsystem_vendor = buf.read_value()?;
        let bus_id = buf.read_value()?;

        Ok(Self {
            device_name,
            port_id,
            pci_device,
            pci_vendor,
            pci_subsystem_device,
            pci_subsystem_vendor,
            bus_id,
        })
    }
}

#[derive(Debug)]
pub struct HidDeviceData {
    vendor_id: u16,
    product_id: u16,
    release_number: u16,
    usage_page: u16,
    usage: u16,
    interface_number: i32,
    serial_number: String,
    manufacturer: String,
    product: String,
    path: String,
}

impl DeserFromBuf for HidDeviceData {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> crate::OpenRgbResult<Self> {
        let vendor_id = buf.read_value()?;
        let product_id = buf.read_value()?;
        let release_number = buf.read_value()?;
        let usage_page = buf.read_value()?;
        let usage = buf.read_value()?;
        let interface_number = buf.read_value()?;
        let serial_number = buf.read_value()?;
        let manufacturer = buf.read_value()?;
        let product = buf.read_value()?;
        let path = buf.read_value()?;

        Ok(Self {
            vendor_id,
            product_id,
            release_number,
            usage_page,
            usage,
            interface_number,
            serial_number,
            manufacturer,
            product,
            path,
        })
    }
}

#[derive(Debug)]
pub struct UsbDeviceData {
    vendor_id: u16,
    product_id: u16,
    serial_number: String,
    manufacturer: String,
    product: String,
}

impl DeserFromBuf for UsbDeviceData {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> crate::OpenRgbResult<Self> {
        let vendor_id = buf.read_value()?;
        let product_id = buf.read_value()?;
        let serial_number = buf.read_value()?;
        let manufacturer = buf.read_value()?;
        let product = buf.read_value()?;

        Ok(Self {
            vendor_id,
            product_id,
            serial_number,
            manufacturer,
            product,
        })
    }
}

#[derive(Debug)]
pub struct SerialPortData {
    pub port_string: String,
}

impl DeserFromBuf for SerialPortData {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> crate::OpenRgbResult<Self> {
        Ok(Self {
            port_string: buf.read_value()?,
        })
    }
}
