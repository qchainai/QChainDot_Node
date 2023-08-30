use frame_support::dispatch::RawOrigin;
use sp_std::{marker::PhantomData};
use frame_support::traits::FindAuthor;
use pallet_evm::EnsureAddressOrigin;
use sp_core::crypto::AccountId32;
use sp_core::{H160};
use sp_runtime::ConsensusEngineId;
use sp_core_hashing::keccak_256;

pub struct EnsureAddressHashing;

impl<OuterOrigin> EnsureAddressOrigin<OuterOrigin> for EnsureAddressHashing
    where
        OuterOrigin: Into<Result<RawOrigin<AccountId32>, OuterOrigin>> + From<RawOrigin<AccountId32>>,
{
    type Success = AccountId32;

    fn try_address_origin(address: &H160, origin: OuterOrigin) -> Result<AccountId32, OuterOrigin> {
        origin.into().and_then(|o| match o {
            RawOrigin::Signed(who) if keccak_256(AsRef::<[u8; 32]>::as_ref(&who))[12..] == address[0..20] => {
                Ok(who)
            }
            r => Err(OuterOrigin::from(r)),
        })
    }
}
