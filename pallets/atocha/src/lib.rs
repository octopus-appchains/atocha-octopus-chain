#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
pub use pallet::*;

use frame_support::sp_runtime::app_crypto::TryFrom;
use frame_support::sp_runtime::traits::{IdentifyAccount, Verify};
use frame_support::sp_runtime::MultiSignature;
use frame_support::sp_runtime::MultiSigner;
use hex::ToHex;
use sha2::Digest;
use sp_application_crypto::sr25519;
use sp_application_crypto::sr25519::Public;
use sp_application_crypto::sr25519::Signature;

use sp_core::sp_std::vec::Vec;

mod traits;
mod types;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// mod challenge;

#[frame_support::pallet]
pub mod pallet {
	use frame_benchmarking::frame_support::sp_runtime::Perbill;
	use crate::types::*;
	use frame_support::{dispatch::DispatchResultWithPostInfo, dispatch::DispatchResult, pallet_prelude::*};
	use frame_support::traits::{Currency, LockableCurrency, ReservableCurrency};
	use frame_system::pallet_prelude::*;
	use hex;
	use sp_core::sp_std::convert::TryInto;
	use sp_runtime::PerThing;
	use sp_std::vec::Vec;
	use pallet_atofinance::traits::{*};
	use pallet_atofinance::types::{ChallengeStatus, PointToken, PuzzleChallengeData};

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The staking balance.
		type Currency: Currency<Self::AccountId>
			+ ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;

		#[pallet::constant]
		type MinBonusOfPuzzle: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type ChallengePeriodLength: Get<Self::BlockNumber>;

		#[pallet::constant]
		type TaxOfTVS: Get<Perbill> ;

		#[pallet::constant]
		type TaxOfTVO: Get<Perbill> ;

