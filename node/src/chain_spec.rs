use std::{collections::BTreeMap, str::FromStr};

use serde::{Deserialize, Serialize};
// Substrate
use sc_service::ChainType;
use sp_core::{sr25519, storage::Storage, Pair, Public, H160, U256};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::Perbill;
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_state_machine::BasicExternalities;
// Frontier
use qchain_template_runtime::{AccountId, Balance, Block, EnableManualSeal, GenesisConfig, MaxNominations, Signature, StakerStatus, WASM_BINARY};
use qchain_template_runtime::{CouncilConfig, ImOnlineConfig, NominationPoolsConfig, SessionConfig, StakingConfig};
use qchain_template_runtime::AuthorityDiscoveryConfig;
use qchain_template_runtime::constants::currency::DOLLARS;
use sp_consensus_babe::AuthorityId as BabeId;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use qchain_template_runtime::opaque::SessionKeys;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sc_chain_spec::ChainSpecExtension;
use sp_core::crypto::AccountId32;

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

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

/// Specialized `ChainSpec` for development.
pub type DevChainSpec = sc_service::GenericChainSpec<DevGenesisExt, Extensions>;

/// Extension for the dev genesis config to support a custom changes to the genesis state.
#[derive(Serialize, Deserialize)]
pub struct DevGenesisExt {
	/// Genesis config.
	genesis_config: GenesisConfig,
	/// The flag that if enable manual-seal mode.
	enable_manual_seal: Option<bool>,
}

impl sp_runtime::BuildStorage for DevGenesisExt {
	fn assimilate_storage(&self, storage: &mut Storage) -> Result<(), String> {
		BasicExternalities::execute_with_storage(storage, || {
			if let Some(enable_manual_seal) = &self.enable_manual_seal {
				EnableManualSeal::set(enable_manual_seal);
			}
		});
		self.genesis_config.assimilate_storage(storage)
	}
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
	seed: &str,
) -> (AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<AuthorityDiscoveryId>(seed),
	)
}

fn session_keys(
	grandpa: GrandpaId,
	babe: BabeId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
	SessionKeys { grandpa, babe, im_online, authority_discovery }
}

pub fn development_config(enable_manual_seal: Option<bool>) -> DevChainSpec {
	let wasm_binary = WASM_BINARY.expect("WASM not available");

	DevChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			DevGenesisExt {
				genesis_config: testnet_genesis(
					wasm_binary,
					// Sudo account
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					// Pre-funded accounts
					vec![
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
						get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					],

					// Initial PoA authorities
					vec![authority_keys_from_seed("Alice")],
					vec![],
					42,
				),
				enable_manual_seal,
			}
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		None,
		// Extensions
		Default::default(),
	)
}

pub fn local_testnet_config() -> ChainSpec {
	let wasm_binary = WASM_BINARY.expect("WASM not available");

	ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
				vec![
					authority_keys_from_seed("Alice"),
					authority_keys_from_seed("Bob"),
				],
				vec![],
				42,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		None,
		// Extensions
		Default::default(),
	)
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	sudo_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,
	)>,
	initial_nominators: Vec<AccountId>,
	chain_id: u64,
) -> GenesisConfig {
	use qchain_template_runtime::{
		BabeConfig, BalancesConfig, EVMChainIdConfig, EVMConfig, GrandpaConfig, SudoConfig,
		SystemConfig,
	};

	let mut rng = rand::thread_rng();

	const ENDOWMENT: Balance = 10_000_000 * DOLLARS;
	const STASH: Balance = ENDOWMENT / 1000;
	let stakers = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
		.chain(initial_nominators.iter().map(|x| {
			use rand::{seq::SliceRandom, Rng};
			let limit = (MaxNominations::get() as usize).min(initial_authorities.len());
			let count = rng.gen::<usize>() % limit;
			let nominations = initial_authorities
				.as_slice()
				.choose_multiple(&mut rng, count)
				.into_iter()
				.map(|choice| choice.0.clone())
				.collect::<Vec<_>>();
			(x.clone(), x.clone(), STASH, StakerStatus::Nominator(nominations))
		}))
		.collect::<Vec<_>>();

	GenesisConfig {
		// System
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(sudo_key),
		},

		// Monetary
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1 << 60))
				.collect(),
		},
		transaction_payment: Default::default(),

		grandpa: GrandpaConfig { authorities: vec![] },
