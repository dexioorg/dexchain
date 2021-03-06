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

//! # Evm Accounts Module
//!
//! ## Overview
//!
//! Evm Accounts module provide a two way mapping between Substrate accounts and
//! EVM accounts so user only have deal with one account / private key.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use codec::Encode;
use ethereum::{TransactionAction, TransactionSignature};
use ethereum_types::{H160, H256, U256};
use frame_support::{
    ensure,
    pallet_prelude::*,
    traits::{
        Currency, ExistenceRequirement, IsType, OnKilledAccount, ReservableCurrency,
        WithdrawReasons,
    },
    transactional,
};
use frame_system::{ensure_signed, pallet_prelude::*};
use pallet_ethereum;
use pallet_evm::{FeeCalculator, GasWeightMapping};
use primitives::{AccountIndex, EvmAddress};
use sp_core::{crypto::AccountId32, ecdsa};
use sp_io::{
    crypto::secp256k1_ecdsa_recover,
    hashing::{blake2_256, keccak_256},
};
use sp_runtime::{
    traits::{LookupError, StaticLookup, Zero},
    MultiAddress, SaturatedConversion,
};
use sp_std::{marker::PhantomData, vec::Vec};

mod benchmarking;
mod mock;
mod tests;
pub mod weights;

pub use module::*;
pub use weights::WeightInfo;

pub type EcdsaSignature = ecdsa::Signature;

/// Type alias for currency balance.
pub type BalanceOf<T> =
    <<T as Config>::NativeCurrency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// A mapping between `AccountId` and `EvmAddress`.
pub trait AddressMapping<AccountId> {
    fn get_account_id(evm: &EvmAddress) -> AccountId;
    fn get_evm_address(account_id: &AccountId) -> Option<EvmAddress>;
    fn get_or_create_evm_address(account_id: &AccountId) -> EvmAddress;
    fn get_default_evm_address(account_id: &AccountId) -> EvmAddress;
    fn is_linked(account_id: &AccountId, evm: &EvmAddress) -> bool;
}

#[frame_support::pallet]
pub mod module {

    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_ethereum::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The Currency for managing Evm account assets.
        type NativeCurrency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// Mapping from address to account id.
        type EvmAddressMapping: AddressMapping<Self::AccountId>;

        /// Weight information for the extrinsics in this module.
        type WeightInfo: WeightInfo;
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {
        /// Mapping between Substrate accounts and EVM accounts
        /// claim account. \[account_id, evm_address\]
        ClaimAccount(T::AccountId, EvmAddress),
    }

    /// Error for evm accounts module.
    #[pallet::error]
    pub enum Error<T> {
        /// AccountId has mapped
        AccountIdHasMapped,
        /// Eth address has mapped
        EthAddressHasMapped,
        /// Bad signature
        BadSignature,
        /// Invalid signature
        InvalidSignature,
        /// Account ref count is not zero
        NonZeroRefCount,
        /// Account still has active reserved
        StillHasActiveReserved,
        /// AccountId not mapped
        AccountIdNotMapped,
    }

    #[pallet::storage]
    #[pallet::getter(fn accounts)]
    pub type Accounts<T: Config> = StorageMap<_, Twox64Concat, EvmAddress, T::AccountId>;

    #[pallet::storage]
    #[pallet::getter(fn evm_addresses)]
    pub type EvmAddresses<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, EvmAddress>;

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Claim account mapping between Substrate accounts and EVM accounts.
        /// Ensure eth_address has not been mapped.
        ///
        /// - `eth_address`: The address to bind to the caller's account
        /// - `eth_signature`: A signature generated by the address to prove
        ///   ownership
        #[pallet::weight(<T as module::Config>::WeightInfo::claim_account())]
        #[transactional]
        pub fn claim_account(
            origin: OriginFor<T>,
            eth_address: EvmAddress,
            eth_signature: EcdsaSignature,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // ensure account_id and eth_address has not been mapped
            ensure!(
                !EvmAddresses::<T>::contains_key(&who),
                Error::<T>::AccountIdHasMapped
            );
            ensure!(
                !Accounts::<T>::contains_key(eth_address),
                Error::<T>::EthAddressHasMapped
            );

            // recover evm address from signature
            let address =
                Self::eth_recover(&eth_signature, &who.using_encoded(to_ascii_hex), &[][..])
                    .ok_or(Error::<T>::BadSignature)?;
            ensure!(eth_address == address, Error::<T>::InvalidSignature);

            // check if the evm padded address already exists
            let account_id = T::EvmAddressMapping::get_account_id(&eth_address);
            if frame_system::Module::<T>::account_exists(&account_id) {
                // merge balance from `evm padded address` to `origin`
                Self::merge_account(&account_id, &who)?;
            }

            Accounts::<T>::insert(eth_address, &who);
            EvmAddresses::<T>::insert(&who, eth_address);

            Self::deposit_event(Event::ClaimAccount(who, eth_address));

            Ok(().into())
        }

