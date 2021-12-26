#![cfg_attr(not(feature = "std"), no_std)]

use super::*;

pub struct ChallengeManager<T>(PhantomData<T>);

pub trait Config: super::Config {
	type ChallengeThreshold: Get<Perbill>;
	type RaisingPeriodLength: Get<<Self as frame_system::Config>::BlockNumber>;
}

impl<T: Config>
	IAtoChallenge<
		T::AccountId,
		PuzzleSubjectHash,
		BalanceOf<T>,
		PuzzleChallengeData<T::AccountId, T::BlockNumber, BalanceOf<T>>,
		ChallengeStatus<T::BlockNumber>,
		Error<T>,
	> for ChallengeManager<T>
{
	fn issue_challenge(
		who: T::AccountId,
		pid: &PuzzleSubjectHash,
		deposit: BalanceOf<T>,
	) -> DispatchResult {
		let storage_ledger = <AtoFinanceLedger<T>>::try_get(pid).ok();
		ensure!(storage_ledger.is_some(), Error::<T>::PuzzleNotExists);
		ensure!(!<PuzzleChallengeInfo<T>>::contains_key(pid), Error::<T>::ChallengeAlreadyExists);
		// Get threshold.
		let threshold_balance = Self::get_balance_threshold(pid);
		let real_deposit = deposit.min(threshold_balance);

		// Create challenge data.
		T::Currency::transfer(
			&who,
			&crate::Pallet::<T>::account_id(),
			real_deposit,
			ExistenceRequirement::KeepAlive,
		)?;

		let current_block_number = <frame_system::Pallet<T>>::block_number();

		let mut raise_group: Vec<(T::AccountId, BalanceOf<T>)> = Vec::new();
		raise_group.push((who.clone(), real_deposit));

		let challenge_status = || {
			if real_deposit >= threshold_balance {
				return ChallengeStatus::RaiseCompleted(current_block_number);
			}
			ChallengeStatus::Raise(current_block_number)
		};

		// Create challenge data
		let challenge_data = PuzzleChallengeData {
			raised_total: real_deposit,
			status: challenge_status(),
			create_bn: current_block_number,
			creator: who,
			start_bn: None,
			end_bn: None,
			raised_group: raise_group,
		};
		<PuzzleChallengeInfo<T>>::insert(pid, challenge_data);
		Ok(())
	}

	fn get_balance_threshold(pid: &PuzzleSubjectHash) -> BalanceOf<T> {
		let storage_ledger = <AtoFinanceLedger<T>>::try_get(pid).ok();
		if storage_ledger.is_none() {
			return Zero::zero();
		}
		let storage_ledger = storage_ledger.unwrap();
		let storage_total_balance = storage_ledger.total;
		T::ChallengeThreshold::get() * storage_total_balance
	}

	fn get_total_raise(pid: &PuzzleSubjectHash) -> BalanceOf<T> {
		let challenge_info = <PuzzleChallengeInfo<T>>::get(&pid);
		challenge_info.raised_total
	}

	fn challenge_crowdloan(
		who: T::AccountId,
		pid: &PuzzleSubjectHash,
		deposit: BalanceOf<T>,
	) -> DispatchResult {
		let mut challenge_data = Self::check_get_challenge_info(pid)?;
		let threshold_balance = Self::get_balance_threshold(pid);
		let remaining_funds = threshold_balance.saturating_sub(challenge_data.raised_total);
		ensure!(remaining_funds > Zero::zero(), Error::<T>::EndOfRaising);
		let deposit = deposit.min(remaining_funds);
		T::Currency::transfer(
			&who,
			&crate::Pallet::<T>::account_id(),
			deposit,
			ExistenceRequirement::KeepAlive,
		)?;
		let current_block_number = <frame_system::Pallet<T>>::block_number();
		let raised_total = challenge_data.raised_total.saturating_add(deposit);
		let challenge_status = || {
			if raised_total >= threshold_balance {
				return ChallengeStatus::RaiseCompleted(current_block_number);
			}
			ChallengeStatus::Raise(current_block_number)
		};

		challenge_data.status = challenge_status();
		challenge_data.raised_total = raised_total;
		challenge_data.raised_group.push((who.clone(), deposit));
		<PuzzleChallengeInfo<T>>::insert(pid, challenge_data);
		Ok(())
	}

	fn get_challenge_status(pid: &PuzzleSubjectHash) -> Option<ChallengeStatus<T::BlockNumber>> {
		if !<PuzzleChallengeInfo<T>>::contains_key(&pid) {
			return None;
		}
		let challenge_info = <PuzzleChallengeInfo<T>>::get(&pid);
		Some(challenge_info.status)
	}

	fn check_get_challenge_info(
		pid: &PuzzleSubjectHash,
	) -> Result<PuzzleChallengeData<T::AccountId, T::BlockNumber, BalanceOf<T>>, Error<T>> {
		if !<PuzzleChallengeInfo<T>>::contains_key(&pid) {
			return Err(Error::<T>::ChallengeNotExists);
		}
		let challenge_info = <PuzzleChallengeInfo<T>>::get(&pid);
		let period_len = T::RaisingPeriodLength::get();
		let current_block_number = <frame_system::Pallet<T>>::block_number();
		if current_block_number > challenge_info.create_bn.saturating_add(period_len) {
			return Err(Error::<T>::RaisingPeriodExpired);
		}
		Ok(challenge_info)
	}

	fn has_the_raising_period_expired(pid: &PuzzleSubjectHash) -> bool {
		if !<PuzzleChallengeInfo<T>>::contains_key(&pid) {
			return true;
		}
		let period_len = T::RaisingPeriodLength::get();
		let current_block_number = <frame_system::Pallet<T>>::block_number();
		let challenge_info = <PuzzleChallengeInfo<T>>::get(&pid);
		current_block_number > challenge_info.create_bn.saturating_add(period_len)
	}

	fn back_challenge_crowdloan(pid: &PuzzleSubjectHash, tax: Perbill) -> bool {
		todo!()
	}
}
