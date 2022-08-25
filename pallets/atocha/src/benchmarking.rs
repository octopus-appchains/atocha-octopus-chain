#![cfg(feature = "runtime-benchmarks")]
use super::*;
use atocha_constants::{DOLLARS, MINUTES};
use frame_support::traits::Currency;
use frame_system::Origin;
use frame_support::assert_ok;

use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, whitelisted_caller, impl_benchmark_test_suite, account};
use pallet_atofinance::traits::IPuzzleLedger;

use crate::Pallet as AtochaModule;
use crate::types::{PuzzleAnswerExplain, PuzzleAnswerHash};

fn handler_create_puzzle<T: Config>(puzzle_hash: PuzzleSubjectHash, answer_hash: PuzzleAnswerHash) {
	let config = AtoConfig::<T>::get().unwrap();
	let amount: Option<BalanceOf<T>> = (DOLLARS * 100u128).try_into().ok();
	let amount = amount.unwrap();
	let caller: T::AccountId = whitelisted_caller();
	let make_balance =  (<T as Config>::Currency::minimum_balance() + config.min_bonus_of_puzzle ) * 2000u32.into() ;
	let _ = <T as Config>::Currency::make_free_balance_be(&caller, make_balance);
	let puzzle_version = 1;
	AtochaModule::<T>::create_puzzle(RawOrigin::Signed(caller).into(), puzzle_hash.clone(), answer_hash, amount, puzzle_version);
	assert!(<PuzzleInfo<T>>::contains_key(&puzzle_hash));
}

fn handler_answer_puzzle<T: Config>(caller: T::AccountId, puzzle_hash: PuzzleSubjectHash, answer_hash: PuzzleAnswerHash, explain: PuzzleAnswerExplain) {
	assert_ok!(AtochaModule::<T>::answer_puzzle(RawOrigin::Signed(caller).into(), puzzle_hash.clone(), answer_hash.clone(), explain));
	assert!(<PuzzleInfo<T>>::contains_key(&puzzle_hash));
	assert!(<PuzzleDirectAnswer<T>>::contains_key(&puzzle_hash, &answer_hash));
	let puzzle_info = <PuzzleInfo<T>>::get(&puzzle_hash);
	let puzzle_info = puzzle_info.unwrap();
}

fn handler_commit_challenge<T: Config>(caller: T::AccountId, puzzle_hash: PuzzleSubjectHash, deposit: BalanceOf<T>) {
	AtochaModule::<T>::commit_challenge(RawOrigin::Signed(caller).into(), puzzle_hash, deposit);
}

fn get_min_bonus_of_puzzle<T: Config>() -> BalanceOf<T> {
	let config = AtoConfig::<T>::get().unwrap();
	config.min_bonus_of_puzzle
}

fn wrap_account_with_balance<T: Config>(who: T::AccountId) -> T::AccountId {
	let make_balance =  (<T as Config>::Currency::minimum_balance() + get_min_bonus_of_puzzle::<T>() ) * 20000u32.into() ;
	let _ = <T as Config>::Currency::make_free_balance_be(&who, make_balance);
	who
}

fn get_dollars<T: Config>(amount: u128) -> BalanceOf<T> {
	let result: Option<BalanceOf<T>> = (DOLLARS.saturating_mul(amount)).try_into().ok();
	result.unwrap()
}

