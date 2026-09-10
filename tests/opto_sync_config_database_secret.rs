use opto_sync_lib_core::{parse_opto_sync_config, OptoSyncConfigError};

#[test]
fn database_bindings_must_be_url_and_secret() {
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
name = "database_url"
key = "DATABASE_URL"
kind = "url"
required = false
secret = false

[server]
enabled = true
database_url_binding = "database_url"
"#;
    assert!(matches!(parse_opto_sync_config(config), Err(OptoSyncConfigError::BindingSecretMismatch { .. })));
}
