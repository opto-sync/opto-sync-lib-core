use opto_sync_lib_core::{parse_opto_sync_config, OptoSyncConfigError};

#[test]
fn rejects_oversized_config_before_toml_parsing() {
    let oversized = "x".repeat(256 * 1024 + 1);
    assert_eq!(
        parse_opto_sync_config(&oversized).unwrap_err(),
        OptoSyncConfigError::TooLarge
    );
}
