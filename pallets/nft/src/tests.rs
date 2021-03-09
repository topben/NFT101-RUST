use crate::mock::*;
use super::*;
use frame_support::{assert_ok};

#[test]
fn test_ntf_create() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_ok!(NftModule::create(Origin::signed(1), "hello".into()));
		let lock_event = TestEvent::nft_event(RawEvent::NftCreated(1, 0));
		assert!(System::events().iter().any(|a| a.event == lock_event));
	});
}