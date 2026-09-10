use std::{fs, path::PathBuf};

use opto_sync_lib_core::load_opto_sync_repo_root;

#[test]
fn loads_only_the_explicit_repo_root_contract() {
    let mut root = std::env::temp_dir();
    root.push(format!("opto-sync-config-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create temp repo root");

    let config = r#"
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
name = "bind_addr"
key = "OPTO_SYNC_API_BIND"
kind = "string"
required = false
secret = false
default = "127.0.0.1:8080"

[server]
enabled = true
bind_addr_binding = "bind_addr"
"#;

    fs::write(root.join(".opto-sync.toml"), config).expect("write contract");
    let loaded = load_opto_sync_repo_root(PathBuf::from(&root)).expect("load repo-root contract");
    assert_eq!(loaded.version, 1);

    fs::remove_dir_all(root).expect("cleanup");
}
