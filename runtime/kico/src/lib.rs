#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use codec::{Decode, Encode};
use dico_primitives::{
	constants::{currency::*, time::*},
	AccountId, AccountIndex, Address, Amount, AssetId, Balance, BlockNumber, CurrencyId, Hash, Header, Index, Moment,
	PoolId, Price, Signature,
};
use orml_traits::{create_median_value_data_provider, parameter_type_with_key, DataFeeder, DataProviderExtended};
use pallet_currencies::BasicCurrencyAdapter;
use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	offchain::storage_lock::BlockNumberProvider,
	traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, IdentifyAccount, Verify, Zero},
	transaction_validity::{TransactionPriority, TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, DispatchResult, MultiSignature, Percent,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use static_assertions::const_assert;

// A few exports that help ease life for downstream crates.
use frame_support::{
	construct_runtime, match_type, parameter_types,
	traits::{
		All, Currency, Imbalance, InstanceFilter, KeyOwnerProofSystem, LockIdentifier, MaxEncodedLen, OnUnbalanced,
		U128CurrencyToVote,
	},
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
		DispatchClass, IdentityFee, Weight,
	},
	PalletId, RuntimeDebug,
};
use frame_system::limits::{BlockLength, BlockWeights};
use frame_system::{EnsureOneOf, EnsureRoot};
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::u32_trait::{_1, _2, _3, _4, _5};
use sp_runtime::curve::PiecewiseLinear;

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{MultiAddress, Perbill, Permill};

// Polkadot & XCM imports
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;
use xcm::v0::{BodyId, Junction::*, MultiAsset, MultiLocation, MultiLocation::*, NetworkId, Xcm};
use xcm_builder::{
	AccountId32Aliases, AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom, CurrencyAdapter, EnsureXcmOrigin,
	FixedWeightBounds, IsConcrete, LocationInverter, NativeAsset, ParentAsSuperuser, ParentIsDefault,
	RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountId32AsNative,
	SignedToAccountId32, SovereignSignedViaLocation, TakeWeightCredit, UsingComponents,
};
use xcm_executor::{Config, XcmExecutor};

pub use pallet_amm;
pub use pallet_farm;
pub use pallet_farm_extend;
pub use pallet_kyc;
pub use pallet_lbp;
pub use pallet_pricedao;
/// Import the template pallet.
pub use pallet_template;

use pallet_farm_rpc_runtime_api as farm_rpc;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive =
	frame_executive::Executive<Runtime, Block, frame_system::ChainContext<Runtime>, Runtime, AllPallets>;

impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
	}
}

/// This runtime version.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("KICO"),
	impl_name: create_runtime_str!("KICO"),
	authoring_version: 1,
	spec_version: 1,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

/// We assume that ~10% of the block weight is consumed by `on_initalize` handlers.
/// This is used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 0.5 of a second of compute with a 12 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND / 2;

parameter_types! {
	pub const BlockHashCount: BlockNumber = 250;
	pub const Version: RuntimeVersion = VERSION;
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub const SS58Prefix: u16 = 42;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type Call = Call;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = AccountIdLookup<AccountId, ()>;
	/// The index type for storing how many extrinsics an account has signed.
	type Index = Index;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The header type.
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// The ubiquitous event type.
	type Event = Event;
	/// The ubiquitous origin type.
	type Origin = Origin;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// Runtime version.
	type Version = Version;
	/// Converts a module to an index of this module in the runtime.
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = ();
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = ();
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = ();
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = RuntimeBlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = RuntimeBlockLength;
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
	/// The action to take on a Runtime Upgrade
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

impl pallet_utility::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub MultisigDepositBase: Balance = deposit(1, 88);
	pub MultisigDepositFactor: Balance = deposit(0, 32);
	pub const MaxSignatories: u16 = 100;
}

impl pallet_multisig::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Balances;
	type DepositBase = MultisigDepositBase;
	type DepositFactor = MultisigDepositFactor;
	type MaxSignatories = MaxSignatories;
	type WeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 1 * CENTS;
	pub const TransferFee: u128 = 1 * CENTS;
	pub const CreationFee: u128 = 1 * CENTS;
	pub const TransactionByteFee: u128 = 1 * MILLICENTS;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
}

impl pallet_transaction_payment::Config for Runtime {
	type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = ();
}

