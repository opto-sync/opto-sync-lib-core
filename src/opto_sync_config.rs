#![forbid(unsafe_code)]

use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt, fs,
    path::Path,
};
use thiserror::Error;
use url::Url;

pub const OPTO_SYNC_CONFIG_FILENAME: &str = ".opto-sync.toml";
pub const OPTO_SYNC_CONFIG_CONTRACT_REPOSITORY: &str = "opto-sync/opto-sync-interfaces";
pub const OPTO_SYNC_CONFIG_CONTRACT_REVISION: &str =
    "91390f0e1a76ae809f84f51c91e3f4cd4160a07c";
pub const OPTO_SYNC_CONFIG_TJSV_REVISION: &str =
    "4a5d049218adc2740d4cf78f612caf7f38f6f64c";
pub const MAX_CONFIG_FILE_BYTES: usize = 256 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OptoSyncMode {
    Client,
    Server,
    Hybrid,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EnvKind {
    String,
    Bool,
    Integer,
    Double,
    Json,
    Url,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub enum Flags2EnvPrecedence {
    #[serde(rename = "argv-over-env")]
    ArgvOverEnv,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub enum ConflictPolicy {
    #[serde(rename = "server-wins")]
    ServerWins,
    #[serde(rename = "client-wins")]
    ClientWins,
    #[serde(rename = "last-write-wins")]
    LastWriteWins,
    #[serde(rename = "manual")]
    Manual,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Flags2EnvConfig {
    pub contract: String,
    pub require_audit: bool,
    pub precedence: Flags2EnvPrecedence,
}

#[derive(Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EnvBinding {
    pub name: String,
    pub key: String,
    pub kind: EnvKind,
    pub required: bool,
    pub secret: bool,
    #[serde(rename = "default")]
    pub default_value: Option<String>,
    pub description: Option<String>,
}

impl fmt::Debug for EnvBinding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = formatter.debug_struct("EnvBinding");
        debug.field("name", &self.name);
        debug.field("key", &self.key);
        debug.field("kind", &self.kind);
        debug.field("required", &self.required);
        debug.field("secret", &self.secret);
        if self.secret && self.default_value.is_some() {
            debug.field("default_value", &"[REDACTED]");
        } else {
            debug.field("default_value", &self.default_value);
        }
        debug.field("description", &self.description);
        debug.finish()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SyncPolicy {
    pub push_interval_ms: u32,
    pub pull_interval_ms: u32,
    pub max_batch_size: u32,
    pub conflict_policy: ConflictPolicy,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OptoSyncClientConfig {
    pub enabled: bool,
    pub api_base_url_binding: Option<String>,
    pub device_id_binding: Option<String>,
    pub auth_token_binding: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OptoSyncServerConfig {
    pub enabled: bool,
    pub bind_addr_binding: Option<String>,
    pub database_url_binding: Option<String>,
    pub nats_url_binding: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OptoSyncConfig {
    pub version: u32,
    pub mode: OptoSyncMode,
    pub strict: bool,
    pub flags2env: Flags2EnvConfig,
    pub sync: SyncPolicy,
    pub env: Vec<EnvBinding>,
    pub client: Option<OptoSyncClientConfig>,
    pub server: Option<OptoSyncServerConfig>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConfigValue {
    String(String),
    Bool(bool),
    Integer(i64),
    Double(f64),
    Json(JsonValue),
    Url(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueSource {
    Argv,
    Environment,
    Default,
}

#[derive(Clone, PartialEq)]
pub struct ResolvedBinding {
    env_key: String,
    value: ConfigValue,
    source: ValueSource,
    secret: bool,
}

impl ResolvedBinding {
    pub fn env_key(&self) -> &str {
        &self.env_key
    }

    pub fn value(&self) -> &ConfigValue {
        &self.value
    }

    pub fn source(&self) -> ValueSource {
        self.source
    }

    pub fn is_secret(&self) -> bool {
        self.secret
    }
}

impl fmt::Debug for ResolvedBinding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = formatter.debug_struct("ResolvedBinding");
        debug.field("env_key", &self.env_key);
        debug.field("source", &self.source);
        debug.field("secret", &self.secret);
        if self.secret {
            debug.field("value", &"[REDACTED]");
        } else {
            debug.field("value", &self.value);
        }
        debug.finish()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedOptoSyncConfig {
    mode: OptoSyncMode,
    sync: SyncPolicy,
    values: BTreeMap<String, ResolvedBinding>,
}

impl ResolvedOptoSyncConfig {
    pub fn mode(&self) -> OptoSyncMode {
        self.mode
    }

    pub fn sync_policy(&self) -> SyncPolicy {
        self.sync
    }

    pub fn binding(&self, name: &str) -> Option<&ResolvedBinding> {
        self.values.get(name)
    }

    pub fn bindings(&self) -> &BTreeMap<String, ResolvedBinding> {
        &self.values
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum OptoSyncConfigError {
    #[error("invalid .opto-sync.toml: {0}")]
    Toml(String),
    #[error("failed to read .opto-sync.toml: {0}")]
    Io(String),
    #[error(".opto-sync.toml exceeds 256 KiB")]
    TooLarge,
    #[error("unsupported Opto Sync config version {0}")]
    UnsupportedVersion(u32),
    #[error("Opto Sync strict mode must be enabled")]
    StrictModeRequired,
    #[error("flags-2-env audit must be enabled")]
    FlagsAuditRequired,
    #[error("flags-2-env contract must be repository-root .cli-flags.toml")]
    UnsafeFlagsContract,
    #[error("invalid sync policy: {0}")]
    InvalidSyncPolicy(&'static str),
    #[error("invalid Opto Sync binding name: {0}")]
    InvalidBindingName(String),
    #[error("invalid Opto Sync environment key: {0}")]
    InvalidEnvironmentKey(String),
    #[error("duplicate Opto Sync binding name: {0}")]
    DuplicateBindingName(String),
    #[error("duplicate Opto Sync environment key: {0}")]
    DuplicateEnvironmentKey(String),
    #[error("secret binding may not declare a plaintext default: {0}")]
    SecretDefault(String),
    #[error("secret binding may not be supplied through argv: {0}")]
    SecretFromArgv(String),
    #[error("required Opto Sync binding is unresolved: {0}")]
    MissingRequiredBinding(String),
    #[error("Opto Sync role is inconsistent with mode: {0}")]
    ModeRoleMismatch(&'static str),
    #[error("unknown Opto Sync binding reference {binding} at {field}")]
    UnknownBindingReference { field: &'static str, binding: String },
    #[error("Opto Sync binding {binding} at {field} must use kind {expected:?}")]
    BindingKindMismatch {
        field: &'static str,
        binding: String,
        expected: EnvKind,
    },
    #[error("Opto Sync binding {binding} at {field} has invalid secret policy")]
    BindingSecretMismatch { field: &'static str, binding: String },
    #[error("invalid boolean value for Opto Sync binding: {0}")]
    InvalidBoolean(String),
    #[error("invalid integer value for Opto Sync binding: {0}")]
    InvalidInteger(String),
    #[error("invalid double value for Opto Sync binding: {0}")]
    InvalidDouble(String),
    #[error("invalid JSON value for Opto Sync binding: {0}")]
    InvalidJson(String),
    #[error("invalid URL value for Opto Sync binding: {0}")]
    InvalidUrl(String),
}

pub fn parse_opto_sync_config(input: &str) -> Result<OptoSyncConfig, OptoSyncConfigError> {
    if input.len() > MAX_CONFIG_FILE_BYTES {
        return Err(OptoSyncConfigError::TooLarge);
    }
    let config = toml::from_str::<OptoSyncConfig>(input)
        .map_err(|error| OptoSyncConfigError::Toml(error.to_string()))?;
    validate_opto_sync_config(&config)?;
    Ok(config)
}

pub fn load_opto_sync_config(path: impl AsRef<Path>) -> Result<OptoSyncConfig, OptoSyncConfigError> {
    let bytes = fs::read(path.as_ref()).map_err(|error| OptoSyncConfigError::Io(error.to_string()))?;
    if bytes.len() > MAX_CONFIG_FILE_BYTES {
        return Err(OptoSyncConfigError::TooLarge);
    }
    let text = std::str::from_utf8(&bytes)
        .map_err(|error| OptoSyncConfigError::Toml(error.to_string()))?;
    parse_opto_sync_config(text)
}

pub fn load_repo_root(repo_root: impl AsRef<Path>) -> Result<OptoSyncConfig, OptoSyncConfigError> {
    load_opto_sync_config(repo_root.as_ref().join(OPTO_SYNC_CONFIG_FILENAME))
}

pub fn validate_opto_sync_config(config: &OptoSyncConfig) -> Result<(), OptoSyncConfigError> {
    if config.version != 1 {
        return Err(OptoSyncConfigError::UnsupportedVersion(config.version));
    }
    if !config.strict {
        return Err(OptoSyncConfigError::StrictModeRequired);
    }
    if !config.flags2env.require_audit {
        return Err(OptoSyncConfigError::FlagsAuditRequired);
    }
    if config.flags2env.contract != ".cli-flags.toml" {
        return Err(OptoSyncConfigError::UnsafeFlagsContract);
    }
    validate_sync_policy(config.sync)?;
    validate_mode(config)?;

    let mut names = BTreeSet::new();
    let mut keys = BTreeSet::new();
    let mut by_name = BTreeMap::new();
    for binding in &config.env {
        if !is_binding_name(&binding.name) {
            return Err(OptoSyncConfigError::InvalidBindingName(binding.name.clone()));
        }
        if !is_environment_key(&binding.key) {
            return Err(OptoSyncConfigError::InvalidEnvironmentKey(binding.key.clone()));
        }
        if !names.insert(binding.name.as_str()) {
            return Err(OptoSyncConfigError::DuplicateBindingName(binding.name.clone()));
        }
        if !keys.insert(binding.key.as_str()) {
            return Err(OptoSyncConfigError::DuplicateEnvironmentKey(binding.key.clone()));
        }
        if binding.secret && binding.default_value.is_some() {
            return Err(OptoSyncConfigError::SecretDefault(binding.name.clone()));
        }
        by_name.insert(binding.name.as_str(), binding);
    }

    if let Some(client) = &config.client {
        validate_reference(
            &by_name,
            "client.api_base_url_binding",
            client.api_base_url_binding.as_deref(),
            EnvKind::Url,
            false,
        )?;
        validate_reference(
            &by_name,
            "client.device_id_binding",
            client.device_id_binding.as_deref(),
            EnvKind::String,
            false,
        )?;
        validate_reference(
            &by_name,
            "client.auth_token_binding",
            client.auth_token_binding.as_deref(),
            EnvKind::String,
            true,
        )?;
    }

    if let Some(server) = &config.server {
        validate_reference(
            &by_name,
            "server.bind_addr_binding",
            server.bind_addr_binding.as_deref(),
            EnvKind::String,
            false,
        )?;
        validate_reference(
            &by_name,
            "server.database_url_binding",
            server.database_url_binding.as_deref(),
            EnvKind::Url,
            true,
        )?;
        validate_reference(
            &by_name,
            "server.nats_url_binding",
            server.nats_url_binding.as_deref(),
            EnvKind::Url,
            true,
        )?;
    }

    Ok(())
}

pub fn merge_environment(
    ambient: &BTreeMap<String, String>,
    argv_overrides: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let mut merged = ambient.clone();
    merged.extend(argv_overrides.iter().map(|(key, value)| (key.clone(), value.clone())));
    merged
}

pub fn resolve_opto_sync_config(
    config: &OptoSyncConfig,
    ambient: &BTreeMap<String, String>,
    argv_overrides: &BTreeMap<String, String>,
) -> Result<ResolvedOptoSyncConfig, OptoSyncConfigError> {
    validate_opto_sync_config(config)?;
    let mut values = BTreeMap::new();

    for binding in &config.env {
        if binding.secret && argv_overrides.contains_key(&binding.key) {
            return Err(OptoSyncConfigError::SecretFromArgv(binding.name.clone()));
        }

        let resolved = if let Some(value) = argv_overrides.get(&binding.key) {
            Some((value.as_str(), ValueSource::Argv))
        } else if let Some(value) = ambient.get(&binding.key) {
            Some((value.as_str(), ValueSource::Environment))
        } else {
            binding
                .default_value
                .as_deref()
                .map(|value| (value, ValueSource::Default))
        };

        let Some((raw_value, source)) = resolved else {
            if binding.required {
                return Err(OptoSyncConfigError::MissingRequiredBinding(binding.name.clone()));
            }
            continue;
        };

        if binding.required && raw_value.is_empty() {
            return Err(OptoSyncConfigError::MissingRequiredBinding(binding.name.clone()));
        }

        values.insert(
            binding.name.clone(),
            ResolvedBinding {
                env_key: binding.key.clone(),
                value: coerce_value(binding, raw_value)?,
                source,
                secret: binding.secret,
            },
        );
    }

    Ok(ResolvedOptoSyncConfig {
        mode: config.mode,
        sync: config.sync,
        values,
    })
}

fn validate_sync_policy(sync: SyncPolicy) -> Result<(), OptoSyncConfigError> {
    if !(100..=3_600_000).contains(&sync.push_interval_ms) {
        return Err(OptoSyncConfigError::InvalidSyncPolicy("push_interval_ms"));
    }
    if !(100..=3_600_000).contains(&sync.pull_interval_ms) {
        return Err(OptoSyncConfigError::InvalidSyncPolicy("pull_interval_ms"));
    }
    if !(1..=10_000).contains(&sync.max_batch_size) {
        return Err(OptoSyncConfigError::InvalidSyncPolicy("max_batch_size"));
    }
    Ok(())
}

fn validate_mode(config: &OptoSyncConfig) -> Result<(), OptoSyncConfigError> {
    let client_enabled = config.client.as_ref().is_some_and(|role| role.enabled);
    let server_enabled = config.server.as_ref().is_some_and(|role| role.enabled);
    match config.mode {
        OptoSyncMode::Client if client_enabled && !server_enabled => Ok(()),
        OptoSyncMode::Server if server_enabled && !client_enabled => Ok(()),
        OptoSyncMode::Hybrid if client_enabled && server_enabled => Ok(()),
        OptoSyncMode::Client => Err(OptoSyncConfigError::ModeRoleMismatch(
            "client mode requires only client.enabled = true",
        )),
        OptoSyncMode::Server => Err(OptoSyncConfigError::ModeRoleMismatch(
            "server mode requires only server.enabled = true",
        )),
        OptoSyncMode::Hybrid => Err(OptoSyncConfigError::ModeRoleMismatch(
            "hybrid mode requires client.enabled = true and server.enabled = true",
        )),
    }
}

fn validate_reference(
    by_name: &BTreeMap<&str, &EnvBinding>,
    field: &'static str,
    reference: Option<&str>,
    expected_kind: EnvKind,
    expected_secret: bool,
) -> Result<(), OptoSyncConfigError> {
    let Some(reference) = reference else {
        return Ok(());
    };
    let binding = by_name
        .get(reference)
        .copied()
        .ok_or_else(|| OptoSyncConfigError::UnknownBindingReference {
            field,
            binding: reference.to_owned(),
        })?;
    if binding.kind != expected_kind {
        return Err(OptoSyncConfigError::BindingKindMismatch {
            field,
            binding: reference.to_owned(),
            expected: expected_kind,
        });
    }
    if binding.secret != expected_secret {
        return Err(OptoSyncConfigError::BindingSecretMismatch {
            field,
            binding: reference.to_owned(),
        });
    }
    Ok(())
}

fn is_binding_name(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some('a'..='z'))
        && chars.all(|character| character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_')
        && value.len() <= 64
}

fn is_environment_key(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some('A'..='Z') | Some('_'))
        && chars.all(|character| character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_')
        && value.len() <= 128
}

fn coerce_value(binding: &EnvBinding, raw: &str) -> Result<ConfigValue, OptoSyncConfigError> {
    match binding.kind {
        EnvKind::String => Ok(ConfigValue::String(raw.to_owned())),
        EnvKind::Bool => match raw {
            "true" => Ok(ConfigValue::Bool(true)),
            "false" => Ok(ConfigValue::Bool(false)),
            _ => Err(OptoSyncConfigError::InvalidBoolean(binding.name.clone())),
        },
        EnvKind::Integer => raw
            .parse::<i64>()
            .map(ConfigValue::Integer)
            .map_err(|_| OptoSyncConfigError::InvalidInteger(binding.name.clone())),
        EnvKind::Double => raw
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
            .map(ConfigValue::Double)
            .ok_or_else(|| OptoSyncConfigError::InvalidDouble(binding.name.clone())),
        EnvKind::Json => serde_json::from_str::<JsonValue>(raw)
            .map(ConfigValue::Json)
            .map_err(|_| OptoSyncConfigError::InvalidJson(binding.name.clone())),
        EnvKind::Url => Url::parse(raw)
            .map(|url| ConfigValue::Url(url.to_string()))
            .map_err(|_| OptoSyncConfigError::InvalidUrl(binding.name.clone())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn server_config() -> &'static str {
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
    }

    #[test]
    fn parses_valid_server_config() {
        let parsed = parse_opto_sync_config(server_config()).expect("valid config");
        assert_eq!(parsed.mode, OptoSyncMode::Server);
        assert_eq!(parsed.sync.max_batch_size, 256);
    }

    #[test]
    fn argv_precedes_environment_and_default() {
        let parsed = parse_opto_sync_config(server_config()).expect("valid config");
        let ambient = BTreeMap::from([("OPTO_SYNC_BIND".to_owned(), "127.0.0.1:8081".to_owned())]);
        let argv = BTreeMap::from([("OPTO_SYNC_BIND".to_owned(), "127.0.0.1:8082".to_owned())]);
        let resolved = resolve_opto_sync_config(&parsed, &ambient, &argv).expect("resolved");
        let binding = resolved.binding("bind_addr").expect("binding");
        assert_eq!(binding.source(), ValueSource::Argv);
        assert_eq!(binding.value(), &ConfigValue::String("127.0.0.1:8082".to_owned()));
    }

    #[test]
    fn rejects_secret_default() {
        let invalid = server_config().replace(
            "[server]",
            "[[env]]\nname = \"database_url\"\nkey = \"DATABASE_URL\"\nkind = \"url\"\nrequired = true\nsecret = true\ndefault = \"postgres://plaintext\"\n\n[server]",
        );
        assert!(matches!(
            parse_opto_sync_config(&invalid),
            Err(OptoSyncConfigError::SecretDefault(name)) if name == "database_url"
        ));
    }

    #[test]
    fn rejects_secret_from_argv() {
        let input = r#"
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
conflict_policy = "client-wins"
[[env]]
name = "auth_token"
key = "OPTO_SYNC_AUTH_TOKEN"
kind = "string"
required = false
secret = true
[client]
enabled = true
auth_token_binding = "auth_token"
"#;
        let parsed = parse_opto_sync_config(input).expect("valid config");
        let argv = BTreeMap::from([("OPTO_SYNC_AUTH_TOKEN".to_owned(), "do-not-log".to_owned())]);
        assert!(matches!(
            resolve_opto_sync_config(&parsed, &BTreeMap::new(), &argv),
            Err(OptoSyncConfigError::SecretFromArgv(name)) if name == "auth_token"
        ));
    }

    #[test]
    fn requires_role_projection_to_match_mode() {
        let invalid = server_config().replace("mode = \"server\"", "mode = \"hybrid\"");
        assert!(matches!(
            parse_opto_sync_config(&invalid),
            Err(OptoSyncConfigError::ModeRoleMismatch(_))
        ));
    }

    #[test]
    fn debug_redacts_secret_values() {
        let binding = ResolvedBinding {
            env_key: "TOKEN".to_owned(),
            value: ConfigValue::String("super-secret".to_owned()),
            source: ValueSource::Environment,
            secret: true,
        };
        let rendered = format!("{binding:?}");
        assert!(rendered.contains("[REDACTED]"));
        assert!(!rendered.contains("super-secret"));
    }
}
