use opto_sync_lib_core::{parse_opto_sync_config, OptoSyncConfigError};

#[test]
fn rejects_role_binding_kind_and_secret_policy_mismatches() {
    let wrong_kind = r#"
version = 1
mode = "server"
strict = true

[flags2env]
contract = ".cli-flags.toml"
require_audit = true
precedence = "argv-over-env"

[sync]
push_interval_ms = 1000
pull_interval_ms = 1000
max_batch_size = 8
conflict_policy = "server-wins"

[[env]]
name = "database_url"
key = "DATABASE_URL"
kind = "string"
required = false
secret = true

[server]
enabled = true
database_url_binding = "database_url"
"#;
    assert!(matches!(
        parse_opto_sync_config(wrong_kind),
        Err(OptoSyncConfigError::BindingKindMismatch { .. })
    ));

    let wrong_secret = wrong_kind
        .replace("kind = \"string\"", "kind = \"url\"")
        .replace("secret = true", "secret = false");
    assert!(matches!(
        parse_opto_sync_config(&wrong_secret),
        Err(OptoSyncConfigError::BindingSecretMismatch { .. })
    ));
}
