//! Self-contained replacement for the fork's developer-local capture scratch test.

use {
    solana_sdk::{hash::Hash, signature::Signer, signer::keypair::keypair_from_seed},
    solana_shred::shred::{merkle, ProcessShredsStats, ReedSolomonCache, Shred, Shredder},
    test_case::test_case,
};

#[test_case(false, false)]
#[test_case(true, false)]
#[test_case(true, true)]
fn multiple_missing_data_shreds_recover_exact_payloads(chained: bool, last_in_slot: bool) {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .unwrap();
    let keypair = keypair_from_seed(&[7; 32]).unwrap();
    let data: Vec<_> = (0..16_384).map(|index| (index % 251) as u8).collect();
    let cache = ReedSolomonCache::default();
    let original = merkle::make_shreds_from_data(
        &pool,
        &keypair,
        chained.then(|| Hash::new_from_array([8; 32])),
        &data,
        42,
        41,
        1,
        0,
        last_in_slot,
        0,
        0,
        &cache,
        &mut ProcessShredsStats::default(),
    )
    .unwrap();
    let mut available = Vec::new();
    let mut missing = Vec::new();
    for shred in &original {
        if matches!(shred, merkle::Shred::ShredData(_)) && shred.index() % 3 != 0 {
            missing.push(shred);
        } else {
            // Recovery must work on parsed wire payloads, not just generator objects.
            available.push(merkle::Shred::from_payload(shred.payload().clone()).unwrap());
        }
    }
    assert!(missing.len() > 1);
    let recovered = merkle::recover(available.clone(), &cache)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(recovered.len(), missing.len());
    for shred in &recovered {
        let expected = missing
            .iter()
            .find(|other| other.index() == shred.index())
            .unwrap();
        assert_eq!(shred, *expected);
        let parsed = Shred::new_from_serialized_shred(shred.payload().clone()).unwrap();
        parsed.sanitize().unwrap();
        assert!(parsed.verify(&keypair.pubkey()));
    }
    available.extend(recovered);
    available.retain(|shred| matches!(shred, merkle::Shred::ShredData(_)));
    available.sort_by_key(merkle::Shred::index);
    let payloads = || available.iter().map(|shred| shred.payload().as_ref());
    assert_eq!(Shredder::deshred(payloads()).unwrap(), data);
    assert_eq!(Shredder::deshred_unchecked(payloads()).unwrap(), data);
}