        /// Claim account mapping between Substrate accounts and a generated EVM
        /// address based off of those accounts.
        /// Ensure eth_address has not been mapped
        #[pallet::weight(<T as module::Config>::WeightInfo::claim_default_account())]
        pub fn claim_default_account(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // ensure account_id has not been mapped
            ensure!(
                !EvmAddresses::<T>::contains_key(&who),
                Error::<T>::AccountIdHasMapped
            );

            let eth_address = T::EvmAddressMapping::get_or_create_evm_address(&who);

            Self::deposit_event(Event::ClaimAccount(who, eth_address));

            Ok(().into())
        }

        /// Transact an Ethereum transaction.
        /// Similar to the transact function of pallet-ethereum, but using origin to trigger
        #[pallet::weight(<T as pallet_evm::Config>::GasWeightMapping::gas_to_weight(*gas_limit))]
        #[transactional]
        pub fn origin_transact(
            origin: OriginFor<T>,
            action: TransactionAction,
            value: BalanceOf<T>,
            input: Option<Vec<u8>>,
            gas_limit: u64,
            auto_claim: bool,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            if !auto_claim {
                ensure!(
                    EvmAddresses::<T>::contains_key(&who),
                    Error::<T>::AccountIdNotMapped
                );
            }

            let nonce = frame_system::Module::<T>::account_nonce(&who);
            // Get the evm address of accountId
            let evm_address = T::EvmAddressMapping::get_or_create_evm_address(&who.clone());
            let transaction = ethereum::Transaction {
                nonce: U256::from(nonce.saturated_into::<u64>()),
                gas_price: T::FeeCalculator::min_gas_price(),
                gas_limit: U256::from(gas_limit),
                action,
                value: U256::from(value.saturated_into::<u128>()),
                input: input.unwrap_or(Vec::new()),
                signature: Self::empty_tx_signature(),
            };

            let pdi = pallet_ethereum::Module::<T>::do_transact(evm_address, transaction)?;

            Ok(pdi)
        }

