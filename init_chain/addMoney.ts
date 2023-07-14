const { ApiPromise } = require('@polkadot/api');
const { Keyring } = require('@polkadot/keyring');

const phrase = '0x422ca7e726545999c56203306f58bdda71d0f659bd9c0da61650f90b66054b30'
const amount = '10000000000000000'
const validator = '5DJmutj2YKfvAGSTi6eYcctRMKzePYTdaq3r8hcYpeSDzZU5'

async function addMoney () {
    const api = await ApiPromise.create();
    const keyring = new Keyring({ type: 'ed25519' });

    const main = keyring.addFromUri(phrase, { name: 'first pair' }, 'sr25519');

    const tx = await api.tx.balances
        .transfer(validator, amount)

    const ext = api.tx.utility.batchAll([tx])
    const hash = await ext.signAndSend(main)

    console.log(`Submitted with hash ${hash}`)
}

addMoney().catch(console.error).finally(() => process.exit());