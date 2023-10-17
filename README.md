# Qchain node
This tutorial describes how to deploy Qchain network.

# 1 Boot node setup
The boot node starts first, and subsequently the remaining nodes will be connected to it.
Clone the repository.
```
git clone git@bitbucket.org:qchainai/node
```
Go to substrate.
```
cd substrate
```
Compiling.
```
cargo build --release
```
Generating the boot node key.
```
./target/release/qchain-template-node key generate --scheme sr25519
```
```
Secret phrase:       unfold mistake inmate surprise wire cement category lock wink brand peanut police
  Network ID:        substrate
  Secret seed:       0xf2349b7f6ae245f28537e9eaada908d9cd8641c1c9755adddde2d1fbdd00f878
  Public key (hex):  0x4213974ec796a89637ff04b3e7779cd2ccb4b6c5e175be0b06fdbdf0dbb5c2cc
  Account ID:        0x4213974ec796a89637ff04b3e7779cd2ccb4b6c5e175be0b06fdbdf0dbb5c2cc
  Public key (SS58): 5DZLqWxKiJZ2hjrQhcw1r8RRenqVdx6gzuhzo5KHjB3HfnZZ
  SS58 Address:      5DZLqWxKiJZ2hjrQhcw1r8RRenqVdx6gzuhzo5KHjB3HfnZZ
  ```
Generating a mnemonic phrase for session keys.
```
./target/release/qchain-template-node key generate --scheme sr25519
```
```
Secret phrase:       cluster truck panther color mutual coast enhance uniform twelve sibling donor trust
  Network ID:        substrate
  Secret seed:       0x3cf728f635cb00c9f9707ad0e034d3be37b7aedacb9ff1cc766c9d138289246f
  Public key (hex):  0x49547ea2c85de9a46ce58a27ce5474e6329eda94baf650bf593daaf879aec828
  Account ID:        0x49547ea2c85de9a46ce58a27ce5474e6329eda94baf650bf593daaf879aec828
  Public key (SS58): 5DirTDpgGTXFWKnvWCAdVyFbHkftoyZmHzeykQQqX1tTGfdk
  SS58 Address:      5DirTDpgGTXFWKnvWCAdVyFbHkftoyZmHzeykQQqX1tTGfdk
```
Paste the phrase into init_chain/prepare-test-net.sh.
```
SECRET="cluster truck panther color mutual coast enhance uniform twelve sibling donor trust"
```
Generating session keys for the boot node.
```
bash prepare-test-net.sh 1
```
The first two lines will contain the id of the account with which the node will start and its secret key, which will be needed to replenish the validator's balance.
```
// 5CtjdTubKvn7hwep37JzmUHxFejHfP8A59z5h4GKpAh9HiXu
 0x422ca7e726545999c56203306f58bdda71d0f659bd9c0da61650f90b66054b30

(
// 5CtjdTubKvn7hwep37JzmUHxFejHfP8A59z5h4GKpAh9HiXu
array_bytes::hex_n_into_unchecked("24a1bee3138fd67f3d1956f8c28333537d1dcd38a48257ebdbecfb77d744f741"),
// 5FCgiNgNULkdDGdBikyZakSXCaZB9Ki57L4hTBuAA4BaHr7T
array_bytes::hex_n_into_unchecked("8acad0df0ba49f4ed4777c519cbb7a3b61d5264e3f33fbd33b1a3c602a62b743"),
// 5HLjD1Jh7GDQLPkw32Y7cw5xeMLgRG537jBBHgvBNrSZcS6z
array_bytes::hex2array_unchecked("e965a3f1a6cd997b8f45cb4f21dde629442c3bea9652eb43973a612a3ad33c95").unchecked_into(),
// 5E9ctwPTAA27CYgU1NxRJYm9yTArqLgY3CoPor6azH6BEAgi
array_bytes::hex2array_unchecked("5c37fe818c78c54b43214f9d827683bd2bc49f169c1d8936b2efa4d042ae453f").unchecked_into(),
// 5CvZxtKNghYiLyX15ZcRPamBpfMFdtj3izFJyC41Lk5qmMMe
array_bytes::hex2array_unchecked("2607b35d2e5b5696bb8915b4b8043f62057d8f981a3b4254022fc8778a3bc12c").unchecked_into(),
// 5CkvjLKJE84bLzq4NiPLERAQiGQN1nuLXriqiv9BjbWk16aB
array_bytes::hex2array_unchecked("1ead27010d526a2b824669b5911868d65f653e0678b30ea928e90501af3a4e79").unchecked_into(),
),

```
We replace the session keys in node/cli/src/chain_spec.rs with the keys generated above.
```
fn staging_testnet_config_genesis() -> GenesisConfig {
	    		let initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)> = vec![
          (
// 5CtjdTubKvn7hwep37JzmUHxFejHfP8A59z5h4GKpAh9HiXu
array_bytes::hex_n_into_unchecked("24a1bee3138fd67f3d1956f8c28333537d1dcd38a48257ebdbecfb77d744f741"),
// 5FCgiNgNULkdDGdBikyZakSXCaZB9Ki57L4hTBuAA4BaHr7T
array_bytes::hex_n_into_unchecked("8acad0df0ba49f4ed4777c519cbb7a3b61d5264e3f33fbd33b1a3c602a62b743"),
// 5HLjD1Jh7GDQLPkw32Y7cw5xeMLgRG537jBBHgvBNrSZcS6z
array_bytes::hex2array_unchecked("e965a3f1a6cd997b8f45cb4f21dde629442c3bea9652eb43973a612a3ad33c95").unchecked_into(),
// 5E9ctwPTAA27CYgU1NxRJYm9yTArqLgY3CoPor6azH6BEAgi
array_bytes::hex2array_unchecked("5c37fe818c78c54b43214f9d827683bd2bc49f169c1d8936b2efa4d042ae453f").unchecked_into(),
// 5CvZxtKNghYiLyX15ZcRPamBpfMFdtj3izFJyC41Lk5qmMMe
array_bytes::hex2array_unchecked("2607b35d2e5b5696bb8915b4b8043f62057d8f981a3b4254022fc8778a3bc12c").unchecked_into(),
// 5CkvjLKJE84bLzq4NiPLERAQiGQN1nuLXriqiv9BjbWk16aB
array_bytes::hex2array_unchecked("1ead27010d526a2b824669b5911868d65f653e0678b30ea928e90501af3a4e79").unchecked_into(),
	),
          ];
	   	#--snip--
		}
```
Generate a root configuration key.
```
 ./target/release/qchain-template-node key generate --scheme sr25519
```
```
Secret phrase:       radio curious laugh tired naive horse atom marine slush claw violin leaf
  Network ID:        substrate
  Secret seed:       0x2faff86e7d6860c1d666be355b8b8b076b66b8dc434672bb4a2a8dba41ddab6b
  Public key (hex):  0x06ea535681b7737566604ef9065f4b382873a35437f606ce9911c8f064d80b2e
  Account ID:        0x06ea535681b7737566604ef9065f4b382873a35437f606ce9911c8f064d80b2e
  Public key (SS58): 5CDmkfgHkzDYakGTcKjaBvX5wBGZjg26ftT2Rgn4oD8oEL4g
  SS58 Address:      5CDmkfgHkzDYakGTcKjaBvX5wBGZjg26ftT2Rgn4oD8oEL4g
```
Change the root key in chain_spec.rs.
```
let root_key: AccountId = hex![
    // 5CDmkfgHkzDYakGTcKjaBvX5wBGZjg26ftT2Rgn4oD8oEL4g
    "06ea535681b7737566604ef9065f4b382873a35437f606ce9911c8f064d80b2e"
].into();
```
Compiling.
```
cargo build --release
```
Starting the boot node.

