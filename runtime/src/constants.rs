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

/// Money matters.
pub mod currency {
	use primitives::Balance;

	pub const DOLLARS: Balance = 1_000_000_000_000_000_000;
	pub const CENTS: Balance = DOLLARS / 100;
	pub const MILLICENTS: Balance = CENTS / 1_000;
	pub const MICROCENTS: Balance = MILLICENTS / 1_000;

	// Ethereum units
	pub const ETHER: Balance = 1_000_000_000_000_000_000;
	pub const MILLIETHER: Balance = 1_000_000_000_000_000;
	pub const MICROETHER: Balance = 1_000_000_000_000;
	pub const GWEI: Balance = 1_000_000_000;
	pub const MWEI: Balance = 1_000_000;
	pub const KWEI: Balance = 1_000;
	pub const WEI: Balance = 1;

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 15 * CENTS + (bytes as Balance) * 6 * CENTS
	}
}

/// Time and blocks.
pub mod time {
	use primitives::{Moment, BlockNumber};
	// mainnet
	pub const MILLISECS_PER_BLOCK: Moment = 6000;
	// Testnet
//	pub const MILLISECS_PER_BLOCK: Moment = 1000;
	pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;
	// Kusama
	pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 1 * HOURS;
	// Mainnet
//	pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 4 * HOURS;
	// Testnet
//	pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 10 * MINUTES;

	// These time units are defined in number of blocks.
	pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
	pub const HOURS: BlockNumber = MINUTES * 60;
	pub const DAYS: BlockNumber = HOURS * 24;

	// 1 in 4 blocks (on average, not counting collisions) will be primary babe blocks.
	pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);
}


/// Fee-related.
pub mod fee {
	pub use sp_runtime::Perbill;
	use primitives::Balance;
	use frame_support::weights::{
		WeightToFeePolynomial, WeightToFeeCoefficient, WeightToFeeCoefficients,
		constants::ExtrinsicBaseWeight,
	};
	use smallvec::smallvec;

	/// The block saturation level. Fees will be updates based on this value.
	pub const TARGET_BLOCK_FULLNESS: Perbill = Perbill::from_percent(25);

	/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
	/// node's balance type.
	///
	/// This should typically create a mapping between the following ranges:
	///   - [0, MAXIMUM_BLOCK_WEIGHT]
	///   - [Balance::min, Balance::max]
	///
	/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
	///   - Setting it to `0` will essentially disable the weight fee.
	///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
	pub struct WeightToFee;
	impl WeightToFeePolynomial for WeightToFee {
		type Balance = Balance;
		fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
			// in Kusama, extrinsic base weight (smallest non-zero weight) is mapped to 1/10 CENT:
			let p = super::currency::CENTS;
			let q = 10 * Balance::from(ExtrinsicBaseWeight::get());
			smallvec![WeightToFeeCoefficient {
				degree: 1,
				negative: false,
				coeff_frac: Perbill::from_rational(p % q, q),
				coeff_integer: p / q,
			}]
		}
	}
}