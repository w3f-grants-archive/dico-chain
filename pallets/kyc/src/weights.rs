//! Autogenerated weights for pallet_kyc
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-01-30, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("tico"), DB CACHE: 128

// Executed Command:
// target/release/dico
// benchmark
// --chain=tico
// --execution=wasm
// --wasm-execution=compiled
// --pallet=pallet_kyc
// --extrinsic=*
// --steps=50
// --repeat=20
// --template=./.maintain/pallet-weight-template.hbs
// --output
// ./pallets/kyc/src/weights.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_kyc.
pub trait WeightInfo {
    fn set_kyc(r: u32, ) -> Weight;
    fn clear_kyc(r: u32, ) -> Weight;
    fn remove_kyc(r: u32, ) -> Weight;
    fn apply_certification(r: u32, ) -> Weight;
    fn add_ias(r: u32, ) -> Weight;
    fn add_sword_holder(r: u32, ) -> Weight;
    fn ias_set_fee(r: u32, ) -> Weight;
    fn sword_holder_set_fee(r: u32, ) -> Weight;
    fn kill_ias(r: u32, ) -> Weight;
    fn kill_sword_holder(r: u32, ) -> Weight;
    fn request_judgement(n: u32, ) -> Weight;
    fn ias_provide_judgement(n: u32, ) -> Weight;
    fn sword_holder_provide_judgement(n: u32, ) -> Weight;
}

