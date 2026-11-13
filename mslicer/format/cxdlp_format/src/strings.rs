use anyhow::{Result, ensure};
use common::serde::{Deserializer, Serializer, SliceDeserializer};

use crate::reader::ensure_remaining;

/// Write `value` as a big endian `u32` length, the UTF-8 bytes and a null byte.
///
/// An empty `value` becomes a zero length with no bytes, as UVtools writes it.
pub fn write_terminated(ser: &mut impl Serializer, value: &str) {
    if value.is_empty() {
        ser.write_u32_be(0);
        return;
    }
    ser.write_u32_be(value.len() as u32 + 1);
    ser.write_bytes(value.as_bytes());
    ser.write_u8(0);
}

/// Read a string that [`write_terminated`] wrote, without its null bytes.
pub fn read_terminated(des: &mut SliceDeserializer) -> Result<String> {
    ensure_remaining(des, 4, "string length")?;
    let length = des.read_u32_be() as usize;
    ensure_remaining(des, length, "string")?;
    let bytes = des.read_slice(length);
    let text = String::from_utf8_lossy(bytes);
    Ok(text.trim_end_matches('\0').to_owned())
}

/// Write `value` as a big endian `u32` byte length and UTF-16 big endian text.
pub fn write_utf16(ser: &mut impl Serializer, value: &str) {
    let units: Vec<u16> = value.encode_utf16().collect();
    ser.write_u32_be(units.len() as u32 * 2);
    for unit in units {
        ser.write_u16_be(unit);
    }
}

/// Read a string that [`write_utf16`] wrote.
pub fn read_utf16(des: &mut SliceDeserializer) -> Result<String> {
    ensure_remaining(des, 4, "string length")?;
    let length = des.read_u32_be() as usize;
    ensure!(length % 2 == 0, "UTF-16 string has an odd byte length.");
    ensure_remaining(des, length, "string")?;
    let units: Vec<u16> = (0..length / 2).map(|_| des.read_u16_be()).collect();
    Ok(String::from_utf16(&units)?)
}

/// Return `value` as the shortest decimal text after rounding to `decimals` places.
pub fn format_decimal(value: f32, decimals: i32) -> String {
    let factor = 10f32.powi(decimals);
    format!("{}", (value * factor).round() / factor)
}
