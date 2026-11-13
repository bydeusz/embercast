//! Creality CXDLP format, version 3 (`.cxdlp`).
//!
//! The Creality Halot printers read this format. All numbers are big endian,
//! except the compensation fields in [`SlicerInfoV3`], which are little endian.
//!
//! ## References
//!
//! - [UVtools `CrealityCXDLPFile.cs`](https://github.com/sn4k3/UVtools/blob/master/UVtools.Core/FileFormats/CrealityCXDLPFile.cs)

mod area;
mod checksum;
mod encoding;
mod file;
mod header;
mod layer;
mod preview;
mod printer;
mod reader;
mod strings;

pub use area::{Span, largest_enclosed_area};
pub use checksum::crc32_raw;
pub use encoding::{LayerEncoder, decode_lines};
pub use file::File;
pub use header::{Header, SlicerInfo, SlicerInfoV3};
pub use layer::{Layer, Line};
pub use preview::PreviewImage;
pub use printer::CrealityPrinter;

/// The magic string at the start and at the end of each file, with its null byte.
pub const MAGIC: &[u8] = b"CXSW3DV2\0";
/// The two bytes that close each block of the file.
pub const PAGE_BREAK: &[u8] = &[0x0D, 0x0A];
/// The format version that this crate writes.
pub const VERSION: u16 = 3;
/// The largest column count that a line can address (14 bits).
pub const MAX_WIDTH: u32 = 1 << 14;
/// The largest row count that a line can address (13 bits).
pub const MAX_HEIGHT: u32 = 1 << 13;
