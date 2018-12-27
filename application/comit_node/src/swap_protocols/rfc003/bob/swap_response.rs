use crate::{
    comit_client::SwapReject,
    swap_protocols::rfc003::{ethereum::Seconds, state_machine::StateMachineResponse},
};
use bitcoin_support::Blocks;

#[derive(Clone, Debug, PartialEq)]
pub enum SwapResponseKind {
    BitcoinEthereum(
        Result<
            StateMachineResponse<secp256k1_support::KeyPair, ethereum_support::Address, Seconds>,
            SwapReject,
        >,
    ),
    EthereumBitcoin(
        Result<
            StateMachineResponse<ethereum_support::Address, secp256k1_support::KeyPair, Blocks>,
            SwapReject,
        >,
    ),
}

impl
    From<
        Result<
            StateMachineResponse<secp256k1_support::KeyPair, ethereum_support::Address, Seconds>,
            SwapReject,
        >,
    > for SwapResponseKind
{
    fn from(
        result: Result<
            StateMachineResponse<secp256k1_support::KeyPair, ethereum_support::Address, Seconds>,
            SwapReject,
        >,
    ) -> Self {
        SwapResponseKind::BitcoinEthereum(result)
    }
}

impl
    From<
        Result<
            StateMachineResponse<ethereum_support::Address, secp256k1_support::KeyPair, Blocks>,
            SwapReject,
        >,
    > for SwapResponseKind
{
    fn from(
        result: Result<
            StateMachineResponse<ethereum_support::Address, secp256k1_support::KeyPair, Blocks>,
            SwapReject,
        >,
    ) -> Self {
        SwapResponseKind::EthereumBitcoin(result)
    }
}
