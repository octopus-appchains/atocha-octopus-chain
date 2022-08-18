use super::*;
use crate::mock::init_puzzle_ledger;
use crate::*;

#[test]
fn test_answer_get_reward_with_creator() {
	new_test_ext().execute_with(|| {
		let current_bn = 10;
		System::set_block_number(current_bn);

		const ACCOUNT_ID_1: u64 = 1;
		const ACCOUNT_ID_2: u64 = 2;
		const ACCOUNT_ID_3: u64 = 3;
		assert_eq!(Balances::free_balance(ACCOUNT_ID_1), 100_000_000_000_000);
		assert_eq!(Balances::free_balance(ACCOUNT_ID_2), 200_000_000_000_000);
		assert_eq!(Balances::free_balance(ACCOUNT_ID_3), 300_000_000_000_000);

		let puzzle_hash = toVec("TEST_PUZZLE_HASH");
		init_puzzle_ledger(puzzle_hash.clone());
		assert_eq!(<TokenReward<Test>>::get_total_bonus(&puzzle_hash, current_bn), Some(60_000_000_000_000));

		assert_eq!(Balances::free_balance(ACCOUNT_ID_1), 70_000_000_000_000);

		// Try to claim rewards of empty puzzle
		assert_noop!(
			<TokenReward<Test>>::answer_get_reward(
				&toVec("TEST_PUZZLE_HASH-ERROR"),
				ACCOUNT_ID_1,
				current_bn,
				Perbill::from_percent(10)
			),
			Error::<Test>::PuzzleNotExists,
		);

		assert_ok!(<TokenReward<Test>>::answer_get_reward(
			&puzzle_hash,
			ACCOUNT_ID_1,
			current_bn,
			Perbill::from_percent(10)
		));
		assert_eq!(
			60_000_000_000_000u128
				.saturating_sub(Perbill::from_percent(10) * 60_000_000_000_000u128),
			54_000_000_000_000u128
		);

		assert_eq!(Balances::free_balance(ACCOUNT_ID_1), 70_000_000_000_000 + 54_000_000_000_000);

		// Check storage.
		// pub create_bn: BlockNumber,
		// pub tax: PerVal,
		// pub reward_type: RewardType,
		// pub total: BalanceOf,
		// pub payout: BalanceOf,
		// pub beneficiaries: Vec<(Account, PerVal)>,
		let pot_reward_record = AtoFinanceReward::<Test>::get(&puzzle_hash).unwrap();
		let total_reward_balance = <TokenReward<Test>>::get_total_bonus(&puzzle_hash, current_bn).unwrap();
		assert_eq!(
			pot_reward_record,
			PotRewardData {
				create_bn: 10,
				tax: Perbill::from_percent(10),
				reward_type: RewardType::CreatorReward,
				total: total_reward_balance,
				payout: Perbill::from_percent(100 - 10) * total_reward_balance,
				beneficiaries: vec![(ACCOUNT_ID_1, Perbill::from_percent(100))]
			}
		);

		// Try to claim rewards repeatedly.
		assert_noop!(
			<TokenReward<Test>>::answer_get_reward(
				&puzzle_hash,
				ACCOUNT_ID_1,
				current_bn,
				Perbill::from_percent(10)
			),
			Error::<Test>::RewardHasBeenClaimed,
		);

		// Try to claim rewards repeatedly.
		assert_noop!(
			<TokenReward<Test>>::answer_get_reward(
				&puzzle_hash,
				ACCOUNT_ID_3,
				current_bn,
				Perbill::from_percent(10)
			),
			Error::<Test>::RewardHasBeenClaimed,
		);
	});
}

#[test]
fn test_answer_get_reward_with_other() {
	new_test_ext().execute_with(|| {
		let current_bn = 10;
		System::set_block_number(current_bn);

		const ACCOUNT_ID_1: u64 = 1;
		const ACCOUNT_ID_2: u64 = 2;
		const ACCOUNT_ID_3: u64 = 3;
		assert_eq!(Balances::free_balance(ACCOUNT_ID_1), 100_000_000_000_000);
		assert_eq!(Balances::free_balance(ACCOUNT_ID_2), 200_000_000_000_000);
		assert_eq!(Balances::free_balance(ACCOUNT_ID_3), 300_000_000_000_000);

		let puzzle_hash = toVec("TEST_PUZZLE_HASH");
		init_puzzle_ledger(puzzle_hash.clone());
		assert_eq!(<TokenReward<Test>>::get_total_bonus(&puzzle_hash, current_bn), Some(60_000_000_000_000));

		assert_eq!(Balances::free_balance(ACCOUNT_ID_1), 70_000_000_000_000);

		// Try to claim rewards of empty puzzle
		assert_noop!(
			<TokenReward<Test>>::answer_get_reward(
				&toVec("TEST_PUZZLE_HASH-ERROR"),
				ACCOUNT_ID_3,
				current_bn,
				Perbill::from_percent(10)
			),
			Error::<Test>::PuzzleNotExists,
		);

		assert_ok!(<TokenReward<Test>>::answer_get_reward(
			&puzzle_hash,
			ACCOUNT_ID_3,
			current_bn,
			Perbill::from_percent(10)
		));
		assert_eq!(
			60_000_000_000_000u128
				.saturating_sub(Perbill::from_percent(10) * 60_000_000_000_000u128),
			54_000_000_000_000u128
		);

		assert_eq!(Balances::free_balance(ACCOUNT_ID_3), 300_000_000_000_000 + 54_000_000_000_000);

		// Check storage.
		// pub create_bn: BlockNumber,
		// pub tax: PerVal,
		// pub reward_type: RewardType,
		// pub total: BalanceOf,
		// pub payout: BalanceOf,
		// pub beneficiaries: Vec<(Account, PerVal)>,
		let pot_reward_record = AtoFinanceReward::<Test>::get(&puzzle_hash).unwrap();
		let total_reward_balance = <TokenReward<Test>>::get_total_bonus(&puzzle_hash, current_bn).unwrap();
		assert_eq!(
			pot_reward_record,
			PotRewardData {
				create_bn: 10,
				tax: Perbill::from_percent(10),
				reward_type: RewardType::AnswerReward,
				total: total_reward_balance,
				payout: Perbill::from_percent(100 - 10) * total_reward_balance,
				beneficiaries: vec![(ACCOUNT_ID_3, Perbill::from_percent(100))]
			}
		);

		// Try to claim rewards repeatedly.
		assert_noop!(
			<TokenReward<Test>>::answer_get_reward(
				&puzzle_hash,
				ACCOUNT_ID_1,
				current_bn,
				Perbill::from_percent(10)
			),
			Error::<Test>::RewardHasBeenClaimed,
		);

		// Try to claim rewards repeatedly.
		assert_noop!(
			<TokenReward<Test>>::answer_get_reward(
				&puzzle_hash,
				ACCOUNT_ID_3,
				current_bn,
				Perbill::from_percent(10)
			),
			Error::<Test>::RewardHasBeenClaimed,
		);
	});
}

