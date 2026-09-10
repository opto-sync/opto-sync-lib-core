use opto_sync_lib_core::{parse_opto_sync_config, OptoSyncConfigError};

fn base() -> String {
    r#"
version = 1
mode = "server"
strict = true

[flags2env]
contract = ".cli-flags.toml"
require_audit = true
precedence = "argv-over-env"

[sync]
push_interval_ms = 3000
pull_interval_ms = 3000
max_batch_size = 64
conflict_policy = "server-wins"

[[env]]
name = "database_url"
key = "DATABASE_URL"
kind = "url"
required = false
secret = true

[server]
enabled = true
database_url_binding = "database_url"
"#
    .to_owned()
}

#[test]
fn rejects_plaintext_secret_default() {
    let config = base().replace(
        "secret = true",
        "secret = true\ndefault = \"postgres://plaintext.example/db\"",
    );
    assert!(matches!(
        parse_opto_sync_config(&config),
        Err(OptoSyncConfigError::SecretDefault(_))
    ));
}

#[test]
fn rejects_unknown_fields_and_unbounded_sync_values() {
    let unknown = base().replace("strict = true", "strict = true\nunknown = true");
    assert!(matches!(parse_opto_sync_config(&unknown), Err(OptoSyncConfigError::Toml(_))));

    let unbounded = base().replace("max_batch_size = 64", "max_batch_size = 10001");
    assert_eq!(
        parse_opto_sync_config(&unbounded).unwrap_err(),
        OptoSyncConfigError::InvalidSyncPolicy("max_batch_size")
    );
}
