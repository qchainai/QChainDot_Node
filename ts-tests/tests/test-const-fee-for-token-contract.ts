import { expect } from "chai";
import { AbiItem } from "web3-utils";

import MyToken from "../build/contracts/MyToken.json";
import {
    GENESIS_ACCOUNT,
    GENESIS_ACCOUNT_PRIVATE_KEY,
    FIRST_CONTRACT_ADDRESS,
    BLOCK_HASH_COUNT,
    ETH_BLOCK_GAS_LIMIT, GENESIS_ACCOUNT_BALANCE, EVM_CONST_FEE, SOME_ACCOUNT,
} from "./config";
import { createAndFinalizeBlock, createAndFinalizeBlockNowait, customRequest, describeWithFrontier } from "./util";

describeWithFrontier("Frontier RPC (Token Contract Methods)", (context) => {
    const TEST_CONTRACT_BYTECODE = MyToken.bytecode;
    const TEST_CONTRACT_ABI = MyToken.abi as AbiItem[];

    // Those test are ordered. In general this should be avoided, but due to the time it takes
    // to spin up a frontier node, it saves a lot of time.

    before("create the contract", async function () {
        this.timeout(15000);
        const tx = await context.web3.eth.accounts.signTransaction(
            {
                from: GENESIS_ACCOUNT,
                data: TEST_CONTRACT_BYTECODE,
                value: "0x00",
                gasPrice: "0x3B9ACA00",
                gas: "0x100000",
            },
            GENESIS_ACCOUNT_PRIVATE_KEY
        );
        await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
        await createAndFinalizeBlock(context.web3);
    });

    it("get transaction by hash", async () => {
        const latestBlock = await context.web3.eth.getBlock("latest");
        expect(latestBlock.transactions.length).to.equal(1);

        const txHash = latestBlock.transactions[0];
        const tx = await context.web3.eth.getTransaction(txHash);
        expect(tx.hash).to.equal(txHash);
    });

    it("should return contract method result with correct gas spent", async function () {

        const expectedGenesisBalance = (
            BigInt(GENESIS_ACCOUNT_BALANCE) -
            BigInt(EVM_CONST_FEE)
        ).toString();

        const contract = new context.web3.eth.Contract(TEST_CONTRACT_ABI, FIRST_CONTRACT_ADDRESS, {
            from: GENESIS_ACCOUNT,
            gasPrice: "0x3B9ACA00",
        });

        const tx = await context.web3.eth.accounts.signTransaction(
            {
                from: GENESIS_ACCOUNT,
                data: await contract.methods.transfer(SOME_ACCOUNT, 1).encodeABI(),
                value: "0x00",
                gasPrice: "0x3B9ACA00",
                gas: "0x100000",
            },
            GENESIS_ACCOUNT_PRIVATE_KEY
        );
        await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);

        // ).to.equal("true");
        expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT)).to.equal(expectedGenesisBalance);
    });
});
