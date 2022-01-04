#![no_std]
use super::Event as AtochaEvent;
use crate::mock::toVec;
use crate::mock::AccountId;
use crate::pallet::*;
use crate::{mock::*, Error};
use frame_support::sp_runtime::app_crypto::sr25519::Signature;
use frame_support::sp_runtime::traits::{IdentifyAccount, Verify, Zero};
use frame_support::{assert_noop, assert_ok};
use pallet_atofinance::traits::*;

use crate::types::*;
use sp_core::hexdisplay::HexDisplay;
use sp_core::sr25519::Public;
use sp_runtime::AccountId32;
use pallet_atofinance::imps::challenge_manager::ChallengeManager;
use pallet_atofinance::types::ChallengeStatus;

const CONST_ORIGIN_IS_CREATOR: u8 = 1;
const CONST_ORIGIN_IS_ANSWER_1: u8 = 2;
const CONST_ORIGIN_IS_ANSWER_2: u8 = 3;
const CONST_ORIGIN_IS_ANSWER_3: u8 = 4;
const CONST_ORIGIN_IS_ANSWER_4: u8 = 5;

#[test]
fn test_create_puzzle() {
	new_test_ext().execute_with(|| {
		System::set_block_number(5);

		let puzzle_hash = toVec("PUZZLE_TX_ID");
		let answer_plain_txt = toVec("ANSWER_HASH_256");
		let answer_hash = make_answer_sha256(answer_plain_txt.clone(), puzzle_hash.clone());

		// println!("sha256_answer= {:?}", hex::encode(answer_hash.clone()));

		handle_create_puzzle(
			toAid(CONST_ORIGIN_IS_CREATOR),
			puzzle_hash.clone(),
			answer_hash.clone(),
		);

		// (PuzzleSubjectHash, PuzzleAnswerHash, PuzzleTicket, PuzzleRelationType, PuzzleStatus, u64, u64 )
		let relation_info = AtochaModule::puzzle_info(puzzle_hash.clone()).unwrap();

		assert_eq!(
			relation_info,
			PuzzleInfoData {
				account: toAid(CONST_ORIGIN_IS_CREATOR),
				answer_hash,
				answer_explain: None,
				// answer_nonce: toVec("NONCE"),
				puzzle_status: PuzzleStatus::PUZZLE_STATUS_IS_SOLVING,
				create_bn: 5,
				reveal_answer: None,
				reveal_bn: None,
				puzzle_version: 1,
			}
		);
		//
		System::assert_last_event(
			AtochaEvent::PuzzleCreated(
				toAid(CONST_ORIGIN_IS_CREATOR),
				puzzle_hash, //.as_bytes().to_vec(),
				5,
			)
			.into(),
		);
	});
}

