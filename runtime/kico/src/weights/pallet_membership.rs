//! Autogenerated weights for `pallet_membership`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-01-29, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("tico"), DB CACHE: 128

// Executed Command:
// target/release/dico
// benchmark
// --chain=kico
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_membership
// --extrinsic=*
// --steps=50
// --repeat=20
// --raw
// --output=./runtime/tico/src/weights/pallet_membership.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_membership`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_membership::WeightInfo for WeightInfo<T> {
    // Storage: TechnicalMembership Members (r:1 w:1)
    // Storage: TechnicalCommittee Proposals (r:1 w:0)
    // Storage: TechnicalCommittee Members (r:0 w:1)
    // Storage: TechnicalCommittee Prime (r:0 w:1)
    fn add_member(m: u32, ) -> Weight {
        (24_594_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((114_000 as Weight).saturating_mul(m as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: TechnicalMembership Members (r:1 w:1)
    // Storage: TechnicalCommittee Proposals (r:1 w:0)
    // Storage: TechnicalMembership Prime (r:1 w:0)
    // Storage: TechnicalCommittee Members (r:0 w:1)
    // Storage: TechnicalCommittee Prime (r:0 w:1)
    fn remove_member(m: u32, ) -> Weight {
        (29_119_000 as Weight)
            // Standard Error: 0
            .saturating_add((105_000 as Weight).saturating_mul(m as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: TechnicalMembership Members (r:1 w:1)
    // Storage: TechnicalCommittee Proposals (r:1 w:0)
    // Storage: TechnicalMembership Prime (r:1 w:0)
    // Storage: TechnicalCommittee Members (r:0 w:1)
    // Storage: TechnicalCommittee Prime (r:0 w:1)
    fn swap_member(m: u32, ) -> Weight {
        (29_397_000 as Weight)
            // Standard Error: 0
            .saturating_add((116_000 as Weight).saturating_mul(m as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: TechnicalMembership Members (r:1 w:1)
    // Storage: TechnicalCommittee Proposals (r:1 w:0)
    // Storage: TechnicalMembership Prime (r:1 w:0)
    // Storage: TechnicalCommittee Members (r:0 w:1)
    // Storage: TechnicalCommittee Prime (r:0 w:1)
    fn reset_member(m: u32, ) -> Weight {
        (29_939_000 as Weight)
            // Standard Error: 0
            .saturating_add((245_000 as Weight).saturating_mul(m as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: TechnicalMembership Members (r:1 w:1)
    // Storage: TechnicalCommittee Proposals (r:1 w:0)
    // Storage: TechnicalMembership Prime (r:1 w:1)
    // Storage: TechnicalCommittee Members (r:0 w:1)
    // Storage: TechnicalCommittee Prime (r:0 w:1)
    fn change_key(m: u32, ) -> Weight {
        (30_881_000 as Weight)
            // Standard Error: 0
            .saturating_add((116_000 as Weight).saturating_mul(m as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
    }
    // Storage: TechnicalMembership Members (r:1 w:0)
    // Storage: TechnicalMembership Prime (r:0 w:1)
    // Storage: TechnicalCommittee Prime (r:0 w:1)
    fn set_prime(m: u32, ) -> Weight {
        (8_081_000 as Weight)
            // Standard Error: 0
            .saturating_add((81_000 as Weight).saturating_mul(m as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: TechnicalMembership Prime (r:0 w:1)
    // Storage: TechnicalCommittee Prime (r:0 w:1)
    fn clear_prime(m: u32, ) -> Weight {
        (3_014_000 as Weight)
            // Standard Error: 0
            .saturating_add((2_000 as Weight).saturating_mul(m as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
}