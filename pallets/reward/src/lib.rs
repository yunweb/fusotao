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
use codec::Decode;
use frame_support::{
    debug, decl_error, decl_event, decl_module, decl_storage, ensure,
    traits::{BalanceStatus, Currency, Get, ReservableCurrency},
    weights::{DispatchClass, Weight},
    Parameter,
};
use frame_system::{ensure_root, ensure_signed};
use fuso_support::traits::{Participants, ProofOfSecurity};
use sp_runtime::traits::{
    AtLeast32BitUnsigned, CheckEqual, CheckedAdd, CheckedSub, Hash, MaybeDisplay,
    MaybeMallocSizeOf, MaybeSerializeDeserialize, Member,
};
use sp_std::{
    collections::btree_set::BTreeSet,
    convert::{From, TryInto},
    fmt::Debug,
    prelude::*,
};

pub mod curve;

pub type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

pub type PositiveImbalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::PositiveImbalance;

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    // TODO
    type Currency: ReservableCurrency<Self::AccountId>;

    type Era: Get<u32>;

    type Participants: Participants<Self::AccountId>;

    type ExternalChainAddress: Parameter
        + Member
        + MaybeSerializeDeserialize
        + Debug
        + MaybeDisplay
        + Ord
        + Default;

    type ProofOfSecurity: ProofOfSecurity<Self::AccountId>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Rewards {

        Bonus get(fn bonus): BalanceOf<T> = curve::CURVE[0].try_into().ok().unwrap();

        Vitality get(fn vitality): Weight = 0;

        LockedReward get(fn locked_reward): map hasher(blake2_128_concat)
            T::AccountId => BalanceOf<T>;
    }
}

decl_event! {
    pub enum Event<T>
    where
        BlockNumber = <T as frame_system::Trait>::BlockNumber,
        Balance = BalanceOf<T>
    {
        RewardIssued(BlockNumber, Balance),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn on_finalize(now: T::BlockNumber) {
            let weights = <frame_system::Module<T>>::block_weight().get(DispatchClass::Normal);
            Vitality::put(Self::vitality() + weights);
        }

        fn on_initialize(now: T::BlockNumber) -> Weight {
            if TryInto::<u32>::try_into(now).ok().unwrap() % T::Era::get() == 0 {
                if T::ProofOfSecurity::pos_enabled() {
                    Self::reward_to_pos();
                    Self::reward_to_council();
                } else {
                    Self::reward_to_council();
                }
                Self::release();
                Vitality::put(0);
                T::MaximumBlockWeight::get()
            } else {
                0
            }
        }
    }
}

impl<T: Trait> Module<T> {
    fn release() {}

    fn reward_to_pos() {}

    fn reward_to_council() {}
}
