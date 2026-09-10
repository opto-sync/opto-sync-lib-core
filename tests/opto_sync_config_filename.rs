#[test]
fn canonical_filename_is_repo_root_opto_sync_toml() {
    assert_eq!(opto_sync_lib_core::OPTO_SYNC_CONFIG_FILENAME, ".opto-sync.toml");
}
