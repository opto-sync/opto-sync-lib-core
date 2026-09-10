use opto_sync_lib_core::{parse_opto_sync_config, OptoSyncConfigError};

#[test]
fn server_bind_address_must_not_be_secret() {
    let config = r#"
version = 1
mode = "server"
strict = true

[flags2env]
contract = ".cli-flags.toml"
require_audit = true
precedence = "argv-over-env"

[sync]
push_interval_ms = 1000
pull_interval_ms = 1000
max_batch_size = 8
conflict_policy = "server-wins"

[[env]]
name = "bind_addr"
key = "OPTO_SYNC_API_BIND"
kind = "string"
required = false
secret = true

[server]
enabled = true
bind_addr_binding = "bind_addr"
"#;
    assert!(matches!(parse_opto_sync_config(config), Err(OptoSyncConfigError::BindingSecretMismatch { .. })));
}