        /// Transact an Ethereum raw transaction.
        /// Similar to the transact function of pallet-ethereum, but using origin to trigger
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
        pub fn origin_transact_raw(
            origin: OriginFor<T>,
            raw: Vec<u8>,
            auto_claim: bool,
        ) -> DispatchResultWithPostInfo {
            // TODO:

            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    // empty_tx_signature
    fn empty_tx_signature() -> TransactionSignature {
        let r: H256 = H256([
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x01,
        ]);
        let s = r.clone();
        let signer = T::ChainId::get() * 2 + 35;
        let sig = TransactionSignature::new(signer, r, s).unwrap();
        sig
    }

    // Constructs the message that Ethereum RPC's `personal_sign` and `eth_sign`
    // would sign.
    pub fn ethereum_signable_message(what: &[u8], extra: &[u8]) -> Vec<u8> {
        let prefix = b"dexio evm:";
        let mut l = prefix.len() + what.len() + extra.len();
        let mut rev = Vec::new();
        while l > 0 {
            rev.push(b'0' + (l % 10) as u8);
            l /= 10;
        }
        let mut v = b"\x19Ethereum Signed Message:\n".to_vec();
        v.extend(rev.into_iter().rev());
        v.extend_from_slice(&prefix[..]);
        v.extend_from_slice(what);
        v.extend_from_slice(extra);
        v
    }

    // Attempts to recover the Ethereum address from a message signature signed by
    // using the Ethereum RPC's `personal_sign` and `eth_sign`.
    pub fn eth_recover(s: &EcdsaSignature, what: &[u8], extra: &[u8]) -> Option<EvmAddress> {
        let msg = keccak_256(&Self::ethereum_signable_message(what, extra));
        let mut res = EvmAddress::default();
        res.0
            .copy_from_slice(&keccak_256(&secp256k1_ecdsa_recover(&s.0, &msg).ok()?[..])[12..]);
        Some(res)
    }

    // Returns an Etherum public key derived from an Ethereum secret key.
    pub fn eth_public(secret: &secp256k1::SecretKey) -> secp256k1::PublicKey {
        secp256k1::PublicKey::from_secret_key(secret)
    }

    // Returns an Etherum address derived from an Ethereum secret key.
    pub fn eth_address(secret: &secp256k1::SecretKey) -> EvmAddress {
        EvmAddress::from_slice(&keccak_256(&Self::eth_public(secret).serialize()[1..65])[12..])
    }

    // Constructs a message and signs it.
    pub fn eth_sign(secret: &secp256k1::SecretKey, what: &[u8], extra: &[u8]) -> EcdsaSignature {
        let msg = keccak_256(&Self::ethereum_signable_message(
            &to_ascii_hex(what)[..],
            extra,
        ));
        let (sig, recovery_id) = secp256k1::sign(&secp256k1::Message::parse(&msg), secret);
        let mut r = [0u8; 65];
        r[0..64].copy_from_slice(&sig.serialize()[..]);
        r[64] = recovery_id.serialize();
        EcdsaSignature::from_slice(&r)
    }

    #[transactional]
    fn merge_account(source: &T::AccountId, dest: &T::AccountId) -> DispatchResult {
        let reserved = T::NativeCurrency::reserved_balance(&source);
        ensure!(reserved.is_zero(), Error::<T>::StillHasActiveReserved);

        let free = T::NativeCurrency::free_balance(&source);

        let _ = T::NativeCurrency::resolve_creating(
            &dest,
            T::NativeCurrency::withdraw(
                &source,
                free,
                WithdrawReasons::TRANSFER,
                ExistenceRequirement::AllowDeath,
            )?,
        );

        Ok(())
    }
}

// Creates a an EvmAddress from an AccountId by appending the bytes "evm:" to
// the account_id and hashing it.
fn account_to_default_evm_address(account_id: &impl Encode) -> EvmAddress {
    let payload = (b"evm:", account_id);
    EvmAddress::from_slice(&payload.using_encoded(blake2_256)[0..20])
}

pub struct EvmAddressMapping<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> AddressMapping<T::AccountId> for EvmAddressMapping<T>
where
    T::AccountId: IsType<AccountId32>,
{
    // Returns the AccountId used go generate the given EvmAddress.
    fn get_account_id(address: &EvmAddress) -> T::AccountId {
        if let Some(acc) = Accounts::<T>::get(address) {
            acc
        } else {
            let mut data: [u8; 32] = [0u8; 32];
            data[0..4].copy_from_slice(b"evm:");
            data[4..24].copy_from_slice(&address[..]);
            AccountId32::from(data).into()
        }
    }

    // Returns the EvmAddress associated with a given AccountId or the
    // underlying EvmAddress of the AccountId.
    // Returns None if there is no EvmAddress associated with the AccountId
    // and there is no underlying EvmAddress in the AccountId.
    fn get_evm_address(account_id: &T::AccountId) -> Option<EvmAddress> {
        // Return the EvmAddress if a mapping to account_id exists
        EvmAddresses::<T>::get(account_id).or_else(|| {
            let data: &[u8] = account_id.into_ref().as_ref();
            // Return the underlying EVM address if it exists otherwise return None
            if data.starts_with(b"evm:") {
                Some(EvmAddress::from_slice(&data[4..24]))
            } else {
                None
            }
        })
    }

    // Returns the EVM address associated with an account ID and generates an
    // account mapping if no association exists.
    fn get_or_create_evm_address(account_id: &T::AccountId) -> EvmAddress {
        Self::get_evm_address(account_id).unwrap_or_else(|| {
            let addr = account_to_default_evm_address(account_id);

            // create reverse mapping
            Accounts::<T>::insert(&addr, &account_id);
            EvmAddresses::<T>::insert(&account_id, &addr);

            addr
        })
    }

    fn get_default_evm_address(account_id: &T::AccountId) -> EvmAddress {
        account_to_default_evm_address(account_id)
    }

    // Returns true if a given AccountId is associated with a given EvmAddress
    // and false if is not.
    fn is_linked(account_id: &T::AccountId, evm: &EvmAddress) -> bool {
        Self::get_evm_address(account_id).as_ref() == Some(evm)
            || &account_to_default_evm_address(account_id.into_ref()) == evm
    }
}

impl<T: Config> pallet_evm::AddressMapping<T::AccountId> for EvmAddressMapping<T>
where
    T::AccountId: IsType<AccountId32>,
{
    fn into_account_id(address: H160) -> T::AccountId {
        Self::get_account_id(&address)
    }
}

pub struct CallKillAccount<T>(PhantomData<T>);
impl<T: Config> OnKilledAccount<T::AccountId> for CallKillAccount<T> {
    fn on_killed_account(who: &T::AccountId) {
        // remove the reserve mapping that could be created by
        // `get_or_create_evm_address`
        Accounts::<T>::remove(account_to_default_evm_address(who.into_ref()));

        // remove mapping created by `claim_account`
        if let Some(evm_addr) = Module::<T>::evm_addresses(who) {
            Accounts::<T>::remove(evm_addr);
            EvmAddresses::<T>::remove(who);
        }
    }
}

impl<T: Config> StaticLookup for Pallet<T> {
    type Source = MultiAddress<T::AccountId, AccountIndex>;
    type Target = T::AccountId;

    fn lookup(a: Self::Source) -> Result<Self::Target, LookupError> {
        match a {
            MultiAddress::Address20(i) => Ok(T::EvmAddressMapping::get_account_id(
                &EvmAddress::from_slice(&i),
            )),
            _ => Err(LookupError),
        }
    }

    fn unlookup(a: Self::Target) -> Self::Source {
        MultiAddress::Id(a)
    }
}

/// Converts the given binary data into ASCII-encoded hex. It will be twice
/// the length.
pub fn to_ascii_hex(data: &[u8]) -> Vec<u8> {
    let mut r = Vec::with_capacity(data.len() * 2);
    let mut push_nibble = |n| r.push(if n < 10 { b'0' + n } else { b'a' - 10 + n });
    for &b in data.iter() {
        push_nibble(b / 16);
        push_nibble(b % 16);
    }
    r
}
