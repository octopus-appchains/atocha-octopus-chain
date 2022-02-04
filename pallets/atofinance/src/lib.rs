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
use sp_runtime::generic::Era;
use sp_std::marker::PhantomData;
use atocha_constants::MINUTES;

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
	use frame_support::sp_std::convert::TryInto;
	use frame_support::traits::ExistenceRequirement;
	use frame_system::pallet_prelude::*;
	use sp_core::sp_std::vec::Vec;
	use crate::imps::point_exchange::PointExchange;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {

		type AtoPropose: IAtoPropose<PuzzleSubjectHash>;

		type CouncilOrigin: EnsureOrigin<Self::Origin>;

		/// The staking balance.
		type Currency: Currency<Self::AccountId>
			+ ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;


		// #[pallet::constant]
		// type ExchangeEraLength: Get<Self::BlockNumber>; // 10
		//
		// #[pallet::constant]
		// type ExchangeHistoryDepth: Get<u32>; // 3
		//
		// #[pallet::constant]
		// type ExchangeMaxRewardListSize: Get<u32>; // 3
		//
		// #[pallet::constant]
		// type IssuancePerBlock: Get<BalanceOf<Self>>;

		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		#[pallet::constant]
		type PalletId: Get<PalletId>;

		// #[pallet::constant]
		// type PerEraOfBlockNumber: Get<Self::BlockNumber>;

		/// Handler for the rewards.
		type RewardHandler: OnUnbalanced<PositiveImbalanceOf<Self>>;

		/// Handler for the slashed deposits.
		type SlashHandler: OnUnbalanced<NegativeImbalanceOf<Self>>;

		// #[pallet::constant]
		// type StorageBaseFee: Get<BalanceOf<Self>>;

	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn ato_config)]
	pub type AtoConfig<T: Config> = StorageValue<_, ConfigData<BalanceOf<T>, T::BlockNumber, Perbill>, OptionQuery>;


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


	// #[pallet::storage]
	// #[pallet::getter(fn ato_point_top_list)]
	// pub type AtoPointTopList<T> = StorageValue<_, Vec<(<T as frame_system::Config>::AccountId, PointToken)>>;


	#[pallet::storage]
	#[pallet::getter(fn last_update_block_info_of_point_exchage)]
	pub type LastUpdateBlockInfoOfPointExchage<T> = StorageValue<_, <T as frame_system::Config>::BlockNumber>;

	#[pallet::storage]
	#[pallet::getter(fn last_exchange_reward_era)]
	pub type LastExchangeRewardEra<T> = StorageValue<_, ExchangeEra>;

	#[pallet::storage]
	#[pallet::getter(fn current_exchange_reward_era)]
	pub type CurrentExchangeRewardEra<T> = StorageValue<_, ExchangeEra>;

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
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn point_exchange_info)]
	pub type PointExchangeInfo<T> = StorageMap<
		_,
		Blake2_128Concat,
		ExchangeEra, //
		Vec<(
			<T as frame_system::Config>::AccountId,
			PointToken,
			Option<ExchangeInfo<PointToken, BalanceOf<T>, Perbill>>
		)>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn storage_ledger)]
	pub type StorageLedger<T> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		StorageHash,
		Blake2_128Concat,
		StorageLength,
		(
			<T as frame_system::Config>::AccountId,
		 	<T as frame_system::Config>::BlockNumber,
			BalanceOf<T>,
		),
	>;


	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub exchange_era_length: T::BlockNumber, // : BlockNumber = 6 * MINUTES; //1 * HOURS; // MyBe 7 * DAYS
		pub exchange_history_depth: u32, // u32 = 10;
		pub exchange_max_reward_list_size: u32, // u32 = 3; // Will 10 to product. // MyBe 10 size
		pub issuance_per_block: BalanceOf<T>, // Balance = 1902587519025900000; // 100000000 * 0.1 / 365 / 14400 = 1902587519025900000
		pub per_era_of_block_number: T::BlockNumber, // BlockNumber = 1 * MINUTES; // MyBe 1 * DAY
		pub challenge_threshold: Perbill, // Perbill = Perbill::from_percent(60);
		pub raising_period_length: T::BlockNumber, // BlockNumber = 10 * MINUTES;
		pub storage_base_fee: BalanceOf<T>, // Balance = 10000;
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			let ato_config = Pallet::<T>::get_ato_config();
			Self {
				exchange_era_length: ato_config.exchange_era_length,
				exchange_history_depth: ato_config.exchange_history_depth,
				exchange_max_reward_list_size: ato_config.exchange_max_reward_list_size,
				issuance_per_block: ato_config.issuance_per_block,
				per_era_of_block_number: ato_config.per_era_of_block_number,
				challenge_threshold: ato_config.challenge_threshold,
				raising_period_length: ato_config.raising_period_length,
				storage_base_fee: ato_config.storage_base_fee
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			let finance_account = <Pallet<T>>::account_id();
			if T::Currency::total_balance(&finance_account).is_zero() {
				T::Currency::deposit_creating(&finance_account, T::Currency::minimum_balance());
			}

			AtoConfig::<T>::put(ConfigData{
				exchange_era_length: self.exchange_era_length,
				exchange_history_depth: self.exchange_history_depth,
				exchange_max_reward_list_size: self.exchange_max_reward_list_size,
				issuance_per_block: self.issuance_per_block,
				per_era_of_block_number: self.per_era_of_block_number,
				challenge_threshold: self.challenge_threshold,
				raising_period_length: self.raising_period_length,
				storage_base_fee: self.storage_base_fee
			});
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(now: T::BlockNumber) -> Weight {
			let mut result_width: Weight = 0;
			let current_era = PointExchange::<T>::get_current_era();
			// println!("get_last_reward_era = {:?}, {:?} ", &PointExchange::<T>::get_last_reward_era(), &current_era);
			if 0 != current_era &&
				PointExchange::<T>::get_last_reward_era().saturating_add(2) <= current_era
			{
				log::info!(
					"AtoFinance - on_initialize = last reward era = {:?}, current era = {:?}",
					&PointExchange::<T>::get_last_reward_era(),
					&current_era
				);
				// TODO:: Collect error information for debug.
				let execute_result = PointExchange::<T>::execute_exchange(
					current_era.saturating_sub(1),
					Self::get_point_issuance(PointExchange::<T>::get_era_length())
				);
				log::info!(
					"AtoFinance - execute_result = {:?}",
					&execute_result
				);
				result_width += 10;
			}

			//
			let storage_exchange_reward_era = CurrentExchangeRewardEra::<T>::get();
			if storage_exchange_reward_era.is_none() ||
				storage_exchange_reward_era.unwrap() != current_era
			{
				result_width += 1;
				CurrentExchangeRewardEra::<T>::put(current_era);
			}
			result_width
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ApplyPointReward { who: T::AccountId , apply_era: ExchangeEra},
		ChallengeDeposit { who: T::AccountId, deposit: BalanceOf<T> },
		ChallengeStatusChange { challenge_status: ChallengeStatus<T::BlockNumber, Perbill> },
		AtoConfigUpdate { config_data: ConfigData<BalanceOf<T>, T::BlockNumber, Perbill>},
		PreStorage { who: T::AccountId, fee: BalanceOf<T>, storage_hash: StorageHash, storage_length: StorageLength },
		TakeTokenReward { pid: PuzzleSubjectHash, payout: BalanceOf<T>, fee: BalanceOf<T> },
		TakePointReward { pid: PuzzleSubjectHash, payout: PointToken, fee: PointToken },
		PointsExchange { era: ExchangeEra, exchange_list: Vec<(T::AccountId, ExchangeInfo<PointToken, BalanceOf<T>, Perbill>)> }
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	#[derive(PartialEq, Eq)]
	pub enum Error<T> {
		//
		BeneficiaryListNotEmpty,
		//
		ChallengeAlreadyExists,
		//
		ChallengeNotExists,
		//
		ChallengeStatusError,
		//
		DepositAlreadyExists,
		//
		DepositNotFound,
		//
		EndOfRaising,
		//
		EraNotEnded,
		//
		ExceededMaximumFeeLimit,
		//
		ExchangeListIsEmpty,
		//
		ExchangeApplyAlreadyExists,
		//
		ExchangeRewardEnded,
		//
		InsufficientBalance,
		//
		InsufficientPoint,
		//
		KickAwaySickExchange,
		//
		LastExchangeRewardClearing,
		//
		LedgerOwnerNotMatch,
		//
		NeedARefundFirst,
		//
		NotAnswer,
		//
		NotCreator,
		/// Error names should be descriptive.
		NoneValue,
		//
		NotPointToken,
		//
		ReserveFailed,
		//
		RefundFailed,
		//
		PointTokenIncreaseFailure,
		//
		PuzzlePeriodError,
		//
		PuzzleNotExists,
		//
		StakingNotFound,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		//
		StorageFeesTooHigh,
		//
		TicketFeeHasBeenPaid,
		//
		TicketFeeNotPaid,
		//
		TooFewPoints,
		//
		RaisingPeriodExpired,
		//
		RewardHasBeenClaimed,
		//
		WrongPaymentRatio,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		//
		#[pallet::weight(100)]
		pub fn pre_storage(
			origin: OriginFor<T>,
			storage_hash: StorageHash, // Arweave tx - id
			storage_length: StorageLength,
			max_fee: BalanceOf<T>,
		) -> DispatchResult {
			// check signer
			let who = ensure_signed(origin)?;
			// get fee.
			let storage_fee = Self::calculate_storage_fee(storage_length);
			ensure!(storage_fee.is_some(), Error::<T>::StorageFeesTooHigh);
			let storage_fee = storage_fee.unwrap();
			ensure!(storage_fee <= max_fee, Error::<T>::ExceededMaximumFeeLimit);
			//
			T::Currency::transfer(&who, &Self::account_id(), storage_fee, ExistenceRequirement::KeepAlive)?;
			StorageLedger::<T>::insert(storage_hash.clone(), storage_length, (who.clone(), Self::get_current_bn(), storage_fee.clone()));
			//who: T::AccountId, fee: BalanceOf<T>, storage_hash: StorageHash, storage_length: StorageLength
			Self::deposit_event(Event::<T>::PreStorage {
				who: who.clone(),
				fee: storage_fee,
				storage_hash,
				storage_length,
			});
			Ok(().into())
		}

		#[pallet::weight(100)]
		pub fn apply_point_reward(
			origin: OriginFor<T>,
		) -> DispatchResult {
			// check signer
			let who = ensure_signed(origin)?;
			// get fee.
			PointExchange::<T>::apply_exchange(who.clone())?;

			Self::deposit_event(Event::<T>::ApplyPointReward {
				who: who.clone(),
				apply_era: PointExchange::<T>::get_current_era(),
			});
			Ok(().into())
		}

		#[pallet::weight(100)]
		pub fn update_config(
			origin: OriginFor<T>,
			challenge_threshold: Perbill, // = Perbill::from_percent(60);
			exchange_era_length: T::BlockNumber, // = 6 * MINUTES; //1 * HOURS; // MyBe 7 * DAYS
			exchange_history_depth: u32, // = 10;
			exchange_max_reward_list_size: u32, // = 3; // Will 10 to product. // MyBe 10 size
			issuance_per_block: BalanceOf<T>, // = 1902587519025900000; // 100000000 * 0.1 / 365 / 14400 = 1902587519025900000
			per_era_of_block_number: T::BlockNumber, // = 1 * MINUTES; // MyBe 1 * DAY
			raising_period_length: T::BlockNumber, // = 10 * MINUTES;
			storage_base_fee: BalanceOf<T>, //= 10000;
		) -> DispatchResultWithPostInfo {
			// check signer
			T::CouncilOrigin::ensure_origin(origin)?;
			let config_data = ConfigData{
				exchange_era_length,
				exchange_history_depth,
				exchange_max_reward_list_size,
				issuance_per_block,
				per_era_of_block_number,
				challenge_threshold,
				raising_period_length,
				storage_base_fee
			};
			AtoConfig::<T>::put(config_data.clone());
			Self::deposit_event(Event::<T>::AtoConfigUpdate {
				config_data,
			});
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}

	fn get_ato_config() -> ConfigData<BalanceOf<T>, T::BlockNumber, Perbill> {
		let config = AtoConfig::<T>::get();
		if config.is_some() {
			return config.unwrap();
		}

		let issuance_per_block: Option<BalanceOf<T>> = 1902587519025900000u128.try_into().ok();
		let issuance_per_block = issuance_per_block.unwrap();
		ConfigData {
			exchange_era_length: MINUTES.saturating_mul(6).into(),
			exchange_history_depth: 10,
			exchange_max_reward_list_size: 3,
			issuance_per_block,
			per_era_of_block_number: MINUTES.saturating_mul(1).into(),
			challenge_threshold: Perbill::from_percent(60),
			raising_period_length: MINUTES.saturating_mul(10).into(),
			storage_base_fee: 10000u32.into()
		}
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

	pub fn calculate_storage_fee(data_length: u64) -> Option<BalanceOf<T>> {
		let base_length_balance: Result<BalanceOf<T>, _> = data_length.try_into();
		let ato_config = Pallet::<T>::get_ato_config();
		if let Ok(data_balance) = base_length_balance {
			return Some(ato_config.storage_base_fee.saturating_mul(data_balance));
		}
		None
	}

	pub fn get_point_issuance(duration_len: T::BlockNumber) -> BalanceOf<T> {
		// 100000000 * 0.1 / 365  = 27 397.260273973
		// 100000000 * 0.1 / 365 / 14400 = 1902587519025900000
		let duration_num: u32 = duration_len.unique_saturated_into();
		let ato_config = Pallet::<T>::get_ato_config();
		let issuance_per_day = ato_config.issuance_per_block;
		issuance_per_day.saturating_mul(duration_num.into())
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

