use super::*;
use crate::AtoFinanceLedger;
use crate::imps::point_exchange::PointExchange;
use crate::imps::PointManager;

#[test]
fn test_do_bonus() {
	new_test_ext().execute_with(|| {
		System::set_block_number(5);

		System::set_block_number(5);
		assert_eq!(<PointExchange<Test>>::get_max_reward_count(), 3);
		assert_eq!(<PointExchange<Test>>::get_history_depth(), 3);
		assert_eq!(<PointExchange<Test>>::get_era_length(), 10);
		assert_eq!(<PointExchange<Test>>::get_current_era(), 0);
		System::set_block_number(10);
		assert_eq!(<PointExchange<Test>>::get_current_era(), 1);
		System::set_block_number(15);
		assert_eq!(<PointExchange<Test>>::get_current_era(), 1);
		System::set_block_number(20);
		assert_eq!(<PointExchange<Test>>::get_current_era(), 2);
		System::set_block_number(25);
		assert_eq!(<PointExchange<Test>>::get_current_era(), 2);
		assert_eq!(<PointExchange<Test>>::get_reward_list(2), vec![]);

		// apply
		const ACCOUNT_ID_1: u64 = 1;
		<PointManager<Test>>::increase_points_to(&ACCOUNT_ID_1,100);
		const ACCOUNT_ID_2: u64 = 2;
		<PointManager<Test>>::increase_points_to(&ACCOUNT_ID_2,200);
		const ACCOUNT_ID_3: u64 = 3;
		<PointManager<Test>>::increase_points_to(&ACCOUNT_ID_3,300);
		const ACCOUNT_ID_4: u64 = 4;
		<PointManager<Test>>::increase_points_to(&ACCOUNT_ID_4,400);
		const ACCOUNT_ID_5: u64 = 5;
		<PointManager<Test>>::increase_points_to(&ACCOUNT_ID_5,500);
		const ACCOUNT_ID_6: u64 = 6;
		<PointManager<Test>>::increase_points_to(&ACCOUNT_ID_6,600);

		assert_ok!(<PointExchange<Test>>::apply_exchange(ACCOUNT_ID_3));
		// Vec<(T::AccountId, PointToken, Option<ExchangeInfo<PointToken, BalanceOf<T>, Perbill>>)>
		assert_eq!(<PointExchange<Test>>::get_reward_list(2), vec![
			(ACCOUNT_ID_3, 300, None),
		]);


		assert_ok!(<PointExchange<Test>>::apply_exchange(ACCOUNT_ID_5));
		assert_eq!(<PointExchange<Test>>::get_reward_list(2), vec![
			(ACCOUNT_ID_5, 500, None),
			(ACCOUNT_ID_3, 300, None),
		]);

		assert_ok!(<PointExchange<Test>>::apply_exchange(ACCOUNT_ID_4));
		assert_eq!(<PointExchange<Test>>::get_reward_list(2), vec![
			(ACCOUNT_ID_5, 500, None),
			(ACCOUNT_ID_4, 400, None),
			(ACCOUNT_ID_3, 300, None),
		]);

		assert_ok!(<PointExchange<Test>>::apply_exchange(ACCOUNT_ID_6));
		assert_eq!(<PointExchange<Test>>::get_reward_list(2), vec![
			(ACCOUNT_ID_6, 500, None),
			(ACCOUNT_ID_5, 500, None),
			(ACCOUNT_ID_4, 400, None),
		]);

		assert_noop!(<PointExchange<Test>>::apply_exchange(ACCOUNT_ID_3), Error::<Test>::TooFewPoints);
		assert_noop!(<PointExchange<Test>>::apply_exchange(ACCOUNT_ID_2), Error::<Test>::TooFewPoints);
		assert_noop!(<PointExchange<Test>>::apply_exchange(ACCOUNT_ID_1), Error::<Test>::TooFewPoints);
		assert_noop!(<PointExchange<Test>>::apply_exchange(ACCOUNT_ID_6), Error::<Test>::ExchangeApplied);
		assert_noop!(<PointExchange<Test>>::apply_exchange(ACCOUNT_ID_5), Error::<Test>::ExchangeApplied);
		assert_noop!(<PointExchange<Test>>::apply_exchange(ACCOUNT_ID_4), Error::<Test>::ExchangeApplied);

		//
	});
}

