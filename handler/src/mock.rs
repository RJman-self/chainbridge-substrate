#![cfg(test)]

use std::collections::BTreeMap;

use super::*;
use crate::{self as handler};

use frame_support::{ord_parameter_types, parameter_types};
use frame_system::EnsureSignedBy;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};
use xpallet_assets::{AssetInfo, AssetRestrictions, Chain};

// pub(crate) type AccountId = AccountId32;
pub(crate) type AccountId = u128;
pub(crate) type BlockNumber = u64;
pub(crate) type Balance = u128;
pub(crate) type Amount = i128;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const X_ETH: AssetId = 3;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
    type BaseCallFilter = ();
    type BlockWeights = ();
    type BlockLength = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 0;
}
impl pallet_balances::Config for Test {
    type MaxLocks = ();
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

parameter_types! {
    pub const ChainXAssetId: AssetId = 0;
}

impl xpallet_assets_registrar::Config for Test {
    type Event = Event;
    type NativeAssetId = ChainXAssetId;
    type RegistrarHandler = ();
    type WeightInfo = ();
}

impl xpallet_assets::Config for Test {
    type Event = Event;
    type Currency = Balances;
    type Amount = Amount;
    type TreasuryAccount = ();
    type OnCreatedAccount = frame_system::Provider<Test>;
    type OnAssetChanged = ();
    type WeightInfo = ();
}

parameter_types! {
    pub const TestChainId: u8 = 5;
    pub const ProposalLifetime: u64 = 50;
    pub XBNBResourceId: chainbridge::ResourceId = chainbridge::derive_resource_id(1, b"xbnb");
    pub XBNB: AssetId = 100;
    pub XETHResourceId: chainbridge::ResourceId = chainbridge::derive_resource_id(0, b"xeth");
    pub XETH: AssetId = X_ETH;
}

ord_parameter_types! {
    pub const AdminOrigin: AccountId = 1;
}

impl chainbridge::Config for Test {
    type Event = Event;
    type AdminOrigin = EnsureSignedBy<AdminOrigin, AccountId>;
    type Proposal = Call;
    type ChainId = TestChainId;
    type ProposalLifetime = ProposalLifetime;
}

ord_parameter_types! {
    pub const RegistorOrigin: AccountId = 1;
}

impl Config for Test {
    type Event = Event;
    type RegistorOrigin = EnsureSignedBy<RegistorOrigin, AccountId>;
    type BridgeOrigin = chainbridge::EnsureBridge<Test>;
}

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        XAssetsRegistrar: xpallet_assets_registrar::{Pallet, Call, Storage, Event, Config},
        XAssets: xpallet_assets::{Pallet, Call, Storage, Event<T>, Config<T>},
        ChainBridge: chainbridge::{Pallet, Call, Storage, Event<T>},
        Handler: handler::{Pallet, Call, Storage, Event<T>},
    }
);

pub(crate) fn eth() -> (AssetId, AssetInfo, AssetRestrictions) {
    (
        X_ETH,
        AssetInfo::new::<Test>(
            b"X-ETH".to_vec(),
            b"X-ETH".to_vec(),
            Chain::Ethereum,
            18,
            b"ChainX's cross-chain Ethereum".to_vec(),
        )
        .unwrap(),
        AssetRestrictions::DESTROY_WITHDRAWAL,
    )
}
pub struct ExtBuilder;
impl Default for ExtBuilder {
    fn default() -> Self {
        Self
    }
}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let mut storage = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        let eth_assets = eth();
        let assets = vec![(eth_assets.0, eth_assets.1, eth_assets.2, true, true)];

        let mut endowed = BTreeMap::new();
        let endowed_info = vec![(ALICE, 1000)];
        endowed.insert(eth_assets.0, endowed_info);

        let mut init_assets = vec![];
        let mut assets_restrictions = vec![];
        for (a, b, c, d, e) in assets {
            init_assets.push((a, b, d, e));
            assets_restrictions.push((a, c))
        }

        let _ = xpallet_assets_registrar::GenesisConfig {
            assets: init_assets,
        }
        .assimilate_storage::<Test>(&mut storage);

        let _ = xpallet_assets::GenesisConfig::<Test> {
            assets_restrictions,
            endowed: endowed,
        }
        .assimilate_storage(&mut storage);

        let ext = sp_io::TestExternalities::new(storage);
        ext
    }
}
