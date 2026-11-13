const POLYNOMIAL: u32 = 0xEDB8_8320;
const TABLE: [u32; 256] = build_table();

const fn build_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut index = 0;
    while index < 256 {
        let mut value = index as u32;
        let mut bit = 0;
        while bit < 8 {
            value = if value & 1 == 0 {
                value >> 1
            } else {
                (value >> 1) ^ POLYNOMIAL
            };
            bit += 1;
        }
        table[index] = value;
        index += 1;
    }
    table
}

/// Return the CRC-32 of `data` without the start and end inversion.
///
/// Version 3 files end with this value. The calculation starts at zero and
/// does not invert the result, so the value is not the usual CRC-32.
pub fn crc32_raw(data: &[u8]) -> u32 {
    data.iter().fold(0, |crc, &byte| {
        TABLE[((crc ^ byte as u32) & 0xFF) as usize] ^ (crc >> 8)
    })
}

/// Return the XOR of all bytes in `data`, as version 2 files store it.
pub fn xor_checksum(data: &[u8]) -> u8 {
    data.iter().fold(0, |acc, &byte| acc ^ byte)
}
