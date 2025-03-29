//! Test utilities for NFTMap pallet

use crate as pallet_nftmap;
use frame_support::{
    parameter_types,
    traits::{ConstU16, ConstU64},
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet
frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        NFTMap: pallet_nftmap,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type Nonce = u64;
    type Block = Block;
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    //type Header = sp_runtime::generic::Header<u64, BlakeTwo256>; Not a member
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_nftmap::Config for Test {
    type RuntimeEvent = RuntimeEvent;
}

// Build genesis storage according to the mock runtime
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    sp_io::TestExternalities::new(t)
}

// Build genesis storage with initial NFT mappings
pub fn new_test_ext_with_nfts(nft_mappers: Vec<(u64, u32)>) -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    
    pallet_nftmap::GenesisConfig::<Test> {
        nft_mappers,
    }
    .assimilate_storage(&mut t)
    .unwrap();
    
    sp_io::TestExternalities::new(t)
}