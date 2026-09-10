use opto_sync_lib_core::{parse_opto_sync_config, resolve_opto_sync_config, OptoSyncConfigError};
use std::collections::BTreeMap;

#[test]
fn rejects_malformed_url_binding_values() {
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
name = "api_base_url"
key = "OPTO_SYNC_API_HTTP_BASE"
kind = "url"
required = true
secret = false

[client]
enabled = true
api_base_url_binding = "api_base_url"
"#).expect("valid config shape");

    let ambient = BTreeMap::from([(
        "OPTO_SYNC_API_HTTP_BASE".to_owned(),
        "not a url".to_owned(),
    )]);
    assert!(matches!(
        resolve_opto_sync_config(&config, &ambient, &BTreeMap::new()),
        Err(OptoSyncConfigError::InvalidUrl(_))
    ));
}
