#![cfg(test)]

use super::*;
use frame_support::{construct_runtime, ord_parameter_types, parameter_types};
use frame_system::EnsureSignedBy;
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup};

pub type BlockNumber = u64;
pub type AccountId = u128;

pub const ALICE: AccountId = 1;

mod handler {
    pub use super::super::*;
}

parameter_types! {
    pub const BlockHashCount: BlockNumber = 250;
}

impl frame_system::Config for Runtime {
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Call = Call;
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type BlockWeights = ();
    type BlockLength = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BaseCallFilter = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
}

ord_parameter_types! {
    pub const AdminOrigin: AccountId = 11;
}

parameter_types! {
    pub const LocalChainId: chainbridge::ChainId = 2;
    pub const ProposalLifetime: BlockNumber = 10;
}

impl chainbridge::Config for Runtime {
    type Event = Event;
    type AdminOrigin = EnsureSignedBy<AdminOrigin, AccountId>;
    type Proposal = Call;
    type ChainId = LocalChainId;
    type ProposalLifetime = ProposalLifetime;
}

parameter_types! {
    pub const PcxAssetId: u32 = 0;
}

impl xpallet_assets_registrar::Config for Runtime {
    type Event = Event;
    type NativeAssetId = PcxAssetId;
    type RegistrarHandler = ();
    type WeightInfo = xpallet_assets_registrar::weights::SubstrateWeight<Runtime>;
}

impl xpallet_assets::Config for Runtime {
    type Event = Event;
    type Currency = Balance;
    type Amount = Amount;
    type TreasuryAccount = ();
    type OnCreatedAccount = frame_system::Provider<Runtime>;
    type OnAssetChanged = ();
    type WeightInfo = xpallet_assets::weights::SubstrateWeight<Runtime>;
}

ord_parameter_types! {
    pub const RegistorOrigin: AccountId = 12;
}

impl Config for Runtime {
    type Event = Event;
    type RegistorOrigin = EnsureSignedBy<RegistorOrigin, AccountId>;
    type BridgeOrigin = chainbridge::EnsureBridge<Runtime>;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
        ChainBridge: chainbridge::{Pallet, Call, Storage, Event<T>},
        XAssetsRegistrar: xpallet_assets_registrar::{Pallet, Call, Storage, Event, Config},
        XAssets: xpallet_assets::{Pallet, Call, Storage, Event<T>, Config<T>},
        Hander: handler::{Pallet, Call, Storage, Event<T>},
    }
);

pub struct ExtBuilder {
    endowed_accounts: Vec<(AccountId, CurrencyId, Balance)>,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            endowed_accounts: vec![(ALICE, PcxAssetId::get(), 1_000u128)],
        }
    }
}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap();

        t.into()
    }
}
