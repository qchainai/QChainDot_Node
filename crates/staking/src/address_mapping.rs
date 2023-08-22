use sp_application_crypto::sp_core::crypto::AccountId32;
use sp_application_crypto::sp_core::{H256, Hasher};

pub struct HashedAccountMapping<H>(sp_std::marker::PhantomData<H>);

impl<H: Hasher<Out = H256>> AccountMapping<AccountId32> for HashedAccountMapping<H> {
    fn into_account_id(address: AccountId32) -> AccountId32 {
        let mut data = [0u8; 24];
        data[0..4].copy_from_slice(b"evm:");
        data[4..24].copy_from_slice(&<AccountId32 as AsRef<[u8;32]>>::as_ref(&address)[..20]);
        let hash = H::hash(&data);

        AccountId32::from(Into::<[u8; 32]>::into(hash))
    }
}

pub trait AccountMapping<A> {
    fn into_account_id(address: A) -> A;
}
