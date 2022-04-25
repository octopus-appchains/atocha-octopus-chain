#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::weights::Weight;
use super::*;
use sp_runtime::traits::Scale;
pub struct PointExchange<T>(PhantomData<T>);

// IPointExchange
// <T::AccountId, BalanceOf<T>, PuzzleSubjectHash, T::BlockNumber, DispatchResult>
impl<T: Config> IPointExchange<T::AccountId, T::BlockNumber, ExchangeEra, PointToken, BalanceOf<T>, ExchangeInfo<PointToken, BalanceOf<T>, Perbill>, Weight, Perbill> for PointExchange<T> {
	fn apply_exchange(who: T::AccountId) -> DispatchResult {
		let current_era = Self::get_current_era();
		ensure!(current_era > Zero::zero(), Error::<T>::LastExchangeRewardClearing);

		if PointExchangeInfo::<T>::contains_key(current_era.saturating_sub(1)) {
			let previous_reward_era = LastExchangeRewardEra::<T>::get();
			ensure!(previous_reward_era.is_some(), Error::<T>::LastExchangeRewardClearing);
			ensure!(previous_reward_era.unwrap() == current_era.saturating_sub(1), Error::<T>::LastExchangeRewardClearing);
		}

		// Get use point value .
		let apply_point = PointManager::<T>::get_total_points(&who);
		ensure!(apply_point > 0, Error::<T>::TooFewPoints);
		// Update and sort list.
		ensure!(Self::update_apply_list_point(), Error::<T>::ExchangeRewardEnded);

		let mut apply_list = Self::get_reward_list(current_era);
		if apply_list.is_empty() {
			apply_list.push((who, apply_point, None));
			LastUpdateBlockInfoOfPointExchage::<T>::put(<frame_system::Pallet<T>>::block_number());
			PointExchangeInfo::<T>::insert(current_era, apply_list);
			return Ok(());
		}

		ensure!( !apply_list.iter().any(|x|&x.0 == &who), Error::<T>::ExchangeApplyAlreadyExists );
		let origin_list_length = apply_list.len();
		if (origin_list_length < Self::get_max_reward_list_size() as usize) {
			apply_list.push((who, apply_point, None));
		}else{
			if let Some((original_who, original_point, origin_info)) = apply_list.get(Self::get_max_reward_list_size() as usize - 1) {
				ensure!(&apply_point > original_point, Error::<T>::TooFewPoints);
				apply_list.push((who, apply_point, None));
			}
		}
		apply_list.sort_by(|(_, point_a, _),(_, point_b, _)|{
			point_b.cmp(point_a)
		});
		LastUpdateBlockInfoOfPointExchage::<T>::put(<frame_system::Pallet<T>>::block_number());
		PointExchangeInfo::<T>::insert(current_era, apply_list);
		// // Update old list point.
		// if let Some((original_who, original_point, origin_info)) = apply_list.pop() {
		// 	if (origin_list_length < Self::get_max_reward_list_size() as usize) {
		// 		apply_list.push((who, apply_point, None));
		// 		apply_list.push((original_who, original_point, origin_info));
		// 	}else{
		// 		ensure!(apply_point > original_point, Error::<T>::TooFewPoints);
		// 		apply_list.push((who, apply_point, None));
		// 	}
		// 	apply_list.sort_by(|(_, point_a, _),(_, point_b, _)|{
		// 		point_b.cmp(point_a)
		// 	});
		// 	LastUpdateBlockInfoOfPointExchage::<T>::put(<frame_system::Pallet<T>>::block_number());
		// 	PointExchangeInfo::<T>::insert(current_era, apply_list);
		// }
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
		let current_bn = <frame_system::Pallet<T>>::block_number();
		if let Some(last_update_bn) = LastUpdateBlockInfoOfPointExchage::<T>::get() {
			if last_update_bn == current_bn {
				return true;
			}
		}
		if !have_final_info {
			let new_apply_list:  Vec<(
				T::AccountId,
				PointToken,
				Option<ExchangeInfo<PointToken, BalanceOf<T>, Perbill>>
			)> = apply_list.into_iter().map(|(who, _, info_data)|{
				let new_point = PointManager::<T>::get_total_points(&who);
				(who, new_point, info_data)
			}).collect();
			LastUpdateBlockInfoOfPointExchage::<T>::put(current_bn);
			PointExchangeInfo::<T>::insert(&Self::get_current_era(), new_apply_list);
			return true;
		}
		false
	}