		type PuzzleLedger: IPuzzleLedger<
			<Self as frame_system::Config>::AccountId,
			BalanceOf<Self>,
			PuzzleSubjectHash,
			<Self as frame_system::Config>::BlockNumber,
			DispatchResult
		>;
		type PuzzleRewardOfToken: IPuzzleReward<
			<Self as frame_system::Config>::AccountId,
			BalanceOf<Self>,
			PuzzleSubjectHash,
			<Self as frame_system::Config>::BlockNumber,
			DispatchResult,
			PerVal = Perbill,
		>;
		type PuzzleRewardOfPoint: IPuzzleReward<
			<Self as frame_system::Config>::AccountId,
			PointToken,
			PuzzleSubjectHash,
			<Self as frame_system::Config>::BlockNumber,
			DispatchResult,
			PerVal = Perbill,
		>;
		type AtoChallenge: IAtoChallenge<
			<Self as frame_system::Config>::AccountId,
			PuzzleSubjectHash,
			BalanceOf<Self>,
			PuzzleChallengeData<<Self as frame_system::Config>::AccountId, Self::BlockNumber, BalanceOf<Self>>,
			ChallengeStatus<Self::BlockNumber>,
			pallet_atofinance::Error<Self>,
		>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn puzzle_info)]
	pub type PuzzleInfo<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		PuzzleSubjectHash,
		PuzzleInfoData<T::AccountId, T::BlockNumber>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn puzzle_direct_answer)]
	pub type PuzzleDirectAnswer<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		PuzzleSubjectHash,
		Blake2_128Concat,
		PuzzleAnswerHash,
		PuzzleAnswerData<T::AccountId, T::BlockNumber>,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// creator id, puzzle_hash, create block number , duration block number,
		PuzzleCreated(T::AccountId, PuzzleSubjectHash, CreateBn<T::BlockNumber>), // remove . DurationBn
		AnswerCreated(T::AccountId, PuzzleAnswerHash, PuzzleSubjectHash, CreateBn<T::BlockNumber>),
	}

	#[pallet::error]
	pub enum Error<T> {
		PuzzleAlreadyExist,
		AnswerAlreadyExist,
		WrongAnswer,
		PuzzleNotExist,
		PuzzleHasBeenSolved,
		PuzzleStatusErr,
		PuzzleMinBonusInsufficient,
		PuzzleNotSolvedChallengeFailed,
		ChallengePeriodIsNotEnd,
		ChallengePeriodIsEnd,
		BeingChallenged,
		NoRightToReward,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		u64: From<<T as frame_system::Config>::BlockNumber>,
	{
		#[pallet::weight(1000)]
		pub fn create_puzzle(
			origin: OriginFor<T>,
			puzzle_hash: PuzzleSubjectHash, // Arweave tx - id
			answer_hash: PuzzleAnswerHash,
			answer_explain: Option<PuzzleAnswerExplain>,
			amount: BalanceOf<T>,
			puzzle_version: PuzzleVersion,
		) -> DispatchResultWithPostInfo {
			// check signer
			let who = ensure_signed(origin)?;
			ensure!(!<PuzzleInfo<T>>::contains_key(&puzzle_hash), Error::<T>::PuzzleAlreadyExist);

			// Check amount > MinBonus
			ensure!(amount >= T::MinBonusOfPuzzle::get(), Error::<T>::PuzzleMinBonusInsufficient);

			//
			let current_block_number = <frame_system::Pallet<T>>::block_number();

			// pid: PuzzleHash, who: AccountId, amount: BalanceOf, create_bn: BlockNumber,
			T::PuzzleLedger::do_bonus(puzzle_hash.clone(), who.clone(), amount.clone(), current_block_number)?;

			let puzzle_content = PuzzleInfoData {
				account: who.clone(),
				answer_hash,
				answer_explain,
				puzzle_status: PuzzleStatus::PUZZLE_STATUS_IS_SOLVING,
				create_bn: current_block_number,
				reveal_answer: None,
				reveal_bn: None,
				puzzle_version,
			};
			<PuzzleInfo<T>>::insert(puzzle_hash.clone(), puzzle_content);

			// send event
			Self::deposit_event(Event::PuzzleCreated(
				who,
				puzzle_hash,
				current_block_number.into(),
			));
			//
			Ok(().into())
		}

		#[pallet::weight(1000)]
		pub fn answer_puzzle(
			origin: OriginFor<T>,
			puzzle_hash: PuzzleSubjectHash,
			answer_hash: PuzzleAnswerHash,
			// ticket: PuzzleTicket,
		) -> DispatchResultWithPostInfo {
			// check signer
			let who = ensure_signed(origin)?;

			//
			let current_block_number = <frame_system::Pallet<T>>::block_number();

			// Puzzle need exists.
			ensure!(<PuzzleInfo<T>>::contains_key(&puzzle_hash), Error::<T>::PuzzleNotExist);

			// Get puzzle
			let mut puzzle_content = <PuzzleInfo<T>>::get(&puzzle_hash).unwrap();
			ensure!(
				puzzle_content.puzzle_status == PuzzleStatus::PUZZLE_STATUS_IS_SOLVING,
				Error::<T>::PuzzleHasBeenSolved
			);

			// let mut answer_store_list: Vec<AnswerContent<T>> = Vec::new();
			let answer_list_opt = <PuzzleDirectAnswer<T>>::get(&puzzle_hash, &answer_hash);
			ensure!(answer_list_opt.is_none(), Error::<T>::AnswerAlreadyExist);

			// check answer is right,
			let update_answer_sign =
				Self::make_answer_sign(answer_hash.clone(), puzzle_hash.clone());

			let answer_status_check = || -> PuzzleAnswerStatus {
				if update_answer_sign == puzzle_content.answer_hash {
					puzzle_content.puzzle_status = PuzzleStatus::PUZZLE_STATUS_IS_SOLVED;
					puzzle_content.reveal_bn = Some(current_block_number);
					puzzle_content.reveal_answer = Some(who.clone());
					<PuzzleInfo<T>>::insert(&puzzle_hash, puzzle_content);
					PuzzleAnswerStatus::ANSWER_HASH_IS_MATCH
				} else {
					PuzzleAnswerStatus::ANSWER_HASH_IS_MISMATCH
				}
			};

			// create new answer tuple.
			let answer_content = PuzzleAnswerData {
				account: who.clone(),
				answer_status: answer_status_check(),
				create_bn: current_block_number.clone(),
			};

			<PuzzleDirectAnswer<T>>::insert(
				puzzle_hash.clone(),
				answer_hash.clone(),
				answer_content,
			);

			// send event
			Self::deposit_event(Event::AnswerCreated(
				who,
				answer_hash,
				puzzle_hash,
				current_block_number,
			));
			//
			Ok(().into())
		}

		#[pallet::weight(1000)]
		pub fn take_answer_reward(
			origin: OriginFor<T>,
			puzzle_hash: PuzzleSubjectHash,
		) -> DispatchResult {
			// check signer
			let who = ensure_signed(origin)?;
			// 1\ Check puzzle status is PUZZLE_STATUS_IS_SOLVED AND current_bn - Some(reveal_bn)  > T::ChallengePeriodLength
			let mut puzzle_content = <PuzzleInfo<T>>::get(&puzzle_hash).unwrap();
			ensure!(
				puzzle_content.puzzle_status == PuzzleStatus::PUZZLE_STATUS_IS_SOLVED,
				Error::<T>::PuzzleStatusErr
			);

			// NoRightToReward
			// Get winner answer.
			ensure!(puzzle_content.reveal_answer == Some(who.clone()), Error::<T>::NoRightToReward);

			//
			let current_block_number = <frame_system::Pallet<T>>::block_number();
			let reveal_bn = puzzle_content.reveal_bn.unwrap();
			// println!("reveal_bn = {:?} current_block_number = {:?}, periodlength={:?}", reveal_bn, current_block_number, T::ChallengePeriodLength::get());
			ensure!(
				current_block_number - reveal_bn > T::ChallengePeriodLength::get(),
				Error::<T>::ChallengePeriodIsNotEnd
			);

			// Check Challenged
			let challenge_info = T::AtoChallenge::check_get_active_challenge_info(&puzzle_hash);
			ensure!(
				challenge_info.is_err(),
				Error::<T>::BeingChallenged
			);

			let tax_fee = || {
				if puzzle_content.account == who {
					T::TaxOfTVS::get()
				}else{
					T::TaxOfTVO::get()
				}
			};

			// Take points.
			T::PuzzleRewardOfPoint::answer_get_reward(&puzzle_hash, who.clone(), reveal_bn, tax_fee())?;
			// Take balance.
			T::PuzzleRewardOfToken::answer_get_reward(&puzzle_hash, who.clone(), reveal_bn, tax_fee())?;

			Ok(().into())
		}

		#[pallet::weight(1000)]
		pub fn commit_challenge(
			origin: OriginFor<T>,
			puzzle_hash: PuzzleSubjectHash, // Arweave tx - id
			deposit: BalanceOf<T>,
			// answer_hash: PuzzleAnswerHash,
			// answer_explain: Option<PuzzleAnswerExplain>,
			// answer_nonce: PuzzleAnswerNonce,
			// puzzle_version: PuzzleVersion,
		) -> DispatchResult {
			// check signer
			let who = ensure_signed(origin)?;
			let mut puzzle_content = <PuzzleInfo<T>>::get(&puzzle_hash).unwrap();
			ensure!(
				puzzle_content.puzzle_status == PuzzleStatus::PUZZLE_STATUS_IS_SOLVED,
				Error::<T>::PuzzleStatusErr
			);
			//
			let current_block_number = <frame_system::Pallet<T>>::block_number();
			let reveal_bn = puzzle_content.reveal_bn.unwrap();
			// println!("reveal_bn = {:?} current_block_number = {:?}, periodlength={:?}", reveal_bn, current_block_number, T::ChallengePeriodLength::get());
			ensure!(
				current_block_number - reveal_bn <= T::ChallengePeriodLength::get(),
				Error::<T>::ChallengePeriodIsEnd
			);
			//
			T::AtoChallenge::issue_challenge(who.clone(), &puzzle_hash, deposit)?;
			//
			Ok(().into())
		}

		#[pallet::weight(1000)]
		pub fn recognition_challenge(
			origin: OriginFor<T>,
			puzzle_hash: PuzzleSubjectHash, // Arweave tx - id
			deposit: BalanceOf<T>,
		) -> DispatchResult {
			// check signer
			let who = ensure_signed(origin)?;
			let mut puzzle_content = <PuzzleInfo<T>>::get(&puzzle_hash).unwrap();
			ensure!(
				puzzle_content.puzzle_status == PuzzleStatus::PUZZLE_STATUS_IS_SOLVED,
				Error::<T>::PuzzleStatusErr
			);
			//

			//
			T::AtoChallenge::recognition_challenge(&puzzle_hash)?;
			//
			Ok(().into())
		}

	}
}

