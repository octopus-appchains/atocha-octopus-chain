#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::sp_runtime::traits::StaticLookup;
use super::*;

pub struct ChallengeManager<T>(PhantomData<T>);

pub trait Config: super::Config + pallet_balances::Config  {
	// type ChallengeThreshold: Get<Perbill>;
	// type RaisingPeriodLength: Get<<Self as frame_system::Config>::BlockNumber>;
}

// impl<T: Config> ChallengeManager<T> {
// 	fn xxx(acc: <T::Lookup as StaticLookup>::Source, amount: T::Balance) -> T::Proposal{
// 		// Call::System(frame_system::Call::remark { remark: value.encode() })
// 		// Call::Balances(pallet_balances::Call::transfer{dest: acc, value: amount});
// 		pallet_balances::Call::<T>::transfer{dest: acc, value: amount}
// 	}
// }

impl<T: Config>
	IAtoChallenge<
		T::AccountId,
		PuzzleSubjectHash,
		BalanceOf<T>,
		PuzzleChallengeData<T::AccountId, T::BlockNumber, BalanceOf<T>, Perbill>,
		ChallengeStatus<T::BlockNumber, Perbill>,
		Error<T>,
		T::BlockNumber,
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

		crate::Pallet::<T>::deposit_event(Event::ChallengeDeposit{
			pid: pid.clone(),
			who: who.clone(),
			deposit: real_deposit.clone(),
			deposit_type: ChallengeDepositType::Issue,
		});

		let current_block_number = <frame_system::Pallet<T>>::block_number();

		let mut raise_group: Vec<(T::AccountId, BalanceOf<T>)> = Vec::new();
		raise_group.push((who.clone(), real_deposit));

		let mut start_bn = None;
		let challenge_status = || {
			if real_deposit >= threshold_balance {
				// start_bn = Some(current_block_number);
				return ChallengeStatus::RaiseCompleted(current_block_number);
			}
			ChallengeStatus::Raise(current_block_number)
		};

		let current_status = challenge_status();
		if let ChallengeStatus::RaiseCompleted(x) = current_status.clone() {
			start_bn = Some(x);
		}

		// Create challenge data
		let challenge_data = PuzzleChallengeData {
			raised_total: real_deposit,
			status: current_status,
			create_bn: current_block_number,
			creator: who,
			start_bn: start_bn,
			end_bn: None,
			raised_group: raise_group,
		};

		<PuzzleChallengeInfo<T>>::insert(pid, challenge_data.clone());

		match challenge_data.status {
			ChallengeStatus::RaiseCompleted(_x) => {
				crate::Pallet::<T>::deposit_event(Event::ChallengeStatusChange {
					challenge_status: challenge_data.status.clone(),
					pid: pid.clone(),
				});
				T::AtoPropose::challenge_propose(pid.clone());
			},
			_ => {}
		}

		Ok(())
	}

	fn get_balance_threshold(pid: &PuzzleSubjectHash) -> BalanceOf<T> {
		let storage_ledger = <AtoFinanceLedger<T>>::try_get(pid).ok();
		if storage_ledger.is_none() {
			return Zero::zero();
		}
		let storage_ledger = storage_ledger.unwrap();
		let storage_total_balance = storage_ledger.total;
		let ato_config = Pallet::<T>::get_ato_config();
		// T::ChallengeThreshold::get() * storage_total_balance
		ato_config.challenge_threshold * storage_total_balance
	}

	fn get_total_raise(pid: &PuzzleSubjectHash) -> BalanceOf<T> {
		let challenge_info = <PuzzleChallengeInfo<T>>::get(&pid);
		if challenge_info.is_none() {
			return Zero::zero();
		}
		challenge_info.unwrap().raised_total
	}

	fn challenge_crowdloan(
		who: T::AccountId,
		pid: &PuzzleSubjectHash,
		deposit: BalanceOf<T>,
	) -> DispatchResult {
		let mut challenge_data = Self::check_get_active_challenge_info(pid)?;
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

		crate::Pallet::<T>::deposit_event(Event::ChallengeDeposit{
			pid: pid.clone(),
			who: who.clone(),
			deposit: deposit.clone(),
			deposit_type: ChallengeDepositType::Crowdloan,
		});

		let current_block_number = <frame_system::Pallet<T>>::block_number();
		let raised_total = challenge_data.raised_total.saturating_add(deposit);
		let challenge_status = || {
			if raised_total >= threshold_balance {
				return ChallengeStatus::RaiseCompleted(current_block_number);
			}
			ChallengeStatus::Raise(current_block_number)
		};


		let current_status = challenge_status();
		let mut start_bn = None;
		if let ChallengeStatus::RaiseCompleted(x) = current_status.clone() {
			start_bn = Some(x);
		}

		challenge_data.start_bn = start_bn;
		challenge_data.status = current_status;
		challenge_data.raised_total = raised_total;
		challenge_data.raised_group.push((who.clone(), deposit));
		<PuzzleChallengeInfo<T>>::insert(pid, challenge_data.clone());

		match challenge_data.status {
			ChallengeStatus::RaiseCompleted(_x) => {
				crate::Pallet::<T>::deposit_event(Event::ChallengeStatusChange{
					challenge_status: challenge_data.status.clone(),
					pid: pid.clone(),
				});
				T::AtoPropose::challenge_propose(pid.clone());
			},
			_ => {}
		}

		Ok(())
	}

	fn has_the_raising_period_expired(pid: &PuzzleSubjectHash) -> bool {
		// if !<PuzzleChallengeInfo<T>>::contains_key(&pid) {
		// 	return true;
		// }
		let challenge_info = <PuzzleChallengeInfo<T>>::get(&pid);
		if challenge_info.is_none() {
			return true;
		}
		let challenge_info = challenge_info.unwrap();

		let ato_config = Pallet::<T>::get_ato_config();

		let period_len = ato_config.raising_period_length;
		let current_block_number = <frame_system::Pallet<T>>::block_number();

		current_block_number > challenge_info.create_bn.saturating_add(period_len)
	}

	fn get_challenge_status(pid: &PuzzleSubjectHash) -> Option<ChallengeStatus<T::BlockNumber, Perbill>> {
		if !<PuzzleChallengeInfo<T>>::contains_key(&pid) {
			return None;
		}
		let challenge_info = <PuzzleChallengeInfo<T>>::get(&pid);
		Some(challenge_info.unwrap().status)
	}

	fn recognition_challenge(pid: &PuzzleSubjectHash) -> DispatchResult {
		Self::back_challenge_crowdloan(pid, Perbill::from_percent(0))
	}

	fn back_challenge_crowdloan(pid: &PuzzleSubjectHash, tax: Perbill) -> DispatchResult {
		// let challenge_data = Self::check_get_active_challenge_info(pid);
		let challenge_data = <PuzzleChallengeInfo<T>>::get(&pid);
		ensure!(challenge_data.is_some(), Error::<T>::ChallengeNotExists);

		let mut challenge_data = challenge_data.unwrap();
		let is_allown = match challenge_data.status {
			ChallengeStatus::RaiseBackFunds(_, _) => false,
			ChallengeStatus::JudgePassed(_) => false,
			ChallengeStatus::JudgeRejected(_) => false,
			_ => true,
		};
		ensure!(is_allown, Error::<T>::ChallengeStatusError);

		let pot_infos = crate::Pallet::<T>::pot();
		ensure!(pot_infos.1 >= challenge_data.raised_total, Error::<T>::InsufficientBalance);

		let current_block_number = <frame_system::Pallet<T>>::block_number();
		challenge_data.status = ChallengeStatus::RaiseBackFunds(current_block_number, tax);
		<PuzzleChallengeInfo<T>>::insert(&pid, challenge_data.clone());
		crate::Pallet::<T>::deposit_event(Event::ChallengeStatusChange{
			challenge_status: challenge_data.status.clone(),
			pid: pid.clone(),
		});

		let mut total_pay: BalanceOf<T> = Zero::zero();
		for (acc, deposit) in challenge_data.raised_group {
			let deposit = deposit - tax * deposit;
			let res = T::Currency::transfer(
				&crate::Pallet::<T>::account_id(),
				&acc,
				deposit,
				ExistenceRequirement::KeepAlive,
			);
			total_pay+=deposit;
		}

		let im_balance = T::Currency::slash(&crate::Pallet::<T>::account_id(), challenge_data.raised_total.saturating_sub(total_pay));
		T::SlashHandler::on_unbalanced(im_balance.0);

		Ok(())
	}

	/// Check and get the active challenges.
	fn check_get_active_challenge_info(
		pid: &PuzzleSubjectHash,
	) -> Result<PuzzleChallengeData<T::AccountId, T::BlockNumber, BalanceOf<T>, Perbill>, Error<T>> {

		let challenge_info = <PuzzleChallengeInfo<T>>::get(&pid);
		if challenge_info.is_none() {
			return Err(Error::<T>::ChallengeNotExists);
		}

		let challenge_info = challenge_info.unwrap();

		let period_len = Self::get_raising_period_Length(); // T::RaisingPeriodLength::get();

		match challenge_info.status {
			ChallengeStatus::Raise(bn) => {
				let current_block_number = <frame_system::Pallet<T>>::block_number();
				// if current_block_number > bn.saturating_add(period_len) {
				if current_block_number > challenge_info.create_bn.saturating_add(period_len) {
					return Err(Error::<T>::RaisingPeriodExpired);
				}
			},
			ChallengeStatus::RaiseCompleted(bn) => {},
			_ => {
				return Err(Error::<T>::EndOfRaising);
			}
		};

		Ok(challenge_info)
	}

	fn get_raising_period_Length() -> T::BlockNumber {
		let ato_config = Pallet::<T>::get_ato_config();
		// T::RaisingPeriodLength::get()
		ato_config.raising_period_length
	}

	fn get_list_of_challengers(pid: &PuzzleSubjectHash) -> Vec<(T::AccountId, Perbill)> {
		let mut result_vec = Vec::new();
		let challenge_data = Self::check_get_active_challenge_info(pid);
		if challenge_data.is_err() {
			return result_vec;
		}
		let challenge_data = challenge_data.unwrap();
		if challenge_data.raised_total == Zero::zero() {
			return result_vec;
		}
		let raised_total = challenge_data.raised_total;

		let mut raised_len: usize = Zero::zero();
		let raised_max_len = challenge_data.raised_group.len();
		let mut all_percent = Perbill::from_percent(0);
		let mut total_amount: BalanceOf<T> = Zero::zero();
		for (acc, balance) in challenge_data.raised_group {
			raised_len+=1;
			if raised_len == raised_max_len {
				let tmp_percent = Perbill::from_percent(100).saturating_sub(all_percent);
				result_vec.push((acc, Perbill::from_percent(100).saturating_sub(all_percent)));
				all_percent = all_percent.saturating_add(tmp_percent)
			}else{
				// let tmp_percent = balance.saturating_mul(100u32.into()) / raised_total;
				let tmp_percent = Perbill::from_rational(balance, raised_total);
				result_vec.push((acc, tmp_percent));
				all_percent = all_percent.saturating_add(tmp_percent);
			}
			total_amount+=balance;
		}
		assert_eq!(all_percent, Perbill::from_percent(100));
		assert_eq!(total_amount, challenge_data.raised_total);

		result_vec
	}

	fn final_challenge(pid: &PuzzleSubjectHash, status: ChallengeStatus<T::BlockNumber, Perbill>) -> DispatchResult {
		// ensure!(<PuzzleChallengeInfo<T>>::contains_key(&pid), Error::<T>::ChallengeNotExists);
		let mut challenge_info = <PuzzleChallengeInfo<T>>::get(&pid);
		ensure!(challenge_info.is_some(), Error::<T>::ChallengeNotExists);
		let mut challenge_info = challenge_info.unwrap();

		let in_status = match status {
			ChallengeStatus::Raise(_) => {None}
			ChallengeStatus::RaiseCompleted(_) => {None}
			ChallengeStatus::RaiseBackFunds(_, _) => {None}
			ChallengeStatus::JudgePassed(_) => {Some(status)}
			ChallengeStatus::JudgeRejected(_) => {Some(status)}
		};

		if let Some(s) = in_status {
			let bn = match challenge_info.status {
				ChallengeStatus::RaiseBackFunds(x, _) => {x},
				_ => {Zero::zero()}
			};
			let current_block_number = <frame_system::Pallet<T>>::block_number();
			challenge_info.end_bn = Some(current_block_number);
			ensure!(bn != Zero::zero(), Error::<T>::NeedARefundFirst);
			challenge_info.status = s.clone();
			<PuzzleChallengeInfo<T>>::insert(&pid, challenge_info.clone());
			crate::Pallet::<T>::deposit_event(Event::ChallengeStatusChange{
				challenge_status: challenge_info.status,
				pid: pid.clone(),
			});
			return Ok(());
		};
		DispatchResult::Err(Error::<T>::ChallengeStatusError.into())
	}

	fn challenge_failed(pid: &PuzzleSubjectHash ) -> Result<(), Error<T>> {
		let challenge_data = Self::check_get_active_challenge_info(pid);
		ensure!(challenge_data.is_ok(), Error::<T>::ChallengeNotExists);
		// let mut challenge_data = challenge_data.unwrap();
		// let raised_total = challenge_data.raised_total;
		// let im_balance = T::Currency::slash(&crate::Pallet::<T>::account_id(), raised_total);
		// T::SlashHandler::on_unbalanced(im_balance.0);
		Self::back_challenge_crowdloan(pid, Perbill::from_percent(100));
		Ok(())
	}
}
