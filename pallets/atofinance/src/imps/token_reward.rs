#![cfg_attr(not(feature = "std"), no_std)]

use super::*;

pub struct TokenReward<T>(PhantomData<T>);
impl<T: Config> IPuzzleReward<T::AccountId, BalanceOf<T>, PuzzleSubjectHash, T::BlockNumber, DispatchResult>
	for TokenReward<T>
{
	type PerVal = Perbill;
	type Imbalance = NegativeImbalanceOf<T>;
	type OnBurn = T::SlashHandler;
	// type OnBurn = T::RewardHandler;
	// type FundPool = T::AccountId;

	fn get_total_bonus(pid: &PuzzleSubjectHash, _cut_bn: T::BlockNumber) -> Option<BalanceOf<T>> {
		let storage_ledger = <AtoFinanceLedger<T>>::try_get(pid).ok();
		if storage_ledger.is_none() {
			return None;
		}
		let pot_ledger = storage_ledger.unwrap();
		Some(pot_ledger.total)
	}

	fn answer_get_reward(
		pid: &PuzzleSubjectHash,
		beneficiary: T::AccountId,
		_cut_bn: T::BlockNumber,
		tax: Self::PerVal,
	) -> DispatchResult {
		//
		let pot_reward = <AtoFinanceReward<T>>::try_get(pid).ok();
		ensure!(pot_reward.is_none(), Error::<T>::RewardHasBeenClaimed);

		let storage_ledger = <AtoFinanceLedger<T>>::try_get(pid).ok();
		ensure!(storage_ledger.is_some(), Error::<T>::PuzzleNotExists);
		let storage_ledger = storage_ledger.unwrap();

		// Check beneficiary is creator
		let mut reward_type = RewardType::AnswerReward;
		if &storage_ledger.owner == &beneficiary {
			reward_type = RewardType::CreatorReward;
		}

		let total_balance = storage_ledger.total;

		let tax_fee = tax * total_balance;
		let payout = total_balance - tax_fee;
		T::Currency::transfer(
			&crate::Pallet::<T>::account_id(),
			&beneficiary,
			payout,
			ExistenceRequirement::KeepAlive,
		)?;

		let negative_imbalance = T::Currency::slash(&crate::Pallet::<T>::account_id(), tax_fee);
		Self::OnBurn::on_unbalanced(negative_imbalance.0);

		let mut beneficiaries = Vec::new();
		beneficiaries.push((beneficiary, Perbill::from_percent(100)));

		let pot_reward = PotRewardData {
			create_bn: <frame_system::Pallet<T>>::block_number(),
			tax,
			reward_type,
			total: total_balance,
			payout,
			beneficiaries,
		};
		<AtoFinanceReward<T>>::insert(pid, pot_reward);
		Ok(())
	}

	fn challenge_get_reward(
		pid: &PuzzleSubjectHash,
		beneficiaries: Vec<(T::AccountId, Self::PerVal)>,
		_cut_bn: T::BlockNumber,
		tax: Self::PerVal,
	) -> DispatchResult {
		let pot_reward = <AtoFinanceReward<T>>::try_get(pid).ok();
		ensure!(pot_reward.is_none(), Error::<T>::RewardHasBeenClaimed);

		let storage_ledger = <AtoFinanceLedger<T>>::try_get(pid).ok();
		ensure!(storage_ledger.is_some(), Error::<T>::PuzzleNotExists);
		let storage_ledger = storage_ledger.unwrap();

		// Check proportion of verification beneficiaries paid.
		// The beneficiary list cannot be empty.
		let beneficiaries_len = beneficiaries.len() as u32;
		ensure!(beneficiaries_len > 0, Error::<T>::BeneficiaryListNotEmpty);

		if beneficiaries_len == 1 {
			ensure!(
				beneficiaries[0].1 == Self::PerVal::from_percent(100),
				Error::<T>::WrongPaymentRatio
			);
		} else {
			let mut sum_proportion = Self::PerVal::zero();
			ensure!(
				beneficiaries.clone().into_iter().enumerate().any(|(idx, (_, pay_proportion))| {
					// When lenght = 3
					// 0 , (3-1) > sum
					// 1,  (3-1) > sum
					// 2, = (3-1) do check
					if idx as u32 == beneficiaries.len() as u32 - 1u32
						&& Self::PerVal::from_percent(100).saturating_sub(sum_proportion)
							== pay_proportion
					{
						return true;
					} else {
						sum_proportion = sum_proportion.saturating_add(pay_proportion);
					}
					false
				}),
				Error::<T>::WrongPaymentRatio
			);
		}

		let total_balance = storage_ledger.total;

		println!(" total_balance = {:?}", total_balance);

		// Check liquidity.
		let (pot_account, free_balance) = crate::Pallet::<T>::pot();
		ensure!(free_balance >= total_balance, Error::<T>::InsufficientBalance);

		let tax_fee = tax * total_balance;
		let payout = total_balance - tax_fee;
		// let paid: BalanceOf<T> = Zero::zero();

		let mut storage_beneficiaries = Vec::new();
		for (beneficiary, pay_proportion) in beneficiaries.clone().into_iter() {
			println!(
				"RUN Transfer : {:?}, proportion : {:?} pay = {:?} ",
				&beneficiary,
				&pay_proportion,
				pay_proportion * payout,
			);
			T::Currency::transfer(
				&pot_account,
				&beneficiary,
				pay_proportion * payout,
				ExistenceRequirement::KeepAlive,
			)?;
			storage_beneficiaries.push((beneficiary, pay_proportion));
		}
		// Burn fee.
		let negative_imbalance = T::Currency::slash(&pot_account, tax_fee);
		Self::OnBurn::on_unbalanced(negative_imbalance.0);

		let pot_reward = PotRewardData {
			create_bn: <frame_system::Pallet<T>>::block_number(),
			tax,
			reward_type: RewardType::ChallengerReward,
			total: total_balance,
			payout,
			beneficiaries: storage_beneficiaries,
		};
		<AtoFinanceReward<T>>::insert(pid, pot_reward);
		Ok(())
	}
}
