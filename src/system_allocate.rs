// Copyright (C) 2019 Intel Corporation. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use std::option::Option;

pub struct SystemAllocator {

}

impl SystemAllocator {
    pub fn new() -> Self {
        SystemAllocator {}
    }
    // Return base address.
    pub fn allocate_pio_addresses(&mut self, _size: u64) -> Option<u64> {
        Some(0)
    }
    pub fn allocate_mmio_addresses(&mut self, _size: u64) -> Option<u64> {
        Some(0)
    }
}