#[test]
fn test_answer_puzzle() {
	new_test_ext().execute_with(|| {
		System::set_block_number(5);

		// let puzzle_hash_err_ex_id = toVec("PUZZLE_TX_ERR_ID");
		// let puzzle_hash = toVec("PUZZLE_TX_ID");
		// let answer_hash = toVec("ANSWER_HASH_256");
		// let sha256_answer = make_answer_sha256(answer_hash.clone(), puzzle_hash.clone());

		let puzzle_hash = toVec("PUZZLE_TX_ID");
		let answer_plain_txt = toVec("ANSWER_HASH_256");
		let answer_plain_txt_err = toVec("ANSWER_HASH_ERROR_256");
		let answer_hash = make_answer_sha256(answer_plain_txt.clone(), puzzle_hash.clone());

		// check initial status.
		let answer_answer = AtochaModule::puzzle_direct_answer(&puzzle_hash, &answer_plain_txt);
		assert_eq!(None, answer_answer);

		// if puzzle not exists.
		assert_noop!(
			// Try to call create answer, but the puzzle not exists.
			AtochaModule::answer_puzzle(
				Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_1)),
				puzzle_hash.clone(),
				answer_plain_txt.clone(),
			),
			Error::<Test>::PuzzleNotExist
		);

		// Create puzzle hash on the chain.
		handle_create_puzzle(
			toAid(CONST_ORIGIN_IS_CREATOR),
			puzzle_hash.clone(),
			answer_hash.clone(),
		);

		System::set_block_number(15);

		assert_ok!(AtochaModule::answer_puzzle(
			Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_1)),
			puzzle_hash.clone(),
			answer_plain_txt_err.clone(),
		));

		let answer_list =
			<PuzzleDirectAnswer<Test>>::iter_prefix(puzzle_hash.clone()).collect::<Vec<_>>(); // ::puzzle_direct_answer(&toVec("PUZZLE_HASH"));
		assert_eq!(1, answer_list.len());

		assert_eq!(
			answer_list[0],
			(
				answer_plain_txt_err.clone(),
				PuzzleAnswerData {
					account: toAid(CONST_ORIGIN_IS_ANSWER_1),
					// puzzle_ticket: 500,
					answer_status: PuzzleAnswerStatus::ANSWER_HASH_IS_MISMATCH,
					create_bn: 15,
				}
			)
		);

		// Check puzzle status.
		let relation_info = AtochaModule::puzzle_info(puzzle_hash.clone()).unwrap();
		assert_eq!(
			relation_info,
			PuzzleInfoData {
				account: toAid(CONST_ORIGIN_IS_CREATOR),
				answer_hash: answer_hash.clone(),
				answer_explain: None,
				puzzle_status: PuzzleStatus::PUZZLE_STATUS_IS_SOLVING,
				create_bn: 5,
				reveal_answer: None,
				reveal_bn: None,
				puzzle_version: 1,
			}
		);

		// ------------
		assert_ok!(AtochaModule::answer_puzzle(
			Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_1)),
			puzzle_hash.clone(),
			answer_plain_txt.clone(),
		));

		// check answer list count.
		let answer_list =
			<PuzzleDirectAnswer<Test>>::iter_prefix(puzzle_hash.clone()).collect::<Vec<_>>(); // ::puzzle_direct_answer(&toVec("PUZZLE_HASH"));
		assert_eq!(2, answer_list.len());

		assert_eq!(
			answer_list[1],
			(
				answer_plain_txt.clone(),
				PuzzleAnswerData {
					account: toAid(CONST_ORIGIN_IS_ANSWER_1),
					// puzzle_ticket: 500,
					answer_status: PuzzleAnswerStatus::ANSWER_HASH_IS_MATCH,
					create_bn: 15,
				}
			)
		);

		// Check puzzle status.
		let relation_info = AtochaModule::puzzle_info(puzzle_hash.clone()).unwrap();
		assert_eq!(
			relation_info,
			PuzzleInfoData {
				account: toAid(CONST_ORIGIN_IS_CREATOR),
				answer_hash: answer_hash.clone(),
				answer_explain: None,
				puzzle_status: PuzzleStatus::PUZZLE_STATUS_IS_SOLVED,
				create_bn: 5,
				reveal_answer: Some(toAid(CONST_ORIGIN_IS_ANSWER_1)),
				reveal_bn: Some(15),
				puzzle_version: 1,
			}
		);

		assert_noop!(
			// Try to call create answer, but the puzzle not exists.
			AtochaModule::answer_puzzle(
				Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_1)),
				puzzle_hash.clone(),
				answer_plain_txt.clone(),
			),
			Error::<Test>::PuzzleHasBeenSolved
		);
	});
}

