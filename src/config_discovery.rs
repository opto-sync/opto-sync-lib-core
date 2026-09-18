#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::opto_sync_config::{load_opto_sync_config, OptoSyncConfig, OptoSyncConfigError};

/// Preferred runtime filename requested by the fleet rollout.
pub const OPTO_SYNC_CONFIG_FILENAME: &str = ".opto-cfg.toml";
/// Compatibility name already deployed across the Opto Sync fleet.
pub const OPTO_SYNC_CONFIG_LEGACY_FILENAME: &str = ".opto-sync.toml";

const CONFIG_FILENAMES: [&str; 2] = [
    OPTO_SYNC_CONFIG_FILENAME,
    OPTO_SYNC_CONFIG_LEGACY_FILENAME,
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoveredOptoSyncConfig {
    pub path: PathBuf,
    pub at_repository_root: bool,
}

#[derive(Debug, Error)]
pub enum OptoSyncConfigDiscoveryError {
    #[error("failed to resolve the current working directory: {0}")]
    CurrentDirectory(#[source] std::io::Error),
    #[error("no .opto-cfg.toml or .opto-sync.toml found from {start} to the filesystem root")]
    NotFound { start: PathBuf },
    #[error("both .opto-cfg.toml and .opto-sync.toml exist in {directory}; keep exactly one")]
    Ambiguous { directory: PathBuf },
    #[error(transparent)]
    Config(#[from] OptoSyncConfigError),
}

fn starting_directory(start: &Path) -> &Path {
    if start.is_file() {
        start.parent().unwrap_or(start)
    } else {
        start
    }
}

/// Finds the nearest Opto Sync config while walking from `start` toward `/`.
///
/// The requested `.opto-cfg.toml` name is preferred for new repositories, while
/// `.opto-sync.toml` remains a compatibility name. If both occur in the same
/// directory the lookup fails closed rather than silently choosing an authority.
pub fn discover_opto_sync_config(
    start: impl AsRef<Path>,
) -> Result<DiscoveredOptoSyncConfig, OptoSyncConfigDiscoveryError> {
    let start = starting_directory(start.as_ref());

    for directory in start.ancestors() {
        let matches = CONFIG_FILENAMES
            .iter()
            .map(|name| directory.join(name))
            .filter(|candidate| candidate.is_file())
            .collect::<Vec<_>>();

        match matches.as_slice() {
            [] => continue,
            [path] => {
                let at_repository_root = directory.join(".git").is_dir();
                if !at_repository_root {
                    tracing::warn!(
                        config_path = %path.display(),
                        config_directory = %directory.display(),
                        "Opto Sync config is not adjacent to a .git directory; using nearest config"
                    );
                }
                return Ok(DiscoveredOptoSyncConfig {
                    path: path.clone(),
                    at_repository_root,
                });
            }
            _ => {
                return Err(OptoSyncConfigDiscoveryError::Ambiguous {
                    directory: directory.to_path_buf(),
                });
            }
        }
    }

    Err(OptoSyncConfigDiscoveryError::NotFound {
        start: start.to_path_buf(),
    })
}

pub fn discover_opto_sync_config_from_cwd(
) -> Result<DiscoveredOptoSyncConfig, OptoSyncConfigDiscoveryError> {
    let cwd = std::env::current_dir().map_err(OptoSyncConfigDiscoveryError::CurrentDirectory)?;
    discover_opto_sync_config(cwd)
}

pub fn load_nearest_opto_sync_config(
    start: impl AsRef<Path>,
) -> Result<OptoSyncConfig, OptoSyncConfigDiscoveryError> {
    let discovered = discover_opto_sync_config(start)?;
    load_opto_sync_config(discovered.path).map_err(Into::into)
}

pub fn load_nearest_opto_sync_config_from_cwd(
) -> Result<OptoSyncConfig, OptoSyncConfigDiscoveryError> {
    let discovered = discover_opto_sync_config_from_cwd()?;
    load_opto_sync_config(discovered.path).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, time::{SystemTime, UNIX_EPOCH}};

    fn scratch(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("opto-sync-{name}-{}-{nonce}", std::process::id()))
    }

    #[test]
    fn nearest_config_wins_and_nested_config_is_marked_non_root() {
        let root = scratch("nearest");
        let nested = root.join("services/api");
        fs::create_dir_all(root.join(".git")).expect("git dir");
        fs::create_dir_all(&nested).expect("nested dir");
        fs::write(root.join(OPTO_SYNC_CONFIG_LEGACY_FILENAME), "version = 1\n").expect("root config");
        fs::write(nested.join(OPTO_SYNC_CONFIG_FILENAME), "version = 1\n").expect("nested config");

        let found = discover_opto_sync_config(nested.join("src")).expect("discover nearest");
        assert_eq!(found.path, nested.join(OPTO_SYNC_CONFIG_FILENAME));
        assert!(!found.at_repository_root);

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn repository_root_config_is_recognized() {
        let root = scratch("root");
        let nested = root.join("services/api/src");
        fs::create_dir_all(root.join(".git")).expect("git dir");
        fs::create_dir_all(&nested).expect("nested dir");
        fs::write(root.join(OPTO_SYNC_CONFIG_FILENAME), "version = 1\n").expect("config");

        let found = discover_opto_sync_config(&nested).expect("discover root config");
        assert_eq!(found.path, root.join(OPTO_SYNC_CONFIG_FILENAME));
        assert!(found.at_repository_root);

        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn aliases_in_same_directory_fail_closed() {
        let root = scratch("ambiguous");
        fs::create_dir_all(&root).expect("root");
        fs::write(root.join(OPTO_SYNC_CONFIG_FILENAME), "version = 1\n").expect("new config");
        fs::write(root.join(OPTO_SYNC_CONFIG_LEGACY_FILENAME), "version = 1\n").expect("legacy config");

        let error = discover_opto_sync_config(&root).expect_err("must reject ambiguity");
        assert!(matches!(error, OptoSyncConfigDiscoveryError::Ambiguous { .. }));

        fs::remove_dir_all(root).expect("cleanup");
    }
}
