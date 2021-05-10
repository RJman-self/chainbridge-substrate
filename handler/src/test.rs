#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};
use sp_runtime::traits::BadOrigin;

#[test]
fn register_resource_id_work() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);

        assert_eq!(Handler::resource_ids(PcxAssetId::get()), None);
        assert_eq!(Handler::currency_ids(PcxAssetIdResourceId::get()), None);

        assert_noop!(
            Handler::register_resource_id(
                Origin::signed(ALICE),
                PcxAssetIdResourceId::get(),
                PcxAssetId::get()
            ),
            BadOrigin,
        );

        assert_noop!(
            Handler::register_resource_id(
                Origin::signed(RegistorOrigin::get()),
                PcxAssetIdResourceId::get(),
                WETH::get()
            ),
            Error::<Runtime>::ResourceIdCurrencyIdNotMatch,
        );

        assert_noop!(
            Handler::register_resource_id(
                Origin::signed(RegistorOrigin::get()),
                WETHResourceId::get(),
                PcxAssetId::get()
            ),
            Error::<Runtime>::ResourceIdCurrencyIdNotMatch,
        );

        assert_ok!(Handler::register_resource_id(
            Origin::signed(RegistorOrigin::get()),
            PcxAssetIdResourceId::get(),
            PcxAssetId::get()
        ));
        let register_event = Event::ecosystem_chainsafe(crate::Event::RegisterResourceId(
            PcxAssetIdResourceId::get(),
            PcxAssetId::get(),
        ));
        assert!(System::events()
            .iter()
            .any(|record| record.event == register_event));

        assert_eq!(
            Handler::resource_ids(PcxAssetId::get()),
            Some(PcxAssetIdResourceId::get())
        );
        assert_eq!(
            Handler::currency_ids(PcxAssetIdResourceId::get()),
            Some(PcxAssetId::get())
        );

        assert_noop!(
            Handler::register_resource_id(
                Origin::signed(RegistorOrigin::get()),
                PcxAssetIdResourceId::get(),
                PcxAssetId::get()
            ),
            Error::<Runtime>::ResourceIdAlreadyRegistered,
        );
    });
}

#[test]
fn remove_resource_id_work() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);

        assert_ok!(Handler::register_resource_id(
            Origin::signed(RegistorOrigin::get()),
            PcxAssetIdResourceId::get(),
            PcxAssetId::get()
        ));
        assert_eq!(
            Handler::resource_ids(PcxAssetId::get()),
            Some(PcxAssetIdResourceId::get())
        );
        assert_eq!(
            Handler::currency_ids(PcxAssetIdResourceId::get()),
            Some(PcxAssetId::get())
        );

        assert_noop!(
            Handler::remove_resource_id(Origin::signed(ALICE), PcxAssetIdResourceId::get()),
            BadOrigin,
        );

        assert_ok!(Handler::remove_resource_id(
            Origin::signed(RegistorOrigin::get()),
            PcxAssetIdResourceId::get()
        ));
        let unregister_event = Event::ecosystem_chainsafe(crate::Event::UnregisterResourceId(
            PcxAssetIdResourceId::get(),
            PcxAssetId::get(),
        ));
        assert!(System::events()
            .iter()
            .any(|record| record.event == unregister_event));
    });
}

