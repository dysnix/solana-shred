//! Preserve the pinned unchecked-deshred contract.

use solana_shred::shred::{Error, Shred, ShredFlags, Shredder};

fn data_shred(index: u32, data: &[u8], flags: ShredFlags) -> Shred {
    Shred::new_from_data(42, index, 1, data, flags, 0, 1, 0)
}

#[test]
fn unchecked_deshred_allows_a_segment_without_the_final_complete_flag() {
    let shreds = [
        data_shred(0, b"first", ShredFlags::empty()),
        data_shred(1, b"second", ShredFlags::empty()),
    ];
    let payloads = || shreds.iter().map(|shred| shred.payload().as_ref());
    assert_eq!(
        Shredder::deshred_unchecked(payloads()).unwrap(),
        b"firstsecond"
    );
    assert!(matches!(
        Shredder::deshred(payloads()),
        Err(Error::ErasureError(_))
    ));
}

#[test]
fn unchecked_deshred_still_rejects_noncontiguous_indices() {
    let shreds = [
        data_shred(0, b"first", ShredFlags::empty()),
        data_shred(2, b"second", ShredFlags::DATA_COMPLETE_SHRED),
    ];
    assert!(matches!(
        Shredder::deshred_unchecked(shreds.iter().map(|shred| shred.payload().as_ref())),
        Err(Error::ErasureError(_))
    ));
}

#[test]
fn unchecked_deshred_still_rejects_trailing_shreds_after_complete_data() {
    let shreds = [
        data_shred(0, b"first", ShredFlags::DATA_COMPLETE_SHRED),
        data_shred(1, b"second", ShredFlags::DATA_COMPLETE_SHRED),
    ];
    assert!(matches!(
        Shredder::deshred_unchecked(shreds.iter().map(|shred| shred.payload().as_ref())),
        Err(Error::InvalidDeshredSet)
    ));
}
