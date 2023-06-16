//! A collection of node-specific RPC methods.

use std::sync::Arc;
use node_primitives::BlockNumber;
use futures::channel::mpsc;
use jsonrpsee::RpcModule;
// Substrate
use sc_client_api::{AuxStore, backend::{Backend, StorageProvider}, client::BlockchainEvents};
use sc_consensus_manual_seal::rpc::EngineCommand;
use sc_finality_grandpa::{FinalityProofProvider, GrandpaJustificationStream, SharedAuthoritySet, SharedVoterState};
use sc_rpc::SubscriptionTaskExecutor;
use sc_rpc_api::DenyUnsafe;
use sc_service::TransactionPool;
use sc_transaction_pool::ChainApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sc_consensus_babe::{BabeConfiguration, Epoch};
use sc_consensus_epochs::SharedEpochChanges;
use sp_keystore::SyncCryptoStorePtr;
use sp_runtime::traits::Block as BlockT;
// Runtime
use qchain_template_runtime::{opaque::Block, AccountId, Balance, Hash, Index};
use crate::chain_spec::ChainSpec;
use sc_rpc_spec_v2::chain_spec::{ChainSpec as RpcChainSpec, ChainSpecApiServer};
use sc_consensus_babe_rpc::{Babe, BabeApiServer};
use sp_consensus_babe::BabeApi;
use sc_finality_grandpa_rpc::{Grandpa, GrandpaApiServer};
mod eth;
pub use self::eth::{create_eth, overrides_handle, EthDeps};
use sc_sync_state_rpc::{SyncState, SyncStateApiServer};
use sc_rpc::dev::{Dev, DevApiServer};
use substrate_state_trie_migration_rpc::{StateMigration, StateMigrationApiServer};
use mmr_rpc::{Mmr, MmrApiServer};
use sp_consensus::SelectChain;

/// Extra dependencies for BABE.
pub struct BabeDeps {
	/// BABE protocol config.
	pub babe_config: BabeConfiguration,
	/// BABE pending epoch changes.
	pub shared_epoch_changes: SharedEpochChanges<Block, Epoch>,
	/// The keystore that manages the keys of the node.
	pub keystore: SyncCryptoStorePtr,
}

/// Extra dependencies for GRANDPA
pub struct GrandpaDeps<B> {
	/// Voting round info.
	pub shared_voter_state: SharedVoterState,
	/// Authority set info.
	pub shared_authority_set: SharedAuthoritySet<Hash, BlockNumber>,
	/// Receives notifications about justification events from Grandpa.
	pub justification_stream: GrandpaJustificationStream<Block>,
	/// Executor to drive the subscription manager in the Grandpa RPC handler.
	pub subscription_executor: SubscriptionTaskExecutor,
	/// Finality proof provider.
	pub finality_provider: Arc<FinalityProofProvider<B, Block>>,
}


/// Full client dependencies.
pub struct FullDeps<C, P, CT, SC, B, A: ChainApi> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
	/// Manual seal command sink
	pub command_sink: Option<mpsc::Sender<EngineCommand<Hash>>>,
	/// Ethereum-compatibility specific dependencies.
	pub eth: EthDeps<C, P, A, CT, Block>,

	pub select_chain: SC,
	pub chain_spec: Box<dyn sc_chain_spec::ChainSpec>,
	/// BABE specific dependencies.
	pub babe: BabeDeps,
	/// GRANDPA specific dependencies.
	pub grandpa: GrandpaDeps<B>,
}

/// Instantiate all Full RPC extensions.
pub fn create_full<C, P, BE, CT, SC, A: ChainApi>(
	deps: FullDeps<C, P, CT, SC, BE, A>,
	subscription_task_executor: SubscriptionTaskExecutor,
	backend: Arc<BE>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
	C: ProvideRuntimeApi<Block>
		+ sc_client_api::BlockBackend<Block>
		+ HeaderBackend<Block>
		+ AuxStore
		+ HeaderMetadata<Block, Error = BlockChainError>
		+ Sync
		+ Send
		+ 'static,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
	C::Api: sp_block_builder::BlockBuilder<Block>,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: fp_rpc::ConvertTransactionRuntimeApi<Block>,
	C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,
	// C::Api: mmr_rpc::MmrRuntimeApi<Block, <Block as sp_runtime::traits::Block>::Hash, BlockNumber>,
	C::Api: BabeApi<Block>,
	SC: SelectChain<Block> + 'static,
	C: BlockchainEvents<Block> + 'static,
	C: HeaderBackend<Block>
		+ HeaderMetadata<Block, Error = BlockChainError>
		+ StorageProvider<Block, BE>,
	BE: Backend<Block> + 'static,
	P: TransactionPool<Block = Block> + 'static,
	A: ChainApi<Block = Block> + 'static,
	CT: fp_rpc::ConvertTransaction<<Block as BlockT>::Extrinsic> + Send + Sync + 'static,
{
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
	use sc_consensus_manual_seal::rpc::{ManualSeal, ManualSealApiServer};
	use substrate_frame_rpc_system::{System, SystemApiServer};

	let mut io = RpcModule::new(());
	let FullDeps {
		client,
		pool,
		deny_unsafe,
		command_sink,
		eth, select_chain, chain_spec, babe, grandpa,
	} = deps;

	let BabeDeps { keystore, babe_config, shared_epoch_changes } = babe;
	let GrandpaDeps {
		shared_voter_state,
		shared_authority_set,
		justification_stream,
		subscription_executor,
		finality_provider,
	} = grandpa;

	let chain_name = chain_spec.name().to_string();
	let genesis_hash = client.block_hash(0).ok().flatten().expect("Genesis block exists; qed");
	let properties = chain_spec.properties();
	io.merge(RpcChainSpec::new(chain_name, genesis_hash, properties).into_rpc())?;

	io.merge(System::new(client.clone(), pool, deny_unsafe).into_rpc())?;
	// io.merge(Mmr::new(client.clone()).into_rpc())?;
	io.merge(TransactionPayment::new(client.clone()).into_rpc())?;

	io.merge(
		Babe::new(
			client.clone(),
			shared_epoch_changes.clone(),
			keystore,
			babe_config,
			select_chain,
			deny_unsafe,
		)
			.into_rpc(),
	)?;
	io.merge(
		Grandpa::new(
			subscription_executor,
			shared_authority_set.clone(),
			shared_voter_state,
			justification_stream,
			finality_provider,
		)
			.into_rpc(),
	)?;

	if let Some(command_sink) = command_sink {
		io.merge(
			// We provide the rpc handler with the sending end of the channel to allow the rpc
			// send EngineCommands to the background block authorship task.
			ManualSeal::new(command_sink).into_rpc(),
		)?;
	}

	io.merge(
		SyncState::new(chain_spec, client.clone(), shared_authority_set, shared_epoch_changes)?
			.into_rpc(),
	)?;

	io.merge(StateMigration::new(client.clone(), backend, deny_unsafe).into_rpc())?;
	io.merge(Dev::new(client, deny_unsafe).into_rpc())?;


	// Ethereum compatibility RPCs
	let io = create_eth::<_, _, _, _, _, _>(io, eth, subscription_task_executor)?;

	Ok(io)
}
