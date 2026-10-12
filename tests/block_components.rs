use {
    solana_entry::entry::Entry,
    solana_hash::Hash,
    solana_keypair::Keypair,
    solana_shred::{
        block_component::{self, BlockComponent},
        shred::{ProcessShredsStats, ReedSolomonCache, Shred, Shredder},
    },
};

fn shreds(data: &[u8], last: bool) -> Vec<Shred> {
    Shredder::new(42, 41, 0, 1)
        .unwrap()
        .make_shreds_from_data_slice(
            &Keypair::new(),
            data,
            last,
            Hash::default(),
            0,
            0,
            &ReedSolomonCache::default(),
            &mut ProcessShredsStats::default(),
        )
        .unwrap()
        .collect()
}

fn deshred(shreds: &[Shred]) -> Result<Vec<u8>, solana_shred::shred::Error> {
    Shredder::deshred(shreds.iter().filter(|s| s.is_data()).map(Shred::payload))
}

#[test]
fn pre_alpenglow_entries_keep_their_wire_encoding() {
    let entries = vec![Entry {
        num_hashes: 1,
        hash: Hash::default(),
        transactions: vec![],
    }];
    let wire = wincode::serialize(&entries).unwrap();
    let data = deshred(&shreds(&wire, false)).unwrap();
    assert_eq!(data, wire);
    assert_eq!(
        block_component::decode(&data).unwrap(),
        BlockComponent::EntryBatch(entries)
    );
}

#[test]
fn header_is_preserved_as_a_marker_instead_of_an_empty_batch() {
    let component = BlockComponent::new_block_header(41, Hash::default());
    let wire = wincode::serialize(&component).unwrap();
    let data = deshred(&shreds(&wire, false)).unwrap();
    assert_eq!(data, wire);
    assert_eq!(block_component::decode(&data).unwrap(), component);
}

#[test]
fn checked_deshred_rejects_incomplete_or_gapped_segments() {
    let generated = shreds(&vec![1; 40_000], false);
    let data: Vec<_> = generated.into_iter().filter(Shred::is_data).collect();
    assert!(data.len() > 32);
    assert!(deshred(&data[..32]).is_err());
    let with_gap: Vec<_> = data
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != 1)
        .map(|(_, s)| s.clone())
        .collect();
    assert!(deshred(&with_gap).is_err());
    assert_eq!(deshred(&data).unwrap(), vec![1; 40_000]);
}

#[test]
fn malformed_markers_and_trailing_bytes_are_rejected() {
    let header =
        wincode::serialize(&BlockComponent::new_block_header(41, Hash::default())).unwrap();
    for length in 9..header.len() {
        assert!(block_component::decode(&header[..length]).is_err());
    }
    let mut trailing = header.clone();
    trailing.push(1);
    assert!(block_component::decode(&trailing).is_err());
    let mut unknown_version = header;
    unknown_version[8..10].copy_from_slice(&999u16.to_le_bytes());
    assert!(block_component::decode(&unknown_version).is_err());
}

#[test]
fn fixed_fec_zero_padding_is_supported_for_entries_and_markers() {
    let entries = vec![Entry {
        num_hashes: 1,
        hash: Hash::default(),
        transactions: vec![],
    }];
    for component in [
        BlockComponent::EntryBatch(entries),
        BlockComponent::new_block_header(41, Hash::default()),
    ] {
        let mut wire = wincode::serialize(&component).unwrap();
        wire.resize(wire.len() + 1024, 0);
        assert_eq!(block_component::decode(&wire).unwrap(), component);
        wire.push(1);
        assert!(block_component::decode(&wire).is_err());
    }
    assert_eq!(
        block_component::decode(&[0; 2048]).unwrap(),
        BlockComponent::EntryBatch(vec![])
    );
}

#[test]
fn legacy_empty_batches_and_empty_shred_padding_remain_supported() {
    assert_eq!(
        block_component::decode(&[0; 8]).unwrap(),
        BlockComponent::EntryBatch(vec![])
    );
    let padded = deshred(&shreds(&[], true)).unwrap();
    assert_eq!(
        block_component::decode(&padded).unwrap(),
        BlockComponent::EntryBatch(vec![])
    );
    assert!(block_component::decode(&[]).is_err());
}

#[test]
fn all_alpenglow_markers_roundtrip_with_footer_certificate() {
    use solana_shred::block_component::{
        BlockFinalizationCert, BlockFooterV1, GenesisCertBlockMarker, UpdateParentV1,
        VersionedBlockMarker,
    };
    let markers = [
        VersionedBlockMarker::from_block_footer(BlockFooterV1 {
            bank_hash: Hash::default(),
            block_producer_time_nanos: 123,
            block_user_agent: b"agave-4.3".to_vec(),
            block_final_cert: Some(BlockFinalizationCert::new_for_tests()),
            skip_reward_cert: None,
            notar_reward_cert: None,
        }),
        VersionedBlockMarker::from_update_parent(UpdateParentV1 {
            new_parent_slot: 40,
            new_parent_block_id: Hash::default(),
        }),
        VersionedBlockMarker::from_genesis_cert_block_marker(GenesisCertBlockMarker {
            slot: 41,
            block_id: Hash::default(),
            bls_signature: solana_bls_signatures::Signature(
                [0; solana_bls_signatures::BLS_SIGNATURE_AFFINE_SIZE],
            ),
            bitmap: vec![1],
        }),
    ];
    for marker in markers {
        let component = BlockComponent::new_block_marker(marker);
        let wire = wincode::serialize(&component).unwrap();
        let data = deshred(&shreds(&wire, true)).unwrap();
        assert_eq!(block_component::decode(&data).unwrap(), component);
    }
}
