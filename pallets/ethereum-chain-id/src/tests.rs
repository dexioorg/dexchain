use crate::mock::*;
use frame_support::traits::Get;

#[test]
fn it_works_for_get_chain_id() {
    // default
    new_test_ext().execute_with(|| {
        // Read pallet storage and assert an expected result.
        assert_eq!(EthereumChainId::get(), primitives::ChainId::Testnet.u64());
    });

    // testnet
    ExtBuilder::testnet().build().execute_with(|| {
        // Read pallet storage and assert an expected result.
        assert_eq!(EthereumChainId::get(), primitives::ChainId::Testnet.u64());
    });

    // mainnet
    ExtBuilder::mainnet().build().execute_with(|| {
        // Read pallet storage and assert an expected result.
        assert_eq!(EthereumChainId::get(), primitives::ChainId::Mainnet.u64());
    });
}
