macro_rules! _match_role {
    ($role:ident, $fn:tt) => {
        #[allow(clippy::redundant_closure_call)]
        match $role {
            RoleKind::Alice => {
                #[allow(dead_code)]
                type Role = Alice<AL, BL, AA, BA>;
                $fn()
            }
            RoleKind::Bob => {
                #[allow(dead_code)]
                type Role = Bob<AL, BL, AA, BA>;
                $fn()
            }
        }
    };
}

#[macro_export]
macro_rules! with_swap_types {
    ($metadata:expr, $fn:tt) => {{
        use crate::swap_protocols::rfc003::roles::{Alice, Bob};
        use bitcoin_support::BitcoinQuantity;
        use ethereum_support::EtherQuantity;
        let metadata = $metadata;

        match metadata {
            Metadata {
                alpha_ledger: LedgerKind::Bitcoin,
                beta_ledger: LedgerKind::Ethereum,
                alpha_asset: AssetKind::Bitcoin,
                beta_asset: AssetKind::Ether,
                role,
            } => {
                #[allow(dead_code)]
                type AL = Bitcoin;
                #[allow(dead_code)]
                type BL = Ethereum;
                #[allow(dead_code)]
                type AA = BitcoinQuantity;
                #[allow(dead_code)]
                type BA = EtherQuantity;

                _match_role!(role, $fn)
            }
            Metadata {
                alpha_ledger: LedgerKind::Bitcoin,
                beta_ledger: LedgerKind::Ethereum,
                alpha_asset: AssetKind::Bitcoin,
                beta_asset: AssetKind::Erc20,
                role,
            } => {
                #[allow(dead_code)]
                type AL = Bitcoin;
                #[allow(dead_code)]
                type BL = Ethereum;
                #[allow(dead_code)]
                type AA = BitcoinQuantity;
                #[allow(dead_code)]
                type BA = Erc20Quantity;

                _match_role!(role, $fn)
            }
            Metadata {
                alpha_ledger: LedgerKind::Ethereum,
                beta_ledger: LedgerKind::Bitcoin,
                alpha_asset: AssetKind::Ether,
                beta_asset: AssetKind::Bitcoin,
                role,
            } => {
                #[allow(dead_code)]
                type AL = Ethereum;
                #[allow(dead_code)]
                type BL = Bitcoin;
                #[allow(dead_code)]
                type AA = EtherQuantity;
                #[allow(dead_code)]
                type BA = BitcoinQuantity;

                _match_role!(role, $fn)
            }
            _ => unimplemented!(),
        }
    }};
}
