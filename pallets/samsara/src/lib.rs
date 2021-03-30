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
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::{Get, LockIdentifier, LockableCurrency, WithdrawReasons},
    weights::{DispatchClass, Weight},
};
use frame_system::ensure_signed;
use sp_runtime::{
    traits::{CheckedAdd, Saturating, Zero},
    Perbill,
};
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub type BalanceOf<T> = <<T as Trait>::Locks as pallet_balances::Trait>::Balance;
pub type AccountIdOf<T> = <<T as Trait>::Locks as frame_system::Trait>::AccountId;

pub const SAMSARA_ID: LockIdentifier = *b"samsaras";

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    type VitalityBlock: Get<u32>;

    type TermDuration: Get<Self::BlockNumber>;

    type VoteDuration: Get<Self::BlockNumber>;

    type MinimumVitalityWeight: Get<Weight>;

    type VoteBalancePerbill: Get<Perbill>;

    type Currency: LockableCurrency<Self::AccountId>;

    type Locks: pallet_balances::Trait;
}

decl_storage! {
    trait Store for Module<T: Trait> as Samsara {
        Vitality get(fn vitality): Vec<(T::BlockNumber, Weight)>;

        VitalityTotal get(fn vitality_total): Weight = 0;

        StartVote get(fn start_vote): Option<T::BlockNumber>;

        VoterMembers get(fn voter_members): Vec<(T::AccountId, BalanceOf<T>)>;
    }
}

decl_event! {
    pub enum Event<T>
    where
        BlockNumber = <T as frame_system::Trait>::BlockNumber,
        AccountId = <T as frame_system::Trait>::AccountId,
        Balance = BalanceOf<T>
    {
        ClearVote(BlockNumber),
        Voted(AccountId, Balance),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        AmountZero,
        VoteNotStarted,
        InsufficientBalance,
        Overflow,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        const VitalityBlock: u32 = T::VitalityBlock::get();

        const TermDuration: T::BlockNumber = T::TermDuration::get();

        const VoteDuration: T::BlockNumber = T::VoteDuration::get();

        const MinimumVitalityWeight: Weight = T::MinimumVitalityWeight::get();

        const VoteBalancePerbill: Perbill = T::VoteBalancePerbill::get();

        // vote
        #[weight = 1_000_000]
        fn vote(origin,
            #[compact] amount: BalanceOf<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // check input amount
            ensure!(!amount.is_zero(), Error::<T>::AmountZero);

            // sender encode
            let from: Vec<u8> = sender.encode();
            let account =
                <<T as Trait>::Locks as frame_system::Trait>::AccountId::decode(&mut from.as_ref())
                    .unwrap_or_default();

            let free_balance = <pallet_balances::Module<T::Locks>>::usable_balance(&account);
            ensure!(free_balance > amount, Error::<T>::InsufficientBalance);

            ensure!(Self::start_vote().is_some(), Error::<T>::VoteNotStarted);

            // push member
            VoterMembers::<T>::try_mutate(|member| -> DispatchResult {
                member.push((sender.clone(), amount));
                Ok(())
            })?;

            // lock currency
            Self::lock_currency(&account, amount)?;

            Self::deposit_event(RawEvent::Voted(sender, amount));

            Ok(())
        }

        fn on_finalize(now: T::BlockNumber) {
            Self::finalize(now);
        }
    }
}

impl<T: Trait> Module<T> {
    fn finalize(now: T::BlockNumber) {
        let vitality_len = Self::vitality().len() as u32;
        let vitality_block = T::VitalityBlock::get();

        if now > T::TermDuration::get() {
            let mut vitality = Self::vitality();
            let current_weight =
                <frame_system::Module<T>>::block_weight().get(DispatchClass::Normal);
            if vitality_len >= vitality_block {
                let (_, first_weight) = vitality.remove(0);
                VitalityTotal::put(Self::vitality_total() - first_weight + current_weight);
            } else {
                VitalityTotal::put(Self::vitality_total() + current_weight);
            }

            vitality.push((now, current_weight));

            Vitality::<T>::put(vitality);
        }

        Self::check_vote(&vitality_len, &vitality_block);
    }

    fn check_vote(vitality_len: &u32, vitality_block: &u32) {
        if vitality_len == vitality_block
            && Self::vitality_total() < T::MinimumVitalityWeight::get()
        {
            if let Some(start_vote_block) = Self::start_vote() {
                let vote_block =
                    frame_system::Module::<T>::block_number().saturating_sub(start_vote_block);
                // vote over
                if vote_block > T::VoteDuration::get() {
                    Self::check_amount();
                }
            } else {
                StartVote::<T>::put(frame_system::Module::<T>::block_number());
            }
        }
    }

    // lock currency
    fn lock_currency(account: &AccountIdOf<T>, amount: BalanceOf<T>) -> DispatchResult {
        let lock_balance = pallet_balances::Locks::<T::Locks>::get(account);
        let vote_lock = lock_balance.iter().find(|b| b.id == SAMSARA_ID);
        if let Some(lock) = vote_lock {
            let total_lock_balance = amount
                .checked_add(&lock.amount)
                .ok_or(Error::<T>::Overflow)?;
            <pallet_balances::Module<T::Locks>>::extend_lock(
                SAMSARA_ID,
                account,
                total_lock_balance,
                WithdrawReasons::all(),
            );
        } else {
            <pallet_balances::Module<T::Locks>>::set_lock(
                SAMSARA_ID,
                account,
                amount,
                WithdrawReasons::all(),
            );
        }
        Ok(())
    }

    fn check_amount() {
        let total_amount: BalanceOf<T> = Self::voter_members()
            .iter()
            .fold(Zero::zero(), |acc: BalanceOf<T>, x| acc.saturating_add(x.1));
        let total_issuance = <pallet_balances::Module<T::Locks>>::total_issuance();
        let vote_balance_perbill = T::VoteBalancePerbill::get();
        let can_samsara_balance = vote_balance_perbill.mul_ceil(total_issuance);

        if total_amount > can_samsara_balance {
            // TODO: start samsara
        }
        // clear vote
        Self::clear_vote();
    }

    fn clear_vote() {
        // unlock balance
        for i in Self::voter_members() {
            T::Currency::remove_lock(SAMSARA_ID, &i.0);
        }

        if Self::voter_members().len() > 0 {
            VoterMembers::<T>::kill();
        }
        if Self::start_vote().is_some() {
            StartVote::<T>::kill();
        }

        Self::deposit_event(RawEvent::ClearVote(
            frame_system::Module::<T>::block_number(),
        ));
    }
}
