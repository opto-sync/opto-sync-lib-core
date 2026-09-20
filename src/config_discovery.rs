#![forbid(unsafe_code)]

use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use thiserror::Error;

use crate::opto_sync_config::{
    load_opto_sync_config, OptoSyncConfig, OptoSyncConfigError, OPTO_SYNC_CONFIG_FILENAME,
};

/// Hard ceiling on implicit ancestor discovery.
pub const MAX_DISCOVERY_ANCESTORS: usize = ores_config_discovery::MAX_ANCESTORS;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoveredOptoSyncConfig {
    pub path: PathBuf,
    /// Fleet placement evidence is intentionally stricter than Git's worktree
    /// detection: only an adjacent `.git` directory counts as repository-root
    /// placement. A `.git` file remains a traversal boundary but is reported as
    /// non-root placement.
    pub at_repository_root: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadedOptoSyncConfig {
    pub source_path: PathBuf,
    pub at_repository_root: bool,
    pub config: OptoSyncConfig,
}

#[derive(Debug, Error)]
pub enum OptoSyncConfigDiscoveryError {
    #[error("failed to resolve the current working directory: {0}")]
    CurrentDirectory(#[source] std::io::Error),
    #[error("no {OPTO_SYNC_CONFIG_FILENAME} found from {start} to the repository boundary")]
    NotFound { start: PathBuf },
    #[error("unsafe Opto Sync config leaf at {path}; expected a regular non-symlink file")]
    UnsafeConfigLeaf { path: PathBuf },
    #[error("failed to inspect Opto Sync config path {path}: {source}")]
    Metadata {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(transparent)]
    Config(#[from] OptoSyncConfigError),
}

#[must_use]
pub fn is_repo_root(directory: &Path) -> bool {
    fs::symlink_metadata(directory.join(".git")).is_ok_and(|metadata| metadata.file_type().is_dir())
}

/// Finds the nearest canonical Opto Sync config while walking from `start`
/// toward the first Git repository boundary.
///
/// `.opto-sync.toml` is the single registered/canonical filename. The earlier
/// draft `.opto-cfg.toml` spelling is deliberately not an alias: accepting it
/// here would create a second authority that conflicts with the reviewed fleet
/// registry and `opto_sync_config` contract.
///
/// Shared discovery owns start-path canonicalization (including file starts),
/// the 64-ancestor cap, Git-boundary handling, and symlink refusal. Opto Sync
/// owns only its stricter non-regular-leaf policy, strict repository-root
/// placement, warning, and missing-config semantics.
pub fn discover_opto_sync_config(
    start: impl AsRef<Path>,
) -> Result<DiscoveredOptoSyncConfig, OptoSyncConfigDiscoveryError> {
    let start = start.as_ref();

    let located = ores_config_discovery::Search::new(OPTO_SYNC_CONFIG_FILENAME)
        .refuse_non_regular(true)
        .from(start)
        .map_err(|error| match error {
            ores_config_discovery::DiscoveryError::Unreadable { path, kind } => {
                OptoSyncConfigDiscoveryError::Metadata {
                    path,
                    source: std::io::Error::from(kind),
                }
            }
            ores_config_discovery::DiscoveryError::Symlink(path)
            | ores_config_discovery::DiscoveryError::NotRegularFile(path) => {
                OptoSyncConfigDiscoveryError::UnsafeConfigLeaf { path }
            }
            _ => OptoSyncConfigDiscoveryError::NotFound {
                start: start.to_path_buf(),
            },
        })?;
    let Some(located) = located else {
        return Err(OptoSyncConfigDiscoveryError::NotFound {
            start: start.to_path_buf(),
        });
    };
    let discovered = DiscoveredOptoSyncConfig {
        at_repository_root: located.beside_git_directory(),
        path: located.path,
    };
    warn_unless_repo_root(&discovered);
    Ok(discovered)
}

pub fn discover_opto_sync_config_from_cwd(
) -> Result<DiscoveredOptoSyncConfig, OptoSyncConfigDiscoveryError> {
    let cwd = std::env::current_dir().map_err(OptoSyncConfigDiscoveryError::CurrentDirectory)?;
    discover_opto_sync_config(cwd)
}

/// Loads the owner-selected config while retaining the exact selected path for
/// provenance/digest receipts.
pub fn load_nearest_opto_sync_config_with_source(
    start: impl AsRef<Path>,
) -> Result<LoadedOptoSyncConfig, OptoSyncConfigDiscoveryError> {
    let discovered = discover_opto_sync_config(start)?;
    let config = load_opto_sync_config(&discovered.path)?;
    Ok(LoadedOptoSyncConfig {
        source_path: discovered.path,
        at_repository_root: discovered.at_repository_root,
        config,
    })
}

pub fn load_nearest_opto_sync_config_from_cwd_with_source(
) -> Result<LoadedOptoSyncConfig, OptoSyncConfigDiscoveryError> {
    let cwd = std::env::current_dir().map_err(OptoSyncConfigDiscoveryError::CurrentDirectory)?;
    load_nearest_opto_sync_config_with_source(cwd)
}

/// Compatibility helper for callers that only need the parsed config. New
/// provenance-sensitive server startup should prefer the `*_with_source` form.
pub fn load_nearest_opto_sync_config(
    start: impl AsRef<Path>,
) -> Result<OptoSyncConfig, OptoSyncConfigDiscoveryError> {
    Ok(load_nearest_opto_sync_config_with_source(start)?.config)
}

/// Compatibility helper for callers that only need the parsed config.
pub fn load_nearest_opto_sync_config_from_cwd(
) -> Result<OptoSyncConfig, OptoSyncConfigDiscoveryError> {
    Ok(load_nearest_opto_sync_config_from_cwd_with_source()?.config)
}

fn warn_unless_repo_root(discovered: &DiscoveredOptoSyncConfig) {
    if discovered.at_repository_root {
        return;
    }
    let _ = discovery_logger()
        .warn(vec![next_loggers::json!({
            "event": "ores.config.not_at_repo_root",
            "ores.config.file": OPTO_SYNC_CONFIG_FILENAME,
            "ores.config.path": discovered.path.display().to_string(),
            "ores.config.at_repo_root": false,
            "detail": "Opto Sync configuration was selected without an adjacent .git directory; confirm this file is meant to govern the running service"
        })])
        .send();
}

fn discovery_logger() -> &'static next_loggers::Logger {
    static LOGGER: OnceLock<next_loggers::Logger> = OnceLock::new();
    LOGGER.get_or_init(|| {
        next_loggers::Logger::new(next_loggers::Options {
            app_name: "opto-sync-lib-core".into(),
            name: Some("config-discovery".into()),
            ..next_loggers::Options::default()
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn scratch(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("opto-sync-{name}-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&root).expect("root");
        root
    }

    #[test]
    fn nearest_canonical_config_wins_and_retains_source_path() {
        let root = scratch("nearest");
        let nested = root.join("services/api");
        let start = nested.join("src");
        fs::create_dir_all(root.join(".git")).expect("git dir");
        fs::create_dir_all(&start).expect("start dir");
        fs::write(root.join(OPTO_SYNC_CONFIG_FILENAME), "version = 1\n").expect("root config");
        fs::write(nested.join(OPTO_SYNC_CONFIG_FILENAME), "version = 1\n").expect("nested config");

        let found = discover_opto_sync_config(&start).expect("discover nearest");
        assert_eq!(found.path, nested.join(OPTO_SYNC_CONFIG_FILENAME));
        assert!(!found.at_repository_root);
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn file_start_uses_shared_parent_directory_semantics() {
        let root = scratch("file-start");
        fs::create_dir_all(root.join(".git")).expect("git dir");
        let config = root.join(OPTO_SYNC_CONFIG_FILENAME);
        fs::write(&config, "version = 1\n").expect("config");
        let source = root.join("src/main.rs");
        fs::create_dir_all(source.parent().expect("source parent")).expect("source parent dir");
        fs::write(&source, "fn main() {}\n").expect("source file");

        let found = discover_opto_sync_config(&source).expect("file start discovers parent config");
        assert_eq!(found.path, config);
        assert!(found.at_repository_root);
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn nonexistent_start_fails_closed_as_metadata_error() {
        let root = scratch("missing-start");
        let missing = root.join("does/not/exist");
        let error =
            discover_opto_sync_config(&missing).expect_err("missing start must fail closed");
        match error {
            OptoSyncConfigDiscoveryError::Metadata { path, source } => {
                assert_eq!(path, missing);
                assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
            }
            other => panic!("expected metadata error for missing start, got {other:?}"),
        }
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn repository_root_requires_a_git_directory() {
        let root = scratch("root-dir");
        fs::create_dir_all(root.join(".git")).expect("git dir");
        fs::write(root.join(OPTO_SYNC_CONFIG_FILENAME), "version = 1\n").expect("config");
        assert!(
            discover_opto_sync_config(&root)
                .expect("find")
                .at_repository_root
        );
        fs::remove_dir_all(root).expect("cleanup");

        let worktree = scratch("root-file");
        fs::write(worktree.join(".git"), "gitdir: /elsewhere\n").expect("git file");
        fs::write(worktree.join(OPTO_SYNC_CONFIG_FILENAME), "version = 1\n").expect("config");
        assert!(
            !discover_opto_sync_config(&worktree)
                .expect("find")
                .at_repository_root
        );
        fs::remove_dir_all(worktree).expect("cleanup");
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_git_directory_is_boundary_but_not_root_placement() {
        use std::os::unix::fs::symlink;

        let root = scratch("git-symlink");
        let actual_git = root.join("actual-git");
        fs::create_dir_all(&actual_git).expect("actual git dir");
        symlink(&actual_git, root.join(".git")).expect("git symlink");
        fs::write(root.join(OPTO_SYNC_CONFIG_FILENAME), "version = 1\n").expect("config");

        assert!(has_git_boundary(&root));
        assert!(!is_repo_root(&root));
        assert!(
            !discover_opto_sync_config(&root)
                .expect("find")
                .at_repository_root
        );
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn discovery_does_not_escape_the_first_git_boundary() {
        let outer = scratch("boundary");
        let repo = outer.join("repo");
        let deep = repo.join("services/api");
        fs::create_dir_all(repo.join(".git")).expect("git dir");
        fs::create_dir_all(&deep).expect("deep");
        fs::write(outer.join(OPTO_SYNC_CONFIG_FILENAME), "version = 1\n").expect("outer config");

        let error = discover_opto_sync_config(&deep).expect_err("must stop at repo boundary");
        assert!(matches!(
            error,
            OptoSyncConfigDiscoveryError::NotFound { .. }
        ));
        fs::remove_dir_all(outer).expect("cleanup");
    }

    #[test]
    fn unregistered_opto_cfg_spelling_is_not_an_alias() {
        let root = scratch("unregistered");
        fs::create_dir_all(root.join(".git")).expect("git dir");
        fs::write(root.join(".opto-cfg.toml"), "version = 1\n").expect("stale name");
        let error = discover_opto_sync_config(&root).expect_err("unregistered name must not load");
        assert!(matches!(
            error,
            OptoSyncConfigDiscoveryError::NotFound { .. }
        ));
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[cfg(unix)]
    #[test]
    fn symlink_config_leaf_is_rejected_before_read() {
        use std::os::unix::fs::symlink;
        let root = scratch("symlink");
        fs::create_dir_all(root.join(".git")).expect("git dir");
        let real = root.join("real.toml");
        fs::write(&real, "version = 1\n").expect("real");
        symlink(&real, root.join(OPTO_SYNC_CONFIG_FILENAME)).expect("symlink");
        let error = discover_opto_sync_config(&root).expect_err("symlink must fail closed");
        assert!(matches!(
            error,
            OptoSyncConfigDiscoveryError::UnsafeConfigLeaf { .. }
        ));
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn discovery_is_bounded_to_sixty_four_ancestors() {
        let root = scratch("bounded");
        let mut deep = root.clone();
        for index in 0..(MAX_DISCOVERY_ANCESTORS + 2) {
            deep.push(format!("d{index}"));
        }
        fs::create_dir_all(&deep).expect("deep tree");
        fs::write(root.join(OPTO_SYNC_CONFIG_FILENAME), "version = 1\n").expect("too-far config");
        let error =
            discover_opto_sync_config(&deep).expect_err("too-far config must not be selected");
        assert!(matches!(
            error,
            OptoSyncConfigDiscoveryError::NotFound { .. }
        ));
        fs::remove_dir_all(root).expect("cleanup");
    }
}
