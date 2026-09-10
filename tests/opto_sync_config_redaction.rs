use std::collections::BTreeMap;

use opto_sync_lib_core::{parse_opto_sync_config, resolve_opto_sync_config};

#[test]
fn secret_debug_output_is_redacted() {
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
max_batch_size = 1
conflict_policy = "manual"

[[env]]
name = "auth_token"
key = "SHARED_AUTH_CUSTOMER_TOKEN"
kind = "string"
required = true
secret = true

[client]
enabled = true
auth_token_binding = "auth_token"
"#).expect("valid config");

    let ambient = BTreeMap::from([(
        "SHARED_AUTH_CUSTOMER_TOKEN".to_owned(),
        "super-secret-value".to_owned(),
    )]);
    let resolved = resolve_opto_sync_config(&config, &ambient, &BTreeMap::new()).expect("resolved");
    let rendered = format!("{:?}", resolved.binding("auth_token").expect("token"));
    assert!(rendered.contains("[REDACTED]"));
    assert!(!rendered.contains("super-secret-value"));
}