// #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug,
// MaxEncodedLen)] pub enum ProxyType {
// 	Any,
// }
// impl Default for ProxyType {
// 	fn default() -> Self {
// 		Self::Any
// 	}
// }
//
// impl InstanceFilter<Call> for ProxyType {
// 	fn filter(&self, _c: &Call) -> bool {
// 		match self {
// 			ProxyType::Any => true,
// 		}
// 	}
// 	fn is_superset(&self, o: &Self) -> bool {
// 		match (self, o) {
// 			(ProxyType::Any, _) => true,
// 		}
// 	}
// }

// parameter_types! {
// 	// One storage item; key size 32, value size 8; .
// 	pub const ProxyDepositBase: Balance = deposit(1, 40);
// 	// Additional storage item size of 33 bytes.
// 	pub const ProxyDepositFactor: Balance = deposit(0, 33);
// 	pub const MaxProxies: u16 = 32;
// 	// One storage item; key size 32, value size 16
// 	pub const AnnouncementDepositBase: Balance = deposit(1, 48);
// 	pub const AnnouncementDepositFactor: Balance = deposit(0, 66);
// 	pub const MaxPending: u16 = 32;
// }

// impl pallet_proxy::Config for Runtime {
// 	type Event = Event;
// 	type Call = Call;
// 	type Currency = Balances;
// 	type ProxyType = ProxyType;
// 	type ProxyDepositBase = ProxyDepositBase;
// 	type ProxyDepositFactor = ProxyDepositFactor;
// 	type MaxProxies = MaxProxies;
// 	type WeightInfo = pallet_proxy::weights::SubstrateWeight<Runtime>;
// 	type MaxPending = MaxPending;
// 	type CallHasher = BlakeTwo256;
// 	type AnnouncementDepositBase = AnnouncementDepositBase;
// 	type AnnouncementDepositFactor = AnnouncementDepositFactor;
// }

impl pallet_randomness_collective_flip::Config for Runtime {}

impl pallet_sudo::Config for Runtime {
	type Call = Call;
	type Event = Event;
}

parameter_types! {
	pub const LaunchPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
	pub const VotingPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
	pub const FastTrackVotingPeriod: BlockNumber = 3 * 24 * 60 * MINUTES;
	pub const InstantAllowed: bool = true;
	pub const MinimumDeposit: Balance = 100 * DOLLARS;
	pub const EnactmentPeriod: BlockNumber = 30 * 24 * 60 * MINUTES;
	pub const CooloffPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
	// One cent: $10,000 / MB
	pub const PreimageByteDeposit: Balance = 1 * CENTS;
	pub const MaxVotes: u32 = 100;
	pub const MaxProposals: u32 = 100;
}

impl pallet_democracy::Config for Runtime {
	type Proposal = Call;
	type Event = Event;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriod;
	type VotingPeriod = VotingPeriod;
	type MinimumDeposit = MinimumDeposit;
	/// A straight majority of the council can decide what their next motion is.
	type ExternalOrigin = pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, CouncilCollective>;
	/// A super-majority can have the next scheduled referendum be a straight majority-carries vote.
	type ExternalMajorityOrigin = pallet_collective::EnsureProportionAtLeast<_3, _4, AccountId, CouncilCollective>;
	/// A unanimous council can have the next scheduled referendum be a straight default-carries
	/// (NTB) vote.
	type ExternalDefaultOrigin = pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, CouncilCollective>;
	/// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
	/// be tabled immediately and with a shorter voting/enactment period.
	type FastTrackOrigin = pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, TechnicalCollective>;
	type InstantOrigin = pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, TechnicalCollective>;
	type InstantAllowed = InstantAllowed;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	// To cancel a proposal which has been passed, 2/3 of the council must agree to it.
	type CancellationOrigin = pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, CouncilCollective>;
	// To cancel a proposal before it has been passed, the technical committee must be unanimous or
	// Root must agree.
	type CancelProposalOrigin = EnsureOneOf<
		AccountId,
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, TechnicalCollective>,
	>;
	type BlacklistOrigin = EnsureRoot<AccountId>;
	// Any single technical committee member may veto a coming council proposal, however they can
	// only do it once and it lasts only for the cool-off period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollective>;
	type CooloffPeriod = CooloffPeriod;
	type PreimageByteDeposit = PreimageByteDeposit;
	type OperationalPreimageOrigin = pallet_collective::EnsureMember<AccountId, CouncilCollective>;
	type Slash = Treasury;
	type Scheduler = Scheduler;
	type PalletsOrigin = OriginCaller;
	type MaxVotes = MaxVotes;
	type WeightInfo = pallet_democracy::weights::SubstrateWeight<Runtime>;
	type MaxProposals = MaxProposals;
}

parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const CandidacyBond: Balance = 10 * DOLLARS;
	// 1 storage item created, key size is 32 bytes, value size is 16+16.
	pub const VotingBondBase: Balance = deposit(1, 64);
	// additional data per vote is 32 bytes (account id).
	pub const VotingBondFactor: Balance = deposit(0, 32);
	pub const TermDuration: BlockNumber = 7 * DAYS;
	pub const DesiredMembers: u32 = 13;
	pub const DesiredRunnersUp: u32 = 7;
	pub const ElectionsPhragmenPalletId: LockIdentifier = *b"phrelect";
}

// Make sure that there are no more than `MaxMembers` members elected via elections-phragmen.
const_assert!(DesiredMembers::get() <= CouncilMaxMembers::get());

impl pallet_elections_phragmen::Config for Runtime {
	type Event = Event;
	type PalletId = ElectionsPhragmenPalletId;
	type Currency = Balances;
	type ChangeMembers = Council;
	// NOTE: this implies that council's genesis members cannot be set directly and must come from
	// this module.
	type InitializeMembers = Council;
	type CurrencyToVote = U128CurrencyToVote;
	type CandidacyBond = CandidacyBond;
	type VotingBondBase = VotingBondBase;
	type VotingBondFactor = VotingBondFactor;
	type LoserCandidate = ();
	type KickedMember = ();
	type DesiredMembers = DesiredMembers;
	type DesiredRunnersUp = DesiredRunnersUp;
	type TermDuration = TermDuration;
	type WeightInfo = pallet_elections_phragmen::weights::SubstrateWeight<Runtime>;
}

impl pallet_bounties::Config for Runtime {
	type Event = Event;
	type BountyDepositBase = BountyDepositBase;
	type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
	type BountyUpdatePeriod = BountyUpdatePeriod;
	type BountyCuratorDeposit = BountyCuratorDeposit;
	type BountyValueMinimum = BountyValueMinimum;
	type DataDepositPerByte = DataDepositPerByte;
	type MaximumReasonLength = MaximumReasonLength;
	type WeightInfo = pallet_bounties::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const TechnicalMotionDuration: BlockNumber = 5 * DAYS;
	pub const TechnicalMaxProposals: u32 = 100;
	pub const TechnicalMaxMembers: u32 = 100;
}

type TechnicalCollective = pallet_collective::Instance2;
impl pallet_collective::Config<TechnicalCollective> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = TechnicalMotionDuration;
	type MaxProposals = TechnicalMaxProposals;
	type MaxMembers = TechnicalMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

type EnsureRootOrHalfCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>,
>;
impl pallet_membership::Config<pallet_membership::Instance1> for Runtime {
	type Event = Event;
	type AddOrigin = EnsureRootOrHalfCouncil;
	type RemoveOrigin = EnsureRootOrHalfCouncil;
	type SwapOrigin = EnsureRootOrHalfCouncil;
	type ResetOrigin = EnsureRootOrHalfCouncil;
	type PrimeOrigin = EnsureRootOrHalfCouncil;
	type MembershipInitialized = TechnicalCommittee;
	type MembershipChanged = TechnicalCommittee;
	type MaxMembers = TechnicalMaxMembers;
	type WeightInfo = pallet_membership::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: Balance = 1 * DOLLARS;
	pub const SpendPeriod: BlockNumber = 1 * DAYS;
	pub const Burn: Permill = Permill::from_percent(50);
	pub const TipCountdown: BlockNumber = 1 * DAYS;
	pub const TipFindersFee: Percent = Percent::from_percent(20);
	pub const TipReportDepositBase: Balance = 1 * DOLLARS;
	pub const DataDepositPerByte: Balance = 1 * CENTS;
	pub const BountyDepositBase: Balance = 1 * DOLLARS;
	pub const BountyDepositPayoutDelay: BlockNumber = 1 * DAYS;
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const BountyUpdatePeriod: BlockNumber = 14 * DAYS;
	pub const MaximumReasonLength: u32 = 16384;
	pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
	pub const BountyValueMinimum: Balance = 5 * DOLLARS;
	pub const MaxApprovals: u32 = 100;
}

