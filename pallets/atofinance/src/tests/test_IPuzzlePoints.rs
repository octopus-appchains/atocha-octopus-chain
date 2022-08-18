use super::*;
use crate::imps::PointManager;
use crate::{AtoFinanceLedger};

// fn get_total_points() -> PToken;
// fn increase_points_to(who: &AccountId) -> DResult;
// fn reduce_points_to(who: &AccountId) -> DResult;
// fn get_issuance_points() -> PToken;

#[test]
fn test_point_manager() {
	new_test_ext().execute_with(|| {
		System::set_block_number(5);

		const ACCOUNT_ID_1: u64 = 1;
		const ACCOUNT_ID_2: u64 = 2;
		const ACCOUNT_ID_3: u64 = 3;
		const ACCOUNT_ID_4: u64 = 4;
		const ACCOUNT_ID_5: u64 = 5;
		const ACCOUNT_ID_6: u64 = 6;
		const ACCOUNT_ID_7: u64 = 7;
		const ACCOUNT_ID_8: u64 = 8;
		const ACCOUNT_ID_9: u64 = 9;

		// Dispatch a signed extrinsic.
		assert_eq!(<PointManager<Test>>::get_total_points(&ACCOUNT_ID_1), 0);
		assert_eq!(<PointManager<Test>>::get_total_points(&ACCOUNT_ID_2), 0);
		assert_eq!(<PointManager<Test>>::get_issuance_points(), 0);
		assert_ok!(<PointManager<Test>>::increase_points_to(&ACCOUNT_ID_1, 100_000_000_000_000));
		assert_eq!(<PointManager<Test>>::get_issuance_points(), 100_000_000_000_000);
		assert_ok!(<PointManager<Test>>::increase_points_to(&ACCOUNT_ID_2, 50_000_000_000_000));
		assert_eq!(<PointManager<Test>>::get_issuance_points(), 150_000_000_000_000);
		assert_eq!(<PointManager<Test>>::get_total_points(&ACCOUNT_ID_1), 100_000_000_000_000);
		assert_eq!(<PointManager<Test>>::get_total_points(&ACCOUNT_ID_2), 50_000_000_000_000);

		// assert_eq!(<PointManager<Test>>::get_ranking_safety_count(), 3);
		// assert_eq!(<PointManager<Test>>::get_ranking_list(), vec![
		// 	(ACCOUNT_ID_1, 100_000_000_000_000),
		// 	(ACCOUNT_ID_2, 50_000_000_000_000),
		// ]);
		//
		// assert_ok!(<PointManager<Test>>::increase_points_to(&ACCOUNT_ID_2, 50_000_000_000_000));
		// assert_eq!(<PointManager<Test>>::get_issuance_points(), 200_000_000_000_000);
		// assert_eq!(<PointManager<Test>>::get_ranking_list(), vec![
		// 	(ACCOUNT_ID_1, 100_000_000_000_000),
		// 	(ACCOUNT_ID_2, 100_000_000_000_000),
		// ]);
		//
		// assert_ok!(<PointManager<Test>>::increase_points_to(&ACCOUNT_ID_2, 50_000_000_000_000));
		// assert_eq!(<PointManager<Test>>::get_issuance_points(), 250_000_000_000_000);
		// assert_eq!(<PointManager<Test>>::get_ranking_list(), vec![
		// 	(ACCOUNT_ID_2, 150_000_000_000_000),
		// 	(ACCOUNT_ID_1, 100_000_000_000_000),
		// ]);
		//
		// assert_ok!(<PointManager<Test>>::increase_points_to(&ACCOUNT_ID_3, 300_000_000_000_000));
		// assert_eq!(<PointManager<Test>>::get_issuance_points(), 550_000_000_000_000);
		// assert_eq!(<PointManager<Test>>::get_ranking_list(), vec![
		// 	(ACCOUNT_ID_3, 300_000_000_000_000),
		// 	(ACCOUNT_ID_2, 150_000_000_000_000),
		// 	(ACCOUNT_ID_1, 100_000_000_000_000),
		// ]);
		//
		// assert_ok!(<PointManager<Test>>::increase_points_to(&ACCOUNT_ID_4, 50_000_000_000_000));
		// assert_eq!(<PointManager<Test>>::get_issuance_points(), 600_000_000_000_000);
		// assert_eq!(<PointManager<Test>>::get_ranking_list(), vec![
		// 	(ACCOUNT_ID_3, 300_000_000_000_000),
		// 	(ACCOUNT_ID_2, 150_000_000_000_000),
		// 	(ACCOUNT_ID_1, 100_000_000_000_000),
		// ]);
		//
		// assert_ok!(<PointManager<Test>>::increase_points_to(&ACCOUNT_ID_5, 50_000_000_000_000));
		// assert_eq!(<PointManager<Test>>::get_issuance_points(), 650_000_000_000_000);
		// assert_eq!(<PointManager<Test>>::get_ranking_list(), vec![
		// 	(ACCOUNT_ID_3, 300_000_000_000_000),
		// 	(ACCOUNT_ID_2, 150_000_000_000_000),
		// 	(ACCOUNT_ID_1, 100_000_000_000_000),
		// ]);
		//
		// assert_ok!(<PointManager<Test>>::reduce_points_to(&ACCOUNT_ID_3, 200_000_000_000_000));
		// assert_eq!(<PointManager<Test>>::get_issuance_points(), 350_000_000_000_000);
		// assert_eq!(<PointManager<Test>>::get_ranking_list(), vec![
		// 	(ACCOUNT_ID_2, 150_000_000_000_000),
		// 	(ACCOUNT_ID_3, 100_000_000_000_000),
		// 	(ACCOUNT_ID_1, 100_000_000_000_000),
		// ]);
		//
		// assert_ok!(<PointManager<Test>>::reduce_points_to(&ACCOUNT_ID_2, 150_000_000_000_000));
		// assert_eq!(<PointManager<Test>>::get_issuance_points(), 200_000_000_000_000);
		// assert_eq!(<PointManager<Test>>::get_ranking_list(), vec![
		// 	(ACCOUNT_ID_3, 100_000_000_000_000),
		// 	(ACCOUNT_ID_1, 100_000_000_000_000),
		// 	(ACCOUNT_ID_4, 50_000_000_000_000),
		// ]);


	});
}

