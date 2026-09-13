use std::collections::BTreeMap;

use opto_sync_lib_core::merge_opto_sync_environment;

#[test]
fn merge_environment_is_pure_and_argv_wins() {
    let ambient = BTreeMap::from([
        ("A".to_owned(), "ambient".to_owned()),
        ("B".to_owned(), "ambient-b".to_owned()),
    ]);
    let argv = BTreeMap::from([
        ("A".to_owned(), "argv".to_owned()),
        ("C".to_owned(), "argv-c".to_owned()),
    ]);
    let merged = merge_opto_sync_environment(&ambient, &argv);
    assert_eq!(merged.get("A").map(String::as_str), Some("argv"));
    assert_eq!(merged.get("B").map(String::as_str), Some("ambient-b"));
    assert_eq!(merged.get("C").map(String::as_str), Some("argv-c"));
    assert_eq!(ambient.get("A").map(String::as_str), Some("ambient"));
}
