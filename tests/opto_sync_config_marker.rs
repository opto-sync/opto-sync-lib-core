#[test]
fn runtime_projection_is_not_a_contract_authority() {
    assert_ne!(opto_sync_lib_core::OPTO_SYNC_CONFIG_CONTRACT_REPOSITORY, "opto-sync/opto-sync-lib-core");
}