#[test]
fn test_calculate_points_of_puzzle() {
	new_test_ext().execute_with(|| {
		System::set_block_number(5);

		const ACCOUNT_ID_1: u64 = 1;
		const ACCOUNT_ID_2: u64 = 2;
		const ACCOUNT_ID_3: u64 = 3;

		let puzzle_hash = toVec("ABC");
		// puzzle must exists.
		assert_ok!(AtochaPot::do_bonus(
			puzzle_hash.clone(),
			ACCOUNT_ID_1,
			10_000_000_000_000,
			5u32.into()
		));

		assert_ok!(AtochaPot::do_sponsorship(
			puzzle_hash.clone(),
			ACCOUNT_ID_1,
			20_000_000_000_000,
			15u32.into(), // block number
			"Some-Things-1".as_bytes().to_vec()
		));

		assert_ok!(AtochaPot::do_sponsorship(
			puzzle_hash.clone(),
			ACCOUNT_ID_2,
			10_000_000_000_000,
			30u32.into(), // block number
			"Some-Things-2".as_bytes().to_vec()
		));

		assert_ok!(AtochaPot::do_sponsorship(
			puzzle_hash.clone(),
			ACCOUNT_ID_3,
			30_000_000_000_000,
			30u32.into(), // block number
			"Some-Things-2".as_bytes().to_vec()
		));

		let pot_ledger = AtoFinanceLedger::<Test>::get(&puzzle_hash).unwrap();
		assert_eq!(pot_ledger.funds, 10_000_000_000_000);
		assert_eq!(pot_ledger.total, 70_000_000_000_000);
		assert_eq!(pot_ledger.sponsor_list.len(), 4);

		// Calculate the total reward-points, assuming per-era=5 blocks and current block-number is 60.
		// diff	                            durn-period	  Point
		// 60	30	30,000,000,000,000 	30	6			180,000,000,000,000
		// 60	30	10,000,000,000,000 	30	6			60,000,000,000,000
		// 60	15	20,000,000,000,000 	45	9			180,000,000,000,000
		// 60	5	10,000,000,000,000 	55	11			110,000,000,000,000
		// SUM:						          			530,000,000,000,000
		let current_bn = 60u64;
		assert_eq!(
			PointManager::<Test>::calculate_points_of_puzzle(current_bn, &puzzle_hash, 5),
			530_000_000_000_000
		);

		assert_eq!(
			<PointManager<Test>>::calculate_points_of_puzzle(
				100,
				&toVec("TEST_PUZZLE_HASH-ERROR"),
				5
			),
			0
		);

	});
}
