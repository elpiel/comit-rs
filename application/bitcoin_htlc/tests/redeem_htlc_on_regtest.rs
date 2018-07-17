extern crate bitcoin;
extern crate bitcoin_htlc;
extern crate bitcoin_node;
extern crate bitcoin_rpc;
extern crate bitcoin_rpc_helpers;
extern crate bitcoin_support;
extern crate bitcoin_witness;
extern crate common_types;
extern crate hex;
extern crate secp256k1_support;

use bitcoin_htlc::Htlc;
use bitcoin_node::BitcoinNode;
use bitcoin_rpc::BitcoinRpcApi;
use bitcoin_rpc_helpers::RegtestHelperClient;
use bitcoin_support::serialize::serialize_hex;
use bitcoin_support::{Address, BitcoinQuantity, Network, PrivateKey, PubkeyHash};
use bitcoin_witness::{PrimedInput, PrimedTransaction};
use common_types::secret::Secret;
use secp256k1_support::ToPublicKey;
use std::str::FromStr;

fn fund_htlc(
    client: &bitcoin_rpc::BitcoinCoreClient,
) -> (
    bitcoin_rpc::TransactionId,
    bitcoin_rpc::TransactionOutput,
    BitcoinQuantity,
    Htlc,
    u32,
    Secret,
    PrivateKey,
    PrivateKey,
) {
    let success_privkey =
        PrivateKey::from_str("cSrWvMrWE3biZinxPZc1hSwMMEdYgYsFpB6iEoh8KraLqYZUUCtt").unwrap();
    let success_pubkey_hash: PubkeyHash = success_privkey.secret_key().to_public_key().into();
    let refund_privkey =
        PrivateKey::from_str("cNZUJxVXghSri4dUaNW8ES3KiFyDoWVffLYDz7KMcHmKhLdFyZPx").unwrap();
    let secret = Secret::from(*b"hello world, you are beautiful!!");
    let refund_pubkey_hash: PubkeyHash = refund_privkey.secret_key().to_public_key().into();
    let sequence_lock = 10;

    let amount = BitcoinQuantity::from_satoshi(100_000_001);

    let htlc = Htlc::new(
        success_pubkey_hash,
        refund_pubkey_hash,
        secret.hash(),
        sequence_lock,
    );

    let htlc_address = htlc.compute_address(Network::BitcoinCoreRegtest);

    let txid = client
        .send_to_address(&htlc_address.clone().into(), amount.bitcoin())
        .unwrap()
        .into_result()
        .unwrap();

    client.generate(1).unwrap();

    let vout = client.find_vout_for_address(&txid, &htlc_address);

    (
        txid,
        vout.clone(),
        amount,
        htlc,
        sequence_lock,
        secret,
        success_privkey,
        refund_privkey,
    )
}

#[test]
fn redeem_htlc_with_secret() {
    let bitcoin_node = BitcoinNode::new();
    let client = bitcoin_node.get_client();
    client.generate(432).unwrap();

    let (txid, vout, input_amount, htlc, _, secret, private_key, _) = fund_htlc(&client);
    let secret_key = private_key.secret_key();

    assert!(
        htlc.can_be_unlocked_with(&secret, &secret_key).is_ok(),
        "Should be unlockable with the given secret and secret_key"
    );

    let alice_addr: Address = client
        .get_new_address()
        .unwrap()
        .into_result()
        .unwrap()
        .into();

    let fee = BitcoinQuantity::from_satoshi(1000);

    let redeem_tx = PrimedTransaction {
        inputs: vec![
            PrimedInput::new(
                txid.into(),
                vout.n,
                input_amount,
                htlc.unlock_with_secret(private_key.secret_key().clone(), secret),
            ),
        ],
        output_address: alice_addr.clone(),
        locktime: 0,
    }.sign_with_fee(fee);

    let redeem_tx_hex = serialize_hex(&redeem_tx).unwrap();

    let raw_redeem_tx = bitcoin_rpc::SerializedRawTransaction::from(redeem_tx_hex.as_str());

    let rpc_redeem_txid = client
        .send_raw_transaction(raw_redeem_tx)
        .unwrap()
        .into_result()
        .unwrap();

    client.generate(1).unwrap();

    assert!(
        client
            .find_utxo_at_tx_for_address(&rpc_redeem_txid, &alice_addr)
            .is_some(),
        "utxo should exist after redeeming htlc"
    );
}

#[test]
fn redeem_refund_htlc() {
    let bitcoin_node = BitcoinNode::new();
    let client = bitcoin_node.get_client();
    client.generate(432).unwrap();

    let (txid, vout, input_amount, htlc, nsequence, _, _, private_key) = fund_htlc(&client);

    let alice_addr: Address = client
        .get_new_address()
        .unwrap()
        .into_result()
        .unwrap()
        .into();
    let fee = BitcoinQuantity::from_satoshi(1000);

    let redeem_tx = PrimedTransaction {
        inputs: vec![
            PrimedInput::new(
                txid.clone().into(),
                vout.n,
                input_amount,
                htlc.unlock_after_timeout(private_key.secret_key().clone()),
            ),
        ],
        output_address: alice_addr.clone(),
        locktime: 0,
    }.sign_with_fee(fee);

    let redeem_tx_hex = serialize_hex(&redeem_tx).unwrap();

    let raw_redeem_tx = bitcoin_rpc::SerializedRawTransaction::from(redeem_tx_hex.as_str());

    let rpc_redeem_txid_error = client
        .send_raw_transaction(raw_redeem_tx.clone())
        .unwrap()
        .into_result();

    // It should fail because it's too early
    assert!(rpc_redeem_txid_error.is_err());
    let error = rpc_redeem_txid_error.unwrap_err();

    assert_eq!(error.code, -26);
    ///RPC_VERIFY_REJECTED = -26, !< Transaction or block was rejected by network rules
    assert!(error.message.contains("non-BIP68-final"));

    client.generate(nsequence).unwrap();

    let _txn = client
        .get_transaction(&txid)
        .unwrap()
        .into_result()
        .unwrap();

    let rpc_redeem_txid = client
        .send_raw_transaction(raw_redeem_tx)
        .unwrap()
        .into_result()
        .unwrap();

    client.generate(1).unwrap();

    assert!(
        client
            .find_utxo_at_tx_for_address(&rpc_redeem_txid, &alice_addr)
            .is_some(),
        "utxo should exist after refunding htlc"
    );
}