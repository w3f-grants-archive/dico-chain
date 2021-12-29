//! A set of constant values used in substrate runtime.
#![cfg_attr(not(feature = "std"), no_std)]
/// Money matters.
pub use super::{Balance, BlockNumber, Moment};

pub mod currency {
	use super::Balance;

	pub const MICROCENTS: Balance = 1_000_000;
	pub const MILLICENTS: Balance = 1_000 * MICROCENTS;
	pub const CENTS: Balance = 1_000 * MILLICENTS; // assume this is worth about a cent.
	pub const DOLLARS: Balance = 100 * CENTS;
	// kilo
	pub const KILODOLLARS: Balance = 1000 * DOLLARS;
	// million
	pub const MILLIONDOLLARS: Balance = 1000 * KILODOLLARS;
	// billion
	pub const BILLIONDOLLARS: Balance = 1000 * MILLIONDOLLARS;

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 15 * CENTS + (bytes as Balance) * 6 * CENTS
	}
}

/// Time.
pub mod time {
	use super::{BlockNumber, Moment};
	/// Since BABE is probabilistic this is the average expected block time that
	/// we are targeting. Blocks will be produced at a minimum duration defined
	/// by `SLOT_DURATION`, but some slots will not be allocated to any
	/// authority and hence no block will be produced. We expect to have this
	/// block time on average following the defined slot duration and the value
	/// of `c` configured for BABE (where `1 - c` represents the probability of
	/// a slot being empty).
	/// This value is only used indirectly to define the unit constants below
	/// that are expressed in blocks. The rest of the code should use
	/// `SLOT_DURATION` instead (like the Timestamp pallet for calculating the
	/// minimum period).
	///
	/// If using BABE with secondary slots (default) then all of the slots will
	/// always be assigned, in which case `MILLISECS_PER_BLOCK` and
	/// `SLOT_DURATION` should have the same value.
	///
	/// <https://research.web3.foundation/en/latest/polkadot/block-production/Babe.html#-6.-practical-results>
	pub const MILLISECS_PER_BLOCK: Moment = 12000;
	pub const SECS_PER_BLOCK: Moment = MILLISECS_PER_BLOCK / 1000;

	pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

	// 1 in 4 blocks (on average, not counting collisions) will be primary BABE
	// blocks.
	pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

	pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 10 * MINUTES;
	pub const EPOCH_DURATION_IN_SLOTS: u64 = {
		const SLOT_FILL_RATE: f64 = MILLISECS_PER_BLOCK as f64 / SLOT_DURATION as f64;

		(EPOCH_DURATION_IN_BLOCKS as f64 * SLOT_FILL_RATE) as u64
	};

	// These time units are defined in number of blocks.
	pub const MINUTES: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);
	pub const HOURS: BlockNumber = MINUTES * 60;
	pub const DAYS: BlockNumber = HOURS * 24;
}

pub mod parachains {

	/// native
	pub mod native {
		pub mod KICO {
			pub const AssetId: u32 = 0;
			pub const TokenSymbol: &[u8] = "KICO".as_bytes();
		}
		pub mod LT {
			pub const AssetId: u32 = 100;
			pub const TokenSymbol: &[u8] = "LT".as_bytes();
		}
	}

	/// kusama
	pub mod kusama {
		pub mod KSM {
			pub const AssetId: u32 = 1;
		}
	}

	/// listen
	pub mod listen {
		pub const PARA_ID: u32 = 1002;
		pub mod LTP {
			pub const AssetId: u32 = 101;
			pub const TokenSymbol: &[u8] = "LTP".as_bytes();
		}
	}
}
