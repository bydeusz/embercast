use anyhow::{Result, bail};
use common::serde::{Deserializer, Serializer, SliceDeserializer};

use crate::{
    PAGE_BREAK,
    reader::{ensure_remaining, expect_page_break},
};

/// One vertical line of pixels with one gray value.
///
/// The line starts at row `y_start` and stops at row `y_end`, both inclusive,
/// in column `x`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Line {
    pub x: u16,
    pub y_start: u16,
    pub y_end: u16,
    pub gray: u8,
}

/// The pixel data of one layer.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Layer {
    /// The area of the largest shape in the layer, in square millimeters times 1000.
    pub largest_area: u32,
    /// The lines of the layer, sorted by column and then by row.
    pub lines: Vec<Line>,
}

impl Line {
    /// The byte count of one line in the file.
    pub const SIZE: usize = 6;

    /// Return the six bytes of the line: 13 bits `y_start`, 13 bits `y_end`, 14 bits `x`, then the gray value.
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let (y_start, y_end, x) = (self.y_start as u32, self.y_end as u32, self.x as u32);
        [
            (y_start >> 5) as u8,
            ((y_start << 3) | (y_end >> 10)) as u8,
            (y_end >> 2) as u8,
            ((y_end << 6) | (x >> 8)) as u8,
            x as u8,
            self.gray,
        ]
    }

    /// Return the line that [`Line::to_bytes`] wrote into `bytes`.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let [b0, b1, b2, b3, b4, gray] = [0, 1, 2, 3, 4, 5].map(|i| bytes[i] as u32);
        Self {
            y_start: ((((b0 << 8) | b1) >> 3) & 0x1FFF) as u16,
            y_end: ((((b1 << 16) | (b2 << 8) | b3) >> 6) & 0x1FFF) as u16,
            x: (((b3 << 8) | b4) & 0x3FFF) as u16,
            gray: gray as u8,
        }
    }
}

impl Layer {
    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_u32_be(self.largest_area);
        ser.write_u32_be(self.lines.len() as u32);
        for line in &self.lines {
            ser.write_bytes(&line.to_bytes());
        }
        ser.write_bytes(PAGE_BREAK);
    }

    pub fn deserialize(des: &mut SliceDeserializer) -> Result<Self> {
        ensure_remaining(des, 8, "layer header")?;
        let largest_area = des.read_u32_be();
        let line_count = des.read_u32_be() as usize;
        let Some(byte_count) = line_count.checked_mul(Line::SIZE) else {
            bail!("Layer has too many lines: {line_count}.");
        };
        ensure_remaining(des, byte_count, "layer lines")?;

        let lines = des
            .read_slice(byte_count)
            .chunks_exact(Line::SIZE)
            .map(Line::from_bytes)
            .collect();
        expect_page_break(des, "layer")?;

        Ok(Self {
            largest_area,
            lines,
        })
    }
}
