#![cfg(test)]

use super::*;
use frame_support::sp_runtime::DispatchError::BadOrigin;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};

// Update country referendum parameters tests
#[test]
fn update_country_referendum_parameters_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(BOB);
		assert_ok!(GovernanceModule::update_referendum_parameters(
			origin.clone(),
			BOB_COUNTRY_ID,
			REFERENDUM_PARAMETERS
		));
		assert_eq!(
			last_event(),
			Event::Governance(crate::Event::ReferendumParametersUpdated(BOB_COUNTRY_ID))
		);
	});
}

#[test]
fn update_country_referendum_parameters_when_not_country_owner_does_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		assert_noop!(
			GovernanceModule::update_referendum_parameters(origin.clone(), BOB_COUNTRY_ID, REFERENDUM_PARAMETERS),
			Error::<Runtime>::AccountIsNotMetaverseOwner
		);
	});
}

// Creating preimage tests
#[test]
fn create_new_preimage_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		let encoded_proposal = set_balance_proposal(4);
		assert_ok!(GovernanceModule::note_preimage(origin.clone(), encoded_proposal));
		assert_eq!(Balances::free_balance(&ALICE), 99987);
		let hash = set_balance_proposal_hash(4);
		assert_eq!(
			last_event(),
			Event::Governance(crate::Event::PreimageNoted(hash, ALICE, 13))
		);
	});
}

// Creating proposal tests
#[test]
fn create_new_proposal_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		let hash = set_balance_proposal_hash(4);
		add_preimage(hash, false);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		assert_eq!(Balances::free_balance(&ALICE), 99400);
		assert_eq!(
			last_event(),
			Event::Governance(crate::Event::ProposalSubmitted(ALICE, BOB_COUNTRY_ID, 0))
		);
	});
}

#[test]
fn create_new_proposal_when_not_enough_funds_does_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(BOB);
		let hash = set_balance_proposal_hash(4);
		add_preimage(hash, false);
		assert_noop!(
			GovernanceModule::propose(
				origin.clone(),
				BOB_COUNTRY_ID,
				600,
				hash.clone(),
				PROPOSAL_DESCRIPTION.to_vec()
			),
			Error::<Runtime>::InsufficientBalance
		);
	});
}

#[test]
fn create_new_proposal_when_too_small_deposit_does_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(BOB);
		let hash = set_balance_proposal_hash(4);
		add_preimage(hash, false);
		assert_noop!(
			GovernanceModule::propose(
				origin.clone(),
				BOB_COUNTRY_ID,
				40,
				hash.clone(),
				PROPOSAL_DESCRIPTION.to_vec()
			),
			Error::<Runtime>::DepositTooLow
		);
	});
}

#[test]
fn create_new_proposal_when_not_country_member_does_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let hash = set_balance_proposal_hash(4);
		add_preimage(hash, false);
		assert_noop!(
			GovernanceModule::propose(
				Origin::signed(5).clone(),
				ALICE_COUNTRY_ID,
				400,
				hash.clone(),
				PROPOSAL_DESCRIPTION.to_vec()
			),
			Error::<Runtime>::AccountIsNotMetaverseMember
		);
	});
}

#[test]
fn create_new_proposal_when_queue_full_does_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(BOB);
		let hash = set_balance_proposal_hash(4);
		add_preimage(hash, false);
		let parameters = ReferendumParameters {
			voting_threshold: Some(VoteThreshold::RelativeMajority),
			min_proposal_launch_period: 12,
			voting_period: 5,
			enactment_period: 10,
			max_proposals_per_metaverse: 0,
		};
		assert_ok!(GovernanceModule::update_referendum_parameters(
			origin.clone(),
			BOB_COUNTRY_ID,
			parameters
		));
		assert_noop!(
			GovernanceModule::propose(
				origin.clone(),
				BOB_COUNTRY_ID,
				400,
				hash.clone(),
				PROPOSAL_DESCRIPTION.to_vec()
			),
			Error::<Runtime>::ProposalQueueFull
		);
	});
}