	fn execute_exchange(era: ExchangeEra, mint_balance: BalanceOf<T>, min_tax: Perbill) -> DispatchResult {
		ensure!(era < Self::get_current_era(), Error::<T>::EraNotEnded );

		// ensure!(PointExchangeInfo::<T>::contains_key(era), Error::<T>::ExchangeListIsEmpty);
		if !PointExchangeInfo::<T>::contains_key(era) {
			LastExchangeRewardEra::<T>::put(era);
			return DispatchResult::Err(Error::<T>::ExchangeListIsEmpty.into());
		}

		if let Some(last_exec_era) = LastExchangeRewardEra::<T>::get() {
			ensure!(last_exec_era < era, Error::<T>::ExchangeRewardEnded);
		}

		Self::update_apply_list_point();
		// count total point.
		let mut exchange_list = PointExchangeInfo::<T>::get(era);
		if exchange_list.len() == 0 {
			LastExchangeRewardEra::<T>::put(era);
			return DispatchResult::Err(Error::<T>::ExchangeListIsEmpty.into());
		}
		ensure!(!exchange_list.iter().any(|(_, _, info_data)|{info_data.is_some()}), Error::<T>::ExchangeRewardEnded);

		// split array
		// TODO:: Has a bug need a test for it!
		let mut lost_list = Vec::new();
		if exchange_list.len() > Self::get_max_reward_list_size() as usize {
			lost_list = exchange_list.split_off(Self::get_max_reward_list_size() as usize);
		}

		let mut total_point: PointToken = Zero::zero();
		for x in exchange_list.clone().into_iter() {
			total_point = total_point.saturating_add(x.1);
		}
		//
		let mut sum_proportion: Perbill = Perbill::from_percent(0);
		let mut all_pay: BalanceOf<T> = Zero::zero();
		let mut new_exchange_list = Vec::new();

		//
		let mint_tax = min_tax * mint_balance;
		let mint_balance = mint_balance.saturating_sub(mint_tax);

		for (idx, (who, apply_point, mut info_data)) in exchange_list.clone().into_iter().enumerate() {
			let mut current_proportion = Perbill::from_percent(0);;
			if idx == exchange_list.len().saturating_sub(1) {
				current_proportion = Perbill::from_percent(100) - sum_proportion ;
				let take_token = mint_balance - all_pay;
				info_data = Some(ExchangeInfo {
					proportion: current_proportion.clone(),
					pay_point: apply_point,
					take_token: take_token,
				});
				all_pay += take_token;
			} else {
				current_proportion = Perbill::from_rational(apply_point, total_point);
				let take_token = current_proportion * mint_balance;
				info_data = Some(ExchangeInfo {
					proportion: current_proportion.clone(),
					pay_point: apply_point,
					take_token: take_token,
				});
				all_pay += take_token;
			}
			new_exchange_list.push((who, apply_point, info_data.clone()));
			sum_proportion = sum_proportion + current_proportion;
		}

		if mint_balance != all_pay {
			log::warn!(
				"execute_exchange error on `KickAwaySickExchange` because: mint_balance != all_pay {:?} != {:?}",
				&mint_balance,
				&all_pay
			);
			return DispatchResult::Err(Error::<T>::KickAwaySickExchange.into());
		}
		// ensure!(mint_balance == all_pay, Error::<T>::KickAwaySickExchange);
		if sum_proportion != Perbill::from_percent(100) {
			log::warn!(
				"execute_exchange error on `KickAwaySickExchange` because: sum_proportion {:?} != 100% ",
				&sum_proportion
			);
			return DispatchResult::Err(Error::<T>::KickAwaySickExchange.into());
		}
		// ensure!(sum_proportion == Perbill::from_percent(100), Error::<T>::KickAwaySickExchange);
		let mut event_list = Vec::new();
		for (who, apply_point, info_data) in new_exchange_list.clone() {
			let info_data = info_data.unwrap();
			PointManager::<T>::reduce_points_to(&who, info_data.pay_point);
			T::RewardHandler::on_unbalanced(T::Currency::deposit_creating(&who, info_data.take_token));
			event_list.push((who.clone(), info_data.clone()));
		}

		// Append lost list.
		new_exchange_list.append(&mut lost_list);

		PointExchangeInfo::<T>::insert(era, new_exchange_list.clone());
		LastExchangeRewardEra::<T>::put(era);

		T::SlashHandler::on_unbalanced(T::Currency::issue(mint_tax));

		crate::Pallet::<T>::deposit_event(Event::PointsExchange{
			era: era,
			exchange_list: event_list,
		});
		Ok(())
	}

