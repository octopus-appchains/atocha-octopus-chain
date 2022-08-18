use super::*;
use crate::AtoFinanceLedger;

#[test]
fn test_do_bonus() {
	new_test_ext().execute_with(|| {
		System::set_block_number(5);

		const ACCOUNT_ID_1: u64 = 1;
		const ACCOUNT_ID_2: u64 = 2;
		// Dispatch a signed extrinsic.
		assert_eq!(Balances::free_balance(ACCOUNT_ID_1), 100_000_000_000_000);
		assert_eq!(Balances::free_balance(ACCOUNT_ID_2), 200_000_000_000_000);

		//
		let puzzle_hash = toVec("TEST_PUZZLE_HASH");

		// assert_noop!(
		// 	AtochaPot::do_bonus(puzzle_hash.clone(), ACCOUNT_ID_1, 150000000000000),
		// 	Error::<Test>::InsufficientBalance
		// );

		// Get Error::<Test>::InsufficientBalance
		let res = AtochaPot::do_bonus(
			puzzle_hash.clone(),
			ACCOUNT_ID_1,
			150_000_000_000_000,
			5u32.into(),
		);
		assert!(res.is_err());

		// pid: PuzzleSubjectHash,
		// who: T::AccountId,
		// amount: BalanceOf<T>,
		assert_ok!(AtochaPot::do_bonus(
			puzzle_hash.clone(),
			ACCOUNT_ID_1,
			50_000_000_000_000,
			5u32.into()
		));
		assert_eq!(Balances::free_balance(ACCOUNT_ID_1), 50_000_000_000_000);
		let pot_ledger = AtoFinanceLedger::<Test>::get(&puzzle_hash).unwrap();
		assert_eq!(pot_ledger.funds, 50_000_000_000_000);
		assert_eq!(pot_ledger.total, 50_000_000_000_000);

		// Change owner not allowed.
		assert_noop!(
			AtochaPot::do_bonus(puzzle_hash.clone(), ACCOUNT_ID_2, 50_000_000_000_000, 5u32.into()),
			Error::<Test>::LedgerOwnerNotMatch
		);

		// Additional bound
		assert_ok!(AtochaPot::do_bonus(
			puzzle_hash.clone(),
			ACCOUNT_ID_1,
			10_000_000_000_000,
			5u32.into()
		));
		assert_eq!(Balances::free_balance(ACCOUNT_ID_1), 40_000_000_000_000);
		let pot_ledger = AtoFinanceLedger::<Test>::get(&puzzle_hash).unwrap();
		assert_eq!(pot_ledger.funds, 60_000_000_000_000);
		assert_eq!(pot_ledger.total, 60_000_000_000_000);
	});
}

#[test]
fn test_do_sponsorship() {
	new_test_ext().execute_with(|| {
		const ACCOUNT_ID_1: u64 = 1;
		const ACCOUNT_ID_2: u64 = 2;
		// Dispatch a signed extrinsic.
		assert_eq!(Balances::free_balance(ACCOUNT_ID_1), 100_000_000_000_000);
		assert_eq!(Balances::free_balance(ACCOUNT_ID_2), 200_000_000_000_000);

		//
		let puzzle_hash = toVec("TEST_PUZZLE_HASH");
		// puzzle must exists.
		assert_noop!(
			AtochaPot::do_sponsorship(
				puzzle_hash.clone(),
				ACCOUNT_ID_1,
				20_000_000_000_000,
				1u32.into(),
				"Some-Things".as_bytes().to_vec()
			),
			Error::<Test>::PuzzleNotExists
		);

		assert_ok!(AtochaPot::do_bonus(
			puzzle_hash.clone(),
			ACCOUNT_ID_1,
			10_000_000_000_000,
			5u32.into()
		));
		assert_eq!(Balances::free_balance(ACCOUNT_ID_1), 90_000_000_000_000);
		let pot_ledger = AtoFinanceLedger::<Test>::get(&puzzle_hash).unwrap();
		assert_eq!(pot_ledger.funds, 10_000_000_000_000);
		assert_eq!(pot_ledger.total, 10_000_000_000_000);

		assert_ok!(AtochaPot::do_sponsorship(
			puzzle_hash.clone(),
			ACCOUNT_ID_1,
			20_000_000_000_000,
			5u32.into(), // block number
			"Some-Things-1".as_bytes().to_vec()
		));
		let pot_ledger = AtoFinanceLedger::<Test>::get(&puzzle_hash).unwrap();
		assert_eq!(pot_ledger.funds, 10_000_000_000_000);
		assert_eq!(pot_ledger.total, 30_000_000_000_000);
		assert_eq!(pot_ledger.sponsor_list.len(), 2);
		assert_eq!(
			pot_ledger.sponsor_list[0],
			SponsorData {
				sponsor: ACCOUNT_ID_1,
				funds: 20_000_000_000_000,
				create_bn: 5,
				reason: toVec("Some-Things-1")
			}
		);
		assert_eq!(
			pot_ledger.sponsor_list[1],
			SponsorData {
				sponsor: ACCOUNT_ID_1,
				funds: 10_000_000_000_000,
				create_bn: 5,
				reason: Vec::new(),
			}
		);

		assert_ok!(AtochaPot::do_sponsorship(
			puzzle_hash.clone(),
			ACCOUNT_ID_2,
			30_000_000_000_000,
			6u32.into(), // block number
			"Some-Things-2".as_bytes().to_vec()
		));
		let pot_ledger = AtoFinanceLedger::<Test>::get(&puzzle_hash).unwrap();
		assert_eq!(pot_ledger.funds, 10_000_000_000_000);
		assert_eq!(pot_ledger.total, 60_000_000_000_000);
		assert_eq!(pot_ledger.sponsor_list.len(), 3);
		assert_eq!(
			pot_ledger.sponsor_list[0],
			SponsorData {
				sponsor: ACCOUNT_ID_2,
				funds: 30_000_000_000_000,
				create_bn: 6,
				reason: toVec("Some-Things-2")
			}
		);
		assert_eq!(
			pot_ledger.sponsor_list[1],
			SponsorData {
				sponsor: ACCOUNT_ID_1,
				funds: 20_000_000_000_000,
				create_bn: 5,
				reason: toVec("Some-Things-1")
			}
		);

		assert_eq!(
			pot_ledger.sponsor_list[2],
			SponsorData {
				sponsor: ACCOUNT_ID_1,
				funds: 10_000_000_000_000,
				create_bn: 5,
				reason: Vec::new(),
			}
		);
	});
}
