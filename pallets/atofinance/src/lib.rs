#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::DispatchResult;
use sp_std::vec::Vec;
// use frame_benchmarking::frame_support::traits::WithdrawReasons;
use frame_support::ensure;
use frame_support::sp_runtime::traits::{
	AccountIdConversion, Saturating, UniqueSaturatedInto, Zero,
};
use frame_support::sp_runtime::{Perbill, Permill};
use frame_support::sp_std::convert::{TryInto};
use frame_support::traits::{Currency, ExistenceRequirement, Get, OnUnbalanced, StorageInstance};
use atocha_constants::MINUTES;

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

pub mod migrations;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

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
	use frame_support::sp_runtime::traits::{StaticLookup, Zero};
	use frame_support::sp_runtime::{Perbill};
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, PalletId};
	use frame_support::{
		// pallet_prelude::*,
		traits::{
			Currency, EnsureOrigin, Get,
			LockableCurrency, OnUnbalanced, ReservableCurrency,
		},
		weights::Weight,
	};
	use frame_support::sp_std::convert::TryInto;
	use frame_support::traits::{ExistenceRequirement, StorageVersion};
	use frame_system::pallet_prelude::*;
	use sp_core::sp_std::vec::Vec;
	use atocha_constants::MINUTES;
	use crate::imps::point_exchange::PointExchange;
	use crate::imps::PointManager;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(18);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {

		type AtoPropose: IAtoPropose<PuzzleSubjectHash>;

		type CouncilOrigin: EnsureOrigin<Self::Origin>;

		/// The staking balance.
		type Currency: Currency<Self::AccountId>
			+ ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;

		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// Handler for the rewards.
		type RewardHandler: OnUnbalanced<PositiveImbalanceOf<Self>>;

		/// Handler for the slashed deposits.
		type SlashHandler: OnUnbalanced<NegativeImbalanceOf<Self>>;

	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// #[pallet::storage]
	// #[pallet::getter(fn ato_config)]
	// pub type AtoConfig<T: Config> = StorageValue<_, ConfigData<BalanceOf<T>, T::BlockNumber, Perbill>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn ato_config)]
	pub type AtoConfig2<T: Config> = StorageValue<_, ConfigData<BalanceOf<T>, T::BlockNumber, Perbill>, OptionQuery>;

	// TODO:: Kami need test upgrade with test net.
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
		OptionQuery,
	>;

	//TODO:: Kami need test upgrade with test net.
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
		OptionQuery,
	>;

	// TODO:: Kami set OptionQuery with this storage so you be mast test with testnet.
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
		OptionQuery,
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
	#[pallet::getter(fn last_update_block_info_of_point_exchage)]
	pub type LastUpdateBlockInfoOfPointExchage<T> = StorageValue<_, <T as frame_system::Config>::BlockNumber>;

	#[pallet::storage]
	#[pallet::getter(fn last_exchange_reward_era)]
	pub type LastExchangeRewardEra<T> = StorageValue<_, ExchangeEra>;

	#[pallet::storage]
	#[pallet::getter(fn current_exchange_reward_era)]
	pub type CurrentExchangeRewardEra<T> = StorageValue<_, ExchangeEra>;

	#[pallet::storage]
	#[pallet::getter(fn exchange_reward_era_start_bn)]
	pub type ExchangeRewardEraStartBn<T> = StorageMap<
		_,
		Blake2_128Concat,
		ExchangeEra,
		<T as frame_system::Config>::BlockNumber,
		OptionQuery
	>;

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
		pub exchange_era_block_length: T::BlockNumber, // : BlockNumber = 6 * MINUTES; //1 * HOURS; // MyBe 7 * DAYS
		pub exchange_history_depth: u32, // u32 = 10;
		pub exchange_max_reward_list_size: u32, // u32 = 3; // Will 10 to product. // MyBe 10 size
		pub issuance_per_block: BalanceOf<T>, // Balance = 1902587519025900000; // 100000000 * 0.1 / 365 / 14400 = 1902587519025900000
		pub point_reward_epoch_block_length: T::BlockNumber, // BlockNumber = 1 * MINUTES; // MyBe 1 * DAY
		pub challenge_threshold: Perbill, // Perbill = Perbill::from_percent(60);
		pub raising_period_length: T::BlockNumber, // BlockNumber = 10 * MINUTES;
		pub storage_base_fee: BalanceOf<T>, // Balance = 10000;
		pub mint_tax: Perbill,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			let issuance_per_block: Option<BalanceOf<T>> = 1902587519025900000u128.try_into().ok();
			let issuance_per_block = issuance_per_block.unwrap();
			let ato_config = ConfigData {
				exchange_era_block_length: MINUTES.saturating_mul(6).into(),
				exchange_history_depth: 10,
				exchange_max_reward_list_size: 3,
				issuance_per_block,
				point_reward_epoch_block_length: MINUTES.saturating_mul(1).into(),
				challenge_threshold: Perbill::from_percent(60),
				raising_period_length: MINUTES.saturating_mul(10).into(),
				storage_base_fee: 10000u32.into(),
				mint_tax: Perbill::from_percent(5),
			};

			Self {
				exchange_era_block_length: ato_config.exchange_era_block_length,
				exchange_history_depth: ato_config.exchange_history_depth,
				exchange_max_reward_list_size: ato_config.exchange_max_reward_list_size,
				issuance_per_block: ato_config.issuance_per_block,
				point_reward_epoch_block_length: ato_config.point_reward_epoch_block_length,
				challenge_threshold: ato_config.challenge_threshold,
				raising_period_length: ato_config.raising_period_length,
				storage_base_fee: ato_config.storage_base_fee,
				mint_tax: ato_config.mint_tax,
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

			AtoConfig2::<T>::put(ConfigData{
				exchange_era_block_length: self.exchange_era_block_length,
				exchange_history_depth: self.exchange_history_depth,
				exchange_max_reward_list_size: self.exchange_max_reward_list_size,
				issuance_per_block: self.issuance_per_block,
				point_reward_epoch_block_length: self.point_reward_epoch_block_length,
				challenge_threshold: self.challenge_threshold,
				raising_period_length: self.raising_period_length,
				storage_base_fee: self.storage_base_fee,
				mint_tax: self.mint_tax,
			});
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(now: T::BlockNumber) -> Weight {
			let mut result_width: Weight =  PointExchange::<T>::check_and_update_era(now);
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
				let config = Self::get_ato_config();
				let execute_result = PointExchange::<T>::execute_exchange(
					current_era.saturating_sub(1),
					Self::get_point_issuance(PointExchange::<T>::get_era_length()),
					config.mint_tax,
				);
				log::info!(
					"AtoFinance - execute_result = {:?}",
					&execute_result
				);
				result_width += PointExchange::<T>::get_max_reward_list_size() as Weight ;
			}

			//
			// let storage_exchange_reward_era = CurrentExchangeRewardEra::<T>::get();
			// if storage_exchange_reward_era.is_none() ||
			// 	storage_exchange_reward_era.unwrap() != current_era
			// {
			// 	result_width += 1;
			// 	CurrentExchangeRewardEra::<T>::put(current_era);
			// }
			result_width
		}

		// fn on_runtime_upgrade() -> frame_support::weights::Weight {
		// 	log::info!("ato-finance upgrade in.");
		// 	let _ = <AtoConfig<T>>::translate::<OldConfigData<BalanceOf<T>, T::BlockNumber, Perbill>, _>(|maybe_old_data| {
		// 		maybe_old_data.map(|old_data| {
		// 			log::info!(
		// 				target: "runtime::ato-finance",
		// 				"migrated AtoConfig add mint_tax field.",
		// 			);
		// 			ConfigData {
		// 				exchange_era_block_length: old_data.exchange_era_block_length,
		// 				exchange_history_depth: old_data.exchange_history_depth,
		// 				exchange_max_reward_list_size: old_data.exchange_max_reward_list_size,
		// 				issuance_per_block: old_data.issuance_per_block,
		// 				point_reward_epoch_block_length: old_data.point_reward_epoch_block_length,
		// 				challenge_threshold: old_data.challenge_threshold,
		// 				raising_period_length: old_data.raising_period_length,
		// 				storage_base_fee: old_data.storage_base_fee,
		// 				mint_tax: Perbill::from_percent(5),
		// 			}
		// 		})
		// 	});
		// 	T::DbWeight::get().writes(1)
		// }
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Apply for `Points` to exchange `Ato-Tokens`.
		ApplyPointReward { who: T::AccountId , apply_era: ExchangeEra},
		/// Update `AtoFinance` module configuration.
		AtoConfigUpdate { config_data: ConfigData<BalanceOf<T>, T::BlockNumber, Perbill>},
		/// Challenger increases deposit fee.
		ChallengeDeposit { pid: PuzzleSubjectHash, who: T::AccountId, deposit: BalanceOf<T>, deposit_type: ChallengeDepositType},
		/// Challengers must raise funds before this block.
		ChallengeRaisePeriodDeadline { pid: PuzzleSubjectHash, deadline: T::BlockNumber },
		/// Challenger information status changed.
		ChallengeStatusChange { pid: PuzzleSubjectHash, challenge_status: ChallengeStatus<T::BlockNumber, Perbill> },
		/// When puzzle create or someone sponsored.
		PuzzleDeposit { pid: PuzzleSubjectHash, who: T::AccountId, deposit: BalanceOf<T>, tip: Vec<u8>, kind: PuzzleDepositType},
		/// Pre-stored resources succeeded.
		PreStorage { who: T::AccountId, fee: BalanceOf<T>, storage_hash: StorageHash, storage_length: StorageLength },
		/// Answer received `ATO-Token` rewards.
		TakeTokenReward { pid: PuzzleSubjectHash, payout: BalanceOf<T>, fee: BalanceOf<T> },
		/// Answer received `Point` rewards.
		TakePointReward { pid: PuzzleSubjectHash, payout: PointToken, fee: PointToken },
		/// `Point` exchange `Ato-Token` is executed by the system.
		PointsExchange { era: ExchangeEra, exchange_list: Vec<(T::AccountId, ExchangeInfo<PointToken, BalanceOf<T>, Perbill>)> },
		/// Mint some `PointToken` with root privileges for testing.
		MintPoints { dest: T::AccountId, points: PointToken, }
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
		ChallengeDepositTooLow,
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
		//
		BeneficiaryNotFound,
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
		pub fn refresh_point_reward_rank_list(
			origin: OriginFor<T>,
		) -> DispatchResult {
			// check signer
			let _who = ensure_signed(origin)?;
			PointExchange::<T>::update_apply_list_point();
			Ok(().into())
		}


		#[pallet::weight(100)]
		pub fn update_config(
			origin: OriginFor<T>,
			challenge_threshold: Perbill, // = Perbill::from_percent(60);
			exchange_era_block_length: T::BlockNumber, // = 6 * MINUTES; //1 * HOURS; // MyBe 7 * DAYS
			exchange_history_depth: u32, // = 10;
			exchange_max_reward_list_size: u32, // = 3; // Will 10 to product. // MyBe 10 size
			issuance_per_block: BalanceOf<T>, // = 1902587519025900000; // 100000000 * 0.1 / 365 / 14400 = 1902587519025900000
			point_reward_epoch_block_length: T::BlockNumber, // = 1 * MINUTES; // MyBe 1 * DAY
			raising_period_length: T::BlockNumber, // = 10 * MINUTES;
			storage_base_fee: BalanceOf<T>, //= 10000;
			mint_tax: Perbill,
		) -> DispatchResultWithPostInfo {
			// check signer
			T::CouncilOrigin::ensure_origin(origin)?;
			let config_data = ConfigData{
				exchange_era_block_length,
				exchange_history_depth,
				exchange_max_reward_list_size,
				issuance_per_block,
				point_reward_epoch_block_length,
				challenge_threshold,
				raising_period_length,
				storage_base_fee,
				mint_tax
			};
			AtoConfig2::<T>::put(config_data.clone());
			Self::deposit_event(Event::<T>::AtoConfigUpdate {
				config_data,
			});
			Ok(().into())
		}

		#[pallet::weight(100)]
		pub fn mint_points(
			origin: OriginFor<T>,
			dest: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] points: PointToken,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			let dest = T::Lookup::lookup(dest)?;
			PointManager::<T>::increase_points_to(&dest, points)?;
			Self::deposit_event(Event::<T>::MintPoints {
				dest,
				points
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
		let config = AtoConfig2::<T>::get();
		if config.is_some() {
			return config.unwrap();
		}
		let issuance_per_block: Option<BalanceOf<T>> = 1902587519025900000u128.try_into().ok();
		let issuance_per_block = issuance_per_block.unwrap();
		ConfigData {
			exchange_era_block_length: MINUTES.saturating_mul(6).into(),
			exchange_history_depth: 10,
			exchange_max_reward_list_size: 3,
			issuance_per_block,
			point_reward_epoch_block_length: MINUTES.saturating_mul(1).into(),
			challenge_threshold: Perbill::from_percent(60),
			raising_period_length: MINUTES.saturating_mul(10).into(),
			storage_base_fee: 10000u32.into(),
			mint_tax: Perbill::from_percent(5),
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

		Self::deposit_event(Event::<T>::PuzzleDeposit {
			pid: pid.clone(),
			who: who,
			deposit: amount,
			tip: "".as_bytes().to_vec(),
			kind: PuzzleDepositType::Initial,
		});

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
		let sponsor_data = SponsorData { sponsor: who.clone(), funds: amount, create_bn, reason: reason.clone() };

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

		Self::deposit_event(Event::<T>::PuzzleDeposit {
			pid: pid.clone(),
			who: who,
			deposit: amount,
			tip: reason,
			kind: PuzzleDepositType::Sponsored
		});

		Ok(())
	}

	fn get_pot_ledger( pid: PuzzleSubjectHash ) -> Option<PotLedgerData<T::AccountId, BalanceOf<T>, T::BlockNumber>> {
		<AtoFinanceLedger<T>>::try_get(&pid).ok()
	}
}

