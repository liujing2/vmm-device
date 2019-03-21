// Copyright 2019 Intel Corporation. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::sync::{Arc, Mutex};
use byteorder::{ByteOrder, LittleEndian};
use super::dev::*;
use super::device_manager::*;
use super::pci_device::*;

#[derive(Clone)]
pub struct PciBus {
    pub devices: Vec<Arc<Mutex<PciDevice>>>,
    pub config_address_reg: u32,
}

impl PciBus {
    pub fn new() -> Self {
        PciBus {
            devices: Vec::new(),
            config_address_reg: 0,
        }
    }

    pub fn insert(&mut self, dev: Arc<Mutex<PciDevice>>) {
        self.devices.push(dev);
    }

    fn parse_config_address(&self, config_address: u32) -> (usize, usize, usize, usize) {
        const BUS_NUMBER_OFFSET: usize = 16;
        const BUS_NUMBER_MASK: u32 = 0x00ff;
        const DEVICE_NUMBER_OFFSET: usize = 11;
        const DEVICE_NUMBER_MASK: u32 = 0x1f;
        const FUNCTION_NUMBER_OFFSET: usize = 8;
        const FUNCTION_NUMBER_MASK: u32 = 0x07;
        const REGISTER_NUMBER_OFFSET: usize = 2;
        const REGISTER_NUMBER_MASK: u32 = 0x3f;

        let bus_number = ((config_address >> BUS_NUMBER_OFFSET) & BUS_NUMBER_MASK) as usize;
        let device_number = ((config_address >> DEVICE_NUMBER_OFFSET) & DEVICE_NUMBER_MASK) as usize;
        let function_number =
            ((config_address >> FUNCTION_NUMBER_OFFSET) & FUNCTION_NUMBER_MASK) as usize;
        let register_number =
            ((config_address >> REGISTER_NUMBER_OFFSET) & REGISTER_NUMBER_MASK) as usize;

        (bus_number, device_number, function_number, register_number)
    }


    fn set_config_address(&mut self, offset: u64, data: &[u8]) {
        if offset as usize + data.len() > 4 {
            return;
        }
        let (mask, value): (u32, u32) = match data.len() {
            1 => (
                0x0000_00ff << (offset * 8),
                (data[0] as u32) << (offset * 8),
            ),
            2 => (
                0x0000_ffff << (offset * 16),
                ((data[1] as u32) << 8 | data[0] as u32) << (offset * 16),
            ),
            4 => (0xffff_ffff, LittleEndian::read_u32(data)),
            _ => return,
        };
        self.config_address_reg = (self.config_address_reg & !mask) | value;
    }

    pub fn config_address_read(&self, addr: u64, data: &mut [u8]) {
        let value: u32 = match addr {
            0xcf8...0xcfb => self.config_address_reg,
            0xcfc...0xcff => {
                let (_bus, device, _function, register) =
                    self.parse_config_address(self.config_address_reg & !0x8000_0000);

                self.devices
                    .get(device - 1)
                    .map_or(0xffff_ffff, |d| d.lock()
                    .expect("failed to acquire lock")
                    .config_register_read(register))
            },
            _ => 0xffff_ffff,
        };
        // Only allow reads to the register boundary.
        let start = (addr - 0xcf8) as usize % 4;
        let end = start + data.len();
        if end <= 4 {
            for i in start..end {
                data[i - start] = (value >> (i * 8)) as u8;
            }
        } else {
            for d in data {
                *d = 0xff;
            }
        }
    }


    pub fn config_address_write(&mut self, addr: u64, data: &mut [u8]) {
        match addr {
            0xcf8...0xcfb => { self.set_config_address(addr - 0xcf8, data); }
            0xcfc...0xcff => {
                let enabled = (self.config_address_reg & 0x8000_0000) != 0;
                if !enabled {
                    return;
                }
                let (_bus, device, _function, register) =
                    self.parse_config_address(self.config_address_reg & !0x8000_0000);
                if let Some(d) = self.devices.get(device - 1) {
                    d.lock().expect("failed to acquire lock")
                            .config_register_write(register, addr - 0xcfc, data);
                }
            }
            _ => return
        }
    }

}


impl IoOps for PciBus {
    fn read(&self, addr: u64, data: &mut [u8]) {
        let value: u32 = match addr {
            0xcf8...0xcfb => self.config_address_reg,
            0xcfc...0xcff => {
                let (_bus, device, _function, register) =
                    self.parse_config_address(self.config_address_reg & !0x8000_0000);

                self.devices
                    .get(device - 1)
                    .map_or(0xffff_ffff, |d| d.lock()
                    .expect("failed to acquire lock")
                    .config_register_read(register))
            },
            _ => 0xffff_ffff,
        };
        // Only allow reads to the register boundary.
        let start = (addr - 0xcf8) as usize % 4;
        let end = start + data.len();
        if end <= 4 {
            for i in start..end {
                data[i - start] = (value >> (i * 8)) as u8;
            }
        } else {
            for d in data {
                *d = 0xff;
            }
        }
    }
 
    fn write(&mut self, addr: u64, data: &[u8]) {
        match addr {
            0xcf8...0xcfb => { self.set_config_address(addr - 0xcf8, data); }
            0xcfc...0xcff => {
                let enabled = (self.config_address_reg & 0x8000_0000) != 0;
                if !enabled {
                    return;
                }
                let (_bus, device, _function, register) =
                    self.parse_config_address(self.config_address_reg & !0x8000_0000);
                if let Some(d) = self.devices.get(device - 1) {
                    d.lock().expect("failed to acquire lock")
                            .config_register_write(register, addr - 0xcfc, data);
                }
            }
            _ => return
        }
    }
 

}

impl Device for PciBus {
    fn get_name(&self) -> String {
        String::from("")
    }
}

pub fn pci_bus_init(sys_bus: &mut SysBus, mgr: &mut DeviceManager) {
    let pci_bus = Arc::new(Mutex::new(PciBus::new()));

    assert!(mgr.register_pio(0xcf8, 8, pci_bus.clone()).is_ok());
    sys_bus.insert(pci_bus.clone());
}

