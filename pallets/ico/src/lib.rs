// Copyright 2021 DICO  Developer.
// This file is part of DICO

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

#![allow(unused_imports)]
#![allow(unused_parens)]
#![allow(unused_doc_comments)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(unused_must_use)]
#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

use crate::pallet::*;
pub use crate::pallet::*;
use codec::{Decode, Encode};
use currencies::{currencies_trait::CurrenciesHandler, DicoAssetInfo, DicoAssetMetadata};
pub use dico_primitives::{currency::*, time::*};
pub use dico_primitives::{AssetId, CurrencyId};
use dico_primitives::{Balance, Price};
use dico_treasury::traits::DicoTreasuryHandler;
use frame_support::traits::ExistenceRequirement;
pub use frame_support::{
	debug, decl_error, decl_event, decl_module, decl_storage, ensure, runtime_print,
	traits::{
		BalanceStatus as Status, Currency, EnsureOrigin,
		ExistenceRequirement::{AllowDeath, KeepAlive},
		Get, LockIdentifier, OnUnbalanced, ReservableCurrency, WithdrawReasons,
	},
	transactional,
	weights::Weight,
	Blake2_256, IterableStorageDoubleMap, IterableStorageMap,
};
pub use frame_system::{self as system, ensure_none, ensure_root, ensure_signed};
use orml_tokens::BalanceLock;
use orml_tokens::{self as tokens, Locks};
use orml_traits::{BalanceStatus, MultiCurrency, MultiReservableCurrency};
use pallet_kyc::{traits::KycHandler, types::AreaCode};
use pallet_pricedao::traits::PriceData;
pub use primitive_types::U256;
use sp_runtime::app_crypto::sp_core::sandbox::ERR_EXECUTION;
use sp_runtime::traits::{CheckedAdd, CheckedSub, StaticLookup};
pub use sp_runtime::{
	traits::CheckedMul,
	traits::{AccountIdConversion, CheckedDiv, Saturating, Zero},
	DispatchError, DispatchResult, Percent, RuntimeDebug, SaturatedConversion,
};
pub use sp_std::convert::{Into, TryFrom, TryInto};
use sp_std::vec::Vec;
use sp_std::{collections::btree_map::BTreeMap, prelude::*, result};
use traits::{IcoHandler, PowerHandler};
use scale_info::TypeInfo;
use weights::WeightInfo;

pub mod mock;
pub mod tests;
pub mod traits;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
const ICO_ID: LockIdentifier = *b"ico     ";
const HalfDuration: u128 = 200_000_000u128 * USD;

#[derive(PartialEq, Encode, Decode, RuntimeDebug, Clone, TypeInfo)]
pub enum IcoStatus {
	Checking,
	Failed,
	Passed,
}

impl Default for IcoStatus {
	fn default() -> Self {
		IcoStatus::Checking
	}
}

#[derive(PartialEq, Encode, Decode, RuntimeDebug, Clone, TypeInfo)]
pub struct InviteInfo<AccountId, Balance, AssetId> {
	inviter: AccountId,
	invitee: AccountId,
	currency_id: AssetId,
	index: u32,
	reward: Option<Balance>,
}

#[derive(PartialEq, Encode, Decode, RuntimeDebug, Clone, TypeInfo)]
pub struct InitiatedIco<CurrencyId, Status, Balance> {
	desc: Vec<u8>,
	currency_id: CurrencyId,
	token_symbol: Vec<u8>,
	decimals: u8,
	index: u32,
	status: Status,
	amount: Balance,
}

#[derive(PartialEq, Eq, Encode, Decode, Default, RuntimeDebug, Clone, TypeInfo)]
pub struct Release<AccountId, Block, CurrencyId, NativeBalance> {
	/// who requests the token release
	who: AccountId,
	currency_id: CurrencyId,
	/// the ico unique id
	index: u32,
	/// When do requests start
	request_time: Block,
	/// What percentage to release
	percent: Percent,
	pledge: NativeBalance,
}

#[derive(PartialEq, Encode, Decode, Default, RuntimeDebug, Clone, TypeInfo)]
pub struct IcoLock<Balance, BlockNumber> {
	/// When does the lockup start
	start_block: BlockNumber,
	/// the ico unique id
	index: u32,
	total_amount: Balance,
	/// the amount that have unlocked
	unlock_amount: Balance,
	unlock_duration: BlockNumber,
	/// How often to unlock
	per_duration_unlock_amount: Balance,
}

#[derive(PartialEq, Encode, Decode, Default, RuntimeDebug, Clone, TypeInfo)]
pub struct IcoParameters<BlockNumber, Balance, CurrencyId, AreaCode> {
	pub desc: Vec<u8>,
	/// The asset ID of the project's token
	pub currency_id: CurrencyId,
	/// The project's official website
	pub official_website: Vec<u8>,
	pub is_must_kyc: bool,
	/// Maximum number of times a user can participate in an ICO
	pub user_ico_max_times: u8,
	/// Total issuance of the project's tokens
	pub total_issuance: Balance,
	/// The total number of tokens in circulation
	pub total_circulation: Balance,
	/// The lifetime of an ICO
	pub ico_duration: BlockNumber,
	/// How many tokens will the project party take out to participate in the
	/// ICO
	pub total_ico_amount: Balance,
	/// Minimum number of USdT users can participate in.
	pub user_min_amount: Balance,
	/// Maximum number of USdT users can participate in.
	pub user_max_amount: Balance,
	/// The asset_id of token required by the initiator
	pub exchange_token: CurrencyId,
	/// The total amount of token required by the initiator
	pub exchange_token_total_amount: Balance,
	/// AreaCode these not participate in the ico
	pub exclude_area: Vec<AreaCode>,
	/// The proportion of lock-ups required
	pub lock_proportion: Percent,
	/// How often to unlock
	pub unlock_duration: BlockNumber,
	pub per_duration_unlock_amount: Balance,
}

#[derive(PartialEq, Encode, Decode, Default, RuntimeDebug, Clone, TypeInfo)]
pub struct UnRelease<MultiBalanceOf, CurrencyIdOf, AccountId> {
	currency_id: CurrencyIdOf,
	inviter: Option<AccountId>,
	index: u32,
	/// the asset_id that it has not been released
	unreleased_currency_id: CurrencyIdOf,
	/// Total value of the token participating in an ICO
	total_usdt: MultiBalanceOf,
	/// Record of ico participation
	tags: Vec<(MultiBalanceOf, MultiBalanceOf, MultiBalanceOf, MultiBalanceOf)>,
	/// The total number of the token received by ico.
	total: MultiBalanceOf,
	released: MultiBalanceOf,
	refund: MultiBalanceOf,
	reward: Option<MultiBalanceOf>,
}

#[derive(PartialEq, Encode, Decode, RuntimeDebug, Clone, TypeInfo)]
pub struct PendingInfo<IcoInfo, Balance> {
	pub ico: IcoInfo,
	pub pledge_dico: Balance,
	pub pledge_exchange_token: Balance,
}

#[derive(PartialEq, Encode, Decode, Eq, RuntimeDebug, Clone, TypeInfo)]
pub struct PowerMultiple {
	pub up: u32,
	pub down: u32,
}

impl Default for PowerMultiple {
	fn default() -> Self {
		PowerMultiple { up: 10, down: 10 }
	}
}

#[derive(PartialEq, Encode, Decode, Default, RuntimeDebug, Clone, TypeInfo)]
pub struct IcoInfo<BlockNumber, Balance, CurrencyId, AreaCode, AccountId> {
	desc: Vec<u8>,
	/// The block numbers that start ico
	start_time: Option<BlockNumber>,
	is_already_kyc: bool,
	initiator: AccountId,
	/// Total value of the token participating in an ICO
	total_usdt: Balance,
	tag: Option<Balance>,
	is_terminated: bool,
	project_name: Vec<u8>,
	token_symbol: Vec<u8>,
	decimals: u8,
	/// the ico unique id
	index: Option<u32>,
	/// The proportion that has been allowed to be released
	already_released_proportion: Percent,
	/// The asset ID of the project's token
	currency_id: CurrencyId,
	/// The project's official website
	official_website: Vec<u8>,
	/// Maximum number of times a user can participate in an ICO
	user_ico_max_times: u8,
	is_must_kyc: bool,
	/// Total issuance of the project's tokens
	total_issuance: Balance,
	/// The total number of tokens in circulation
	total_circulation: Balance,
	/// The lifetime of an ICO
	ico_duration: BlockNumber,
	/// How many tokens will the project party take out to participate in the
	/// ICO
	total_ico_amount: Balance,
	/// Minimum number of USdT users can participate in.
	user_min_amount: Balance,
	/// Maximum number of USdT users can participate in.
	user_max_amount: Balance,
	/// The asset_id of token required by the initiator.
	exchange_token: CurrencyId,
	/// The total amount of token required by the initiator.
	exchange_token_total_amount: Balance,
	/// AreaCode these not participate in the ico.
	exclude_area: Vec<AreaCode>,
	/// The proportion of lock-ups required.
	lock_proportion: Percent,
	/// How often to unlock.
	unlock_duration: BlockNumber,
	per_duration_unlock_amount: Balance,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::{
		Blake2_128Concat, IsType, OptionQuery, StorageDoubleMap, StorageMap, StorageValue, ValueQuery,
	};
	use frame_support::traits::Hooks;
	use frame_system::pallet_prelude::*;

	pub(crate) type MultiBalanceOf<T> =
		<<T as Config>::MultiCurrency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
	type BalanceOf<T> = <<T as Config>::NativeCurrency as Currency<<T as system::Config>::AccountId>>::Balance;
	type NegativeImbalanceOf<T> =
		<<T as Config>::NativeCurrency as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

	#[pallet::config]
	#[pallet::disable_frame_system_supertrait_check]
	pub trait Config: system::Config + tokens::Config {
		type Event: From<Event<Self>>
			+ Into<<Self as system::Config>::Event>
			+ IsType<<Self as frame_system::Config>::Event>;
		type PermitIcoOrigin: EnsureOrigin<Self::Origin>;

		type RejectIcoOrigin: EnsureOrigin<Self::Origin>;

		type PermitReleaseOrigin: EnsureOrigin<Self::Origin>;

		type TerminateIcoOrigin: EnsureOrigin<Self::Origin>;

		type OnSlash: OnUnbalanced<NegativeImbalanceOf<Self>>;

		type MultiCurrency: MultiCurrency<Self::AccountId, CurrencyId = AssetId>
			+ MultiReservableCurrency<Self::AccountId>;

