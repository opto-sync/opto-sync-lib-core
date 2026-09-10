use std::collections::BTreeMap;

use opto_sync_lib_core::{parse_opto_sync_config, resolve_opto_sync_config, OptoSyncConfigValue};

#[test]
fn ambient_environment_beats_domain_default_when_argv_is_absent() {
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
required = false
secret = false
default = "http://127.0.0.1:8080/"

[client]
enabled = true
api_base_url_binding = "api_base_url"
"#).expect("valid config");

    let ambient = BTreeMap::from([(
        "OPTO_SYNC_API_HTTP_BASE".to_owned(),
        "https://environment.example/".to_owned(),
    )]);
    let resolved = resolve_opto_sync_config(&config, &ambient, &BTreeMap::new()).expect("resolved");
    assert_eq!(
        resolved.binding("api_base_url").expect("binding").value(),
        &OptoSyncConfigValue::Url("https://environment.example/".to_owned())
    );
}