#[test]
fn test_challenge_get_reward() {
	new_test_ext().execute_with(|| {
		let current_bn = 60;
		System::set_block_number(current_bn);

		const ACCOUNT_ID_1: u64 = 1;
		const ACCOUNT_ID_2: u64 = 2;
		const ACCOUNT_ID_3: u64 = 3;
		assert_eq!(Balances::free_balance(ACCOUNT_ID_1), 100_000_000_000_000);
		assert_eq!(Balances::free_balance(ACCOUNT_ID_2), 200_000_000_000_000);
		assert_eq!(Balances::free_balance(ACCOUNT_ID_3), 300_000_000_000_000);

		let puzzle_hash = toVec("TEST_PUZZLE_HASH");
		init_puzzle_ledger(puzzle_hash.clone());
		assert_eq!(<TokenReward<Test>>::get_total_bonus(&puzzle_hash, current_bn), Some(60_000_000_000_000));

		assert_eq!(Balances::free_balance(ACCOUNT_ID_1), 70_000_000_000_000);
		assert_eq!(Balances::free_balance(ACCOUNT_ID_2), 170_000_000_000_000);

		// Change owner not allowed.
		// Try to claim rewards of empty puzzle
		assert_noop!(
			<TokenReward<Test>>::challenge_get_reward(
				&toVec("TEST_PUZZLE_HASH-ERROR"),
				vec![
					(ACCOUNT_ID_1, Perbill::from_percent(30)),
					(ACCOUNT_ID_2, Perbill::from_percent(70)),
				],
				current_bn,
				Perbill::from_percent(10)
			),
			Error::<Test>::PuzzleNotExists,
		);

		assert_noop!(
			<TokenReward<Test>>::challenge_get_reward(
				&toVec("TEST_PUZZLE_HASH"),
				vec![],
				current_bn,
				Perbill::from_percent(10)
			),
			Error::<Test>::BeneficiaryListNotEmpty,
		);

		assert_noop!(
			<TokenReward<Test>>::challenge_get_reward(
				&toVec("TEST_PUZZLE_HASH"),
				vec![
					(ACCOUNT_ID_1, Perbill::from_percent(30)),
					(ACCOUNT_ID_2, Perbill::from_percent(40)),
					(ACCOUNT_ID_3, Perbill::from_percent(40)),
				],
				current_bn,
				Perbill::from_percent(10)
			),
			Error::<Test>::WrongPaymentRatio,
		);

		assert_ok!(<TokenReward<Test>>::challenge_get_reward(
			&puzzle_hash,
			vec![
				(ACCOUNT_ID_1, Perbill::from_percent(30)),
				(ACCOUNT_ID_2, Perbill::from_percent(70)),
			],
			current_bn,
			Perbill::from_percent(10)
		));

		assert_eq!(
			60_000_000_000_000u128
				.saturating_sub(Perbill::from_percent(10) * 60_000_000_000_000u128),
			54_000_000_000_000u128
		);

		assert_eq!(Perbill::from_percent(30) * 54_000_000_000_000u128, 16_200_000_000_000u128);
		assert_eq!(Perbill::from_percent(70) * 54_000_000_000_000u128, 37_800_000_000_000u128);

		assert_eq!(
			Balances::free_balance(ACCOUNT_ID_1),
			70_000_000_000_000 + 16_200_000_000_000u128
		);
		assert_eq!(
			Balances::free_balance(ACCOUNT_ID_2),
			170_000_000_000_000 + 37_800_000_000_000u128
		);

		let pot_reward_record = AtoFinanceReward::<Test>::get(&puzzle_hash).unwrap();
		let total_reward_token = <TokenReward<Test>>::get_total_bonus(&puzzle_hash, current_bn).unwrap();
		assert_eq!(
			pot_reward_record,
			PotRewardData {
				create_bn: 60,
				tax: Perbill::from_percent(10),
				reward_type: RewardType::ChallengerReward,
				total: total_reward_token,
				payout: Perbill::from_percent(100 - 10) * total_reward_token,
				beneficiaries: vec![
					(ACCOUNT_ID_1, Perbill::from_percent(30)),
					(ACCOUNT_ID_2, Perbill::from_percent(70)),
				]
			}
		);
	});
}
