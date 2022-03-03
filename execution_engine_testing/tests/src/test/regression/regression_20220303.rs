use std::{collections::BTreeMap, convert::TryInto};

use casper_engine_test_support::{LmdbWasmTestBuilder, UpgradeRequestBuilder};
use casper_execution_engine::core::engine_state::SystemContractRegistry;
use casper_hashing::Digest;
use casper_types::{
    system::mint, AccessRights, CLValue, ContractHash, EraId, Key, ProtocolVersion, StoredValue,
    URef,
};
use once_cell::sync::Lazy;

use crate::lmdb_fixture;

const DEFAULT_ACTIVATION_POINT: EraId = EraId::new(1);

static MINT_CONTRACT_HASH: Lazy<ContractHash> = Lazy::new(|| {
    // Taken out of the lmdb RELEASE_1_3_1 fixture
    let bytes =
        base16::decode("ecfb68b4b6a9fc33aad5ba41961bb8b5078dc0bc97d6135104da5b1e1b212350").unwrap();
    let hash_bytes: [u8; 32] = bytes.as_slice().try_into().expect("should convert");
    ContractHash::new(hash_bytes)
});

#[ignore]
#[test]
fn should_update_contract_metadata_at_upgrade_with_minor_bump() {
    let (mut builder, lmdb_fixture_state, _temp_dir) =
        lmdb_fixture::builder_from_global_state_fixture(lmdb_fixture::RELEASE_1_3_1);

    let old_protocol_version = lmdb_fixture_state.genesis_protocol_version();

    let old_contract = builder
        .get_contract(*MINT_CONTRACT_HASH)
        .expect("should have mint contract");
    assert_eq!(old_contract.protocol_version(), old_protocol_version);

    let new_protocol_version = ProtocolVersion::from_parts(
        old_protocol_version.value().major,
        old_protocol_version.value().minor + 1,
        0,
    );

    let global_state_update =
        apply_global_state_update(&builder, lmdb_fixture_state.post_state_hash);

    let mut upgrade_request = {
        UpgradeRequestBuilder::new()
            .with_current_protocol_version(old_protocol_version)
            .with_new_protocol_version(new_protocol_version)
            .with_activation_point(DEFAULT_ACTIVATION_POINT)
            .with_global_state_update(global_state_update)
            .build()
    };

    builder
        .upgrade_with_upgrade_request(*builder.get_engine_state().config(), &mut upgrade_request)
        .expect_upgrade_success();

    let new_contract = builder
        .get_contract(*MINT_CONTRACT_HASH)
        .expect("should have mint contract");

    assert_eq!(
        old_contract.contract_package_hash(),
        new_contract.contract_package_hash()
    );
    assert_eq!(
        old_contract.contract_wasm_hash(),
        new_contract.contract_wasm_hash()
    );
    assert_ne!(old_contract.entry_points(), new_contract.entry_points());
    assert_eq!(
        new_contract.entry_points(),
        &mint::mint_entry_points(),
        "should have new entrypoints written"
    );
    assert_eq!(new_contract.protocol_version(), new_protocol_version);
}

#[ignore]
#[test]
fn should_update_contract_metadata_at_upgrade_with_patch_bump() {
    let (mut builder, lmdb_fixture_state, _temp_dir) =
        lmdb_fixture::builder_from_global_state_fixture(lmdb_fixture::RELEASE_1_3_1);

    let old_protocol_version = lmdb_fixture_state.genesis_protocol_version();

    let old_contract = builder
        .get_contract(*MINT_CONTRACT_HASH)
        .expect("should have mint contract");
    assert_eq!(old_contract.protocol_version(), old_protocol_version);

    let new_protocol_version = ProtocolVersion::from_parts(
        old_protocol_version.value().major,
        old_protocol_version.value().minor,
        old_protocol_version.value().patch + 1,
    );

    let global_state_update =
        apply_global_state_update(&builder, lmdb_fixture_state.post_state_hash);

    let mut upgrade_request = {
        UpgradeRequestBuilder::new()
            .with_current_protocol_version(old_protocol_version)
            .with_new_protocol_version(new_protocol_version)
            .with_activation_point(DEFAULT_ACTIVATION_POINT)
            .with_global_state_update(global_state_update)
            .build()
    };

    builder
        .upgrade_with_upgrade_request(*builder.get_engine_state().config(), &mut upgrade_request)
        .expect_upgrade_success();

    let new_contract = builder
        .get_contract(*MINT_CONTRACT_HASH)
        .expect("should have mint contract");

    assert_eq!(
        old_contract.contract_package_hash(),
        new_contract.contract_package_hash()
    );
    assert_eq!(
        old_contract.contract_wasm_hash(),
        new_contract.contract_wasm_hash()
    );
    assert_ne!(old_contract.entry_points(), new_contract.entry_points());
    assert_eq!(
        new_contract.entry_points(),
        &mint::mint_entry_points(),
        "should have new entrypoints written"
    );
    assert_eq!(new_contract.protocol_version(), new_protocol_version);
}

fn apply_global_state_update(
    builder: &LmdbWasmTestBuilder,
    post_state_hash: Digest,
) -> BTreeMap<Key, StoredValue> {
    let key = URef::new([0u8; 32], AccessRights::all()).into();

    let system_contract_hashes = builder
        .query(Some(post_state_hash), key, &Vec::new())
        .expect("Must have stored system contract hashes")
        .as_cl_value()
        .expect("must be CLValue")
        .clone()
        .into_t::<SystemContractRegistry>()
        .expect("must convert to btree map");

    let mut global_state_update = BTreeMap::<Key, StoredValue>::new();
    let registry = CLValue::from_t(system_contract_hashes)
        .expect("must convert to StoredValue")
        .into();

    global_state_update.insert(Key::SystemContractRegistry, registry);

    global_state_update
}
