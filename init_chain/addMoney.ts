const { ApiPromise } = require('@polkadot/api');
const { Keyring } = require('@polkadot/keyring');

const phrase = '0x422ca7e726545999c56203306f58bdda71d0f659bd9c0da61650f90b66054b30'
const validator = '5H46JJ9jAoDFgYRdv8RUx9jPehHqHqRenpJnkdysMjSeftNm';
const amount = '100000000000000000000'

async function addMoney () {
    const api = await ApiPromise.create();

    const keyring = new Keyring({ type: 'ed25519' });

    const bootNodeAccount = keyring.addFromUri(phrase, { name: 'first pair' }, 'sr25519');

    const { data: { free: previousFree }, nonce: previousNonce } = await api.query.system.account(bootNodeAccount.address);

    console.log(`${bootNodeAccount.address} has a balance of ${previousFree}, nonce ${previousNonce}`);

    const transfer = api.tx.balances.transfer(validator, amount);

    const hash = await transfer.signAndSend(bootNodeAccount);

    console.log('Transfer sent with hash', hash.toHex());
}

addMoney().catch(console.error).finally(() => process.exit());