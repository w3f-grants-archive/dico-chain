//! Autogenerated weights for pallet_amm
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-04-15, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("kico"), DB CACHE: 1024

// Executed Command:
// target/release/dico
// benchmark
// --chain=kico
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_amm
// --extrinsic=*
// --steps=50
// --repeat=20
// --template=./.maintain/pallet-weight-template.hbs
// --output
// ./pallets/amm/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_amm.
pub trait WeightInfo {
	fn add_liquidity() -> Weight;
	fn remove_liquidity() -> Weight;
	fn swap_exact_assets_for_assets() -> Weight;
	fn swap_assets_for_exact_assets() -> Weight;
}

/// Weights for pallet_amm using the Substrate node and recommended hardware.
pub struct DicoWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for DicoWeight<T> {
	// Storage: AMM Liquidity (r:1 w:1)
	// Storage: Currencies DicoAssetsInfo (r:3 w:1)
	// Storage: AMM NextLiquidityId (r:1 w:1)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	// Storage: Tokens Accounts (r:5 w:5)
	// Storage: System Account (r:2 w:2)
	fn add_liquidity() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: AMM Liquidity (r:1 w:1)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	// Storage: Tokens Accounts (r:5 w:5)
	// Storage: System Account (r:1 w:0)
	fn remove_liquidity() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: AMM Liquidity (r:1 w:1)
	// Storage: Tokens Accounts (r:4 w:4)
	// Storage: System Account (r:2 w:0)
	fn swap_exact_assets_for_assets() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: AMM Liquidity (r:1 w:1)
	// Storage: Tokens Accounts (r:4 w:4)
	// Storage: System Account (r:2 w:0)
	fn swap_assets_for_exact_assets() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: AMM Liquidity (r:1 w:1)
	// Storage: Currencies DicoAssetsInfo (r:3 w:1)
	// Storage: AMM NextLiquidityId (r:1 w:1)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	// Storage: Tokens Accounts (r:5 w:5)
	// Storage: System Account (r:2 w:2)
	fn add_liquidity() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: AMM Liquidity (r:1 w:1)
	// Storage: Tokens TotalIssuance (r:1 w:1)
	// Storage: Tokens Accounts (r:5 w:5)
	// Storage: System Account (r:1 w:0)
	fn remove_liquidity() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: AMM Liquidity (r:1 w:1)
	// Storage: Tokens Accounts (r:4 w:4)
	// Storage: System Account (r:2 w:0)
	fn swap_exact_assets_for_assets() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: AMM Liquidity (r:1 w:1)
	// Storage: Tokens Accounts (r:4 w:4)
	// Storage: System Account (r:2 w:0)
	fn swap_assets_for_exact_assets() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
}