--node-key uses the Public key (hex) of the boot node
```
./target/release/qchain-template-node \
--chain=CustomSpec.json \
-d data/bootnode \
--name bootnode \
--port 30333 \
--ws-port 9944 \
--rpc-port 8545 \
--unsafe-ws-external \
--unsafe-rpc-external \
--rpc-methods=unsafe \
--rpc-cors=all \
--telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
--validator \
```
Local node identity will be needed later to connect the validator nodes to the boot node.
```
2023-06-06 14:22:56 Substrate Node    
2023-06-06 14:22:56 ✌️  version 3.0.0-dev-13ed4508e65    
2023-06-06 14:22:56 ❤️  by Parity Technologies <admin@parity.io>, 2017-2023    
2023-06-06 14:22:56 📋 Chain specification: Staging Testnet    
2023-06-06 14:22:56 🏷  Node name: bootnode    
2023-06-06 14:22:56 👤 Role: AUTHORITY    
2023-06-06 14:22:56 💾 Database: RocksDb at data/bootnode/chains/staging_testnet/db/full    
2023-06-06 14:22:56 ⛓  Native runtime: node-268 (substrate-node-0.tx2.au10)    
2023-06-06 14:22:59 [0] 💸 generated 1 npos voters, 1 from validators and 0 nominators    
2023-06-06 14:22:59 [0] 💸 generated 1 npos targets    
2023-06-06 14:23:00 🔨 Initializing Genesis block/state (state: 0xea69…106e, header-hash: 0x1cc0…1a93)    
2023-06-06 14:23:00 👴 Loading GRANDPA authority set from genesis on what appears to be first startup.    
2023-06-06 14:23:02 👶 Creating empty BABE epoch changes on what appears to be first startup.    
2023-06-06 14:23:02 Using default protocol ID "sup" because none is configured in the chain specs    
2023-06-06 14:23:02 🏷  Local node identity is: 12D3KooWMLaHH2ubp7kEYNGoRwWt3JQtQYnWmLiJhDcZvimVj7hm    
2023-06-06 14:23:02 💻 Operating system: linux    
2023-06-06 14:23:02 💻 CPU architecture: x86_64    
2023-06-06 14:23:02 💻 Target environment: gnu    
2023-06-06 14:23:02 💻 CPU: AMD Ryzen 7 3700X 8-Core Processor    
2023-06-06 14:23:02 💻 CPU cores: 8    
2023-06-06 14:23:02 💻 Memory: 32047MB    
2023-06-06 14:23:02 💻 Kernel: 5.19.0-43-generic    
2023-06-06 14:23:02 💻 Linux distribution: Ubuntu 22.04.2 LTS    
2023-06-06 14:23:02 💻 Virtual machine: no    
2023-06-06 14:23:02 📦 Highest known block at #0    
2023-06-06 14:23:02 〽️ Prometheus exporter started at 127.0.0.1:9615    
2023-06-06 14:23:02 Running JSON-RPC server: addr=127.0.0.1:9944, allowed origins=["*"]    
2023-06-06 14:23:02 🏁 CPU score: 1.35 GiBs    
2023-06-06 14:23:02 🏁 Memory score: 11.60 GiBs    
2023-06-06 14:23:02 🏁 Disk score (seq. writes): 59.30 MiBs    
2023-06-06 14:23:02 🏁 Disk score (rand. writes): 73.81 MiBs    
2023-06-06 14:23:02 ⚠️  The hardware does not meet the minimal requirements for role 'Authority'.    
2023-06-06 14:23:02 👶 Starting BABE Authorship worker    
2023-06-06 14:23:07 💤 Idle (0 peers), best: #0 (0x1cc0…1a93), finalized #0 (0x1cc0…1a93), ⬇ 0 ⬆ 0   
```
Installing session keys.

