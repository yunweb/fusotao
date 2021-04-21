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
use crate::mock::*;
use frame_support::assert_ok;

// test council
#[test]
fn test_init_validators() {
    council_test_ext().execute_with(|| {
        let current_validators = SessionModule::validators();
        let validators = CouncilModule::validators();
        let members = CouncilModule::members();
        let end_session_block = CouncilModule::end_session_block();

        // session validators length: 0
        assert_eq!(current_validators.len(), 0);
        // council validators length: 3
        assert_eq!(validators.len(), 3);
        assert_eq!(validators.contains(&ALICE), true);
        assert_eq!(validators.contains(&BOB), true);
        assert_eq!(validators.contains(&CHRIS), true);
        // voter member length: 0
        assert_eq!(members.len(), 0);
        // new session block: 10
        assert_eq!(end_session_block, 10);

        SessionModule::rotate_session();
        // session validators length: 3
        assert_eq!(SessionModule::validators().len(), 0);
    });
}

#[test]
fn test_new_session() {
    council_test_ext().execute_with(|| {
        run_to_block(12);

        // council voter members length: 0
        assert_eq!(CouncilModule::members().len(), 0);

        assert_ok!(ElectionsModule::add_candidate(Origin::signed(ALICE), ALICE));
        assert_ok!(ElectionsModule::add_candidate(Origin::signed(ALICE), BOB));
        assert_ok!(ElectionsModule::add_candidate(Origin::signed(ALICE), CHRIS));
        assert_ok!(ElectionsModule::add_candidate(Origin::signed(ALICE), DAVE));
        assert_ok!(ElectionsModule::add_candidate(Origin::signed(ALICE), EVE));
        assert_ok!(ElectionsModule::vote(Origin::signed(ALICE), ALICE, 1, 2000));
        assert_ok!(ElectionsModule::vote(Origin::signed(ALICE), BOB, 1, 2000));
        assert_ok!(ElectionsModule::vote(Origin::signed(ALICE), CHRIS, 1, 2000));
        assert_ok!(ElectionsModule::vote(Origin::signed(ALICE), DAVE, 1, 2000));
        assert_ok!(ElectionsModule::vote(Origin::signed(ALICE), EVE, 1, 2000));

        run_to_block(75);
        SessionModule::rotate_session();
        // council voter members length: 5
        assert_eq!(CouncilModule::members().len(), 5);

        // session validators length: 5
        assert_eq!(SessionModule::validators().len(), 5);

        run_to_block(155);
        SessionModule::rotate_session();
        // new session validators length: 5
        assert_eq!(SessionModule::validators().len(), 5);

        // if lt 4 session, also use last sessions
        assert_ok!(ElectionsModule::add_candidate(Origin::signed(ALICE), ALICE));
        assert_ok!(ElectionsModule::add_candidate(Origin::signed(ALICE), BOB));
        assert_ok!(ElectionsModule::add_candidate(Origin::signed(ALICE), CHRIS));
        assert_ok!(ElectionsModule::vote(Origin::signed(ALICE), ALICE, 3, 2000));
        assert_ok!(ElectionsModule::vote(Origin::signed(ALICE), BOB, 3, 2000));
        assert_ok!(ElectionsModule::vote(Origin::signed(ALICE), CHRIS, 3, 2000));

        run_to_block(215);
        SessionModule::rotate_session();
        // council voter members length: 5
        assert_eq!(CouncilModule::members().len(), 5);
        // new session validators length: 5
        assert_eq!(SessionModule::validators().len(), 5);

        // if gt 21 voter
        for account in ALL_ACCOUNT.iter() {
            // add candidate
            assert_ok!(ElectionsModule::add_candidate(
                Origin::signed(ALICE),
                *account
            ));
            // vote
            assert_ok!(ElectionsModule::vote(
                Origin::signed(ALICE),
                *account,
                4,
                2000
            ));
        }

        run_to_block(275);
        SessionModule::rotate_session();
        // council voter members length: 21
        assert_eq!(CouncilModule::members().len(), 21);
        // new session validators length: 21
        assert_eq!(SessionModule::validators().len(), 21);
    });
}

