use sp_core::U256;
use sp_runtime::{ArithmeticError, RuntimeDebug};

pub trait PriceProvider<CurrencyId> {
	type Price;
	fn get_price_from_swap(currency_id: CurrencyId, stable_coin: CurrencyId) -> Option<Self::Price>;
	fn get_price_from_oracle(currency_id: CurrencyId) -> Option<Self::Price>;
	fn get_uint(currency_id: CurrencyId) -> Option<U256>;
}

pub trait PriceData<CurrencyId> {
	type Price;
	fn get_price(currency_id: CurrencyId, stable_coin: CurrencyId) -> Option<Self::Price>;
}
