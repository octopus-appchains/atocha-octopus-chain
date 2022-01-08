use appchain_barnacle_runtime::{
	opaque::Block, opaque::SessionKeys, AccountId, BabeConfig, Balance, BalancesConfig,
	GenesisConfig, GrandpaConfig, ImOnlineConfig, OctopusAppchainConfig, OctopusLposConfig,CouncilConfig,ElectionsConfig,
	SessionConfig, Signature, SudoConfig, SystemConfig, DOLLARS, WASM_BINARY,
};
use beefy_primitives::crypto::AuthorityId as BeefyId;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_octopus_appchain::AuthorityId as OctopusId;
use sc_chain_spec::ChainSpecExtension;
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_consensus_babe::AuthorityId as BabeId;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use hex_literal::hex;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public, H256};
use sp_runtime::{
	app_crypto::sp_core::crypto::UncheckedFrom,
	traits::{IdentifyAccount, Verify},
	Perbill,
};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: sc_client_api::ForkBlocks<Block>,
	/// Known bad block hashes.
	pub bad_blocks: sc_client_api::BadBlocks<Block>,
	/// The light sync state extension used by the sync-state rpc.
	pub light_sync_state: sc_sync_state_rpc::LightSyncStateExtension,
}

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

pub fn octopus_testnet_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../resources/testnet.json")[..])
}

