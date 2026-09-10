use opto_sync_lib_core::{parse_opto_sync_config, OptoSyncConfigError};

#[test]
fn nats_bindings_must_be_url_and_secret() {
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
name = "nats_url"
key = "NATS_URL"
kind = "url"
required = false
secret = false

[server]
enabled = true
nats_url_binding = "nats_url"
"#;
    assert!(matches!(parse_opto_sync_config(config), Err(OptoSyncConfigError::BindingSecretMismatch { .. })));
}
