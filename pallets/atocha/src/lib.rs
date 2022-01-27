#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
pub use pallet::*;

use frame_support::sp_runtime::app_crypto::TryFrom;
use frame_support::sp_runtime::traits::{IdentifyAccount, Verify};
use frame_support::sp_runtime::{DispatchResult, MultiSignature};
use frame_support::sp_runtime::MultiSigner;
use frame_support::ensure;

use hex::ToHex;
use sha2::Digest;
use sp_application_crypto::sr25519;
use sp_application_crypto::sr25519::Public;
use sp_application_crypto::sr25519::Signature;
use sp_runtime::SaturatedConversion;
use sp_core::sp_std::vec::Vec;
use pallet_atofinance::traits::IAtoChallenge;
use pallet_atofinance::types::ChallengeStatus;
use crate::types::PuzzleSubjectHash;
use crate::types::PuzzleStatus;

mod traits;
pub mod types;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// mod challenge;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::sp_runtime::{Perbill, RuntimeDebug};
	use crate::types::*;
	use frame_support::{dispatch::DispatchResultWithPostInfo, dispatch::DispatchResult, pallet_prelude::*};
	use frame_support::dispatch::Dispatchable;
	use frame_support::traits::{Currency, LockableCurrency, ReservableCurrency};
	use frame_system::pallet_prelude::*;
	use hex;
	use sp_core::sp_std::convert::TryInto;
	use sp_runtime::PerThing;
	use sp_runtime::sp_std::fmt::Debug;
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

		// type Call: Dispatchable + Debug;

		#[pallet::constant]
		type MinBonusOfPuzzle: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type ChallengePeriodLength: Get<Self::BlockNumber>;

		#[pallet::constant]
		type TaxOfTCR: Get<Perbill> ;

		#[pallet::constant]
		type TaxOfTVS: Get<Perbill> ;

		#[pallet::constant]
		type TaxOfTVO: Get<Perbill> ;

		#[pallet::constant]
		type TaxOfTI: Get<Perbill> ;

		#[pallet::constant]
		type PenaltyOfCP: Get<Perbill>;

		#[pallet::constant]
		type MaxSponsorExplainLen: Get<u32>;

		#[pallet::constant]
		type MaxAnswerExplainLen: Get<u32>;

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
			OnBurn = (),
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
			PuzzleChallengeData<<Self as frame_system::Config>::AccountId, Self::BlockNumber, BalanceOf<Self>, Perbill>,
			ChallengeStatus<Self::BlockNumber, Perbill>,
			pallet_atofinance::Error<Self>,
		>;

		type AtoPointsManage: IPuzzlePoints<
			<Self as frame_system::Config>::AccountId,
			PointToken,
			<Self as frame_system::Config>::BlockNumber,
			PuzzleSubjectHash,
			DispatchResult
		>;

		type CouncilOrigin: EnsureOrigin<Self::Origin>;
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
		PuzzleCreated(T::AccountId, PuzzleSubjectHash, CreateBn<T::BlockNumber>, BalanceOf<T>), // remove . DurationBn
		AdditionalSponsorship(T::AccountId, PuzzleSubjectHash, CreateBn<T::BlockNumber>, BalanceOf<T>, PuzzleSponsorExplain), // remove . DurationBn
		AnswerCreated(T::AccountId, PuzzleAnswerHash, PuzzleSubjectHash, CreateBn<T::BlockNumber>),
		AnswerMatch(PuzzleSubjectHash, Vec<u8>, PuzzleAnswerHash,PuzzleAnswerHash),
		AnswerMisMatch(PuzzleSubjectHash, Vec<u8>, PuzzleAnswerHash,PuzzleAnswerHash),
		// IssueChallenge(T::AccountId, PuzzleSubjectHash, BalanceOf<T>,),
		CrowdloanChallenge(T::AccountId, PuzzleSubjectHash, BalanceOf<T>,),
		CreatorPointSlash(PuzzleSubjectHash, PointSlashData<T::AccountId, Perbill, PointToken>),
		ChallengePassed(PuzzleSubjectHash,ChallendRewardData<T::AccountId, Perbill>),
	}

	#[pallet::error]
	pub enum Error<T> {
		AnswerAlreadyExist,
		BeingChallenged,
		ChallengeCrowdloanPeriodNotEnd,
		ChallengePeriodIsNotEnd,
		ChallengeListNotEmpty,
		ChallengePeriodIsEnd,
		ChallengeHasBeenSubmitted,
		ChallengeNotExists,
		ChallengeHasBeenDisbanded,
		NoRightToReward,
		PuzzleNotExist,
		PuzzleHasBeenSolved,
		PuzzleStatusErr,
		PuzzleMinBonusInsufficient,
		ExplainTooLong,
		PuzzleAlreadyExist,
		PuzzleNotSolvedChallengeFailed,
		WrongAnswer,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	{

		#[pallet::weight(100)]
		pub fn additional_sponsorship(
			origin: OriginFor<T>,
			puzzle_hash: PuzzleSubjectHash, // Arweave tx - id
			#[pallet::compact] amount: BalanceOf<T>,
			reason: Option<PuzzleSponsorExplain>,
		) -> DispatchResultWithPostInfo {
			// check signer
			let who = ensure_signed(origin)?;

			// Check amount > MinBonus
			ensure!(amount >= T::MinBonusOfPuzzle::get(), Error::<T>::PuzzleMinBonusInsufficient);

			//
			let current_block_number = <frame_system::Pallet<T>>::block_number();

			let mut reason_v8 = Vec::new();
			if let Some(r) = reason {
				ensure!(r.len() as u32 <= T::MaxSponsorExplainLen::get(), Error::<T>::ExplainTooLong);
				reason_v8 = r;
			}

			ensure!(<PuzzleInfo<T>>::contains_key(&puzzle_hash), Error::<T>::PuzzleNotExist);

			// Get puzzle
			let puzzle_content = <PuzzleInfo<T>>::get(&puzzle_hash).unwrap();
			ensure!(
				puzzle_content.puzzle_status == PuzzleStatus::PUZZLE_STATUS_IS_SOLVING,
				Error::<T>::PuzzleHasBeenSolved
			);

			// pid: PuzzleHash, who: AccountId, amount: BalanceOf, create_bn: BlockNumber,
			// T::PuzzleLedger::do_bonus(puzzle_hash.clone(), who.clone(), amount.clone(), current_block_number)?;
			T::PuzzleLedger::do_sponsorship(puzzle_hash.clone(), who.clone(), amount.clone(), current_block_number, reason_v8.clone())?;

			// send event
			Self::deposit_event(Event::AdditionalSponsorship(
				who,
				puzzle_hash,
				current_block_number.into(),
				amount,
				reason_v8,
			));
			//
			Ok(().into())
		}

		#[pallet::weight(100)]
		pub fn answer_puzzle(
			origin: OriginFor<T>,
			puzzle_hash: PuzzleSubjectHash,
			answer_hash: PuzzleAnswerHash,
			answer_explain: PuzzleAnswerExplain,
		) -> DispatchResultWithPostInfo {
			// check signer
			let who = ensure_signed(origin)?;

			ensure!(answer_explain.len() as u32 <= T::MaxAnswerExplainLen::get(), Error::<T>::ExplainTooLong);

			//
			let current_block_number = <frame_system::Pallet<T>>::block_number();

			// Puzzle need exists.
			ensure!(<PuzzleInfo<T>>::contains_key(&puzzle_hash), Error::<T>::PuzzleNotExist);

			// Get puzzle
			let puzzle_content = <PuzzleInfo<T>>::get(&puzzle_hash).unwrap();
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
				// println!(" update_answer_sign {:?} == puzzle_content.answer_hash {:?} ", &update_answer_sign, &puzzle_content.answer_hash  );
				if update_answer_sign == puzzle_content.answer_hash {
					let mut update_puzzle_content = puzzle_content.clone();
					update_puzzle_content.puzzle_status = PuzzleStatus::PUZZLE_STATUS_IS_SOLVED;
					update_puzzle_content.reveal_bn = Some(current_block_number);
					update_puzzle_content.reveal_answer = Some(who.clone());
					<PuzzleInfo<T>>::insert(&puzzle_hash, update_puzzle_content);

					Self::deposit_event(Event::<T>::AnswerMatch(
						puzzle_hash.clone(),
						answer_hash.clone(),
						update_answer_sign.clone(),
						puzzle_content.answer_hash.clone()
					));

					PuzzleAnswerStatus::ANSWER_HASH_IS_MATCH
				} else {
					Self::deposit_event(Event::<T>::AnswerMisMatch(
						puzzle_hash.clone(),
						answer_hash.clone(),
						update_answer_sign.clone(),
						puzzle_content.answer_hash.clone()
					));
					PuzzleAnswerStatus::ANSWER_HASH_IS_MISMATCH
				}
			};

			// create new answer tuple.
			let answer_content = PuzzleAnswerData {
				account: who.clone(),
				answer_status: answer_status_check(),
				answer_explain,
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

		#[pallet::weight(100)]
		pub fn create_puzzle(
			origin: OriginFor<T>,
			puzzle_hash: PuzzleSubjectHash, // Arweave tx - id
			answer_hash: PuzzleAnswerHash,
			#[pallet::compact] amount: BalanceOf<T>,
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
				amount,
			));
			//
			Ok(().into())
		}

		#[pallet::weight(100)]
		pub fn commit_challenge(
			origin: OriginFor<T>,
			puzzle_hash: PuzzleSubjectHash, // Arweave tx - id
			#[pallet::compact] deposit: BalanceOf<T>,
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
			// Self::deposit_event(Event::<T>::IssueChallenge(
			// 	who.clone(),
			// 	puzzle_hash.clone(),
			// 	deposit.clone(),
			// ));

			//
			Ok(().into())
		}

		#[pallet::weight(100)]
		pub fn challenge_pull_out(
			origin: OriginFor<T>,
			puzzle_hash: PuzzleSubjectHash, // Arweave tx - id
			#[pallet::compact] deposit: BalanceOf<T>,
		) -> DispatchResult {
			// check signer
			let who = ensure_signed(origin)?;

			let challenge_status = T::AtoChallenge::get_challenge_status(&puzzle_hash);
			ensure!(challenge_status.is_some(), Error::<T>::ChallengeNotExists);

			let current_bn = <frame_system::Pallet<T>>::block_number();
			let challenge_status = challenge_status.unwrap();
			match  challenge_status {
				ChallengeStatus::Raise(raise_bn) => {
					// ensure!(current_bn - raise_bn > T::ChallengeCrowdloanPeriodLength::get(), Error::<T>::ChallengeCrowdloanPeriodNotEnd )
					ensure!(T::AtoChallenge::check_get_active_challenge_info(&puzzle_hash).is_ok(), Error::<T>::ChallengeCrowdloanPeriodNotEnd );
				},
				ChallengeStatus::RaiseBackFunds(_raise_bn, _) => {
					return DispatchResult::Err(Error::<T>::ChallengeHasBeenDisbanded.into());
				},
				_ => {
					return DispatchResult::Err(Error::<T>::ChallengeHasBeenSubmitted.into());
				}
			}

			//
			T::AtoChallenge::back_challenge_crowdloan(&puzzle_hash, T::TaxOfTCR::get());

			//
			Ok(().into())
		}

		#[pallet::weight(100)]
		pub fn challenge_crowdloan(
			origin: OriginFor<T>,
			puzzle_hash: PuzzleSubjectHash, // Arweave tx - id
			#[pallet::compact] deposit: BalanceOf<T>,
		) -> DispatchResult {
			// check signer
			let who = ensure_signed(origin)?;
			let mut puzzle_content = <PuzzleInfo<T>>::get(&puzzle_hash).unwrap();
			ensure!(
				puzzle_content.puzzle_status == PuzzleStatus::PUZZLE_STATUS_IS_SOLVED,
				Error::<T>::PuzzleStatusErr
			);
			//
			T::AtoChallenge::challenge_crowdloan(who.clone(), &puzzle_hash, deposit)?;
			// Self::deposit_event(Event::<T>::IssueChallenge(
			// 	who.clone(),
			// 	puzzle_hash.clone(),
			// 	deposit.clone(),
			// ));
			//
			Ok(().into())
		}

		#[pallet::weight(100)]
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

			let tax_fee = |acc| {
				if acc == &who {
					T::TaxOfTVS::get()
				}else{
					T::TaxOfTVO::get()
				}
			};

			puzzle_content.puzzle_status = PuzzleStatus::PUZZLE_STATUS_IS_FINAL;
			<PuzzleInfo<T>>::insert(&puzzle_hash, puzzle_content.clone());

			let creator_acc = puzzle_content.account.clone();

			// Take points.
			T::PuzzleRewardOfPoint::answer_get_reward(&puzzle_hash, who.clone(), reveal_bn, tax_fee(&creator_acc))?;
			// Take balance.
			T::PuzzleRewardOfToken::answer_get_reward(&puzzle_hash, who.clone(), reveal_bn, tax_fee(&creator_acc))?;

			Ok(().into())
		}

		#[pallet::weight(100)]
		pub fn recognition_challenge(
			origin: OriginFor<T>,
			puzzle_hash: PuzzleSubjectHash, // Arweave tx - id
		) -> DispatchResultWithPostInfo {
			// check signer
			T::CouncilOrigin::ensure_origin(origin)?;
			let mut puzzle_content = <PuzzleInfo<T>>::get(&puzzle_hash).unwrap();
			ensure!(
				puzzle_content.puzzle_status == PuzzleStatus::PUZZLE_STATUS_IS_SOLVED,
				Error::<T>::PuzzleStatusErr
			);
			//
			let reveal_bn = puzzle_content.reveal_bn.unwrap();

			//Get challenge list
			let beneficiaries = T::AtoChallenge::get_list_of_challengers(&puzzle_hash);
			ensure!(
				beneficiaries.len() > 0 as usize,
				Error::<T>::ChallengeListNotEmpty
			);
			//

			T::PuzzleRewardOfToken::challenge_get_reward(&puzzle_hash, beneficiaries.clone(), reveal_bn, T::TaxOfTI::get())?;
			T::AtoChallenge::recognition_challenge(&puzzle_hash)?;

			let current_block_number = <frame_system::Pallet<T>>::block_number();
			T::AtoChallenge::final_challenge(&puzzle_hash, ChallengeStatus::JudgePassed(current_block_number));
			puzzle_content.puzzle_status = PuzzleStatus::PUZZLE_STATUS_IS_FINAL;
			<PuzzleInfo<T>>::insert(&puzzle_hash, puzzle_content.clone());

			let create_total_point = T::AtoPointsManage::get_total_points(&puzzle_content.account);
			if create_total_point > 0 {
				let cut_down_point = T::PenaltyOfCP::get() * create_total_point;
				T::AtoPointsManage::reduce_points_to(&puzzle_content.account, cut_down_point)?;
				Self::deposit_event(Event::<T>::CreatorPointSlash(
					puzzle_hash.clone(),
					PointSlashData {
						who: puzzle_content.account.clone(),
						rate_cp: T::PenaltyOfCP::get(),
						total: create_total_point.clone(),
						slash: cut_down_point.clone(),
					},
				));
			}

			Self::deposit_event(Event::<T>::ChallengePassed(
				puzzle_hash.clone(),
				ChallendRewardData {
					beneficiaries: beneficiaries.clone(),
					rate_ti: T::TaxOfTI::get(),
				}
			));

			Ok(().into())
		}

		// refuse_challenge
		// #[pallet::weight(0)]
		// pub fn refuse_challenge(
		// 	origin: OriginFor<T>,
		// 	puzzle_hash: PuzzleSubjectHash, // Arweave tx - id
		// ) -> DispatchResult {
		// 	// check signer
		// 	T::CouncilOrigin::ensure_origin(origin)?;
		// 	let mut puzzle_content = <PuzzleInfo<T>>::get(&puzzle_hash).unwrap();
		// 	ensure!(
		// 		puzzle_content.puzzle_status == PuzzleStatus::PUZZLE_STATUS_IS_SOLVED,
		// 		Error::<T>::PuzzleStatusErr
		// 	);
		// 	//
		// 	// let reveal_bn = puzzle_content.reveal_bn.unwrap();
		//
		// 	//Get challenge list
		// 	let beneficiaries = T::AtoChallenge::get_list_of_challengers(&puzzle_hash);
		// 	ensure!(
		// 		beneficiaries.len() > 0 as usize,
		// 		Error::<T>::ChallengeListNotEmpty
		// 	);
		//
		// 	//
		// 	let current_block_number = <frame_system::Pallet<T>>::block_number();
		//
		// 	T::AtoChallenge::challenge_failed(&puzzle_hash);
		// 	T::AtoChallenge::final_challenge(&puzzle_hash, ChallengeStatus::JudgeRejected(current_block_number));
		//
		// 	Ok(().into())
		// }

		// // refuse_challenge
		// #[pallet::weight(0)]
		// pub fn cancel_challenge_crowdloan(
		// 	origin: OriginFor<T>,
		// 	puzzle_hash: PuzzleSubjectHash, // Arweave tx - id
		// ) -> DispatchResult {
		// 	// check signer
		// 	let who = ensure_signed(origin)?;
		// 	//Get beneficiarie list
		// 	let beneficiaries = T::AtoChallenge::get_list_of_challengers(&puzzle_hash);
		//
		//
		// 	let mut puzzle_content = <PuzzleInfo<T>>::get(&puzzle_hash).unwrap();
		// 	ensure!(
		// 		puzzle_content.puzzle_status == PuzzleStatus::PUZZLE_STATUS_IS_SOLVED,
		// 		Error::<T>::PuzzleStatusErr
		// 	);
		// 	//
		// 	// let reveal_bn = puzzle_content.reveal_bn.unwrap();
		//
		// 	//Get challenge list
		// 	let beneficiaries = T::AtoChallenge::get_list_of_challengers(&puzzle_hash);
		// 	ensure!(
		// 		beneficiaries.len() > 0 as usize,
		// 		Error::<T>::ChallengeListNotEmpty
		// 	);
		//
		// 	//
		// 	let current_block_number = <frame_system::Pallet<T>>::block_number();
		// 	T::AtoChallenge::final_challenge(&puzzle_hash, ChallengeStatus::JudgeRejected(current_block_number));
		//
		// 	Ok(().into())
		// }

	}
}