#[test]
fn test_take_answer_reward_with_crator() {
	new_test_ext().execute_with(|| {
		System::set_block_number(5);

		let puzzle_hash = toVec("PUZZLE_TX_ID");
		let answer_plain_txt = toVec("ANSWER_HASH_256");
		let answer_plain_txt_err = toVec("ANSWER_HASH_ERROR_256");
		let answer_hash = make_answer_sha256(answer_plain_txt.clone(), puzzle_hash.clone());

		// Create puzzle hash on the chain.
		handle_create_puzzle(
			toAid(CONST_ORIGIN_IS_CREATOR),
			puzzle_hash.clone(),
			answer_hash.clone(),
		);

		// Check puzzle no answers.
		let answer_list =
			<PuzzleDirectAnswer<Test>>::iter_prefix(puzzle_hash.clone()).collect::<Vec<_>>(); // ::puzzle_direct_answer(&toVec("PUZZLE_HASH"));
		assert_eq!(0, answer_list.len());

		let mut puzzle_content = <PuzzleInfo<Test>>::get(&puzzle_hash).unwrap();
		assert_eq!(puzzle_content.puzzle_status, PuzzleStatus::PUZZLE_STATUS_IS_SOLVING);

		// Check puzzle no win answers.
		System::set_block_number(15);

		assert_ok!(AtochaModule::answer_puzzle(
			Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_1)),
			puzzle_hash.clone(),
			answer_plain_txt_err.clone(),
		));

		let answer_list =
			<PuzzleDirectAnswer<Test>>::iter_prefix(puzzle_hash.clone()).collect::<Vec<_>>(); // ::puzzle_direct_answer(&toVec("PUZZLE_HASH"));
		assert_eq!(1, answer_list.len());

		let mut puzzle_content = <PuzzleInfo<Test>>::get(&puzzle_hash).unwrap();
		assert_eq!(puzzle_content.puzzle_status, PuzzleStatus::PUZZLE_STATUS_IS_SOLVING);

		// bn < ChallengePeriodLength: BlockNumber = 100;
		System::set_block_number(15 + 50);

		assert_noop!(
			// Try to call create answer, but the puzzle not exists.
			AtochaModule::take_answer_reward(
				Origin::signed(toAid(CONST_ORIGIN_IS_CREATOR)),
				puzzle_hash.clone(),
			),
			Error::<Test>::PuzzleStatusErr
		);

		// Update sure answer on chain.
		assert_ok!(AtochaModule::answer_puzzle(
			Origin::signed(toAid(CONST_ORIGIN_IS_CREATOR)),
			puzzle_hash.clone(),
			answer_plain_txt.clone(),
		));

		let mut puzzle_content = <PuzzleInfo<Test>>::get(&puzzle_hash).unwrap();
		assert_eq!(puzzle_content.puzzle_status, PuzzleStatus::PUZZLE_STATUS_IS_SOLVED);

		assert_noop!(
			// Try to call create answer, but the puzzle not exists.
			AtochaModule::take_answer_reward(
				Origin::signed(toAid(CONST_ORIGIN_IS_CREATOR)),
				puzzle_hash.clone(),
			),
			Error::<Test>::ChallengePeriodIsNotEnd
		);

		// Now you can receive rewards.
		System::set_block_number( 15 + 50 + 100 );
		assert_noop!(
			// Try to call create answer, but the puzzle not exists.
			AtochaModule::take_answer_reward(
				Origin::signed(toAid(CONST_ORIGIN_IS_CREATOR)),
				puzzle_hash.clone(),
			),
			Error::<Test>::ChallengePeriodIsNotEnd
		);
		System::set_block_number(15 + 50 + 100 + 1 );
		let original_balance = Balances::free_balance(toAid(CONST_ORIGIN_IS_CREATOR));
		let original_point = <pallet_atofinance::imps::PointManager<Test>>::get_total_points(&toAid(CONST_ORIGIN_IS_CREATOR));
		assert_eq!(original_point, Zero::zero());

		assert_ok!(AtochaModule::take_answer_reward(
				Origin::signed(toAid(CONST_ORIGIN_IS_CREATOR)),
				puzzle_hash.clone(),
		));
		// Deduct TVS tax.
		assert_eq!(Balances::free_balance(toAid(CONST_ORIGIN_IS_CREATOR)), original_balance + 100*DOLLARS - TaxOfTVS::get() * 100*DOLLARS);
		let reward_period_count = (65 - 5) / PerEraOfBlockNumber::get();
		let winanswer_remain_points: Balance = 100*DOLLARS * reward_period_count as Balance;

		let total_bonus = pallet_atofinance::imps::PointReward::<Test>::get_total_bonus(&puzzle_hash, 65);
		assert_eq!(total_bonus.unwrap(), winanswer_remain_points);
		assert_eq!(<pallet_atofinance::imps::PointManager<Test>>::get_total_points(&toAid(CONST_ORIGIN_IS_CREATOR)),
				   original_point + winanswer_remain_points - TaxOfTVS::get() * winanswer_remain_points);

	});
}

