use opto_sync_lib_core::{parse_opto_sync_config, OptoSyncConfigError};

#[test]
fn rejects_duplicate_logical_names_and_environment_keys() {
    let duplicate_name = r#"
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
name = "value"
key = "VALUE_A"
kind = "string"
required = false
secret = false

[[env]]
name = "value"
key = "VALUE_B"
kind = "string"
required = false
secret = false

[client]
enabled = true
"#;
    assert!(matches!(
        parse_opto_sync_config(duplicate_name),
        Err(OptoSyncConfigError::DuplicateBindingName(_))
    ));

    let duplicate_key = duplicate_name.replace("name = \"value\"\nkey = \"VALUE_B\"", "name = \"other\"\nkey = \"VALUE_A\"");
    assert!(matches!(
        parse_opto_sync_config(&duplicate_key),
        Err(OptoSyncConfigError::DuplicateEnvironmentKey(_))
    ));
}
