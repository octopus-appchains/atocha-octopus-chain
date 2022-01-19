use crate as pallet_atocha;
use frame_support::parameter_types;
use frame_support::sp_runtime::app_crypto::sp_core::sr25519::Signature;
use frame_support::sp_runtime::traits::{IdentifyAccount, Verify};
use frame_support::PalletId;
use frame_system as system;
use sp_core::hashing::sha2_256;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Permill,Perbill,
};

use crate::types::PuzzleVersion;
use frame_support::assert_ok;
use frame_support::traits::{Contains, GenesisBuild};


type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
pub(crate) type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

pub type Balance = u128;
pub type BlockNumber = u64;
pub const DOLLARS: Balance = 1_000_000_000_000;


// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		AtochaModule: pallet_atocha::{Pallet, Call, Storage, Event<T>},
		AtochaPot: pallet_atofinance::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

parameter_types! {
	pub const MinBonusOfPuzzle: Balance = 100 * DOLLARS;
	pub const ChallengePeriodLength: BlockNumber = 100;
	pub const TaxOfTVS: Perbill = Perbill::from_percent(5); //  When creator reveal puzzle that it tax fee .
	pub const TaxOfTVO: Perbill = Perbill::from_percent(10); // When answer reveal puzzle that it tax fee.
	pub const TaxOfTI: Perbill = Perbill::from_percent(10);
	pub const PenaltyOfCP: Perbill = Perbill::from_percent(10);
	pub const MaxSponsorExplainLen: u32 = 256;
	pub const MaxAnswerExplainLen: u32 = 1024;
}

impl crate::Config for Test {
	type Event = Event;
	type Currency = <Self as pallet_atofinance::Config>::Currency;
	type MinBonusOfPuzzle = MinBonusOfPuzzle;
	type ChallengePeriodLength = ChallengePeriodLength;
	type PuzzleLedger = AtochaPot; // pallet_atofinance::Pallet<Test>;
	type PuzzleRewardOfToken = pallet_atofinance::imps::TokenReward<Self>;
	type PuzzleRewardOfPoint = pallet_atofinance::imps::PointReward<Self>;
	type AtoChallenge = pallet_atofinance::imps::challenge_manager::ChallengeManager<Self>;
	type AtoPointsManage = pallet_atofinance::imps::PointManager<Self>;
	type TaxOfTVS = TaxOfTVS;
	type TaxOfTVO = TaxOfTVO;
	type TaxOfTI = TaxOfTI;
	type PenaltyOfCP = PenaltyOfCP;
	type MaxSponsorExplainLen = MaxSponsorExplainLen;
	type MaxAnswerExplainLen = MaxAnswerExplainLen;
	type CouncilOrigin = frame_system::EnsureRoot<AccountId>;
}

parameter_types! {
	pub const AresFinancePalletId: PalletId = PalletId(*b"ocw/fund");
	// pub const BasicDollars: Balance = DOLLARS;
	// pub const TicketFee: Balance = 5 * DOLLARS;
	// pub const DepositFee: Balance = 100 * DOLLARS;
	// pub const DayBlockCount: u32 = 14400;
	// pub const StakingPeriod: u32 = 10;
	pub const PerEraOfBlockNumber: BlockNumber = 5;
	// pub TargetIssuanceRate: Permill = Permill::from_float(0.1);
	pub ChallengeThreshold: Perbill = Perbill::from_float(0.6);
	pub RaisingPeriodLength: BlockNumber = 5;
	pub StorageBaseFee: Balance = 1000;
}

impl pallet_atofinance::imps::challenge_manager::Config for Test {
	type ChallengeThreshold = ChallengeThreshold;
	type RaisingPeriodLength = RaisingPeriodLength;
}

impl pallet_atofinance::Config for Test {
	type Event = Event;
	type PalletId = AresFinancePalletId;
	type Currency = pallet_balances::Pallet<Self>;
	type SlashHandler = ();
	type RewardHandler = ();
	type PerEraOfBlockNumber = PerEraOfBlockNumber;
	type AtoPropose = ();
	type StorageBaseFee = StorageBaseFee;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
	pub const MaxLocks: u32 = 10;
}
impl pallet_balances::Config for Test {
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type MaxLocks = MaxLocks;
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(toAid(1), 1_000_000_000_000_000),
			(toAid(2), 2_000_000_000_000_000),
			(toAid(3), 3_000_000_000_000_000),
			(toAid(4), 4_000_000_000_000_000),
			(toAid(5), 5_000_000_000_000_000),
			(toAid(6), 6_000_000_000_000_000),
		],
	}
		.assimilate_storage(&mut t)
		.unwrap();

	// crate::GenesisConfig::<Test> { _pt: Default::default() }
	// 	.assimilate_storage(&mut t)
	// 	.unwrap();

	pallet_atofinance::GenesisConfig::<Test> { _pt: Default::default() }
		.assimilate_storage(&mut t)
		.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub fn make_answer_sha256(answer_hash: Vec<u8>, puzzle_hash_txid: Vec<u8>) -> Vec<u8> {
	// let mut answer_hash = toVec("ANSWER_HASH");
	let mut sha_answer_hash = sha2_256(answer_hash.as_slice()).to_vec();
	// println!("On test sha_answer_hash B1 = {:?}", sha_answer_hash);
	let sha1_answer_hex = &hex::encode(&sha_answer_hash);
	let mut sha1_ansser_vec = sha1_answer_hex.as_bytes().to_vec();
	// println!("On test sha_answer_hash B2 = {:?}", sha1_ansser_vec);
	let mut puzzle_hash_txid = puzzle_hash_txid.clone();
	sha1_ansser_vec.append(&mut puzzle_hash_txid);
	// println!("On test sha_answer_hash B2 = {:?}", sha1_ansser_vec);
	// let raw_str = sp_std::str::from_utf8(sha1_ansser_vec.as_slice());
	let sha256_answer = sha2_256(sha1_ansser_vec.as_slice());
	shaToVec(sha256_answer.to_vec())
}

pub fn shaToVec (sha_vec: Vec<u8>) ->Vec<u8> {
	let mut result_answer_u8 = [0u8; 32 * 2];
	// Answer sha256 to encode slice
	let encode_result =
		hex::encode_to_slice(&sha_vec.as_slice(), &mut result_answer_u8 as &mut [u8]);
	assert!(encode_result.is_ok(), "make_answer_sign to Hex failed.");
	result_answer_u8.to_vec()
}

pub(crate) fn handle_create_puzzle(
	account_id: AccountId,
	puzzle_hash: Vec<u8>,
	answer_hash: Vec<u8>,
	// answer_signed: Vec<u8>,
	// answer_nonce: &str,
	// ticket: PuzzleTicket,
	// duration: DurationBn,
) {
	let origin = Origin::signed(account_id);
	// let puzzle_hash = puzzle_hash.as_bytes().to_vec();
	// let answer_signed = answer_signed;
	// let answer_nonce = answer_nonce.as_bytes().to_vec();
	let puzzle_version: PuzzleVersion = 1;

	// Dispatch a signed extrinsic.
	assert_ok!(AtochaModule::create_puzzle(
		origin,
		puzzle_hash.clone(),
		answer_hash.clone(),
		100 * DOLLARS,
		puzzle_version.clone()
	));
}

pub(crate) fn toAid(start: u8) -> AccountId {
	AccountId::from_raw([start; 32])
}

pub(crate) fn toVec(to_str: &str) -> Vec<u8> {
	to_str.as_bytes().to_vec()
}
