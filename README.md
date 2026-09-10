# opto-sync-lib-core

SeaORM boundary for Opto Sync on PostgreSQL and CockroachDB.

## Domain configuration

Strict `.opto-sync.toml` runtime resolution is implemented in `src/opto_sync_config.rs`. The authored TypeSpec and JSON Schema Draft 2020-12 authorities remain in `opto-sync/opto-sync-interfaces`; the runtime projection is admitted against that contract through pinned TJSV provenance in `config-source.json`. Executables keep `.cli-flags.toml` and `flags-2-env` as the sole argv boundary.
