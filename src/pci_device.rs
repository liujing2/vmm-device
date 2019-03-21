// Copyright 2019 Intel Corporation. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use super::dev::Device;
use super::pci_configuration::PciConfiguration;

pub trait PciDevice: Send + Device {
    /// Gets the configuration registers of the Pci Device.
    fn config_registers(&self) -> &PciConfiguration;
    /// Gets the configuration registers of the Pci Device for modification.
    fn config_registers_mut(&mut self) -> &mut PciConfiguration;

    /// Read the configuration register according to register index.
    fn config_register_read(&self, _reg_idx: usize) -> u32 {0}

    /// Write the configuration register according to register index and offset.
    fn config_register_write(&self, reg_idx: usize, offset: u64, data: &[u8]);
}

