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

//! EvmAccounts pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;

use crate::Pallet as EvmAccounts;

fn alice() -> secp256k1::SecretKey {
    secp256k1::SecretKey::parse(&keccak_256(b"Alice")).unwrap()
}

benchmarks! {

    claim_account {
        let caller: T::AccountId = whitelisted_caller();
    }: _(RawOrigin::Signed(caller), EvmAccounts::<T>::eth_address(&alice()), EvmAccounts::<T>::eth_sign(&alice(), &caller.encode(), &[][..]))

    claim_default_account {
        let caller = whitelisted_caller();
    }: _(RawOrigin::Signed(caller))
}

impl_benchmark_test_suite!(
    EvmAccounts,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime,
);
