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
#![recursion_limit = "256"]
use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    traits::{BalanceStatus, Currency, ReservableCurrency},
    weights::Weight,
    Parameter,
};
use frame_system::ensure_signed;
use sp_runtime::{
    traits::{
        CheckEqual, CheckedAdd, CheckedSub, MaybeDisplay, MaybeMallocSizeOf,
        MaybeSerializeDeserialize, Member, Saturating, SimpleBitOps, StaticLookup, Zero,
    },
    RuntimeDebug,
};
use sp_std::prelude::*;

pub trait Trait: frame_system::Trait {
    type Event: From<Event> + Into<<Self as frame_system::Trait>::Event>;

    type Currency: Currency<Self::AccountId>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Samsara {
    }
}

decl_event! {
    pub enum Event
    {
        Dummy,
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
            0
        }
    }
}