		type NativeCurrency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		type CurrenciesHandler: CurrenciesHandler<
			AssetId,
			DicoAssetMetadata,
			DispatchError,
			Self::AccountId,
			MultiBalanceOf<Self>,
			sp_runtime::DispatchResult,
		>;
		type DicoTreasuryHandler: DicoTreasuryHandler<Self::AccountId>;

		type PriceData: PriceData<AssetId, Price = Balance>;

		type KycHandler: KycHandler<Self::AccountId, AreaCode>;

		type WeightInfo: WeightInfo;

		type GetNativeCurrencyId: Get<AssetId>;
		#[pallet::constant]
		type InitiatorPledge: Get<MultiBalanceOf<Self>>;
		#[pallet::constant]
		type InitiatorBond: Get<Percent>;
		#[pallet::constant]
		type RequestPledge: Get<BalanceOf<Self>>;
		#[pallet::constant]
		type RequestExpire: Get<Self::BlockNumber>;
		#[pallet::constant]
		type IcoTotalReward: Get<MultiBalanceOf<Self>>;
		#[pallet::constant]
		type TerminateProtectPeriod: Get<Percent>;
		#[pallet::constant]
		type ReleaseProtectPeriod: Get<Percent>;
		#[pallet::constant]
		type ChillDuration: Get<Self::BlockNumber>;
		#[pallet::constant]
		type InviterRewardProportion: Get<Percent>;
		#[pallet::constant]
		type InviteeRewardProportion: Get<Percent>;
		#[pallet::constant]
		type USDCurrencyId: Get<AssetId>;
	}

