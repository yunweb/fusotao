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

use frame_support::{assert_noop, assert_ok};

use crate::mock::*;
use crate::*;

// test samsara
#[test]
fn test_not_gt_term_duration() {
    samsara_test_ext().execute_with(|| {
        run_to_block(8);

        let vitality = SamsaraModule::vitality();
        let vitality_total = SamsaraModule::vitality_total();
        let start_vote = SamsaraModule::start_vote();
        let voter_members = SamsaraModule::voter_members();

        assert_eq!(vitality.len(), 0);
        assert_eq!(vitality_total, 0);
        assert_eq!(start_vote, None);
        assert_eq!(voter_members.len(), 0);

        // test create vote, AmountZero
        assert_noop!(
            SamsaraModule::vote(Origin::signed(1), 0),
            Error::<Test>::AmountZero
        );

        // test create vote, InsufficientBalance
        assert_noop!(
            SamsaraModule::vote(Origin::signed(10), 100),
            Error::<Test>::InsufficientBalance
        );

        // test create vote, VoteNotStarted
        assert_noop!(
            SamsaraModule::vote(Origin::signed(ALICE), 100),
            Error::<Test>::VoteNotStarted
        );
    });
}

#[test]
fn test_gt_term_duration_and_not_to_vitality() {
    samsara_test_ext().execute_with(|| {
        run_to_block(108);

        let vitality = SamsaraModule::vitality();
        let vitality_total = SamsaraModule::vitality_total();
        let start_vote = SamsaraModule::start_vote();
        let voter_members = SamsaraModule::voter_members();

        assert_eq!(vitality.len(), 7);
        assert_eq!(vitality_total, 37030336000);
        assert_eq!(start_vote, None);
        assert_eq!(voter_members.len(), 0);

        // test create vote, VoteNotStarted
        assert_noop!(
            SamsaraModule::vote(Origin::signed(ALICE), 100),
            Error::<Test>::VoteNotStarted
        );
    });
}

#[test]
fn test_gt_term_duration_and_vitality_too_low() {
    samsara_test_ext().execute_with(|| {
        run_to_block(130);

        let vitality = SamsaraModule::vitality();
        let vitality_total = SamsaraModule::vitality_total();
        let start_vote = SamsaraModule::start_vote();
        let voter_members = SamsaraModule::voter_members();

        assert_eq!(vitality.len(), 21);
        assert_eq!(vitality_total, 111091008000);
        assert_eq!(start_vote, Some(122));
        assert_eq!(voter_members.len(), 0);

        // test create vote, Ok
        assert_ok!(SamsaraModule::vote(Origin::signed(ALICE), 100));

        // assert voter member length is 1
        assert_eq!(SamsaraModule::voter_members().len(), 1);

        // assert not lock balance
        assert_eq!(Balances::usable_balance(&ALICE), 59999999900);

        // create vote again
        assert_ok!(SamsaraModule::vote(Origin::signed(ALICE), 400));

        // assert voter member length is 2
        assert_eq!(SamsaraModule::voter_members().len(), 2);

        // assert not lock balance
        assert_eq!(Balances::usable_balance(&ALICE), 59999999500);

        // create vote again
        assert_ok!(SamsaraModule::vote(Origin::signed(BOB), 10000));

        // assert voter member length is 3
        assert_eq!(SamsaraModule::voter_members().len(), 3);

        // assert not lock balance
        assert_eq!(Balances::usable_balance(&BOB), 609999990000);

        // vote over, new vote
        run_to_block(185);

        assert_eq!(SamsaraModule::vitality().len(), 21);
        assert_eq!(SamsaraModule::vitality_total(), 111091008000);
        assert_eq!(SamsaraModule::start_vote(), Some(184));
        assert_eq!(SamsaraModule::voter_members().len(), 0);
        // assert unlock balance
        assert_eq!(Balances::usable_balance(&ALICE), 60000000000);
        assert_eq!(Balances::usable_balance(&BOB), 610000000000);
    });
}
