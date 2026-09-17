# solana-shred

Standalone Solana 2.2 shred parsing, Merkle Reed–Solomon FEC recovery, shred
generation, and deshredding. No Blockstore, RocksDB, or git dependencies.

```toml
[dependencies]
solana-shred = "=2.2.0"
```

The public API lives in `solana_shred::shred`. Deshredding returns raw ledger
entry bytes; entry decoding is the caller's responsibility.

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
`Shredder::deshred_unchecked` relaxes only the final-marker requirement; it
still rejects gaps and trailing data after a completed segment. Callers must
validate and order input shreds. Recover missing Merkle shreds with
`shred::merkle::recover` and `shred::ReedSolomonCache` before deshredding.

## Provenance

The `src/shred.rs`, `src/shredder.rs`, and `src/shred/` modules, including their
upstream regression tests, were extracted from `ledger/src/` in:

- Repository: <https://github.com/jito-foundation/jito-solana>
- Commit: `cafa3ba4d4c6c3702d13545d9b434861ebc06036` (Jito Solana 2.2)
- License: Apache-2.0; the upstream license is included in `LICENSE`.

Supporting `solana-entry`, `solana-perf`, `solana-measure`, `solana-metrics`,
and `solana-rayon-threadlimit` crates are pinned to published `2.2.0` releases.
Their source directories were compared byte-for-byte with the pinned commit
and are identical. Building does not fetch the upstream repository.

Changes from the pinned source:

- Move `MAX_DATA_SHREDS_PER_SLOT` and the storage-independent
  `verify_shred_slots` predicate out of Blockstore; preserve the regression test.
- Update internal imports and omit the unused nightly frozen-ABI derives.
- Remove unused storage-only identifiers, re-padding and erasure-mismatch
  helpers, and the generic recovery wrapper. Keep helpers used only by
  upstream wire-format tests under `cfg(test)`.
- Apply formatting and compiler/Clippy compatibility fixes without changing
  wire layout or recovery algorithms. Constructor argument-count lint
  expectations preserve the original independent protocol fields.
- Replace a developer-local binary-capture scratch test with self-contained
  multi-erasure tests covering unchained, chained, and retransmitter-signed
  Merkle shreds. Four originally ignored exhaustive/paranoid tests remain
  explicitly ignored.

Version `2.2.0` identifies the extracted implementation lineage, not support
for every future wire format. Entry/transaction codec upgrades are independent.

## Development

Use Rust 1.95.0 and run:

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo test --locked --doc
cargo package --locked
```

When changing the implementation, compare against the recorded upstream commit,
retain signature/wire/FEC regression tests, and validate representative real
shred traffic in a consumer. Do not restore the monolithic `solana-ledger`
dependency to obtain a shred utility.

## Release

Review the package contents and tests, publish `solana-shred` version `2.2.0`
with `cargo publish --locked`, and tag the same source commit as `2.2.0`.
Git tags are not uploaded to crates.io; the crate version is taken from
`Cargo.toml`.
