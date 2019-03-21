// Copyright 2019 Intel Corporation. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use super::dev::Device;
//use super::pci_configuration::PciConfiguration;

// This trait will use pci_configuration::PciConfiguration but for clear design
// review and less dependency in example device realization, we temporarily use
// simple value for now.
pub trait PciDevice: Send + Device {
    /// Gets the configuration registers of the Pci Device.
    fn config_registers(&self) -> &[u32];
    /// Gets the configuration registers of the Pci Device for modification.
    fn config_registers_mut(&mut self) -> &mut [u32];

    /// Read the configuration register according to register index.
    fn config_register_read(&self, _reg_idx: usize) -> u32 {0}

    /// Write the configuration register according to register index and offset.
    fn config_register_write(&mut self, reg_idx: usize, offset: u64, data: &[u8]);
}

