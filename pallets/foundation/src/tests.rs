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

use crate::mock::*;
use crate::*;
use frame_support::assert_ok;
use frame_support::dispatch::DispatchResult;

// test foundation
#[test]
fn test_foundation_storage() {
    foundation_test_ext().execute_with(|| {
        assert_ok!(init_reserve_balance());
        let alice_reserve_balance = FoundationModule::foundation(ALICE);
        let bob_reserve_balance = FoundationModule::foundation(BOB);
        assert_eq!(alice_reserve_balance, 50000000000);
        assert_eq!(bob_reserve_balance, 51000000000);

        // assert have two map storage
        let len = <FoundationModule as crate::Store>::Foundation::iter().count();
        assert_eq!(len, 2);
    });
}

#[test]
fn test_delay_not_unlock() {
    foundation_test_ext().execute_with(|| {
        assert_ok!(init_reserve_balance());
        run_to_block(1);
        assert_eq!(Balances::reserved_balance(&ALICE), 50000000000);
        assert_eq!(Balances::reserved_balance(&BOB), 51000000000);

        run_to_block(8);
        assert_eq!(Balances::reserved_balance(&ALICE), 50000000000);
        assert_eq!(Balances::reserved_balance(&BOB), 51000000000);

        run_to_block(9);
        assert_eq!(Balances::reserved_balance(&ALICE), 50000000000);
        assert_eq!(Balances::reserved_balance(&BOB), 51000000000);
    });
}

#[test]
fn test_first_unlock() {
    foundation_test_ext().execute_with(|| {
        assert_ok!(init_reserve_balance());
        run_to_block(10);

        // first free balance
        // free balance: 50000000, so reserved balance will be 49950000000
        assert_eq!(Balances::reserved_balance(&ALICE), 49950000000);
        // free balance: 51000000, so reserved balance will be 50949000000
        assert_eq!(Balances::reserved_balance(&BOB), 50949000000);

        run_to_block(29);
        // free balance: 50000000, so reserved balance will be 49950000000
        assert_eq!(Balances::reserved_balance(&ALICE), 49950000000);
        // free balance: 51000000, so reserved balance will be 50949000000
        assert_eq!(Balances::reserved_balance(&BOB), 50949000000);
    });
}

#[test]
fn test_middle_unlock() {
    foundation_test_ext().execute_with(|| {
        assert_ok!(init_reserve_balance());
        run_to_block(185);

        assert_eq!(Balances::reserved_balance(&ALICE), 49550000000);
        assert_eq!(Balances::reserved_balance(&BOB), 50541000000);

        run_to_block(5678);
        assert_eq!(Balances::reserved_balance(&ALICE), 35800000000);
        assert_eq!(Balances::reserved_balance(&BOB), 36516000000);

        run_to_block(13685);
        assert_eq!(Balances::reserved_balance(&ALICE), 15800000000);
        assert_eq!(Balances::reserved_balance(&BOB), 16116000000);
    });
}

#[test]
fn test_last_unlock() {
    foundation_test_ext().execute_with(|| {
        assert_ok!(init_reserve_balance());
        run_to_block(19976);

        assert_eq!(Balances::reserved_balance(&ALICE), 50000000);
        assert_eq!(Balances::reserved_balance(&BOB), 51000000);
    });
}

#[test]
fn test_last_free_all_balance() {
    foundation_test_ext().execute_with(|| {
        assert_ok!(init_reserve_balance());
        run_to_block(19996);

        assert_eq!(Balances::reserved_balance(&ALICE), 0);
        assert_eq!(Balances::reserved_balance(&BOB), 0);
    });
}

#[test]
fn test_already_free_all_balance() {
    foundation_test_ext().execute_with(|| {
        assert_ok!(init_reserve_balance());
        run_to_block(20000);

        assert_eq!(Balances::reserved_balance(&ALICE), 0);
        assert_eq!(Balances::reserved_balance(&BOB), 0);

        run_to_block(30000);

        assert_eq!(Balances::reserved_balance(&ALICE), 0);
        assert_eq!(Balances::reserved_balance(&BOB), 0);
    });
}

#[test]
fn other_reason_to_reserve_balance() {
    foundation_test_ext().execute_with(|| {
        assert_ok!(init_reserve_balance());

        run_to_block(5678);
        assert_eq!(Balances::reserved_balance(&ALICE), 35800000000);
        assert_eq!(Balances::reserved_balance(&BOB), 36516000000);

        // other reason reserve balance
        // add 2000 to reserve balance
        assert_ok!(Balances::reserve(&ALICE, 2000));
        // add 182346 to reserve balance
        assert_ok!(Balances::reserve(&BOB, 182346));
        assert_eq!(Balances::reserved_balance(&ALICE), 35800002000);
        assert_eq!(Balances::reserved_balance(&BOB), 36516182346);

        run_to_block(19996);
        // last cycle to free all fund balance, but other reason balance reserved
        assert_eq!(Balances::reserved_balance(&ALICE), 2000);
        assert_eq!(Balances::reserved_balance(&BOB), 182346);

        run_to_block(25000);

        assert_eq!(Balances::reserved_balance(&ALICE), 2000);
        assert_eq!(Balances::reserved_balance(&BOB), 182346);
    });
}
