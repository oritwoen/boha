use crate::utils::{cache_path, timestamp_to_date, Transaction, RETRY_DELAY};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::Duration;

const ETH_RATE_LIMIT_DELAY: Duration = Duration::from_millis(250);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EtherscanResponse {
    pub status: String,
    pub message: String,
    pub result: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EtherscanTx {
    pub hash: String,
    #[serde(rename = "timeStamp")]
    pub timestamp: String,
    pub from: String,
    pub to: String,
    pub value: String,
    #[serde(rename = "isError")]
    pub is_error: String,
}

fn wei_to_eth(wei: &str) -> f64 {
    wei.parse::<u128>().unwrap_or(0) as f64 / 1e18
}

pub fn load_from_cache(collection: &str, address: &str) -> Option<Vec<EtherscanTx>> {
    let path = cache_path(collection, address);
    if path.exists() {
        let content = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    } else {
        None
    }
}

pub fn save_to_cache(
    collection: &str,
    address: &str,
    txs: &[EtherscanTx],
) -> Result<(), Box<dyn std::error::Error>> {
    let path = cache_path(collection, address);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(txs)?;
    std::fs::write(path, content)?;
    Ok(())
}

pub async fn fetch_transactions(
    client: &reqwest::Client,
    address: &str,
    api_key: &str,
) -> Result<Vec<EtherscanTx>, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.etherscan.io/v2/api?chainid=1&module=account&action=txlist&address={}&startblock=0&endblock=99999999&sort=asc&apikey={}",
        address, api_key
    );

    for attempt in 0..5 {
        if attempt > 0 {
            let delay = RETRY_DELAY * (1 << attempt.min(3));
            eprintln!(
                "    Retry {}/5, waiting {}s...",
                attempt + 1,
                delay.as_secs()
            );
            tokio::time::sleep(delay).await;
        }

        tokio::time::sleep(ETH_RATE_LIMIT_DELAY).await;

        let response = match client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("    Request error: {}", e);
                continue;
            }
        };

        if response.status().as_u16() == 429 {
            continue;
        }

        if !response.status().is_success() {
            return Err(format!("API error: {}", response.status()).into());
        }

        let api_response: EtherscanResponse = response.json().await?;

        if api_response.status != "1" {
            if let Some(msg) = api_response.result.as_str() {
                return Err(format!("Etherscan API error: {}", msg).into());
            }
            return Err(format!("Etherscan API error: {}", api_response.message).into());
        }

        let txs: Vec<EtherscanTx> = serde_json::from_value(api_response.result)?;
        return Ok(txs);
    }

    Err("Rate limited after 5 attempts".into())
}

pub fn categorize_transactions(
    puzzle_address: &str,
    txs: Vec<EtherscanTx>,
    author_addresses: &HashSet<String>,
    puzzle_status: &str,
) -> Vec<Transaction> {
    let mut result = Vec::new();
    let puzzle_lower = puzzle_address.to_lowercase();

    let mut has_funding = false;

    for tx in &txs {
        if tx.is_error == "1" {
            continue;
        }

        let to_lower = tx.to.to_lowercase();
        let from_lower = tx.from.to_lowercase();
        let timestamp: i64 = tx.timestamp.parse().unwrap_or(0);
        let amount = wei_to_eth(&tx.value);

        if amount == 0.0 {
            continue;
        }

        if to_lower == puzzle_lower {
            let tx_type = if !has_funding { "funding" } else { "increase" };

            result.push(Transaction {
                tx_type: tx_type.to_string(),
                txid: tx.hash.clone(),
                date: Some(timestamp_to_date(timestamp)),
                amount: Some(amount),
            });
            has_funding = true;
        } else if from_lower == puzzle_lower {
            let to_author = author_addresses
                .iter()
                .any(|a| a.to_lowercase() == to_lower);

            let tx_type = if to_author {
                "decrease"
            } else if puzzle_status == "swept" {
                "sweep"
            } else {
                "claim"
            };

            result.push(Transaction {
                tx_type: tx_type.to_string(),
                txid: tx.hash.clone(),
                date: Some(timestamp_to_date(timestamp)),
                amount: Some(amount),
            });
        }
    }

    result
}
