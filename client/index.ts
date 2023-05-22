import Web3 from "web3";
import { ethers } from "ethers";
import { JsonRpcResponse } from "web3-core-helpers";
import { spawn, ChildProcess } from "child_process";

export const CHAIN_ID = 42;

export const PORT = 9931;
// export const RPC_PORT = 9933;
export const RPC_PORT = 8545;
// export const RPC_PORT = 30333;
export const WS_PORT = 9944;

export const URL = "http://0.0.0.0";
// export const URL = "http://185.183.34.79";

export const DISPLAY_LOG = process.env.FRONTIER_LOG || false;
export const FRONTIER_LOG = process.env.FRONTIER_LOG || "info";
export const FRONTIER_BUILD = process.env.FRONTIER_BUILD || "release";

export const SPAWNING_TIME = 60000;
export const GENESIS_ACCOUNT_PRIVATE_KEY = "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";
export const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
export const SOME_ACCOUNT = "0xd43593c715fdd31c61141abd04a99fd6822c8558";

export const METAMASK_ACCOUNT = "0x0b54EfF6833c7DF3520A45a264FA59a0e8011a97";
export const VALIDATOR_ACCOUNT = "0x15fdd31C61141abd04A99FD6822c8558854ccDe3";
export const SECOND_META_ACCOUNT = "0xd19aC91E692CbcF1aEF633c95e8Bd06621D41f02";
export const METAMASK_PRIVATE = "0x0822f887352a5c50f11139ef52ce7b8f261c12c28f5ecdbc4ca881ad73a74a70";
export const METAMASK_ACCOUNT_OLD = "0x9921Ea2077972B51950496EFa02e68F0ad2bc4D6";
export const META_PROD = "0xD64Dc5b35C3F1F794918DCA459d0F98aA60B08D5";

export const SOME_ADDRESS = "1000000000000000000000000000000000000002";

export async function customRequest(web3: Web3, method: string, params: any[]) {
	return new Promise<JsonRpcResponse>((resolve, reject) => {
		(web3.currentProvider as any).send(
			{
				jsonrpc: "2.0",
				id: 1,
				method,
				params,
			},
			(error: Error | null, result?: JsonRpcResponse) => {
				if (error) {
					reject(
						`Failed to send custom request (${method} (${params.join(",")})): ${
							error.message || error.toString()
						}`
					);
				}
				resolve(result);
			}
		);
	});
}

// Create a block and finalize it.
// It will include all previously executed transactions since the last finalized block.
export async function createAndFinalizeBlock(web3: Web3, finalize: boolean = true) {
	const response = await customRequest(web3, "engine_createBlock", [true, finalize, null]);
	if (!response.result) {
		throw new Error(`Unexpected result: ${JSON.stringify(response)}`);
	}
	await new Promise((resolve) => setTimeout(() => resolve(), 500));
}

// Create a block and finalize it.
// It will include all previously executed transactions since the last finalized block.
export async function createAndFinalizeBlockNowait(web3: Web3) {
	const response = await customRequest(web3, "engine_createBlock", [true, true, null]);
	if (!response.result) {
		throw new Error(`Unexpected result: ${JSON.stringify(response)}`);
	}
}


function createClient() {
	let context: {
		web3: Web3;
		ethersjs: ethers.providers.JsonRpcProvider;
	} = {
		web3:  new Web3(`${URL}:${RPC_PORT}`),
		ethersjs: new ethers.providers.StaticJsonRpcProvider(`${URL}:${RPC_PORT}`)
	};
	return context
}



let context = createClient()


async function start() {

	let core_balance = await context.web3.eth.getBalance(SOME_ACCOUNT)

	console.log("Core balance: ", core_balance)

	let balance = await context.web3.eth.getBalance(GENESIS_ACCOUNT)

	console.log("Balance: ", balance);

	let validator_balance= await context.web3.eth.getBalance(VALIDATOR_ACCOUNT)

	console.log("Validator balance: ", validator_balance);

	let meta_balance= await context.web3.eth.getBalance(METAMASK_ACCOUNT)

	console.log("Metamask: ", meta_balance);

	let meta_balance_old = await context.web3.eth.getBalance(METAMASK_ACCOUNT_OLD)

	console.log("Metamask old: ", meta_balance_old);

	let test_balance= await context.web3.eth.getBalance(SOME_ADDRESS)

	console.log("Test account before: ", test_balance);

	const gasPrice = "0x10B9ACA000000"; // 1000000000
	const tx = await context.web3.eth.accounts.signTransaction(
		{
			from: GENESIS_ACCOUNT,
			to: META_PROD,
			value: 10000000000000000000,
			gasPrice: gasPrice,
			gas: "0x1000000",
		},
		GENESIS_ACCOUNT_PRIVATE_KEY,
	);
	let result = await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);

	console.log(result)
	test_balance = await context.web3.eth.getBalance(SECOND_META_ACCOUNT, "pending")

	console.log("Test account after: ", test_balance);

	test_balance = await context.web3.eth.getBalance(METAMASK_ACCOUNT, "pending")

	console.log("Meta account after: ", test_balance);


	validator_balance= await context.web3.eth.getBalance(VALIDATOR_ACCOUNT, "pending")

	console.log("Validator balance after: ", validator_balance);

	return meta_balance
}

start().then(result => console.log(result))