	fn get_current_era() -> ExchangeEra {
		// let current_bn = <frame_system::Pallet<T>>::block_number();
		// (current_bn / Self::get_era_length()).unique_saturated_into()
		if let Some(current_ear) = CurrentExchangeRewardEra::<T>::get() {
			return current_ear;
		}
		// Is era start
		Self::check_and_update_era(<frame_system::Pallet<T>>::block_number());
		1
	}

	fn check_and_update_era(current_bn: T::BlockNumber) -> Weight {
		let mut w_read: Weight= 1;
		let mut w_write: Weight = 0;
		let current_era = CurrentExchangeRewardEra::<T>::get();
		if current_era.is_none() {
			CurrentExchangeRewardEra::<T>::put(1);
			ExchangeRewardEraStartBn::<T>::insert(1, current_bn);
			w_write+=2;
		}else{
			let current_era = current_era.unwrap();
			// Get exchange era start bn.
			w_read+=1;

			if let Some(era_start_bn) = ExchangeRewardEraStartBn::<T>::get(current_era) {
				let diff_bn = current_bn.saturating_sub(era_start_bn);
				if diff_bn >= Self::get_era_length() {
					let new_era = current_era.saturating_add(1);
					CurrentExchangeRewardEra::<T>::put(new_era);
					ExchangeRewardEraStartBn::<T>::insert(new_era, current_bn);
					w_write+=2;
				}
			}else{
				ExchangeRewardEraStartBn::<T>::insert(current_era, current_bn);
			}
		}
		w_read + w_write
	}

	fn get_last_reward_era() -> ExchangeEra {
		if let Some(era) = LastExchangeRewardEra::<T>::get() {
			return era;
		}
		0
	}

	fn get_max_reward_list_size() -> u32 {
		// T::ExchangeMaxRewardListSize::get() // 3
		let ato_config = Pallet::<T>::get_ato_config();
		ato_config.exchange_max_reward_list_size
	}

	fn get_era_length() -> T::BlockNumber {
		// T::ExchangeHistoryDepth::get(); // 3
		// T::ExchangeMaxRewardListSize::get(); // 3
		// T::ExchangeEraLength::get() // 10
		let ato_config = Pallet::<T>::get_ato_config();
		ato_config.exchange_era_block_length
	}

	fn get_reward_list(era: ExchangeEra) -> Vec<(T::AccountId, PointToken, Option<ExchangeInfo<PointToken, BalanceOf<T>, Perbill>>)> {
		PointExchangeInfo::<T>::get(&Self::get_current_era())
		// get max_reward_count .
		// let mut apply_list = PointExchangeInfo::<T>::get(&Self::get_current_era());
		// if apply_list.len() <= Self::get_max_reward_list_size() as usize {
		// 	return apply_list;
		// }
		// apply_list.split_off(Self::get_max_reward_list_size() as usize);
		// apply_list
	}

	fn get_history_depth() -> u32 {
		// T::ExchangeHistoryDepth::get() // 3
		let ato_config = Pallet::<T>::get_ato_config();
		ato_config.exchange_history_depth
	}
}