impl pallet_treasury::Config for Runtime {
	type PalletId = TreasuryPalletId;
	type Currency = Balances;
	type ApproveOrigin = EnsureOneOf<
		AccountId,
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<_3, _5, AccountId, CouncilCollective>,
	>;
	type RejectOrigin = EnsureOneOf<
		AccountId,
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>,
	>;
	type Event = Event;
	type OnSlash = ();
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BurnDestination = ();
	type SpendFunds = Bounties;
	type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
	type MaxApprovals = MaxApprovals;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
		RuntimeBlockWeights::get().max_block;
	pub const MaxScheduledPerBlock: u32 = 50;
}

impl pallet_scheduler::Config for Runtime {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = MaxScheduledPerBlock;
	type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const UncleGenerations: u32 = 0;
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type UncleGenerations = UncleGenerations;
	type FilterUncle = ();
	type EventHandler = CollatorSelection;
}

parameter_types! {
	pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
	pub const Period: u32 = 6 * HOURS;
	pub const Offset: u32 = 0;
}

impl pallet_session::Config for Runtime {
	type Event = Event;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	// we don't have stash and controller, thus we don't need the convert as well.
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
	type SessionManager = CollatorSelection;
	// Essentially just Aura, but lets be pedantic.
	type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
	type WeightInfo = ();
}

parameter_types! {
	pub const PotId: PalletId = PalletId(*b"dico/pcs");
	pub const MaxCandidates: u32 = 1000;
	pub const MinCandidates: u32 = 1;
	pub const MaxInvulnerables: u32 = 100;
}

type EnsureRootOrMoreThanHalfCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>,
>;

impl pallet_collator_selection::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type UpdateOrigin = EnsureRootOrMoreThanHalfCouncil;
	type PotId = PotId;
	type MaxCandidates = MaxCandidates;
	type MinCandidates = MinCandidates;
	type MaxInvulnerables = MaxInvulnerables;
	// should be a multiple of session or things will get inconsistent
	type KickThreshold = Period;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ValidatorRegistration = Session;
	type WeightInfo = ();
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type Event = Event;
	type OnValidationData = ();
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type OutboundXcmpMessageSource = XcmpQueue;
	type DmpMessageHandler = DmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}

parameter_types! {
	pub const RelayLocation: MultiLocation = X1(Parent);
	pub const RelayNetwork: NetworkId = NetworkId::Polkadot;
	pub RelayOrigin: Origin = cumulus_pallet_xcm::Origin::Relay.into();
	pub Ancestry: MultiLocation = X1(Parachain(ParachainInfo::parachain_id().into()));
}

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the default `AccountId`.
	ParentIsDefault<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
	AccountId32Aliases<RelayNetwork, AccountId>,
);

/// Means for transacting assets on this chain.
pub type LocalAssetTransactor = CurrencyAdapter<
	// Use this currency:
	Balances,
	// Use this currency when it is a fungible asset matching the given location or name:
	IsConcrete<RelayLocation>,
	// Do a simple punn to convert an AccountId32 MultiLocation into a native chain account ID:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We don't track any teleports.
	(),
>;

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId, Origin>,
	// Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
	// recognised.
	RelayChainAsNative<RelayOrigin, Origin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognised.
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
	// Superuser converter for the Relay-chain (Parent) location. This will allow it to issue a
	// transaction from the Root origin.
	ParentAsSuperuser<Origin>,
	// Native signed account converter; this just converts an `AccountId32` origin into a normal
	// `Origin::Signed` origin of the same 32-byte value.
	SignedAccountId32AsNative<RelayNetwork, Origin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	XcmPassthrough<Origin>,
);

parameter_types! {
	// One XCM operation is 1_000_000 weight - almost certainly a conservative estimate.
	pub UnitWeightCost: Weight = 1_000_000;
	// One UNIT buys 1 second of weight.
	pub const WeightPrice: (MultiLocation, u128) = (X1(Parent), DOLLARS);
}

match_type! {
	pub type ParentOrParentsUnitPlurality: impl Contains<MultiLocation> = {
		X1(Parent) | X2(Parent, Plurality { id: BodyId::Unit, .. })
	};
}

pub type Barrier = (
	TakeWeightCredit,
	AllowTopLevelPaidExecutionFrom<All<MultiLocation>>,
	AllowUnpaidExecutionFrom<ParentOrParentsUnitPlurality>,
	// ^^^ Parent & its unit plurality gets free execution
);

