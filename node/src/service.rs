//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

use std::{cell::RefCell, sync::Arc, time::Duration};
use fp_rpc::{ConvertTransactionRuntimeApi, EthereumRuntimeRPCApi};
use frame_system_rpc_runtime_api::AccountNonceApi;

use futures::{channel::mpsc, prelude::*};
use pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi;
// Substrate
use prometheus_endpoint::Registry;
use sc_client_api::{Backend, BlockBackend, BlockImportOperation, StateBackendFor};
use sc_consensus::BasicQueue;
use sc_executor::{NativeElseWasmExecutor, NativeExecutionDispatch};
use sc_network_common::service::NetworkEventStream;
use sc_network_common::protocol::event::Event;
use sc_network_common::sync::warp::WarpSyncParams;
use sc_service::{error::Error as ServiceError, Configuration, PartialComponents, TaskManager, GenesisBlockBuilder, new_db_backend, BuildGenesisBlock, resolve_state_version_from_wasm};
use sc_telemetry::{Telemetry, TelemetryHandle, TelemetryWorker};
use sp_api::{ConstructRuntimeApi, TransactionFor};
use sp_consensus_aura::sr25519::AuthorityPair as AuraPair;
use sp_consensus_babe::BabeApi;
use sc_consensus_babe::{self, SlotProportion};
use sp_core::U256;
use sp_finality_grandpa::GrandpaApi;
use sp_runtime::traits::BlakeTwo256;
use sp_trie::PrefixedMemoryDB;
// Runtime
use qchain_template_runtime::{opaque::Block, Hash, TransactionConverter, AccountId, Index, Balance};

use crate::{
	cli::Sealing,
	client::{BaseRuntimeApiCollection, FullBackend, FullClient, RuntimeApiCollection},
	eth::{
		new_frontier_partial, spawn_frontier_tasks, FrontierBackend, FrontierBlockImport,
		FrontierPartialComponents,
	},
};
pub use crate::{
	client::{Client, TemplateRuntimeExecutor},
	eth::{db_config_dir, EthConfiguration},
};
use crate::rpc::EthDeps;

type BasicImportQueue<Client> = sc_consensus::DefaultImportQueue<Block, Client>;
type FullPool<Client> = sc_transaction_pool::FullPool<Block, Client>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

type GrandpaBlockImport<Client> =
	sc_finality_grandpa::GrandpaBlockImport<FullBackend, Block, Client, FullSelectChain>;
type GrandpaLinkHalf<Client> = sc_finality_grandpa::LinkHalf<Block, Client, FullSelectChain>;
type BoxBlockImport<Client> = sc_consensus::BoxBlockImport<Block, TransactionFor<Client, Block>>;

pub fn new_partial<RuntimeApi, Executor>(
	config: &Configuration,
) -> Result<
	PartialComponents<
		FullClient<RuntimeApi, Executor>,
		FullBackend,
		FullSelectChain,
		BasicImportQueue<FullClient<RuntimeApi, Executor>>,
		FullPool<FullClient<RuntimeApi, Executor>>,
		(
			Option<Telemetry>,
			// BoxBlockImport<FullClient<RuntimeApi, Executor>>,
			sc_consensus_babe::BabeBlockImport<Block, FullClient<RuntimeApi, Executor>, GrandpaBlockImport<FullClient<RuntimeApi, Executor>>,>,
			GrandpaLinkHalf<FullClient<RuntimeApi, Executor>>,
			Arc<FrontierBackend>,
			sc_consensus_babe::BabeLink<Block>,
		),
	>,
	ServiceError,
