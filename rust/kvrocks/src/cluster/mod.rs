pub mod crc16;
pub mod slots;

pub use crc16::{crc16, hash_tag, slot};
pub use slots::SlotMap;
