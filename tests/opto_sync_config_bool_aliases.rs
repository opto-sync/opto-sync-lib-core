use std::collections::BTreeMap;

use opto_sync_lib_core::{parse_opto_sync_config, resolve_opto_sync_config, OptoSyncConfigValue};

#[test]
fn canonical_boolean_values_are_coerced() {
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
    for (raw, expected) in [("true", true), ("1", true), ("false", false), ("0", false)] {
        let ambient = BTreeMap::from([("OPTO_SYNC_FEATURE".to_owned(), raw.to_owned())]);
        let resolved = resolve_opto_sync_config(&config, &ambient, &BTreeMap::new()).expect("resolved");
        assert_eq!(resolved.binding("feature").expect("feature").value(), &OptoSyncConfigValue::Bool(expected));
    }
}
