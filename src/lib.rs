#![forbid(unsafe_code)]

pub mod config;
pub mod connection;
pub mod error;
pub mod flavor;
pub mod opto_sync_config;
pub mod schema;

pub use config::CoreConfig;
pub use connection::CorePool;
pub use error::CoreError;
pub use flavor::DatabaseFlavor;
pub use opto_sync_config::{
    load_opto_sync_config, load_repo_root as load_opto_sync_repo_root,
    merge_environment as merge_opto_sync_environment, parse_opto_sync_config,
    resolve_opto_sync_config, validate_opto_sync_config, ConfigValue as OptoSyncConfigValue,
    EnvBinding as OptoSyncEnvBinding, EnvKind as OptoSyncEnvKind, OptoSyncConfig,
    OptoSyncConfigError, OptoSyncMode, ResolvedOptoSyncConfig, SyncPolicy,
    OPTO_SYNC_CONFIG_CONTRACT_REPOSITORY, OPTO_SYNC_CONFIG_CONTRACT_REVISION,
    OPTO_SYNC_CONFIG_FILENAME, OPTO_SYNC_CONFIG_TJSV_REVISION,
};
pub use schema::SCHEMA_REVISION;
