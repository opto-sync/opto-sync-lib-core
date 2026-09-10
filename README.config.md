# `.opto-sync.toml` runtime support

This crate contains the strict Rust implementation projection for the Opto Sync domain configuration contract.

The authored authorities remain `opto-sync/opto-sync-interfaces` TypeSpec and JSON Schema Draft 2020-12 sources. `ORESoftware/typespec-json-schema-validator` admits those authorities fail-closed; generated schemas and this Rust module are implementation/evidence, never additional authorities.

Executables continue to audit and parse `.cli-flags.toml` through the official `flags-2-env` runtime. They should pass only argv-derived `provided_flags` as the override map to `resolve_opto_sync_config`, preserving precedence `argv > ambient environment > non-secret domain default`. Secret bindings must not be supplied through argv and must not declare plaintext defaults.

Repositories containing both client and server entry points use `mode = "hybrid"` and one shared binding inventory. Each entry point consumes only the role-specific references relevant to that executable.
