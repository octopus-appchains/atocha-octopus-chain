#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::DispatchResult;
use sp_std::vec::Vec;
// use frame_benchmarking::frame_support::traits::WithdrawReasons;
use frame_support::ensure;
use frame_support::sp_runtime::traits::{
	AccountIdConversion, Bounded, Saturating, UniqueSaturatedInto, Zero,
};
use frame_support::sp_runtime::{Perbill, Permill, SaturatedConversion};
use frame_support::sp_std::convert::{TryFrom, TryInto};
use frame_support::storage::types::{OptionQuery, StorageMap};
use frame_support::traits::{Currency, ExistenceRequirement, Get, OnUnbalanced, StorageInstance};
use sp_std::marker::PhantomData;

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

mod storage;
pub mod traits;
pub mod types;

use crate::types::*;
use traits::*;

pub mod imps;
pub use imps::TokenReward;

#[frame_support::pallet]
pub mod pallet {
	use crate::traits::*;
	use crate::types::*;
	use frame_support::sp_runtime::traits::Zero;
	use frame_support::sp_runtime::{Perbill, Permill};
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, PalletId};
	use frame_support::{
		pallet_prelude::*,
		traits::{
			Currency, CurrencyToVote, EnsureOrigin, EstimateNextNewSession, Get, LockIdentifier,
			LockableCurrency, OnUnbalanced, ReservableCurrency, UnixTime,
		},
		weights::Weight,
	};
	use frame_system::pallet_prelude::*;
	use sp_core::sp_std::vec::Vec;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The staking balance.
		type Currency: Currency<Self::AccountId>
			+ ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;

		/// Handler for the slashed deposits.
		type SlashHandler: OnUnbalanced<NegativeImbalanceOf<Self>>;

		/// Handler for the rewards.
		type RewardHandler: OnUnbalanced<PositiveImbalanceOf<Self>>;

		#[pallet::constant]
		type PalletId: Get<PalletId>;

		// #[pallet::constant]
		// type BasicDollars: Get<BalanceOf<Self>>;

		// #[pallet::constant]
		// type TicketFee: Get<BalanceOf<Self>>;

		// #[pallet::constant]
		// type DepositFee: Get<BalanceOf<Self>>;

		// #[pallet::constant]
		// type DayBlockCount: Get<u32>;

		// #[pallet::constant]
		// type StakingPeriod: Get<u32>;

		#[pallet::constant]
		type PerEraOfBlockNumber: Get<Self::BlockNumber>;

		type AtoPropose: IAtoPropose<PuzzleSubjectHash>;
		// // type PuzzleStatus: IPuzzleStatus<PuzzleSubjectHash>;
		// #[pallet::constant]
		// type TargetIssuanceRate: Get<Permill>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	// #[pallet::storage]
	// #[pallet::getter(fn something)]
	// // Learn more about declaring storage items:
	// // https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
	// pub type Something<T> = StorageValue<_, u32>;

	// #[pallet::storage]
	// #[pallet::getter(fn ato_staking_fee)]
	// pub type AtoStakingFee<T> = StorageValue<_, BalanceOf<T>>;

	// #[pallet::storage]
	// #[pallet::getter(fn ato_deposit_fee)]
	// pub type AtoDepositFee<T> = StorageDoubleMap<
	// 	_,
	// 	Blake2_128Concat,
	// 	PuzzleSubjectHash, // puzlle_hash,
	// 	Blake2_128Concat,
	// 	<T as frame_system::Config>::AccountId, // pay or pay to account. ,,
	// 	(BalanceOf<T>, <T as frame_system::Config>::BlockNumber),
	// 	ValueQuery,
	// >;

	// #[pallet::storage]
	// #[pallet::getter(fn ato_staking_pool)]
	// pub type AtoStakingPool<T> = StorageDoubleMap<
	// 	_,
	// 	Blake2_128Concat,
	// 	PuzzleSubjectHash, // puzlle_hash,
	// 	Blake2_128Concat,
	// 	<T as frame_system::Config>::AccountId, // pay or pay to account. ,,
	// 	Vec<(BalanceOf<T>, <T as frame_system::Config>::BlockNumber)>,
	// 	ValueQuery,
	// >;

	// #[pallet::storage]
	// #[pallet::getter(fn ato_staking_trace)]
	// pub type AtoStakingTrace<T> = StorageDoubleMap<
	// 	_,
	// 	Blake2_128Concat,
	// 	<T as frame_system::Config>::AccountId, // account id .
	// 	Blake2_128Concat,
	// 	PuzzleSubjectHash, // puzzle hash key
	// 	BalanceOf<T>,
	// 	ValueQuery,
	// >;

	// #[pallet::storage]
	// #[pallet::getter(fn ato_sponsor_fee)]
	// pub type AtoSponsorFee<T> = StorageDoubleMap<
	// 	_,
	// 	Blake2_128Concat,
	// 	PuzzleSubjectHash, // puzzle hash key
	// 	Blake2_128Concat,
	// 	<T as frame_system::Config>::AccountId, // account id .
	// 	BalanceOf<T>,
	// 	ValueQuery,
	// >;

	// #[pallet::storage]
	// #[pallet::getter(fn ato_ticket_fee)]
	// pub type AtoTicketFee<T> = StorageDoubleMap<
	// 	_,
	// 	Blake2_128Concat,
	// 	PuzzleSubjectHash, // puzzle hash key
	// 	Blake2_128Concat,
	// 	<T as frame_system::Config>::AccountId, // account id .
	// 	(BalanceOf<T>, <T as frame_system::Config>::BlockNumber),
	// 	ValueQuery,
	// >;

	// #[pallet::storage]
	// #[pallet::getter(fn ato_staking_interest_rate)]
	// pub type AtoStakingInterestRate<T> = StorageMap<
	// 	_,
	// 	Blake2_128Concat,
	// 	AtoStakingPeriod, // puzzle hash key
	// 	AtoInterestRate,
	// 	ValueQuery,
	// >;

	//
	#[pallet::storage]
	#[pallet::getter(fn ato_finanace_ledger)]
	pub type AtoFinanceLedger<T> = StorageMap<
		_,
		Blake2_128Concat,
		PuzzleSubjectHash, // puzzle hash key
		PotLedgerData<
			<T as frame_system::Config>::AccountId,
			BalanceOf<T>,
			<T as frame_system::Config>::BlockNumber,
		>,
		ValueQuery,
	>;

	//
	#[pallet::storage]
	#[pallet::getter(fn ato_finance_reward)]
	pub type AtoFinanceReward<T> = StorageMap<
		_,
		Blake2_128Concat,
		PuzzleSubjectHash, // puzzle hash key
		PotRewardData<
			<T as frame_system::Config>::AccountId,
			<T as frame_system::Config>::BlockNumber,
			BalanceOf<T>,
			Perbill,
		>,
		ValueQuery,
	>;

	//
	#[pallet::storage]
	#[pallet::getter(fn ato_point_reward)]
	pub type AtoPointReward<T> = StorageMap<
		_,
		Blake2_128Concat,
		PuzzleSubjectHash, // puzzle hash key
		PotRewardData<
			<T as frame_system::Config>::AccountId,
			<T as frame_system::Config>::BlockNumber,
			PointToken,
			Perbill,
		>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn ato_point_ledger)]
	pub type AtoPointLedger<T> = StorageMap<
		_,
		Blake2_128Concat,
		<T as frame_system::Config>::AccountId, //
		PointToken,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn ato_point_total)]
	pub type AtoPointTotal<T> = StorageValue<_, PointToken>;

	#[pallet::storage]
	#[pallet::getter(fn puzzle_challenge_info)]
	pub type PuzzleChallengeInfo<T> = StorageMap<
		_,
		Blake2_128Concat,
		PuzzleSubjectHash, //
		PuzzleChallengeData<
			<T as frame_system::Config>::AccountId,
			<T as frame_system::Config>::BlockNumber,
			BalanceOf<T>, Perbill,
		>,
		ValueQuery,
	>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub _pt: PhantomData<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { _pt: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			let finance_account = <Pallet<T>>::account_id();
			if T::Currency::total_balance(&finance_account).is_zero() {
				T::Currency::deposit_creating(&finance_account, T::Currency::minimum_balance());
			}
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SomethingStored(u32, T::AccountId),
		StakingInterestRate(AtoInterestRate, AtoStakingPeriod),
		ChallengeStatusChange(ChallengeStatus<T::BlockNumber, Perbill>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	#[derive(PartialEq, Eq)]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		//
		DepositAlreadyExists,
		//
		InsufficientBalance,
		//
		DepositNotFound,
		//
		TicketFeeHasBeenPaid,
		//
		TicketFeeNotPaid,
		//
		RefundFailed,
		//
		NeedARefundFirst,
		//
		ChallengeStatusError,
		//
		ReserveFailed,
		//
		PuzzlePeriodError,
		//
		StakingNotFound,
		//
		PuzzleNotExists,
		//
		ChallengeAlreadyExists,
		//
		ChallengeNotExists,
		//
		RaisingPeriodExpired,
		//
		EndOfRaising,
		//
		RewardHasBeenClaimed,
		//
		BeneficiaryListNotEmpty,
		//
		WrongPaymentRatio,
		//
		NotAnswer,
		//
		NotCreator,
		//
		LedgerOwnerNotMatch,
		//
		InsufficientPoint,
		//
		NotPointToken,
		//
		PointTokenIncreaseFailure,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}

impl<T: Config> Pallet<T> {
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}

	pub fn pot() -> (T::AccountId, BalanceOf<T>) {
		let account_id = Self::account_id();
		let balance =
			T::Currency::free_balance(&account_id).saturating_sub(T::Currency::minimum_balance());
		(account_id, balance)
	}

	pub fn get_current_bn() -> T::BlockNumber {
		<frame_system::Pallet<T>>::block_number()
	}
}

