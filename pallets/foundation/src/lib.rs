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
    debug, decl_error, decl_event, decl_module, decl_storage, ensure,
    traits::{Currency, Get, ReservableCurrency},
    weights::Weight,
};
use frame_system::ensure_root;
use sp_runtime::{traits::Convert, Perbill};
use sp_std::{collections::btree_set::BTreeSet, convert::TryInto, prelude::*};

pub type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    type UnlockDelay: Get<Self::BlockNumber>;

    type UnlockPeriod: Get<u32>;

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
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        Dummy,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event() = default;

        fn on_initialize(now: T::BlockNumber) -> Weight {
            if now < T::UnlockDelay::get() {
                0
            } else {
                if TryInto::<u32>::try_into(now).ok().unwrap() % T::UnlockPeriod::get() == 0 {
                    // TODO unlock some funds
                    T::MaximumBlockWeight::get()
                } else {
                    0
                }
            }
        }
    }
}