#[test]
fn test_take_answer_reward_with_other() {
	new_test_ext().execute_with(|| {
		System::set_block_number(5);

		let puzzle_hash = toVec("PUZZLE_TX_ID");
		let answer_plain_txt = toVec("ANSWER_HASH_256");
		let answer_plain_txt_err = toVec("ANSWER_HASH_ERROR_256");
		let answer_hash = make_answer_sha256(answer_plain_txt.clone(), puzzle_hash.clone());

		// Create puzzle hash on the chain.
		handle_create_puzzle(
			toAid(CONST_ORIGIN_IS_CREATOR),
			puzzle_hash.clone(),
			answer_hash.clone(),
		);

		// Check puzzle no answers.
		let answer_list =
			<PuzzleDirectAnswer<Test>>::iter_prefix(puzzle_hash.clone()).collect::<Vec<_>>(); // ::puzzle_direct_answer(&toVec("PUZZLE_HASH"));
		assert_eq!(0, answer_list.len());

		let mut puzzle_content = <PuzzleInfo<Test>>::get(&puzzle_hash).unwrap();
		assert_eq!(puzzle_content.puzzle_status, PuzzleStatus::PUZZLE_STATUS_IS_SOLVING);

		// Check puzzle no win answers.
		System::set_block_number(15);

		assert_ok!(AtochaModule::answer_puzzle(
			Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_1)),
			puzzle_hash.clone(),
			answer_plain_txt_err.clone(),
		));

		let answer_list =
			<PuzzleDirectAnswer<Test>>::iter_prefix(puzzle_hash.clone()).collect::<Vec<_>>(); // ::puzzle_direct_answer(&toVec("PUZZLE_HASH"));
		assert_eq!(1, answer_list.len());

		let mut puzzle_content = <PuzzleInfo<Test>>::get(&puzzle_hash).unwrap();
		assert_eq!(puzzle_content.puzzle_status, PuzzleStatus::PUZZLE_STATUS_IS_SOLVING);

		// bn < ChallengePeriodLength: BlockNumber = 100;
		System::set_block_number(15 + 50);

		assert_noop!(
			// Try to call create answer, but the puzzle not exists.
			AtochaModule::take_answer_reward(
				Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_1)),
				puzzle_hash.clone(),
			),
			Error::<Test>::PuzzleStatusErr
		);

		// Update sure answer on chain.
		assert_ok!(AtochaModule::answer_puzzle(
			Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_2)),
			puzzle_hash.clone(),
			answer_plain_txt.clone(),
		));

		let mut puzzle_content = <PuzzleInfo<Test>>::get(&puzzle_hash).unwrap();
		assert_eq!(puzzle_content.puzzle_status, PuzzleStatus::PUZZLE_STATUS_IS_SOLVED);

		assert_noop!(
			// Try to call create answer, but the puzzle not exists.
			AtochaModule::take_answer_reward(
				Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_2)),
				puzzle_hash.clone(),
			),
			Error::<Test>::ChallengePeriodIsNotEnd
		);

		// Now you can receive rewards.
		System::set_block_number( 15 + 50 + 100 );
		assert_noop!(
			// Try to call create answer, but the puzzle not exists.
			AtochaModule::take_answer_reward(
				Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_2)),
				puzzle_hash.clone(),
			),
			Error::<Test>::ChallengePeriodIsNotEnd
		);
		System::set_block_number(15 + 50 + 100 + 1 );
		let original_balance = Balances::free_balance(toAid(CONST_ORIGIN_IS_ANSWER_2));
		let original_point = <pallet_atofinance::imps::PointManager<Test>>::get_total_points(&toAid(CONST_ORIGIN_IS_CREATOR));
		assert_eq!(original_point, Zero::zero());

		assert_noop!(
			// Try to call create answer, but the puzzle not exists.
			AtochaModule::take_answer_reward(
				Origin::signed(toAid(CONST_ORIGIN_IS_CREATOR)),
				puzzle_hash.clone(),
			),
			Error::<Test>::NoRightToReward
		);

		assert_ok!(AtochaModule::take_answer_reward(
				Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_2)),
				puzzle_hash.clone(),
		));


		// Deduct TVS tax.
		assert_eq!(Balances::free_balance(toAid(CONST_ORIGIN_IS_ANSWER_2)), original_balance + 100*DOLLARS - TaxOfTVO::get() * 100*DOLLARS);
		let reward_period_count = (65 - 5) / PerEraOfBlockNumber::get();
		let winanswer_remain_points: Balance = 100*DOLLARS * reward_period_count as Balance;

		let total_bonus = pallet_atofinance::imps::PointReward::<Test>::get_total_bonus(&puzzle_hash, 65);
		assert_eq!(total_bonus.unwrap(), winanswer_remain_points);

		assert_eq!(<pallet_atofinance::imps::PointManager<Test>>::get_total_points(&toAid(CONST_ORIGIN_IS_ANSWER_2)),
				   original_point + winanswer_remain_points - TaxOfTVO::get() * winanswer_remain_points);
	});
}


