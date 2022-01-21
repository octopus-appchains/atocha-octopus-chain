#![cfg_attr(not(feature = "std"), no_std)]

use super::*;
use sp_runtime::traits::Scale;
pub struct PointExchange<T>(PhantomData<T>);

// IPointExchange
// <T::AccountId, BalanceOf<T>, PuzzleSubjectHash, T::BlockNumber, DispatchResult>
impl<T: Config> IPointExchange<T::AccountId, T::BlockNumber, ExchangeEra, PointToken, BalanceOf<T>, ExchangeInfo<PointToken, BalanceOf<T>, Perbill>> for PointExchange<T> {
	fn apply_exchange(who: T::AccountId) -> DispatchResult {
		// Get use point value .
		let apply_point = PointManager::<T>::get_total_points(&who);
		ensure!(apply_point > 0, Error::<T>::TooFewPoints);

		let current_era = Self::get_current_era();
		let mut apply_list = PointExchangeInfo::<T>::get(&Self::get_current_era());
		if apply_list.is_empty() {
			apply_list.push((who, apply_point, None));
			PointExchangeInfo::<T>::insert(current_era, apply_list);
			return OK(());
		}

		// Update old list point.
		Self::update_apply_list_point();

		apply_list.binary_search_by(|old_data| old_data.1.cmp(apply_point))
		Ok(())
	}

	fn update_apply_list_point() {
		todo!()
	}

	fn execute_exchange(era: ExchangeEra, mint_balance: BalanceOf<T>) -> DispatchResult {
		ensure!(era < Self::get_current_era(), Error::<T>::EraNotEnded );
		Ok(())
	}

	fn get_current_era() -> ExchangeEra {
		let current_bn = <frame_system::Pallet<T>>::block_number();
		(current_bn / Self::get_era_length()).unique_saturated_into()
	}

	fn get_max_reward_count() -> u32 {
		3
	}

	fn get_era_length() -> T::BlockNumber {
		10u32.into()
	}

	fn get_reward_list(era: ExchangeEra) -> Vec<(T::AccountId, PointToken, Option<ExchangeInfo<PointToken, BalanceOf<T>, Perbill>>)> {
		// get max_reward_count .
		let mut apply_list = PointExchangeInfo::<T>::get(&Self::get_current_era());
		if apply_list.len() <= Self::get_max_reward_count() as usize {
			return apply_list;
		}
		apply_list.split_off(Self::get_max_reward_count() as usize);
		apply_list
	}

	fn get_history_depth() -> u32 {
		3
	}
}
