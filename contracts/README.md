# Contract consumption boundary

`opto-sync-lib-core` implements runtime and persistence projections; it is not the authored wire-contract authority.

The authoritative Opto Sync interface contracts live in `opto-sync/opto-sync-interfaces` as independent human-authored TypeSpec and JSON Schema Draft 2020-12 peers. `config-source.json` pins the reviewed interface revision and the TJSV admission revision used for the local `.opto-sync.toml` projection.

Rules for this repository:

- do not re-author payload or wire schemas here;
- generated or runtime Rust types are implementation projections, never peer authorities;
- preserve exact external wire names when mapping into Rust;
- fail closed when a pinned authority is missing, unsupported, or disagrees with its peer;
- schema/database changes are coordinated through Declarative Migrations; application startup must not silently mutate server schema.

A change to `src/opto_sync_config.rs` that changes accepted configuration semantics must be paired with an authored contract change in `opto-sync-interfaces` and refreshed immutable provenance in `config-source.json`.
