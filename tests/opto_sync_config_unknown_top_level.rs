use opto_sync_lib_core::{parse_opto_sync_config, OptoSyncConfigError};

#[test]
fn unknown_top_level_fields_are_rejected() {
    let config = r#"
version = 1
mode = "client"
strict = true
surprise = "nope"

[flags2env]
contract = ".cli-flags.toml"
require_audit = true
precedence = "argv-over-env"

[sync]
push_interval_ms = 1000
pull_interval_ms = 1000
max_batch_size = 8
conflict_policy = "manual"

[client]
enabled = true
"#;
    assert!(matches!(parse_opto_sync_config(config), Err(OptoSyncConfigError::Toml(_))));
}
