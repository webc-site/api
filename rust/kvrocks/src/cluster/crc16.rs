use memchr::memchr;

pub const CRC: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_XMODEM);

pub const CLUSTER_SLOTS: usize = 16384;
pub const CLUSTER_SLOT_MASK: u16 = (CLUSTER_SLOTS - 1) as u16;

pub fn crc16(buf: &[u8]) -> u16 {
    CRC.checksum(buf)
}

pub fn hash_tag(key: &[u8]) -> &[u8] {
    if let Some(s) = memchr(b'{', key)
        && let Some(e) = memchr(b'}', &key[s + 1..])
        && e > 0
    {
        return &key[s + 1..s + 1 + e];
    }
    key
}

pub fn slot(key: &[u8]) -> u16 {
    let tag = hash_tag(key);
    crc16(tag) & CLUSTER_SLOT_MASK
}
