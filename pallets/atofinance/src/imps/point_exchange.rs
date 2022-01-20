#![cfg_attr(not(feature = "std"), no_std)]

use super::*;
use sp_runtime::traits::Scale;
pub struct PointExchange<T>(PhantomData<T>);

// IPointExchange
// <T::AccountId, BalanceOf<T>, PuzzleSubjectHash, T::BlockNumber, DispatchResult>
impl<T: Config> IPointExchange<T::AccountId, T::BlockNumber, ExchangeEra, PointToken, ExchangeInfo<PointToken, BalanceOf<T>, Perbill>> for PointExchange<T> {
	fn apply_exchange(who: T::AccountId) -> DispatchResult {
		todo!()
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
		Vec::new()
	}

	fn get_history_depth() -> u32 {
		3
	}
}
