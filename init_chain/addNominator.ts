import {ApiPromise, WsProvider} from "@polkadot/api";
const { Keyring } = require('@polkadot/keyring');

const privateKey = '0x914e250ee2409385ec6f57a9880fca3f86b774cd0835eb1d93cbb62b4fac5d23'
const amount = '1000000000000000000'
const url = 'ws://127.0.0.1:9944'

async function addNominator () {

    const wsProvider = new WsProvider(url);
    const api = await ApiPromise.create({ provider: wsProvider });
    const keyring = new Keyring({ type: 'ed25519' });

    // const main = keyring.addFromUri(privateKey, { name: 'first pair' }, 'sr25519');
    const main = keyring.addFromUri('//Alice', { name: 'Bob default' });

    console.log(main.address)

    const call0 = api.tx.staking.bond(main.address, amount, "Staked")
    const call1 = api.tx.staking.nominate(['5GNJqTPyNqANBkUVMN1LPPrxXnFouWXoe2wNSmmEoLctxiZY'])
    //0x8eaf04151687736326c9fea17e25fc5287613693
    const call2 = api.tx.staking.setController('5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty')
    const ext = api.tx.utility.batchAll([call0, call1, call2])

    const hash = await ext.signAndSend(main)
    console.log('Added nominator with hash', hash.toHex());
}

addNominator().catch(console.error).finally(() => process.exit());
