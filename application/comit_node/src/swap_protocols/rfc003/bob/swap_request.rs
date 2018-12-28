use crate::swap_protocols::{
    ledger::{Bitcoin, Ethereum},
    metadata_store::{AssetKind, LedgerKind, Metadata, RoleKind},
    rfc003::{Ledger, SecretHash},
};
use bitcoin_support::BitcoinQuantity;
use ethereum_support::{Erc20Quantity, EtherQuantity};

#[derive(Clone, Debug, PartialEq, LabelledGeneric)]
pub struct SwapRequest<AL: Ledger, BL: Ledger, AA, BA> {
    pub alpha_asset: AA,
    pub beta_asset: BA,
    pub alpha_ledger: AL,
    pub beta_ledger: BL,
    pub alpha_ledger_refund_identity: AL::Identity,
    pub beta_ledger_redeem_identity: BL::Identity,
    pub alpha_ledger_lock_duration: AL::LockDuration,
    pub secret_hash: SecretHash,
}

impl From<SwapRequest<Bitcoin, Ethereum, BitcoinQuantity, EtherQuantity>> for Metadata {
    fn from(_: SwapRequest<Bitcoin, Ethereum, BitcoinQuantity, EtherQuantity>) -> Self {
        Self {
            alpha_ledger: LedgerKind::Bitcoin,
            beta_ledger: LedgerKind::Ethereum,
            alpha_asset: AssetKind::Bitcoin,
            beta_asset: AssetKind::Ether,
            role: RoleKind::Bob,
        }
    }
}

impl From<SwapRequest<Bitcoin, Ethereum, BitcoinQuantity, Erc20Quantity>> for Metadata {
    fn from(_: SwapRequest<Bitcoin, Ethereum, BitcoinQuantity, Erc20Quantity>) -> Self {
        Self {
            alpha_ledger: LedgerKind::Bitcoin,
            beta_ledger: LedgerKind::Ethereum,
            alpha_asset: AssetKind::Bitcoin,
            beta_asset: AssetKind::Erc20,
            role: RoleKind::Bob,
        }
    }
}

impl From<SwapRequest<Ethereum, Bitcoin, EtherQuantity, BitcoinQuantity>> for Metadata {
    fn from(_: SwapRequest<Ethereum, Bitcoin, EtherQuantity, BitcoinQuantity>) -> Self {
        Self {
            alpha_ledger: LedgerKind::Ethereum,
            beta_ledger: LedgerKind::Bitcoin,
            alpha_asset: AssetKind::Ether,
            beta_asset: AssetKind::Bitcoin,
            role: RoleKind::Bob,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SwapRequestKind {
    BitcoinEthereumBitcoinQuantityEtherQuantity(
        SwapRequest<Bitcoin, Ethereum, BitcoinQuantity, EtherQuantity>,
    ),
    BitcoinEthereumBitcoinQuantityErc20Quantity(
        SwapRequest<Bitcoin, Ethereum, BitcoinQuantity, Erc20Quantity>,
    ),
    EthereumBitcoinEtherQuantityBitcoinQuantity(
        SwapRequest<Ethereum, Bitcoin, EtherQuantity, BitcoinQuantity>,
    ),
}
