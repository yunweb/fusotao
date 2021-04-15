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
use frame_support::traits::{Currency, Get, LockIdentifier, LockableCurrency, WithdrawReasons};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure, Parameter,
};
use frame_system::ensure_signed;
use fuso_support::{collections::binary_heap::BinaryHeap, traits::Referendum};
use sp_runtime::traits::{
    AtLeast32Bit, Bounded, CheckedAdd, CheckedSub, Member, One, Saturating, Zero,
};
use sp_runtime::RuntimeDebug;
use sp_std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// pledger struct
#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct Pledger<AccountId, BlockNumber, Balance> {
    pub account: AccountId,
    block_number: BlockNumber,
    pub amount: Balance,
}

// voter struct
#[derive(Eq, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct Voter<VoteIndex: Eq, AccountId: Eq, BlockNumber: Eq, Balance: Ord> {
    round: VoteIndex,
    pub account: AccountId,
    amount: Balance,
    pub pledger: Vec<Pledger<AccountId, BlockNumber, Balance>>,
}

impl<V: Eq, A: Eq, B: Eq, T: Ord> PartialOrd for Voter<V, A, B, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.amount.cmp(&other.amount))
    }
}

impl<V: Eq, A: Eq, B: Eq, T: Ord> Ord for Voter<V, A, B, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.amount.cmp(&other.amount)
    }
}

impl<V: Eq, A: Eq, B: Eq, T: Ord> PartialEq for Voter<V, A, B, T> {
    fn eq(&self, other: &Self) -> bool {
        self.amount == other.amount
    }
}

pub const ELECTIONS_ID: LockIdentifier = *b"election";

pub type BalanceOf<T> = <T as pallet_balances::Trait>::Balance;

pub type MemberOf<T> = BinaryHeap<
    Voter<
        <T as Trait>::VoteIndex,
        <T as frame_system::Trait>::AccountId,
        <T as frame_system::Trait>::BlockNumber,
        BalanceOf<T>,
    >,
>;

pub trait Trait: frame_system::Trait + pallet_balances::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    type Currency: LockableCurrency<Self::AccountId>;

    type VotePeriod: Get<Self::BlockNumber>;

    type MinimumVotingLock: Get<BalanceOf<Self>>;

    type VoteIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
}

