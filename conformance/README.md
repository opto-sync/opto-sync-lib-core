# Conformance

Conformance in this repository is implementation conformance: the real Rust runtime and SeaORM boundaries must satisfy the contracts they consume.

Current executable evidence is the repository's existing Rust suite:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo test --manifest-path admin-orm/Cargo.toml --all-targets --all-features
```

The strict `.opto-sync.toml` projection is exercised through the Rust tests around `src/opto_sync_config.rs`; contract provenance is recorded in `config-source.json` and separately admitted through the TypeSpec/JSON-Schema peer-authority workflow.

## Claim boundary

This repository currently does **not** own the Opto Sync conflict-resolution engine. Therefore a generic revision/tombstone ordering model is not lib-core conformance evidence and is intentionally not maintained here. Conflict/convergence models belong beside the implementation they refine or in the cross-runtime E2E/formal harness, with explicit implementation commits.

Exact-head evidence rules apply: queued, skipped, zero-step, stale-head, billing/admission-blocked, missing-run, and historical-only results are not green.
