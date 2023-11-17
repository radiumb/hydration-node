// Copyright (C) 2020-2023  Intergalactic, Limited (GIB).
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Referrals pallet
//!

#![cfg_attr(not(feature = "std"), no_std)]

mod weights;

#[cfg(test)]
mod tests;

use frame_support::pallet_prelude::{DispatchResult, Get};
use frame_system::{ensure_signed, pallet_prelude::OriginFor};
use sp_core::bounded::BoundedVec;

pub use pallet::*;

use weights::WeightInfo;

pub type ReferralCode<S> = BoundedVec<u8, S>;

const MIN_CODE_LENGTH: usize = 3;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(crate) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Maximuem referrral code length.
		type CodeLength: Get<u32>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	/// Referral codes
	/// Maps an account to a referral code.
	#[pallet::getter(fn referral_account)]
	pub(super) type ReferralCodes<T: Config> =
		StorageMap<_, Blake2_128Concat, ReferralCode<T::CodeLength>, T::AccountId>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		CodeRegistered {
			code: ReferralCode<T::CodeLength>,
			account: T::AccountId,
		},
	}

	#[pallet::error]
	#[cfg_attr(test, derive(PartialEq, Eq))]
	pub enum Error<T> {
		TooLong,
		TooShort,
		InvalidCharacter,
		AlreadyExists,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register new referral code.
		///
		/// `origin` pays the registration fee.
		/// `code` is assigned to the given `account`.
		///
		/// Length of the `code` must be at least `MIN_CODE_LENGTH`.
		/// Maximum length is limited to `T::CodeLength`.
		/// `code` must contain only alfa-numeric characters and all characters will be converted to upper case.
		///
		/// /// Parameters:
		/// - `origin`:
		/// - `code`:
		/// - `account`:
		///
		/// Emits `CodeRegistered` event when successful.
		///
		#[pallet::call_index(0)]
		#[pallet::weight(<T as Config>::WeightInfo::register_code())]
		pub fn register_code(origin: OriginFor<T>, code: Vec<u8>, account: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let code: ReferralCode<T::CodeLength> = code.try_into().map_err(|_| Error::<T>::TooLong)?;

			ensure!(code.len() >= MIN_CODE_LENGTH, Error::<T>::TooShort);

			//TODO: can we do without cloning ?
			ensure!(
				code.clone()
					.into_inner()
					.iter()
					.all(|c| char::is_alphanumeric(*c as char)),
				Error::<T>::InvalidCharacter
			);

			ReferralCodes::<T>::mutate(code.clone(), |v| -> DispatchResult {
				ensure!(v.is_none(), Error::<T>::AlreadyExists);
				*v = Some(account.clone());
				Self::deposit_event(Event::CodeRegistered { code, account });
				Ok(())
			})
		}
	}
}

impl<T: Config> Pallet<T> {}