```
bash init_chain/setKeys.sh
```
After setting the session keys, imports of new blocks should begin.
```
2023-06-06 14:25:12 🙌 Starting consensus session on top of parent 0x1cc0aeb2c79a96dbb8468c61322d844db2c2218fe2481b675227d63773001a93    
2023-06-06 14:25:12 🎁 Prepared block for proposing at 1 (0 ms) [hash: 0xe373105438946d3e149c633799ec56685982253bbbd0a56bead206de17c4f032; parent_hash: 0x1cc0…1a93; extrinsics (1): [0x6a83…a231]]    
2023-06-06 14:25:12 🔖 Pre-sealed block for proposal at 1. Hash now 0x056286914d062f0b74c4b5b244e01d8233f95cac4fe83921b1fb9c5fa679c91c, previously 0xe373105438946d3e149c633799ec56685982253bbbd0a56bead206de17c4f032.    
2023-06-06 14:25:12 👶 New epoch 0 launching at block 0x0562…c91c (block slot 562016904 >= start slot 562016904).    
2023-06-06 14:25:12 👶 Next epoch starts at slot 562017104    
2023-06-06 14:25:12 ✨ Imported #1 (0x0562…c91c)    
2023-06-06 14:25:12 💤 Idle (0 peers), best: #1 (0x0562…c91c), finalized #0 (0x1cc0…1a93), ⬇ 0 ⬆ 0    
2023-06-06 14:25:15 🙌 Starting consensus session on top of parent 0x056286914d062f0b74c4b5b244e01d8233f95cac4fe83921b1fb9c5fa679c91c    
2023-06-06 14:25:15 🎁 Prepared block for proposing at 2 (0 ms) [hash: 0xc5ef1fb89c94e058a34259de91cae5d3243d0043cc18f3d2526e4132c23872ed; parent_hash: 0x0562…c91c; extrinsics (1): [0x5409…e5fb]]    
2023-06-06 14:25:15 🔖 Pre-sealed block for proposal at 2. Hash now 0x3bf3f61a316e2a848f206543f1fe561e8ac5fa60eb4f7f5bef741b3b45da0954, previously 0xc5ef1fb89c94e058a34259de91cae5d3243d0043cc18f3d2526e4132c23872ed.    
2023-06-06 14:25:15 ✨ Imported #2 (0x3bf3…0954)    
2023-06-06 14:25:17 💤 Idle (0 peers), best: #2 (0x3bf3…0954), finalized #0 (0x1cc0…1a93), ⬇ 0 ⬆ 0    
```
Reboot bootnode!

