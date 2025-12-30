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

pub async fn fetch(address: &str) -> Result<Balance, BalanceError> {
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

pub async fn fetch_many(addresses: &[&str]) -> Vec<Result<Balance, BalanceError>> {
    let futures: Vec<_> = addresses.iter().map(|addr| fetch(addr)).collect();
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
}
