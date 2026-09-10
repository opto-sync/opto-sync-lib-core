use opto_sync_lib_core::{parse_opto_sync_config, OptoSyncConfigError};

#[test]
fn sync_interval_bounds_are_enforced() {
    let base = r#"
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
"#;
    let too_fast = base.replace("push_interval_ms = 1000", "push_interval_ms = 99");
    assert_eq!(
        parse_opto_sync_config(&too_fast).unwrap_err(),
        OptoSyncConfigError::InvalidSyncPolicy("push_interval_ms")
    );
    let too_slow = base.replace("pull_interval_ms = 1000", "pull_interval_ms = 3600001");
    assert_eq!(
        parse_opto_sync_config(&too_slow).unwrap_err(),
        OptoSyncConfigError::InvalidSyncPolicy("pull_interval_ms")
    );
}
