#![warn(unused_extern_crates, missing_debug_implementations, rust_2018_idioms)]
#![deny(unsafe_code)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

pub mod ethereum_wallet;
pub mod htlc_harness;
pub mod parity_client;

use crate::htlc_harness::{
    ether_harness, CustomSizeSecret, EtherHarnessParams, HTLC_TIMEOUT, SECRET,
};
use ethereum_support::{Bytes, EtherQuantity, H256, U256};
use spectral::prelude::*;
use testcontainers::clients::Cli;

const HTLC_GAS_COST: u64 = 10885000;
const HTLC_GAS_COST_SHORT_SECRET: u64 = 10878600;
// keccak256(Redeemed())
const REDEEMED_LOG_MSG: &str = "0xB8CAC300E37F03AD332E581DEA21B2F0B84EAAADC184A295FEF71E81F44A7413";
// keccak256(Refunded())
const REFUNDED_LOG_MSG: &str = "0x5D26862916391BF49478B2F5103B0720A842B45EF145A268F2CD1FB2AED55178";

#[test]
fn given_deployed_htlc_when_redeemed_with_secret_then_money_is_transferred() {
    let docker = Cli::default();
    let (alice, bob, htlc, client, _handle, _container) =
        ether_harness(&docker, EtherHarnessParams::default());

    assert_eq!(
        client.eth_balance_of(bob),
        EtherQuantity::from_eth(0.0).wei()
    );
    assert_eq!(
        client.eth_balance_of(alice),
        EtherQuantity::from_eth(0.6).wei() - U256::from(HTLC_GAS_COST)
    );

    assert_eq!(
        client.eth_balance_of(htlc),
        EtherQuantity::from_eth(0.4).wei()
    );

    // Send correct secret to contract
    client.send_data(htlc, Some(Bytes(SECRET.to_vec())));

    assert_eq!(
        client.eth_balance_of(bob),
        EtherQuantity::from_eth(0.4).wei()
    );
    assert_eq!(
        client.eth_balance_of(alice),
        EtherQuantity::from_eth(0.6).wei() - U256::from(HTLC_GAS_COST)
    );
    assert_eq!(
        client.eth_balance_of(htlc),
        EtherQuantity::from_eth(0.0).wei()
    );
}

#[test]
fn given_deployed_htlc_when_refunded_after_timeout_then_money_is_refunded() {
    let docker = Cli::default();
    let (alice, bob, htlc, client, _handle, _container) =
        ether_harness(&docker, EtherHarnessParams::default());

    assert_eq!(
        client.eth_balance_of(bob),
        EtherQuantity::from_eth(0.0).wei()
    );
    assert_eq!(
        client.eth_balance_of(alice),
        EtherQuantity::from_eth(0.6).wei() - U256::from(HTLC_GAS_COST)
    );
    assert_eq!(
        client.eth_balance_of(htlc),
        EtherQuantity::from_eth(0.4).wei()
    );

    // Wait for the contract to expire
    ::std::thread::sleep(HTLC_TIMEOUT);
    ::std::thread::sleep(HTLC_TIMEOUT);
    client.send_data(htlc, None);

    assert_eq!(
        client.eth_balance_of(bob),
        EtherQuantity::from_eth(0.0).wei()
    );
    assert_eq!(
        client.eth_balance_of(alice),
        EtherQuantity::from_eth(1.0).wei() - U256::from(HTLC_GAS_COST)
    );
    assert_eq!(
        client.eth_balance_of(htlc),
        EtherQuantity::from_eth(0.0).wei()
    );
}

#[test]
fn given_deployed_htlc_when_timeout_not_yet_reached_and_wrong_secret_then_nothing_happens() {
    let docker = Cli::default();
    let (alice, bob, htlc, client, _handle, _container) =
        ether_harness(&docker, EtherHarnessParams::default());

    assert_eq!(
        client.eth_balance_of(bob),
        EtherQuantity::from_eth(0.0).wei()
    );
    assert_eq!(
        client.eth_balance_of(alice),
        EtherQuantity::from_eth(0.6).wei() - U256::from(HTLC_GAS_COST)
    );
    assert_eq!(
        client.eth_balance_of(htlc),
        EtherQuantity::from_eth(0.4).wei()
    );

    // Wait for the contract to expire
    client.send_data(htlc, None);

    assert_eq!(
        client.eth_balance_of(bob),
        EtherQuantity::from_eth(0.0).wei()
    );
    assert_eq!(
        client.eth_balance_of(alice),
        EtherQuantity::from_eth(0.6).wei() - U256::from(HTLC_GAS_COST)
    );
    assert_eq!(
        client.eth_balance_of(htlc),
        EtherQuantity::from_eth(0.4).wei()
    );
}

#[test]
fn given_htlc_and_redeem_should_emit_redeem_log_msg() {
    let docker = Cli::default();
    let (_alice, _bob, htlc, client, _handle, _container) =
        ether_harness(&docker, EtherHarnessParams::default());

    // Send incorrect secret to contract
    let transaction_receipt = client.send_data(htlc, Some(Bytes(b"I'm a h4x0r".to_vec())));
    assert_that(&transaction_receipt.logs).has_length(0);

    // Send correct secret to contract
    let transaction_receipt = client.send_data(htlc, Some(Bytes(SECRET.to_vec())));

    assert_that(&transaction_receipt.logs).has_length(1);
    let topic: H256 = REDEEMED_LOG_MSG.into();
    assert_that(&transaction_receipt.logs[0].topics).has_length(1);
    assert_that(&transaction_receipt.logs[0].topics).contains(topic);
}