#[test]
fn test_take_answer_reward_with_challenge_win() {
	new_test_ext().execute_with(|| {
		System::set_block_number(5);

		let puzzle_hash = toVec("PUZZLE_TX_ID");
		let answer_plain_txt = toVec("ANSWER_HASH_256");
		let answer_plain_txt_err = toVec("ANSWER_HASH_ERROR_256");
		let answer_hash = make_answer_sha256(answer_plain_txt.clone(), puzzle_hash.clone());

		// Create puzzle hash on the chain.
		handle_create_puzzle(
			toAid(CONST_ORIGIN_IS_CREATOR),
			puzzle_hash.clone(),
			answer_hash.clone(),
		);

		// Check puzzle no answers.
		let answer_list =
			<PuzzleDirectAnswer<Test>>::iter_prefix(puzzle_hash.clone()).collect::<Vec<_>>(); // ::puzzle_direct_answer(&toVec("PUZZLE_HASH"));
		assert_eq!(0, answer_list.len());

		let mut puzzle_content = <PuzzleInfo<Test>>::get(&puzzle_hash).unwrap();
		assert_eq!(puzzle_content.puzzle_status, PuzzleStatus::PUZZLE_STATUS_IS_SOLVING);

		// Check puzzle no win answers.
		System::set_block_number(15);

		assert_ok!(AtochaModule::answer_puzzle(
			Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_1)),
			puzzle_hash.clone(),
			answer_plain_txt_err.clone(),
		));

		let answer_list =
			<PuzzleDirectAnswer<Test>>::iter_prefix(puzzle_hash.clone()).collect::<Vec<_>>(); // ::puzzle_direct_answer(&toVec("PUZZLE_HASH"));
		assert_eq!(1, answer_list.len());

		let mut puzzle_content = <PuzzleInfo<Test>>::get(&puzzle_hash).unwrap();
		assert_eq!(puzzle_content.puzzle_status, PuzzleStatus::PUZZLE_STATUS_IS_SOLVING);

		// bn < ChallengePeriodLength: BlockNumber = 100;
		System::set_block_number(15 + 50);

		assert_noop!(
			// Try to call create answer, but the puzzle not exists.
			AtochaModule::take_answer_reward(
				Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_1)),
				puzzle_hash.clone(),
			),
			Error::<Test>::PuzzleStatusErr
		);

		// Update sure answer on chain.
		assert_ok!(AtochaModule::answer_puzzle(
			Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_2)),
			puzzle_hash.clone(),
			answer_plain_txt.clone(),
		));

		let mut puzzle_content = <PuzzleInfo<Test>>::get(&puzzle_hash).unwrap();
		assert_eq!(puzzle_content.puzzle_status, PuzzleStatus::PUZZLE_STATUS_IS_SOLVED);

		assert_noop!(
			// Try to call create answer, but the puzzle not exists.
			AtochaModule::take_answer_reward(
				Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_2)),
				puzzle_hash.clone(),
			),
			Error::<Test>::ChallengePeriodIsNotEnd
		);

		// Now you can receive rewards.
		System::set_block_number( 15 + 50 + 100 );
		assert_noop!(
			// Try to call create answer, but the puzzle not exists.
			AtochaModule::take_answer_reward(
				Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_2)),
				puzzle_hash.clone(),
			),
			Error::<Test>::ChallengePeriodIsNotEnd
		);

		System::set_block_number(15 + 50 + 100 + 1 );
		assert_noop!(
			// Try to call create answer, but the puzzle not exists.
			AtochaModule::commit_challenge(
				Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_3)),
				puzzle_hash.clone(),
				10 * DOLLARS
			),
			Error::<Test>::ChallengePeriodIsEnd
		);

		System::set_block_number( 15 + 50 + 100 );
		// --- Be challenged
		assert_ok!(AtochaModule::commit_challenge(
				Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_3)),
				puzzle_hash.clone(),
				10 * DOLLARS
		));

		System::set_block_number(15 + 50 + 100 + 1 );
		let original_balance = Balances::free_balance(toAid(CONST_ORIGIN_IS_ANSWER_2));
		let original_point = <pallet_atofinance::imps::PointManager<Test>>::get_total_points(&toAid(CONST_ORIGIN_IS_CREATOR));
		assert_eq!(original_point, Zero::zero());

		assert_noop!(
			// Try to call create answer, but the puzzle not exists.
			AtochaModule::take_answer_reward(
				Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_2)),
				puzzle_hash.clone(),
			),
			Error::<Test>::BeingChallenged
		);

		// Begin raise
		assert_ok!(AtochaModule::challenge_crowdloan(
				Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_4)),
				puzzle_hash.clone(),
				100 * DOLLARS
		));

		assert_eq!(ChallengeManager::<Test>::get_total_raise(&puzzle_hash) , 60 * DOLLARS);
		assert_eq!(
			ChallengeManager::<Test>::get_challenge_status(&puzzle_hash).unwrap(),
			ChallengeStatus::RaiseCompleted(15 + 50 + 100 + 1)
		);

		System::set_block_number(15 + 50 + 200 + 1 );

		assert_noop!(
			// Try to call create answer, but the puzzle not exists.
			AtochaModule::take_answer_reward(
				Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_2)),
				puzzle_hash.clone(),
			),
			Error::<Test>::BeingChallenged
		);

		//
		assert_ok!(AtochaModule::recognition_challenge (
				Origin::root(),
				puzzle_hash.clone(),
		));


		assert_noop!(
			// Try to call create answer, but the puzzle not exists.
			AtochaModule::take_answer_reward(
				Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_2)),
				puzzle_hash.clone(),
			),
			Error::<Test>::PuzzleStatusErr
		);
	});
}

