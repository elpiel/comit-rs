use bitcoin_support::BitcoinQuantity;
use common_types::{
    ledger::{bitcoin::Bitcoin, ethereum::Ethereum, Ledger},
    TradingSymbol,
};
use ethereum_support::EthereumQuantity;
use event_store::Event;
use exchange_api_client::OfferResponseBody;
use secret::Secret;
use std::str::FromStr;
use swaps::TradeId;

// State after exchange has made an offer
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OfferCreated<B, S>
where
    B: Ledger,
    S: Ledger,
{
    pub uid: TradeId,
    pub symbol: TradingSymbol,
    pub rate: f64,
    pub buy_amount: B::Quantity,
    pub sell_amount: S::Quantity,
}

impl From<OfferResponseBody> for OfferCreated<Ethereum, Bitcoin> {
    fn from(offer: OfferResponseBody) -> Self {
        OfferCreated {
            uid: offer.uid,
            symbol: offer.symbol,
            rate: offer.rate,
            buy_amount: EthereumQuantity::from_str(offer.buy_amount.as_str()).unwrap(),
            sell_amount: BitcoinQuantity::from_str(offer.sell_amount.as_str()).unwrap(),
        }
    }
}

impl From<OfferResponseBody> for OfferCreated<Bitcoin, Ethereum> {
    fn from(offer: OfferResponseBody) -> Self {
        OfferCreated {
            uid: offer.uid,
            symbol: offer.symbol,
            rate: offer.rate,
            buy_amount: BitcoinQuantity::from_str(offer.sell_amount.as_str()).unwrap(),
            sell_amount: EthereumQuantity::from_str(offer.buy_amount.as_str()).unwrap(),
        }
    }
}

impl<B: Ledger, S: Ledger> Event for OfferCreated<B, S> {
    type Prev = ();
}

// State after client accepts trade offer
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OrderCreated<B, S>
where
    B: Ledger,
    S: Ledger,
{
    pub uid: TradeId,
    pub client_success_address: B::Address,
    pub client_refund_address: S::Address,
    pub secret: Secret,
    pub long_relative_timelock: S::Time,
}

impl Event for OrderCreated<Ethereum, Bitcoin> {
    type Prev = OfferCreated<Ethereum, Bitcoin>;
}
impl Event for OrderCreated<Bitcoin, Ethereum> {
    type Prev = OfferCreated<Bitcoin, Ethereum>;
}

#[derive(Clone, Debug)]
pub struct OrderTaken<B, S>
where
    B: Ledger,
    S: Ledger,
{
    pub uid: TradeId,
    pub exchange_refund_address: B::Address,
    // This is embedded in the HTLC but we keep it here as well for completeness
    pub exchange_success_address: S::Address,
    pub exchange_contract_time_lock: u64,
}

impl Event for OrderTaken<Ethereum, Bitcoin> {
    type Prev = OrderCreated<Ethereum, Bitcoin>;
}
impl Event for OrderTaken<Bitcoin, Ethereum> {
    type Prev = OrderCreated<Bitcoin, Ethereum>;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContractDeployed<B>
where
    B: Ledger,
{
    pub uid: TradeId,
    pub address: B::Address,
}

impl Event for ContractDeployed<Ethereum> {
    type Prev = OrderTaken<Ethereum, Bitcoin>;
}
