use super::*;
use frame_support::sp_runtime::traits::{MaybeDisplay, MaybeSerializeDeserialize, Member};
use frame_support::sp_runtime::PerThing;
use frame_support::sp_std::fmt::Debug;
use frame_support::traits::OnUnbalanced;
use frame_support::traits::TryDrop;
use frame_support::Parameter;
use sp_runtime::Perbill;
use types::*;

// Used to uniformly perform accounting operations on Puzzle
pub trait IPuzzleLedger<AccountId, BalanceOf, PuzzleHash, BlockNumber, DResult> {
	//
	fn do_bonus(
		pid: PuzzleHash,
		who: AccountId,
		amount: BalanceOf,
		create_bn: BlockNumber,
	) -> DResult;

	//
	fn do_sponsorship(
		pid: PuzzleHash,
		who: AccountId,
		amount: BalanceOf,
		create_bn: BlockNumber,
		reason: Vec<u8>,
	) -> DResult;
}

pub trait IPuzzleReward<AccountId, BalanceOf, PuzzleHash, BlockNumber, DResult> {
	type PerVal: PerThing;
	type Imbalance: TryDrop;
	type OnBurn: OnUnbalanced<Self::Imbalance>;
	// type FundPool: Parameter
	// 	+ Member
	// 	+ MaybeSerializeDeserialize
	// 	+ Debug
	// 	+ MaybeDisplay
	// 	+ Ord
	// 	+ Default
	// 	+ MaxEncodedLen;

	//
	fn get_total_bonus(pid: &PuzzleHash, cut_bn: BlockNumber) -> Option<BalanceOf>;

	//
	fn answer_get_reward(pid: &PuzzleHash, beneficiary: AccountId, cut_bn: BlockNumber, tax: Self::PerVal) -> DResult;

	//
	fn challenge_get_reward(
		pid: &PuzzleHash,
		beneficiaries: Vec<(AccountId, Self::PerVal)>,
		cut_bn: BlockNumber,
		tax: Self::PerVal,
	) -> DResult;
}

pub trait IPuzzlePoints<AccountId, PToken, BlockNumber, PuzzleHash, DResult> {
	fn get_total_points(who: &AccountId) -> PToken;
	fn increase_points_to(who: &AccountId, pt: PToken) -> DResult;
	fn reduce_points_to(who: &AccountId, pt: PToken) -> DResult;
	fn get_issuance_points() -> PToken;
	fn calculate_points_of_puzzle(
		current_bn: BlockNumber,
		pid: &PuzzleHash,
		per_bn: BlockNumber,
	) -> PToken;
}

pub trait IAtoChallenge<AccountId, PuzzleHash, BalanceOf, DataInfo, Status, Error> {
	fn issue_challenge(who: AccountId, pid: &PuzzleHash, deposit: BalanceOf) -> DispatchResult;
	fn get_balance_threshold(pid: &PuzzleHash) -> BalanceOf;
	fn get_total_raise(pid: &PuzzleHash) -> BalanceOf;
	fn challenge_crowdloan(who: AccountId, pid: &PuzzleHash, deposit: BalanceOf) -> DispatchResult;
	fn has_the_raising_period_expired(pid: &PuzzleHash) -> bool;
	fn get_challenge_status(pid: &PuzzleHash) -> Option<Status>;
	fn recognition_challenge(pid: &PuzzleHash) -> DispatchResult;
	fn back_challenge_crowdloan(pid: &PuzzleHash, tax: Perbill) -> DispatchResult;
	fn check_get_active_challenge_info(pid: &PuzzleHash) -> Result<DataInfo, Error>;
	fn get_list_of_challengers(pid: &PuzzleHash) ->  Vec<(AccountId, Perbill)>;
	fn final_challenge(pid: &PuzzleHash, status: Status) -> DispatchResult;
	fn challenge_failed(pid: &PuzzleHash) -> Result<(), Error>;
}

pub trait IAtoPropose<PuzzleHash>  {
	fn challenge_propose(puzzle_hash: PuzzleHash) -> DispatchResult ;
}

pub trait IPointExchange<AccountId, BlockNumber, Era, PToken, Balance, Info>  {
	fn apply_exchange(who :AccountId) -> DispatchResult ;
	fn execute_exchange(era: Era, mint_balance: Balance) -> DispatchResult;
	fn update_apply_list_point() -> bool;
	fn get_current_era() -> Era;
	fn get_last_reward_era() -> Era;
	fn get_era_length() -> BlockNumber;
	fn get_max_reward_list_size() -> u32;
	fn get_reward_list(era: Era) -> Vec<(AccountId, PToken, Option<Info>)>;
	fn get_history_depth() -> u32;
}

impl IAtoPropose<Vec<u8>> for () {
	fn challenge_propose(puzzle_hash: Vec<u8>) -> DispatchResult {
		Ok(())
	}
}