//212 53 147
// 167 131 159 191 202 109 161 41 255 158 47 245 33 17 91 126 180 33 59 33 80 134 252 52 22 199 163 64 233 68 204 73
// 7374
		// EVM compatibility
		evm_chain_id: EVMChainIdConfig { chain_id },
		evm: EVMConfig {
			accounts: {
				let mut map = BTreeMap::new();
				map.insert(
					H160::from_str("be5ddb1579b72e84524fc29e78609e3caf42e85a")
						.expect("internal H160 is valid; qed"),
					fp_evm::GenesisAccount {
						balance: U256::from_str("3B05FCC548B94B056680000")
							.expect("internal U256 is valid; qed"),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);
				map.insert(
					H160::from_str("d43593c715fdd31c61141abd04a99fd6822c8558")
						.expect("internal H160 is valid; qed"),
					fp_evm::GenesisAccount {
						balance: U256::from_str("3B05FCC548B94B056680000")
							.expect("internal U256 is valid; qed"),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);
				// Core team: 1 141 678 367,76 QDT
				map.insert(
					H160::from_str("8eaf04151687736326c9fea17e25fc5287613693")
						.expect("internal H160 is valid; qed"),
					fp_evm::GenesisAccount {
						balance: U256::from_str("3B05FCC548B94B056680000")
							.expect("internal U256 is valid; qed"),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);
				// Advisors: 434 925 092,48 QDT
				map.insert(
					H160::from_str("6be02d1d3665660d22ff9624b7be0551ee1ac91b")
						.expect("internal H160 is valid; qed"),
					fp_evm::GenesisAccount {
						balance: U256::from_str("167C2F881BB44D576400000")
							.expect("internal U256 is valid; qed"),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);

				map.insert(
					H160::from_str("0Ebb2E8151b555fEBd4750300ffed34F32b8024d")
						.expect("internal H160 is valid; qed"),
					fp_evm::GenesisAccount {
						balance: U256::from_str("167C2F881BB44D576400000")
							.expect("internal U256 is valid; qed"),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);

				// Private sale: 190 279 727,96 QDT
				map.insert(
					H160::from_str("9921Ea2077972B51950496EFa02e68F0ad2bc4D6")
						.expect("internal H160 is valid; qed"),
					fp_evm::GenesisAccount {
						balance: U256::from_str("9D654CB8C1EE1D63BC0000")
							.expect("internal U256 is valid; qed"),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);
				// Public sale: 951 398 639,8 QDT
				map.insert(
					H160::from_str("0b54EfF6833c7DF3520A45a264FA59a0e8011a97")
						.expect("internal H160 is valid; qed"),
					fp_evm::GenesisAccount {
						balance: U256::from_str("312FA7F9BC9A692F2AC0000")
							.expect("internal U256 is valid; qed"),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);
				map
			},
		},
		ethereum: Default::default(),
		dynamic_fee: Default::default(),
		base_fee: Default::default(),
		authority_discovery: AuthorityDiscoveryConfig { keys: vec![] },
		babe: BabeConfig {
			authorities: vec![],
			epoch_config: Some(qchain_template_runtime::BABE_GENESIS_EPOCH_CONFIG),
		},
		council: CouncilConfig::default(),
		im_online: ImOnlineConfig { keys: vec![] },
		nomination_pools: NominationPoolsConfig {
			min_create_bond: 10 * DOLLARS,
			min_join_bond: 1 * DOLLARS,
			..Default::default()
		},
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()),
					)
				})
				.collect::<Vec<_>>(),
		},
		staking: StakingConfig {
			validator_count: initial_authorities.len() as u32,
			minimum_validator_count: initial_authorities.len() as u32,
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			stakers,
			..Default::default()
		},
		treasury: Default::default(),
	}
}
