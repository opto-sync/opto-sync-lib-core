#[test]
fn merged_authority_and_validator_revisions_are_pinned() {
    assert_eq!(opto_sync_lib_core::OPTO_SYNC_CONFIG_CONTRACT_REVISION.len(), 40);
    assert_eq!(opto_sync_lib_core::OPTO_SYNC_CONFIG_TJSV_REVISION.len(), 40);
}
