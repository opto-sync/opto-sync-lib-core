use std::collections::BTreeMap;

use opto_sync_lib_core::{parse_opto_sync_config, resolve_opto_sync_config, OptoSyncConfigValue};

#[test]
fn non_secret_default_is_used_only_as_last_precedence_source() {
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
name = "device_id"
key = "OPTO_SYNC_DEVICE_ID"
kind = "string"
required = false
secret = false
default = "local-device"

[client]
enabled = true
device_id_binding = "device_id"
"#).expect("valid config");

    let resolved = resolve_opto_sync_config(&config, &BTreeMap::new(), &BTreeMap::new()).expect("resolved");
    assert_eq!(
        resolved.binding("device_id").expect("binding").value(),
        &OptoSyncConfigValue::String("local-device".to_owned())
    );
}
