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
    debug, decl_error, decl_event, decl_module, decl_storage,
    traits::Get,
    traits::{LockIdentifier, LockableCurrency, WithdrawReasons},
    weights::Weight,
};
use frame_system::ensure_root;
use fuso_pallet_elections::{BalanceOf, MemberOf, Pledger, ELECTIONS_ID};
use fuso_support::traits::Referendum;
use sp_runtime::traits::{Convert, One, Saturating};
use sp_std::vec::Vec;
use sp_std::{collections::btree_set::BTreeSet, convert::TryInto, prelude::*};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

const SESSIONS_ID: LockIdentifier = *b"sessions";

pub trait Trait:
    frame_system::Trait + fuso_pallet_elections::Trait + pallet_session::Trait + pallet_balances::Trait
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    type MinValidators: Get<u32>;

    type MaxValidators: Get<u32>;

    type CouncilTerm: Get<Self::BlockNumber>;

    type StartCouncil: Get<Self::BlockNumber>;

    type Elections: Referendum<Self::BlockNumber, u32, MemberOf<Self>>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Council {

        pub Members get(fn members): MemberOf<T>;

        pub Validators get(fn validators): BTreeSet<T::AccountId>;

    }
    add_extra_genesis {
        config(validators): Vec<T::AccountId>;
        build(|config: &GenesisConfig<T>| {
            <Module<T>>::initialize_validators(&config.validators)
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
        const StartCouncil: T::BlockNumber = T::StartCouncil::get();

        const MinValidators: u32 = T::MinValidators::get();

        const MaxValidators: u32 = T::MaxValidators::get();

        const CouncilTerm: T::BlockNumber = T::CouncilTerm::get();

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

        fn on_initialize(now: T::BlockNumber) -> Weight {
            let round = T::Elections::get_round();
            if Self::end_session_block() == now {
                <pallet_session::Module<T>>::rotate_session();
                T::Elections::proposal(now);
            }
            if round > 0 {
                let block = Self::end_session_block().saturating_sub(One::one());
                if block == now {
                    let members = Self::update_validators(round);
                    if let Some(validators) = members {
                        // to change lock id
                        Self::to_change_id();
                        let min_validators = T::MinValidators::get().try_into().unwrap();
                        if validators.len() >= min_validators {
                            Validators::<T>::put(validators);
                        }
                    }
                }
            }
            0
        }
    }
}

impl<T: Trait> Module<T> {
    fn end_session_block() -> T::BlockNumber {
        let start_council = T::StartCouncil::get();
        let council_term = T::CouncilTerm::get();
        let round = T::Elections::get_round();
        let index = <T as frame_system::Trait>::BlockNumber::from(round);
        start_council.saturating_add(index.saturating_mul(council_term))
    }

    fn initialize_validators(validators: &Vec<T::AccountId>) {
        let init = validators
            .iter()
            .map(|x| x.clone())
            .collect::<BTreeSet<T::AccountId>>();
        Validators::<T>::put(&init);
    }

    fn update_validators(index: u32) -> Option<BTreeSet<T::AccountId>> {
        let result = T::Elections::get_result(index);
        if let Some(voter_members) = result {
            let min_validators = T::MinValidators::get().try_into().unwrap();

            // must gt min_validators, otherwise return None
            if voter_members.len() >= min_validators {
                // last round sessions, unlock currency
                let current_validators = Self::members();
                for i in current_validators.iter() {
                    for j in i.pledger.iter() {
                        // unlock sessions currency
                        Self::unlock_currency(SESSIONS_ID, &j.account);
                    }
                }

                let max_validators = T::MaxValidators::get().try_into().unwrap();

                if voter_members.len() > max_validators {
                    let mut voter: MemberOf<T> = Vec::new();

                    // get top max number
                    let mut session = BTreeSet::new();
                    for (index, item) in voter_members.iter().enumerate() {
                        if index < max_validators {
                            session.insert(item.account.clone());
                            voter.push(item.clone());
                        } else {
                            // unlock pledger currency
                            Self::unlock_pledger_currency(&item.pledger, false);
                        }
                    }

                    <Members<T>>::put(voter);
                    return Some(session);
                } else {
                    <Members<T>>::put(voter_members.clone());

                    let validators = voter_members
                        .iter()
                        .map(|v| v.account.clone())
                        .collect::<BTreeSet<T::AccountId>>();
                    return Some(validators);
                }
            } else if voter_members.len() > 0 {
                // unlock all ledger currency
                for i in voter_members.iter() {
                    Self::unlock_pledger_currency(&i.pledger, true);
                }
            }
            return None;
        }
        None
    }

    fn to_change_id() {
        let members = Self::members();
        for i in members.iter() {
            for j in i.pledger.iter() {
                Self::change_lock_id(&j.account);
            }
        }
    }

    fn change_lock_id(account: &T::AccountId) {
        let lock_balance = pallet_balances::Module::<T>::locks(account);
        let election_lock = lock_balance.iter().find(|b| b.id == ELECTIONS_ID);
        if let Some(lock) = election_lock {
            Self::unlock_currency(ELECTIONS_ID, account);
            pallet_balances::Module::<T>::set_lock(
                SESSIONS_ID,
                account,
                lock.amount,
                WithdrawReasons::all(),
            );
        }
    }

    // unlock pledger currency
    fn unlock_pledger_currency(
        pledger: &Vec<Pledger<T::AccountId, T::BlockNumber, BalanceOf<T>>>,
        is_unlock_all: bool,
    ) {
        for j in pledger.iter() {
            if is_unlock_all {
                pallet_balances::Module::<T>::remove_lock(ELECTIONS_ID, &j.account);
            } else {
                let lock_balance = pallet_balances::Module::<T>::locks(&j.account);
                let election_lock = lock_balance.iter().find(|b| b.id == ELECTIONS_ID);
                if let Some(lock) = election_lock {
                    if lock.amount > j.amount {
                        // sub lock amount
                        let lock_amount = lock.amount.saturating_sub(j.amount);
                        pallet_balances::Module::<T>::set_lock(
                            ELECTIONS_ID,
                            &j.account,
                            lock_amount,
                            WithdrawReasons::all(),
                        );
                    } else {
                        // remove lock amount
                        pallet_balances::Module::<T>::remove_lock(ELECTIONS_ID, &j.account);
                    }
                }
            }
        }
    }

    // unlock currency
    fn unlock_currency(id: LockIdentifier, account: &T::AccountId) {
        pallet_balances::Module::<T>::remove_lock(id, account);
    }
}

impl<T: Trait> pallet_session::ShouldEndSession<T::BlockNumber> for Module<T> {
    fn should_end_session(now: T::BlockNumber) -> bool {
        Self::end_session_block() == now
    }
}

impl<T: Trait> pallet_session::SessionManager<T::AccountId> for Module<T> {
    fn new_session(new_index: u32) -> Option<Vec<T::AccountId>> {
        // set validators from vote pallet
        let validators = Self::validators().iter().cloned().collect();
        Some(validators)
    }

    fn end_session(end_index: u32) {}

    fn start_session(start_index: u32) {}
}

impl<T: Trait> frame_support::traits::EstimateNextSessionRotation<T::BlockNumber> for Module<T> {
    fn estimate_next_session_rotation(_now: T::BlockNumber) -> Option<T::BlockNumber> {
        None
    }

    // The validity of this weight depends on the implementation of `estimate_next_session_rotation`
    fn weight(_now: T::BlockNumber) -> u64 {
        0
    }
}

pub struct ValidatorOf<T>(sp_std::marker::PhantomData<T>);

impl<T: Trait> Convert<T::AccountId, Option<T::AccountId>> for ValidatorOf<T> {
    fn convert(account: T::AccountId) -> Option<T::AccountId> {
        Some(account)
    }
}
