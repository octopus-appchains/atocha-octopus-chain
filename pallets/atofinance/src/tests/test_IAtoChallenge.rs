use super::*;
use crate::imps::challenge_manager::ChallengeManager;

#[test]
fn test_issue_challenge() {
	new_test_ext().execute_with(|| {
		System::set_block_number(5);

		const ACCOUNT_ID_1: u64 = 1;
		const ACCOUNT_ID_2: u64 = 2;
		const ACCOUNT_ID_3: u64 = 3;
		const ACCOUNT_ID_4: u64 = 4;
		// Dispatch a signed extrinsic.
		assert_eq!(Balances::free_balance(ACCOUNT_ID_1), 100_000_000_000_000);
		assert_eq!(Balances::free_balance(ACCOUNT_ID_2), 200_000_000_000_000);
		assert_eq!(Balances::free_balance(ACCOUNT_ID_3), 300_000_000_000_000);
		assert_eq!(Balances::free_balance(ACCOUNT_ID_4), 400_000_000_000_000);

		//
		let puzzle_hash = toVec("TEST_PUZZLE_HASH");
		// puzzle ledger must exists.
		assert_noop!(
			ChallengeManager::<Test>::issue_challenge(
				ACCOUNT_ID_2,
				&puzzle_hash,
				2_000_000_000_000,
			),
			Error::<Test>::PuzzleNotExists
		);

		// Create puzzle ledger.
		assert_ok!(AtochaPot::do_bonus(
			puzzle_hash.clone(),
			ACCOUNT_ID_1,
			10_000_000_000_000,
			5u32.into()
		));
		assert_eq!(Balances::free_balance(ACCOUNT_ID_1), 100_000_000_000_000 - 10_000_000_000_000);

		assert_ok!(ChallengeManager::<Test>::issue_challenge(
			ACCOUNT_ID_2,
			&puzzle_hash,
			2_000_000_000_000,
		));
		assert_eq!(Balances::free_balance(ACCOUNT_ID_2), 200_000_000_000_000 - 2_000_000_000_000);

		// puzzle ledger must exists.
		assert_noop!(
			ChallengeManager::<Test>::issue_challenge(
				ACCOUNT_ID_1,
				&puzzle_hash,
				2_000_000_000_000,
			),
			Error::<Test>::ChallengeAlreadyExists
		);

		// Check
		assert_eq!(
			ChallengeManager::<Test>::get_balance_threshold(&puzzle_hash),
			Perbill::from_percent(60) * 10_000_000_000_000
		);
		assert_eq!(ChallengeManager::<Test>::get_total_raise(&puzzle_hash), 2_000_000_000_000);
		assert_eq!(ChallengeManager::<Test>::has_the_raising_period_expired(&puzzle_hash), false);
		assert_eq!(
			ChallengeManager::<Test>::get_challenge_status(&puzzle_hash),
			Some(ChallengeStatus::Raise(5))
		);

		// Add raise
		assert_ok!(ChallengeManager::<Test>::challenge_crowdloan(
			ACCOUNT_ID_2,
			&puzzle_hash,
			3_000_000_000_000,
		));
		assert_eq!(
			ChallengeManager::<Test>::get_total_raise(&puzzle_hash),
			2_000_000_000_000 + 3_000_000_000_000
		);
		assert_eq!(
			Balances::free_balance(ACCOUNT_ID_2),
			200_000_000_000_000 - (2_000_000_000_000 + 3_000_000_000_000)
		);

		// Add raise
		assert_ok!(ChallengeManager::<Test>::challenge_crowdloan(
			ACCOUNT_ID_3,
			&puzzle_hash,
			3_000_000_000_000,
		));

		assert_eq!(ChallengeManager::<Test>::get_total_raise(&puzzle_hash), 6_000_000_000_000);
		// Because the remaining raised funds have been met
		assert_eq!(Balances::free_balance(ACCOUNT_ID_3), 300_000_000_000_000 - 1_000_000_000_000);
		//
		assert_noop!(
			ChallengeManager::<Test>::challenge_crowdloan(
				ACCOUNT_ID_4,
				&puzzle_hash,
				2_000_000_000_000,
			),
			Error::<Test>::EndOfRaising
		);

		assert_eq!(
			ChallengeManager::<Test>::get_challenge_status(&puzzle_hash),
			Some(ChallengeStatus::RaiseCompleted(5))
		);

		let challenge_data = ChallengeManager::<Test>::check_get_active_challenge_info(&puzzle_hash);
		assert!(challenge_data.is_ok());
		let challenge_data = challenge_data.unwrap();
		assert_eq!(
			challenge_data,
			PuzzleChallengeData {
				raised_total: 6_000_000_000_000,
				status: ChallengeStatus::RaiseCompleted(5),
				create_bn: 5,
				creator: ACCOUNT_ID_2,
				start_bn: None,
				end_bn: None,
				raised_group: vec![
					(ACCOUNT_ID_2, 2_000_000_000_000),
					(ACCOUNT_ID_2, 3_000_000_000_000),
					(ACCOUNT_ID_3, 1_000_000_000_000),
				]
			}
		);
	});
}

#[test]
fn test_issue_challenge_raise_expired() {
	new_test_ext().execute_with(|| {
		System::set_block_number(5);
		const ACCOUNT_ID_1: u64 = 1;
		const ACCOUNT_ID_2: u64 = 2;
		const ACCOUNT_ID_3: u64 = 3;
		//
		let puzzle_hash = toVec("TEST_PUZZLE_HASH");
		// Create puzzle ledger.
		assert_ok!(AtochaPot::do_bonus(
			puzzle_hash.clone(),
			ACCOUNT_ID_1,
			10_000_000_000_000,
			5u32.into()
		));
		assert_ok!(ChallengeManager::<Test>::issue_challenge(
			ACCOUNT_ID_2,
			&puzzle_hash,
			2_000_000_000_000,
		));

		System::set_block_number(20);
		//
		assert_noop!(
			ChallengeManager::<Test>::challenge_crowdloan(
				ACCOUNT_ID_3,
				&puzzle_hash,
				2_000_000_000_000,
			),
			Error::<Test>::RaisingPeriodExpired
		);

		assert_eq!(
			ChallengeManager::<Test>::get_challenge_status(&puzzle_hash),
			Some(ChallengeStatus::Raise(5))
		);
	});
}

#[test]
fn test_issue_challenge_raise_direct_done() {
	new_test_ext().execute_with(|| {
		System::set_block_number(5);
		const ACCOUNT_ID_1: u64 = 1;
		const ACCOUNT_ID_2: u64 = 2;
		//
		let puzzle_hash = toVec("TEST_PUZZLE_HASH");
		// Create puzzle ledger.
		assert_ok!(AtochaPot::do_bonus(
			puzzle_hash.clone(),
			ACCOUNT_ID_1,
			10_000_000_000_000,
			5u32.into()
		));
		assert_ok!(ChallengeManager::<Test>::issue_challenge(
			ACCOUNT_ID_2,
			&puzzle_hash,
			6_000_000_000_000,
		));

		assert_eq!(
			ChallengeManager::<Test>::get_challenge_status(&puzzle_hash),
			Some(ChallengeStatus::RaiseCompleted(5))
		);
	});
}
