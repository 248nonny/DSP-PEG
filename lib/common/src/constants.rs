// Must be cache aligned (64 bytes)
pub const SHARED_BASE_PHYSICAL_ADDR: usize = 0x10000000;

pub const SHARED_SIZE: usize = 0x01000000;

// Must also be cache aligned (64 bytes)
pub const LOGGING_RING_BUFFER_LOCATION: usize = 0x80; // leave 128 bytes blank
