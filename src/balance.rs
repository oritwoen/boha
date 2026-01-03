use crate::Chain;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BalanceError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    #[error("API error: {0}")]
    Api(String),
    #[error("Unsupported chain: {0}")]
    UnsupportedChain(String),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Balance {
    pub confirmed: u64,
    pub unconfirmed: i64,
}

impl Balance {
    pub fn total(&self) -> i64 {
        self.confirmed as i64 + self.unconfirmed
    }

    pub fn confirmed_btc(&self) -> f64 {
        self.confirmed as f64 / 100_000_000.0
    }

    pub fn total_btc(&self) -> f64 {
        self.total() as f64 / 100_000_000.0
    }

    pub fn confirmed_eth(&self) -> f64 {
        self.confirmed as f64 / 1e18
    }

    pub fn total_eth(&self) -> f64 {
        self.total() as f64 / 1e18
    }
}

#[derive(Deserialize)]
struct MempoolAddressResponse {
    chain_stats: MempoolStats,
    mempool_stats: MempoolStats,
}

#[derive(Deserialize)]
struct MempoolStats {
    funded_txo_sum: u64,
    spent_txo_sum: u64,
}

#[derive(Deserialize)]
struct EtherscanResponse {
    status: String,
    message: String,
    result: String,
}

async fn fetch_btc(address: &str) -> Result<Balance, BalanceError> {
    let url = format!("https://mempool.space/api/address/{}", address);

    let response: MempoolAddressResponse = reqwest::get(&url)
        .await?
        .error_for_status()
        .map_err(|e| {
            if e.status() == Some(reqwest::StatusCode::BAD_REQUEST) {
                BalanceError::InvalidAddress(address.to_string())
            } else {
                BalanceError::Request(e)
            }
        })?
        .json()
        .await?;

    let confirmed = response.chain_stats.funded_txo_sum - response.chain_stats.spent_txo_sum;
    let unconfirmed =
        response.mempool_stats.funded_txo_sum as i64 - response.mempool_stats.spent_txo_sum as i64;

    Ok(Balance {
        confirmed,
        unconfirmed,
    })
}

async fn fetch_eth(address: &str) -> Result<Balance, BalanceError> {
    dotenvy::dotenv().ok();

    let api_key = std::env::var("ETHERSCAN_API_KEY")
        .map_err(|_| BalanceError::Api("ETHERSCAN_API_KEY environment variable not set".into()))?;

    let url = format!(
        "https://api.etherscan.io/v2/api?chainid=1&module=account&action=balance&address={}&apikey={}",
        address, api_key
    );

    let response: EtherscanResponse = reqwest::get(&url)
        .await?
        .error_for_status()
        .map_err(BalanceError::Request)?
        .json()
        .await?;

    if response.status != "1" {
        return Err(BalanceError::Api(format!(
            "Etherscan API error: {}",
            response.message
        )));
    }

    let wei: u128 = response
        .result
        .parse()
        .map_err(|_| BalanceError::Api("Failed to parse balance".into()))?;

    Ok(Balance {
        confirmed: wei as u64,
        unconfirmed: 0,
    })
}

pub async fn fetch(address: &str, chain: Chain) -> Result<Balance, BalanceError> {
    match chain {
        Chain::Bitcoin => fetch_btc(address).await,
        Chain::Ethereum => fetch_eth(address).await,
        _ => Err(BalanceError::UnsupportedChain(chain.name().to_string())),
    }
}

pub async fn fetch_many(addresses: &[(&str, Chain)]) -> Vec<Result<Balance, BalanceError>> {
    let futures: Vec<_> = addresses
        .iter()
        .map(|(addr, chain)| fetch(addr, *chain))
        .collect();
    futures::future::join_all(futures).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balance_btc_conversion() {
        let balance = Balance {
            confirmed: 100_000_000,
            unconfirmed: 50_000_000,
        };

        assert_eq!(balance.confirmed_btc(), 1.0);
        assert_eq!(balance.total_btc(), 1.5);
        assert_eq!(balance.total(), 150_000_000);
    }

    #[test]
    fn test_balance_negative_unconfirmed() {
        let balance = Balance {
            confirmed: 100_000_000,
            unconfirmed: -30_000_000,
        };

        assert_eq!(balance.total(), 70_000_000);
        assert_eq!(balance.total_btc(), 0.7);
    }

    #[test]
    fn test_balance_zero() {
        let balance = Balance::default();

        assert_eq!(balance.confirmed, 0);
        assert_eq!(balance.unconfirmed, 0);
        assert_eq!(balance.confirmed_btc(), 0.0);
        assert_eq!(balance.total_btc(), 0.0);
        assert_eq!(balance.total(), 0);
    }

    #[test]
    fn test_balance_max_btc_supply() {
        let balance = Balance {
            confirmed: 2_100_000_000_000_000,
            unconfirmed: 0,
        };

        assert_eq!(balance.confirmed_btc(), 21_000_000.0);
        assert_eq!(balance.total_btc(), 21_000_000.0);
    }

    #[test]
    fn test_balance_negative_total_from_large_pending_outgoing() {
        let balance = Balance {
            confirmed: 100_000_000,
            unconfirmed: -150_000_000,
        };

        assert_eq!(balance.total(), -50_000_000);
        assert_eq!(balance.total_btc(), -0.5);
    }

    #[test]
    fn test_balance_eth_conversion() {
        let balance = Balance {
            confirmed: 1_000_000_000_000_000_000,
            unconfirmed: 0,
        };

        assert_eq!(balance.confirmed_eth(), 1.0);
        assert_eq!(balance.total_eth(), 1.0);
    }

    #[tokio::test]
    #[ignore]
    async fn test_fetch_btc_satoshi_genesis_address_has_funds() {
        let result = fetch("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa", Chain::Bitcoin).await;
        assert!(result.is_ok());
        let balance = result.unwrap();
        assert!(balance.confirmed > 0);
    }

    #[tokio::test]
    #[ignore]
    async fn test_fetch_btc_invalid_address_returns_error() {
        let result = fetch("invalid_address_xyz", Chain::Bitcoin).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore]
    async fn test_fetch_btc_valid_empty_address() {
        let result = fetch("1111111111111111111114oLvT2", Chain::Bitcoin).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_fetch_many_btc_known_addresses() {
        let genesis = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
        let satoshi_dice = "1dice8EMZmqKvrGE4Qc9bUFf9PX3xaYDp";
        let results =
            fetch_many(&[(genesis, Chain::Bitcoin), (satoshi_dice, Chain::Bitcoin)]).await;

        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_fetch_eth_vitalik_address() {
        let result = fetch(
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            Chain::Ethereum,
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_fetch_unsupported_chain() {
        let result = fetch("some_address", Chain::Litecoin).await;
        assert!(matches!(result, Err(BalanceError::UnsupportedChain(_))));
    }
}