pub struct XcmConfig;
impl Config for XcmConfig {
	type Call = Call;
	type XcmSender = XcmRouter;
	// How to withdraw and deposit an asset.
	type AssetTransactor = LocalAssetTransactor;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	type IsReserve = NativeAsset;
	type IsTeleporter = NativeAsset; // <- should be enough to allow teleportation of UNIT
	type LocationInverter = LocationInverter<Ancestry>;
	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<UnitWeightCost, Call>;
	type Trader = UsingComponents<IdentityFee<Balance>, RelayLocation, AccountId, Balances, ()>;
	type ResponseHandler = (); // Don't handle responses for now.
}

/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = SignedToAccountId32<Origin, AccountId, RelayNetwork>;

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
);

impl pallet_xcm::Config for Runtime {
	type Event = Event;
	type SendXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmExecuteFilter = All<(MultiLocation, Xcm<Call>)>;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = All<(MultiLocation, Vec<MultiAsset>)>;
	type XcmReserveTransferFilter = ();
	type Weigher = FixedWeightBounds<UnitWeightCost, Call>;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ChannelInfo = ParachainSystem;
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ExecuteOverweightOrigin = frame_system::EnsureRoot<AccountId>;
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
}

/// Configure the pallet template in pallets/template.
impl pallet_template::Config for Runtime {
	type Event = Event;
}

parameter_types! {
	pub const KYCPalletId: PalletId = PalletId(*b"dico/kyc");
	pub const MaxIAS: u32 = 200;
	pub const MaxSwordHolder: u32 = 200;
	pub const KYCBasicDeposit: Balance = 100 * DOLLARS;
	pub const KYCServiceDeposit: Balance = 10000 * DOLLARS;
	pub const AmmPalletId: PalletId = PalletId(*b"dico/amm");
	pub const FarmPalletId: PalletId = PalletId(*b"dico/fam");
	pub const LBPPalletId: PalletId = PalletId(*b"dico/lbp");
	pub const FarmExtendPalletId: PalletId = PalletId(*b"dico/fme");
}

/// Configure the pallet template in pallets/template.
impl pallet_kyc::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type PalletId = KYCPalletId;
	type BasicDeposit = KYCBasicDeposit;
	type ServiceDeposit = KYCServiceDeposit;
	type MaxIAS = MaxIAS;
	type MaxSwordHolder = MaxSwordHolder;
	type Slashed = Treasury;
	type Randomness = RandomnessCollectiveFlip;
	type ForceOrigin = EnsureRootOrHalfCouncil;
	type IASOrigin = EnsureRootOrHalfCouncil;
	type SwordHolderOrigin = EnsureRootOrHalfCouncil;
	type WeightInfo = pallet_kyc::weights::SubstrateWeight<Runtime>;
}

impl pallet_amm::Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type LiquidityAssetIdBase = MaxCreatableCurrencyId;
	type PalletId = AmmPalletId;
	type WeightInfo = pallet_amm::weights::DicoWeight<Runtime>;
}

impl pallet_lbp::Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type PalletId = LBPPalletId;
	type LbpId = u32;
	type WeightInfo = pallet_lbp::weights::DicoWeight<Runtime>;
	type TreasuryHandler = DicoTreasury;
	type FounderSetOrigin = pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>;
}

impl pallet_farm::Config for Runtime {
	type Event = Event;
	type PoolId = u32;
	type Currency = Currencies;
	type FounderSetOrigin = pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>;
	type NativeAssetId = DICOAssetId;
	type PalletId = FarmPalletId;
	type WeightInfo = pallet_farm::weights::DicoWeight<Runtime>;
}

impl pallet_farm_extend::Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type PalletId = FarmExtendPalletId;
	type PoolExtendId = u32;
	type WeightInfo = pallet_farm_extend::weights::DicoWeight<Runtime>;
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: AssetId| -> Balance {
		Zero::zero()
	};
}

impl orml_tokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = AssetId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = ();
	type MaxLocks = MaxLocks;
	type DustRemovalWhitelist = ();
}

parameter_types! {
	pub MinVestedTransfer: Balance = 0;
	pub const MaxVestingSchedules: u32 = 200;
}

pub struct RelaychainBlockNumberProvider<T>(sp_std::marker::PhantomData<T>);

impl<T: cumulus_pallet_parachain_system::Config> BlockNumberProvider for RelaychainBlockNumberProvider<T> {
	type BlockNumber = BlockNumber;

	fn current_block_number() -> Self::BlockNumber {
		cumulus_pallet_parachain_system::Pallet::<T>::validation_data()
			.map(|d| d.relay_parent_number)
			.unwrap_or_default()
	}
}

