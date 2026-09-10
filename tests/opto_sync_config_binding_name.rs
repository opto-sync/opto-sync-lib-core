use opto_sync_lib_core::{parse_opto_sync_config, OptoSyncConfigError};

#[test]
fn rejects_noncanonical_binding_names() {
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
name = "Bad-Name"
key = "OPTO_SYNC_VALUE"
kind = "string"
required = false
secret = false

[client]
enabled = true
"#;
    assert!(matches!(
        parse_opto_sync_config(config),
        Err(OptoSyncConfigError::InvalidBindingName(_))
    ));
}
