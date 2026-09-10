use opto_sync_lib_core::{load_opto_sync_repo_root, OptoSyncConfigError};

#[test]
fn missing_repo_root_contract_is_an_io_failure() {
    let mut root = std::env::temp_dir();
    root.push(format!("opto-sync-missing-config-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create empty root");
    assert!(matches!(load_opto_sync_repo_root(&root), Err(OptoSyncConfigError::Io(_))));
    std::fs::remove_dir_all(root).expect("cleanup");
}