	#[pallet::storage]
	#[pallet::getter(fn pending_ico)]
	pub type PendingIco<T: Config> = StorageValue<
		_,
		Vec<
			PendingInfo<IcoInfo<T::BlockNumber, MultiBalanceOf<T>, AssetId, AreaCode, T::AccountId>, MultiBalanceOf<T>>,
		>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn ico)]
	pub type Ico<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		AssetId,
		Blake2_128Concat,
		u32,
		IcoInfo<T::BlockNumber, MultiBalanceOf<T>, AssetId, AreaCode, T::AccountId>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn ico_assets_info)]
	pub type UnReleaseAssets<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Vec<UnRelease<MultiBalanceOf<T>, AssetId, T::AccountId>>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn release_info)]
	pub type RequestReleaseInfo<T: Config> =
		StorageValue<_, Vec<Release<T::AccountId, T::BlockNumber, AssetId, BalanceOf<T>>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn locks)]
	pub type IcoLocks<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		AssetId,
		Vec<IcoLock<MultiBalanceOf<T>, T::BlockNumber>>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn total_usdt)]
	pub type TotalUsdt<T: Config> = StorageValue<_, MultiBalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total_num)]
	pub type TotalNum<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn indexes)]
	pub type Indexs<T: Config> = StorageMap<_, Blake2_128Concat, AssetId, Vec<u32>, ValueQuery>;

	#[pallet::storage]
	pub type IsUnservePledge<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, AssetId, Blake2_128Concat, u32, bool, ValueQuery>;

	#[pallet::storage]
	pub type IcoMinUsdtAmount<T: Config> = StorageValue<_, MultiBalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	pub type IcoMaxUsdtAmount<T: Config> = StorageValue<_, MultiBalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn initiate_icoes_of)]
	pub type InitiatedIcoesOf<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Vec<InitiatedIco<AssetId, IcoStatus, MultiBalanceOf<T>>>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn icoes_of)]
	pub type IcoesOf<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<(AssetId, u32)>, ValueQuery>;

	#[pallet::storage]
	pub type PassedIcoes<T: Config> = StorageValue<_, Vec<(AssetId, u32)>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn power_multiple_of)]
	pub type PowerMultipleOf<T: Config> = StorageMap<_, Blake2_128Concat, AssetId, PowerMultiple, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total_power_of)]
	pub type TotalPowerOf<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, MultiBalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn invite_info_of)]
	pub type InviteInfoOf<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		AssetId,
		Vec<InviteInfo<T::AccountId, MultiBalanceOf<T>, AssetId>>,
		ValueQuery,
	>;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// The project party initiates an ICO
		#[pallet::weight(<T as pallet::Config>::WeightInfo::initiate_ico())]
		pub fn initiate_ico(
			origin: OriginFor<T>,
			info: IcoParameters<T::BlockNumber, MultiBalanceOf<T>, AssetId, AreaCode>,
		) -> DispatchResult {
			let initiator = ensure_signed(origin)?;

			Self::initiate_ico_simple_check(&info, &initiator)?;
			let exchange_token_metadata = T::CurrenciesHandler::get_metadata(info.exchange_token)?;
			let metadata = T::CurrenciesHandler::get_metadata(info.currency_id)?;

			let bond = T::InitiatorBond::get() * info.exchange_token_total_amount;
			let total_num = <TotalNum<T>>::get().checked_add(1u32).ok_or(Error::<T>::Overflow)?;
			T::MultiCurrency::reserve(T::GetNativeCurrencyId::get(), &initiator, T::InitiatorPledge::get())?;
			T::MultiCurrency::reserve(info.exchange_token, &initiator, bond);
			T::MultiCurrency::reserve(info.currency_id, &initiator, info.total_ico_amount)?;

			<PendingIco<T>>::mutate(|h| {
				h.push(PendingInfo {
					ico: IcoInfo {
						desc: info.desc.clone(),
						start_time: None,
						is_already_kyc: Self::is_already_kyc(&initiator),
						initiator: initiator.clone(),
						total_usdt: MultiBalanceOf::<T>::from(0u32),
						tag: None,
						is_terminated: false,
						project_name: metadata.name.clone(),
						token_symbol: metadata.symbol.clone(),
						decimals: metadata.decimals,
						index: Some(total_num),
						already_released_proportion: Percent::from_percent(0u8),
						currency_id: info.currency_id,
						official_website: info.official_website.clone(),
						is_must_kyc: info.is_must_kyc,
						user_ico_max_times: info.user_ico_max_times,
						total_issuance: info.total_issuance,
						total_circulation: info.total_circulation,
						ico_duration: info.ico_duration,
						total_ico_amount: info.total_ico_amount,
						user_min_amount: info.user_min_amount,
						user_max_amount: info.user_max_amount,
						exchange_token: info.exchange_token,
						exchange_token_total_amount: info.exchange_token_total_amount,
						exclude_area: info.exclude_area.clone(),
						lock_proportion: info.lock_proportion,
						unlock_duration: info.unlock_duration,
						per_duration_unlock_amount: info.per_duration_unlock_amount,
					},
					pledge_dico: T::InitiatorPledge::get(),
					pledge_exchange_token: bond,
				})
			});

			<TotalNum<T>>::put(total_num);

			InitiatedIcoesOf::<T>::mutate(&initiator, |h| {
				h.push(InitiatedIco {
					currency_id: info.currency_id,
					token_symbol: metadata.symbol,
					decimals: metadata.decimals,
					index: total_num,
					status: IcoStatus::Checking,
					desc: info.desc.clone(),
					amount: info.total_ico_amount,
				})
			});

			Self::deposit_event(Event::InitiateIco(
				initiator,
				info.exchange_token,
				info.total_ico_amount,
			));
			Ok(())
		}

		/// The foundation agrees to the ICO of the project party
		#[pallet::weight(<T as pallet::Config>::WeightInfo::permit_ico())]
		pub fn permit_ico(origin: OriginFor<T>, currency_id: AssetId) -> DispatchResult {
			T::PermitIcoOrigin::try_origin(origin).map_err(|_| Error::<T>::BadOrigin)?;

			let mut pending_ico = <PendingIco<T>>::get();
			let pos_opt = pending_ico.iter().position(|h| currency_id == h.ico.currency_id);

			match pos_opt {
				None => return Err(Error::<T>::PendingIcoNotExists123)?,
				Some(pos) => {
					let mut pending_info = pending_ico.swap_remove(pos);

					let bond_amount = pending_info.pledge_exchange_token;

					T::MultiCurrency::repatriate_reserved(
						pending_info.ico.exchange_token,
						&pending_info.ico.initiator,
						&T::DicoTreasuryHandler::get_treasury_account_id(),
						bond_amount,
						BalanceStatus::Free,
					);
					T::MultiCurrency::unreserve(
						T::GetNativeCurrencyId::get(),
						&pending_info.ico.initiator,
						pending_info.pledge_dico,
					);
					<PendingIco<T>>::put(pending_ico);
					pending_info.ico.start_time = Some(
						Self::now()
							.checked_add(&T::ChillDuration::get())
							.ok_or(Error::<T>::Overflow)?,
					);

					let mut index: u32 = 0;
					match pending_info.ico.index {
						None => Err(Error::<T>::IcoIndexNotExists)?,
						Some(x) => index = x,
					}
					<Ico<T>>::insert(&pending_info.ico.currency_id, index, &pending_info.ico);
					Indexs::<T>::mutate(currency_id, |h| h.push(index));
					IsUnservePledge::<T>::insert(currency_id, index, true);
					Self::update_initiate_icoes(currency_id, index, &pending_info.ico.initiator, IcoStatus::Passed);
					PassedIcoes::<T>::mutate(|h| h.push((currency_id, index)));

					Self::deposit_event(Event::PermitIco(
						pending_info.ico.initiator,
						pending_info.ico.currency_id,
					));
				}
			}
			Ok(())
		}

		/// The foundation opposes the ICO of the project party
		#[pallet::weight(<T as pallet::Config>::WeightInfo::reject_ico())]
		pub fn reject_ico(origin: OriginFor<T>, currency_id: AssetId) -> DispatchResult {
			T::RejectIcoOrigin::try_origin(origin).map_err(|_| Error::<T>::BadOrigin)?;

			let mut pending_ico = <PendingIco<T>>::get();
			let pos_opt = pending_ico.iter().position(|h| currency_id == h.ico.currency_id);

			match pos_opt {
				None => return Err(Error::<T>::PendingIcoNotExists123)?,
				Some(pos) => {
					let pending_info = pending_ico.swap_remove(pos);
					let mut index: u32 = 0;
					match pending_info.ico.index {
						None => Err(Error::<T>::IcoIndexNotExists)?,
						Some(x) => index = x,
					}
					T::MultiCurrency::unreserve(
						pending_info.ico.exchange_token,
						&pending_info.ico.initiator,
						pending_info.pledge_exchange_token,
					);
					T::MultiCurrency::slash_reserved(
						T::GetNativeCurrencyId::get(),
						&pending_info.ico.initiator,
						pending_info.pledge_dico,
					);
					T::MultiCurrency::unreserve(
						pending_info.ico.currency_id,
						&pending_info.ico.initiator,
						pending_info.ico.total_ico_amount,
					);
					<PendingIco<T>>::put(pending_ico);
					Self::update_initiate_icoes(currency_id, index, &pending_info.ico.initiator, IcoStatus::Failed);
					Self::deposit_event(Event::RejectIco(
						pending_info.ico.initiator,
						pending_info.ico.currency_id,
					));
				}
			}
			Ok(())
		}

		/// User participation in ICO
		#[pallet::weight(<T as pallet::Config>::WeightInfo::join())]
		#[transactional]
		pub fn join(
			origin: OriginFor<T>,
			currency_id: AssetId,
			index: u32,
			amount: MultiBalanceOf<T>,
			inviter: Option<<T::Lookup as StaticLookup>::Source>,
		) -> DispatchResult {
			let user = ensure_signed(origin)?;

			let inviter = match inviter {
				None => None,
				Some(x) => Some(T::Lookup::lookup(x)?),
			};

			if let Some(inv) = inviter.as_ref() {
				ensure!(&user != inv, Error::<T>::CanNotInviteYouself);
				ensure!(
					Self::is_member(inv, currency_id, index, false),
					Error::<T>::InviterNotInIco
				);
			}

			let mut ico = <Ico<T>>::get(currency_id, index).ok_or(Error::<T>::IcoNotExists123)?;
			let exchange_token_decimals = T::CurrenciesHandler::get_metadata(ico.exchange_token)?.decimals;
			let mut total_usdt =
				Self::exchange_token_convert_usdt(ico.exchange_token, exchange_token_decimals, amount)?;
			ensure!(total_usdt != MultiBalanceOf::<T>::from(0u32), Error::<T>::PowerIsZero);
			let new_total_usdt = ico.total_usdt.checked_add(&total_usdt).ok_or(Error::<T>::Overflow)?;

			Self::join_simple_check(&ico, amount, ico.currency_id, index, &user, total_usdt)?;

			let user_exchange_amount = Self::swap(&user, amount, &ico)?;

			Self::insert_ico_assets_info(&user, &ico, amount, total_usdt, &inviter);
			Self::update_user_icoes(&user, currency_id, index);
			// Must be at the end
			ico.total_usdt = new_total_usdt;
			Ico::<T>::insert(currency_id, index, &ico);
			Self::add_user_power(&user, total_usdt)?;
			Self::insert_invite_info(&user, inviter, currency_id, index);

			Self::deposit_event(Event::Join(user, currency_id, index, amount, user_exchange_amount));
			Ok(())
		}

		/// DAO terminate the ico
		#[pallet::weight(<T as pallet::Config>::WeightInfo::terminate_ico())]
		pub fn terminate_ico(origin: OriginFor<T>, currency_id: AssetId, index: u32) -> DispatchResult {
			T::TerminateIcoOrigin::try_origin(origin).map_err(|_| Error::<T>::BadOrigin)?;

			let mut ico = <Ico<T>>::get(currency_id, index).ok_or(Error::<T>::IcoNotExists123)?;
			ensure!(!ico.is_terminated, Error::<T>::IcoTerminated);

			match ico.start_time.as_ref() {
				Some(time) => {
					#[cfg(test)]
					println!(
						"time: {:?}, now:{:?}",
						time.saturating_add(T::TerminateProtectPeriod::get() * ico.ico_duration),
						Self::now()
					);
					ensure!(
						time.saturating_add(T::TerminateProtectPeriod::get() * ico.ico_duration) < Self::now(),
						Error::<T>::TerminateProtectTime
					);
				}
				None => Err(Error::<T>::StartTimeNotExists)?,
			}

			ico.is_terminated = true;
			<Ico<T>>::insert(currency_id, index, ico);
			Self::remove_passed_icoes(currency_id, index);

			Self::deposit_event(Event::TerminateIco(currency_id, index));
			Ok(())
		}

		/// The project party requests the release of the funds
		#[pallet::weight(<T as pallet::Config>::WeightInfo::request_release())]
		pub fn request_release(
			origin: OriginFor<T>,
			currency_id: AssetId,
			index: u32,
			percent: Percent,
		) -> DispatchResult {
			let initiator = ensure_signed(origin)?;

			let ico = <Ico<T>>::get(currency_id, index).ok_or(Error::<T>::IcoNotExists123)?;
			match ico.start_time.as_ref() {
				Some(time) => {
					#[cfg(test)]
					println!(
						"time: {:?}, now:{:?}",
						time.saturating_add(T::ReleaseProtectPeriod::get() * ico.ico_duration),
						Self::now()
					);
					ensure!(
						time.saturating_add(T::ReleaseProtectPeriod::get() * ico.ico_duration) < Self::now()
							|| ico.is_terminated,
						Error::<T>::ReleaseProtectTime
					);
				}
				None => Err(Error::<T>::StartTimeNotExists)?,
			}
			ensure!(ico.already_released_proportion < percent, Error::<T>::ProportionTooLow);
			ensure!(&initiator == &ico.initiator, Error::<T>::NotInitiator);
			ensure!(!ico.is_terminated, Error::<T>::IcoTerminated);
			ensure!(
				Self::get_request_release_info(currency_id, index).is_none(),
				Error::<T>::AlreadyRequest
			);

			T::NativeCurrency::reserve(&initiator, T::RequestPledge::get())?;

			<RequestReleaseInfo<T>>::mutate(|h| {
				h.push(Release {
					who: initiator,
					currency_id: currency_id,
					index: index,
					request_time: Self::now(),
					percent: percent,
					pledge: T::RequestPledge::get(),
				})
			});

			Self::deposit_event(Event::RequestRelease(currency_id, index, percent));
			Ok(())
		}

		/// The project party cancels the request for release of funds.
		#[pallet::weight(<T as pallet::Config>::WeightInfo::cancel_request())]
		pub fn cancel_request(origin: OriginFor<T>, currency_id: AssetId, index: u32) -> DispatchResult {
			let initiator = ensure_signed(origin)?;

			let ico = <Ico<T>>::get(currency_id, index).ok_or(Error::<T>::IcoNotExists123)?;

			ensure!(&initiator == &ico.initiator, Error::<T>::NotInitiator);

			let release_info_opt = Self::get_request_release_info(currency_id, index);
			match release_info_opt {
				None => Err(Error::<T>::RequestNotExists)?,
				Some(release_info) => {
					let slash = release_info.pledge / <BalanceOf<T>>::from(2u32);
					let unreserve = release_info.pledge.saturating_sub(slash);
					T::OnSlash::on_unbalanced(T::NativeCurrency::slash_reserved(&initiator, slash).0);
					T::NativeCurrency::unreserve(&initiator, unreserve);
					Self::remove_request_release_info(Some(currency_id), index, false);
				}
			}

			Self::deposit_event(Event::CancelRequest(currency_id, index));
			Ok(())
		}

		/// DAO allow asset release
		#[pallet::weight(<T as pallet::Config>::WeightInfo::permit_release())]
		pub fn permit_release(origin: OriginFor<T>, currency_id: AssetId, index: u32) -> DispatchResult {
			T::PermitReleaseOrigin::try_origin(origin).map_err(|_| Error::<T>::BadOrigin)?;

			let mut ico = <Ico<T>>::get(currency_id, index).ok_or(Error::<T>::IcoNotExists123)?;
			let release_info_opt = Self::get_request_release_info(currency_id, index);

			match release_info_opt {
				None => Err(Error::<T>::RequestNotExists)?,
				Some(release_info) => {
					ensure!(!ico.is_terminated, Error::<T>::IcoTerminated);
					ensure!(
						release_info.percent > ico.already_released_proportion,
						Error::<T>::ProportionTooLow
					);

					ico.already_released_proportion = release_info.percent;
					Self::remove_request_release_info(Some(currency_id), index, false);

					T::NativeCurrency::unreserve(&ico.initiator, release_info.pledge);

					Ico::<T>::insert(currency_id, index, ico);
					Self::deposit_event(Event::PermitRelease(currency_id, index, release_info));
				}
			}
			Ok(())
		}

		/// Users release their own asset.
		#[pallet::weight(<T as pallet::Config>::WeightInfo::user_release_ico_amount())]
		pub fn user_release_ico_amount(origin: OriginFor<T>, currency_id: AssetId, index: u32) -> DispatchResult {
			let user = ensure_signed(origin)?;

			let _ = Self::do_release(user, currency_id, index, true)?;
			Ok(())
		}

		/// Users unlock their funds.
		#[pallet::weight(<T as pallet::Config>::WeightInfo::unlock())]
		pub fn unlock(origin: OriginFor<T>, currency_id: AssetId, index: u32) -> DispatchResult {
			let user = ensure_signed(origin)?;

			ensure!(
				<IcoLocks<T>>::contains_key(&user, &currency_id),
				Error::<T>::LockIsEmpty
			);

			<IcoLocks<T>>::try_mutate(&user, &currency_id, |h| {
				let (total, locks) = Self::unlock_asset(&user, &currency_id, index, true, h);
				if total == <MultiBalanceOf<T>>::from(0u32) {
					if cfg!(any(feature = "std", feature = "runtime-benchmarks", test)) {
						return Ok(());
					}
					return Err(Error::<T>::UnlockAmountIsZero);
				} else {
					*h = locks;
					Self::deposit_event(Event::UnlockAsset(currency_id, user.clone(), total));
					Ok(())
				}
			})?;
			Ok(())
		}

		/// The root sets the maximum and minimum ico amount.
		///
		/// This two values applies to all ICOs.
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_system_ico_amount_bound())]
		pub fn set_system_ico_amount_bound(
			origin: OriginFor<T>,
			min_amount: MultiBalanceOf<T>,
			max_amount: MultiBalanceOf<T>,
		) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(min_amount <= max_amount, Error::<T>::MaxLessThanMin);
			ensure!(max_amount != MultiBalanceOf::<T>::from(0u32), Error::<T>::MaxIsZero);

			IcoMinUsdtAmount::<T>::put(min_amount);
			IcoMaxUsdtAmount::<T>::put(max_amount);
			Self::deposit_event(Event::SetSystemIcoAmountBound(min_amount, max_amount));
			Ok(())
		}

		/// The initiator set the maximum and minimum ico amount.
		#[pallet::weight(<T as pallet::Config>::WeightInfo::initiator_set_ico_amount_bound())]
		pub fn initiator_set_ico_amount_bound(
			origin: OriginFor<T>,
			currency_id: AssetId,
			index: u32,
			min_amount: MultiBalanceOf<T>,
			max_amount: MultiBalanceOf<T>,
		) -> DispatchResult {
			let user = ensure_signed(origin)?;

			let mut ico = <Ico<T>>::get(currency_id, index).ok_or(Error::<T>::IcoNotExists123)?;

			ensure!(ico.initiator == user, Error::<T>::NotInitiator);

			Self::check_bound(min_amount, max_amount)?;

			ico.user_min_amount = min_amount;
			ico.user_max_amount = max_amount;
			Ico::<T>::insert(currency_id, index, ico);

			Self::deposit_event(Event::InitiatorSetIcoAmountBound(
				currency_id,
				index,
				min_amount,
				max_amount,
			));
			Ok(())
		}

		/// The initiator sets per user ico max times of him project.
		#[pallet::weight(<T as pallet::Config>::WeightInfo::initiator_set_ico_max_times())]
		pub fn initiator_set_ico_max_times(
			origin: OriginFor<T>,
			currency_id: AssetId,
			index: u32,
			max_times: u8,
		) -> DispatchResult {
			let user = ensure_signed(origin)?;

			let mut ico = <Ico<T>>::get(currency_id, index).ok_or(Error::<T>::IcoNotExists123)?;

			ensure!(ico.initiator == user, Error::<T>::NotInitiator);
			ensure!(ico.user_ico_max_times != max_times, Error::<T>::DuplicateSet);
			ensure!(max_times != 0u8, Error::<T>::MaxIsZero);

			ico.user_ico_max_times = max_times;
			Ico::<T>::insert(currency_id, index, ico);

			Self::deposit_event(Event::SetIcoMaxCount(currency_id, index, max_times));
			Ok(())
		}

		/// When the end of the ico, users get the reward.
		#[pallet::weight(<T as pallet::Config>::WeightInfo::get_reward())]
		pub fn get_reward(origin: OriginFor<T>, currency_id: AssetId, index: u32) -> DispatchResult {
			let user = ensure_signed(origin)?;

			let reward = Self::do_reward(&user, currency_id, index, true)?;

			Self::deposit_event(Event::GetReward(currency_id, index, user, reward));
			Ok(())
		}

		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_asset_power_multiple())]
		pub fn set_asset_power_multiple(
			origin: OriginFor<T>,
			currency_id: AssetId,
			multiple: PowerMultiple,
		) -> DispatchResult {
			ensure_root(origin)?;
			let old_multiple = PowerMultipleOf::<T>::get(currency_id);
			ensure!(multiple.down != 0 && multiple.up != 0, Error::<T>::DownIsZero);
			ensure!(old_multiple != multiple, Error::<T>::MultipleNotChange);
			PowerMultipleOf::<T>::insert(currency_id, &multiple);
			Self::deposit_event(Event::SetAssetPowerMultiple(currency_id, multiple));
			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(n: T::BlockNumber) {
			Self::remove_request_release_info(None, 0u32, true);
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		Test(T::AccountId),
		InitiateIco(T::AccountId, AssetId, MultiBalanceOf<T>),
		PermitIco(T::AccountId, AssetId),
		RejectIco(T::AccountId, AssetId),
		Join(T::AccountId, AssetId, u32, MultiBalanceOf<T>, MultiBalanceOf<T>),
		TerminateIco(AssetId, u32),
		RequestRelease(AssetId, u32, Percent),
		CancelRequest(AssetId, u32),
		PermitRelease(
			AssetId,
			u32,
			Release<<T as system::Config>::AccountId, <T as system::Config>::BlockNumber, AssetId, BalanceOf<T>>,
		),
		UnlockAsset(AssetId, T::AccountId, MultiBalanceOf<T>),
		GetReward(AssetId, u32, T::AccountId, MultiBalanceOf<T>),
		UserReleaseIcoAmount(AssetId, u32, MultiBalanceOf<T>),
		InitiatorSetIcoAmountBound(AssetId, u32, MultiBalanceOf<T>, MultiBalanceOf<T>),
		SetSystemIcoAmountBound(MultiBalanceOf<T>, MultiBalanceOf<T>),
		SetIcoMaxCount(AssetId, u32, u8),
		TerminatedGiveBackAmount(T::AccountId, AssetId, u32, MultiBalanceOf<T>),
		UnreservedInitiatorRemainPledgeAmount(AssetId, u32, MultiBalanceOf<T>),
		SetAssetPowerMultiple(AssetId, PowerMultiple),
	}

	#[pallet::error]
	pub enum Error<T> {
		BeingIco,
		PendingIcoNotExists123,
		IcoNotExists123,
		InExcludeArea,
		BadOrigin,
		IsPendingIco,
		InitiatorIsYourself,
		AmountIsZero,
		TokenShouldBeDifferent,
		DurationIsZero,
		IcoExpire,
		IcoNotExpireOrTerminated,
		IcoTerminated,
		NotInitiator,
		AlreadyRequest,
		RequestNotExists,
		RequestExpire,
		LockIsEmpty,
		UnlockAmountIsZero,
		NotIcoMember,
		Overflow,
		ProportionTooLow,
		UnreleaseAmountIsZero,
		RewardIsZero,
		IcoTimesToMax,
		NativeCurrencyId,
		UnreleasedAmountIsZero,
		AmountNotMeetProjectRequirement,
		AmountNotMeetSystemRequirement,
		MaxLessThanMin,
		MaxIsZero,
		DuplicateSet,
		CirculationMoreThanIssuance,
		IcoAmountMoreThanCirculation,
		MinAmountTooLow,
		MaxAmountTooLarge,
		MaxAmountIsZero,
		BalanceInsufficient,
		UserIcoAmountTooLow,
		UserIcoAmountToMax,
		ProjectIcoAmountToMax,
		DivByZero,
		IsNotStartIcoTime,
		InviterNotInIco,
		CanNotInviteYouself,
		TerminateProtectTime,
		ExchangeTokenBalanceTooLow,
		AlreadyGetReward,
		ReleaseProtectTime,
		NotKycUser,
		UserAreaNotExists,
		IcoIndexNotExists,
		StartTimeNotExists,
		MultipleNotChange,
		DownIsZero,
		PowerIsZero,
		PriceNotExists,
	}

	impl<T: Config> Pallet<T> {
		/// Whether or not project member(exclude project party）
		pub fn is_member(who: &T::AccountId, currency_id: AssetId, index: u32, is_exclude_manager: bool) -> bool {
			let mut unrelease_info_vec = UnReleaseAssets::<T>::get(&who);
			if let Some(pos) = unrelease_info_vec
				.iter()
				.position(|h| h.currency_id == currency_id && h.index == index)
			{
				let info = unrelease_info_vec.swap_remove(pos);
				if is_exclude_manager {
					if info.unreleased_currency_id != currency_id {
						return false;
					}
				}
				return true;
			} else {
				false
			}
		}

		fn do_reward(
			user: &T::AccountId,
			currency_id: AssetId,
			index: u32,
			is_do: bool,
		) -> result::Result<MultiBalanceOf<T>, DispatchError> {
			let mut ico = <Ico<T>>::get(currency_id, index).ok_or(Error::<T>::IcoNotExists123)?;
			ensure!(&ico.initiator != user, Error::<T>::InitiatorIsYourself);

			ensure!(Self::is_ico_expire(&ico), Error::<T>::IcoNotExpireOrTerminated);

			let total_reward = Self::calculate_total_reward(&ico);
			let asset_info =
				Self::get_unrelease_asset_info(&user, currency_id, index).ok_or(Error::<T>::NotIcoMember)?;
			ensure!(asset_info.reward.is_none(), Error::<T>::AlreadyGetReward);
			let tags = asset_info.tags;

			let classify = Self::split_user_amount(ico.total_usdt, tags);
			let mut reward = Self::caculate_user_reward(classify, ico.total_usdt, total_reward);
			ensure!(reward > MultiBalanceOf::<T>::from(0u32), Error::<T>::RewardIsZero);
			reward = Self::reward(&user, asset_info.inviter, reward, is_do, currency_id, index);
			if is_do {
				UnReleaseAssets::<T>::mutate(user.clone(), |vec| {
					if let Some(pos) = vec
						.iter()
						.position(|h| h.currency_id == currency_id && h.index == index)
					{
						let mut unreleased = vec.swap_remove(pos);
						unreleased.reward = Some(reward);
						vec.push(unreleased);
					}
				});

				if ico.tag == None {
					let mut new_usdt = TotalUsdt::<T>::get();
					new_usdt = new_usdt.saturating_add(ico.total_usdt);
					ico.tag = Some(new_usdt);
					Ico::<T>::insert(currency_id, index, ico);
					TotalUsdt::<T>::put(new_usdt);
				}
			}

			Ok(reward)
		}

		fn reward(
			user: &T::AccountId,
			inviter: Option<T::AccountId>,
			amount: MultiBalanceOf<T>,
			is_do: bool,
			currency_id: AssetId,
			index: u32,
		) -> MultiBalanceOf<T> {
			let mut user_total_amount = amount;
			if let Some(inviter) = inviter {
				let user_p = T::InviteeRewardProportion::get() * amount;
				user_total_amount.saturating_sub(user_p);
				if is_do {
					T::MultiCurrency::deposit(T::GetNativeCurrencyId::get(), &user, user_p);

					let inviter_reward = T::InviterRewardProportion::get() * amount;
					T::MultiCurrency::deposit(T::GetNativeCurrencyId::get(), &inviter, inviter_reward);
					Self::update_invite_info(&user, &inviter, currency_id, index, inviter_reward);
				}
			}
			if is_do {
				T::MultiCurrency::deposit(T::GetNativeCurrencyId::get(), &user, amount);
			}

			user_total_amount
		}

		fn is_already_kyc(who: &T::AccountId) -> bool {
			if T::KycHandler::get_user_area(who).is_none() {
				return false;
			}
			true
		}

		fn caculate_user_reward(
			info: Vec<(u32, MultiBalanceOf<T>)>,
			total_usdt: MultiBalanceOf<T>,
			total_reward: MultiBalanceOf<T>,
		) -> MultiBalanceOf<T> {
			let mut user_usdt = MultiBalanceOf::<T>::from(0u32);
			let tmpt = 50u32;
			for (n, amount) in info.iter() {
				if *n <= 5u32 {
					user_usdt += (*amount + (Percent::from_percent((50 - (n - 1) * 10) as u8) * *amount))
				} else if *n > 5 && *n <= 10 {
					user_usdt += (*amount - (Percent::from_percent(((n - 5) * 10) as u8) * *amount));
				}
			}

			Self::u256_convert_to_balance(
				Self::balance_convert_to_u256(user_usdt) * Self::balance_convert_to_u256(total_reward)
					/ Self::balance_convert_to_u256(total_usdt),
			)
		}

		fn update_invite_info(
			inviter: &T::AccountId,
			invitee: &T::AccountId,
			currency_id: AssetId,
			index: u32,
			reward: MultiBalanceOf<T>,
		) {
			let mut invitees = InviteInfoOf::<T>::get(&inviter, currency_id);
			if let Some(x) = invitees
				.iter()
				.position(|h| &h.invitee == invitee && h.currency_id == currency_id && h.index == index)
			{
				let mut info = invitees.swap_remove(x);
				if info.reward.is_none() {
					info.reward = Some(reward);
					invitees.push(info);
					InviteInfoOf::<T>::insert(&inviter, currency_id, invitees);
				}
			}
		}

		fn split_user_amount(
			total_amount: MultiBalanceOf<T>,
			info: Vec<(
				MultiBalanceOf<T>,
				MultiBalanceOf<T>,
				MultiBalanceOf<T>,
				MultiBalanceOf<T>,
			)>,
		) -> Vec<(u32, MultiBalanceOf<T>)> {
			const NUM: u32 = 10;
			let av_amount = total_amount / NUM.saturated_into::<MultiBalanceOf<T>>();
			let mut result: Vec<(u32, MultiBalanceOf<T>)> = vec![];

			for (amount, tag_amount, _, _) in info.iter() {
				let mut start_amount = tag_amount.saturating_sub(*amount);
				let mut n = (start_amount / av_amount).saturated_into::<u32>() + 1;
				loop {
					let next_amount = n.saturated_into::<MultiBalanceOf<T>>() * av_amount;
					if next_amount > *tag_amount {
						if tag_amount.saturating_sub(start_amount) > MultiBalanceOf::<T>::from(0u32) {
							result.push((n, tag_amount.saturating_sub(start_amount)));
						}
						break;
					}
					result.push((n, next_amount.saturating_sub(start_amount)));
					start_amount = next_amount;
					n += 1;
				}
			}

			result
		}

		pub fn calculate_total_reward(
			ico: &IcoInfo<T::BlockNumber, MultiBalanceOf<T>, AssetId, AreaCode, T::AccountId>,
		) -> MultiBalanceOf<T> {
			let mut this_time_total_usdt = ico.total_usdt;
			let total_usdt = match ico.tag {
				Some(tag_usdt) => tag_usdt,
				None => TotalUsdt::<T>::get().saturating_add(this_time_total_usdt),
			};

			let mut num = total_usdt / HalfDuration.saturated_into::<MultiBalanceOf<T>>();
			let mut remain = total_usdt % HalfDuration.saturated_into::<MultiBalanceOf<T>>();

			let mut power = MultiBalanceOf::<T>::from(0u32);
			loop {
				if this_time_total_usdt.saturating_sub(remain) > MultiBalanceOf::<T>::from(0u32) {
					power += (remain / (2u32.pow(num.saturated_into::<u32>())).saturated_into::<MultiBalanceOf<T>>());
					this_time_total_usdt = this_time_total_usdt.saturating_sub(remain);
					remain = HalfDuration.saturated_into::<MultiBalanceOf<T>>();
					num = num.saturating_sub(1u32.saturated_into::<MultiBalanceOf<T>>());
				} else {
					power += (this_time_total_usdt
						/ (2u32.pow(num.saturated_into::<u32>())).saturated_into::<MultiBalanceOf<T>>());
					break;
				}
			}

			let first_total_reward = T::IcoTotalReward::get() / 2u32.saturated_into::<MultiBalanceOf<T>>();

			Self::u256_convert_to_balance(
				Self::balance_convert_to_u256(power) * Self::balance_convert_to_u256(first_total_reward)
					/ Self::balance_convert_to_u256(HalfDuration.saturated_into::<MultiBalanceOf<T>>()),
			)
		}

		fn do_release(
			user: T::AccountId,
			currency_id: AssetId,
			index: u32,
			is_do: bool,
		) -> result::Result<MultiBalanceOf<T>, DispatchError> {
			let mut this_time_total_release = MultiBalanceOf::<T>::from(0u32);

			let mut is_oprate = false;
			let mut ico = <Ico<T>>::get(currency_id, index).ok_or(Error::<T>::IcoNotExists123)?;

			/// For initiator
			if Self::is_ico_expire(&ico)
				&& (IsUnservePledge::<T>::contains_key(currency_id, index)
					&& IsUnservePledge::<T>::get(currency_id, index))
			{
				if is_do {
					Self::unreserve_initiator_remain_token(&ico);
				}
				is_oprate = true;
			}

			/// For users
			let asset_info_opt = Self::get_unrelease_asset_info(&user, currency_id, index);
			match asset_info_opt {
				None => {
					if is_oprate && &user == &ico.initiator {
						return Ok(MultiBalanceOf::<T>::from(0u32));
					} else {
						return Err(Error::<T>::NotIcoMember)?;
					}
				}
				Some(asset_info) => {
					let total = asset_info.total;
					let released = asset_info.released;
					let refund = asset_info.refund;

					let should_released = ico.already_released_proportion * total;

					ensure!(
						(total.saturating_sub(released)).saturating_sub(refund) > MultiBalanceOf::<T>::from(0u32),
						Error::<T>::UnreleasedAmountIsZero
					);

					if ico.is_terminated
						&& &user != &ico.initiator
						&& refund == MultiBalanceOf::<T>::from(0u32)
						&& ico.already_released_proportion != Percent::from_percent(100u8)
					{
						let remain_amount = total.saturating_sub(should_released);
						if is_do {
							Self::terminated_released_user_token(&user, remain_amount, &ico);
						}
						this_time_total_release += remain_amount;
						is_oprate = true;
					}

					let mut thistime_release_amount = should_released.saturating_sub(released);

					if thistime_release_amount == MultiBalanceOf::<T>::from(0u32) && is_oprate == false {
						return Err(Error::<T>::UnreleaseAmountIsZero)?;
					} else if thistime_release_amount == MultiBalanceOf::<T>::from(0u32) && is_oprate == true {
						return Ok(this_time_total_release);
					};

					if is_do {
						Self::common_release(user, ico, thistime_release_amount);
					}

					Ok(this_time_total_release.saturating_add(thistime_release_amount))
				}
			}
		}

		fn common_release(
			user: T::AccountId,
			ico: IcoInfo<T::BlockNumber, MultiBalanceOf<T>, AssetId, AreaCode, T::AccountId>,
			thistime_release_amount: MultiBalanceOf<T>,
		) {
			let project_currency_id = ico.exchange_token;
			let index = ico.index.unwrap();
			if user == ico.initiator {
				T::MultiCurrency::unreserve(project_currency_id, &ico.initiator, thistime_release_amount);
			} else {
				let user_keep_lock_amount = ico.lock_proportion * thistime_release_amount;
				let user_unlock_amount = thistime_release_amount.saturating_sub(user_keep_lock_amount);
				if user_unlock_amount > MultiBalanceOf::<T>::from(0u32) {
					T::MultiCurrency::unreserve(ico.currency_id, &user, user_unlock_amount);
				}
				if user_keep_lock_amount > MultiBalanceOf::<T>::from(0u32) {
					<IcoLocks<T>>::mutate(&user, ico.currency_id, |h| {
						h.push(IcoLock {
							start_block: Self::now(),
							index: index,
							total_amount: user_keep_lock_amount,
							unlock_amount: 0u32.saturated_into::<MultiBalanceOf<T>>(),
							unlock_duration: ico.unlock_duration,
							per_duration_unlock_amount: Self::u256_convert_to_balance((Self::balance_convert_to_u256(thistime_release_amount) * Self::balance_convert_to_u256(ico.per_duration_unlock_amount)) / Self::balance_convert_to_u256(ico.total_ico_amount)),
						})
					});
				}
			}

			Self::update_user_unreleased_assets_info(
				&user,
				&ico,
				thistime_release_amount,
				MultiBalanceOf::<T>::from(0u32),
				false,
				None,
				&None,
			);

			Self::deposit_event(Event::UserReleaseIcoAmount(
				ico.currency_id,
				index,
				thistime_release_amount,
			));
		}

		fn terminated_released_user_token(
			user: &T::AccountId,
			remain_amount: MultiBalanceOf<T>,
			ico: &IcoInfo<T::BlockNumber, MultiBalanceOf<T>, AssetId, AreaCode, T::AccountId>,
		) {
			let exchange_token_amount = Self::get_swap_token_amount(false, remain_amount, &ico);

			/// For users
			T::MultiCurrency::repatriate_reserved(
				ico.currency_id,
				&user,
				&ico.initiator,
				remain_amount,
				BalanceStatus::Free,
			);
			Self::update_user_unreleased_assets_info(
				&ico.initiator,
				&ico,
				exchange_token_amount,
				MultiBalanceOf::<T>::from(0u32),
				false,
				Some(true),
				&None,
			);

			/// For initiator
			T::MultiCurrency::repatriate_reserved(
				ico.exchange_token,
				&ico.initiator,
				&user,
				exchange_token_amount,
				BalanceStatus::Free,
			);
			Self::update_user_unreleased_assets_info(
				&user,
				&ico,
				remain_amount,
				MultiBalanceOf::<T>::from(0u32),
				false,
				Some(true),
				&None,
			);

			Self::deposit_event(Event::TerminatedGiveBackAmount(
				user.clone(),
				ico.currency_id,
				ico.index.unwrap(),
				remain_amount,
			));
		}

		fn unreserve_initiator_remain_token(
			ico: &IcoInfo<T::BlockNumber, MultiBalanceOf<T>, AssetId, AreaCode, T::AccountId>,
		) {
			let index = ico.index.unwrap();
			let (project_total_amount, _, _) =
				Self::get_total_and_released_amount(ico.currency_id, index, &ico.initiator);
			let unico_amount = ico.exchange_token_total_amount.saturating_sub(project_total_amount);
			let remain_project_token = Self::get_swap_token_amount(true, unico_amount, &ico);
			T::MultiCurrency::unreserve(ico.currency_id, &ico.initiator, remain_project_token);
			IsUnservePledge::<T>::remove(ico.currency_id, index);
			Self::deposit_event(Event::UnreservedInitiatorRemainPledgeAmount(
				ico.currency_id,
				index,
				remain_project_token,
			));
		}

		fn is_ico_expire(ico: &IcoInfo<T::BlockNumber, MultiBalanceOf<T>, AssetId, AreaCode, T::AccountId>) -> bool {
			if cfg!(any(feature = "std", feature = "runtime-benchmarks", test)) {
				return true;
			}
			if let Some(time) = ico.start_time {
				if !(ico.is_terminated || (time + ico.ico_duration < Self::now())) {
					return false;
				}
			}
			true
		}

		fn is_pending_ico(currency_id: &AssetId) -> bool {
			let pending_ico = <PendingIco<T>>::get();
			let pos_opt = pending_ico.iter().position(|h| currency_id == &h.ico.currency_id);
			match pos_opt {
				None => {
					return false;
				}
				Some(x) => {
					return true;
				}
			}
		}

		fn insert_invite_info(invitee: &T::AccountId, inviter: Option<T::AccountId>, currency_id: AssetId, index: u32) {
			match inviter {
				Some(inviter) => {
					let invitees = InviteInfoOf::<T>::get(&inviter, currency_id);
					if let None = invitees
						.iter()
						.position(|h| &h.invitee == invitee && h.currency_id == currency_id && h.index == index)
					{
						InviteInfoOf::<T>::mutate(&inviter, currency_id, |k| {
							k.push(InviteInfo {
								inviter: inviter.clone(),
								invitee: invitee.clone(),
								currency_id: currency_id,
								index: index,
								reward: None,
							})
						});
					}
				}
				None => {}
			}
		}

		fn update_user_icoes(who: &T::AccountId, currency_id: AssetId, index: u32) {
			let mut icoes = IcoesOf::<T>::get(&who);
			if let Some(pos) = icoes.iter().position(|h| h.0 == currency_id && h.1 == index) {
			} else {
				icoes.push((currency_id, index))
			}
			IcoesOf::<T>::insert(who, icoes);
		}

		fn insert_ico_assets_info(
			who: &T::AccountId,
			ico: &IcoInfo<T::BlockNumber, MultiBalanceOf<T>, AssetId, AreaCode, T::AccountId>,
			amount: MultiBalanceOf<T>,
			total_usdt: MultiBalanceOf<T>,
			inviter: &Option<T::AccountId>,
		) {
			Self::update_user_unreleased_assets_info(&who, &ico, amount, total_usdt, true, None, &inviter);
			Self::update_user_unreleased_assets_info(&ico.initiator, ico, amount, total_usdt, true, None, &inviter);
		}

		#[transactional]
		fn swap(
			who: &T::AccountId,
			amount: MultiBalanceOf<T>,
			ico: &IcoInfo<T::BlockNumber, MultiBalanceOf<T>, AssetId, AreaCode, T::AccountId>,
		) -> result::Result<MultiBalanceOf<T>, DispatchError> {
			let initiator = &ico.initiator;
			let this_time_project_token_amount = Self::get_swap_token_amount(true, amount, &ico);

			T::MultiCurrency::transfer(ico.exchange_token, &who, &initiator, amount)?;
			T::MultiCurrency::reserve(ico.exchange_token, &initiator, amount)?;

			T::MultiCurrency::repatriate_reserved(
				ico.currency_id,
				&initiator,
				who,
				this_time_project_token_amount,
				BalanceStatus::Reserved,
			)?;
			Ok(this_time_project_token_amount)
		}

		fn get_swap_token_amount(
			is_main: bool,
			amount: MultiBalanceOf<T>,
			ico: &IcoInfo<T::BlockNumber, MultiBalanceOf<T>, AssetId, AreaCode, T::AccountId>,
		) -> MultiBalanceOf<T> {
			let total_project_token_amount = ico.total_ico_amount;
			let total_exchange_token_amount = ico.exchange_token_total_amount;
			let mut result: U256;
			if is_main {
				result = Self::balance_convert_to_u256(amount)
					* Self::balance_convert_to_u256(total_project_token_amount)
					/ Self::balance_convert_to_u256(total_exchange_token_amount)
			} else {
				result = Self::balance_convert_to_u256(amount)
					* Self::balance_convert_to_u256(total_exchange_token_amount)
					/ Self::balance_convert_to_u256(total_project_token_amount)
			}
			Self::u256_convert_to_balance(result)
		}

		fn update_user_unreleased_assets_info(
			who: &T::AccountId,
			ico: &IcoInfo<T::BlockNumber, MultiBalanceOf<T>, AssetId, AreaCode, T::AccountId>,
			amount: MultiBalanceOf<T>,
			total_usdt: MultiBalanceOf<T>,
			is_join: bool,
			is_terminated: Option<bool>,
			inviter: &Option<T::AccountId>,
		) {
			let is_initiator = who == &ico.initiator;
			let mut unreleased_currency_id = ico.currency_id;
			let mut total = amount;
			let mut user = who;

			let index = ico.index.unwrap();
			let main_token_amount = amount;

			if is_initiator {
				user = &ico.initiator;
				unreleased_currency_id = ico.exchange_token;
			} else {
				if is_join {
					total = Self::get_swap_token_amount(true, amount, &ico);
				}
			}

			let mut assets_info = <UnReleaseAssets<T>>::get(&user);
			let mut new_info: UnRelease<MultiBalanceOf<T>, AssetId, T::AccountId>;
			if let Some(pos) = assets_info
				.iter()
				.position(|h| h.currency_id == ico.currency_id && h.index == index)
			{
				new_info = assets_info.swap_remove(pos);

				if is_join {
					if who != &ico.initiator {
						new_info.tags.push((
							total_usdt,
							ico.total_usdt.saturating_add(total_usdt),
							main_token_amount,
							total,
						));
					} else {
						new_info.tags = vec![];
					}
					new_info.total_usdt = new_info.total_usdt.saturating_add(total_usdt);
					new_info.total = new_info.total.saturating_add(total);
					if new_info.inviter.is_none() {
						new_info.inviter = inviter.clone();
					}
				} else {
					if is_terminated.is_some() {
						new_info.refund = total;
					} else {
						new_info.released = new_info.released.saturating_add(total);
					}
				}
			} else {
				if who != &ico.initiator {
					new_info = UnRelease {
						currency_id: ico.currency_id,
						index: index,
						inviter: inviter.clone(),
						tags: vec![(
							total_usdt,
							ico.total_usdt.saturating_add(total_usdt),
							main_token_amount,
							total,
						)],
						total_usdt: total_usdt,
						unreleased_currency_id: unreleased_currency_id,
						total: total,
						refund: MultiBalanceOf::<T>::from(0u32),
						released: MultiBalanceOf::<T>::from(0u32),
						reward: None,
					};
				} else {
					new_info = UnRelease {
						currency_id: ico.currency_id.clone(),
						index: index,
						inviter: inviter.clone(),
						tags: vec![],
						total_usdt: total_usdt,
						unreleased_currency_id: unreleased_currency_id,
						total: total,
						refund: MultiBalanceOf::<T>::from(0u32),
						released: MultiBalanceOf::<T>::from(0u32),
						reward: None,
					};
				}
			};
			assets_info.push(new_info);
			UnReleaseAssets::<T>::insert(user, assets_info);
		}

		fn join_simple_check(
			ico: &IcoInfo<T::BlockNumber, MultiBalanceOf<T>, AssetId, AreaCode, T::AccountId>,
			amount: MultiBalanceOf<T>,
			currency_id: AssetId,
			index: u32,
			who: &T::AccountId,
			total_usdt: MultiBalanceOf<T>,
		) -> result::Result<bool, DispatchError> {
			ensure!(
				amount != 0u128.saturated_into::<MultiBalanceOf<T>>(),
				Error::<T>::AmountIsZero
			);

			match ico.start_time {
				Some(time) => {
					#[cfg(test)]
					println!("time: {:?}, now:{:?}", time, Self::now());
					ensure!(time <= Self::now(), Error::<T>::IsNotStartIcoTime);
					if cfg!(any(feature = "std", feature = "runtime-benchmarks", test)) == false {
						ensure!(ico.ico_duration + time >= Self::now(), Error::<T>::IcoExpire);
					}
				}

				None => Err(Error::<T>::StartTimeNotExists)?,
			}
			ensure!(!ico.is_terminated, Error::<T>::IcoTerminated);
			ensure!(&ico.initiator != who, Error::<T>::InitiatorIsYourself);
			ensure!(
				!Self::is_exclude_area(&who, &currency_id, index)?,
				Error::<T>::InExcludeArea
			);

			let mut total_usdt = total_usdt;
			let unreleased_info_opt = Self::get_unrelease_asset_info(&who, currency_id, index);
			match unreleased_info_opt.as_ref() {
				None => {}
				Some(x) => {
					total_usdt = total_usdt.checked_add(&x.total_usdt).ok_or(Error::<T>::Overflow)?;
				}
			}
			runtime_print!(
				"total_usdt: {:?}, user_min_amount: {:?}, user_max_amount: {:?}",
				total_usdt,
				ico.user_min_amount,
				ico.user_max_amount
			);
			ensure!(
				total_usdt >= ico.user_min_amount && total_usdt <= ico.user_max_amount,
				Error::<T>::AmountNotMeetProjectRequirement
			);
			ensure!(
				total_usdt >= IcoMinUsdtAmount::<T>::get() && total_usdt <= IcoMaxUsdtAmount::<T>::get(),
				Error::<T>::AmountNotMeetSystemRequirement
			);

			if let Some(info) = unreleased_info_opt {
				ensure!(
					info.tags.len().saturating_add(1) <= ico.user_ico_max_times.into(),
					Error::<T>::IcoTimesToMax
				);
			}

			let initiator_total_amount = Self::get_total_and_released_amount(ico.currency_id, index, &ico.initiator).0;

			ensure!(
				ico.exchange_token_total_amount.saturating_sub(initiator_total_amount) >= amount,
				Error::<T>::ProjectIcoAmountToMax
			);
			Ok(true)
		}

		fn get_unrelease_asset_info(
			who: &T::AccountId,
			currency_id: AssetId,
			index: u32,
		) -> Option<UnRelease<MultiBalanceOf<T>, AssetId, T::AccountId>> {
			let mut unrelease_info_vec = UnReleaseAssets::<T>::get(&who);
			if let Some(pos) = unrelease_info_vec
				.iter()
				.position(|h| h.currency_id == currency_id && h.index == index)
			{
				return Some(unrelease_info_vec.swap_remove(pos));
			}
			None
		}

		fn update_initiate_icoes(currency_id: AssetId, index: u32, who: &T::AccountId, status: IcoStatus) {
			let mut icoes = InitiatedIcoesOf::<T>::get(&who);
			if let Some(pos) = icoes
				.iter()
				.position(|h| h.currency_id == currency_id && h.index == index)
			{
				let mut ico = icoes.swap_remove(pos);
				ico.status = status;
				icoes.push(ico);
				InitiatedIcoesOf::<T>::insert(who, icoes);
			}
		}

		fn now() -> T::BlockNumber {
			<system::Pallet<T>>::block_number()
		}

		fn is_exclude_area(
			who: &T::AccountId,
			currency_id: &AssetId,
			index: u32,
		) -> result::Result<bool, DispatchError> {
			let ico = <Ico<T>>::get(currency_id, index).ok_or(Error::<T>::IcoNotExists123)?;
			if &ico.is_must_kyc == &true {
				if Self::is_already_kyc(&who) {
					let nations = ico.exclude_area;
					if !nations.is_empty() {
						let nation_opt = Self::get_uesr_area(&who);
						match nation_opt {
							None => {
								return Err(Error::<T>::UserAreaNotExists)?;
							}
							Some(nation) => {
								if let Some(pos) = nations.iter().position(|h| *h == nation) {
									return Ok(true);
								}
							}
						};
					}
				} else {
					Err(Error::<T>::NotKycUser)?;
				}
			}

			Ok(false)
		}

		fn exchange_token_convert_usdt(
			currency_id: AssetId,
			decimals: u8,
			amount: MultiBalanceOf<T>,
		) -> result::Result<MultiBalanceOf<T>, DispatchError> {
			let price = Self::get_token_price(currency_id);
			ensure!(price != MultiBalanceOf::<T>::from(0u32), Error::<T>::PriceNotExists);
			let decimals_amount = 10u128
				.saturating_pow(decimals as u32)
				.saturated_into::<MultiBalanceOf<T>>();
			let mut usdt = Self::u256_convert_to_balance(
				Self::balance_convert_to_u256(price) * Self::balance_convert_to_u256(amount)
					/ Self::balance_convert_to_u256(decimals_amount),
			);

			let multiple = PowerMultipleOf::<T>::get(currency_id);

			usdt = (usdt * multiple.up.saturated_into::<MultiBalanceOf<T>>())
				.checked_div(&multiple.down.saturated_into::<MultiBalanceOf<T>>())
				.ok_or(Error::<T>::DivByZero)?;

			Ok(usdt)
		}

		pub fn get_token_price(currency_id: AssetId) -> MultiBalanceOf<T> {
			if currency_id == T::USDCurrencyId::get() {
				return USD.saturated_into::<MultiBalanceOf<T>>();
			}
			match T::PriceData::get_price(currency_id, T::USDCurrencyId::get()) {
				Some(x) => {
					runtime_print!(
						" ---------------the token {:?}, price is {:?} ------------------",
						currency_id,
						x
					);
					return x.saturated_into::<MultiBalanceOf<T>>();
				}
				None => return MultiBalanceOf::<T>::from(0u32),
			}
		}

		fn get_uesr_area(who: &T::AccountId) -> Option<AreaCode> {
			T::KycHandler::get_user_area(who)
		}

		fn unlock_asset(
			who: &T::AccountId,
			currency_id: &AssetId,
			index: u32,
			is_do: bool,
			locks: &mut [IcoLock<MultiBalanceOf<T>, T::BlockNumber>],
		) -> (MultiBalanceOf<T>, Vec<IcoLock<MultiBalanceOf<T>, T::BlockNumber>>) {
			let mut total = <MultiBalanceOf<T>>::from(0u32);
			for i in 0..locks.len() {
				if locks[i].index != index {
					continue;
				}

				let time = Self::now().saturating_sub(locks[i].start_block);
				if locks[i].unlock_duration == T::BlockNumber::from(0u32) {
					let unlock_amount = locks[i].total_amount.saturating_sub(locks[i].unlock_amount);
					if is_do {
						T::MultiCurrency::unreserve(*currency_id, &who, unlock_amount);
					}
					total += unlock_amount;
				} else {
					let num = time / locks[i].unlock_duration;
					let total_unlock_amount = (num
						.saturated_into::<u32>()
						.saturated_into::<MultiBalanceOf<T>>()
						.saturating_mul(locks[i].per_duration_unlock_amount))
					.min(locks[i].total_amount);
					let this_time_unlock_amount = total_unlock_amount.saturating_sub(locks[i].unlock_amount);
					if is_do {
						locks[i].unlock_amount = total_unlock_amount;
						T::MultiCurrency::unreserve(*currency_id, &who, this_time_unlock_amount);
					}
					total += this_time_unlock_amount;
				}
			}
			if is_do {
				let mut locks = locks.to_vec();
				locks.retain(|h| h.total_amount != h.unlock_amount);
			}

			(total, locks.to_vec())
		}

		fn check_bound(min: MultiBalanceOf<T>, max: MultiBalanceOf<T>) -> DispatchResult {
			ensure!(min >= IcoMinUsdtAmount::<T>::get(), Error::<T>::MinAmountTooLow);
			ensure!(max != MultiBalanceOf::<T>::from(0u32), Error::<T>::MaxAmountIsZero);
			ensure!(min <= max, Error::<T>::MaxLessThanMin);
			ensure!(max <= IcoMaxUsdtAmount::<T>::get(), Error::<T>::MaxAmountTooLarge);
			Ok(())
		}

		fn remove_passed_icoes(currency_id: AssetId, index: u32) {
			PassedIcoes::<T>::mutate(|h| h.retain(|i| !(i.0 == currency_id && i.1 == index)));
		}

		fn remove_request_release_info(currency_id: Option<AssetId>, index: u32, is_all: bool) {
			let mut release_info = <RequestReleaseInfo<T>>::get();
			if is_all {
				release_info.retain(|h| {
					if h.request_time + T::RequestExpire::get() > Self::now() {
						true
					} else {
						T::OnSlash::on_unbalanced(T::NativeCurrency::slash_reserved(&h.who, h.pledge).0);
						false
					}
				});
			} else {
				if let Some(currency_id) = currency_id {
					if let Some(pos) = release_info
						.iter()
						.position(|h| h.currency_id == currency_id && h.index == index)
					{
						release_info.swap_remove(pos);
					}
				}
			}
			<RequestReleaseInfo<T>>::put(release_info);
		}

		fn initiate_ico_simple_check(
			info: &IcoParameters<T::BlockNumber, MultiBalanceOf<T>, AssetId, AreaCode>,
			who: &T::AccountId,
		) -> result::Result<bool, DispatchError> {
			ensure!(info.user_ico_max_times > 0, Error::<T>::MaxIsZero);
			ensure!(
				info.exchange_token != info.currency_id,
				Error::<T>::TokenShouldBeDifferent
			);
			ensure!(
				info.exchange_token_total_amount != 0u128.saturated_into::<MultiBalanceOf<T>>()
					&& info.total_ico_amount != 0u128.saturated_into::<MultiBalanceOf<T>>(),
				Error::<T>::AmountIsZero
			);
			ensure!(
				info.total_issuance >= info.total_circulation,
				Error::<T>::CirculationMoreThanIssuance
			);
			ensure!(
				info.total_circulation >= info.total_ico_amount,
				Error::<T>::IcoAmountMoreThanCirculation
			);

			Self::check_bound(info.user_min_amount, info.user_max_amount)?;

			ensure!(
				info.ico_duration > T::BlockNumber::from(0u32),
				Error::<T>::DurationIsZero
			);

			if info.lock_proportion > Percent::from_percent(0u8) {
				ensure!(
					info.unlock_duration > T::BlockNumber::from(0u32),
					Error::<T>::DurationIsZero
				);
				if cfg!(any(feature = "std", feature = "runtime-benchmarks", test)) == false {
					ensure!(
						info.per_duration_unlock_amount > 0u128.saturated_into::<MultiBalanceOf<T>>(),
						Error::<T>::UnlockAmountIsZero
					);
				}
			}

			ensure!(!Self::is_pending_ico(&info.currency_id), Error::<T>::IsPendingIco);
			ensure!(
				T::MultiCurrency::can_reserve(info.currency_id, &who, info.total_ico_amount),
				Error::<T>::BalanceInsufficient
			);

			if cfg!(any(feature = "std", feature = "runtime-benchmarks", test)) == false {
				ensure!(
					T::MultiCurrency::can_reserve(
						info.exchange_token,
						&who,
						T::InitiatorBond::get() * info.exchange_token_total_amount
					),
					Error::<T>::ExchangeTokenBalanceTooLow
				);
			}

			Ok(true)
		}

		pub fn get_total_and_released_amount(
			currency_id: AssetId,
			index: u32,
			who: &T::AccountId,
		) -> (MultiBalanceOf<T>, MultiBalanceOf<T>, MultiBalanceOf<T>) {
			let mut unrelease_info_vec = UnReleaseAssets::<T>::get(&who);
			if let Some(pos) = unrelease_info_vec
				.iter()
				.position(|h| h.currency_id == currency_id && h.index == index)
			{
				let info = unrelease_info_vec.swap_remove(pos);
				return (info.total, info.released, info.total_usdt);
			}
			(
				MultiBalanceOf::<T>::from(0u32),
				MultiBalanceOf::<T>::from(0u32),
				MultiBalanceOf::<T>::from(0u32),
			)
		}

		fn get_request_release_info(
			currency_id: AssetId,
			index: u32,
		) -> Option<Release<T::AccountId, T::BlockNumber, AssetId, BalanceOf<T>>> {
			let mut release_info = <RequestReleaseInfo<T>>::get();
			if let Some(pos) = release_info
				.iter()
				.position(|h| h.currency_id == currency_id && h.index == index)
			{
				return Some(release_info.swap_remove(pos));
			}
			None
		}

		pub fn balance_convert_to_u256(amount: MultiBalanceOf<T>) -> U256 {
			amount.saturated_into::<u128>().saturating_add(1u128).into()
		}

		pub fn u256_convert_to_balance(num: U256) -> MultiBalanceOf<T> {
			<u128 as TryFrom<U256>>::try_from(num)
				.unwrap_or_default()
				.saturated_into::<MultiBalanceOf<T>>()
				.saturating_add(MultiBalanceOf::<T>::from(1u32))
		}

		/// The total amount that can release.
		/// Provided for RPC use
		pub fn can_release_amount(user: T::AccountId, currency_id: AssetId, index: u32) -> MultiBalanceOf<T> {
			Self::do_release(user, currency_id, index, false).unwrap_or_default()
		}

		/// The total amount that can unlock.
		/// Provided for RPC use
		pub fn can_unlock_amount(user: T::AccountId, currency_id: AssetId, index: u32) -> MultiBalanceOf<T> {
			let (total, _) = Self::unlock_asset(
				&user,
				&currency_id,
				index,
				false,
				&mut <IcoLocks<T>>::get(&user, &currency_id),
			);
			total
		}

		/// The reward amount
		/// Provided for RPC use
		pub fn get_reward_amount(user: T::AccountId, currency_id: AssetId, index: u32) -> MultiBalanceOf<T> {
			Self::do_reward(&user, currency_id, index, false).unwrap_or_default()
		}

		/// the total amount that can join ico
		/// Provided for RPC use
		pub fn can_join_amount(
			user: T::AccountId,
			currency_id: AssetId,
			index: u32,
		) -> (MultiBalanceOf<T>, MultiBalanceOf<T>) {
			let ico = match <Ico<T>>::get(currency_id, index) {
				None => return (MultiBalanceOf::<T>::from(0u32), MultiBalanceOf::<T>::from(0u32)),
				Some(x) => x,
			};
			let exchange_token_id = ico.exchange_token;
			let max_times = ico.user_ico_max_times;
			let user_max_amount = ico.user_max_amount;
			let mut user_min_amount = ico.user_min_amount.max(IcoMinUsdtAmount::<T>::get());

			if user == ico.initiator {
				return (MultiBalanceOf::<T>::from(0u32), MultiBalanceOf::<T>::from(0u32));
			}
			runtime_print!("the user is {:?}", user);
			let exchange_token_decimals = match T::CurrenciesHandler::get_metadata(exchange_token_id) {
				Ok(x) => x.decimals,
				_ => return (MultiBalanceOf::<T>::from(0u32), MultiBalanceOf::<T>::from(0u32)),
			};
			runtime_print!("exchange token decimals is {:?}", exchange_token_decimals);
			if Self::is_ico_expire(&ico) {
				return (MultiBalanceOf::<T>::from(0u32), MultiBalanceOf::<T>::from(0u32));
			}
			runtime_print!("the ico is not expire");
			match ico.start_time {
				Some(time) => {
					if Self::now() < time {
						return (MultiBalanceOf::<T>::from(0u32), MultiBalanceOf::<T>::from(0u32));
					}
				}
				None => return (MultiBalanceOf::<T>::from(0u32), MultiBalanceOf::<T>::from(0u32)),
			}

			let user_remain_usdt = match Self::get_unrelease_asset_info(&user, currency_id, index) {
				Some(x) => {
					user_min_amount = user_min_amount.saturating_sub(x.total_usdt);
					if max_times <= 1 && x.tags.len() > 0 {
						return (MultiBalanceOf::<T>::from(0u32), MultiBalanceOf::<T>::from(0u32));
					} else {
						(user_max_amount.min(IcoMaxUsdtAmount::<T>::get())).saturating_sub(x.total_usdt)
					}
				}
				None => user_max_amount.min(IcoMaxUsdtAmount::<T>::get()),
			};
			runtime_print!("remain usdt amount is {:?}", user_remain_usdt);

			let mul = PowerMultipleOf::<T>::get(exchange_token_id);
			let a = MultiBalanceOf::<T>::from(mul.up);
			let b = MultiBalanceOf::<T>::from(mul.down);
			let price = Self::get_token_price(exchange_token_id) * a / b;

			if price == MultiBalanceOf::<T>::from(0u32) {
				return (MultiBalanceOf::<T>::from(0u32), MultiBalanceOf::<T>::from(0u32));
			}
			runtime_print!("the token price is {:?}", price);

			let decimals_convert_amount = 10u128
				.saturating_pow(exchange_token_decimals as u32)
				.saturated_into::<MultiBalanceOf<T>>();

			let user_remain_exchange_amount = Self::u256_convert_to_balance(
				Self::balance_convert_to_u256(user_remain_usdt)
					* Self::balance_convert_to_u256(decimals_convert_amount)
					/ Self::balance_convert_to_u256(price),
			);
			runtime_print!(
				"user can join exchange token amount is {:?}",
				user_remain_exchange_amount
			);
			let project_remain_exchange_amount =
				match Self::get_unrelease_asset_info(&ico.initiator, currency_id, index) {
					None => ico.exchange_token_total_amount,
					Some(x) => ico.exchange_token_total_amount.saturating_sub(x.total),
				};

			let max_join_amount = project_remain_exchange_amount.min(user_remain_exchange_amount);
			runtime_print!(
				"project can join exchange token amount is {:?}",
				project_remain_exchange_amount
			);
			let min_join_amount = Self::u256_convert_to_balance(
				Self::balance_convert_to_u256(user_min_amount) * Self::balance_convert_to_u256(decimals_convert_amount)
					/ Self::balance_convert_to_u256(price),
			);
			runtime_print!("the min amount more than the max amount");
			if min_join_amount > max_join_amount {
				return (MultiBalanceOf::<T>::from(0u32), MultiBalanceOf::<T>::from(0u32));
			}

			(min_join_amount, max_join_amount)
		}
	}
}