>
where
	RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>,
	RuntimeApi: Send + Sync + 'static,
	RuntimeApi::RuntimeApi:
		BaseRuntimeApiCollection<StateBackend = StateBackendFor<FullBackend, Block>>
		+ BabeApi<Block> + GrandpaApi<Block> + AccountNonceApi<Block, AccountId, Index>
		+ TransactionPaymentRuntimeApi<Block, Balance> + ConvertTransactionRuntimeApi<Block>
		+ EthereumRuntimeRPCApi<Block>,
	Executor: NativeExecutionDispatch + 'static,
{
	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> Result<_, sc_telemetry::Error> {
			let worker = TelemetryWorker::new(16)?;
			let telemetry = worker.handle().new_telemetry(endpoints);
			Ok((worker, telemetry))
		})
		.transpose()?;

	let executor = NativeElseWasmExecutor::<Executor>::new(
		config.wasm_method,
		config.default_heap_pages,
		config.max_runtime_instances,
		config.runtime_cache_size,
	);
	{
		use sp_api::BlockT;
		use sp_api::HeaderT;
		let backend = new_db_backend::<Block>(config.db_config())?;

		{
			let genesis_block_builder = GenesisBlockBuilder::new(
				config.chain_spec.as_storage_builder(),
				!config.no_genesis(),
				backend.clone(),
				executor.clone(),
			)?;

			let genesis_storage = config.chain_spec.as_storage_builder()
				.build_storage()
				.map_err(sp_blockchain::Error::Storage)?;

			let genesis_state_version =
				resolve_state_version_from_wasm(&genesis_storage, &executor)?;
			let mut op = backend.begin_operation()?;
			let state_root =
				op.set_genesis_state(genesis_storage, !config.no_genesis(), genesis_state_version).unwrap();

			log::info!("State root: {:?}, state version: {:?}", state_root, genesis_state_version);
		}

		let genesis_block_builder = GenesisBlockBuilder::new(
			config.chain_spec.as_storage_builder(),
			!config.no_genesis(),
			backend.clone(),
			executor.clone(),
		)?;
		let genesis_block = genesis_block_builder.build_genesis_block().unwrap().0;

		log::info!("Genesis block: {:?}", genesis_block);
		log::info!("Gen block: {:?} {:?}", genesis_block.header().state_root(), genesis_block.header().hash());
	}

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, _>(
			config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
		)?;
	let client = Arc::new(client);

	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager
			.spawn_handle()
			.spawn("telemetry", None, worker.run());
		telemetry
	});

	let select_chain = sc_consensus::LongestChain::new(backend.clone());
	let (grandpa_block_import, grandpa_link) = sc_finality_grandpa::block_import(
		client.clone(),
		&client,
		select_chain.clone(),
		telemetry.as_ref().map(|x| x.handle()),
	)?;
	let justification_import = grandpa_block_import.clone();
	let (block_import, babe_link) = sc_consensus_babe::block_import(
		sc_consensus_babe::configuration(&*client)?,
		grandpa_block_import,
		client.clone(),
	)?;


	let frontier_backend = Arc::new(FrontierBackend::open(
		client.clone(),
		&config.database,
		&db_config_dir(config),
	)?);
	let slot_duration = babe_link.config().slot_duration();
	let import_queue = sc_consensus_babe::import_queue(
		babe_link.clone(),
		block_import.clone(),
		Some(Box::new(justification_import)),
		client.clone(),
		select_chain.clone(),
		move |_, ()| async move {
			let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

			let slot =
				sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
					*timestamp,
					slot_duration,
				);

			Ok((slot, timestamp))
		},
		&task_manager.spawn_essential_handle(),
		config.prometheus_registry(),
		telemetry.as_ref().map(|x| x.handle()),
	)?;


	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_essential_handle(),
		client.clone(),
	);


	Ok(PartialComponents {
		client,
		backend,
		keystore_container,
		task_manager,
		select_chain,
		import_queue,
		transaction_pool,
		other: (telemetry, block_import, grandpa_link, frontier_backend, babe_link),
	})
}

/// Build the import queue for the template runtime (manual seal).
pub fn build_manual_seal_import_queue<RuntimeApi, Executor>(
	client: Arc<FullClient<RuntimeApi, Executor>>,
	config: &Configuration,
	_eth_config: &EthConfiguration,
	task_manager: &TaskManager,
	_telemetry: Option<TelemetryHandle>,
	_grandpa_block_import: GrandpaBlockImport<FullClient<RuntimeApi, Executor>>,
	frontier_backend: Arc<FrontierBackend>,
) -> Result<
	(
		BasicImportQueue<FullClient<RuntimeApi, Executor>>,
		BoxBlockImport<FullClient<RuntimeApi, Executor>>,
	),
	ServiceError,
