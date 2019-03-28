// Copyright 2019 Intel Corporation. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

#![allow(unused)]
use super::dev::*;
use super::device_manager::{SysBus, DeviceManager, IoOps};
use super::pci_device::*;
use std::sync::{Arc, Mutex};
use super::pci_bus::*;
use super::system_allocate::*;

pub struct DummyPciDevice {
    pub config_regs: [u32; 64],
}

impl DummyPciDevice {
    pub fn new() -> Self {
            DummyPciDevice {
                config_regs: [0; 64],
            }
        }
}

impl PciDevice for DummyPciDevice {
    fn config_registers(&self) -> &[u32] {
        &self.config_regs
    }
    /// Gets the configuration registers of the Pci Device for modification.
    fn config_registers_mut(&mut self) -> &mut [u32] {
        &mut self.config_regs
    }

    fn config_register_read(&self, reg_idx: usize) -> u32 {
        self.config_regs[reg_idx]
    }

    fn config_register_write(&mut self, reg_idx: usize, offset: u64, data: &[u8]) {
        // Some fake handling here.
        let regs = self.config_registers_mut();
        if let Some(r) = regs.get_mut(reg_idx) {
            *r = *r & (0xffu32 << offset) | data[0] as u32;
        } else {
            println!("bad PCI register write {}", reg_idx);
        }
    } 

}

impl Device for DummyPciDevice {
    fn get_name(&self) -> String {
        String::from("Dummy Pci")
    }
}

pub struct DummyPciBar0 {
    pub dev: Arc<Mutex<DummyPciDevice>>,
    pub size: u64,
    pub addr: u64,
    pub reg_idx: usize,
}

impl DummyPciBar0 {
    pub fn new(device: Arc<Mutex<DummyPciDevice>>) -> Self {
        DummyPciBar0 {
            dev: device,
            size: 0x1000,
            addr: 0,
            reg_idx: 0,
        }
    }
}
impl IoOps for DummyPciBar0 {
    fn read(&self, addr: u64, data: &mut [u8]) {
        }
    fn write(&mut self, addr: u64, data: &[u8]) {}
}

pub fn dummy_init(sys_bus: &mut SysBus, mgr: &mut DeviceManager, sys_res: &mut SystemAllocator) {
    let pci_dev = Arc::new(Mutex::new(DummyPciDevice::new()));
    let mut pci_dev_bar = DummyPciBar0::new(pci_dev.clone());

    sys_bus.insert(pci_dev.clone());

    if let Ok(addr) = mgr.allocate_mmio(sys_res, pci_dev_bar.size) {
        pci_dev_bar.addr = addr;
        mgr.register_mmio(pci_dev_bar.addr, pci_dev_bar.size, Arc::new(Mutex::new(pci_dev_bar)));
    } else {
        println!("No enough resource");
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_init() {
        let mut sys_bus = SysBus::new();
        let mut dev_mgr = DeviceManager::new();
        let mut sys_res = SystemAllocator::new();

        pci_bus_init(&mut sys_bus, &mut dev_mgr);
        dummy_init(&mut sys_bus, &mut dev_mgr, &mut sys_res);
    }

}