#[test]
fn test_take_answer_reward_with_challenge_faild() {
	new_test_ext().execute_with(|| {
		System::set_block_number(5);

		let puzzle_hash = toVec("PUZZLE_TX_ID");
		let answer_plain_txt = toVec("ANSWER_HASH_256");
		let answer_plain_txt_err = toVec("ANSWER_HASH_ERROR_256");
		let answer_hash = make_answer_sha256(answer_plain_txt.clone(), puzzle_hash.clone());

		// Create puzzle hash on the chain.
		handle_create_puzzle(
			toAid(CONST_ORIGIN_IS_CREATOR),
			puzzle_hash.clone(),
			answer_hash.clone(),
		);

		// Check puzzle no answers.
		let answer_list =
			<PuzzleDirectAnswer<Test>>::iter_prefix(puzzle_hash.clone()).collect::<Vec<_>>(); // ::puzzle_direct_answer(&toVec("PUZZLE_HASH"));
		assert_eq!(0, answer_list.len());

		let mut puzzle_content = <PuzzleInfo<Test>>::get(&puzzle_hash).unwrap();
		assert_eq!(puzzle_content.puzzle_status, PuzzleStatus::PUZZLE_STATUS_IS_SOLVING);

		// Check puzzle no win answers.
		System::set_block_number(15);

		assert_ok!(AtochaModule::answer_puzzle(
			Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_1)),
			puzzle_hash.clone(),
			answer_plain_txt_err.clone(),
		));

		let answer_list =
			<PuzzleDirectAnswer<Test>>::iter_prefix(puzzle_hash.clone()).collect::<Vec<_>>(); // ::puzzle_direct_answer(&toVec("PUZZLE_HASH"));
		assert_eq!(1, answer_list.len());

		let mut puzzle_content = <PuzzleInfo<Test>>::get(&puzzle_hash).unwrap();
		assert_eq!(puzzle_content.puzzle_status, PuzzleStatus::PUZZLE_STATUS_IS_SOLVING);

		// bn < ChallengePeriodLength: BlockNumber = 100;
		System::set_block_number(15 + 50);

		assert_noop!(
			// Try to call create answer, but the puzzle not exists.
			AtochaModule::take_answer_reward(
				Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_1)),
				puzzle_hash.clone(),
			),
			Error::<Test>::PuzzleStatusErr
		);

		// Update sure answer on chain.
		assert_ok!(AtochaModule::answer_puzzle(
			Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_2)),
			puzzle_hash.clone(),
			answer_plain_txt.clone(),
		));

		let mut puzzle_content = <PuzzleInfo<Test>>::get(&puzzle_hash).unwrap();
		assert_eq!(puzzle_content.puzzle_status, PuzzleStatus::PUZZLE_STATUS_IS_SOLVED);

		assert_noop!(
			// Try to call create answer, but the puzzle not exists.
			AtochaModule::take_answer_reward(
				Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_2)),
				puzzle_hash.clone(),
			),
			Error::<Test>::ChallengePeriodIsNotEnd
		);

		// Now you can receive rewards.
		System::set_block_number( 15 + 50 + 100 );
		assert_noop!(
			// Try to call create answer, but the puzzle not exists.
			AtochaModule::take_answer_reward(
				Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_2)),
				puzzle_hash.clone(),
			),
			Error::<Test>::ChallengePeriodIsNotEnd
		);

		System::set_block_number(15 + 50 + 100 + 1 );
		assert_noop!(
			// Try to call create answer, but the puzzle not exists.
			AtochaModule::commit_challenge(
				Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_3)),
				puzzle_hash.clone(),
				10 * DOLLARS
			),
			Error::<Test>::ChallengePeriodIsEnd
		);

		System::set_block_number( 15 + 50 + 100 );
		// --- Be challenged
		assert_ok!(AtochaModule::commit_challenge(
				Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_3)),
				puzzle_hash.clone(),
				10 * DOLLARS
		));

		System::set_block_number(15 + 50 + 100 + 1 );
		let original_balance = Balances::free_balance(toAid(CONST_ORIGIN_IS_ANSWER_2));
		let original_point = <pallet_atofinance::imps::PointManager<Test>>::get_total_points(&toAid(CONST_ORIGIN_IS_CREATOR));
		assert_eq!(original_point, Zero::zero());

		assert_noop!(
			// Try to call create answer, but the puzzle not exists.
			AtochaModule::take_answer_reward(
				Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_2)),
				puzzle_hash.clone(),
			),
			Error::<Test>::BeingChallenged
		);

		// Begin raise
		assert_ok!(AtochaModule::challenge_crowdloan(
				Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_4)),
				puzzle_hash.clone(),
				100 * DOLLARS
		));

		assert_eq!(ChallengeManager::<Test>::get_total_raise(&puzzle_hash) , 60 * DOLLARS);
		assert_eq!(
			ChallengeManager::<Test>::get_challenge_status(&puzzle_hash).unwrap(),
			ChallengeStatus::RaiseCompleted(15 + 50 + 100 + 1)
		);

		System::set_block_number(15 + 50 + 200 + 1 );

		assert_noop!(
			// Try to call create answer, but the puzzle not exists.
			AtochaModule::take_answer_reward(
				Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_2)),
				puzzle_hash.clone(),
			),
			Error::<Test>::BeingChallenged
		);

		//
		assert_ok!(AtochaModule::refuse_challenge (
			Origin::root(),
			puzzle_hash.clone(),
		));

		assert_ok!(AtochaModule::take_answer_reward(
				Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_2)),
				puzzle_hash.clone(),
		));

		// Deduct TVS tax.
		assert_eq!(Balances::free_balance(toAid(CONST_ORIGIN_IS_ANSWER_2)), original_balance + 100*DOLLARS - TaxOfTVO::get() * 100*DOLLARS);
		let reward_period_count = (65 - 5) / PerEraOfBlockNumber::get();
		let winanswer_remain_points: Balance = 100*DOLLARS * reward_period_count as Balance;

		let total_bonus = pallet_atofinance::imps::PointReward::<Test>::get_total_bonus(&puzzle_hash, 65);
		assert_eq!(total_bonus.unwrap(), winanswer_remain_points);

		assert_eq!(<pallet_atofinance::imps::PointManager<Test>>::get_total_points(&toAid(CONST_ORIGIN_IS_ANSWER_2)),
				   original_point + winanswer_remain_points - TaxOfTVO::get() * winanswer_remain_points);
	});
}
#[test]
// fn test_issue_challenge_and_take_answer_reward() {
// 	new_test_ext().execute_with(|| {
// 		System::set_block_number(5);
//
// 		let puzzle_hash = toVec("PUZZLE_TX_ID");
// 		let answer_plain_txt = toVec("ANSWER_HASH_256");
// 		let answer_plain_txt_err = toVec("ANSWER_HASH_ERROR_256");
// 		let answer_hash = make_answer_sha256(answer_plain_txt.clone(), puzzle_hash.clone());
//
// 		// Create puzzle hash on the chain.
// 		handle_create_puzzle(
// 			toAid(CONST_ORIGIN_IS_CREATOR),
// 			puzzle_hash.clone(),
// 			answer_hash.clone(),
// 		);
//
//
// 		// Check puzzle no answers.
// 		let answer_list =
// 			<PuzzleDirectAnswer<Test>>::iter_prefix(puzzle_hash.clone()).collect::<Vec<_>>(); // ::puzzle_direct_answer(&toVec("PUZZLE_HASH"));
// 		assert_eq!(0, answer_list.len());
//
// 		let mut puzzle_content = <PuzzleInfo<Test>>::get(&puzzle_hash).unwrap();
// 		assert_eq!(puzzle_content.puzzle_status, PuzzleStatus::PUZZLE_STATUS_IS_SOLVING);
//
// 		// Check puzzle no win answers.
// 		System::set_block_number(15);
//
// 		assert_ok!(AtochaModule::answer_puzzle(
// 			Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_1)),
// 			puzzle_hash.clone(),
// 			answer_plain_txt_err.clone(),
// 		));
//
// 		let answer_list =
// 			<PuzzleDirectAnswer<Test>>::iter_prefix(puzzle_hash.clone()).collect::<Vec<_>>(); // ::puzzle_direct_answer(&toVec("PUZZLE_HASH"));
// 		assert_eq!(1, answer_list.len());
//
// 		let mut puzzle_content = <PuzzleInfo<Test>>::get(&puzzle_hash).unwrap();
// 		assert_eq!(puzzle_content.puzzle_status, PuzzleStatus::PUZZLE_STATUS_IS_SOLVING);
//
// 		// When the puzzle status is solving, issue a challenge for it and you cannot success at the time.
// 		assert_noop!(
// 			// Try to call create answer, but the puzzle not exists.
// 			AtochaModule::issue_challenge(
// 				Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_2)),
// 				puzzle_hash.clone(),
// 				1000 * DOLLARS
// 			),
// 			Error::<Test>::PuzzleNotSolvedChallengeFailed
// 		);
//
// 		// Update sure answer on chain.
// 		assert_ok!(AtochaModule::answer_puzzle(
// 			Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_1)),
// 			puzzle_hash.clone(),
// 			answer_plain_txt.clone(),
// 		));
//
// 		assert_ok!(AtochaModule::issue_challenge(
// 			Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_2)),
// 			puzzle_hash.clone(),
// 			1000 * DOLLARS
// 		));
//
// 		// Check whether the challenge period has expired.
// 		assert_noop!(
// 			// Try to call create answer, but the puzzle not exists.
// 			AtochaModule::take_answer_reward(
// 				Origin::signed(toAid(CONST_ORIGIN_IS_ANSWER_1)),
// 				puzzle_hash.clone(),
// 				1000 * DOLLARS
// 			),
// 			Error::<Test>::PuzzleNotSolvedChallengeFailed
// 		);
//
// 		// Check if there are active challenges
//
// 		// Now you can receive rewards.
//
// 		assert!(false, "not implements");
// 	});
// }

