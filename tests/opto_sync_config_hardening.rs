#![forbid(unsafe_code)]

use opto_sync_lib_core::opto_sync_config::{
    parse_opto_sync_config, resolve_opto_sync_config, OptoSyncConfigError, MAX_CONFIG_FILE_BYTES,
};
use std::collections::BTreeMap;

fn server_config() -> String {
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
max_batch_size = 256
conflict_policy = "last-write-wins"

[[env]]
name = "bind_addr"
key = "OPTO_SYNC_BIND"
kind = "string"
required = false
secret = false
default = "127.0.0.1:8080"

[server]
enabled = true
bind_addr_binding = "bind_addr"
"#
    .to_owned()
}

fn client_secret_config() -> &'static str {
    r#"
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
max_batch_size = 32
conflict_policy = "server-wins"

[[env]]
name = "auth_token"
key = "OPTO_SYNC_AUTH_TOKEN"
kind = "string"
required = true
secret = true

[client]
enabled = true
auth_token_binding = "auth_token"
"#
}

#[test]
fn rejects_unknown_top_level_fields() {
    let invalid = format!("{}\nunexpected = true\n", server_config());
    assert!(matches!(
        parse_opto_sync_config(&invalid),
        Err(OptoSyncConfigError::Toml(_))
    ));
}

#[test]
fn enforces_sync_policy_boundaries() {
    let valid_min = server_config()
        .replace("push_interval_ms = 3000", "push_interval_ms = 100")
        .replace("pull_interval_ms = 3000", "pull_interval_ms = 100")
        .replace("max_batch_size = 256", "max_batch_size = 1");
    parse_opto_sync_config(&valid_min).expect("minimum policy bounds must be accepted");

    let valid_max = server_config()
        .replace("push_interval_ms = 3000", "push_interval_ms = 3600000")
        .replace("pull_interval_ms = 3000", "pull_interval_ms = 3600000")
        .replace("max_batch_size = 256", "max_batch_size = 10000");
    parse_opto_sync_config(&valid_max).expect("maximum policy bounds must be accepted");

    for (needle, replacement, expected_field) in [
        ("push_interval_ms = 3000", "push_interval_ms = 99", "push_interval_ms"),
        ("pull_interval_ms = 3000", "pull_interval_ms = 3600001", "pull_interval_ms"),
        ("max_batch_size = 256", "max_batch_size = 0", "max_batch_size"),
    ] {
        let invalid = server_config().replace(needle, replacement);
        assert!(matches!(
            parse_opto_sync_config(&invalid),
            Err(OptoSyncConfigError::InvalidSyncPolicy(field)) if field == expected_field
        ));
    }
}

#[test]
fn rejects_duplicate_binding_names_and_environment_keys() {
    let duplicate_name = server_config().replace(
        "[server]",
        r#"[[env]]
name = "bind_addr"
key = "OTHER_BIND"
kind = "string"
required = false
secret = false

[server]"#,
    );
    assert!(matches!(
        parse_opto_sync_config(&duplicate_name),
        Err(OptoSyncConfigError::DuplicateBindingName(name)) if name == "bind_addr"
    ));

    let duplicate_key = server_config().replace(
        "[server]",
        r#"[[env]]
name = "other_bind"
key = "OPTO_SYNC_BIND"
kind = "string"
required = false
secret = false

[server]"#,
    );
    assert!(matches!(
        parse_opto_sync_config(&duplicate_key),
        Err(OptoSyncConfigError::DuplicateEnvironmentKey(key)) if key == "OPTO_SYNC_BIND"
    ));
}

#[test]
fn rejects_binding_identifiers_beyond_contract_limits() {
    let name_64 = format!("a{}", "b".repeat(63));
    let name_65 = format!("a{}", "b".repeat(64));
    let key_128 = format!("A{}", "B".repeat(127));
    let key_129 = format!("A{}", "B".repeat(128));

    let valid_name = server_config()
        .replace("name = \"bind_addr\"", &format!("name = \"{name_64}\""))
        .replace(
            "bind_addr_binding = \"bind_addr\"",
            &format!("bind_addr_binding = \"{name_64}\""),
        );
    parse_opto_sync_config(&valid_name).expect("64-character binding name must be accepted");

    let invalid_name = server_config().replace(
        "name = \"bind_addr\"",
        &format!("name = \"{name_65}\""),
    );
    assert!(matches!(
        parse_opto_sync_config(&invalid_name),
        Err(OptoSyncConfigError::InvalidBindingName(name)) if name == name_65
    ));

    let valid_key = server_config().replace(
        "key = \"OPTO_SYNC_BIND\"",
        &format!("key = \"{key_128}\""),
    );
    parse_opto_sync_config(&valid_key).expect("128-character environment key must be accepted");

    let invalid_key = server_config().replace(
        "key = \"OPTO_SYNC_BIND\"",
        &format!("key = \"{key_129}\""),
    );
    assert!(matches!(
        parse_opto_sync_config(&invalid_key),
        Err(OptoSyncConfigError::InvalidEnvironmentKey(key)) if key == key_129
    ));
}

