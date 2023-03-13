//! A rust crate for making requests to the Kraken Rest API and subscribing
//! to Kraken websockets feeds

#![deny(missing_docs)]

mod kraken_rest_client;
pub use kraken_rest_client::*;

mod messages;
use messages::{
    unpack_kraken_result, AddOrderRequest, AssetPairsRequest, CancelAllOrdersAfterRequest, CancelOrderRequest, Empty,
    GetOpenOrdersRequest, KrakenResult, TickerRequest,
};
pub use messages::{
    AddOrderResponse, AssetPairsResponse, AssetTickerInfo, AssetsResponse, BalanceResponse, BsType,
    CancelAllOrdersAfterResponse, CancelAllOrdersResponse, CancelOrderResponse, GetOpenOrdersResponse, GetClosedOrdersResponse,
    GetWebSocketsTokenResponse, OrderAdded, OrderFlag, OrderInfo, OrderStatus, OrderType, SystemStatusResponse,
    TickerResponse, TimeResponse, TxId, UserRefId,
};

use core::convert::TryFrom;
use std::collections::BTreeSet;

// Websockets API support
#[cfg(feature = "ws")]
pub mod ws;

/// A description of a market order to place
#[derive(Debug, Clone)]
pub struct MarketOrder {
    /// Whether to buy or sell
    pub bs_type: BsType,
    /// Volume (in lots)
    pub volume: String,
    /// Asset pair
    pub pair: String,
    /// Order flags (market price protection etc.)
    pub oflags: BTreeSet<OrderFlag>,
    /// Leverage
    pub leverage: Option<String>,
}

/// A description of a limit order to place
#[derive(Debug, Clone)]
pub struct LimitOrder {
    /// Whether to buy or sell
    pub bs_type: BsType,
    /// Volume (in lots)
    pub volume: String,
    /// Asset pair
    pub pair: String,
    /// Price
    pub price: String,
    /// Order flags (post-only etc.)
    pub oflags: BTreeSet<OrderFlag>,
    /// Leverage
    pub leverage: Option<String>,
}

/// A description of a settle-poisiton order to place
#[derive(Debug, Clone)]
pub struct SettlePositionOrder {
    /// Whether to buy or sell
    pub bs_type: BsType,
    /// Volume (in lots)
    pub volume: String,
    /// Asset pair
    pub pair: String,
    /// Order flags (post-only etc.)
    pub oflags: BTreeSet<OrderFlag>,
    /// Leverage
    /// Note: This is not the same as the leverage used to open the position
    /// This is the leverage used to close the position
    ///
    /// For example, if you opened a position with 2x leverage, and then
    /// closed it with 3x leverage, then the leverage used to close the position
    /// would be 3x
    ///
    /// If you opened a position with 2x leverage, and then closed it with 1x leverage,
    /// then the leverage used to close the position would be 1x
    pub leverage: Option<String>,
}

/// A connection to the Kraken REST API
/// This only supports blocking http requests for now
pub struct KrakenRestAPI {
    client: KrakenRestClient,
}

impl KrakenRestAPI {
    /// (Public) Get the kraken system's time
    pub fn time(&self) -> Result<TimeResponse> {
        let result: Result<KrakenResult<TimeResponse>> = self.client.query_public("Time", Empty {});
        result.and_then(unpack_kraken_result)
    }

    /// (Public) Get the kraken system's status
    pub fn system_status(&self) -> Result<SystemStatusResponse> {
        let result: Result<KrakenResult<SystemStatusResponse>> = self.client.query_public("SystemStatus", Empty {});
        result.and_then(unpack_kraken_result)
    }

    /// (Public) Get the list of kraken's supported assets, and info
    pub fn assets(&self) -> Result<AssetsResponse> {
        let result: Result<KrakenResult<AssetsResponse>> = self.client.query_public("Assets", Empty {});
        result.and_then(unpack_kraken_result)
    }

    /// (Public) Get the list of kraken's asset pairs, and info
    ///
    /// Arguments:
    /// * pairs: A list of Kraken asset pair strings to get info about. If empty then all asset pairs
    pub fn asset_pairs(&self, pairs: Vec<String>) -> Result<AssetPairsResponse> {
        let result: Result<KrakenResult<AssetPairsResponse>> = self
            .client
            .query_public("AssetPairs", AssetPairsRequest { pair: pairs.join(",") });
        result.and_then(unpack_kraken_result)
    }

    /// (Public) Get the ticker price for one or more asset pairs
    ///
    /// Arguments:
    /// * pairs: A list of Kraken asset pair strings to get ticker info about
    pub fn ticker(&self, pairs: Vec<String>) -> Result<TickerResponse> {
        let result: Result<KrakenResult<TickerResponse>> = self
            .client
            .query_public("Ticker", TickerRequest { pair: pairs.join(",") });
        result.and_then(unpack_kraken_result)
    }

    /// (Private) Get the balance
    pub fn get_account_balance(&self) -> Result<BalanceResponse> {
        let result: Result<KrakenResult<BalanceResponse>> = self.client.query_private("Balance", Empty {});
        result.and_then(unpack_kraken_result)
    }

    /// (Private) Get a websockets authentication token
    pub fn get_websockets_token(&self) -> Result<GetWebSocketsTokenResponse> {
        let result: Result<KrakenResult<GetWebSocketsTokenResponse>> =
            self.client.query_private("GetWebSocketsToken", Empty {});
        result.and_then(unpack_kraken_result)
    }