// Cancel proposal tests
#[test]
fn cancel_proposal_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		let hash = set_balance_proposal_hash(4);
		add_preimage(hash, false);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		assert_ok!(GovernanceModule::cancel_proposal(origin.clone(), 0, BOB_COUNTRY_ID));
		assert_eq!(Balances::free_balance(&ALICE), 100000);
		assert_eq!(
			last_event(),
			Event::Governance(crate::Event::ProposalCancelled(ALICE, BOB_COUNTRY_ID, 0))
		);
	});
}

#[test]
fn cancel_non_existing_proposal_does_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		assert_noop!(
			GovernanceModule::cancel_proposal(origin.clone(), 0, BOB_COUNTRY_ID),
			Error::<Runtime>::ProposalDoesNotExist
		);
	});
}

#[test]
fn cancel_proposal_that_you_have_not_submitted_does_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		let hash = set_balance_proposal_hash(4);
		add_preimage(hash, false);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		assert_noop!(
			GovernanceModule::cancel_proposal(Origin::signed(BOB), 0, BOB_COUNTRY_ID),
			Error::<Runtime>::NotProposalCreator
		);
	});
}

#[test]
fn cancel_proposal_that_is_a_referendum_does_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		let hash = set_balance_proposal_hash(4);
		add_preimage(hash, false);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		run_to_block(16);
		assert_noop!(
			GovernanceModule::cancel_proposal(origin.clone(), 0, BOB_COUNTRY_ID),
			Error::<Runtime>::ProposalIsReferendum
		);
	});
}

// Fast track proposal tests
#[test]
fn fast_track_proposal_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		let hash = set_balance_proposal_hash(4);
		add_preimage(hash, false);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		assert_ok!(GovernanceModule::fast_track_proposal(
			Origin::signed(ALICE),
			0,
			BOB_COUNTRY_ID
		));
		assert_eq!(
			last_event(),
			Event::Governance(crate::Event::ProposalFastTracked(BOB_COUNTRY_ID, 0))
		);
	});
}

#[test]
fn fast_track_proposal_when_not_country_owner_does_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		let hash = set_balance_proposal_hash(4);
		add_preimage(hash, false);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		assert_noop!(
			GovernanceModule::fast_track_proposal(Origin::signed(BOB), 0, BOB_COUNTRY_ID),
			BadOrigin
		);
	});
}

#[test]
fn fast_track_proposal_that_is_a_referendum_does_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		let hash = set_balance_proposal_hash(4);
		add_preimage(hash, false);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		run_to_block(16);
		assert_noop!(
			GovernanceModule::fast_track_proposal(Origin::signed(ALICE), 0, BOB_COUNTRY_ID),
			Error::<Runtime>::ProposalIsReferendum
		);
	});
}

// Voting tests
#[test]
fn vote_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		let hash = set_balance_proposal_hash(4);
		add_preimage(hash, false);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		run_to_block(16);
		assert_ok!(GovernanceModule::try_vote(Origin::signed(BOB), 0, true));
		// assert_eq!(Balances::free_balance(&BOB), 100);
		assert_eq!(
			last_event(),
			Event::Governance(crate::Event::VoteRecorded(BOB, 0, true))
		);
	});
}

#[test]
fn vote_when_not_country_member_does_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		let hash = set_balance_proposal_hash(4);
		add_preimage(hash, false);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			ALICE_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		run_to_block(16);
		assert_noop!(
			GovernanceModule::try_vote(Origin::signed(BOB), 0, true),
			Error::<Runtime>::AccountIsNotMetaverseMember
		);
	});
}

#[test]
fn vote_more_than_once_does_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		let hash = set_balance_proposal_hash(4);
		add_preimage(hash, false);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		run_to_block(16);
		assert_ok!(GovernanceModule::try_vote(Origin::signed(BOB), 0, true));
		assert_noop!(
			GovernanceModule::try_vote(Origin::signed(BOB), 0, true),
			Error::<Runtime>::AccountAlreadyVoted
		);
	});
}

