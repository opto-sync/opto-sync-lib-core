use opto_sync_lib_core::{parse_opto_sync_config, OptoSyncConfigError};

#[test]
fn auth_tokens_must_be_secret() {
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
name = "auth_token"
key = "SHARED_AUTH_CUSTOMER_TOKEN"
kind = "string"
required = false
secret = false

[client]
enabled = true
auth_token_binding = "auth_token"
"#;
    assert!(matches!(parse_opto_sync_config(config), Err(OptoSyncConfigError::BindingSecretMismatch { .. })));
}
