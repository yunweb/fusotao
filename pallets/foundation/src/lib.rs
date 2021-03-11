// Copyright 2021 UINB Technologies Pte. Ltd.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::{
    decl_event, decl_module, decl_storage,
    traits::{Currency, Get, ReservableCurrency},
    weights::Weight,
};
use sp_runtime::traits::{One, Saturating, Zero};
use sp_runtime::Perbill;
use sp_std::vec::Vec;

pub type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    type UnlockDelay: Get<Self::BlockNumber>;

    type UnlockPeriod: Get<Self::BlockNumber>;

    type UnlockRatioEachPeriod: Get<Perbill>;

    type Currency: ReservableCurrency<Self::AccountId>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Foundation {
        pub Foundation get(fn foundation): map hasher(blake2_128_concat)
            T::AccountId => BalanceOf<T>;
    }
    add_extra_genesis {
        config(fund): Vec<(T::AccountId, BalanceOf<T>)>;
        build(|config: &GenesisConfig<T>| {
            for (account, balance) in &config.fund {
                Foundation::<T>::insert(account, balance);
            }
        })
    }
}

decl_event! {
    pub enum Event<T>
    where
        <T as frame_system::Trait>::AccountId,
        Balance = BalanceOf<T>,
    {
        PreLockedFundUnlocked(AccountId, Balance),
        FundAllUnlockBalance(AccountId),
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event() = default;

        const UnlockDelay: T::BlockNumber = T::UnlockDelay::get();

        const UnlockPeriod: T::BlockNumber = T::UnlockPeriod::get();

        const UnlockRatioEachPeriod: Perbill = T::UnlockRatioEachPeriod::get();

        fn on_initialize(now: T::BlockNumber) -> Weight {
            if now < T::UnlockDelay::get() {
                0
            } else {
                let unlock_delay: T::BlockNumber = T::UnlockDelay::get();
                if (now.saturating_sub(unlock_delay) % T::UnlockPeriod::get()) == Zero::zero() {
                    let unlock_period: T::BlockNumber = T::UnlockPeriod::get();
                    let unlock_ratio_each_period: Perbill = T::UnlockRatioEachPeriod::get();
                    let unlock_total_times = unlock_ratio_each_period.saturating_reciprocal_mul_ceil(One::one());
                    let last_cycle_block = unlock_period.saturating_mul(unlock_total_times);
                    let over_block = unlock_delay.saturating_add(last_cycle_block);
                    if now <= over_block {
                        Self::unlock_fund(now == over_block);
                        return T::MaximumBlockWeight::get();
                    }
                }
                0
            }
        }
    }
}

impl<T: Trait> Module<T> {
    fn unlock_fund(is_last: bool) {
        for item in Foundation::<T>::iter() {
            // (account, balance)
            let account = item.0;
            let all_reserve_balance: BalanceOf<T> = Self::foundation(&account);
            let unlock_ratio_each_period = T::UnlockRatioEachPeriod::get();

            // to be free balance
            let to_free_balance = unlock_ratio_each_period.mul_floor(all_reserve_balance);

            // if last block, free all reserved balance
            if is_last {
                let unlock_total_times =
                    unlock_ratio_each_period.saturating_reciprocal_mul_ceil(One::one());
                let already_free_balance = to_free_balance.saturating_mul(unlock_total_times);
                let last_to_free_balance = all_reserve_balance.saturating_sub(already_free_balance);
                // unreserve
                T::Currency::unreserve(&account, last_to_free_balance);
                Self::deposit_event(RawEvent::FundAllUnlockBalance(account));
            } else {
                // unreserve
                T::Currency::unreserve(&account, to_free_balance);
                Self::deposit_event(RawEvent::PreLockedFundUnlocked(account, to_free_balance));
            }
        }
    }
}
