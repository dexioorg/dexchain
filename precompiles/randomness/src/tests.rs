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

use crate::mock::{evm_test_context, precompile_address, ExtBuilder, Precompiles};
use pallet_evm::PrecompileSet;
use precompile_utils::EvmDataWriter;
use sha3::{Digest, Keccak256};
use sp_core::H256;

#[test]
fn test_randomness() {
    ExtBuilder::default().build().execute_with(|| {
        let selector = &Keccak256::digest(b"random(bytes32)")[0..4];
        println!("selector: {}", hex::encode(selector));
        let input = EvmDataWriter::new()
            .write_raw_bytes(selector)
            .write(H256::from([0u8; 32]))
            .build();
        let to = precompile_address();
        println!("to: {}", hex::encode(to));
        println!("input: {}", hex::encode(&input));
        let out = Precompiles::execute(to, &input, None, &evm_test_context())
            .unwrap()
            .expect("can not execute");
        println!("seed: {}, blocknum: {}", hex::encode(out.1), out.2);
    });
}