    /// (Private) Get the list of open orders
    ///
    /// Arguments:
    /// * userref: An optional user-reference to filter the list of open orders by
    pub fn get_open_orders(&self, userref: Option<UserRefId>) -> Result<GetOpenOrdersResponse> {
        let result: Result<KrakenResult<GetOpenOrdersResponse>> = self
            .client
            .query_private("OpenOrders", GetOpenOrdersRequest { userref });
        result.and_then(unpack_kraken_result)
    }

    /// (Private) Get the list of closed orders
    /// Note: This is not the same as the list of orders that have been cancelled
    /// (see `get_open_orders` for that)
    /// Arguments:
    /// * userref: An optional user-reference to filter the list of closed orders by
    pub fn get_closed_orders(&self, userref: Option<UserRefId>) -> Result<GetClosedOrdersResponse> {
        let result: Result<KrakenResult<GetClosedOrdersResponse>> = self
            .client
            .query_private("ClosedOrders", GetOpenOrdersRequest { userref });
        result.and_then(unpack_kraken_result)
    }

    /// (Private) Cancel order
    ///
    /// Arguments:
    /// * id: A TxId (OR a UserRefId) of order(s) to cancel
    pub fn cancel_order(&self, id: String) -> Result<CancelOrderResponse> {
        let result: Result<KrakenResult<CancelOrderResponse>> = self
            .client
            .query_private("CancelOrder", CancelOrderRequest { txid: id });
        result.and_then(unpack_kraken_result)
    }

    /// (Private) Cancel all orders (regardless of user ref or tx id)
    pub fn cancel_all_orders(&self) -> Result<CancelAllOrdersResponse> {
        let result: Result<KrakenResult<CancelAllOrdersResponse>> = self.client.query_private("CancelAll", Empty {});
        result.and_then(unpack_kraken_result)
    }

    /// (Private) Cancel all orders after
    ///
    /// Arguments:
    /// * timeout: Integer timeout specified in seconds. 0 to disable the timer.
    pub fn cancel_all_orders_after(&self, timeout: u64) -> Result<CancelAllOrdersAfterResponse> {
        let result: Result<KrakenResult<CancelAllOrdersAfterResponse>> = self
            .client
            .query_private("CancelAllOrdersAfter", CancelAllOrdersAfterRequest { timeout });
        result.and_then(unpack_kraken_result)
    }

    /// (Private) Place a market order
    ///
    /// Arguments:
    /// * market_order: Market order object describing the parameters of the order
    /// * user_ref_id: Optional user ref id to attach to the order
    /// * validate: If true, the order is only validated and is not actually placed
    pub fn add_market_order(
        &self,
        market_order: MarketOrder,
        user_ref_id: Option<UserRefId>,
        validate: bool,
    ) -> Result<AddOrderResponse> {
        let req = AddOrderRequest {
            ordertype: OrderType::Market,
            bs_type: market_order.bs_type,
            volume: market_order.volume,
            pair: market_order.pair,
            price: Default::default(),
            oflags: market_order.oflags,
            userref: user_ref_id,
            validate,
            leverage: market_order.leverage,
        };
        let result: Result<KrakenResult<AddOrderResponse>> = self.client.query_private("AddOrder", req);
        result.and_then(unpack_kraken_result)
    }

    /// (Private) Place a limit order
    ///
    /// Arguments:
    /// * limit_order: Limit order object describing the parameters of the order
    /// * user_ref_id: Optional user ref id to attach to the order
    /// * validate: If true, the order is only validated and is not actually placed
    pub fn add_limit_order(
        &self,
        limit_order: LimitOrder,
        user_ref_id: Option<UserRefId>,
        validate: bool,
    ) -> Result<AddOrderResponse> {
        let req = AddOrderRequest {
            ordertype: OrderType::Limit,
            bs_type: limit_order.bs_type,
            volume: limit_order.volume,
            pair: limit_order.pair,
            price: limit_order.price,
            oflags: limit_order.oflags,
            userref: user_ref_id,
            validate,
            leverage: limit_order.leverage,
        };
        let result: Result<KrakenResult<AddOrderResponse>> = self.client.query_private("AddOrder", req);
        result.and_then(unpack_kraken_result)
    }

    /// (Private) Place a settle-position order
    ///
    /// Arguments:
    /// * settle_position_order: Settle-position order object describing the parameters of the order
    /// * user_ref_id: Optional user ref id to attach to the order
    /// * validate: If true, the order is only validated and is not actually placed
    pub fn add_settle_position_order(
        &self,
        settle_position_order: SettlePositionOrder,
        user_ref_id: Option<UserRefId>,
        validate: bool,
    ) -> Result<AddOrderResponse> {
        let req = AddOrderRequest {
            ordertype: OrderType::SettlePosition,
            bs_type: settle_position_order.bs_type,
            volume: settle_position_order.volume,
            pair: settle_position_order.pair,
            oflags: settle_position_order.oflags,
            userref: user_ref_id,
            validate,
            price: Default::default(),
            leverage: settle_position_order.leverage,
        };
        let result: Result<KrakenResult<AddOrderResponse>> = self.client.query_private("AddOrder", req);
        result.and_then(unpack_kraken_result)
    }
}

impl TryFrom<KrakenRestConfig> for KrakenRestAPI {
    type Error = Error;
    fn try_from(src: KrakenRestConfig) -> Result<Self> {
        Ok(KrakenRestAPI {
            client: KrakenRestClient::try_from(src)?,
        })
    }
}