#[test]
fn test_handler_reveal_signed_valid() {
	new_test_ext().execute_with(|| {
        use frame_support::sp_runtime::app_crypto::{Pair, Ss58Codec, TryFrom};
        use sp_application_crypto::sr25519::Public;

        let test_signature = &hex::decode("2aeaa98e26062cf65161c68c5cb7aa31ca050cb5bdd07abc80a475d2a2eebc7b7a9c9546fbdff971b29419ddd9982bf4148c81a49df550154e1674a6b58bac84").expect("Hex invalid")[..];
        let public_id =  Public::from_ss58check("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty").unwrap();
        assert!(AtochaModule::check_signed_valid(public_id, test_signature, "This is a text message".as_bytes()));
    });
}

#[test]
fn test_signed_method() {
	new_test_ext().execute_with(|| {
        System::set_block_number(5);
        //
        use sp_application_crypto::sr25519;
        use sp_application_crypto::sr25519::Signature;
        use sp_runtime::MultiSignature;
        use sp_runtime::MultiSigner;
        use frame_support::sp_runtime::app_crypto::{Pair, Ss58Codec, TryFrom};
        use frame_support::sp_runtime::traits::{IdentifyAccount, Verify};
        use sp_application_crypto::sr25519::Public;

        // sp_core::sr25519::Pair(schnorrkel::Keypair).;

        // let result = AuthorityPair::verify(signature.into(), signature.into(), test_address.into());
        // assert!(result, "Result is true.")

        let msg = &b"test-message"[..];
        let (pair, _) = sr25519::Pair::generate();

        let signature = pair.sign(&msg);
        assert!(sr25519::Pair::verify(&signature, msg, &pair.public()));

        // println!("msg = {:?}", &msg);
        // println!("signature = {:?}", &signature);
        // println!("pair.public() = {:?}", &pair.public());
        // println!("multi_signer.into_account() = {:?}", &multi_signer.into_account());

        let multi_sig = MultiSignature::from(signature); // OK
        let multi_signer = MultiSigner::from(pair.public());
        assert!(multi_sig.verify(msg, &multi_signer.into_account()));

        let multi_signer = MultiSigner::from(pair.public());
        assert!(multi_sig.verify(msg, &multi_signer.into_account()));

        //---------

        let test_signature = &hex::decode("2aeaa98e26062cf65161c68c5cb7aa31ca050cb5bdd07abc80a475d2a2eebc7b7a9c9546fbdff971b29419ddd9982bf4148c81a49df550154e1674a6b58bac84").expect("Hex invalid")[..];
        let signature = Signature::try_from(test_signature);
        let signature = signature.unwrap();
        println!(" signature = {:?}", signature);

        // let account_result =  AccountId::from_ss58check("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty");
        // let account_id = account_result.unwrap();
        // println!(" account_id = {:?} ", account_id);

        let public_id = Public::from_ss58check("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty");
        let public_id = public_id.unwrap();
        println!(" public_id = {:?} ", public_id);

        let multi_sig = MultiSignature::from(signature); // OK
        let multi_signer = MultiSigner::from(public_id);
        assert!(multi_sig.verify("This is a text message".as_bytes(), &multi_signer.into_account()));

        //
        let account_pair = sr25519::Pair::from_string("blur pioneer frown science banana impose avoid law act strategy have bronze//2//stash", None).unwrap();
        let make_public = account_pair.public();
        let make_signature = account_pair.sign("This is a text message".as_bytes());
        let multi_sig = MultiSignature::from(make_signature); // OK
        let multi_signer = MultiSigner::from(make_public);
        assert!(multi_sig.verify("This is a text message".as_bytes(), &multi_signer.into_account()));

    });
}
