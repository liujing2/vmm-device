// Copyright 2019 Intel Corporation. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

//! Handles routing to devices in an address space.
use std::cmp::{Ord, Ordering, PartialEq, PartialOrd};
use std::string::String;
use super::device_manager::DeviceManager;

/// Trait for devices that respond to reads or writes in an arbitrary address space.
///
/// The device does not care where it exists in address space as each method is only given an offset
/// into its allocated portion of address space.
#[allow(unused_variables)]
pub trait Device: Send {
    /// Get the device name.
    fn get_name(&self) -> String; 
    /// Device initialize.
    fn init(&self, dev_manager: &mut DeviceManager) {}
    /// System exit and reset.
    fn exit(&mut self) {}
}

#[derive(Debug, Copy, Clone)]
pub struct Range(pub u64, pub u64);

impl Eq for Range {}

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