fn session_keys(
	babe: BabeId,
	grandpa: GrandpaId,
	im_online: ImOnlineId,
	beefy: BeefyId,
	octopus: OctopusId,
) -> SessionKeys {
	SessionKeys { babe, grandpa, im_online, beefy, octopus }
}

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(
	s: &str,
) -> (AccountId, BabeId, GrandpaId, ImOnlineId, BeefyId, OctopusId) {
	(
		get_account_id_from_seed::<sr25519::Public>(s),
		get_from_seed::<BabeId>(s),
		get_from_seed::<GrandpaId>(s),
		get_from_seed::<ImOnlineId>(s),
		get_from_seed::<BeefyId>(s),
		get_from_seed::<OctopusId>(s),
	)
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
	let mut properties = serde_json::map::Map::new();
	properties.insert("tokenDecimals".into(), 18.into());
	properties.insert("tokenSymbol".into(), "ATO".into());
	properties.insert("SS58Prefix".into(), 1024.into());
	Ok(ChainSpec::from_genesis(
		// Name
		"Atocha Testnet",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				Some(vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
				]),
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		Some(properties),
		// Extensions
		Default::default(),
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
	let mut properties = serde_json::map::Map::new();
	properties.insert("tokenDecimals".into(), 18.into());
	properties.insert("tokenSymbol".into(), "ATO".into());
	properties.insert("SS58Prefix".into(), 1024.into());

	Ok(ChainSpec::from_genesis(
		// Name
		"Atocha Local",
		// ID
		"local_atocha_chain",
		ChainType::Local,
			move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![ // (AccountId, BabeId, GrandpaId, ImOnlineId, BeefyId, OctopusId)
					(
						hex!["ecfd7bd8e5dba988db86d1eb6581f58b07f6603af6bd7f7978e2fe6973ce2b3b"].into(),
						hex!["ba32c05ec684fc939386f386740913221cab2839a36c4a688b06110b2af27636"].unchecked_into(),
						hex!["17f660e50ef43dfd413464272918bcdc93853b1a8e464ea0651d884d39ff0d60"].unchecked_into(),
						hex!["3eab31566cbc8d955cf75f62ba2284a7e086bfaaeaf61554d80b1e2a8d28d31c"].unchecked_into(),
						hex!["037189dae60e6883be7460f1efc30740d415664bcafb91d3101840d8aca61ef792"].unchecked_into(),
						hex!["e85963b110f2d25896e44b3c90b895ea6cdeb60cf7b015f940c7832e5a703558"].unchecked_into(),
					),
					(
						  hex!["d8301ff8160af5fd870c18a1d8c19ed04c67a705eeb4bb8ee68dee8b6d08b03d"].into(),
						  hex!["e6a718f9f868744b40113b02e870f5bd68572825f56ca1bf6b0440da663ec548"].unchecked_into(),
						  hex!["dc99745c24b80d2275d9809b3f2b5de4ebc139fc7296ad70f2486204bb9a8e09"].unchecked_into(),
						  hex!["40145c4e7a734180a306a18e17dafe8aaa606936e9e97ea2f92402bfe0824946"].unchecked_into(),
						  hex!["030ad8a03e48797cb92570106ccc650ac6a25cdb1ffe71457e7d9b26f53b257a95"].unchecked_into(),
						  hex!["a056fa80a559f9aae07feaf2e9ec0118c6a0c7191eb3e94b26bb3281955ab345"].unchecked_into(),
					),
					  (
						  hex!["6af5c7ab6d40dda6e3bc997fda6c264d579b1456100f2138dbbb52c15ac22433"].into(),
						  hex!["803b77cc9ae3ab640cd02ff46a69d6e5a9db93e15c7f3feb6df8898254411038"].unchecked_into(),
						  hex!["9d51325e4827162b0614bb94f9f84a810f19ba7fab2d6ad6fe3a907af79f5cb4"].unchecked_into(),
						  hex!["c88cf70e50c1bee44a1650c1e2973a39da506a6a15a152f74d8f4f707238e045"].unchecked_into(),
						  hex!["0216e0c0dffce6af471d580e882d59eecd05a0f0665f743880a58ff5e1fcafd206"].unchecked_into(),
						  hex!["3c6c1e14692e4df4421b8cd75be90b8ed318d7e93f0c7dfc8abfbe424d89ff05"].unchecked_into(),
					  ),
				],
				// Sudo account
				hex!["ecfd7bd8e5dba988db86d1eb6581f58b07f6603af6bd7f7978e2fe6973ce2b3b"].into(),
				// Pre-funded accounts
				Some(vec![
					hex!["ecfd7bd8e5dba988db86d1eb6581f58b07f6603af6bd7f7978e2fe6973ce2b3b"].into(),
					hex!["d8301ff8160af5fd870c18a1d8c19ed04c67a705eeb4bb8ee68dee8b6d08b03d"].into(),
					hex!["6af5c7ab6d40dda6e3bc997fda6c264d579b1456100f2138dbbb52c15ac22433"].into(),
					hex!["c4362617bcba50a389ff636e263323a5f6fcd351db826926e32d74f9e3513a44"].into(),
					hex!["50d5286923aded90246905b0bf04dd89e4062e48af4bb7218e40dfd0d24d0e0e"].into(),
					hex!["b83a9ab230e3705cf381d871eef46316164679798d4ae5a46510695bb18e8f48"].into(),
					hex!["f6ec24fc050d1009ddb2880e93e3279e4982d7920e78e6e79a0ee7589302900c"].into(),
					hex!["6e120ef780923ef2cc2ae815f4d567071dd4b7396ce78e6ed7e1d40ec189d704"].into(),
					hex!["e8c5a9677a1047f7b721e50b60d3ce6e80be136af1ad5427c1cbba151b62c36a"].into(),
					hex!["4ace5ba9a9c16c25f1eeec05a9d2757c1bab6dc777f3d97c97e9c0f1ca9e0e5a"].into(),
				]),
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		Some(properties),
		// Extensions
		Default::default(),
	))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, BabeId, GrandpaId, ImOnlineId, BeefyId, OctopusId)>,
	root_key: AccountId,
	endowed_accounts: Option<Vec<AccountId>>,
	_enable_println: bool,
) -> GenesisConfig {
	let mut endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(|| {
		vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			get_account_id_from_seed::<sr25519::Public>("Charlie"),
			get_account_id_from_seed::<sr25519::Public>("Dave"),
			get_account_id_from_seed::<sr25519::Public>("Eve"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
		]
	});
	// endow all authorities.
	initial_authorities.iter().map(|x| &x.0).for_each(|x| {
		if !endowed_accounts.contains(x) {
			endowed_accounts.push(x.clone())
		}
	});

	let validators = initial_authorities.iter().map(|x| (x.0.clone(), STASH)).collect::<Vec<_>>();

	const ENDOWMENT: Balance = 10000000 * DOLLARS;
	const STASH: Balance = 1000 * DOLLARS;

	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		},
		balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|x| (x, ENDOWMENT)).collect(),
		},
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						session_keys(
							x.1.clone(),
							x.2.clone(),
							x.3.clone(),
							x.4.clone(),
							x.5.clone(),
						),
					)
				})
				.collect::<Vec<_>>(),
		},
		sudo: SudoConfig { key: root_key },
		babe: BabeConfig {
			authorities: vec![],
			epoch_config: Some(appchain_barnacle_runtime::BABE_GENESIS_EPOCH_CONFIG),
		},
		elections: ElectionsConfig {
			members: endowed_accounts.iter().cloned().map(|member| (member, STASH)).collect(),
		},
		council: CouncilConfig::default(),
		im_online: ImOnlineConfig { keys: vec![] },
		grandpa: GrandpaConfig { authorities: vec![] },
		assets: Default::default(),
		beefy: Default::default(),
		octopus_appchain: OctopusAppchainConfig {
			anchor_contract: "".to_string(),
			asset_id_by_name: vec![("usdc.testnet".to_string(), 0)],
			validators,
			premined_amount: 1024 * DOLLARS,
		},
		octopus_lpos: OctopusLposConfig { era_payout: 2 * DOLLARS, ..Default::default() },
	}
}
