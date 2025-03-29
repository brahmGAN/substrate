//! Tests for NFTMap pallet

use crate::*;
use frame_support::{assert_noop, assert_ok};
use mock::{new_test_ext, new_test_ext_with_nfts, RuntimeEvent, RuntimeOrigin, System, Test, NFTMap};
use sp_runtime::DispatchError;

#[test]
fn add_nft_works() {
    new_test_ext().execute_with(|| {
        // Set block number to 1 for event emission
        System::set_block_number(1);

        // Try to add an NFT as a non-root account (should fail)
        assert_noop!(
            NFTMap::add_nft(RuntimeOrigin::signed(1), 1, 42),
            DispatchError::BadOrigin
        );

        // Add an NFT with root origin (should succeed)
        assert_ok!(NFTMap::add_nft(RuntimeOrigin::root(), 1, 42));

        // Check storage
        assert_eq!(NFTMap::nfts(1), Some(42));

        // Check event
        System::assert_last_event(RuntimeEvent::NFTMap(crate::Event::NFTAdded { who: 1, val: 42 }));

        // Try to add the same NFT again (should fail)
        assert_noop!(
            NFTMap::add_nft(RuntimeOrigin::root(), 1, 99),
            Error::<Test>::AlreadyAdded
        );
    });
}

#[test]
fn update_nft_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // First add an NFT
        assert_ok!(NFTMap::add_nft(RuntimeOrigin::root(), 1, 42));

        // Try to update non-existent NFT (should fail)
        assert_noop!(
            NFTMap::update_nft(RuntimeOrigin::root(), 2, 99),
            Error::<Test>::NotPresent
        );

        // Try to update as non-root (should fail)
        assert_noop!(
            NFTMap::update_nft(RuntimeOrigin::signed(1), 1, 99),
            DispatchError::BadOrigin
        );

        // Update NFT value (should succeed)
        assert_ok!(NFTMap::update_nft(RuntimeOrigin::root(), 1, 99));

        // Check storage updated
        assert_eq!(NFTMap::nfts(1), Some(99));

        // Check event
        System::assert_last_event(RuntimeEvent::NFTMap(crate::Event::UpdatededNFT { who: 1, val: 99 }));
    });
}

#[test]
fn delete_nft_works() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        // First add an NFT
        assert_ok!(NFTMap::add_nft(RuntimeOrigin::root(), 1, 42));

        // Try to delete non-existent NFT (should fail)
        assert_noop!(
            NFTMap::delete_nft(RuntimeOrigin::root(), 2),
            Error::<Test>::NotPresent
        );

        // Try to delete as non-root (should fail)
        assert_noop!(
            NFTMap::delete_nft(RuntimeOrigin::signed(1), 1),
            DispatchError::BadOrigin
        );

        // Delete NFT (should succeed)
        assert_ok!(NFTMap::delete_nft(RuntimeOrigin::root(), 1));

        // Check NFT was removed from storage
        assert_eq!(NFTMap::nfts(1), None);

        // Check event
        System::assert_last_event(RuntimeEvent::NFTMap(crate::Event::RemovedNFT { who: 1 }));

        // Try to delete again (should fail)
        assert_noop!(
            NFTMap::delete_nft(RuntimeOrigin::root(), 1),
            Error::<Test>::NotPresent
        );
    });
}

#[test]
fn genesis_config_works() {
    new_test_ext_with_nfts(vec![(1, 42), (2, 84)]).execute_with(|| {
        // Check the genesis config was applied correctly
        assert_eq!(NFTMap::nfts(1), Some(42));
        assert_eq!(NFTMap::nfts(2), Some(84));
        assert_eq!(NFTMap::nfts(3), None);
    });
}