impl<T: Config> IcoHandler<AssetId, MultiBalanceOf<T>, T::AccountId, DispatchError, T::BlockNumber> for Pallet<T> {
	fn set_ico_for_bench(
		currency_id: AssetId,
		index: u32,
		initiator: T::AccountId,
		joiner: T::AccountId,
		joiner1: T::AccountId,
	) -> DispatchResult {
		runtime_print!("create currency_id:{:?}, index: {:?}", currency_id, index);
		let ico_info: IcoInfo<T::BlockNumber, MultiBalanceOf<T>, AssetId, AreaCode, T::AccountId> = IcoInfo {
			desc: vec![],
			start_time: Some(T::BlockNumber::from(0u32)),
			is_already_kyc: false,
			initiator: initiator.clone(),
			total_usdt: (5000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
			tag: Some((5000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>()),
			is_terminated: false,
			project_name: vec![],
			token_symbol: vec![],
			decimals: 12,
			index: Some(1),
			already_released_proportion: Default::default(),
			currency_id: currency_id,
			official_website: vec![],
			user_ico_max_times: 2,
			is_must_kyc: false,
			total_issuance: (10000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
			total_circulation: (10000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
			ico_duration: T::BlockNumber::from(0u32),
			total_ico_amount: (10000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
			user_min_amount: (100 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
			user_max_amount: (2000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
			exchange_token: T::USDCurrencyId::get(),
			exchange_token_total_amount: (10000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
			exclude_area: vec![AreaCode::AD],
			lock_proportion: Default::default(),
			unlock_duration: T::BlockNumber::from(0u32),
			per_duration_unlock_amount: MultiBalanceOf::<T>::from(0u32),
		};

		<Ico<T>>::insert(currency_id, index, ico_info);

		let info1 = UnRelease {
			currency_id: currency_id,
			inviter: None,
			index: 1,
			unreleased_currency_id: currency_id,
			total_usdt: (500 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
			tags: vec![(
				(500 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
				(500 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
				(500 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
				(500 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
			)],
			total: (5000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
			released: MultiBalanceOf::<T>::from(0u32),
			refund: MultiBalanceOf::<T>::from(0u32),
			reward: None,
		};

		let info2 = UnRelease {
			currency_id: currency_id,
			inviter: None,
			index: 1,
			unreleased_currency_id: T::USDCurrencyId::get(),
			total_usdt: (5000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
			tags: vec![],
			total: (50000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
			released: MultiBalanceOf::<T>::from(0u32),
			refund: MultiBalanceOf::<T>::from(0u32),
			reward: None,
		};

		let info3 = UnRelease {
			currency_id: currency_id,
			inviter: None,
			index: 1,
			unreleased_currency_id: currency_id,
			total_usdt: (1000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
			tags: vec![(
				(1000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
				(1000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
				(1000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
				(1000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
			)],
			total: (5000 * DOLLARS).saturated_into::<MultiBalanceOf<T>>(),
			released: MultiBalanceOf::<T>::from(0u32),
			refund: MultiBalanceOf::<T>::from(0u32),
			reward: None,
		};

		<UnReleaseAssets<T>>::mutate(joiner, |h| h.push(info1));
		<UnReleaseAssets<T>>::mutate(joiner1, |h| h.push(info3));
		<UnReleaseAssets<T>>::mutate(initiator, |h| h.push(info2));

		Ok(())
	}
	fn is_project_ico_member(
		currency_id: AssetId,
		index: u32,
		who: &T::AccountId,
	) -> result::Result<bool, DispatchError> {
		runtime_print!("currency_id:{:?}, index: {:?}", currency_id, index);
		let _ = Ico::<T>::get(currency_id, index).ok_or(Error::<T>::IcoNotExists123)?;
		Ok(Self::is_member(who, currency_id, index, true))
	}

	fn get_user_total_amount(currency_id: AssetId, index: u32, who: &T::AccountId) -> MultiBalanceOf<T> {
		Self::get_total_and_released_amount(currency_id, index, &who).0
	}

	fn get_project_total_ico_amount(currency_id: AssetId, index: u32) -> Result<MultiBalanceOf<T>, DispatchError> {
		runtime_print!("currency_id:{:?}, index: {:?}", currency_id, index);
		let ico = Ico::<T>::get(currency_id, index).ok_or(Error::<T>::IcoNotExists123)?;
		let result = Self::balance_convert_to_u256(ico.total_ico_amount)
			* Self::balance_convert_to_u256(Self::get_total_and_released_amount(currency_id, index, &ico.initiator).0)
			/ Self::balance_convert_to_u256(ico.exchange_token_total_amount);
		let amount = Self::u256_convert_to_balance(result);
		Ok(amount)
	}
}

impl<T: Config> PowerHandler<T::AccountId, DispatchResult, MultiBalanceOf<T>> for Pallet<T> {
	fn sub_user_power(user: &T::AccountId, amount: MultiBalanceOf<T>) -> DispatchResult {
		let old = TotalPowerOf::<T>::get(user);
		let new = old.checked_sub(&amount).ok_or(Error::<T>::Overflow)?;
		TotalPowerOf::<T>::insert(user, new);
		Ok(())
	}

	fn add_user_power(user: &T::AccountId, amount: MultiBalanceOf<T>) -> DispatchResult {
		let old = TotalPowerOf::<T>::get(user);
		let new = old.checked_add(&amount).ok_or(Error::<T>::Overflow)?;
		TotalPowerOf::<T>::insert(user, new);

		Ok(())
	}
}
