use crate::swap_protocols::{
    ledger::{Bitcoin, Ethereum},
    rfc003::{
        actions::{
            bob::{Accept, Decline},
            ActionKind, Actions,
        },
        bitcoin,
        ethereum::{self, EtherHtlc},
        roles::Bob,
        secret::Secret,
        state_machine::*,
    },
};
use bitcoin_support::{BitcoinQuantity, OutPoint};
use bitcoin_witness::PrimedInput;
use ethereum_support::{Bytes, EtherQuantity, U256};

impl OngoingSwap<Bob<Ethereum, Bitcoin, EtherQuantity, BitcoinQuantity>> {
    pub fn fund_action(&self) -> bitcoin::SendToAddress {
        bitcoin::SendToAddress {
            address: self.beta_htlc_params().compute_address(),
            value: self.beta_asset,
        }
    }

    pub fn refund_action(&self, beta_htlc_location: OutPoint) -> bitcoin::SpendOutput {
        bitcoin::SpendOutput {
            output: PrimedInput::new(
                beta_htlc_location,
                self.beta_asset,
                bitcoin::Htlc::from(self.beta_htlc_params())
                    .unlock_after_timeout(self.beta_ledger_refund_identity),
            ),
        }
    }

    pub fn redeem_action(
        &self,
        beta_htlc_location: ethereum_support::Address,
        secret: Secret,
    ) -> ethereum::SendTransaction {
        let data = Bytes::from(secret.raw_secret().to_vec());
        let gas_limit = EtherHtlc::tx_gas_limit();

        ethereum::SendTransaction {
            to: beta_htlc_location,
            data,
            gas_limit,
            value: EtherQuantity::from_wei(U256::zero()),
        }
    }
}

type BobActionKind = ActionKind<
    Accept<Ethereum, Bitcoin>,
    Decline<Ethereum, Bitcoin>,
    (),
    bitcoin::SendToAddress,
    ethereum::SendTransaction,
    bitcoin::SpendOutput,
>;

impl Actions for SwapStates<Bob<Ethereum, Bitcoin, EtherQuantity, BitcoinQuantity>> {
    type ActionKind = BobActionKind;

    fn actions(&self) -> Vec<BobActionKind> {
        use self::SwapStates as SS;
        match *self {
            SS::Start(Start { ref role, .. }) => vec![
                ActionKind::Accept(role.accept_action()),
                ActionKind::Decline(role.decline_action()),
            ],
            SS::AlphaFunded(AlphaFunded { ref swap, .. }) => {
                vec![ActionKind::Fund(swap.fund_action())]
            }
            SS::BothFunded(BothFunded {
                ref beta_htlc_location,
                ref swap,
                ..
            })
            | SS::AlphaRedeemedBetaFunded(AlphaRedeemedBetaFunded {
                ref beta_htlc_location,
                ref swap,
                ..
            })
            | SS::AlphaRefundedBetaFunded(AlphaRefundedBetaFunded {
                ref beta_htlc_location,
                ref swap,
                ..
            }) => vec![ActionKind::Refund(swap.refund_action(*beta_htlc_location))],
            SS::AlphaFundedBetaRedeemed(AlphaFundedBetaRedeemed {
                ref swap,
                ref alpha_htlc_location,
                ref beta_redeemed_tx,
                ..
            }) => vec![ActionKind::Redeem(
                swap.redeem_action(*alpha_htlc_location, beta_redeemed_tx.secret),
            )],
            _ => vec![],
        }
    }
}
