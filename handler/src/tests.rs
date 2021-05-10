#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};
use sp_runtime::traits::BadOrigin;

#[test]
fn register_resource_id_work() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);

        assert_noop!(
            Handler::register_resource_id(Origin::signed(BOB), XETHResourceId::get(), XETH::get()),
            BadOrigin,
        );

        assert_ok!(Handler::register_resource_id(
            Origin::signed(RegistorOrigin::get()),
            XETHResourceId::get(),
            XETH::get()
        ));

        let register_event = Event::handler(crate::Event::RegisterResourceId(
            XETHResourceId::get(),
            XETH::get(),
        ));
        assert!(System::events()
            .iter()
            .any(|record| record.event == register_event));

        assert_eq!(
            Handler::resource_ids(XETH::get()),
            Some(XETHResourceId::get())
        );
        assert_eq!(
            Handler::currency_ids(XETHResourceId::get()),
            Some(XETH::get())
        );

        assert_noop!(
            Handler::register_resource_id(
                Origin::signed(RegistorOrigin::get()),
                XETHResourceId::get(),
                XETH::get()
            ),
            Error::<Test>::ResourceIdAlreadyRegistered,
        );
    });
}

#[test]
fn remove_resource_id_work() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);

        assert_ok!(Handler::register_resource_id(
            Origin::signed(RegistorOrigin::get()),
            XETHResourceId::get(),
            XETH::get()
        ));
        assert_eq!(
            Handler::resource_ids(XETH::get()),
            Some(XETHResourceId::get())
        );
        assert_eq!(
            Handler::currency_ids(XETHResourceId::get()),
            Some(XETH::get())
        );

        assert_noop!(
            Handler::remove_resource_id(Origin::signed(BOB), XETHResourceId::get()),
            BadOrigin,
        );

        assert_ok!(Handler::remove_resource_id(
            Origin::signed(RegistorOrigin::get()),
            XETHResourceId::get()
        ));
        let unregister_event = Event::handler(crate::Event::UnregisterResourceId(
            XETHResourceId::get(),
            XETH::get(),
        ));
        assert!(System::events()
            .iter()
            .any(|record| record.event == unregister_event));
    });
}

#[test]
fn redeem() {
    ExtBuilder::default().build().execute_with(|| {
        let dest_chain_id: chainbridge::ChainId = 0;

        assert_noop!(
            Handler::redeem(
                Origin::signed(ALICE),
                XETH::get(),
                dest_chain_id,
                vec![1],
                10
            ),
            Error::<Test>::InvalidDestChainId,
        );

        assert_ok!(ChainBridge::whitelist_chain(
            Origin::signed(AdminOrigin::get()),
            dest_chain_id
        ));
        assert_noop!(
            Handler::redeem(
                Origin::signed(ALICE),
                XETH::get(),
                dest_chain_id,
                vec![1],
                10
            ),
            Error::<Test>::ResourceIdNotRegistered,
        );

        assert_ok!(Handler::register_resource_id(
            Origin::signed(RegistorOrigin::get()),
            XETHResourceId::get(),
            XETH::get()
        ));
        assert_eq!(XAssets::total_issuance(&XETH::get()), 1000);
        assert_eq!(XAssets::usable_balance(&ALICE, &XETH::get()), 1000);
        assert_ok!(XAssets::can_destroy_usable(&XETH::get()));
        assert_ok!(Handler::redeem(
            Origin::signed(ALICE),
            XETH::get(),
            dest_chain_id,
            vec![1],
            10
        ));
        assert_eq!(XAssets::total_issuance(&XETH::get()), 990);
        assert_eq!(XAssets::usable_balance(&ALICE, &XETH::get()), 990);
    });
}

#[test]
fn deposit() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Handler::deposit(Origin::signed(BOB), BOB, 500, XETHResourceId::get()),
            DispatchError::BadOrigin,
        );

        assert_noop!(
            Handler::deposit(
                Origin::signed(ChainBridge::account_id()),
                ALICE,
                500,
                XETHResourceId::get()
            ),
            Error::<Test>::ResourceIdNotRegistered,
        );

        assert_ok!(Handler::register_resource_id(
            Origin::signed(RegistorOrigin::get()),
            XETHResourceId::get(),
            XETH::get()
        ));
        assert_ok!(XAssets::issue(
            &XETH::get(),
            &ChainBridge::account_id(),
            1000
        ));
        assert_eq!(XAssets::total_issuance(&XETH::get()), 2000);
        assert_eq!(XAssets::usable_balance(&ALICE, &XETH::get()), 1000);
        assert_eq!(
            XAssets::usable_balance(&ChainBridge::account_id(), &XETH::get()),
            1000
        );

        assert_ok!(Handler::deposit(
            Origin::signed(ChainBridge::account_id()),
            ALICE,
            500,
            XETHResourceId::get()
        ));
        assert_eq!(XAssets::total_issuance(&XETH::get()), 2500);
        assert_eq!(XAssets::usable_balance(&ALICE, &XETH::get()), 1500);
    });
}
