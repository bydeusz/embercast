use anyhow::{Result, ensure};
use common::serde::{Deserializer, SliceDeserializer};

use crate::PAGE_BREAK;

/// Return an error if the file has fewer than `needed` bytes after the read position.
///
/// The deserializer stops the program when it reads past the end. Thus the
/// parser calls this function before each block, so that a damaged file gives
/// an error and not a crash.
pub fn ensure_remaining(des: &mut SliceDeserializer, needed: usize, block: &str) -> Result<()> {
    let remaining = des.size().saturating_sub(des.pos());
    ensure!(
        needed <= remaining,
        "File ends inside the {block}: {needed} bytes needed, {remaining} left."
    );
    Ok(())
}

/// Read a page break and return an error if the two bytes are not a page break.
pub fn expect_page_break(des: &mut SliceDeserializer, block: &str) -> Result<()> {
    ensure_remaining(des, PAGE_BREAK.len(), block)?;
    ensure!(
        des.read_slice(PAGE_BREAK.len()) == PAGE_BREAK,
        "The {block} does not end with a page break."
    );
    Ok(())
}
