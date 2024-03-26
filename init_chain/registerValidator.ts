import { ethers } from "ethers";
import { ApiPromise, WsProvider } from "@polkadot/api";

async function bondWithValidate(privateKey: string, url: string, abi: string, amount: bigint) {
    const provider = new ethers.WebSocketProvider(url)
    const wallet = new ethers.Wallet(privateKey, provider)

    const apiProvider = new WsProvider(url);
    const api = await ApiPromise.create({ provider: apiProvider });
    const keys = await api.rpc.author.rotateKeys.raw()

    console.log('keys: ', keys)

    const stakingContract = new ethers.Contract('0xdC408Cf5cD34D074382321098885A9e94C3bFB7A', abi, wallet)
    const tx = await stakingContract.bondWithValidate('0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d', 0, amount, keys, '0x00', 0, false)

    tx.wait().then((receipt: any) => {
        console.log('Transaction successful', receipt.hash);
        process.exit(0);
    }).catch((error: any) => {
        console.error('Transaction failed:', error);
        process.exit(1);
    });
}

const privateKey = "0x123456...";
const url = 'ws://127.0.0.1:8545'
const amount = ethers.parseEther("10000000")
const abi = '[{"inputs":[],"stateMutability":"nonpayable","type":"constructor"},{"anonymous":false,"inputs":[{"indexed":false,"internalType":"bytes32","name":"controller","type":"bytes32"},{"indexed":true,"internalType":"uint256","name":"value","type":"uint256"},{"indexed":true,"internalType":"enum QStakingProxy.StakingRewardDestination","name":"destination","type":"uint8"}],"name":"Bonded","type":"event"},{"anonymous":false,"inputs":[{"indexed":false,"internalType":"address","name":"sender","type":"address"},{"indexed":true,"internalType":"uint256","name":"value","type":"uint256"}],"name":"ExtraBonded","type":"event"},{"anonymous":false,"inputs":[{"indexed":false,"internalType":"address","name":"add","type":"address"},{"indexed":false,"internalType":"bytes32[]","name":"validators","type":"bytes32[]"}],"name":"Nominated","type":"event"},{"anonymous":false,"inputs":[{"indexed":false,"internalType":"address","name":"sender","type":"address"},{"indexed":false,"internalType":"bytes","name":"keys","type":"bytes"},{"indexed":false,"internalType":"bytes","name":"proof","type":"bytes"}],"name":"SettedKeys","type":"event"},{"anonymous":false,"inputs":[{"indexed":false,"internalType":"address","name":"sender","type":"address"},{"indexed":true,"internalType":"uint256","name":"value","type":"uint256"}],"name":"UnBonded","type":"event"},{"anonymous":false,"inputs":[{"indexed":false,"internalType":"address","name":"sender","type":"address"},{"indexed":true,"internalType":"uint256","name":"comission","type":"uint256"},{"indexed":true,"internalType":"bool","name":"isBlockedNewNominates","type":"bool"}],"name":"Validated","type":"event"},{"anonymous":false,"inputs":[{"indexed":false,"internalType":"address","name":"sender","type":"address"},{"indexed":true,"internalType":"uint256","name":"numSlashingSpans","type":"uint256"}],"name":"WithdrawUnBonded","type":"event"},{"inputs":[{"internalType":"address","name":"","type":"address"}],"name":"balances","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"bytes32","name":"_controller","type":"bytes32"},{"internalType":"enum QStakingProxy.StakingRewardDestination","name":"_destination","type":"uint8"},{"internalType":"uint256","name":"_value","type":"uint256"}],"name":"bond","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"uint256","name":"_value","type":"uint256"}],"name":"bondExtra","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"bytes32","name":"_controller","type":"bytes32"},{"internalType":"enum QStakingProxy.StakingRewardDestination","name":"_destination","type":"uint8"},{"internalType":"bytes32[]","name":"_validators","type":"bytes32[]"},{"internalType":"uint256","name":"_value","type":"uint256"}],"name":"bondWithNominate","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"bytes32","name":"_controller","type":"bytes32"},{"internalType":"enum QStakingProxy.StakingRewardDestination","name":"_destination","type":"uint8"},{"internalType":"uint256","name":"_value","type":"uint256"},{"internalType":"bytes","name":"_keys","type":"bytes"},{"internalType":"bytes","name":"_proof","type":"bytes"},{"internalType":"uint256","name":"_commission","type":"uint256"},{"internalType":"bool","name":"isBlockedNewNominates","type":"bool"}],"name":"bondWithValidate","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[],"name":"getValidators","outputs":[{"internalType":"string[]","name":"","type":"string[]"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"bytes32[]","name":"_validators","type":"bytes32[]"}],"name":"nominate","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"bytes","name":"_keys","type":"bytes"},{"internalType":"bytes","name":"_proof","type":"bytes"}],"name":"setKeys","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"uint256","name":"_value","type":"uint256"}],"name":"unBond","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"uint256","name":"_value","type":"uint256"},{"internalType":"uint256","name":"_numSlashingSpans","type":"uint256"}],"name":"unBondWithWithdrawUnBonded","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"uint256","name":"_commission","type":"uint256"},{"internalType":"bool","name":"isBlockedNewNominates","type":"bool"}],"name":"validate","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"uint256","name":"","type":"uint256"}],"name":"validators","outputs":[{"internalType":"string","name":"","type":"string"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"uint256","name":"_numSlashingSpans","type":"uint256"}],"name":"withdrawUnBonded","outputs":[],"stateMutability":"nonpayable","type":"function"}]'
bondWithValidate(privateKey, url, abi, amount).catch((error) => {
    console.error(error);
    process.exitCode = 1;
});
