use std::collections::BTreeMap;

use opto_sync_lib_core::merge_opto_sync_environment;

#[test]
fn composition_keeps_unrelated_environment_keys() {
    let ambient = BTreeMap::from([("UNRELATED".to_owned(), "kept".to_owned())]);
    let merged = merge_opto_sync_environment(&ambient, &BTreeMap::new());
    assert_eq!(merged.get("UNRELATED").map(String::as_str), Some("kept"));
}
