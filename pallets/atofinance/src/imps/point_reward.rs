#![cfg_attr(not(feature = "std"), no_std)]

use super::point_manager::PointManager;
use super::*;
use frame_support::traits::TryDrop;

pub struct PointReward<T>(PhantomData<T>);
impl<T: Config> IPuzzleReward<T::AccountId, PointToken, PuzzleSubjectHash, T::BlockNumber, DispatchResult>
	for PointReward<T>
{
	type PerVal = Perbill;
	type Imbalance = NoneImbalance;
	type OnBurn = ();

	fn get_total_bonus(pid: &PuzzleSubjectHash, cut_bn: T::BlockNumber) -> Option<PointToken> {
		// Get current block number.
		let ato_config = Pallet::<T>::get_ato_config();
		Some(<PointManager<T>>::calculate_points_of_puzzle(
			cut_bn,
			pid,
			// T::PerEraOfBlockNumber::get(),
			ato_config.point_reward_epoch_block_length
		))
	}

	fn answer_get_reward(
		pid: &PuzzleSubjectHash,
		beneficiary: T::AccountId,
		cut_bn: T::BlockNumber,
		tax: Self::PerVal,
	) -> DispatchResult {
		let pot_reward = <AtoPointReward<T>>::try_get(pid).ok();
		ensure!(pot_reward.is_none(), Error::<T>::RewardHasBeenClaimed);

		let storage_ledger = <AtoFinanceLedger<T>>::try_get(pid).ok();
		ensure!(storage_ledger.is_some(), Error::<T>::PuzzleNotExists);
		let storage_ledger = storage_ledger.unwrap();

		// Check beneficiary is creator
		let mut reward_type = RewardType::AnswerReward;
		if &storage_ledger.owner == &beneficiary {
			reward_type = RewardType::CreatorReward;
		}

		let mut beneficiaries = Vec::new();
		match reward_type {
			RewardType::NoneReward => {}
			RewardType::AnswerReward => {
				beneficiaries.push((storage_ledger.owner.clone(), Perbill::from_percent(50)));
				beneficiaries.push((beneficiary.clone(), Perbill::from_percent(50)));
			}
			RewardType::CreatorReward => {
				beneficiaries.push((beneficiary.clone(), Perbill::from_percent(100)));
			}
			RewardType::ChallengerReward => {}
		}

		ensure!(beneficiaries.len() > Zero::zero(), Error::<T>::BeneficiaryNotFound);

		let total_point = Self::get_total_bonus(pid, cut_bn);
		ensure!(total_point.is_some(), Error::<T>::NotPointToken);
		let total_point = total_point.unwrap();

		let tax_fee: PointToken = tax * total_point;
		let payout = total_point - tax_fee;

		let pot_reward = PotRewardData {
			create_bn: <frame_system::Pallet<T>>::block_number(),
			tax,
			reward_type,
			total: total_point,
			payout,
			beneficiaries: beneficiaries.clone(),
		};
		<AtoPointReward<T>>::insert(pid, pot_reward);
		super::Pallet::<T>::deposit_event(Event::<T>::TakePointReward {
			pid: pid.clone(),
			payout: payout.clone(),
			fee: tax_fee.clone(),
		});

		// Go to distributing rewards
		// ------

		for (user, per_bill) in beneficiaries {
			ensure!(
				<PointManager<T>>::increase_points_to(&user, per_bill * payout ).is_ok(),
				Error::<T>::PointTokenIncreaseFailure
			);
		}
		// ------

		Ok(())
	}

	fn challenge_get_reward(
		pid: &PuzzleSubjectHash,
		beneficiaries: Vec<(T::AccountId, Self::PerVal)>,
		cut_bn: T::BlockNumber,
		tax: Self::PerVal,
	) -> DispatchResult {
		// let pot_reward = <AtoPointReward<T>>::try_get(pid).ok();
		// ensure!(pot_reward.is_none(), Error::<T>::RewardHasBeenClaimed);
		//
		// let storage_ledger = <AtoFinanceLedger<T>>::try_get(pid).ok();
		// ensure!(storage_ledger.is_some(), Error::<T>::PuzzleNotExists);
		// let storage_ledger = storage_ledger.unwrap();
		//
		// // Check proportion of verification beneficiaries paid.
		// // The beneficiary list cannot be empty.
		// let beneficiaries_len = beneficiaries.len() as u32;
		// ensure!(beneficiaries_len > 0, Error::<T>::BeneficiaryListNotEmpty);
		//
		// if beneficiaries_len == 1 {
		// 	ensure!(
		// 		beneficiaries[0].1 == Self::PerVal::from_percent(100),
		// 		Error::<T>::WrongPaymentRatio
		// 	);
		// } else {
		// 	let mut sum_proportion = Self::PerVal::zero();
		// 	ensure!(
		// 		beneficiaries.clone().into_iter().enumerate().any(|(idx, (_, pay_proportion))| {
		// 			// When lenght = 3
		// 			// 0 , (3-1) > sum
		// 			// 1,  (3-1) > sum
		// 			// 2, = (3-1) do check
		// 			if idx as u32 == beneficiaries.len() as u32 - 1u32
		// 				&& Self::PerVal::from_percent(100).saturating_sub(sum_proportion)
		// 					== pay_proportion
		// 			{
		// 				return true;
		// 			} else {
		// 				sum_proportion = sum_proportion.saturating_add(pay_proportion);
		// 			}
		// 			false
		// 		}),
		// 		Error::<T>::WrongPaymentRatio
		// 	);
		// }

		// let total_point = Self::get_total_bonus(pid, cut_bn);
		// ensure!(total_point.is_some(), Error::<T>::NotPointToken);
		// let total_point = total_point.unwrap();
		//
		// let tax_fee = tax * total_point;
		// let payout = total_point - tax_fee;
		// // let paid: BalanceOf<T> = Zero::zero();
		//
		// let mut storage_beneficiaries = Vec::new();
		// for (beneficiary, pay_proportion) in beneficiaries.clone().into_iter() {
		// 	ensure!(
		// 		<PointManager<T>>::increase_points_to(&beneficiary, pay_proportion * payout)
		// 			.is_ok(),
		// 		Error::<T>::PointTokenIncreaseFailure
		// 	);
		// 	storage_beneficiaries.push((beneficiary, pay_proportion));
		// }

		// let pot_reward = PotRewardData {
		// 	create_bn: <frame_system::Pallet<T>>::block_number(),
		// 	tax,
		// 	reward_type: RewardType::ChallengerReward,
		// 	total: total_point,
		// 	payout,
		// 	beneficiaries: storage_beneficiaries,
		// };
		// <AtoPointReward<T>>::insert(pid, pot_reward);
		Ok(())
	}
}

pub struct NoneImbalance;
impl TryDrop for NoneImbalance {
	fn try_drop(self) -> Result<(), Self> {
		Ok(())
	}
}