decl_event! {
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        Balance = BalanceOf<T>,
        VoteIndex = <T as Trait>::VoteIndex,
        BlockNumber = <T as frame_system::Trait>::BlockNumber,
    {
        StartProposal(BlockNumber, BlockNumber, VoteIndex),
        AddCandidate(AccountId, AccountId),
        Voted(AccountId, AccountId, Balance),
        StopProposal(VoteIndex),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        NoProposalStarted,
        ProposalOver,
        AmountZero,
        AmountTooLow,
        InsufficientBalance,
        AlreadyIsVoter,
        AlreadyIsCandidate,
        NotCandidate,
        Overflow,
        NotCurrentVoteRound,
        InvalidCandidate,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Votes {
        /// The present candidate list.
        Candidates get(fn candidates): Vec<T::AccountId>;

        VoterMembers get(fn voter_members): MemberOf<T>;

        VoteRoundCount get(fn vote_round_count): T::VoteIndex;

        StartBlockNumber get(fn start_block_number): Option<T::BlockNumber>;

        EndBlockNumber get(fn end_block_number): Option<T::BlockNumber>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        const VotePeriod: T::BlockNumber = T::VotePeriod::get();

        /// The minimum amount to be used as a deposit for a public referendum proposal.
        const MinimumVotingLock: BalanceOf<T> = T::MinimumVotingLock::get();

        #[weight = 1_000]
        pub fn add_candidate(
            origin,
            who: T::AccountId,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            Self::is_proposal()?;

            // set who is candidate
            Self::set_candidate(&who)?;

            Self::deposit_event(RawEvent::AddCandidate(sender, who));
            Ok(())
        }

        // vote
        #[weight = 1_000]
        pub fn vote(origin,
            voter: T::AccountId,
            vote_round: T::VoteIndex,
            #[compact] amount: BalanceOf<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // judging whether it's a vote period
            Self::is_proposal()?;

            // check input vote round
            ensure!(vote_round == Self::vote_round_count(), Error::<T>::NotCurrentVoteRound);

            let voter_members = Self::voter_members();
            let voter_option = voter_members.iter().find(|v| v.account == voter);

            // if voter not included in members
            if voter_option.is_none() {
                // ensure voter include in candidates
                ensure!(Self::candidates().contains(&voter), Error::<T>::NotCandidate);
            }

            // check input amount
            ensure!(!amount.is_zero(), Error::<T>::AmountZero);

            // ensure input amount greater than or equal to MinimumVotingLock
            ensure!(amount >= T::MinimumVotingLock::get(), Error::<T>::AmountTooLow);

            // try vote
            Self::try_vote(&sender, &voter, amount)?;

            Self::deposit_event(RawEvent::Voted(sender, voter, amount));

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    fn is_proposal() -> DispatchResult {
        ensure!(
            Self::start_block_number().is_some(),
            Error::<T>::NoProposalStarted
        );

        let current_block_number = <frame_system::Module<T>>::block_number();
        let end_block_number = Self::end_block_number().unwrap();

        ensure!(
            current_block_number <= end_block_number,
            Error::<T>::ProposalOver
        );
        Ok(())
    }

    // set candidate
    fn set_candidate(who: &T::AccountId) -> DispatchResult {
        // if is voter, notice already
        let voter_members = Self::voter_members();
        let voter_option = voter_members.iter().find(|v| &v.account == who);

        ensure!(voter_option.is_none(), Error::<T>::AlreadyIsVoter);

        ensure!(
            !Self::candidates().contains(&who),
            Error::<T>::AlreadyIsCandidate
        );

        <Candidates<T>>::append(who);

        Ok(())
    }

    fn try_vote(
        sender: &T::AccountId,
        voter: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        let total_balance = pallet_balances::Module::<T>::total_balance(sender);
        // account all lock balance
        let lock_balance = pallet_balances::Module::<T>::locks(sender);
        let election_lock = lock_balance.iter().find(|b| b.id == ELECTIONS_ID);

        if let Some(lock) = election_lock {
            // not lockable balance
            let balance = lock.amount;
            let free_balance = total_balance
                .checked_sub(&balance)
                .ok_or(Error::<T>::Overflow)?;

            // ensure free balance greater than input amount
            ensure!(free_balance > amount, Error::<T>::InsufficientBalance);
        }


        let voter_members = Self::voter_members();
        let voter_option = voter_members.iter().find(|v| &v.account == voter);

        if voter_option.is_some() {
            // update vote
            Self::update_vote(sender, voter, amount)?;
        } else {
            // insert vote
            Self::insert_vote(sender, voter, amount)?;
        }

        Self::lock_currency(sender, amount)?;

        Ok(())
    }

    fn insert_vote(
        sender: &T::AccountId,
        voter: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        // remove candidate
        Candidates::<T>::try_mutate(|candidates| -> DispatchResult {
            let index = candidates
                .iter()
                .position(|a| a == voter)
                .ok_or(Error::<T>::InvalidCandidate)?;
            candidates.remove(index);
            Ok(())
        })?;

        // current pledger
        let pledger = Pledger {
            account: sender.clone(),
            block_number: <frame_system::Module<T>>::block_number(),
            amount: amount.clone(),
        };

        // new vec
        let mut pledger_vec: Vec<Pledger<T::AccountId, T::BlockNumber, BalanceOf<T>>> = Vec::new();
        // push pledger
        pledger_vec.push(pledger);

        let voter_member = Voter {
            account: voter.clone(),
            round: Self::vote_round_count(),
            amount: amount.clone(),
            pledger: pledger_vec,
        };

        VoterMembers::<T>::try_mutate(|voters| -> DispatchResult {
            voters.push(voter_member);
            Ok(())
        })?;

        Ok(())
    }

    fn update_vote(
        sender: &T::AccountId,
        voter: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        // update voter members
        let mut voter_members = Self::voter_members();
        let mut new_voter_members: MemberOf<T> = BinaryHeap::new();

        // iter voter members, and removed elements
        for i in voter_members.drain() {
            let mut data = i;
            if &data.account == voter {
                // current voter add amount
                let total_amount = data
                    .amount
                    .checked_add(&amount)
                    .ok_or(Error::<T>::Overflow)?;
                data.amount = total_amount;

                // current sender has not pledge
                let mut has_pledger = false;

                for j in data.pledger.iter_mut() {
                    if &j.account == sender {
                        // current sender has pledger
                        has_pledger = true;

                        // current sender add to total amount
                        let pledger_total_amount =
                            j.amount.checked_add(&amount).ok_or(Error::<T>::Overflow)?;

                        // update the latest block
                        j.block_number = <frame_system::Module<T>>::block_number();

                        // set current pledger total amount
                        j.amount = pledger_total_amount;
                    }
                }

                if !has_pledger {
                    data.pledger.push(Pledger {
                        account: sender.clone(),
                        block_number: <frame_system::Module<T>>::block_number(),
                        amount: amount,
                    });
                }
            }
            // push new voter members
            new_voter_members.push(data);
        }

        // update storage from voter members
        <VoterMembers<T>>::put(new_voter_members);

        Ok(())
    }

    // lock currency
    fn lock_currency(account: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
        let lock_balance = pallet_balances::Module::<T>::locks(account);
        let vote_lock = lock_balance.iter().find(|b| b.id == ELECTIONS_ID);
        if let Some(lock) = vote_lock {
            let balance = lock.amount;
            let total_lock_balance = amount.checked_add(&balance).ok_or(Error::<T>::Overflow)?;
            pallet_balances::Module::<T>::extend_lock(
                ELECTIONS_ID,
                account,
                total_lock_balance,
                WithdrawReasons::all(),
            );
        } else {
            pallet_balances::Module::<T>::set_lock(
                ELECTIONS_ID,
                account,
                amount,
                WithdrawReasons::all(),
            );
        }
        Ok(())
    }

    pub fn start_proposal(start: T::BlockNumber) -> T::VoteIndex {
        let candidate_period = T::VotePeriod::get();
        let end_block = start.saturating_add(candidate_period);

        // initialize
        <Candidates<T>>::kill();
        <VoterMembers<T>>::kill();

        // set start block number
        <StartBlockNumber<T>>::put(start);

        // set end block number
        <EndBlockNumber<T>>::put(end_block);

        <VoteRoundCount<T>>::put(Self::vote_round_count() + One::one());

        let count = Self::vote_round_count();
        Self::deposit_event(RawEvent::StartProposal(start, end_block, count));

        count
    }
}

impl<T: Trait> Referendum<T::BlockNumber, T::VoteIndex, MemberOf<T>> for Module<T> {
    fn proposal(start: T::BlockNumber) -> T::VoteIndex {
        Self::start_proposal(start)
    }

    fn get_round() -> T::VoteIndex {
        Self::vote_round_count()
    }

    fn get_result(index: T::VoteIndex) -> Option<MemberOf<T>> {
        if Self::vote_round_count() == index {
            return Some(Self::voter_members());
        } else {
            None
        }
    }
}
