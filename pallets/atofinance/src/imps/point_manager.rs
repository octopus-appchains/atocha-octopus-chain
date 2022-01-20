#![cfg_attr(not(feature = "std"), no_std)]

use super::*;
use sp_runtime::traits::Scale;
pub struct PointManager<T>(PhantomData<T>);

impl<T: Config>
	IPuzzlePoints<T::AccountId, PointToken, T::BlockNumber, PuzzleSubjectHash, DispatchResult>
	for PointManager<T>
{
	fn get_total_points(who: &T::AccountId) -> PointToken {
		<AtoPointLedger<T>>::get(who)
	}

	fn increase_points_to(who: &T::AccountId, pt: PointToken) -> DispatchResult {
		let old_points = Self::get_total_points(who);
		<AtoPointLedger<T>>::insert(who, old_points.saturating_add(pt));
		<AtoPointTotal<T>>::put(Self::get_issuance_points().saturating_add(pt));
		Ok(())
	}

	fn reduce_points_to(who: &T::AccountId, pt: PointToken) -> DispatchResult {
		let old_points = Self::get_total_points(who);
		ensure!(&old_points >= &pt, Error::<T>::InsufficientPoint);
		<AtoPointLedger<T>>::insert(who, old_points.saturating_sub(pt));
		<AtoPointTotal<T>>::put(Self::get_issuance_points().saturating_sub(pt));
		Ok(())
	}

	fn get_issuance_points() -> PointToken {
		<AtoPointTotal<T>>::get().unwrap_or(Zero::zero())
	}

	fn calculate_points_of_puzzle(
		current_bn: T::BlockNumber,
		pid: &PuzzleSubjectHash,
		per_bn: T::BlockNumber,
	) -> PointToken {
		let storage_ledger = <AtoFinanceLedger<T>>::try_get(pid).ok();
		// ensure!(storage_ledger.is_some(), Error::<T>::PuzzleNotExists);
		if storage_ledger.is_none() {
			return 0;
		}
		let storage_ledger = storage_ledger.unwrap();
		let point_list: Vec<PointToken> = storage_ledger
			.sponsor_list
			.into_iter()
			.map(|x| {
				let decimal = 1000u32;
				let duration_num: T::BlockNumber = (current_bn.saturating_sub(x.create_bn))
					.saturating_mul(decimal.into())
					/ per_bn;
				// println!(" x.create_bn = {:?}, current_bn = {:?},  duration_num = {:?}", &x.create_bn, &current_bn, &duration_num);
				let duration_num: PointToken = duration_num.unique_saturated_into();
				let sponsor_funds: PointToken = x.funds.unique_saturated_into();
				let base_point: PointToken =
					duration_num.saturating_mul(sponsor_funds) / (decimal as PointToken);
				// let fraction_mut_num = 10u128.checked_pow(fraction.into()).unwrap();
				// base_point.saturating_mul(fraction_mut_num.unique_saturated_into())
				base_point
			})
			.collect();
		point_list.iter().sum()
	}
}
