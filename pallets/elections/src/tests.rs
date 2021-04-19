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

use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use sp_std::vec::Vec;

// test elections
#[test]
fn test_not_start_proposal() {
    elections_test_ext().execute_with(|| {
        assert_noop!(
            ElectionsModule::add_candidate(Origin::signed(ALICE), BOB),
            Error::<Test>::NoProposalStarted
        );

        run_to_block(1000);
        assert_noop!(
            ElectionsModule::vote(Origin::signed(ALICE), BOB, 1, 10000),
            Error::<Test>::NoProposalStarted
        );
    });
}

#[test]
fn test_add_candidate() {
    elections_test_ext().execute_with(|| {
        ElectionsModule::start_proposal(100);

        let end_block = ElectionsModule::end_block_number();
        let round = ElectionsModule::vote_round_count();
        assert_eq!(end_block, Some(140));
        assert_eq!(round, 1);

        assert_ok!(ElectionsModule::add_candidate(Origin::signed(ALICE), BOB));
        let mut vec = Vec::new();
        vec.push(BOB);
        assert_eq!(ElectionsModule::candidates(), vec);

        // set bob again
        assert_noop!(
            ElectionsModule::add_candidate(Origin::signed(BOB), BOB),
            Error::<Test>::AlreadyIsCandidate
        );
        assert_eq!(ElectionsModule::candidates(), vec);

        // vote BOB
        assert_ok!(ElectionsModule::vote(Origin::signed(ALICE), BOB, 1, 10000));
        // set candidate bob again
        assert_noop!(
            ElectionsModule::add_candidate(Origin::signed(BOB), BOB),
            Error::<Test>::AlreadyIsVoter
        );
    });
}

#[test]
fn test_proposal_over() {
    elections_test_ext().execute_with(|| {
        ElectionsModule::start_proposal(100);

        // proposal
        run_to_block(120);
        assert_ok!(ElectionsModule::add_candidate(Origin::signed(ALICE), BOB));

        // proposal over
        run_to_block(150);
        assert_noop!(
            ElectionsModule::add_candidate(Origin::signed(ALICE), BOB),
            Error::<Test>::ProposalOver
        );
    });
}

#[test]
fn test_vote() {
    elections_test_ext().execute_with(|| {
        ElectionsModule::start_proposal(100);
        run_to_block(120);

        assert_ok!(ElectionsModule::add_candidate(Origin::signed(ALICE), BOB));

        // not candidate
        assert_noop!(
            ElectionsModule::vote(Origin::signed(ALICE), EVE, 1, 0),
            Error::<Test>::NotCandidate
        );

        // amount zero
        assert_noop!(
            ElectionsModule::vote(Origin::signed(ALICE), BOB, 1, 0),
            Error::<Test>::AmountZero
        );

        // low amount
        assert_noop!(
            ElectionsModule::vote(Origin::signed(ALICE), BOB, 1, 900),
            Error::<Test>::AmountTooLow
        );

        // vote BOB
        assert_ok!(ElectionsModule::vote(Origin::signed(ALICE), BOB, 1, 10000));
        assert_eq!(ElectionsModule::voter_members().len(), 1);

        if let Some(members) = ElectionsModule::voter_members().first() {
            let pledger = Pledger {
                account: ALICE,
                block_number: 120u64,
                amount: 10000u64,
            };

            assert_eq!(members.round, 1u32);
            assert_eq!(members.account, BOB);
            assert_eq!(members.amount, 10000u64);
            assert_eq!(members.pledger.len(), 1);
            assert_eq!(members.pledger.get(0), Some(&pledger));
        }

        // alice lock balance: 10000
        assert_eq!(Balances::locks(&ALICE)[0].amount, 10000);

        // add DAVE
        assert_ok!(ElectionsModule::add_candidate(Origin::signed(ALICE), DAVE));
        // vote DAVE
        assert_ok!(ElectionsModule::vote(Origin::signed(ALICE), DAVE, 1, 9000));

        assert_eq!(ElectionsModule::voter_members().len(), 2);

        // alice lock balance: 19000
        assert_eq!(Balances::locks(&ALICE)[0].amount, 19000);

        // ALICE is first voter
        if let Some(members) = ElectionsModule::voter_members().first() {
            let pledger = Pledger {
                account: ALICE,
                block_number: 120u64,
                amount: 10000u64,
            };

            assert_eq!(members.round, 1u32);
            assert_eq!(members.account, BOB);
            assert_eq!(members.amount, 10000u64);
            assert_eq!(members.pledger.len(), 1);
            assert_eq!(members.pledger.get(0), Some(&pledger));
        }

        run_to_block(125);
        // vote DAVE
        assert_ok!(ElectionsModule::vote(Origin::signed(BOB), DAVE, 1, 3000));

        assert_eq!(ElectionsModule::voter_members().len(), 2);
        // bob lock balance: 3000
        assert_eq!(Balances::locks(&BOB)[0].amount, 3000);

        if let Some(members) = ElectionsModule::voter_members().first() {
            let pledger_first = Pledger {
                account: ALICE,
                block_number: 120u64,
                amount: 9000u64,
            };
            let pledger_second = Pledger {
                account: BOB,
                block_number: 125u64,
                amount: 3000u64,
            };

            assert_eq!(members.round, 1u32);
            assert_eq!(members.account, DAVE);
            assert_eq!(members.amount, 12000u64);
            assert_eq!(members.pledger.len(), 2);
            assert_eq!(members.pledger.get(0), Some(&pledger_first));
            assert_eq!(members.pledger.get(1), Some(&pledger_second));
        }

        // test InsufficientBalance
        assert_noop!(
            ElectionsModule::vote(Origin::signed(ALICE), DAVE, 1, 50000),
            Error::<Test>::InsufficientBalance
        );

        run_to_block(300);
        ElectionsModule::start_proposal(300);
        // not current around
        assert_noop!(
            ElectionsModule::vote(Origin::signed(ALICE), BOB, 1, 0),
            Error::<Test>::NotCurrentVoteRound
        );
        assert_eq!(ElectionsModule::get_round(), 2);

        // add DAVE
        assert_ok!(ElectionsModule::add_candidate(Origin::signed(ALICE), ALICE));
        assert_ok!(ElectionsModule::add_candidate(Origin::signed(ALICE), BOB));
        assert_ok!(ElectionsModule::add_candidate(Origin::signed(ALICE), CHRIS));
        assert_ok!(ElectionsModule::add_candidate(Origin::signed(ALICE), DAVE));
        // vote DAVE
        assert_ok!(ElectionsModule::vote(Origin::signed(ALICE), ALICE, 2, 9000));
        assert_ok!(ElectionsModule::vote(Origin::signed(ALICE), BOB, 2, 9000));
        assert_ok!(ElectionsModule::vote(Origin::signed(ALICE), CHRIS, 2, 9000));
        assert_ok!(ElectionsModule::vote(Origin::signed(ALICE), DAVE, 2, 9000));

        assert_eq!(ElectionsModule::voter_members().len(), 4);
    });
}