benchmarks! {
	create_puzzle {
		let caller: T::AccountId = wrap_account_with_balance::<T>(whitelisted_caller());
		let puzzle_hash = "PUZZLE_HASH".as_bytes().to_vec(); // to_vec("PUZZLE_HASH");
		let answer_hash = "ANSWER_HASH".as_bytes().to_vec(); // to_vec("ANSWER_HASH");
		let puzzle_version = 1;
	}: _(RawOrigin::Signed(caller), puzzle_hash.clone(), answer_hash.clone(), get_dollars::<T>(100), puzzle_version)
	verify {
		assert!(<PuzzleInfo<T>>::contains_key(&puzzle_hash));
	}

	answer_puzzle {
		let caller: T::AccountId = wrap_account_with_balance::<T>(whitelisted_caller());
		let puzzle_hash = "PUZZLE_HASH".as_bytes().to_vec();
		let answer_hash = "ANSWER_HASH".as_bytes().to_vec();
		handler_create_puzzle::<T>(puzzle_hash.clone(), answer_hash.clone());
	}: _(RawOrigin::Signed(caller), puzzle_hash.clone(), answer_hash.clone(), "PuzzleAnswerExplain".as_bytes().to_vec())
	verify {
		assert!(<PuzzleDirectAnswer<T>>::contains_key(&puzzle_hash, &answer_hash));
	}

	additional_sponsorship {
		let caller: T::AccountId = wrap_account_with_balance::<T>(whitelisted_caller());
		let puzzle_hash = "PUZZLE_HASH".as_bytes().to_vec();
		let answer_hash = "ANSWER_HASH".as_bytes().to_vec();
		handler_create_puzzle::<T>(puzzle_hash.clone(), answer_hash.clone());
	}: _(RawOrigin::Signed(caller), puzzle_hash.clone(), get_dollars::<T>(500), Some("For test.".as_bytes().to_vec()))
	verify {
		assert!(!<PuzzleDirectAnswer<T>>::contains_key(&puzzle_hash, &answer_hash));
		let pot_ledger = T::PuzzleLedger::get_pot_ledger(puzzle_hash);
		assert!(pot_ledger.is_some());
		let pot_ledger = pot_ledger.unwrap();
		assert_eq!(pot_ledger.sponsor_list.len(), 2u32 as usize, "Sponsor list size is 2");
	}

	commit_challenge {
		let caller: T::AccountId = wrap_account_with_balance::<T>(whitelisted_caller());
		let puzzle_hash = "liIoGRFRiXTFOAga2G-TXVu6stHjq4ZDPEcET6v21iw".as_bytes().to_vec();
		let answer_hash = "aaa3ef39d81f3a78f75f8c1a5bc454401746697f86930a809717b3503debd9cd".as_bytes().to_vec();
		let final_answer_raw =  "C".as_bytes().to_vec();
		handler_create_puzzle::<T>(puzzle_hash.clone(), answer_hash.clone());
		handler_answer_puzzle::<T>(caller.clone(), puzzle_hash.clone(), final_answer_raw.clone(), "PuzzleAnswerExplain".as_bytes().to_vec());
	}: _(RawOrigin::Signed(caller), puzzle_hash.clone(), get_dollars::<T>(1000))
	verify {
		assert!(<PuzzleDirectAnswer<T>>::contains_key(&puzzle_hash, &final_answer_raw));
		let challenge_status = T::AtoChallenge::get_challenge_status(&puzzle_hash);
		assert!(challenge_status.is_some());
		log::info!(" challenge_status === {:?}", challenge_status);

	}

    challenge_pull_out {
		frame_system::Pallet::<T>::set_block_number(100u32.into());
		let caller: T::AccountId = wrap_account_with_balance::<T>(whitelisted_caller());
		let puzzle_hash = "liIoGRFRiXTFOAga2G-TXVu6stHjq4ZDPEcET6v21iw".as_bytes().to_vec();
		let answer_hash = "aaa3ef39d81f3a78f75f8c1a5bc454401746697f86930a809717b3503debd9cd".as_bytes().to_vec();
		let final_answer_raw =  "C".as_bytes().to_vec();
		handler_create_puzzle::<T>(puzzle_hash.clone(), answer_hash.clone());
		handler_answer_puzzle::<T>(caller.clone(), puzzle_hash.clone(), final_answer_raw.clone(), "PuzzleAnswerExplain".as_bytes().to_vec());
		handler_commit_challenge::<T>(caller.clone(), puzzle_hash.clone(), get_dollars::<T>(10));
		frame_system::Pallet::<T>::set_block_number(50000u32.into());
		log::info!("blocknumber = {:?}", <frame_system::Pallet<T>>::block_number());
	}: _(RawOrigin::Signed(caller), puzzle_hash.clone())
	verify {
		assert!(<PuzzleDirectAnswer<T>>::contains_key(&puzzle_hash, &final_answer_raw));
		let challenge_status = T::AtoChallenge::get_challenge_status(&puzzle_hash);
		log::info!("challenge_status AA = {:?}", &challenge_status);
		assert_eq!(challenge_status, Some(ChallengeStatus::RaiseFundsBack(50000u32.into(),Perbill::from_percent(10))));
		assert!(challenge_status.is_some());
	}

	challenge_crowdloan {
		frame_system::Pallet::<T>::set_block_number(100u32.into());
		let caller: T::AccountId = wrap_account_with_balance::<T>(whitelisted_caller());
		let puzzle_hash = "liIoGRFRiXTFOAga2G-TXVu6stHjq4ZDPEcET6v21iw".as_bytes().to_vec();
		let answer_hash = "aaa3ef39d81f3a78f75f8c1a5bc454401746697f86930a809717b3503debd9cd".as_bytes().to_vec();
		let final_answer_raw =  "C".as_bytes().to_vec();
		handler_create_puzzle::<T>(puzzle_hash.clone(), answer_hash.clone());
		handler_answer_puzzle::<T>(caller.clone(), puzzle_hash.clone(), final_answer_raw.clone(), "PuzzleAnswerExplain".as_bytes().to_vec());
		handler_commit_challenge::<T>(caller.clone(), puzzle_hash.clone(), get_dollars::<T>(10));
		log::info!("blocknumber = {:?}", <frame_system::Pallet<T>>::block_number());
	}: _(RawOrigin::Signed(caller), puzzle_hash.clone(), get_dollars::<T>(10))
	verify {
		assert!(<PuzzleDirectAnswer<T>>::contains_key(&puzzle_hash, &final_answer_raw));
		let challenge_status = T::AtoChallenge::get_challenge_status(&puzzle_hash);
		log::info!("challenge_status : {:?}", &challenge_status);
		assert!(challenge_status.is_some());
		let challengers = T::AtoChallenge::get_list_of_challengers(&puzzle_hash);
		log::info!("challengers : {:?}", &challengers);
		assert_eq!(challengers.len(), 2);

	}

	take_answer_reward {
		let caller: T::AccountId = wrap_account_with_balance::<T>(whitelisted_caller());
		let puzzle_hash = "liIoGRFRiXTFOAga2G-TXVu6stHjq4ZDPEcET6v21iw".as_bytes().to_vec();
		let answer_hash = "aaa3ef39d81f3a78f75f8c1a5bc454401746697f86930a809717b3503debd9cd".as_bytes().to_vec();
		let final_answer_raw =  "C".as_bytes().to_vec();
		handler_create_puzzle::<T>(puzzle_hash.clone(), answer_hash.clone());
		handler_answer_puzzle::<T>(caller.clone(), puzzle_hash.clone(), final_answer_raw.clone(), "PuzzleAnswerExplain".as_bytes().to_vec());
		//
		frame_system::Pallet::<T>::set_block_number(50000u32.into());
	}: _(RawOrigin::Signed(caller), puzzle_hash.clone())
	verify {
		assert!(<PuzzleDirectAnswer<T>>::contains_key(&puzzle_hash, &final_answer_raw));
		assert!(<PuzzleInfo<T>>::contains_key(&puzzle_hash));
		let puzzle_info = <PuzzleInfo<T>>::get(&puzzle_hash);
		assert!(puzzle_info.is_some());
		let puzzle_info = puzzle_info.unwrap();
		assert_eq!(puzzle_info.puzzle_status, PuzzleStatus::PUZZLE_STATUS_IS_FINAL);
	}

	// #[pallet::weight(100)]
	// 	pub fn recognition_challenge(
	// 		origin: OriginFor<T>,
	// 		puzzle_hash: PuzzleSubjectHash, // Arweave tx - id

	impl_benchmark_test_suite!(
		AtochaModule,
		crate::mock::new_test_ext(),
		crate::mock::Test,
	);
}

