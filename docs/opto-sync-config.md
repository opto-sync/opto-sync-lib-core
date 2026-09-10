# Opto Sync configuration boundary

`.opto-sync.toml` is a domain runtime configuration file, not a command-line schema. Executables keep `.cli-flags.toml` as the only argv contract and parse it through the official `flags-2-env` binding. Only `StructuredParse.provided_flags`-equivalent argv overrides should be handed to the domain resolver; default-bearing parser output must not shadow ambient environment values.

Precedence is `argv override > ambient environment > non-secret .opto-sync.toml default`. Secret bindings reject plaintext defaults and argv delivery. Credential-bearing database, NATS, and auth token values remain environment/secret-store only.

A repository with both client and server entry points declares `mode = "hybrid"`, enables both role tables, and shares one binding inventory. A client-only entry point does not receive server-only database/NATS bindings, and a server-only entry point does not receive client auth/device bindings.

The independent authored authorities live in `opto-sync/opto-sync-interfaces` and are admitted by `ORESoftware/typespec-json-schema-validator`. This Rust implementation is an implementation projection only.
