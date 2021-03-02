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
    debug, decl_error, decl_event, decl_module, decl_storage, ensure, traits::Get,
};
use frame_system::ensure_root;
use fuso_support::traits::Participants;
use sp_runtime::traits::Convert;
use sp_std::{collections::btree_set::BTreeSet, convert::TryInto, prelude::*};

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    type MaxMembers: Get<u32>;

    type CouncilTerm: Get<u32>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Council {

        pub Members get(fn members): BTreeSet<T::AccountId>;

        Expire get(fn expire): bool = true;
    }
    add_extra_genesis {
        config(members): Vec<T::AccountId>;
        build(|config: &GenesisConfig<T>| {
            <Module<T>>::initialize_members(&config.members)
        })
    }
}

decl_event! {
    pub enum Event<T>
    where
        <T as frame_system::Trait>::AccountId,
    {
        MemberAdd(AccountId),
        MemberRemoved(AccountId),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        const MaxMembers: u32 = T::MaxMembers::get();

        const CouncilTerm: u32 = T::CouncilTerm::get();

        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 1000]
        pub fn add_member(origin, owner: T::AccountId) {
            ensure_root(origin)?;
        }

        #[weight = 1000]
        pub fn remove_member(origin, account: T::AccountId) {
            ensure_root(origin)?;
        }
    }
}

impl<T: Trait> Module<T> {
    fn initialize_members(members: &Vec<T::AccountId>) {
        let init = members
            .iter()
            .map(|x| x.clone())
            .collect::<BTreeSet<T::AccountId>>();
        Members::<T>::put(&init);
    }
}

impl<T: Trait> pallet_session::ShouldEndSession<T::BlockNumber> for Module<T> {
    fn should_end_session(now: T::BlockNumber) -> bool {
        TryInto::<u32>::try_into(now).ok().unwrap() % T::CouncilTerm::get() == 0
    }
}

impl<T: Trait> pallet_session::SessionManager<T::AccountId> for Module<T> {
    fn new_session(new_index: u32) -> Option<Vec<T::AccountId>> {
        Expire::put(false);
        // set members in vote pallet
        Some(Self::members().iter().cloned().collect())
    }

    fn end_session(end_index: u32) {}

    fn start_session(start_index: u32) {}
}

impl<T: Trait> Convert<T::AccountId, Option<T::AccountId>> for Module<T> {
    fn convert(account: T::AccountId) -> Option<T::AccountId> {
        Some(account)
    }
}

impl<T: Trait> Participants<T::AccountId> for Module<T> {
    fn get_participants() -> BTreeSet<T::AccountId> {
        Self::members()
    }
}