#[test]
fn test_change_lock_id() {
    council_test_ext().execute_with(|| {
        run_to_block(12);

        // lt 4 voter
        assert_ok!(ElectionsModule::add_candidate(Origin::signed(ALICE), ALICE));
        assert_ok!(ElectionsModule::add_candidate(Origin::signed(ALICE), BOB));
        assert_ok!(ElectionsModule::add_candidate(Origin::signed(ALICE), CHRIS));
        assert_ok!(ElectionsModule::vote(Origin::signed(ALICE), ALICE, 1, 2000));
        assert_ok!(ElectionsModule::vote(Origin::signed(ALICE), BOB, 1, 2000));
        assert_ok!(ElectionsModule::vote(Origin::signed(ALICE), CHRIS, 1, 2000));

        let alice_lock_balance = Balances::locks(&ALICE);
        // not lock currency
        assert_eq!(alice_lock_balance.len(), 1);
        assert_eq!(alice_lock_balance[0].amount, 6000);

        run_to_block(75);
        SessionModule::rotate_session();
        let alice_lock_balance = Balances::locks(&ALICE);
        // not lock currency
        assert_eq!(alice_lock_balance.len(), 0);

        run_to_block(80);
        SessionModule::rotate_session();

        assert_ok!(ElectionsModule::add_candidate(Origin::signed(ALICE), ALICE));
        assert_ok!(ElectionsModule::add_candidate(Origin::signed(ALICE), BOB));
        assert_ok!(ElectionsModule::add_candidate(Origin::signed(ALICE), CHRIS));
        assert_ok!(ElectionsModule::add_candidate(Origin::signed(ALICE), DAVE));
        assert_ok!(ElectionsModule::add_candidate(Origin::signed(ALICE), EVE));
        assert_ok!(ElectionsModule::vote(Origin::signed(ALICE), ALICE, 2, 2000));
        assert_ok!(ElectionsModule::vote(Origin::signed(ALICE), BOB, 2, 2000));
        assert_ok!(ElectionsModule::vote(Origin::signed(ALICE), CHRIS, 2, 2000));
        assert_ok!(ElectionsModule::vote(Origin::signed(ALICE), DAVE, 2, 2000));
        assert_ok!(ElectionsModule::vote(Origin::signed(ALICE), EVE, 2, 2000));
        assert_ok!(ElectionsModule::vote(Origin::signed(BOB), ALICE, 2, 3000));
        assert_ok!(ElectionsModule::vote(Origin::signed(BOB), BOB, 2, 3000));
        assert_ok!(ElectionsModule::vote(Origin::signed(BOB), CHRIS, 2, 3000));
        assert_ok!(ElectionsModule::vote(Origin::signed(BOB), DAVE, 2, 3000));
        assert_ok!(ElectionsModule::vote(Origin::signed(BOB), EVE, 2, 3000));

        let alice_lock_balance = Balances::locks(&ALICE);
        let bob_lock_balance = Balances::locks(&BOB);
        let dave_lock_balance = Balances::locks(&DAVE);

        assert_eq!(alice_lock_balance.len(), 1);
        assert_eq!(bob_lock_balance.len(), 1);
        assert_eq!(dave_lock_balance.len(), 0);
        assert_eq!(alice_lock_balance[0].amount, 10000);
        assert_eq!(bob_lock_balance[0].amount, 15000);

        run_to_block(135);
        SessionModule::rotate_session();
        let alice_lock_balance = Balances::locks(&ALICE);
        let bob_lock_balance = Balances::locks(&BOB);
        let dave_lock_balance = Balances::locks(&DAVE);

        assert_eq!(alice_lock_balance.len(), 1);
        assert_eq!(bob_lock_balance.len(), 1);
        assert_eq!(dave_lock_balance.len(), 0);
        assert_eq!(alice_lock_balance[0].amount, 10000);
        assert_eq!(bob_lock_balance[0].amount, 15000);

        run_to_block(195);
        SessionModule::rotate_session();

        assert_ok!(ElectionsModule::add_candidate(Origin::signed(ALICE), ALICE));
        assert_ok!(ElectionsModule::add_candidate(Origin::signed(ALICE), BOB));
        assert_ok!(ElectionsModule::add_candidate(Origin::signed(ALICE), CHRIS));
        assert_ok!(ElectionsModule::add_candidate(Origin::signed(ALICE), DAVE));
        assert_ok!(ElectionsModule::add_candidate(Origin::signed(ALICE), EVE));
        assert_ok!(ElectionsModule::vote(Origin::signed(ALICE), ALICE, 4, 4000));
        assert_ok!(ElectionsModule::vote(Origin::signed(ALICE), BOB, 4, 4000));
        assert_ok!(ElectionsModule::vote(Origin::signed(ALICE), CHRIS, 4, 4000));
        assert_ok!(ElectionsModule::vote(Origin::signed(ALICE), DAVE, 4, 4000));
        assert_ok!(ElectionsModule::vote(Origin::signed(ALICE), EVE, 4, 4000));
        assert_ok!(ElectionsModule::vote(Origin::signed(BOB), ALICE, 4, 6000));
        assert_ok!(ElectionsModule::vote(Origin::signed(BOB), BOB, 4, 6000));
        assert_ok!(ElectionsModule::vote(Origin::signed(BOB), CHRIS, 4, 6000));
        assert_ok!(ElectionsModule::vote(Origin::signed(BOB), DAVE, 4, 6000));
        assert_ok!(ElectionsModule::vote(Origin::signed(BOB), EVE, 4, 6000));

        let alice_lock_balance = Balances::locks(&ALICE);
        let bob_lock_balance = Balances::locks(&BOB);
        let dave_lock_balance = Balances::locks(&DAVE);

        assert_eq!(alice_lock_balance.len(), 2);
        assert_eq!(bob_lock_balance.len(), 2);
        assert_eq!(dave_lock_balance.len(), 0);
        assert_eq!(alice_lock_balance[0].amount, 10000);
        assert_eq!(alice_lock_balance[1].amount, 20000);
        assert_eq!(bob_lock_balance[0].amount, 15000);
        assert_eq!(bob_lock_balance[1].amount, 30000);

        run_to_block(255);
        SessionModule::rotate_session();

        let alice_lock_balance = Balances::locks(&ALICE);
        let bob_lock_balance = Balances::locks(&BOB);
        let dave_lock_balance = Balances::locks(&DAVE);

        assert_eq!(alice_lock_balance.len(), 1);
        assert_eq!(bob_lock_balance.len(), 1);
        assert_eq!(dave_lock_balance.len(), 0);
        assert_eq!(alice_lock_balance[0].amount, 20000);
        assert_eq!(bob_lock_balance[0].amount, 30000);


        run_to_block(315);
        SessionModule::rotate_session();

        // if gt 21 voter
        for (index, account) in ALL_ACCOUNT.iter().enumerate() {
            let index_of = TryInto::<u64>::try_into(index).ok().unwrap();
            let amount = index_of.saturating_mul(10).saturating_add(2000);
            // add candidate
            assert_ok!(ElectionsModule::add_candidate(
                Origin::signed(ALICE),
                *account
            ));
            // vote
            assert_ok!(ElectionsModule::vote(
                Origin::signed(ALICE),
                *account,
                6,
                amount
            ));
        }

        let alice_lock_balance = Balances::locks(&ALICE);
        let bob_lock_balance = Balances::locks(&BOB);

        assert_eq!(alice_lock_balance.len(), 2);
        assert_eq!(bob_lock_balance.len(), 1);
        assert_eq!(alice_lock_balance[0].amount, 20000);
        // total 27 voter amount
        assert_eq!(alice_lock_balance[1].amount, 57510);

        run_to_block(385);
        SessionModule::rotate_session();

        let alice_lock_balance = Balances::locks(&ALICE);
        let bob_lock_balance = Balances::locks(&BOB);

        assert_eq!(alice_lock_balance.len(), 1);
        assert_eq!(bob_lock_balance.len(), 0);
        // total 21 voter amount: 45360
        assert_eq!(alice_lock_balance[0].amount, 45360);
    });
}