# 2 Deploy staking contract
The staking contract in the cue chain network acts as a proxy and allows interaction with the NPOS consensus mechanisms from the EVM environment.
Let's go to the staking folder, install the dependencies and compile the staking contract.
```
cd staking
yarn install
npx hardhat compile
```
Deploying a contract on the Qchain network.
```
npx hardhat run scripts/deploy.ts --network qchain
```

# 3 Setup the validator
Generating the node key.
```
cd ../
./target/release/qchain-template-node key generate --scheme sr25519
```
```
Secret phrase:       hedgehog ramp skirt file vault lion remain mirror average energy absent thank
  Network ID:        substrate
  Secret seed:       0x15d93a9a6d63789c9854ecbd9f2ef31b2e8e748303f504fa41d98fe4cc955e78
  Public key (hex):  0x8c136f01690d861cbb704b51c32241e7c3c00541655063bcce847c99b22ab2e7
  Account ID:        0x8c136f01690d861cbb704b51c32241e7c3c00541655063bcce847c99b22ab2e7
  Public key (SS58): 5FENLKrkBchK4o1BqGnhQi65fjg4QDyXuywpWMVQXHT9MKqo
  SS58 Address:      5FENLKrkBchK4o1BqGnhQi65fjg4QDyXuywpWMVQXHT9MKqo
  ```
