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

use crate::{Module, Trait};
use frame_support::{dispatch::DispatchResult, impl_outer_origin, parameter_types, weights::Weight};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};
use frame_support::traits::{OnInitialize, OnFinalize, ReservableCurrency};
use pallet_balances as balances;
use crate::*;

pub const ALICE: <Test as system::Trait>::AccountId = 1;
pub const BOB: <Test as system::Trait>::AccountId = 2;
pub const CHRIS: <Test as system::Trait>::AccountId = 3;
#[allow(dead_code)]
pub const DAVE: <Test as system::Trait>::AccountId = 4;
pub const TEAM: <Test as system::Trait>::AccountId = 5;

impl_outer_origin! {
    pub enum Origin for Test {}
}

// Configure a mock runtime to test the pallet.

#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    pub const MinimumVotingLock: u64 = 1000;

    pub const ExistentialDeposit: u64 = 1;
    pub const TransferFee: u128 = 0;
    pub const CreationFee: u128 = 0;

    pub const UnlockDelay: u32 = 10;
    pub const UnlockPeriod: u32 = 20;
    pub const UnlockRatioEachPeriod: Perbill = Perbill::from_perthousand(1);
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

impl Trait for Test {
    type Event = ();
    type UnlockDelay = UnlockDelay;
    type UnlockPeriod = UnlockPeriod;
    type UnlockRatioEachPeriod = UnlockRatioEachPeriod;
    type Currency = balances::Module<Self>;
}

pub type FoundationModule = Module<Test>;
pub type System = frame_system::Module<Test>;
pub type Balances = pallet_balances::Module<Test>;

pub fn init_reserve_balance() -> DispatchResult {
    let fund = <FoundationModule as crate::Store>::Foundation::iter();
    for i in fund {
        Balances::reserve(&i.0, i.1)?
    }
    Ok(())
}

pub fn run_to_block(block: u64) {
    while System::block_number() < block {
        FoundationModule::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        FoundationModule::on_initialize(System::block_number());
    }
}

// Build genesis storage according to the mock runtime.
pub fn foundation_test_ext() -> sp_io::TestExternalities {
    let mut t = system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into();

    balances::GenesisConfig::<Test> {
        balances: vec![(ALICE, 50000000000), (BOB, 510000000000), (CHRIS, 52000000000), (DAVE, 530000000000), (TEAM, 540000000000)],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    crate::GenesisConfig::<Test> {
        fund: vec![
            (
                ALICE,
                50000000000,
            ),
            (
                BOB,
                51000000000,
            ),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
