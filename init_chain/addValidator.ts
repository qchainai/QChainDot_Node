import {ApiPromise} from "@polkadot/api";
const { Keyring } = require('@polkadot/keyring');

const sessionKey = '0x1a63b4b220606cc183e12f9b79c2380feacd0d348b76990d4d907531c13cff6ddcc24607a2480f5cb612fd40fb0890fb0de3389e74c902af35c448106abb4318143edb0279c8a7868e3faa388ac8683452cbf23289f45151c4110391513d797566ae4accf687ed5acd2a631002dab3e90b819ece0afdb68f7391933b06d1911f'
const phrase = '0x054166d7003c2087e6ae511e4cd72ef9cb591054a502c57a00b7f5c149375cf4'
const amount = '1000000000000000000'

async function addValidator () {
    // Instantiate the API
    const api = await ApiPromise.create();

    // Construct the keyring after the API (crypto has an async init)
    const keyring = new Keyring({ type: 'sr25519' });

    const call0 = api.tx.staking.bond(amount, "Staked")

    const call1 = api.tx.session.setKeys(sessionKey, '0x00')

    const call2 = api.tx.staking.validate(['1000000000', false])

    const ext = api.tx.utility.batchAll([call0, call1, call2])

    const main = keyring.addFromUri(phrase, { name: 'first pair' }, 'sr25519');

    const { data: { free: previousFree } } = await api.query.system.account(main.address);

    console.log(`${main.address} has a balance of ${previousFree}`);

    const hash = await ext.signAndSend(main)

    console.log('Added validator with hash', hash.toHex());
}

addValidator().catch(console.error).finally(() => process.exit());
