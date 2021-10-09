// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of DEXCHAIN.
//
// Copyright (c) 2021 Dexio Technologies.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use super::*;
use frame_support::parameter_types;
use frame_support_test::TestRandomness;
use pallet_evm::{AddressMapping, EnsureAddressTruncated, FeeCalculator, PrecompileSet};
use primitives::{Balance, Hash};
use sp_core::{H160, H256, U256};
use sp_runtime::{
    generic,
    traits::{BlakeTwo256, IdentityLookup},
    AccountId32,
};
use sp_std::vec::Vec;

pub type AccountId = AccountId32;
pub type BlockNumber = u32;

pub const ALICE: AccountId = AccountId32::new([0u8; 32]);

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
        Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
        Evm: pallet_evm::{Module, Call, Storage, Event<T>},
    }
);

/// The staking precompile is available at address one in the mock runtime.
pub fn precompile_address() -> H160 {
    H160::from_low_u64_be(2048)
}

#[derive(Debug, Clone, Copy)]
pub struct TestPrecompiles<R>(PhantomData<R>);

impl<R> PrecompileSet for TestPrecompiles<R>
where
    R: pallet_evm::Config + frame_system::Config,
    R: frame_support::traits::Randomness<Hash, BlockNumber>,
    R::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
    <R::Call as Dispatchable>::Origin: From<Option<R::AccountId>>,
{
    fn execute(
        address: H160,
        input: &[u8],
        target_gas: Option<u64>,
        context: &Context,
    ) -> Option<Result<(ExitSucceed, Vec<u8>, u64), ExitError>> {
        match address {
            a if a == precompile_address() => {
                Some(Randomness::<R>::execute(input, target_gas, context))
            }
            _ => None,
        }
    }
}

parameter_types! {
    pub const BlockHashCount: u32 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Call = Call;
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = generic::Header<u32, BlakeTwo256>;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type BlockWeights = ();
    type BlockLength = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BaseCallFilter = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    // type OnSetCode = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = 1000;
}
impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
}
impl pallet_balances::Config for Test {
    type Balance = Balance;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = frame_system::Module<Test>;
    type MaxLocks = ();
    type WeightInfo = ();
}

pub struct FixedGasPrice;

impl FeeCalculator for FixedGasPrice {
    fn min_gas_price() -> U256 {
        // Gas price is always one token per gas.
        1.into()
    }
}

pub struct HashedAddressMapping;

impl AddressMapping<AccountId32> for HashedAddressMapping {
    fn into_account_id(address: H160) -> AccountId32 {
        let mut data = [0u8; 32];
        data[0..20].copy_from_slice(&address[..]);
        AccountId32::from(Into::<[u8; 32]>::into(data))
    }
}

parameter_types! {
    pub const ChainId: u64 = 9000;
    pub BlockGasLimit: U256 = U256::from(u32::max_value());
}

pub type Precompiles = TestPrecompiles<Test>;

impl pallet_evm::Config for Test {
    type FeeCalculator = FixedGasPrice;
    type GasWeightMapping = ();
    type CallOrigin = EnsureAddressTruncated;
    type WithdrawOrigin = EnsureAddressTruncated;
    type AddressMapping = HashedAddressMapping;
    type Currency = Balances;
    type Event = Event;
    type Runner = pallet_evm::runner::stack::Runner<Self>;
    type Precompiles = Precompiles;
    type ChainId = ChainId;
    type OnChargeTransaction = ();
    type BlockGasLimit = BlockGasLimit;
}

impl frame_support::traits::Randomness<Hash, BlockNumber> for Test {
    fn random(subject: &[u8]) -> (Hash, BlockNumber) {
        TestRandomness::<Test>::random(subject)
    }
}

pub struct ExtBuilder();

impl Default for ExtBuilder {
    fn default() -> Self {
        Self()
    }
}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        pallet_balances::GenesisConfig::<Test> {
            balances: vec![(ALICE, 1000000000)],
        }
        .assimilate_storage(&mut t)
        .unwrap();

        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}

// Helper function to give a simple evm context suitable for tests.
// We can remove this once https://github.com/rust-blockchain/evm/pull/35
// is in our dependency graph.
pub fn evm_test_context() -> evm::Context {
    evm::Context {
        address: Default::default(),
        caller: Default::default(),
        apparent_value: From::from(0),
    }
}
