use opto_sync_lib_core::{parse_opto_sync_config, OptoSyncConfigError};

#[test]
fn unsupported_versions_fail_closed() {
    let config = r#"
version = 2
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
    assert_eq!(
        parse_opto_sync_config(config).unwrap_err(),
        OptoSyncConfigError::UnsupportedVersion(2)
    );
}
