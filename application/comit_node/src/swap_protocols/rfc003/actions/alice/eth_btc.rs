use crate::swap_protocols::{
    ledger::{Bitcoin, Ethereum},
    rfc003::{
        actions::{ActionKind, Actions},
        bitcoin,
        ethereum::{self, EtherHtlc, Htlc},
        roles::Alice,
        state_machine::*,
    },
};
use bitcoin_support::{BitcoinQuantity, OutPoint};
use bitcoin_witness::PrimedInput;
use ethereum_support::{Bytes, EtherQuantity};

impl OngoingSwap<Alice<Ethereum, Bitcoin, EtherQuantity, BitcoinQuantity>> {
    pub fn fund_action(&self) -> ethereum::ContractDeploy {
        let htlc = EtherHtlc::from(self.alpha_htlc_params());
        let data = htlc.compile_to_hex().into();
        let gas_limit = htlc.deployment_gas_limit();

        ethereum::ContractDeploy {
            data,
            value: self.alpha_asset,
            gas_limit,
        }
    }

    pub fn refund_action(
        &self,
        alpha_htlc_location: ethereum_support::Address,
    ) -> ethereum::SendTransaction {
        let data = Bytes::default();
        let gas_limit = EtherHtlc::tx_gas_limit();

        ethereum::SendTransaction {
            to: alpha_htlc_location,
            data,
            gas_limit,
            value: EtherQuantity::zero(),
        }
    }

    pub fn redeem_action(&self, beta_htlc_location: OutPoint) -> bitcoin::SpendOutput {
        let htlc: bitcoin::Htlc = self.beta_htlc_params().into();

        bitcoin::SpendOutput {
            output: PrimedInput::new(
                beta_htlc_location,
                self.beta_asset,
                htlc.unlock_with_secret(self.beta_ledger_redeem_identity, &self.secret),
            ),
        }
    }
}

type AliceActionKind = ActionKind<
    (),
    (),
    (),
    ethereum::ContractDeploy,
    bitcoin::SpendOutput,
    ethereum::SendTransaction,
>;

impl Actions for SwapStates<Alice<Ethereum, Bitcoin, EtherQuantity, BitcoinQuantity>> {
    type ActionKind = AliceActionKind;

    fn actions(&self) -> Vec<AliceActionKind> {
        use self::SwapStates as SS;
        match *self {
            SS::Accepted(Accepted { ref swap, .. }) => vec![ActionKind::Fund(swap.fund_action())],
            SS::BothFunded(BothFunded {
                ref alpha_htlc_location,
                ref beta_htlc_location,
                ref swap,
                ..
            }) => vec![
                ActionKind::Redeem(swap.redeem_action(*beta_htlc_location)),
                ActionKind::Refund(swap.refund_action(*alpha_htlc_location)),
            ],
            SS::AlphaFundedBetaRefunded(AlphaFundedBetaRefunded {
                ref swap,
                ref alpha_htlc_location,
                ..
            })
            | SS::AlphaFundedBetaRedeemed(AlphaFundedBetaRedeemed {
                ref swap,
                ref alpha_htlc_location,
                ..
            }) => vec![ActionKind::Refund(swap.refund_action(*alpha_htlc_location))],
            SS::AlphaRefundedBetaFunded(AlphaRefundedBetaFunded {
                ref beta_htlc_location,
                ref swap,
                ..
            })
            | SS::AlphaRedeemedBetaFunded(AlphaRedeemedBetaFunded {
                ref beta_htlc_location,
                ref swap,
                ..
            }) => vec![ActionKind::Redeem(swap.redeem_action(*beta_htlc_location))],
            _ => vec![],
        }
    }
}