impl<T: Config> Pallet<T>
where
	u64: From<<T as frame_system::Config>::BlockNumber>,
{
	fn check_signed_valid(public_id: Public, signature: &[u8], msg: &[u8]) -> bool {
		let signature = Signature::try_from(signature);
		let signature = signature.unwrap();

		let multi_sig = MultiSignature::from(signature); // OK
		let multi_signer = MultiSigner::from(public_id);
		multi_sig.verify(msg, &multi_signer.into_account())
	}

	fn get_current_block_number() -> u64 {
		u64::from(<frame_system::Pallet<T>>::block_number())
	}

	fn make_answer_sign(answer_hash: Vec<u8>, mut answer_nonce: Vec<u8>) -> Vec<u8> {
		let mut sha_answer_hash_x = sha2::Sha256::new();
		sha_answer_hash_x.update(answer_hash.as_slice());
		// Make answer sha256.
		let mut sha1_ansser_vec = sha_answer_hash_x.finalize().as_slice().to_vec();

		// Create answer hex str vec
		let mut result_answer_u8 = [0u8; 32 * 2];
		// Answer sha256 to encode slice
		let encode_result =
			hex::encode_to_slice(&sha1_ansser_vec.as_slice(), &mut result_answer_u8 as &mut [u8]);
		assert!(encode_result.is_ok(), "make_answer_sign to Hex failed.");

		// Convert to Vec<u8>
		let mut result_answer_v8 = result_answer_u8.to_vec();
		// Append nonce str
		result_answer_v8.append(&mut answer_nonce);

		// Make final sha256 = sha256(sha256(Answer)+nonce)
		let mut sha256_answer_final = sha2::Sha256::new();
		sha256_answer_final.update(result_answer_v8.as_slice());
		sha256_answer_final.finalize().as_slice().to_vec()
	}
}