>
where
	RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>,
	RuntimeApi: Send + Sync + 'static,
	RuntimeApi::RuntimeApi:
		RuntimeApiCollection<StateBackend = StateBackendFor<FullBackend, Block>>,
	Executor: NativeExecutionDispatch + 'static,
{
	let frontier_block_import = FrontierBlockImport::new(client.clone(), client, frontier_backend);
	Ok((
		sc_consensus_manual_seal::import_queue(
			Box::new(frontier_block_import.clone()),
			&task_manager.spawn_essential_handle(),
			config.prometheus_registry(),
		),
		Box::new(frontier_block_import),
	))
}

/// Builds a new service for a full client.
pub fn new_full
// <RuntimeApi, Executor>
(
	mut config: Configuration,
	eth_config: EthConfiguration,
	sealing: Option<Sealing>,
) -> Result<TaskManager, ServiceError>
// where
// 	RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>,
// 	RuntimeApi: Send + Sync + 'static,
// 	RuntimeApi::RuntimeApi:
// 		RuntimeApiCollection<StateBackend = StateBackendFor<FullBackend, Block>> +
// 	AccountNonceApi<Block, AccountId, Index>,
// 	Executor: NativeExecutionDispatch + 'static,
{
	// let build_import_queue = if sealing.is_some() {
	// 	build_manual_seal_import_queue::<RuntimeApi, Executor>
	// } else {
	// 	build_aura_grandpa_import_queue::<RuntimeApi, Executor>
	// };

	let PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		other: (mut telemetry, block_import, grandpa_link, frontier_backend, babe_link),
	} = new_partial::<qchain_template_runtime::RuntimeApi, TemplateRuntimeExecutor>(&config)?;

	let FrontierPartialComponents {
		filter_pool,
		fee_history_cache,
		fee_history_cache_limit,
	} = new_frontier_partial(&eth_config)?;

	let grandpa_protocol_name = sc_finality_grandpa::protocol_standard_name(
		&client.block_hash(0)?.expect("Genesis block exists; qed"),
		&config.chain_spec,
	);


	let warp_sync_params = if sealing.is_some() {
		None
	} else {
		config
			.network
			.extra_sets
			.push(sc_finality_grandpa::grandpa_peers_set_config(
				grandpa_protocol_name.clone(),
			));
		let warp_sync: Arc<dyn sc_network::config::WarpSyncProvider<Block>> =
			Arc::new(sc_finality_grandpa::warp_proof::NetworkProvider::new(
				backend.clone(),
				grandpa_link.shared_authority_set().clone(),
				Vec::default(),
			));
		Some(WarpSyncParams::WithProvider(warp_sync))
	};

	let (network, system_rpc_tx, tx_handler_controller, network_starter) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			block_announce_validator_builder: None,
			warp_sync_params,
		})?;

	if config.offchain_worker.enabled {
		sc_service::build_offchain_workers(
			&config,
			task_manager.spawn_handle(),
			client.clone(),
			network.clone(),
		);
	}

	let role = config.role.clone();
	let force_authoring = config.force_authoring;
	let backoff_authoring_blocks =
		Some(sc_consensus_slots::BackoffAuthoringOnFinalizedHeadLagging::default());
	let name = config.network.node_name.clone();
	let enable_grandpa = !config.disable_grandpa && sealing.is_none();
	let prometheus_registry = config.prometheus_registry().cloned();

	// Channel for the rpc handler to communicate with the authorship task.
	// let (command_sink, commands_stream) = mpsc::channel(1000);


	// if let Some(hwbench) = hwbench {
	// 	sc_sysinfo::print_hwbench(&hwbench);
	// 	if !SUBSTRATE_REFERENCE_HARDWARE.check_hardware(&hwbench) && role.is_authority() {
	// 		log::warn!(
	// 			"⚠️  The hardware does not meet the minimal requirements for role 'Authority'."
	// 		);
	// 	}
	//
	// 	if let Some(ref mut telemetry) = telemetry {
	// 		let telemetry_handle = telemetry.handle();
	// 		task_manager.spawn_handle().spawn(
	// 			"telemetry_hwbench",
	// 			None,
	// 			sc_sysinfo::initialize_hwbench_telemetry(telemetry_handle, hwbench),
	// 		);
	// 	}
	// }



	// for ethereum-compatibility rpc.
	config.rpc_id_provider = Some(Box::new(fc_rpc::EthereumSubIdProvider));
	let overrides = crate::rpc::overrides_handle(client.clone());
	let eth_rpc_params = crate::rpc::EthDeps {
		client: client.clone(),
		pool: transaction_pool.clone(),
		graph: transaction_pool.pool().clone(),
		converter: Some(TransactionConverter),
		is_authority: config.role.is_authority(),
		enable_dev_signer: eth_config.enable_dev_signer,
		network: network.clone(),
		frontier_backend: frontier_backend.clone(),
		overrides: overrides.clone(),
		block_data_cache: Arc::new(fc_rpc::EthBlockDataCacheTask::new(
			task_manager.spawn_handle(),
			overrides.clone(),
			eth_config.eth_log_block_cache,
			eth_config.eth_statuses_cache,
			prometheus_registry.clone(),
		)),
		filter_pool: filter_pool.clone(),
		max_past_logs: eth_config.max_past_logs,
		fee_history_cache: fee_history_cache.clone(),
		fee_history_cache_limit,
		execute_gas_limit_multiplier: eth_config.execute_gas_limit_multiplier,
	};


	let (rpc_builder, rpc_setup) = {

		let justification_stream = grandpa_link.justification_stream();
		let shared_authority_set = grandpa_link.shared_authority_set().clone();
		let shared_voter_state = sc_finality_grandpa::SharedVoterState::empty();
		let shared_voter_state2 = shared_voter_state.clone();

		let finality_proof_provider = sc_finality_grandpa::FinalityProofProvider::new_for_service(
			backend.clone(),
			Some(shared_authority_set.clone()),
		);

		let babe_config = babe_link.config().clone();
		let shared_epoch_changes = babe_link.epoch_changes().clone();

		let client = client.clone();
		let pool = transaction_pool.clone();
		let select_chain = select_chain.clone();
		let keystore = keystore_container.sync_keystore();
		let chain_spec = config.chain_spec.cloned_box();

		let rpc_backend = backend.clone();
		let rpc_extensions_builder = move |deny_unsafe, subscription_executor: sc_rpc::SubscriptionTaskExecutor| {
			let deps = crate::rpc::FullDeps {
				client: client.clone(),
				pool: pool.clone(),
				select_chain: select_chain.clone(),
				chain_spec: chain_spec.cloned_box(),
				deny_unsafe,
				babe: crate::rpc::BabeDeps {
					babe_config: babe_config.clone(),
					shared_epoch_changes: shared_epoch_changes.clone(),
					keystore: keystore.clone(),
				},
				grandpa: crate::rpc::GrandpaDeps {
					shared_voter_state: shared_voter_state.clone(),
					shared_authority_set: shared_authority_set.clone(),
					justification_stream: justification_stream.clone(),
					subscription_executor: subscription_executor.clone(),
					finality_provider: finality_proof_provider.clone(),
				},
				command_sink: None,
				eth: eth_rpc_params.clone(),
			};

			crate::rpc::create_full(deps, subscription_executor, rpc_backend.clone()).map_err(Into::into)
		};

		(rpc_extensions_builder, shared_voter_state2)
	};


	// let rpc_builder = {
	// 	let client = client.clone();
	// 	let pool = transaction_pool.clone();
	//
	// 	Box::new(move |deny_unsafe, subscription_task_executor| {
	// 		let deps = crate::rpc::FullDeps {
	// 			client: client.clone(),
	// 			pool: pool.clone(),
	// 			deny_unsafe,
	// 			command_sink: if sealing.is_some() {
	// 				Some(command_sink.clone())
	// 			} else {
	// 				None
	// 			},
	// 			eth: eth_rpc_params.clone(),
	// 		};
	//
	// 		crate::rpc::create_full(deps, subscription_task_executor).map_err(Into::into)
	// 	})
	// };

	let auth_disc_publish_non_global_ips = config.network.allow_non_globals_in_dht;

	let rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		config,
		client: client.clone(),
		backend: backend.clone(),
		task_manager: &mut task_manager,
		keystore: keystore_container.sync_keystore(),
		transaction_pool: transaction_pool.clone(),
		rpc_builder: Box::new(rpc_builder),
		network: network.clone(),
		system_rpc_tx,
		tx_handler_controller,
		telemetry: telemetry.as_mut(),
	})?;

	spawn_frontier_tasks(
		&task_manager,
		client.clone(),
		backend,
		frontier_backend,
		filter_pool,
		overrides,
		fee_history_cache,
		fee_history_cache_limit,
	);

	if let sc_service::config::Role::Authority { .. } = &role {
		let proposer = sc_basic_authorship::ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool.clone(),
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|x| x.handle()),
		);

		let client_clone = client.clone();
		let slot_duration = babe_link.config().slot_duration();
		let babe_config = sc_consensus_babe::BabeParams {
			keystore: keystore_container.sync_keystore(),
			client: client.clone(),
			select_chain,
			env: proposer,
			block_import,
			sync_oracle: network.clone(),
			justification_sync_link: network.clone(),
			create_inherent_data_providers: move |parent, ()| {
				let client_clone = client_clone.clone();
				async move {
					let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

					let slot =
						sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
							*timestamp,
							slot_duration,
						);

					let storage_proof =
						sp_transaction_storage_proof::registration::new_data_provider(
							&*client_clone,
							&parent,
						)?;

					Ok((slot, timestamp, storage_proof))
				}
			},
			force_authoring,
			backoff_authoring_blocks,
			babe_link,
			block_proposal_slot_portion: SlotProportion::new(0.5),
			max_block_proposal_slot_portion: None,
			telemetry: telemetry.as_ref().map(|x| x.handle()),
		};

		let babe = sc_consensus_babe::start_babe(babe_config)?;
		task_manager.spawn_essential_handle().spawn_blocking(
			"babe-proposer",
			Some("block-authoring"),
			babe,
		);
	}

	// Spawn authority discovery module.
	if role.is_authority() {
		let authority_discovery_role =
			sc_authority_discovery::Role::PublishAndDiscover(keystore_container.keystore());
		let dht_event_stream =
			network.event_stream("authority-discovery").filter_map(|e| async move {
				match e {
					Event::Dht(e) => Some(e),
					_ => None,
				}
			});
		let (authority_discovery_worker, _service) =
			sc_authority_discovery::new_worker_and_service_with_config(
				sc_authority_discovery::WorkerConfig {
					publish_non_global_ips: auth_disc_publish_non_global_ips,
					..Default::default()
				},
				client.clone(),
				network.clone(),
				Box::pin(dht_event_stream),
				authority_discovery_role,
				prometheus_registry.clone(),
			);

		task_manager.spawn_handle().spawn(
			"authority-discovery-worker",
			Some("networking"),
			authority_discovery_worker.run(),
		);
	}

	let keystore =
		if role.is_authority() { Some(keystore_container.sync_keystore()) } else { None };

	let config = sc_finality_grandpa::Config {
		// FIXME #1578 make this available through chainspec
		gossip_duration: std::time::Duration::from_millis(333),
		justification_period: 512,
		name: Some(name),
		observer_enabled: false,
		keystore,
		local_role: role,
		telemetry: telemetry.as_ref().map(|x| x.handle()),
		protocol_name: grandpa_protocol_name,
	};
	let shared_voter_state = rpc_setup;

	if enable_grandpa {
		// start the full GRANDPA voter
		// NOTE: non-authorities could run the GRANDPA observer protocol, but at
		// this point the full voter should provide better guarantees of block
		// and vote data availability than the observer. The observer has not
		// been tested extensively yet and having most nodes in a network run it
		// could lead to finality stalls.
		let grandpa_config = sc_finality_grandpa::GrandpaParams {
			config,
			link: grandpa_link,
			network: network.clone(),
			telemetry: telemetry.as_ref().map(|x| x.handle()),
			voting_rule: sc_finality_grandpa::VotingRulesBuilder::default().build(),
			prometheus_registry,
			shared_voter_state,
		};

		// the GRANDPA voter task is considered infallible, i.e.
		// if it fails we take down the service with it.
		task_manager.spawn_essential_handle().spawn_blocking(
			"grandpa-voter",
			None,
			sc_finality_grandpa::run_grandpa_voter(grandpa_config)?,
		);
	}

	network_starter.start_network();
	Ok(task_manager)
}