Generating a mnemonic phrase for session keys.
```
./target/release/qchain-template-node key generate --scheme sr25519
```
```
Secret phrase:       clean wire screen impose about spare shrug own bean wisdom proud traffic
  Network ID:        substrate
  Secret seed:       0xcc774030ece9ed38a9687c563459626f263b95301fd75fd772faf680f760ed08
  Public key (hex):  0x1ca401556ac538cf5752118af9667dd286042cbc3205164598d29f728156e276
  Account ID:        0x1ca401556ac538cf5752118af9667dd286042cbc3205164598d29f728156e276
  Public key (SS58): 5CiFvBFKbzkfzAo2N7eid76tF5X6qv2L3BTzHR29krLMMbAQ
  SS58 Address:      5CiFvBFKbzkfzAo2N7eid76tF5X6qv2L3BTzHR29krLMMbAQ
```
Paste the phrase into prepare-test-net.sh.
```
SECRET="clean wire screen impose about spare shrug own bean wisdom proud traffic"
```
Generating session keys for the node.
```
bash init_chain/prepare-test-net.sh 1
```
The first two lines will contain the id of the account with which the node will start and its secret key, which will be needed to participate in staking.
```
	// 5G9fHKCmjW5Rgdfn4q7PFo7QmFG6YiGsvCEnbvfAd8tkPSD6
 0x63f750a1fbd556394bca63add26f2a64739d9533751f773d82c41453deb799d0

(
// 5G9fHKCmjW5Rgdfn4q7PFo7QmFG6YiGsvCEnbvfAd8tkPSD6
array_bytes::hex_n_into_unchecked("b4b87e0c3216c8bcb51dfa283def26a6986a86318854568daec5b2cfdc869324"),
// 5CZcnx7ZiD8M95xtWvtMrU4KN3Uz2aU4V4F6ckJYhcJexfNS
array_bytes::hex_n_into_unchecked("160d11c794ddd8f53bc02b35ddaae2b90a639f24d76f448c9c1a308901a9074b"),
// 5DhMvxmABKTzAjqrK3BkEqVrspxUaweThNFegfWosLGNUhqM
array_bytes::hex2array_unchecked("48313c92186dd43a8cd157062ba9cfc1edefb9f8e1a49382e2384a18e53303ce").unchecked_into(),
// 5HQGGDHnfYAEwLs4VDXt8A8zN8q2Mv8rfGpuRsaLZsx7w2oi
array_bytes::hex2array_unchecked("ec17eb0436c9747a3e13e589169f5b029f509a1eacf035eda7912d7b0369930d").unchecked_into(),
// 5FqB3nJJRG3QRhE4X7FHzzrM7torBK72VRkk5xTa5GnxxweM
array_bytes::hex2array_unchecked("a69f08832debc934182d8511c855f2a3cb2777bb24ffc62238f01362a7abce49").unchecked_into(),
// 5EkbFkuwmNnzP3mxprG3XVtDqpv8wVkKncKWiKpY1jA6m6WD
array_bytes::hex2array_unchecked("76e40f1f16df6970c98e85d1c207e92d980c3faaa7b8d1171d99f73f092aed69").unchecked_into(),
),
```
Compiling.
```
cargo build --release
```
Starting the first validator.
```
./target/release/qchain-template-node \
--chain=CustomSpec.json \
-d data/validator1 \
--name validator1 \
--port 30334 \
--ws-port 9945 \
--rpc-port 8546 \
--unsafe-ws-external \
--unsafe-rpc-external \
--rpc-methods=unsafe \
--rpc-cors=all \
--telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
--validator \
--bootnodes /ip4/0.0.0.0/tcp/30333/p2p/12D3KooWMLaHH2ubp7kEYNGoRwWt3JQtQYnWmLiJhDcZvimVj7hm \
```
The Genesis block must match the Genesis block of the boot node. The connection should be established and show 1 peer.
```
2023-06-06 14:38:41 Substrate Node    
2023-06-06 14:38:41 ✌️  version 3.0.0-dev-13ed4508e65    
2023-06-06 14:38:41 ❤️  by Parity Technologies <admin@parity.io>, 2017-2023    
2023-06-06 14:38:41 📋 Chain specification: Staging Testnet    
2023-06-06 14:38:41 🏷  Node name: validator1    
2023-06-06 14:38:41 👤 Role: AUTHORITY    
2023-06-06 14:38:41 💾 Database: RocksDb at data/validator1/chains/staging_testnet/db/full    
2023-06-06 14:38:41 ⛓  Native runtime: node-268 (substrate-node-0.tx2.au10)    
2023-06-06 14:38:42 [0] 💸 generated 1 npos voters, 1 from validators and 0 nominators    
2023-06-06 14:38:42 [0] 💸 generated 1 npos targets    
2023-06-06 14:38:44 🔨 Initializing Genesis block/state (state: 0xea69…106e, header-hash: 0x1cc0…1a93)    
2023-06-06 14:38:44 👴 Loading GRANDPA authority set from genesis on what appears to be first startup.    
2023-06-06 14:38:45 👶 Creating empty BABE epoch changes on what appears to be first startup.    
2023-06-06 14:38:45 Using default protocol ID "sup" because none is configured in the chain specs    
2023-06-06 14:38:45 🏷  Local node identity is: 12D3KooWEVGgNNywY8zoZgu2iD2QKw3SYatH42A8fZ3DmrN1RhEW    
2023-06-06 14:38:45 💻 Operating system: linux    
2023-06-06 14:38:45 💻 CPU architecture: x86_64    
2023-06-06 14:38:45 💻 Target environment: gnu    
2023-06-06 14:38:45 💻 CPU: AMD Ryzen 7 3700X 8-Core Processor    
2023-06-06 14:38:45 💻 CPU cores: 8    
2023-06-06 14:38:45 💻 Memory: 32047MB    
2023-06-06 14:38:45 💻 Kernel: 5.19.0-43-generic    
2023-06-06 14:38:45 💻 Linux distribution: Ubuntu 22.04.2 LTS    
2023-06-06 14:38:45 💻 Virtual machine: no    
2023-06-06 14:38:45 📦 Highest known block at #0    
2023-06-06 14:38:45 Running JSON-RPC server: addr=127.0.0.1:9934, allowed origins=["*"]    
2023-06-06 14:38:45 🏁 CPU score: 1.35 GiBs    
2023-06-06 14:38:45 🏁 Memory score: 11.81 GiBs    
2023-06-06 14:38:45 🏁 Disk score (seq. writes): 384.63 MiBs    
2023-06-06 14:38:45 🏁 Disk score (rand. writes): 203.63 MiBs    
2023-06-06 14:38:45 ⚠️  The hardware does not meet the minimal requirements for role 'Authority'.    
2023-06-06 14:38:45 👶 Starting BABE Authorship worker    
2023-06-06 14:38:50 💤 Idle (1 peers), best: #78 (0x7a61…ab01), finalized #0 (0x1cc0…1a93), ⬇ 5.1kiB/s ⬆ 0.9kiB/s    
```
Installing session keys.
```
bash init_chain/setKeys.sh
```
Reboot first validator node!

Adding a validator via a staking contract.
```
cd staking
npx hardhat run scripts/bondWithValidate.ts --network frontier
```

# 4 Setup nominator
Specify your values in the staking/scripts/bondWithNominate.ts
```
const amount = ethers.parseEther("1000000")
const validators = ["0x24a1bee3138fd67f3d1956f8c28333537d1dcd38a48257ebdbecfb77d744f741"]
```
Start the nomination process

```
npx hardhat run scripts/bondWithNominate.ts --network qchain
```