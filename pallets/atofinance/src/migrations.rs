use frame_support::pallet_prelude::Get;
use frame_support::traits::OnRuntimeUpgrade;

/// A struct that does not migration, but only checks that the counter prefix exists and is correct.
pub struct UpdateFinanceConfig<T: crate::Config>(sp_std::marker::PhantomData<T>);
impl<T: crate::Config> OnRuntimeUpgrade for UpdateFinanceConfig<T> {

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		log::info!(
			"migrated AtoConfig add mint_tax field.",
		);

		// let _ = <AtoConfig<T>>::translate::<OldConfigData<BalanceOf<T>, T::BlockNumber, Perbill>, _>(|maybe_old_data| {
		// 	maybe_old_data.map(|old_data| {
		// 		ConfigData {
		// 			exchange_era_block_length: old_data.exchange_era_block_length,
		// 			exchange_history_depth: old_data.exchange_history_depth,
		// 			exchange_max_reward_list_size: old_data.exchange_max_reward_list_size,
		// 			issuance_per_block: old_data.issuance_per_block,
		// 			point_reward_epoch_block_length: old_data.point_reward_epoch_block_length,
		// 			challenge_threshold: old_data.challenge_threshold,
		// 			raising_period_length: old_data.raising_period_length,
		// 			storage_base_fee: old_data.storage_base_fee,
		// 			mint_tax: Perbill::from_percent(5),
		// 		}
		// 	})
		// });
		T::DbWeight::get().writes(1)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {

		Ok(())
	}
}
