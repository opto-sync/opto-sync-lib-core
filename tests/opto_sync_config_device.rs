use opto_sync_lib_core::{parse_opto_sync_config, OptoSyncConfigError};

#[test]
fn device_id_must_be_string_and_non_secret() {
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

[[env]]
name = "device_id"
key = "OPTO_SYNC_DEVICE_ID"
kind = "integer"
required = false
secret = false

[client]
enabled = true
device_id_binding = "device_id"
"#;
    assert!(matches!(parse_opto_sync_config(config), Err(OptoSyncConfigError::BindingKindMismatch { .. })));
}
