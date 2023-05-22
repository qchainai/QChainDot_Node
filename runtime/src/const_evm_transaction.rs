use frame_benchmarking::Zero;
use frame_support::traits::{Currency, ExistenceRequirement, Imbalance, OnUnbalanced, SignedImbalance, WithdrawReasons};
use pallet_balances::NegativeImbalance;
use pallet_evm::{AddressMapping, Config, Error, OnChargeEVMTransaction, Pallet};
use sp_arithmetic::traits::UniqueSaturatedInto;
use sp_core::{H160, U256};
use sp_runtime::Saturating;

type NegativeImbalanceOf<C, T> =
    <C as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;

pub struct EVMConstFeeAdapter<C, OU>(sp_std::marker::PhantomData<(C, OU)>);

const CONST_TRANSACTION_FEE: u64 = 1000000000000000000;

impl<T, C, OU> OnChargeEVMTransaction<T> for EVMConstFeeAdapter<C, OU>
    where
        T: Config,
        C: Currency<<T as frame_system::Config>::AccountId>,

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
{
    // Kept type as Option to satisfy bound of Default
    type LiquidityInfo = Option<NegativeImbalanceOf<C, T>>;

    fn withdraw_fee(who: &H160, fee: U256) -> Result<Self::LiquidityInfo, Error<T>> {
        if fee.is_zero() {
            return Ok(None);
        }
        let payer = T::AddressMapping::into_account_id(*who);
        let fee = U256::from(CONST_TRANSACTION_FEE).unique_saturated_into();
        let imbalance = C::withdraw(
            &payer,
            fee,
            WithdrawReasons::FEE,
            ExistenceRequirement::AllowDeath,
        )
            .map_err(|_| Error::<T>::BalanceLow)?;
        let validator = T::AddressMapping::into_account_id(<Pallet<T>>::find_author());
        let _ = C::deposit_creating(&validator, fee);
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