use opto_sync_lib_core::{parse_opto_sync_config, OptoSyncMode};

#[test]
fn hybrid_mode_requires_and_accepts_both_roles() {
    let config = parse_opto_sync_config(r#"
version = 1
mode = "hybrid"
strict = true

[flags2env]
contract = ".cli-flags.toml"
require_audit = true
precedence = "argv-over-env"

[sync]
push_interval_ms = 1000
pull_interval_ms = 1000
max_batch_size = 8
conflict_policy = "last-write-wins"

[client]
enabled = true

[server]
enabled = true
"#).expect("valid hybrid config");
    assert_eq!(config.mode, OptoSyncMode::Hybrid);
}