fn run_manual_seal_authorship<RuntimeApi, Executor>(
	eth_config: &EthConfiguration,
	sealing: Sealing,
	client: Arc<FullClient<RuntimeApi, Executor>>,
	transaction_pool: Arc<FullPool<FullClient<RuntimeApi, Executor>>>,
	select_chain: FullSelectChain,
	block_import: BoxBlockImport<FullClient<RuntimeApi, Executor>>,
	task_manager: &TaskManager,
	prometheus_registry: Option<&Registry>,
	telemetry: Option<&Telemetry>,
	commands_stream: mpsc::Receiver<sc_consensus_manual_seal::rpc::EngineCommand<Hash>>,
) -> Result<(), ServiceError>
where
	RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>>,
	RuntimeApi: Send + Sync + 'static,
	RuntimeApi::RuntimeApi:
		RuntimeApiCollection<StateBackend = StateBackendFor<FullBackend, Block>>,
	Executor: NativeExecutionDispatch + 'static,
{
	let proposer_factory = sc_basic_authorship::ProposerFactory::new(
		task_manager.spawn_handle(),
		client.clone(),
		transaction_pool.clone(),
		prometheus_registry,
		telemetry.as_ref().map(|x| x.handle()),
	);

	thread_local!(static TIMESTAMP: RefCell<u64> = RefCell::new(0));

	/// Provide a mock duration starting at 0 in millisecond for timestamp inherent.
	/// Each call will increment timestamp by slot_duration making Aura think time has passed.
	struct MockTimestampInherentDataProvider;

	#[async_trait::async_trait]
	impl sp_inherents::InherentDataProvider for MockTimestampInherentDataProvider {
		async fn provide_inherent_data(
			&self,
			inherent_data: &mut sp_inherents::InherentData,
		) -> Result<(), sp_inherents::Error> {
			TIMESTAMP.with(|x| {
				*x.borrow_mut() += qchain_template_runtime::SLOT_DURATION;
				inherent_data.put_data(sp_timestamp::INHERENT_IDENTIFIER, &*x.borrow())
			})
		}

		async fn try_handle_error(
			&self,
			_identifier: &sp_inherents::InherentIdentifier,
			_error: &[u8],
		) -> Option<Result<(), sp_inherents::Error>> {
			// The pallet never reports error.
			None
		}
	}

	let target_gas_price = eth_config.target_gas_price;
	let create_inherent_data_providers = move |_, ()| async move {
		let timestamp = MockTimestampInherentDataProvider;
		let dynamic_fee = fp_dynamic_fee::InherentDataProvider(U256::from(target_gas_price));
		Ok((timestamp, dynamic_fee))
	};

	let manual_seal = match sealing {
		Sealing::Manual => future::Either::Left(sc_consensus_manual_seal::run_manual_seal(
			sc_consensus_manual_seal::ManualSealParams {
				block_import,
				env: proposer_factory,
				client,
				pool: transaction_pool,
				commands_stream,
				select_chain,
				consensus_data_provider: None,
				create_inherent_data_providers,
			},
		)),
		Sealing::Instant => future::Either::Right(sc_consensus_manual_seal::run_instant_seal(
			sc_consensus_manual_seal::InstantSealParams {
				block_import,
				env: proposer_factory,
				client,
				pool: transaction_pool,
				select_chain,
				consensus_data_provider: None,
				create_inherent_data_providers,
			},
		)),
	};

	// we spawn the future on a background thread managed by service.
	task_manager
		.spawn_essential_handle()
		.spawn_blocking("manual-seal", None, manual_seal);
	Ok(())
}

pub fn build_full(
	config: Configuration,
	eth_config: EthConfiguration,
	sealing: Option<Sealing>,
) -> Result<TaskManager, ServiceError> {
	new_full(
		config, eth_config, sealing,
	)
}

pub fn new_chain_ops(
	mut config: &mut Configuration,
	eth_config: &EthConfiguration,
) -> Result<
	(
		Arc<Client>,
		Arc<FullBackend>,
		BasicQueue<Block, PrefixedMemoryDB<BlakeTwo256>>,
		TaskManager,
		Arc<FrontierBackend>,
	),
	ServiceError,
> {
	// config.keystore = sc_service::config::KeystoreConfig::InMemory;
	// let PartialComponents {
	// 	client,
	// 	backend,
	// 	import_queue,
	// 	task_manager,
	// 	other,
	// 	..
	// } = new_partial::<qchain_template_runtime::RuntimeApi, TemplateRuntimeExecutor, _>(
	// 	config,
	// )?;
	// Ok((client, backend, import_queue, task_manager, other.3))
	unimplemented!()
}
