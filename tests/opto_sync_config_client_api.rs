use opto_sync_lib_core::{parse_opto_sync_config, OptoSyncConfigError};

#[test]
fn client_api_base_must_be_url_and_non_secret() {
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
name = "api_base_url"
key = "OPTO_SYNC_API_HTTP_BASE"
kind = "url"
required = false
secret = true

[client]
enabled = true
api_base_url_binding = "api_base_url"
"#;
    assert!(matches!(parse_opto_sync_config(config), Err(OptoSyncConfigError::BindingSecretMismatch { .. })));
}