#[test]
fn do_transfer_to_bridge_work() {
    ExtBuilder::default().build().execute_with(|| {
        let dest_chain_id: chainbridge::ChainId = 0;

        assert_noop!(
            Handler::do_transfer_to_bridge(&ALICE, PcxAssetId::get(), dest_chain_id, vec![1], 10),
            Error::<Runtime>::InvalidDestChainId,
        );

        assert_ok!(ChainBridge::whitelist_chain(
            Origin::signed(AdminOrigin::get()),
            dest_chain_id
        ));
        assert_noop!(
            Handler::do_transfer_to_bridge(&ALICE, PcxAssetId::get(), dest_chain_id, vec![1], 10),
            Error::<Runtime>::ResourceIdNotRegistered,
        );

        assert_ok!(Handler::register_resource_id(
            Origin::signed(RegistorOrigin::get()),
            PcxAssetIdResourceId::get(),
            PcxAssetId::get()
        ));
        assert_eq!(Tokens::total_issuance(PcxAssetId::get()), 1000);
        assert_eq!(Tokens::free_balance(PcxAssetId::get(), &ALICE), 1000);
        assert_eq!(
            Tokens::free_balance(PcxAssetId::get(), &ChainBridge::account_id()),
            0
        );

        assert_ok!(Handler::do_transfer_to_bridge(
            &ALICE,
            PcxAssetId::get(),
            dest_chain_id,
            vec![1],
            10
        ));
        assert_eq!(Tokens::total_issuance(PcxAssetId::get()), 1000);
        assert_eq!(Tokens::free_balance(PcxAssetId::get(), &ALICE), 990);
        assert_eq!(
            Tokens::free_balance(PcxAssetId::get(), &ChainBridge::account_id()),
            10
        );

        assert_ok!(Handler::register_resource_id(
            Origin::signed(RegistorOrigin::get()),
            WETHResourceId::get(),
            WETH::get()
        ));
        assert_ok!(Tokens::deposit(WETH::get(), &ALICE, 1000));
        assert_eq!(Tokens::total_issuance(WETH::get()), 1000);
        assert_eq!(Tokens::free_balance(WETH::get(), &ALICE), 1000);
        assert_eq!(
            Tokens::free_balance(WETH::get(), &ChainBridge::account_id()),
            0
        );

        assert_ok!(Handler::do_transfer_to_bridge(
            &ALICE,
            WETH::get(),
            dest_chain_id,
            vec![1],
            20
        ));
        assert_eq!(Tokens::total_issuance(WETH::get()), 980);
        assert_eq!(Tokens::free_balance(WETH::get(), &ALICE), 980);
        assert_eq!(
            Tokens::free_balance(WETH::get(), &ChainBridge::account_id()),
            0
        );
    });
}

#[test]
fn transfer_from_bridge_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Handler::transfer_from_bridge(
                Origin::signed(ALICE),
                ALICE,
                500,
                PcxAssetIdResourceId::get()
            ),
            BadOrigin,
        );

        assert_noop!(
            Handler::transfer_from_bridge(
                Origin::signed(ChainBridge::account_id()),
                ALICE,
                500,
                PcxAssetIdResourceId::get()
            ),
            Error::<Runtime>::ResourceIdNotRegistered,
        );

        assert_ok!(Handler::register_resource_id(
            Origin::signed(RegistorOrigin::get()),
            PcxAssetIdResourceId::get(),
            PcxAssetId::get()
        ));
        assert_ok!(Tokens::deposit(
            PcxAssetId::get(),
            &ChainBridge::account_id(),
            1000
        ));
        assert_eq!(Tokens::total_issuance(PcxAssetId::get()), 2000);
        assert_eq!(Tokens::free_balance(PcxAssetId::get(), &ALICE), 1000);
        assert_eq!(
            Tokens::free_balance(PcxAssetId::get(), &ChainBridge::account_id()),
            1000
        );

        assert_ok!(Handler::transfer_from_bridge(
            Origin::signed(ChainBridge::account_id()),
            ALICE,
            500,
            PcxAssetIdResourceId::get()
        ));
        assert_eq!(Tokens::total_issuance(PcxAssetId::get()), 2000);
        assert_eq!(Tokens::free_balance(PcxAssetId::get(), &ALICE), 1500);
        assert_eq!(
            Tokens::free_balance(PcxAssetId::get(), &ChainBridge::account_id()),
            500
        );

        assert_ok!(Handler::register_resource_id(
            Origin::signed(RegistorOrigin::get()),
            WETHResourceId::get(),
            WETH::get()
        ));
        assert_eq!(Tokens::total_issuance(WETH::get()), 0);
        assert_eq!(Tokens::free_balance(WETH::get(), &ALICE), 0);
        assert_eq!(
            Tokens::free_balance(WETH::get(), &ChainBridge::account_id()),
            0
        );

        assert_ok!(Handler::transfer_from_bridge(
            Origin::signed(ChainBridge::account_id()),
            ALICE,
            500,
            WETHResourceId::get()
        ));
        assert_eq!(Tokens::total_issuance(WETH::get()), 500);
        assert_eq!(Tokens::free_balance(WETH::get(), &ALICE), 500);
        assert_eq!(
            Tokens::free_balance(WETH::get(), &ChainBridge::account_id()),
            0
        );
    });
}
