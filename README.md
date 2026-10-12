# solana-shred

Standalone Jito/Agave 4.3 shred parsing, Merkle Reed–Solomon FEC recovery, shred
generation, and deshredding. No Blockstore, RocksDB, or git dependencies.

```toml
[dependencies]
solana-shred = { git = "https://github.com/dysnix/solana-shred.git", tag = "4.3.0" }
```

The public API lives in `solana_shred::shred`. Deshredding returns raw ledger
component bytes; `block_component::decode` distinguishes entry batches from markers.

```rust
use solana_shred::shred::{Shred, Shredder};

fn deshred(payloads: Vec<Vec<u8>>) -> Result<Vec<u8>, solana_shred::shred::Error> {
    let shreds = payloads
        .into_iter()
        .map(Shred::new_from_serialized_shred)
        .collect::<Result<Vec<_>, _>>()?;
    Shredder::deshred(shreds.iter().map(|shred| shred.payload().as_ref()))
}
```

`Shredder::deshred` checks continuity and the final data-complete marker.
The unchecked fork patch is removed. Callers must validate and order input
shreds and assemble the complete data segment, which may span multiple FEC sets. Recover missing Merkle shreds with
`shred::merkle::recover` and `shred::ReedSolomonCache` before deshredding.

## Provenance

Transport/FEC and upstream regression tests are extracted from
[Jito v4.3.0-jito.0](https://github.com/jito-foundation/jito-solana/tree/c21f90741c274c8508c1d763559131c518e86764/ledger/src):
commit `c21f90741c274c8508c1d763559131c518e86764`. The upstream Apache-2.0
license is included in `LICENSE`. Support crates use published Agave 4.3
releases; building does not fetch Jito, Blockstore, or RocksDB.

Standalone adaptations expose read-only shred/header accessors and Merkle
recovery, remove the validator-specific filter and unused storage-only helpers,
and omit frozen-ABI derives. Wire layouts, generation and recovery algorithms
are retained. `block_component::decode` uses the published `solana-entry 4.3.0`
codec, accepts fixed-FEC zero padding and legacy empty batches, and rejects
nonzero trailing bytes. It decodes all versioned markers including footer
certificates; decoding does not verify consensus certificates or establish
finality. In Alpenglow an empty entry batch is a block abort.

This release supports current TowerBFT mainnet and Alpenglow entry/marker
encoding. It accepts chained Merkle shreds with optional retransmitter
signatures. Retired legacy/unchained variants are intentionally removed with
upstream: mainnet activated `drop_legacy_shreds` at slot 259200004,
`drop_unchained_merkle_shreds` at 354672000 and `enforce_fixed_fec_set` at
403920000. This is live-network compatibility, not historical ledger replay
support for those retired formats. Version 4.3 changes the standalone Rust API;
consumers must migrate generator calls and header access to the upstream types.

## Development

Use Rust 1.97.1 and run:

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo test --locked --doc
cargo package --locked
```

When updating upstream, retain wire/signature/FEC tests and replay real consumer
traffic. Two paranoid upstream tests remain ignored. Never restore the
monolithic `solana-ledger` dependency. Consumers pin an immutable source commit;
publish tag `4.3.0` only after review and validation. Registry publication is
disabled.
