// Copyright 2019 Intel Corporation. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use super::dev::*;
use std::result;
use std::sync::{Arc, Mutex};
use std::collections::btree_map::BTreeMap;
use super::system_allocate::*;


#[derive(Debug)]
pub enum Error {
    /// The insertion failed because the new device overlapped with an old device.
    Overlap,
    /// The insertion failed because the resource is not enough.
    Oversize,
}

pub type Result<T> = result::Result<T, Error>;

pub trait IoOps {
    fn read(&self, addr: u64, data: &mut [u8]);
    fn write(&mut self, addr: u64, data: &[u8]);
}

pub struct SysBus {
    /// All the buses instance
    pub buses: Vec<Arc<Mutex<Device>>>,
}

impl SysBus {
    pub fn new() -> SysBus {
        SysBus {buses: Vec::new()}
    }

    pub fn insert(&mut self, bus: Arc<Mutex<Device>>) {
        self.buses.push(bus);
    }

}

pub struct DeviceManager {
    /// Range mapping for kvm exit mmio operations
    pub mmio_ops: BTreeMap<Range, Arc<Mutex<IoOps>>>,
    /// Range mapping for kvm exit pio operations
    pub pio_ops: BTreeMap<Range, Arc<Mutex<IoOps>>>,
}

impl DeviceManager {
    pub fn new() -> Self {
        DeviceManager {
            mmio_ops: BTreeMap::new(),
            pio_ops: BTreeMap::new(),
        }
    }

    /// Register the mmio space and operation for kvm exit.
    pub fn register_mmio(&mut self, base: u64, size: u64, io_ops: Arc<Mutex<IoOps>>) -> Result<()> {
        if self.mmio_ops
               .insert(Range(base, size), io_ops)
               .is_some() {
            return Err(Error::Overlap);
        }
        Ok(())
    }

    /// Register the pio space and operation for kvm exit.
    pub fn register_pio(&mut self, base: u64, size: u64, io_ops: Arc<Mutex<IoOps>>) -> Result<()> {
        if self.pio_ops
               .insert(Range(base, size), io_ops)
               .is_some() {
            return Err(Error::Overlap);
        }
        Ok(())
   }

    /// Real devices call allocate_mmio and register_mmio if needed
    pub fn allocate_mmio(&self, mem_res: &mut SystemAllocator, size: u64) -> Result<u64> {
        if let Some(base) = mem_res.allocate_mmio_addresses(size) {
            return Ok(base);
        } else {
            return Err(Error::Overlap);
        }
    }

    pub fn allocate_pio(&self, pio_res: &mut SystemAllocator, size: u64) -> Result<u64> {
        if let Some(base) = pio_res.allocate_pio_addresses(size) {
            return Ok(base);
        } else {
            return Err(Error::Overlap);
        }
    }
}