struct Prefix;
impl StorageInstance for Prefix {
	fn pallet_prefix() -> &'static str {
		"ato"
	}
	const STORAGE_PREFIX: &'static str = "atocha";
}

impl<T: Config>
	IPuzzleLedger<T::AccountId, BalanceOf<T>, PuzzleSubjectHash, T::BlockNumber, DispatchResult>
	for Pallet<T>
{
	fn do_bonus(
		pid: PuzzleSubjectHash,
		who: T::AccountId,
		amount: BalanceOf<T>,
		create_bn: T::BlockNumber,
	) -> DispatchResult {
		// Get puzzle ledger
		let storage_ledger = <AtoFinanceLedger<T>>::try_get(&pid).ok();
		// Create sponsor data
		let sponsor_data =
			SponsorData { sponsor: who.clone(), funds: amount, create_bn, reason: Vec::new() };
		// IF None create new one.
		let mut pot_ledger = PotLedgerData {
			owner: who.clone(),
			total: amount,
			funds: amount,
			sponsor_list: Vec::new(),
		};
		pot_ledger.sponsor_list.push(sponsor_data.clone());

		if storage_ledger.is_some() {
			// Has old data, so check owner same as `who`.
			pot_ledger = storage_ledger.unwrap();
			// if &pot_ledger.owner != &who {
			// 	return Err(Error::<T>::LedgerOwnerNotMatch).into();
			// }
			ensure!(&pot_ledger.owner == &who, Error::<T>::LedgerOwnerNotMatch);
			// Increase puzzle bonus.
			pot_ledger.funds = pot_ledger.funds.saturating_add(amount);
			pot_ledger.total = pot_ledger.total.saturating_add(amount);

			// Add rewards and insert it in reverse order according to the amount of funds.
			match pot_ledger
				.sponsor_list
				.binary_search_by(|sp_data| sp_data.funds.cmp(&amount).reverse())
			{
				Ok(pos) => {
					pot_ledger.sponsor_list.insert(pos, sponsor_data.clone());
				}
				Err(pos) => {
					pot_ledger.sponsor_list.insert(pos, sponsor_data.clone());
				}
			}
		}

		T::Currency::transfer(&who, &Self::account_id(), amount, ExistenceRequirement::KeepAlive)?;

		// Store to ledger data.
		<AtoFinanceLedger<T>>::insert(&pid, pot_ledger);
		Ok(())
	}

	fn do_sponsorship(
		pid: PuzzleSubjectHash,
		who: T::AccountId,
		amount: BalanceOf<T>,
		create_bn: T::BlockNumber,
		reason: Vec<u8>,
	) -> DispatchResult {
		// Get puzzle ldeger
		let storage_ledger = <AtoFinanceLedger<T>>::try_get(&pid).ok();

		// If None return an error.
		ensure!(storage_ledger.is_some(), Error::<T>::PuzzleNotExists);
		let mut pot_ledger = storage_ledger.unwrap();

		// Create SponsorData
		let sponsor_data = SponsorData { sponsor: who.clone(), funds: amount, create_bn, reason };

		// Add rewards and insert it in reverse order according to the amount of funds.
		match pot_ledger
			.sponsor_list
			.binary_search_by(|sp_data| sp_data.funds.cmp(&amount).reverse())
		{
			Ok(pos) => {
				pot_ledger.sponsor_list.insert(pos, sponsor_data.clone());
			}
			Err(pos) => {
				pot_ledger.sponsor_list.insert(pos, sponsor_data.clone());
			}
		}

		T::Currency::transfer(&who, &Self::account_id(), amount, ExistenceRequirement::KeepAlive)?;

		//
		pot_ledger.total = pot_ledger.total.saturating_add(amount);
		// Store to ledger data.
		<AtoFinanceLedger<T>>::insert(&pid, pot_ledger);

		Ok(())
	}
}
