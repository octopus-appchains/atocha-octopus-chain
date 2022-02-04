#![cfg_attr(not(feature = "std"), no_std)]

use super::*;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::Currency;
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;
use sp_std::vec::Vec;

pub type PuzzleSubjectHash = Vec<u8>;
pub type PuzzleAnswerExplain = Vec<u8>;
pub type PuzzleSponsorExplain = Vec<u8>;
pub type AnswerStatus = u8;
pub type PuzzleAnswerHash = Vec<u8>;

// pub type PuzzleAnswerOption = Option<PuzzleAnswerHash>;
// pub type PuzzleTicket = u64;

// pub type PuzzleAnswerSigned = Vec<u8>;
// pub type PuzzleAnswerNonce = Vec<u8>;
pub type CreateBn<B> = B;
pub type DurationBn<B> = B;
pub type RevealBn<B> = B;
pub type PuzzleVersion = u64;

pub type BalanceOf<T> =
<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub const ANSWER_STATUS_IS_EXPECT: AnswerStatus = 1;

// Default maximum is a week.
pub const MAXIMUM_WAITING_BLOCK_NUM: u64 = 100800;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct PuzzleInfoData<Account, BlockNumber> {
	pub account: Account,
	pub answer_hash: PuzzleAnswerHash,
	// pub answer_option: Option<PuzzleAnswerHash>,
	// pub answer_explain: Option<PuzzleAnswerExplain>,
	// pub answer_signed: PuzzleAnswerSigned,
	// pub answer_nonce: PuzzleAnswerNonce,
	// pub puzzle_ticket: PuzzleTicket,
	pub puzzle_status: PuzzleStatus,
	pub create_bn: CreateBn<BlockNumber>,
	pub reveal_answer: Option<Account>,
	pub reveal_bn: Option<RevealBn<BlockNumber>>,
	pub puzzle_version: PuzzleVersion,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum PuzzleStatus {
	PUZZLE_STATUS_IS_SOLVING,
	PUZZLE_STATUS_IS_FINAL,
	PUZZLE_STATUS_IS_SOLVED,
}

impl Default for PuzzleStatus {
	fn default() -> Self {
		Self::PUZZLE_STATUS_IS_SOLVING
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct PuzzleAnswerData<Account, BlockNumber> {
	pub account: Account,
	pub answer_status: PuzzleAnswerStatus,
	pub answer_explain: PuzzleAnswerExplain,
	pub create_bn: CreateBn<BlockNumber>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum PuzzleAnswerStatus {
	ANSWER_HASH_IS_MISMATCH,
	ANSWER_HASH_IS_MATCH,
	ANSWER_STATUS_IS_NONE,
}

impl Default for PuzzleAnswerStatus {
	fn default() -> Self {
		Self::ANSWER_STATUS_IS_NONE
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct PointSlashData<Account, PerThing, PointNum> {
	pub who: Account,
	pub rate_cp: PerThing,
	pub total: PointNum,
	pub slash: PointNum,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct ChallengeRewardData<Account, PerThing> {
	pub beneficiaries: Vec<(Account, PerThing)>,
	pub rate_ti: PerThing,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct ConfigData<Balance, BlockNumber, PerThing> {
	pub min_bonus_of_puzzle: Balance, // MinBonusOfPuzzle: Balance = 100 * DOLLARS;
	pub challenge_period_length: BlockNumber, // ChallengePeriodLength: BlockNumber = 2 * MINUTES ; //1 * HOURS;
	pub tax_of_tcr: PerThing, // TaxOfTCR: Perbill = Perbill::from_percent(10);
	pub tax_of_tvs: PerThing, // TaxOfTVS: Perbill = Perbill::from_percent(5); //  When creator reveal puzzle that it tax fee .
	pub tax_of_tvo: PerThing, // TaxOfTVO: Perbill = Perbill::from_percent(10); // When answer reveal puzzle that it tax fee.
	pub tax_of_ti: PerThing, // TaxOfTI: Perbill = Perbill::from_percent(10);
	pub penalty_of_cp: PerThing, // PenaltyOfCP: Perbill = Perbill::from_percent(10);
	pub max_sponsor_explain_len: u32, // const MaxSponsorExplainLen: u32 = 256;
	pub max_answer_explain_len: u32, // const MaxAnswerExplainLen: u32 = 1024;
}

// impl <T: Config> Default for ConfigData<T> {
// 	fn default() -> Self {
// 		let min_bonus = DOLLARS.saturating_mul(100u128);
// 		let min_bonus: Option<BalanceOf<T>> = min_bonus.try_into().ok();
// 		Self {
// 			min_bonus_of_puzzle: min_bonus.unwrap(), // (100 * DOLLARS).into(),
// 			challenge_period_length: MINUTES.saturating_mul(2).into(),
// 			tax_of_tcr: Perbill::from_percent(10),
// 			tax_of_tvs: Perbill::from_percent(5),
// 			tax_of_tvo: Perbill::from_percent(10),
// 			tax_of_ti: Perbill::from_percent(10),
// 			penalty_of_cp: Perbill::from_percent(10),
// 			max_sponsor_explain_len: 256,
// 			max_answer_explain_len: 1024
// 		}
// 	}
// }
