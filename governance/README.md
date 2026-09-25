# Governance

This repository promotes implementation changes only when their claims are backed by the implementation present here and by exact-head executed evidence.

## Contract changes

The local runtime may consume contracts but must not silently become a competing authority. Contract-semantic changes require:

1. the authored TypeSpec and Draft 2020-12 JSON Schema peers in `opto-sync-interfaces` to be updated independently;
2. TJSV admission to pass fail-closed;
3. `config-source.json` to pin the reviewed immutable revisions;
4. Rust implementation tests to execute against the candidate head.

## Evidence discipline

Do not count queued, skipped, zero-step, stale-head, billing/admission-blocked, missing-run, or historical-only checks as green. Do not infer convergence, CRDT/OT, causal ordering, failover, or durability properties from config/runtime tests.

Formal/model checks must refine an implementation or an explicit contract in scope. A mathematically valid model that is disconnected from this repository's code is design evidence, not implementation conformance.
