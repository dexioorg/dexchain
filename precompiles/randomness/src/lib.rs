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

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use evm::{Context, ExitError, ExitSucceed};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::Precompile;
use precompile_utils::{EvmDataReader, EvmDataWriter, Gasometer, RuntimeHelper};
use primitives::{BlockNumber, Hash};
use sp_core::H256;
use sp_std::{fmt::Debug, marker::PhantomData, vec::Vec};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq, num_enum::TryFromPrimitive)]
enum Action {
    Random = "random(bytes32)",
}

pub struct Randomness<Runtime>(PhantomData<Runtime>);

impl<Runtime> Precompile for Randomness<Runtime>
where
    Runtime: pallet_evm::Config + frame_system::Config,
    Runtime: frame_support::traits::Randomness<Hash, BlockNumber>,
    Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
    <Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
{
    fn execute(
        input: &[u8], //Reminder this is big-endian
        target_gas: Option<u64>,
        _context: &Context,
    ) -> core::result::Result<(ExitSucceed, Vec<u8>, u64), ExitError> {
        log::trace!(target: "randomness-precompile", "In randomness wrapper");

        let mut input = EvmDataReader::new(input);
        match &input.read_selector()? {
            Action::Random => Self::random(input, target_gas),
        }
    }
}

impl<Runtime> Randomness<Runtime>
where
    Runtime: pallet_evm::Config + frame_system::Config,
    Runtime: frame_support::traits::Randomness<Hash, BlockNumber>,
    Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
    <Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
{
    pub fn random(
        mut input: EvmDataReader,
        target_gas: Option<u64>,
    ) -> core::result::Result<(ExitSucceed, Vec<u8>, u64), ExitError> {
        let mut gasometer = Gasometer::new(target_gas);
        gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?; // accounts_payable

        // nonce is bytes32
        let subject: H256 = input.read::<H256>()?.into();
        let (seed, blocknum) = Runtime::random(&subject.encode());

        Ok((
            ExitSucceed::Returned,
            EvmDataWriter::new().write(seed).write(blocknum).build(),
            gasometer.used_gas(),
        ))
    }
}
