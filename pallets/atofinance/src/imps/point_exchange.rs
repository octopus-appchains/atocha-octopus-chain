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

		// Update and sort list.
		ensure!(Self::update_apply_list_point(), Error::<T>::ExchangeRewardEnded);

		// let mut apply_list = PointExchangeInfo::<T>::get(&Self::get_current_era());
		let current_era = Self::get_current_era();
		let mut apply_list = Self::get_reward_list(current_era);
		if apply_list.is_empty() {
			apply_list.push((who, apply_point, None));
			LastUpdateBlockInfoOfPointExchage::<T>::put(<frame_system::Pallet<T>>::block_number());
			PointExchangeInfo::<T>::insert(current_era, apply_list);
			return Ok(());
		}

		ensure!( !apply_list.iter().any(|x|&x.0 == &who), Error::<T>::ExchangeApplyAlreadyExists );

		// Update old list point.
		if let Some((original_who, original_point, origin_info)) = apply_list.pop() {
			ensure!(apply_point > original_point, Error::<T>::TooFewPoints);
			apply_list.push((who, apply_point, None));
			if (apply_list.len() < Self::get_max_reward_count() as usize) {
				apply_list.push((original_who, original_point, origin_info));
			}
			println!("RUN2 : {:?}", &apply_list);
			apply_list.sort_by(|(_, point_a, _),(_, point_b, _)|{
				point_b.cmp(point_a)
			});
			println!("RUN3 : {:?}", &apply_list);
			LastUpdateBlockInfoOfPointExchage::<T>::put(<frame_system::Pallet<T>>::block_number());
			PointExchangeInfo::<T>::insert(current_era, apply_list);
		}

		Ok(())
	}

	fn update_apply_list_point() -> bool  {
		let mut apply_list = PointExchangeInfo::<T>::get(&Self::get_current_era());
		let have_final_info = apply_list.iter().any(|(_, _, info_data)|{
			if info_data.is_some() {
				return true;
			}
			false
		});
		if !have_final_info {
			let new_apply_list:  Vec<(
				T::AccountId,
				PointToken,
				Option<ExchangeInfo<PointToken, BalanceOf<T>, Perbill>>
			)> = apply_list.into_iter().map(|(who, _, info_data)|{
				let new_point = PointManager::<T>::get_total_points(&who);
				(who, new_point, info_data)
			}).collect();
			PointExchangeInfo::<T>::insert(&Self::get_current_era(), new_apply_list);
			return true;
		}
		false
	}

	fn execute_exchange(era: ExchangeEra, mint_balance: BalanceOf<T>) -> DispatchResult {
		ensure!(era < Self::get_current_era(), Error::<T>::EraNotEnded );
		ensure!(PointExchangeInfo::<T>::contains_key(era), Error::<T>::ExchangeListIsEmpty);

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
