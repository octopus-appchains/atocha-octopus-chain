use crate::traits::*;
use crate::types::*;
use crate::{mock::*, Error};
use frame_support::sp_runtime::sp_std::convert::TryInto;
use frame_support::sp_runtime::Permill;
use frame_support::traits::OnInitialize;
use frame_support::{
	assert_err, assert_noop, assert_ok, assert_storage_noop,
	traits::{
		Currency, ExistenceRequirement::AllowDeath, ExistenceRequirement::KeepAlive,
		LockIdentifier, LockableCurrency, ReservableCurrency, WithdrawReasons,
	},
};
use sp_runtime::{traits::Hash, Perbill};

mod test_IAtoChallenge;
mod test_IPointExchange;
mod test_IPuzzleLedger;
mod test_IPuzzlePoints;
mod test_IPuzzleReward_Of_Points;
mod test_IPuzzleReward_Of_Tokens;

#[test]
fn test_Perbill() {
	new_test_ext().execute_with(|| {
		// This is 0.42
		let x = Permill::from_parts(42_0_000);
		let x2 = Permill::from_float(0.42);
		assert_eq!(x, x2);
		// This is 100.000
		let y = x * 100_0000u64;
		assert_eq!(y, 42_0000)
	});
}


#[test]
fn test_PreStorage() {
	new_test_ext().execute_with(|| {
		let current_bn: u64 = 800;
		System::set_block_number(current_bn);
		<AtochaPot as OnInitialize<u64>>::on_initialize(current_bn);
		const ACCOUNT_ID_1: u64 = 2;
		assert_eq!(Balances::free_balance(ACCOUNT_ID_1), 200000000000000);
		assert_noop!(
			AtochaPot::pre_storage(Origin::signed(ACCOUNT_ID_1), "STORAGE_HASH".as_bytes().to_vec(), 9000, 9000 ),
			Error::<Test>::ExceededMaximumFeeLimit,
		);
		assert_ok!(AtochaPot::pre_storage(Origin::signed(ACCOUNT_ID_1), "STORAGE_HASH".as_bytes().to_vec(), 9000, 100000000000000 ));

		assert_eq!(
			AtochaPot::storage_ledger("STORAGE_HASH".as_bytes().to_vec(), 9000),
			Some((ACCOUNT_ID_1, current_bn)),
		);

		assert_eq!(
			AtochaPot::storage_ledger("STORAGE_HASH".as_bytes().to_vec(), 8000),
			None,
		);
	});
}

#[test]
fn test_Transfer() {
	new_test_ext().execute_with(|| {
		let current_bn: u64 = 1;
		System::set_block_number(current_bn);
		<AtochaPot as OnInitialize<u64>>::on_initialize(current_bn);

		const ACCOUNT_ID_1: u64 = 2;
		const ACCOUNT_ID_2: u64 = 2;

		assert_eq!(Balances::free_balance(ACCOUNT_ID_2), 200000000000000);
		Balances::set_lock(*b"12345678", &ACCOUNT_ID_2, 150000000000000, WithdrawReasons::all());
		assert_eq!(Balances::free_balance(ACCOUNT_ID_2), 200000000000000);
		assert_eq!(Balances::usable_balance(ACCOUNT_ID_2.clone()), 50000000000000);

		assert_noop!(
			Balances::reserve(&ACCOUNT_ID_2, 80000000000000),
			pallet_balances::Error::<Test>::LiquidityRestrictions
		);
	});
}
