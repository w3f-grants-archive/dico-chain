#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::{prelude::*, result};
pub trait IcoHandler<CurrencyId, MulBalanceOf, AccountId, DispathErr, BlockNumber> {
	fn is_project_ico_member(currency_id: CurrencyId, index: u32, who: &AccountId) -> result::Result<bool, DispathErr>;
	fn get_user_total_amount(currency_id: CurrencyId, index: u32, who: &AccountId) -> MulBalanceOf;
	fn get_project_total_ico_amount(currency_id: CurrencyId, index: u32) -> result::Result<MulBalanceOf, DispathErr>;
}


impl<CurrencyId: Ord + Clone, MulBalanceOf: Ord + Clone + Default, AccountId: Ord + Clone, DispathErr: Ord + Clone, BlockNumber: Ord + Clone + Default>
	IcoHandler<CurrencyId, MulBalanceOf, AccountId, DispathErr, BlockNumber> for () {
	fn is_project_ico_member(_: CurrencyId, _: u32, _: &AccountId) -> Result<bool, DispathErr> {
		Ok(false)
	}

	fn get_user_total_amount(_: CurrencyId, _: u32, _: &AccountId) -> MulBalanceOf {
		Default::default()
	}

	fn get_project_total_ico_amount(currency_id: CurrencyId, index: u32) -> Result<MulBalanceOf, DispathErr> {
		Ok(Default::default())
	}
}

pub trait PowerHandler<AccountId, DispathResult, Balance> {
	fn sub_user_power(user: &AccountId, amount: Balance) -> DispathResult;
	fn add_user_power(user: &AccountId, amount: Balance) -> DispathResult;
}
