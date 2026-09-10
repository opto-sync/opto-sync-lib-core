use std::collections::BTreeMap;

use opto_sync_lib_core::{parse_opto_sync_config, resolve_opto_sync_config};

#[test]
fn secret_argv_rejection_does_not_echo_rejected_value() {
    let config = parse_opto_sync_config(r#"
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

[client]
enabled = true
auth_token_binding = "auth_token"
"#).expect("valid config");
    let secret = "must-not-appear";
    let argv = BTreeMap::from([("SHARED_AUTH_CUSTOMER_TOKEN".to_owned(), secret.to_owned())]);
    let error = resolve_opto_sync_config(&config, &BTreeMap::new(), &argv).unwrap_err();
    assert!(!error.to_string().contains(secret));
}
