//! Decode complete data segments in both TowerBFT and Alpenglow wire formats.

pub use solana_entry::block_component::*;

/// Entry batches retain their pre-Alpenglow encoding. A zero entry count with
/// additional nonzero bytes is a marker, never an empty entry batch. The
/// all-zero buffer is retained for padded legacy empty batches and the upstream
/// empty-shred compatibility case. Fixed FEC batches can pad complete components
/// with zeros; nonzero trailing data is rejected.
/// In Alpenglow an empty entry batch indicates a block abort; consumers must
/// distinguish that event from a footer.
pub fn decode(data: &[u8]) -> Result<BlockComponent, wincode::ReadError> {
    if data.len() >= 8 && data.iter().all(|byte| *byte == 0) {
        return Ok(BlockComponent::EntryBatch(Vec::new()));
    }
    let mut remaining = data;
    let component = wincode::deserialize_from(&mut remaining)?;
    if remaining.iter().any(|byte| *byte != 0) {
        return Err(wincode::ReadError::TrailingBytes);
    }
    Ok(component)
}
