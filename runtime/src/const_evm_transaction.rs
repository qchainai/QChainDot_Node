use frame_benchmarking::Zero;
use frame_support::traits::{Currency, ExistenceRequirement, Imbalance, OnUnbalanced, SignedImbalance, WithdrawReasons};
use pallet_balances::NegativeImbalance;
use pallet_evm::{AddressMapping, Config, Error, OnChargeEVMTransaction, Pallet};
use sp_arithmetic::traits::UniqueSaturatedInto;
use sp_core::{H160, U256};
use sp_runtime::Saturating;
use frame_support::log;
use sp_staking::StakingInterface;
use frame_election_provider_support::ElectionDataProvider;
use pallet_staking::NominatorsHandle;

type NegativeImbalanceOf<C, T> =
    <C as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

pub struct EVMConstFeeAdapter<C, OU, S>(sp_std::marker::PhantomData<(C, OU, S)>);

const CONST_TRANSACTION_FEE: u128 = 1000000000000000000;

impl<T, C, OU, S> OnChargeEVMTransaction<T> for EVMConstFeeAdapter<C, OU, S>
    where
        T: Config + pallet_staking::Config<CurrencyBalance = u128> + pallet_babe::Config,
        C: Currency<<T as frame_system::Config>::AccountId, Balance = u128>,
        S: StakingInterface<
            AccountId = <T as frame_system::Config>::AccountId,
            Balance = <C as Currency<<T as frame_system::Config>::AccountId>>::Balance,
        > + ElectionDataProvider + NominatorsHandle<T>,
        <S as ElectionDataProvider>::AccountId: core::fmt::Debug,
        C::PositiveImbalance: Imbalance<
            <C as Currency<<T as frame_system::Config>::AccountId>>::Balance,
            Opposite = C::NegativeImbalance,
        >,
        C::NegativeImbalance: Imbalance<
            <C as Currency<<T as frame_system::Config>::AccountId>>::Balance,
            Opposite = C::PositiveImbalance,
        >,
        OU: OnUnbalanced<NegativeImbalanceOf<C, T>>,
        U256: UniqueSaturatedInto<<C as Currency<<T as frame_system::Config>::AccountId>>::Balance>,
        <T as frame_system::Config>::AccountId: From<sp_core::sr25519::Public>
{
    // Kept type as Option to satisfy bound of Default
    type LiquidityInfo = Option<NegativeImbalanceOf<C, T>>;

    fn withdraw_fee(who: &H160, fee: U256) -> Result<Self::LiquidityInfo, Error<T>> {
        let payer = T::AddressMapping::into_account_id(*who);
        let fee = U256::from(CONST_TRANSACTION_FEE).unique_saturated_into();
        let imbalance = C::withdraw(
            &payer,
            fee,
            WithdrawReasons::FEE,
            ExistenceRequirement::AllowDeath,
        )
            .map_err(|err| {
                log::error!("Error: {:?}", err);
                Error::<T>::BalanceLow
            })?;

        let validator = <pallet_staking::Pallet<T>>::author().ok_or_else(
            || {
                log::error!("Failed to find block author");
                Error::<T>::Undefined
            }
        )?.into();

        S::insert_validator_rewards(&validator, CONST_TRANSACTION_FEE).map_err(|err| {
            log::error!("Error while insert validator rewards: {:?}", err);
            Error::<T>::FeeOverflow
        })?;;

        let fee = U256::from(CONST_TRANSACTION_FEE / 10).unique_saturated_into();
        let _ = C::deposit_creating(&validator, fee);

        let exposure = S::get_nominators_shares(&validator).map_err(|err| {
            log::error!("Error while get nominators: {}", err);
            Error::<T>::Undefined
        })?;
        let mut stakers_fee = CONST_TRANSACTION_FEE * 9 / 10;
        let staked = exposure.total - exposure.own;

        for staker in exposure.others {
            let staker_fee = ((CONST_TRANSACTION_FEE * 9 / 10) as f64 / staked as f64 * staker.value as f64) as u128;
            let staker_fee = if staker_fee < stakers_fee {
                stakers_fee -= staker_fee;
                staker_fee
            } else {
                let fee = stakers_fee;
                stakers_fee = 0;
                fee
            };
            log::info!("Staker: {:?}, fee: {:?}", staker.who, staker_fee);
            C::deposit_creating(&staker.who, U256::from(staker_fee).unique_saturated_into());
        }
        if stakers_fee != 0 {
            C::deposit_creating(&validator,  U256::from(stakers_fee).unique_saturated_into());
        }

        Ok(Some(imbalance))
    }

    fn correct_and_deposit_fee(
        who: &H160,
        corrected_fee: U256,
        base_fee: U256,
        already_withdrawn: Self::LiquidityInfo,
    ) -> Self::LiquidityInfo {
        None
    }

    fn pay_priority_fee(tip: Self::LiquidityInfo) {

    }
}


#[cfg(test)]
mod tests {
    use crate::const_evm_transaction::CONST_TRANSACTION_FEE;

    struct Exposure {
        pub total: u128,
        pub own: u128
    }

    #[test]
    fn fee_calculation() {

        let exposure = Exposure{
            total: 100000999999999927562382,
            own: 999999999956657370
        };
        let staker_value: u128 = 99999999999999970905012;
        let mut stakers_fee = CONST_TRANSACTION_FEE * 9 / 10;
        let staked = exposure.total - exposure.own;

        let staker_fee: u128 = ((CONST_TRANSACTION_FEE * 9 / 10) as f64 / staked as f64 * staker_value as f64) as u128;
        let staker_fee = if staker_fee < stakers_fee {
            stakers_fee -= staker_fee;
            staker_fee
        } else {
            let fee = stakers_fee;
            stakers_fee = 0;
            fee
        };
        assert_eq!(staker_fee, 900000000000000000);
    }
}