impl orml_vesting::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type MinVestedTransfer = MinVestedTransfer;
	type VestedTransferOrigin = frame_system::EnsureSigned<AccountId>;
	type WeightInfo = ();
	type MaxVestingSchedules = MaxVestingSchedules;
	type BlockNumberProvider = RelaychainBlockNumberProvider<Runtime>;
}

parameter_types! {
	pub const CreateConsume: Balance = 100 * DOLLARS;
	pub const DICOAssetId: AssetId = 0;
	pub const MaxCreatableCurrencyId: AssetId = 4_000_000_000;
}

impl pallet_currencies::Config for Runtime {
	type Event = Event;
	type MultiCurrency = Tokens;

	type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;

	type GetNativeCurrencyId = DICOAssetId;

	type WeightInfo = ();

	type CreateConsume = CreateConsume;
	type MaxCreatableCurrencyId = MaxCreatableCurrencyId;
}

// price data
/// price
parameter_types! {
	pub const MaxOracleSize: u32 = 5;
	pub const MinimumCount: u32 = 3;  // todo: The minimum number is 3
	pub const ExpiresIn: Moment = 1000 * 60 * 60; // todo: 60 mins
	pub ZeroAccountId: AccountId = AccountId::from([0u8; 32]);
	pub const FeedPledgedBalance: Balance = 500 * DOLLARS;  // todo : pledge 500 dico?
	pub const withdrawExpirationPeriod: BlockNumber = 10 * MINUTES;   // TODO: 5 * DAYS;
}

type DicoDataProvider = pallet_oracle::Instance1;
impl pallet_oracle::Config<DicoDataProvider> for Runtime {
	type Event = Event;
	type OnNewData = ();
	type CombineData = pallet_oracle::DefaultCombineData<Runtime, MinimumCount, ExpiresIn, DicoDataProvider>;
	type Time = Timestamp;
	type OracleKey = CurrencyId;
	type OracleValue = Price;
	type MaxOracleSize = MaxOracleSize;
	type RootOperatorAccountId = ZeroAccountId;
	type WeightInfo = pallet_oracle::weights::OracleWeight<Runtime>;
}

pub type TimeStampedPrice = pallet_oracle::TimestampedValue<Price, Moment>;
create_median_value_data_provider!(
	AggregatedDataProvider,
	CurrencyId,
	Price,
	TimeStampedPrice,
	[DicoOracle]
);

// Aggregated data provider cannot feed.
impl DataFeeder<CurrencyId, Price, AccountId> for AggregatedDataProvider {
	fn feed_value(_: AccountId, _: CurrencyId, _: Price) -> DispatchResult {
		Err("Not supported".into())
	}
}

parameter_types! {
	pub const DicoTreasuryModuleId: PalletId = PalletId(*b"dico/trs");   // todo: modify name
}

type EnsureRootOrTwoThirdsGeneralCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>,
>;

impl pallet_pricedao::Config for Runtime {
	type Event = Event;
	type Source = AggregatedDataProvider;
	type FeedOrigin = EnsureRootOrTwoThirdsGeneralCouncil;
	type UpdateOraclesStorgage = DicoOracle;
	type DicoTreasuryModuleId = DicoTreasuryModuleId;
	type BaseCurrency = Balances;
	type PledgedBalance = FeedPledgedBalance;
	type WithdrawExpirationPeriod = withdrawExpirationPeriod;
	type WeightInfo = pallet_pricedao::weights::PriceWeight<Runtime>;
}

parameter_types! {
	pub const DicoProposalBond: Balance = 100 * DOLLARS;
	pub const DicoSpendPeriod: BlockNumber = 7 * DAYS;

}

impl pallet_dico_treasury::Config for Runtime {
	type ApproveOrigin = EnsureOneOf<
		AccountId,
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, CouncilCollective>,
	>;
	type PalletId = TreasuryPalletId;
	type MultiCurrency = Currencies;
	type RejectOrigin = EnsureOneOf<
		AccountId,
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, CouncilCollective>,
	>;

