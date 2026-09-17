//! Protect the storage-independent, registry-only dependency boundary.

#[test]
fn dependencies_are_storage_independent_and_published() {
    let lockfile = include_str!("../Cargo.lock");
    for package in ["solana-ledger", "rocksdb", "librocksdb-sys"] {
        let declaration = format!("name = \"{package}\"");
        assert!(
            !lockfile.lines().any(|line| line == declaration),
            "{package} must not return to the dependency graph"
        );
    }
    assert!(
        !lockfile
            .lines()
            .any(|line| line.starts_with("source = \"git+")),
        "all dependencies must be published registry crates"
    );
}