#[test]
fn given_htlc_and_refund_should_emit_refund_log_msg() {
    let docker = Cli::default();
    let (_alice, _bob, htlc, client, _handle, _container) =
        ether_harness(&docker, EtherHarnessParams::default());

    // Wait for the contract to expire
    ::std::thread::sleep(HTLC_TIMEOUT);
    ::std::thread::sleep(HTLC_TIMEOUT);
    let transaction_receipt = client.send_data(htlc, None);

    assert_that(&transaction_receipt.logs).has_length(1);
    let topic: H256 = REFUNDED_LOG_MSG.into();
    assert_that(&transaction_receipt.logs[0].topics).has_length(1);
    assert_that(&transaction_receipt.logs[0].topics).contains(topic);
}

#[test]
fn given_short_secret_left_padded_should_not_redeem() -> Result<(), failure::Error> {
    let docker = Cli::default();

    let secret = CustomSizeSecret(vec![
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8, 0u8, 1u8, 2u8, 3u8, 4u8, 6u8, 6u8, 7u8, 9u8, 10u8,
    ]);

    let (alice, bob, htlc, client, _handle, _container) =
        ether_harness(&docker, EtherHarnessParams::from(secret.hash()));

    assert_eq!(
        client.eth_balance_of(bob),
        EtherQuantity::from_eth(0.0).wei()
    );
    assert_eq!(
        client.eth_balance_of(alice),
        EtherQuantity::from_eth(0.6).wei() - U256::from(HTLC_GAS_COST_SHORT_SECRET)
    );

    assert_eq!(
        client.eth_balance_of(htlc),
        EtherQuantity::from_eth(0.4).wei()
    );

    client.send_data(
        htlc,
        Some(Bytes(vec![1u8, 2u8, 3u8, 4u8, 6u8, 6u8, 7u8, 9u8, 10u8])),
    ); // will result in right padding and redemption should fail

    assert_eq!(
        client.eth_balance_of(bob),
        EtherQuantity::from_eth(0.0).wei()
    );
    //    assert_eq!(
    //        client.eth_balance_of(alice),
    //        EtherQuantity::from_eth(0.6).wei() - U256::from(HTLC_GAS_COST)
    //    );
    assert_eq!(
        client.eth_balance_of(htlc),
        EtherQuantity::from_eth(0.4).wei()
    );
    Ok(())
}

#[test]
fn given_short_secret_right_padded_should_redeem() -> Result<(), failure::Error> {
    let docker = Cli::default();
    let secret = CustomSizeSecret(vec![
        1u8, 2u8, 3u8, 4u8, 6u8, 6u8, 7u8, 9u8, 10u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
    ]);

    let (alice, bob, htlc, client, _handle, _container) =
        ether_harness(&docker, EtherHarnessParams::from(secret.hash()));

    assert_eq!(
        client.eth_balance_of(bob),
        EtherQuantity::from_eth(0.0).wei()
    );

    assert_eq!(
        client.eth_balance_of(alice),
        EtherQuantity::from_eth(0.6).wei() - U256::from(HTLC_GAS_COST_SHORT_SECRET)
    );

    assert_eq!(
        client.eth_balance_of(htlc),
        EtherQuantity::from_eth(0.4).wei()
    );

    client.send_data(
        htlc,
        Some(Bytes(vec![1u8, 2u8, 3u8, 4u8, 6u8, 6u8, 7u8, 9u8, 10u8])),
    );

    assert_eq!(
        client.eth_balance_of(bob),
        EtherQuantity::from_eth(0.4).wei()
    );
    //    assert_eq!(
    //        client.eth_balance_of(alice),
    //        EtherQuantity::from_eth(0.6).wei() - U256::from(HTLC_GAS_COST)
    //    );
    assert_eq!(
        client.eth_balance_of(htlc),
        EtherQuantity::from_eth(0.0).wei()
    );
    Ok(())
}

#[test]
fn given_short_secret_not_padded_should_not_redeem() -> Result<(), failure::Error> {
    let docker = Cli::default();
    let secret = CustomSizeSecret(vec![1u8, 2u8, 3u8, 4u8, 6u8, 6u8, 7u8, 9u8, 10u8]);

    let (alice, bob, htlc, client, _handle, _container) =
        ether_harness(&docker, EtherHarnessParams::from(secret.hash()));

    assert_eq!(
        client.eth_balance_of(bob),
        EtherQuantity::from_eth(0.0).wei()
    );

    assert_eq!(
        client.eth_balance_of(alice),
        EtherQuantity::from_eth(0.6).wei() - U256::from(HTLC_GAS_COST_SHORT_SECRET)
    );

    assert_eq!(
        client.eth_balance_of(htlc),
        EtherQuantity::from_eth(0.4).wei()
    );

    client.send_data(
        htlc,
        Some(Bytes(vec![1u8, 2u8, 3u8, 4u8, 6u8, 6u8, 7u8, 9u8, 10u8])),
    ); // will result in right padded secret, hashing this is not equal than our
       // generated secret above

    assert_eq!(
        client.eth_balance_of(bob),
        EtherQuantity::from_eth(0.0).wei()
    );
    //    assert_eq!(
    //        client.eth_balance_of(alice),
    //        EtherQuantity::from_eth(0.6).wei() - U256::from(HTLC_GAS_COST)
    //    );
    assert_eq!(
        client.eth_balance_of(htlc),
        EtherQuantity::from_eth(0.4).wei()
    );
    Ok(())
}