	type Event = Event;
	type GetNativeCurrencyId = DICOAssetId;
	type ProposalBond = DicoProposalBond;
	type SpendPeriod = DicoSpendPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const InitiatorPledge: Balance = 100 * DOLLARS;
	pub const RequestPledge: Balance = 300 * DOLLARS;
	pub const RequestExpire: BlockNumber = 5 * DAYS;
	pub const MinProportion: Percent = Percent::from_percent(0u8);
	pub const IcoTotalReward: Balance = 20_0000_0000 * DOLLARS;
	pub const InitiatorBond: Percent = Percent::from_percent(1u8);
	pub const TerminateProtectPeriod: Percent = Percent::from_percent(33);
	pub const ReleaseProtectPeriod: Percent = Percent::from_percent(33);
	pub const ChillDuration: BlockNumber = 10 * MINUTES;
	pub const InviterRewardProportion: Percent = Percent::from_percent(10u8);
	pub const InviteeRewardProportion: Percent = Percent::from_percent(5u8);
	pub const UsdtCurrencyId: AssetId = 5;

}

impl pallet_ico::Config for Runtime {
	type Event = Event;
	type PermitIcoOrigin = pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, CouncilCollective>;
	type RejectIcoOrigin = pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, CouncilCollective>;
	type PermitReleaseOrigin = pallet_dao::EnsureProportionAtLeast<Runtime, _1, _2, AccountId>;
	type TerminateIcoOrigin = pallet_dao::EnsureProportionAtLeast<Runtime, _1, _2, AccountId>;
	type OnSlash = ();
	type MultiCurrency = Currencies;
	type NativeCurrency = Balances;
	type GetNativeCurrencyId = DICOAssetId;
	type InitiatorPledge = InitiatorPledge;
	type RequestPledge = RequestPledge;
	type RequestExpire = RequestExpire;
	type CurrenciesHandler = Currencies;
	type IcoTotalReward = IcoTotalReward;
	type DicoTreasuryHandler = DicoTreasury;
	type InitiatorBond = InitiatorBond;
	type TerminateProtectPeriod = TerminateProtectPeriod;
	type ReleaseProtectPeriod = ReleaseProtectPeriod;
	type ChillDuration = ChillDuration;
	type InviterRewardProportion = InviterRewardProportion;
	type InviteeRewardProportion = InviteeRewardProportion;
	type PriceData = PriceDao;
	type UsdtCurrencyId = UsdtCurrencyId;
	type KycHandler = Kyc;
}

parameter_types! {
	pub const DicoMotionDuration: BlockNumber = 5 * DAYS;
	pub const DicoMaxProposals: u32 = 100;
}

impl pallet_dao::Config for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = DicoMotionDuration;
	type MaxProposals = DicoMaxProposals;
	type WeightInfo = ();
	type IcoHandler = Ico;
}

parameter_types! {
	pub const MaxClassMetadata: u32 = 1024;
	pub const MaxTokenMetadata: u32 = 1024;
	pub const MaxTokenAttribute: u32 = 1024;

}

