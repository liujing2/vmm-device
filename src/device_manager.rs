// Copyright © 2019 Intel Corporation. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 and BSD-3-Clause

//! System device management.
//!
//! [DeviceManager](struct.DeviceManager.html) responds to manage all devices
//! of virtual machine, store basic device information like name and
//! parent bus, register IO resource callback, unregister devices and help
//! VM IO exit handling.

extern crate vm_allocator;

use self::vm_allocator::SystemAllocator;
use crate::device::*;
use std::cmp::{Ord, Ordering, PartialEq, PartialOrd};
use std::collections::btree_map::BTreeMap;
use std::collections::HashMap;
use std::result;
use std::sync::{Arc, Mutex};
use vm_memory::{GuestAddress, GuestUsize};

/// Guest physical address and size pair to describe a range.
#[derive(Eq, Debug, Copy, Clone)]
pub struct Range(pub GuestAddress, pub GuestUsize);

impl PartialEq for Range {
    fn eq(&self, other: &Range) -> bool {
        self.0 == other.0
    }
}

impl Ord for Range {
    fn cmp(&self, other: &Range) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for Range {
    fn partial_cmp(&self, other: &Range) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

/// Error type for `DeviceManager` usage.
#[derive(Debug)]
pub enum Error {
    /// The insertion failed because the new device overlapped with an old device.
    Overlap,
    /// The insertion failed because the resource is not enough.
    Oversize,
    /// PIO address is none.
    NonePIOAddress,
    /// The insertion failed because device already exists.
    Exist,
    /// The removing fails because the device doesn't exist.
    NonExist,
}

/// Simplify the `Result` type.
pub type Result<T> = result::Result<T, Error>;

/// System device manager serving for all devices management and VM exit handling.
pub struct DeviceManager<'a> {
    /// System allocator reference.
    resource: &'a mut SystemAllocator,
    /// Devices information mapped by name.
    devices: HashMap<String, DeviceDescriptor>,
    /// Range mapping for VM exit mmio operations.
    mmio_bus: BTreeMap<Range, Arc<Mutex<dyn Device>>>,
    /// Range mapping for VM exit pio operations.
    pio_bus: BTreeMap<Range, Arc<Mutex<dyn Device>>>,
}

impl<'a> DeviceManager<'a> {
    /// Create a new `DeviceManager` with a `SystemAllocator` reference which would be
    /// used to allocate resource for devices.
    pub fn new(resource: &'a mut SystemAllocator) -> Self {
        DeviceManager {
            resource,
            devices: HashMap::new(),
            mmio_bus: BTreeMap::new(),
            pio_bus: BTreeMap::new(),
        }
    }

    fn insert(&mut self, dev: DeviceDescriptor) -> Result<()> {
        // Insert if the key is non-present, else report error.
        if self.devices.get(&(dev.name)).is_some() {
            return Err(Error::Exist);
        }
        self.devices.insert(dev.name.clone(), dev);
        Ok(())
    }

    fn remove(&mut self, name: String) -> Option<DeviceDescriptor> {
        self.devices.remove(&name)
    }

    fn device_descriptor(
        &self,
        dev: Arc<Mutex<dyn Device>>,
        parent_bus: Option<Arc<Mutex<dyn Device>>>,
        resource: Vec<Resource>,
    ) -> DeviceDescriptor {
        let name = dev.lock().expect("Failed to require lock").name();
        DeviceDescriptor::new(name, dev.clone(), parent_bus, resource)
    }

    fn allocate_resources(&mut self, resource: &mut Vec<Resource>) -> Result<()> {
        let mut alloc_idx = 0;

        for res in resource.iter_mut() {
            match res.res_type {
                IoType::Pio => {
                    if res.addr.is_none() {
                        return Err(Error::NonePIOAddress);
                    }
                    res.addr = self
                        .resource
                        .allocate_io_addresses(res.addr.unwrap(), res.size);
                }
                IoType::PhysicalMmio | IoType::Mmio => {
                    res.addr = self.resource.allocate_mmio_addresses(res.addr, res.size)
                }
            }
            if res.addr.is_none() {
                // Failed to allocate resource.
                break;
            }
            alloc_idx += 1;
        }

        // Successfully allocate.
        if alloc_idx == resource.len() {
            return Ok(());
        }

        // Failed and free the previous resource.
        self.free_resources(&resource[0..alloc_idx]);
        Err(Error::Overlap)
    }

    fn free_resources(&mut self, resource: &[Resource]) {
        for res in resource.iter() {
            match res.res_type {
                IoType::Pio => self.resource.free_io_addresses(res.addr.unwrap(), res.size),
                IoType::PhysicalMmio | IoType::Mmio => self
                    .resource
                    .free_mmio_addresses(res.addr.unwrap(), res.size),
            }
        }
    }

    fn register_resource(
        &mut self,
        dev: Arc<Mutex<dyn Device>>,
        resource: &mut Vec<Resource>,
    ) -> Result<()> {
        for res in resource.iter() {
            match res.res_type {
                IoType::Pio => {
                    if self
                        .pio_bus
                        .insert(Range(res.addr.unwrap(), res.size), dev.clone())
                        .is_some()
                    {
                        return Err(Error::Overlap);
                    }
                }
                IoType::Mmio => {
                    if self
                        .mmio_bus
                        .insert(Range(res.addr.unwrap(), res.size), dev.clone())
                        .is_some()
                    {
                        return Err(Error::Overlap);
                    }
                }
                IoType::PhysicalMmio => continue,
            };
        }
        Ok(())
    }

    /// Register a new device with its parent bus and resource request set.
    pub fn register_device(
        &mut self,
        dev: Arc<Mutex<dyn Device>>,
        parent_bus: Option<Arc<Mutex<dyn Device>>>,
        resource: &mut Vec<Resource>,
    ) -> Result<()> {
        // Reserve resource
        if let Err(e) = self.allocate_resources(resource) {
            return Err(e);
        }

        // Register device resource
        if let Err(Error::Overlap) = self.register_resource(dev.clone(), resource) {
            return Err(Error::Overlap);
        }

        // Set the allocated resource back
        dev.lock()
            .expect("Failed to acquire lock.")
            .set_resources(resource);

        // Insert bus/device to DeviceManager with parent bus
        let descriptor = self.device_descriptor(dev, parent_bus, resource.to_vec());
        self.insert(descriptor)
    }

    /// Unregister a device from `DeviceManager`.
    pub fn unregister_device(&mut self, dev: Arc<Mutex<dyn Device>>) -> Result<()> {
        let name = dev.lock().expect("Failed to acquire lock").name();

        if let Some(descriptor) = self.remove(name) {
            for res in descriptor.resource.iter() {
                if res.addr.is_some() {
                    match res.res_type {
                        IoType::Pio => self.pio_bus.remove(&Range(res.addr.unwrap(), res.size)),
                        IoType::Mmio => self.mmio_bus.remove(&Range(res.addr.unwrap(), res.size)),
                        IoType::PhysicalMmio => continue,
                    };
                }
            }
            // Free the resource
            self.free_resources(&descriptor.resource);
            Ok(())
        } else {
            Err(Error::NonExist)
        }
    }
}