// Remove vote tests
#[test]
fn remove_vote_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		let hash = set_balance_proposal_hash(4);
		add_preimage(hash, false);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		run_to_block(16);
		assert_ok!(GovernanceModule::try_vote(Origin::signed(BOB), 0, true));
		assert_ok!(GovernanceModule::try_remove_vote(Origin::signed(BOB), 0));
		assert_eq!(Balances::free_balance(&BOB), 500);
		assert_eq!(last_event(), Event::Governance(crate::Event::VoteRemoved(BOB, 0)));
	});
}

#[test]
fn remove_vote_when_you_have_not_voted_does_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		let hash = set_balance_proposal_hash(4);
		add_preimage(hash, false);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		run_to_block(16);
		assert_noop!(
			GovernanceModule::try_remove_vote(Origin::signed(BOB), 0),
			Error::<Runtime>::AccountHasNotVoted
		);
	});
}

// Emergency canceling of referendum tests
#[test]
fn emergency_cancel_referendum_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		let hash = set_balance_proposal_hash(4);
		add_preimage(hash, false);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		ReferendumJuryOf::<Runtime>::insert(BOB_COUNTRY_ID, ALICE);
		run_to_block(18);
		assert_ok!(GovernanceModule::emergency_cancel_referendum(origin.clone(), 0));
		assert_eq!(Balances::free_balance(&ALICE), 100000);
		assert_eq!(last_event(), Event::Governance(crate::Event::ReferendumCancelled(0)));
	});
}

#[test]
fn emergency_cancel_non_existing_referendum_does_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		assert_noop!(
			GovernanceModule::emergency_cancel_referendum(origin.clone(), 3),
			Error::<Runtime>::ReferendumDoesNotExist
		);
	});
}

#[test]
fn emergency_cancel_referendum_when_not_having_privileges_does_not_work() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		let hash = set_balance_proposal_hash(4);
		add_preimage(hash, false);

		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		ReferendumJuryOf::<Runtime>::insert(BOB_COUNTRY_ID, ALICE);
		run_to_block(17);
		assert_noop!(
			GovernanceModule::emergency_cancel_referendum(Origin::signed(BOB), 0),
			BadOrigin
		);
	});
}

// Referendum Finalization Tests
#[test]
fn referendum_proposal_passes() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		let hash = set_balance_proposal_hash(4);
		add_preimage(hash, false);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		run_to_block(16);
		assert_ok!(GovernanceModule::try_vote(Origin::signed(BOB), 0, true));
		run_to_block(27);
		assert_eq!(Balances::free_balance(&ALICE), 100000);
		assert_eq!(
			GovernanceModule::referendum_info(0),
			Some(ReferendumInfo::Finished { passed: true, end: 26 })
		);
		assert_eq!(last_event(), Event::Governance(crate::Event::ReferendumPassed(0)));
	});
}

#[test]
fn referendum_proposal_is_rejected() {
	ExtBuilder::default().build().execute_with(|| {
		let origin = Origin::signed(ALICE);
		let hash = set_balance_proposal_hash(4);
		add_preimage(hash, false);
		assert_ok!(GovernanceModule::propose(
			origin.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		run_to_block(16);

		assert_ok!(GovernanceModule::try_vote(Origin::signed(BOB), 0, false));
		run_to_block(27);
		assert_eq!(Balances::free_balance(&ALICE), 100000);
		assert_eq!(
			GovernanceModule::referendum_info(0),
			Some(ReferendumInfo::Finished { passed: false, end: 26 })
		);
		assert_eq!(last_event(), Event::Governance(crate::Event::ReferendumNotPassed(0)));
	});
}

#[test]
fn referendum_proposal_is_enacted() {
	ExtBuilder::default().build().execute_with(|| {
		let root = Origin::root();
		let proposer = Origin::signed(ALICE);
		let hash = set_balance_proposal_hash(400);
		add_preimage(hash, false);
		assert_ok!(GovernanceModule::propose(
			proposer.clone(),
			BOB_COUNTRY_ID,
			600,
			hash.clone(),
			PROPOSAL_DESCRIPTION.to_vec()
		));
		assert_ok!(GovernanceModule::enact_proposal(root.clone(), 0, BOB_COUNTRY_ID));
		assert_eq!(
			last_event(),
			Event::Governance(crate::Event::ProposalEnacted(BOB_COUNTRY_ID, 0))
		);
	});
}
