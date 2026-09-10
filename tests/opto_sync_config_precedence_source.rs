use std::collections::BTreeMap;

use opto_sync_lib_core::{parse_opto_sync_config, resolve_opto_sync_config};

#[test]
fn argv_and_environment_resolution_do_not_mutate_process_environment() {
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

[client]
enabled = true
"#).expect("valid config");
    resolve_opto_sync_config(&config, &BTreeMap::new(), &BTreeMap::new()).expect("resolved");
}
