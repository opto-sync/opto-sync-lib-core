use opto_sync_lib_core::{parse_opto_sync_config, OptoSyncConfigError};

#[test]
fn disabled_declared_role_fails_mode_validation() {
    let config = r#"
version = 1
mode = "client"
strict = true

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
enabled = false
"#;
    assert!(matches!(parse_opto_sync_config(config), Err(OptoSyncConfigError::ModeRoleMismatch(_))));
}
