# Qdot: Secure Platform for Smart Wallets and Decentralized Applications
Our infrastructure offers a fixed-fee model, ensuring transparent and cost-effective transaction pricing for users, which is a key advantage of fixed fees over variable fees.

By leveraging the Substrate framework, Qdot enables developers to build scalable and interoperable dApps. The platform integrates with Frontier, offering EVM compatibility for deploying Ethereum dApps without modifications. In addition, Qdot utilizes the Nominated Proof-of-Stake (NPoS) consensus mechanism, which promotes decentralization and security.

Qdot's mission is to revolutionize the blockchain ecosystem by providing a secure and user-friendly environment for decentralized applications and smart wallets. Our platform aims to drive innovation and adoption of blockchain technology across various industries.

## Step-by-Step Guide to Becoming a New Qdot Validator and Running Your Own Node

### Step 1: Install Required Software

Install these packages.
```bash
sudo apt install build-essential
sudo apt install --assume-yes git clang curl libssl-dev protobuf-compiler
sudo apt install pkg-config
```
Install rustup and subkey.
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env && rustup default nightly && rustup update
rustup target add wasm32-unknown-unknown --toolchain nightly-2023-01-01
cargo install --force subkey --git https://github.com/paritytech/substrate --locked
```
Check installation.
```bash
rustc --version
```
Checking the configuration.
```bash
rustup show
rustup +nightly show
```
Download and install Node.js from the [official website](https://nodejs.org).
Verify the installation by opening a terminal and running the following commands:
```bash
node --version
npm --version
```

Install ts-node to be able to run scripts:
```bash
sudo apt install ts-node
```

### Step 2: Install and Configure the Validator

Install the Qdot full node:
   ```bash
   git clone https://github.com/qchainai/QDot_Node.git
   git clone https://github.com/qchaingit/QChainDot_Frontier.git
   cd QChainDot_Frontier
   git checkout dev
   cd ../node
   cargo build --release
   ```
Start the node in validator mode:
   ```bash
./target/release/qchain-template-node \
--chain=CustomSpec.json \
-d data/validator \
--name validator \
--port 30333 \
--ws-port 8545 \
--pruning archive \
--validator \
--bootnodes /ip4/38.180.64.210/tcp/30333/p2p/12D3KooWMLaHH2ubp7kEYNGoRwWt3JQtQYnWmLiJhDcZvimVj7hm
```
Wait for your node to synchronize with the network

### Step 3: Register the Validator

Go to the scripts directory and install dependencies:
   ```bash
   cd init_chain
   yarn install
   ```
Open the file registerValidator.ts and replace the validator’s private key in the privateKey field, and also indicate the number of tokens to be locked in staking (amount variable) and the url of your node.
   ```
const privateKey = "0x123456...";
const url = 'ws://127.0.0.1:8545'
const amount = ethers.parseEther("10000000")
   ```
Send a transaction to register a validator by running the script registerValidator.ts:
   ```bash
   ts-node registerValidator.ts
   ```
This console output indicates success:
 ```
   Transaction successful 0x415a461b49ca0d7afece1ee3cb48edfd046046cade3a4dc24bd92cb22736cb7e
 ```

You should also see something like this in your node logs:
 ```
   Validate: staking_log: Log { address: 0xdc408cf5cd34d074382321098885a9e94c3bfb7a, topics: [0xf2541304dbc91a7f4529ea605e36842d2537d7849d9d78d2b9faa18e6219b5c1, 0x0000000000000000000000000000000000000000000000000000000000000000, 0x0000000000000000000000000000000000000000000000000000000000000000], data: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 19, 9, 149, 16, 19, 56, 175, 38, 87, 250, 67, 169, 75, 171, 59, 86, 215, 91, 96, 71] }  
 ```

Now, in order to accept the changes, you will need to restart the node. Just stop your node and start again.

You can now go to the [staking panel](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Fstaking.testnet.qdot.network%3A443#/staking) and see your validator in the "pending" section.
At the end of the next era, he will become active and begin to participate in the production of blocks.**

**Congratulations on becoming a validator! 🚀 Your role in ensuring the network's security is vital. By validating transactions, you're helping to maintain the network's integrity and stability.**

**Being a validator is not only rewarding in terms of network security but also financially lucrative. Validators are rewarded with native tokens for their efforts in securing and validating transactions. 💰**

**Keep up the good work, and thank you for your dedication to the network! 👏**