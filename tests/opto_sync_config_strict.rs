use opto_sync_lib_core::{parse_opto_sync_config, OptoSyncConfigError};

#[test]
fn strict_and_flags_audit_cannot_be_disabled() {
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

    assert_eq!(
        parse_opto_sync_config(&base.replace("strict = true", "strict = false")).unwrap_err(),
        OptoSyncConfigError::StrictModeRequired
    );
    assert_eq!(
        parse_opto_sync_config(&base.replace("require_audit = true", "require_audit = false")).unwrap_err(),
        OptoSyncConfigError::FlagsAuditRequired
    );
}
