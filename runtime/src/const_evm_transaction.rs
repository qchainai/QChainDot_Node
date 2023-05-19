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
        let account_id = T::AddressMapping::into_account_id(*who);
        let imbalance = C::withdraw(
            &account_id,
            U256::from(CONST_TRANSACTION_FEE).unique_saturated_into(),
            WithdrawReasons::FEE,
            ExistenceRequirement::AllowDeath,
        )
            .map_err(|_| Error::<T>::BalanceLow)?;
        Ok(Some(imbalance))
    }

    fn correct_and_deposit_fee(
        who: &H160,
        corrected_fee: U256,
        base_fee: U256,
        already_withdrawn: Self::LiquidityInfo,
    ) -> Self::LiquidityInfo {
        if let Some(paid) = already_withdrawn {
            let account_id = T::AddressMapping::into_account_id(*who);

            // Calculate how much refund we should return
            let refund_amount = paid
                .peek()
                .saturating_sub(corrected_fee.unique_saturated_into());
            // refund to the account that paid the fees. If this fails, the
            // account might have dropped below the existential balance. In
            // that case we don't refund anything.
            let refund_imbalance = C::deposit_into_existing(&account_id, refund_amount)
                .unwrap_or_else(|_| C::PositiveImbalance::zero());

            // Make sure this works with 0 ExistentialDeposit
            // https://github.com/paritytech/substrate/issues/10117
            // If we tried to refund something, the account still empty and the ED is set to 0,
            // we call `make_free_balance_be` with the refunded amount.
            let refund_imbalance = if C::minimum_balance().is_zero()
                && refund_amount > C::Balance::zero()
                && C::total_balance(&account_id).is_zero()
            {
                // Known bug: Substrate tried to refund to a zeroed AccountData, but
                // interpreted the account to not exist.
                match C::make_free_balance_be(&account_id, refund_amount) {
                    SignedImbalance::Positive(p) => p,
                    _ => C::PositiveImbalance::zero(),
                }
            } else {
                refund_imbalance
            };

            // merge the imbalance caused by paying the fees and refunding parts of it again.
            let adjusted_paid = paid
                .offset(refund_imbalance)
                .same()
                .unwrap_or_else(|_| C::NegativeImbalance::zero());

            let (base_fee, tip) = adjusted_paid.split(base_fee.unique_saturated_into());
            // Handle base fee. Can be either burned, rationed, etc ...
            OU::on_unbalanced(base_fee);
            return Some(tip);
        }
        None
    }

    fn pay_priority_fee(tip: Self::LiquidityInfo) {
        // Default Ethereum behaviour: issue the tip to the block author.
        if let Some(tip) = tip {
            let account_id = T::AddressMapping::into_account_id(<Pallet<T>>::find_author());
            let _ = C::deposit_creating(&account_id, tip.peek());
        }
    }
}