#[test]
fn fails_closed_on_oversized_config_input() {
    let mut oversized = server_config();
    oversized.push_str(&" ".repeat(MAX_CONFIG_FILE_BYTES + 1));
    assert_eq!(
        parse_opto_sync_config(&oversized),
        Err(OptoSyncConfigError::TooLarge)
    );
}

#[test]
fn rejects_empty_required_values() {
    let parsed = parse_opto_sync_config(client_secret_config()).expect("valid client config");
    let ambient = BTreeMap::from([("OPTO_SYNC_AUTH_TOKEN".to_owned(), String::new())]);
    assert!(matches!(
        resolve_opto_sync_config(&parsed, &ambient, &BTreeMap::new()),
        Err(OptoSyncConfigError::MissingRequiredBinding(name)) if name == "auth_token"
    ));
}

#[test]
fn rejects_secret_argv_override_even_when_environment_has_a_value() {
    let parsed = parse_opto_sync_config(client_secret_config()).expect("valid client config");
    let ambient = BTreeMap::from([(
        "OPTO_SYNC_AUTH_TOKEN".to_owned(),
        "environment-secret".to_owned(),
    )]);
    let argv = BTreeMap::from([(
        "OPTO_SYNC_AUTH_TOKEN".to_owned(),
        "argv-secret".to_owned(),
    )]);
    assert!(matches!(
        resolve_opto_sync_config(&parsed, &ambient, &argv),
        Err(OptoSyncConfigError::SecretFromArgv(name)) if name == "auth_token"
    ));
}

#[test]
fn resolved_debug_output_redacts_secret_values() {
    let parsed = parse_opto_sync_config(client_secret_config()).expect("valid client config");
    let ambient = BTreeMap::from([(
        "OPTO_SYNC_AUTH_TOKEN".to_owned(),
        "never-print-this-secret".to_owned(),
    )]);
    let resolved = resolve_opto_sync_config(&parsed, &ambient, &BTreeMap::new())
        .expect("required secret supplied from environment");
    let rendered = format!("{resolved:?}");
    assert!(rendered.contains("[REDACTED]"));
    assert!(!rendered.contains("never-print-this-secret"));
}

#[test]
fn rejects_non_finite_double_values() {
    let config = server_config().replace(
        "[server]",
        r#"[[env]]
name = "ratio"
key = "OPTO_SYNC_RATIO"
kind = "double"
required = true
secret = false

[server]"#,
    );
    let parsed = parse_opto_sync_config(&config).expect("valid config");

    for value in ["NaN", "inf", "-inf"] {
        let ambient = BTreeMap::from([("OPTO_SYNC_RATIO".to_owned(), value.to_owned())]);
        assert!(matches!(
            resolve_opto_sync_config(&parsed, &ambient, &BTreeMap::new()),
            Err(OptoSyncConfigError::InvalidDouble(name)) if name == "ratio"
        ));
    }
}

#[test]
fn rejects_malformed_url_values() {
    let config = server_config().replace(
        "[server]",
        r#"[[env]]
name = "database_url"
key = "DATABASE_URL"
kind = "url"
required = true
secret = true

[server]
database_url_binding = "database_url"
"#,
    );
    let parsed = parse_opto_sync_config(&config).expect("valid config");
    let ambient = BTreeMap::from([("DATABASE_URL".to_owned(), "not a url".to_owned())]);
    assert!(matches!(
        resolve_opto_sync_config(&parsed, &ambient, &BTreeMap::new()),
        Err(OptoSyncConfigError::InvalidUrl(name)) if name == "database_url"
    ));
}
