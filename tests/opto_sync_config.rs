use std::collections::BTreeMap;

use opto_sync_lib_core::{
    parse_opto_sync_config, resolve_opto_sync_config, OptoSyncConfigError,
    OptoSyncConfigValue, OptoSyncMode,
};

const HYBRID: &str = r#"
version = 1
mode = "hybrid"
strict = true

[flags2env]
contract = ".cli-flags.toml"
require_audit = true
precedence = "argv-over-env"

[sync]
push_interval_ms = 3000
pull_interval_ms = 3000
max_batch_size = 64
conflict_policy = "last-write-wins"

[[env]]
name = "api_base_url"
key = "OPTO_SYNC_API_HTTP_BASE"
kind = "url"
required = false
secret = false
default = "http://127.0.0.1:8080/"

[[env]]
name = "auth_token"
key = "SHARED_AUTH_CUSTOMER_TOKEN"
kind = "string"
required = false
secret = true

[[env]]
name = "bind_addr"
key = "OPTO_SYNC_API_BIND"
kind = "string"
required = false
secret = false
default = "127.0.0.1:8080"

[client]
enabled = true
api_base_url_binding = "api_base_url"
auth_token_binding = "auth_token"

[server]
enabled = true
bind_addr_binding = "bind_addr"
"#;

#[test]
fn resolves_argv_over_environment_over_default() {
    let config = parse_opto_sync_config(HYBRID).expect("valid hybrid config");
    assert_eq!(config.mode, OptoSyncMode::Hybrid);

    let ambient = BTreeMap::from([
        ("OPTO_SYNC_API_HTTP_BASE".to_owned(), "https://env.example/".to_owned()),
        ("SHARED_AUTH_CUSTOMER_TOKEN".to_owned(), "secret-from-env".to_owned()),
    ]);
    let argv = BTreeMap::from([(
        "OPTO_SYNC_API_HTTP_BASE".to_owned(),
        "https://argv.example/".to_owned(),
    )]);

    let resolved = resolve_opto_sync_config(&config, &ambient, &argv).expect("resolved");
    assert_eq!(
        resolved.binding("api_base_url").expect("api binding").value(),
        &OptoSyncConfigValue::Url("https://argv.example/".to_owned())
    );
    assert!(resolved.binding("auth_token").expect("token").is_secret());
}

#[test]
fn rejects_secret_from_argv() {
    let config = parse_opto_sync_config(HYBRID).expect("valid hybrid config");
    let argv = BTreeMap::from([(
        "SHARED_AUTH_CUSTOMER_TOKEN".to_owned(),
        "must-not-arrive-through-argv".to_owned(),
    )]);
    let error = resolve_opto_sync_config(&config, &BTreeMap::new(), &argv).unwrap_err();
    assert!(matches!(error, OptoSyncConfigError::SecretFromArgv(_)));
}

#[test]
fn rejects_role_mismatch_and_unsafe_contract() {
    let role_error = parse_opto_sync_config(&HYBRID.replace("mode = \"hybrid\"", "mode = \"client\""))
        .unwrap_err();
    assert!(matches!(role_error, OptoSyncConfigError::ModeRoleMismatch(_)));

    let contract_error = parse_opto_sync_config(&HYBRID.replace(
        "contract = \".cli-flags.toml\"",
        "contract = \"../other.toml\"",
    ))
    .unwrap_err();
    assert_eq!(contract_error, OptoSyncConfigError::UnsafeFlagsContract);
}
