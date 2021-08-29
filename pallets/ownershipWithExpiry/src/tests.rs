use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_works() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(OwnershipWithExpiry::create_ownership(
			Origin::signed(1),
			vec![10, 20, 30],
			10u64
		));
	});
}