impl pallet_nft::Config for Runtime {
	type Event = Event;
	type ClassId = u32;
	type TokenId = u32;
	type Currency = Balances;
	type MaxClassMetadata = MaxClassMetadata;
	type MaxTokenMetadata = MaxTokenMetadata;
	type MaxTokenAttribute = MaxTokenAttribute;
	type PowerHandler = Ico;
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = generic::Block<Header, sp_runtime::OpaqueExtrinsic>,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		// System, Utility
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>} = 0,
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 1,
		Utility: pallet_utility::{Pallet, Call, Event} = 2,
		Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 3,
		Sudo: pallet_sudo::{Pallet, Call, Storage, Config<T>, Event<T>} = 4,
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage} = 5,
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage} = 6,
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 7,
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 9,
		// Council,Membership
		Democracy: pallet_democracy::{Pallet, Call, Storage, Config<T>, Event<T>} = 11,
		Council: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 12,
		TechnicalCommittee: pallet_collective::<Instance2>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>} = 13,
		Elections: pallet_elections_phragmen::{Pallet, Call, Storage, Event<T>, Config<T>} = 14,
		TechnicalMembership: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>} = 15,
		// Governance
		Bounties: pallet_bounties::{Pallet, Call, Storage, Event<T>} = 20,
		Treasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>} = 21,
		// Consensus
		Authorship: pallet_authorship::{Pallet, Call, Storage} = 30,
		CollatorSelection: pallet_collator_selection::{Pallet, Call, Storage, Event<T>, Config<T>} = 31,
		Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 32,
		Aura: pallet_aura::{Pallet, Config<T>, Storage} = 33,
		AuraExt: cumulus_pallet_aura_ext::{Pallet, Config, Storage} = 34,
		//  3rd Party
		Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>} = 40,
		Vesting: orml_vesting::{Pallet, Storage, Call, Event<T>, Config<T>} = 41,

		// XCM helpers.
		XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>} = 50,
		PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin} = 51,
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Call, Event<T>, Origin} = 52,
		DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>} = 53,
		ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Config, Storage, Inherent, Event<T>} = 54,
		ParachainInfo: parachain_info::{Pallet, Storage, Config} = 55,
		//local pallet
		TemplatePallet: pallet_template::{Pallet, Call, Storage, Event<T>} = 60,

		Kyc: pallet_kyc::{Pallet, Call, Storage, Event<T>} = 70,

		DicoTreasury: pallet_dico_treasury::{Pallet, Call, Storage, Event<T>} = 71,
		Dao: pallet_dao::{Pallet, Origin<T>, Event<T>, Call, Storage}  = 72,
		Ico: pallet_ico::{Pallet, Event<T>, Call, Storage} = 73,

		AMM: pallet_amm::{Pallet, Call, Storage, Event<T>} = 80,

		Nft: pallet_nft::{Pallet, Call, Storage, Event<T>} = 81,
		LBP: pallet_lbp::{Pallet, Call, Storage, Event<T>} = 82,
		Farm: pallet_farm::{Pallet, Call, Storage, Event<T>} = 83,
		FarmExtend: pallet_farm_extend::{Pallet, Call, Storage, Event<T>}= 84,
		PriceDao: pallet_pricedao::{Pallet, Call, Storage, Event<T>} = 85,
		Currencies: pallet_currencies::{Pallet, Event<T>, Call, Storage} = 86,
		DicoOracle: pallet_oracle::<Instance1>::{Pallet, Storage, Call, Event<T>}= 87,
	}
);

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			Runtime::metadata().into()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(
			extrinsic: <Block as BlockT>::Extrinsic,
		) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl farm_rpc::FarmApi<Block, AccountId, PoolId, Balance> for Runtime {
		fn get_participant_reward(account: AccountId, pid: PoolId) -> Balance {
			let reward = Farm::get_participant_reward(account, pid);

			reward
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}

		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities()
		}
	}

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info() -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info()
		}
	}

	impl pallet_ico_rpc_runtime_api::IcoAmountApi<Block, AccountId, AssetId, Index, Balance> for Runtime {
		fn can_release_amount(account: AccountId, currency_id: CurrencyId, index: Index) -> Balance {
			Ico::can_release_amount(account, currency_id, index)
		}
		fn get_reward_amount(account: AccountId, currency_id: CurrencyId, index: Index) -> Balance {
			Ico::get_reward_amount(account, currency_id, index)
		}
		fn can_unlock_amount(account: AccountId, currency_id: CurrencyId, index: Index) -> Balance {
			Ico::can_unlock_amount(account, currency_id, index)
		}
		fn can_join_amount(account: AccountId, currency_id: CurrencyId, index: Index) -> (Balance, Balance) {
			Ico::can_join_amount(account, currency_id, index)
		}

		fn get_token_price(currency_id: CurrencyId) -> Balance {
			Ico::get_token_price(currency_id)
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		fn account_nonce(account: AccountId) -> Index {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

			use frame_system_benchmarking::Pallet as SystemBench;
			impl frame_system_benchmarking::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
				// Total Issuance
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
				// Execution Phase
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
				// Event Count
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
				// System Events
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
			add_benchmark!(params, batches, pallet_balances, Balances);
			add_benchmark!(params, batches, pallet_timestamp, Timestamp);
			add_benchmark!(params, batches, amm, AMM);
			add_benchmark!(params, batches, farm, Farm);
			add_benchmark!(params, batches, lbp, LBP);
			add_benchmark!(params, batches, farm_extend, FarmExtend);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}
}

struct CheckInherents;

impl cumulus_pallet_parachain_system::CheckInherents<Block> for CheckInherents {
	fn check_inherents(
		block: &Block,
		relay_state_proof: &cumulus_pallet_parachain_system::RelayChainStateProof,
	) -> sp_inherents::CheckInherentsResult {
		let relay_chain_slot = relay_state_proof
			.read_slot()
			.expect("Could not read the relay chain slot from the proof");

		let inherent_data = cumulus_primitives_timestamp::InherentDataProvider::from_relay_chain_slot_and_duration(
			relay_chain_slot,
			sp_std::time::Duration::from_secs(6),
		)
		.create_inherent_data()
		.expect("Could not create the timestamp inherent data");

		inherent_data.check_extrinsics(&block)
	}
}

cumulus_pallet_parachain_system::register_validate_block!(
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
	CheckInherents = CheckInherents,
);
