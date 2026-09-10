use opto_sync_lib_core::{parse_opto_sync_config, OptoSyncConfigError};

#[test]
fn invalid_secret_default_error_does_not_echo_the_value() {
    let secret = "do-not-echo-this-secret";
    let config = format!(r#"
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
secret = true
default = "{secret}"

[client]
enabled = true
auth_token_binding = "auth_token"
"#);
    let error = parse_opto_sync_config(&config).unwrap_err();
    assert!(matches!(error, OptoSyncConfigError::SecretDefault(_)));
    assert!(!error.to_string().contains(secret));
}
