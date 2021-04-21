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

use crate::*;
use crate::{Module, Trait};
use frame_support::traits::{OnFinalize, OnInitialize};
use frame_support::{impl_outer_origin, parameter_types, weights::Weight};
use frame_system as system;
use pallet_balances as balances;
use sp_core::{crypto::key_types, H256};
use sp_runtime::{
    testing::{Header, UintAuthorityId},
    traits::{BlakeTwo256, IdentityLookup, OpaqueKeys},
    KeyTypeId, Perbill,
};

pub const ALICE: <Test as system::Trait>::AccountId = 1;
pub const BOB: <Test as system::Trait>::AccountId = 2;
pub const CHRIS: <Test as system::Trait>::AccountId = 3;
pub const DAVE: <Test as system::Trait>::AccountId = 4;
pub const EVE: <Test as system::Trait>::AccountId = 5;
pub const FERDIE: <Test as system::Trait>::AccountId = 6;

pub const ACCOUNT_1: <Test as system::Trait>::AccountId = 11;
pub const ACCOUNT_2: <Test as system::Trait>::AccountId = 12;
pub const ACCOUNT_3: <Test as system::Trait>::AccountId = 13;
pub const ACCOUNT_4: <Test as system::Trait>::AccountId = 14;
pub const ACCOUNT_5: <Test as system::Trait>::AccountId = 15;
pub const ACCOUNT_6: <Test as system::Trait>::AccountId = 16;
pub const ACCOUNT_7: <Test as system::Trait>::AccountId = 17;
pub const ACCOUNT_8: <Test as system::Trait>::AccountId = 18;
pub const ACCOUNT_9: <Test as system::Trait>::AccountId = 19;
pub const ACCOUNT_10: <Test as system::Trait>::AccountId = 20;
pub const ACCOUNT_11: <Test as system::Trait>::AccountId = 21;
pub const ACCOUNT_12: <Test as system::Trait>::AccountId = 22;
pub const ACCOUNT_13: <Test as system::Trait>::AccountId = 23;
pub const ACCOUNT_14: <Test as system::Trait>::AccountId = 24;
pub const ACCOUNT_15: <Test as system::Trait>::AccountId = 25;
pub const ACCOUNT_16: <Test as system::Trait>::AccountId = 26;
pub const ACCOUNT_17: <Test as system::Trait>::AccountId = 27;
pub const ACCOUNT_18: <Test as system::Trait>::AccountId = 28;
pub const ACCOUNT_19: <Test as system::Trait>::AccountId = 29;
pub const ACCOUNT_20: <Test as system::Trait>::AccountId = 30;
pub const ACCOUNT_21: <Test as system::Trait>::AccountId = 31;

pub const ALL_ACCOUNT: [<Test as system::Trait>::AccountId; 27] = [
    ALICE, BOB, CHRIS, DAVE, EVE, FERDIE, ACCOUNT_1, ACCOUNT_2, ACCOUNT_3, ACCOUNT_4, ACCOUNT_5,
    ACCOUNT_6, ACCOUNT_7, ACCOUNT_8, ACCOUNT_9, ACCOUNT_10, ACCOUNT_11, ACCOUNT_12, ACCOUNT_13,
    ACCOUNT_14, ACCOUNT_15, ACCOUNT_16, ACCOUNT_17, ACCOUNT_18, ACCOUNT_19, ACCOUNT_20, ACCOUNT_21,
];

impl_outer_origin! {
    pub enum Origin for Test {}
}

// Configure a mock runtime to test the pallet.
type AccountId = u64;
pub struct TestSessionHandler;
impl pallet_session::SessionHandler<AccountId> for TestSessionHandler {
    const KEY_TYPE_IDS: &'static [KeyTypeId] = &[key_types::DUMMY];

    fn on_new_session<Ks: OpaqueKeys>(
        _changed: bool,
        _validators: &[(AccountId, Ks)],
        _queued_validators: &[(AccountId, Ks)],
    ) {
    }

    fn on_disabled(_validator_index: usize) {}

    fn on_genesis_session<Ks: OpaqueKeys>(_validators: &[(AccountId, Ks)]) {}
}

#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    pub const ExistentialDeposit: u64 = 1;

    pub const VotePeriod: u32 = 40;
    pub const MinimumVotingLock: u64 = 1000;

    pub const Period: u32 = 1;
    pub const Offset: u32 = 0;

    pub const StartCouncil: u32 = 10;
    pub const CouncilTerm: u32 = 60;
    pub const MinValidators: u32 = 4;
    pub const MaxValidators: u32 = 21;
}

impl system::Trait for Test {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Call = ();
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type DbWeight = ();
    type BlockExecutionWeight = ();
    type ExtrinsicBaseWeight = ();
    type MaximumExtrinsicWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type PalletInfo = ();
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
}

impl balances::Trait for Test {
    type Balance = u64;
    type MaxLocks = ();
    type Event = ();
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

impl fuso_pallet_elections::Trait for Test {
    type Event = ();
    type VotePeriod = VotePeriod;
    type MinimumVotingLock = MinimumVotingLock;
    type VoteIndex = u32;
    type Currency = balances::Module<Self>;
}

impl pallet_session::Trait for Test {
    type Event = ();
    type ValidatorId = <Self as frame_system::Trait>::AccountId;
    type ValidatorIdOf = ValidatorOf<Self>;
    type ShouldEndSession = CouncilModule;
    type NextSessionRotation = CouncilModule;
    type SessionManager = CouncilModule;
    type SessionHandler = TestSessionHandler;
    type Keys = UintAuthorityId;
    type DisabledValidatorsThreshold = ();
    type WeightInfo = ();
}

impl Trait for Test {
    type Event = ();
    type CouncilTerm = CouncilTerm;
    type MinValidators = MinValidators;
    type MaxValidators = MaxValidators;
    type Elections = ElectionsModule;
    type StartCouncil = StartCouncil;
}

pub type CouncilModule = Module<Test>;
pub type ElectionsModule = fuso_pallet_elections::Module<Test>;
pub type SessionModule = pallet_session::Module<Test>;
pub type System = frame_system::Module<Test>;
pub type Balances = pallet_balances::Module<Test>;

pub fn run_to_block(block: u64) {
    while System::block_number() < block {
        CouncilModule::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        CouncilModule::on_initialize(System::block_number());
    }
}

// Build genesis storage according to the mock runtime.
pub fn council_test_ext() -> sp_io::TestExternalities {
    let initial_authorities = vec![ALICE, BOB, CHRIS];

    let mut t = system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into();

    balances::GenesisConfig::<Test> {
        balances: vec![
            (ALICE, 6000000),
            (BOB, 6100000),
            (CHRIS, 52000),
            (DAVE, 53000),
            (EVE, 54000),
            (FERDIE, 55000),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    crate::GenesisConfig::<Test> {
        validators: initial_authorities
            .iter()
            .map(|x| x.clone())
            .collect::<Vec<_>>(),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