impl<T: Config> Pallet<T>
{

	pub fn refuse_challenge(
		puzzle_hash: PuzzleSubjectHash, // Arweave tx - id
	) -> DispatchResult {

		let mut puzzle_content = <PuzzleInfo<T>>::get(&puzzle_hash).unwrap();
		ensure!(
				puzzle_content.puzzle_status == PuzzleStatus::PUZZLE_STATUS_IS_SOLVED,
				Error::<T>::PuzzleStatusErr
			);

		//Get challenge list
		let beneficiaries = T::AtoChallenge::get_list_of_challengers(&puzzle_hash);
		ensure!(
				beneficiaries.len() > 0 as usize,
				Error::<T>::ChallengeListNotEmpty
			);

		//
		let current_block_number = <frame_system::Pallet<T>>::block_number();

		T::AtoChallenge::challenge_failed(&puzzle_hash);
		T::AtoChallenge::final_challenge(&puzzle_hash, ChallengeStatus::JudgeRejected(current_block_number));

		Ok(().into())
	}

	fn check_signed_valid(public_id: Public, signature: &[u8], msg: &[u8]) -> bool {
		let signature = Signature::try_from(signature);
		let signature = signature.unwrap();

		let multi_sig = MultiSignature::from(signature); // OK
		let multi_signer = MultiSigner::from(public_id);
		multi_sig.verify(msg, &multi_signer.into_account())
	}

