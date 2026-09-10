use std::fs;

use opto_sync_lib_core::{load_opto_sync_config, OptoSyncConfigError};

#[test]
fn non_utf8_contract_fails_closed() {
    let mut path = std::env::temp_dir();
    path.push(format!("opto-sync-non-utf8-{}.toml", std::process::id()));
    fs::write(&path, [0xff, 0xfe, 0xfd]).expect("write invalid utf8");
    assert!(matches!(load_opto_sync_config(&path), Err(OptoSyncConfigError::Toml(_))));
    fs::remove_file(path).expect("cleanup");
}
