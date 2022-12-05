//! Autogenerated weights for pallet_pricedao
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-01-29, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("tico"), DB CACHE: 128

// Executed Command:
// target/release/dico
// benchmark
// --chain=tico
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_pricedao
// --extrinsic=*
// --steps=50
// --repeat=20
// --template=./.maintain/pallet-weight-template.hbs
// --output
// ./pallets/pricedao/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_pricedao.
pub trait WeightInfo {
	fn insert_feed_account() -> Weight;
	fn del_feed_account() -> Weight;
	fn unlock_price() -> Weight;
	fn exit_feed() -> Weight;
	fn withdraw() -> Weight;
}

/// Weights for pallet_pricedao using the Substrate node and recommended hardware.
pub struct DicoWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for DicoWeight<T> {
	// Storage: System Account (r:1 w:1)
	// Storage: DicoOracle Members (r:1 w:1)
	// Storage: PriceDao DepositBalance (r:0 w:1)
	fn insert_feed_account() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: PriceDao DepositBalance (r:1 w:1)
	// Storage: System Account (r:2 w:2)
	// Storage: DicoOracle Members (r:1 w:1)
	fn del_feed_account() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: DicoOracle LockedPrice (r:0 w:1)
	fn unlock_price() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: PriceDao DepositBalance (r:1 w:1)
	// Storage: DicoOracle Members (r:1 w:1)
	fn exit_feed() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: PriceDao DepositBalance (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn withdraw() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	// Storage: System Account (r:1 w:1)
	// Storage: DicoOracle Members (r:1 w:1)
	// Storage: PriceDao DepositBalance (r:0 w:1)
	fn insert_feed_account() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: PriceDao DepositBalance (r:1 w:1)
	// Storage: System Account (r:2 w:2)
	// Storage: DicoOracle Members (r:1 w:1)
	fn del_feed_account() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: DicoOracle LockedPrice (r:0 w:1)
	fn unlock_price() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: PriceDao DepositBalance (r:1 w:1)
	// Storage: DicoOracle Members (r:1 w:1)
	fn exit_feed() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
	// Storage: PriceDao DepositBalance (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn withdraw() -> Weight {
		Weight::from_ref_time(20_0000_0000)
	}
}