	fn get_current_block_number() -> u64 {
		let current_bn = <frame_system::Pallet<T>>::block_number();
		current_bn.saturated_into()
	}

	fn make_sha256_hash(txt: Vec<u8>) -> Vec<u8> {
		let mut sha_answer_hash_x = sha2::Sha256::new();
		sha_answer_hash_x.update(txt.as_slice());
		// Make answer sha256.
		let mut sha1_ansser_vec = sha_answer_hash_x.finalize().as_slice().to_vec();
		// Create answer hex str vec
		let mut result_answer_u8 = [0u8; 32 * 2];
		// Answer sha256 to encode slice
		let encode_result =
			hex::encode_to_slice(&sha1_ansser_vec.as_slice(), &mut result_answer_u8 as &mut [u8]);
		assert!(encode_result.is_ok(), "make_answer_sign to Hex failed.");
		result_answer_u8.to_vec()
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
		let mut sha1_ansser_vec = sha256_answer_final.finalize().as_slice().to_vec();

		let mut final_result_u8 = [0u8; 32 * 2];
		let final_encode_result =
			hex::encode_to_slice(&sha1_ansser_vec.as_slice(), &mut final_result_u8 as &mut [u8]);
		assert!(final_encode_result.is_ok(), "make_answer_sign to Hex failed.");
		final_result_u8.to_vec()
	}
}
