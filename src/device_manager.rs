// Copyright 2019 Intel Corporation. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
#![allow(unused)]
use super::dev::*;
use std::result;
use std::sync::{Arc, Mutex};
use std::collections::btree_map::BTreeMap;
use super::system_allocate::*;

///
/// # Example - Use the `DeviceManager`.
///
/// ```
/// # use vmm-device::DeviceManager;
///   let buses = Bus::new();
///   let dev_mgr = DeviceManager::new();
///   let pci_bus = PciBus::new();
///
///   let pci_dev = DummyDevice::new();
///   pci_bus.insert(pci_dev);
///   ...
///   buses.insert(pci_bus);
///
///   buses.init(dev_mgr);
/// ```


#[derive(Debug)]
pub enum Error {
    /// The insertion failed because the new device overlapped with an old device.
    Overlap,
    /// The insertion failed because the resource is not enough.
    Oversize,
}

pub type Result<T> = result::Result<T, Error>;

pub struct IoOps {
    pub read_op: Box<Fn(u64, &mut [u8])>,
    pub write_op: Box<Fn(u64, &mut [u8])>,
}

pub struct Bus {
    /// All the buses instance
    pub buses: Vec<Arc<Mutex<Device>>>,
}

impl Bus {
    pub fn new() -> Bus {
        Bus {buses: Vec::new()}
    }

    pub fn insert(&mut self, bus: Arc<Mutex<Device>>) {
        self.buses.push(bus);
    }

    pub fn init(&self, dev_mgr: &mut DeviceManager) {
        for bus in self.buses.iter() {
            bus.lock().expect("failed to acquire lock").init(dev_mgr);
        }
    }
}

pub struct DeviceManager {
    /// Range mapping for kvm exit mmio operations
    pub mmio_ops: BTreeMap<Range, IoOps>,
    /// Range mapping for kvm exit pio operations
    pub pio_ops: BTreeMap<Range, IoOps>,
}

impl DeviceManager {
    pub fn new() -> Self {
        DeviceManager {
            mmio_ops: BTreeMap::new(),
            pio_ops: BTreeMap::new(),
        }
    }
}

/*
    /// Register the mmio space and operation for kvm exit.
    pub fn register_mmio(&self, base: u64, size: u64, read_op: Box<Fn(u64, &mut [u8])>, write_op: Box<Fn(u64, &mut [u8])>) -> Result<()> {
        if self.mmio_ops
               .insert(Range(base, size), (read_op, write_op))
               .is_some() {
            return Err(Error::Overlap);
        }
        Ok(())
    }

    /// Register the pio space and operation for kvm exit.
    pub fn register_pio(&self, base: u64, size: u64, read_op: Box<Fn(u64, &mut [u8])>, write_op: Box<Fn(u64, &mut [u8])>) -> Result<()> {
        if self.pio_ops
               .insert(Range(base, size), (read_op, write_op))
               .is_some() {
            return Err(Error::Overlap);
        }
        Ok(())
   }

    /// Real devices call allocate_mmio and register_mmio if needed
    pub fn allocate_mmio(&self, mem_res: SystemAllocator, size: u64) -> Result<u64> {
        if let Some(base) = mem_res.allocate_mmio_addresses(size) {
            return Ok(base);
        } else {
            return Err(Error::Overlap);
        }
    }

    pub fn allocate_pio(&self, pio_res: SystemAllocator, size: u64) -> Result<u64> {
        if let Some(base) = pio_res.allocate_pio_addresses(size) {
            return Ok(base);
        } else {
            return Err(Error::Overlap);
        }
    }
*/


