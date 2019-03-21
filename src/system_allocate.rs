use std::option::Option;


pub struct SystemAllocator {

}

impl SystemAllocator {
    // Return base address.
    pub fn allocate_io_addresses(&mut self, _size: u64) -> Option<u64> {
        Some(0)
    }
    pub fn allocate_mmio_addresses(&mut self, _size: u64) -> Option<u64> {
        Some(0)
    }
}
