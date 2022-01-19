#![cfg_attr(not(feature = "std"), no_std)]

use super::*;
use crate::types::EnumPuzzleStatus::AnswerPeriod;
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use frame_support::sp_runtime::PerThing;
// use frame_support::sp_runtime::serde::__private::Default;
use crate::types::RewardType::NoneReward;
use frame_support::traits::LockIdentifier;
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

pub const DEPOSIT_ID: LockIdentifier = *b"ato/dops";

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub type PositiveImbalanceOf<T> = <<T as Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::PositiveImbalance;

pub type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

pub type PurchaseId = Vec<u8>;

pub type PuzzleSubjectHash = Vec<u8>;

pub type AtoStakingPeriod = u64;

pub type PointToken = u128;

pub type StorageHash = Vec<u8>;

pub type StorageLength = u64;

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct AtoInterestRate {
	pub permill: Permill,
	pub fold: u32,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum EnumPuzzleStatus {
	AnswerPeriod,
	RevealPeriod,
	ChallengePeriod,
	BenefitPeriod,
}

impl Default for EnumPuzzleStatus {
	fn default() -> Self {
		AnswerPeriod
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default, TypeInfo)]
pub struct PotLedgerData<AccountId, BalanceOf, BlockNumber> {
	pub owner: AccountId,
	pub total: BalanceOf,
	pub funds: BalanceOf,
	pub sponsor_list: Vec<SponsorData<AccountId, BalanceOf, BlockNumber>>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default, TypeInfo)]
pub struct SponsorData<AccountId, BalanceOf, BlockNumber> {
	pub sponsor: AccountId,
	pub funds: BalanceOf,
	pub create_bn: BlockNumber,
	pub reason: Vec<u8>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum RewardType {
	NoneReward,
	AnswerReward,
	CreatorReward,
	ChallengerReward,
}

impl Default for RewardType {
	fn default() -> Self {
		NoneReward
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default, TypeInfo)]
pub struct PotRewardData<Account, BlockNumber, BalanceOf, PerVal: PerThing> {
	pub create_bn: BlockNumber,
	pub tax: PerVal,
	pub reward_type: RewardType,
	pub total: BalanceOf,
	pub payout: BalanceOf,
	pub beneficiaries: Vec<(Account, PerVal)>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default, TypeInfo)]
pub struct PuzzleChallengeData<Account, BlockNumber, BalanceOf, PerVal: PerThing> {
	pub raised_total: BalanceOf,
	pub status: ChallengeStatus<BlockNumber, PerVal>,
	pub create_bn: BlockNumber,
	pub creator: Account,
	pub start_bn: Option<BlockNumber>,
	pub end_bn: Option<BlockNumber>,
	pub raised_group: Vec<(Account, BalanceOf)>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum ChallengeStatus<BlockNumber, PerVal: PerThing> {
	Raise(BlockNumber),
	RaiseCompleted(BlockNumber),
	RaiseBackFunds(BlockNumber, PerVal),
	JudgePassed(BlockNumber),
	JudgeRejected(BlockNumber),
}

impl<BlockNumber: Default, PerVal: PerThing> Default for ChallengeStatus<BlockNumber, PerVal> {
	fn default() -> Self {
		Self::Raise(Default::default())
	}
}
