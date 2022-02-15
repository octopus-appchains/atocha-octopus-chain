use super::*;
use crate::{AtoFinanceLedger, CurrentExchangeRewardEra, ExchangeRewardEraStartBn};
use crate::imps::point_exchange::PointExchange;
use crate::imps::PointManager;

#[test]
fn test_point_exchange() {
	new_test_ext().execute_with(|| {
		System::set_block_number(5);
		AtochaPot::on_initialize(5);
		assert_eq!(<PointExchange<Test>>::get_max_reward_list_size(), 3);
		assert_eq!(<PointExchange<Test>>::get_history_depth(), 3);
		assert_eq!(<PointExchange<Test>>::get_era_length(), 10);
		assert_eq!(<PointExchange<Test>>::get_current_era(), 1); // 1-5, 2-15, 3-25
		assert_eq!(<CurrentExchangeRewardEra<Test>>::get(), Some(1));
		assert_eq!(<ExchangeRewardEraStartBn<Test>>::get(1), Some(5));
		System::set_block_number(10);
		AtochaPot::on_initialize(10);
		assert_eq!(<PointExchange<Test>>::get_current_era(), 1);
		assert_eq!(<CurrentExchangeRewardEra<Test>>::get(), Some(1));
		assert_eq!(<ExchangeRewardEraStartBn<Test>>::get(1), Some(5));
		System::set_block_number(15);
		AtochaPot::on_initialize(15);
		assert_eq!(<PointExchange<Test>>::get_current_era(), 2);
		assert_eq!(<CurrentExchangeRewardEra<Test>>::get(), Some(2));
		assert_eq!(<ExchangeRewardEraStartBn<Test>>::get(2), Some(15));
		System::set_block_number(20);
		AtochaPot::on_initialize(20);
		assert_eq!(<PointExchange<Test>>::get_current_era(), 2);
		assert_eq!(<CurrentExchangeRewardEra<Test>>::get(), Some(2));
		assert_eq!(<ExchangeRewardEraStartBn<Test>>::get(2), Some(15));
		System::set_block_number(25);
		AtochaPot::on_initialize(25);
		assert_eq!(<PointExchange<Test>>::get_current_era(), 3);
		assert_eq!(<CurrentExchangeRewardEra<Test>>::get(), Some(3));
		assert_eq!(<ExchangeRewardEraStartBn<Test>>::get(3), Some(25));
		assert_eq!(<PointExchange<Test>>::get_reward_list(3), vec![]);

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
		assert_eq!(<PointManager<Test>>::get_issuance_points(), 2100);

		assert_eq!(Balances::free_balance(ACCOUNT_ID_1), 100_000_000_000_000);
		assert_eq!(Balances::free_balance(ACCOUNT_ID_2), 200_000_000_000_000);
		assert_eq!(Balances::free_balance(ACCOUNT_ID_3), 300_000_000_000_000);
		assert_eq!(Balances::free_balance(ACCOUNT_ID_4), 400_000_000_000_000);
		assert_eq!(Balances::free_balance(ACCOUNT_ID_5), 500_000_000_000_000);
		assert_eq!(Balances::free_balance(ACCOUNT_ID_6), 600_000_000_000_000);

		assert_ok!(<PointExchange<Test>>::apply_exchange(ACCOUNT_ID_4));
		// Vec<(T::AccountId, PointToken, Option<ExchangeInfo<PointToken, BalanceOf<T>, Perbill>>)>
		assert_eq!(<PointExchange<Test>>::get_reward_list(3), vec![
			(ACCOUNT_ID_4, 400, None),
		]);

		assert_ok!(<PointExchange<Test>>::apply_exchange(ACCOUNT_ID_5));
		assert_eq!(<PointExchange<Test>>::get_reward_list(3), vec![
			(ACCOUNT_ID_5, 500, None),
			(ACCOUNT_ID_4, 400, None),
		]);

		assert_ok!(<PointExchange<Test>>::apply_exchange(ACCOUNT_ID_3));
		assert_eq!(<PointExchange<Test>>::get_reward_list(3), vec![
			(ACCOUNT_ID_5, 500, None),
			(ACCOUNT_ID_4, 400, None),
			(ACCOUNT_ID_3, 300, None),
		]);

		assert_ok!(<PointExchange<Test>>::apply_exchange(ACCOUNT_ID_6));
		assert_eq!(<PointExchange<Test>>::get_reward_list(3), vec![
			(ACCOUNT_ID_6, 600, None),
			(ACCOUNT_ID_5, 500, None),
			(ACCOUNT_ID_4, 400, None),
			(ACCOUNT_ID_3, 300, None),
		]);

		assert_noop!(<PointExchange<Test>>::apply_exchange(ACCOUNT_ID_3), Error::<Test>::ExchangeApplyAlreadyExists);
		assert_noop!(<PointExchange<Test>>::apply_exchange(ACCOUNT_ID_2), Error::<Test>::TooFewPoints);
		assert_noop!(<PointExchange<Test>>::apply_exchange(ACCOUNT_ID_1), Error::<Test>::TooFewPoints);
		assert_noop!(<PointExchange<Test>>::apply_exchange(ACCOUNT_ID_6), Error::<Test>::ExchangeApplyAlreadyExists);
		assert_noop!(<PointExchange<Test>>::apply_exchange(ACCOUNT_ID_5), Error::<Test>::ExchangeApplyAlreadyExists);
		assert_noop!(<PointExchange<Test>>::apply_exchange(ACCOUNT_ID_4), Error::<Test>::ExchangeApplyAlreadyExists);

		// 1_000_000_000_000_000
		assert_noop!(<PointExchange<Test>>::execute_exchange(3, 1_000_000_000_000_000), Error::<Test>::EraNotEnded);
		assert_noop!(<PointExchange<Test>>::execute_exchange(2, 1_000_000_000_000_000), Error::<Test>::ExchangeListIsEmpty);

		//
		System::set_block_number(35);
		AtochaPot::on_initialize(35);
		assert_eq!(<PointExchange<Test>>::get_current_era(), 4);
		// If old era not be clean, new apply will be deny.
		// assert_noop!(<PointExchange<Test>>::apply_exchange(ACCOUNT_ID_4), Error::<Test>::LastExchangeRewardClearing);

		// on_initialize will call execute_exchange(2
		// AtochaPot::on_initialize(30); // 10_000_000_000_000_000
		assert_noop!(<PointExchange<Test>>::execute_exchange(3, 1_000_000_000_000_000), Error::<Test>::ExchangeRewardEnded);

		assert_eq!(<PointManager<Test>>::get_total_points(&ACCOUNT_ID_1), 100);
		assert_eq!(<PointManager<Test>>::get_total_points(&ACCOUNT_ID_2), 200);
		assert_eq!(<PointManager<Test>>::get_total_points(&ACCOUNT_ID_3), 300);
		assert_eq!(<PointManager<Test>>::get_total_points(&ACCOUNT_ID_4), 0);
		assert_eq!(<PointManager<Test>>::get_total_points(&ACCOUNT_ID_5), 0);
		assert_eq!(<PointManager<Test>>::get_total_points(&ACCOUNT_ID_6), 0);
		assert_eq!(<PointManager<Test>>::get_issuance_points(), 600);

		assert_eq!(Balances::free_balance(ACCOUNT_ID_1), 100_000_000_000_000);
		assert_eq!(Balances::free_balance(ACCOUNT_ID_2), 200_000_000_000_000);
		assert_eq!(Balances::free_balance(ACCOUNT_ID_3), 300_000_000_000_000);
		assert_eq!(Balances::free_balance(ACCOUNT_ID_4), 400_000_000_000_000 + 266_666_667_000_000); // 0.266 * 1000000000000000 266,000,000,000,000
		assert_eq!(Balances::free_balance(ACCOUNT_ID_5), 500_000_000_000_000 + 333333333000000); // 0.333
		assert_eq!(Balances::free_balance(ACCOUNT_ID_6), 600_000_000_000_000 + 400000000000000); // 0.6

		assert_noop!(<PointExchange<Test>>::apply_exchange(ACCOUNT_ID_4), Error::<Test>::TooFewPoints);
		assert_noop!(<PointExchange<Test>>::apply_exchange(ACCOUNT_ID_5), Error::<Test>::TooFewPoints);
		assert_noop!(<PointExchange<Test>>::apply_exchange(ACCOUNT_ID_6), Error::<Test>::TooFewPoints);
		assert_ok!(<PointExchange<Test>>::apply_exchange(ACCOUNT_ID_1));
		assert_ok!(<PointExchange<Test>>::apply_exchange(ACCOUNT_ID_2));
		assert_ok!(<PointExchange<Test>>::apply_exchange(ACCOUNT_ID_3));
		assert_eq!(<PointExchange<Test>>::get_reward_list(4), vec![
			(ACCOUNT_ID_3, 300, None),
			(ACCOUNT_ID_2, 200, None),
			(ACCOUNT_ID_1, 100, None),
		]);

	});
}

