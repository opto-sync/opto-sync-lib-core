use std::collections::BTreeMap;

use opto_sync_lib_core::{parse_opto_sync_config, resolve_opto_sync_config, OptoSyncConfigError};

#[test]
fn invalid_boolean_coercion_is_rejected() {
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
name = "feature"
key = "OPTO_SYNC_FEATURE"
kind = "bool"
required = true
secret = false

[client]
enabled = true
"#).expect("valid config");
    let ambient = BTreeMap::from([("OPTO_SYNC_FEATURE".to_owned(), "maybe".to_owned())]);
    assert!(matches!(
        resolve_opto_sync_config(&config, &ambient, &BTreeMap::new()),
        Err(OptoSyncConfigError::InvalidBoolean(_))
    ));
}