/// Weights for pallet_kyc using the Substrate node and recommended hardware.
pub struct DicoWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for DicoWeight<T> {
    // Storage: Kyc BlackListOf (r:1 w:0)
    // Storage: Kyc KYCOf (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn set_kyc(r: u32, ) -> Weight {
        (50_854_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((78_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: Kyc ApplicationFormList (r:1 w:0)
    // Storage: Kyc KYCOf (r:1 w:1)
    fn clear_kyc(r: u32, ) -> Weight {
        (31_637_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((58_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Kyc BlackListOf (r:1 w:1)
    // Storage: Kyc KYCOf (r:1 w:0)
    // Storage: Kyc ApplicationFormList (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    fn remove_kyc(r: u32, ) -> Weight {
        (74_397_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((78_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: Kyc BlackListOf (r:1 w:0)
    // Storage: Kyc KYCOf (r:1 w:0)
    // Storage: Kyc ApplicationFormList (r:1 w:1)
    // Storage: Kyc Nonce (r:1 w:1)
    // Storage: RandomnessCollectiveFlip RandomMaterial (r:1 w:0)
    // Storage: Kyc IASListOf (r:1 w:0)
    // Storage: Kyc SwordHolderOf (r:1 w:0)
    fn apply_certification(r: u32, ) -> Weight {
        (61_765_000 as Weight)
            // Standard Error: 2_000
            .saturating_add((608_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(7 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: Kyc IASListOf (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn add_ias(r: u32, ) -> Weight {
        (45_331_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((316_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: Kyc SwordHolderOf (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn add_sword_holder(r: u32, ) -> Weight {
        (45_375_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((318_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: Kyc IASListOf (r:1 w:1)
    fn ias_set_fee(r: u32, ) -> Weight {
        (30_062_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((547_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Kyc SwordHolderOf (r:1 w:1)
    fn sword_holder_set_fee(r: u32, ) -> Weight {
        (30_587_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((543_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Kyc IASListOf (r:1 w:1)
    // Storage: Kyc RecordsOf (r:1 w:0)
    fn kill_ias(r: u32, ) -> Weight {
        (54_009_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((443_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Kyc SwordHolderOf (r:1 w:1)
    // Storage: Kyc RecordsOf (r:1 w:0)
    fn kill_sword_holder(r: u32, ) -> Weight {
        (53_762_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((447_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Kyc ApplicationFormList (r:1 w:1)
    // Storage: Kyc KYCOf (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    // Storage: Kyc RecordsOf (r:2 w:2)
    // Storage: Kyc MessageList (r:1 w:1)
    fn request_judgement(_n: u32, ) -> Weight {
        (74_575_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
    }
    // Storage: Kyc IASListOf (r:1 w:0)
    // Storage: Kyc ApplicationFormList (r:1 w:1)
    // Storage: Kyc KYCOf (r:1 w:1)
    // Storage: Kyc RecordsOf (r:2 w:2)
    // Storage: Kyc UniqueIdOf (r:1 w:1)
    // Storage: Kyc MessageList (r:1 w:1)
    fn ias_provide_judgement(_n: u32, ) -> Weight {
        (72_180_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(7 as Weight))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
    }
    // Storage: Kyc SwordHolderOf (r:1 w:0)
    // Storage: Kyc UniqueIdOf (r:1 w:0)
    // Storage: Kyc ApplicationFormList (r:1 w:1)
    // Storage: Kyc KYCOf (r:1 w:1)
    // Storage: System Account (r:3 w:3)
    // Storage: Kyc AreaData (r:1 w:1)
    // Storage: Kyc RecordsOf (r:2 w:2)
    fn sword_holder_provide_judgement(n: u32, ) -> Weight {
        (111_935_000 as Weight)
            // Standard Error: 3_000
            .saturating_add((13_000 as Weight).saturating_mul(n as Weight))
            .saturating_add(T::DbWeight::get().reads(10 as Weight))
            .saturating_add(T::DbWeight::get().writes(8 as Weight))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    // Storage: Kyc BlackListOf (r:1 w:0)
    // Storage: Kyc KYCOf (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn set_kyc(r: u32, ) -> Weight {
        (50_854_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((78_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(RocksDbWeight::get().reads(3 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    // Storage: Kyc ApplicationFormList (r:1 w:0)
    // Storage: Kyc KYCOf (r:1 w:1)
    fn clear_kyc(r: u32, ) -> Weight {
        (31_637_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((58_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    // Storage: Kyc BlackListOf (r:1 w:1)
    // Storage: Kyc KYCOf (r:1 w:0)
    // Storage: Kyc ApplicationFormList (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    fn remove_kyc(r: u32, ) -> Weight {
        (74_397_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((78_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(RocksDbWeight::get().reads(4 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    // Storage: Kyc BlackListOf (r:1 w:0)
    // Storage: Kyc KYCOf (r:1 w:0)
    // Storage: Kyc ApplicationFormList (r:1 w:1)
    // Storage: Kyc Nonce (r:1 w:1)
    // Storage: RandomnessCollectiveFlip RandomMaterial (r:1 w:0)
    // Storage: Kyc IASListOf (r:1 w:0)
    // Storage: Kyc SwordHolderOf (r:1 w:0)
    fn apply_certification(r: u32, ) -> Weight {
        (61_765_000 as Weight)
            // Standard Error: 2_000
            .saturating_add((608_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(RocksDbWeight::get().reads(7 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    // Storage: Kyc IASListOf (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn add_ias(r: u32, ) -> Weight {
        (45_331_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((316_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    // Storage: Kyc SwordHolderOf (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn add_sword_holder(r: u32, ) -> Weight {
        (45_375_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((318_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(2 as Weight))
    }
    // Storage: Kyc IASListOf (r:1 w:1)
    fn ias_set_fee(r: u32, ) -> Weight {
        (30_062_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((547_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    // Storage: Kyc SwordHolderOf (r:1 w:1)
    fn sword_holder_set_fee(r: u32, ) -> Weight {
        (30_587_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((543_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(RocksDbWeight::get().reads(1 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    // Storage: Kyc IASListOf (r:1 w:1)
    // Storage: Kyc RecordsOf (r:1 w:0)
    fn kill_ias(r: u32, ) -> Weight {
        (54_009_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((443_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    // Storage: Kyc SwordHolderOf (r:1 w:1)
    // Storage: Kyc RecordsOf (r:1 w:0)
    fn kill_sword_holder(r: u32, ) -> Weight {
        (53_762_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((447_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(RocksDbWeight::get().reads(2 as Weight))
            .saturating_add(RocksDbWeight::get().writes(1 as Weight))
    }
    // Storage: Kyc ApplicationFormList (r:1 w:1)
    // Storage: Kyc KYCOf (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    // Storage: Kyc RecordsOf (r:2 w:2)
    // Storage: Kyc MessageList (r:1 w:1)
    fn request_judgement(_n: u32, ) -> Weight {
        (74_575_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(6 as Weight))
            .saturating_add(RocksDbWeight::get().writes(6 as Weight))
    }
    // Storage: Kyc IASListOf (r:1 w:0)
    // Storage: Kyc ApplicationFormList (r:1 w:1)
    // Storage: Kyc KYCOf (r:1 w:1)
    // Storage: Kyc RecordsOf (r:2 w:2)
    // Storage: Kyc UniqueIdOf (r:1 w:1)
    // Storage: Kyc MessageList (r:1 w:1)
    fn ias_provide_judgement(_n: u32, ) -> Weight {
        (72_180_000 as Weight)
            .saturating_add(RocksDbWeight::get().reads(7 as Weight))
            .saturating_add(RocksDbWeight::get().writes(6 as Weight))
    }
    // Storage: Kyc SwordHolderOf (r:1 w:0)
    // Storage: Kyc UniqueIdOf (r:1 w:0)
    // Storage: Kyc ApplicationFormList (r:1 w:1)
    // Storage: Kyc KYCOf (r:1 w:1)
    // Storage: System Account (r:3 w:3)
    // Storage: Kyc AreaData (r:1 w:1)
    // Storage: Kyc RecordsOf (r:2 w:2)
    fn sword_holder_provide_judgement(n: u32, ) -> Weight {
        (111_935_000 as Weight)
            // Standard Error: 3_000
            .saturating_add((13_000 as Weight).saturating_mul(n as Weight))
            .saturating_add(RocksDbWeight::get().reads(10 as Weight))
            .saturating_add(RocksDbWeight::get().writes(8 as Weight))
    }
}