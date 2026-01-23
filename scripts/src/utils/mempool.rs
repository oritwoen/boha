use crate::utils::{cache_path, timestamp_to_date, Transaction, RATE_LIMIT_DELAY, RETRY_DELAY};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const DUST_THRESHOLD: u64 = 10_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolTx {
    pub txid: String,
    pub status: MempoolStatus,
    pub vin: Vec<MempoolVin>,
    pub vout: Vec<MempoolVout>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolStatus {
    pub block_time: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolVin {
    pub prevout: Option<MempoolPrevout>,
    pub scriptsig: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolPrevout {
    pub scriptpubkey_address: Option<String>,
    pub value: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolVout {
    pub scriptpubkey_address: Option<String>,
    pub value: u64,
}

fn sats_to_btc(sats: u64) -> f64 {
    sats as f64 / 100_000_000.0
}

pub fn load_from_cache(collection: &str, address: &str) -> Option<Vec<MempoolTx>> {
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
    txs: &[MempoolTx],
) -> Result<(), Box<dyn std::error::Error>> {
    let path = cache_path(collection, address);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(txs)?;
    std::fs::write(path, content)?;
    Ok(())
}

async fn fetch_with_retry(
    client: &reqwest::Client,
    url: &str,
) -> Result<Vec<MempoolTx>, Box<dyn std::error::Error>> {
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

        let response = match client.get(url).send().await {
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

        return Ok(response.json().await?);
    }

    Err("Rate limited after 5 attempts".into())
}

pub async fn fetch_transactions(
    client: &reqwest::Client,
    address: &str,
) -> Result<Vec<MempoolTx>, Box<dyn std::error::Error>> {
    let mut all_txs: Vec<MempoolTx> = Vec::new();
    let mut last_txid: Option<String> = None;

    loop {
        let url = match &last_txid {
            Some(txid) => format!(
                "https://mempool.space/api/address/{}/txs/chain/{}",
                address, txid
            ),
            None => format!("https://mempool.space/api/address/{}/txs", address),
        };

        tokio::time::sleep(RATE_LIMIT_DELAY).await;

        let txs = fetch_with_retry(client, &url).await?;

        if txs.is_empty() {
            break;
        }

        last_txid = txs.last().map(|tx| tx.txid.clone());
        all_txs.extend(txs);
    }

    all_txs.sort_by_key(|tx| tx.status.block_time.unwrap_or(i64::MAX));

    Ok(all_txs)
}

pub fn categorize_transactions(
    puzzle_address: &str,
    txs: Vec<MempoolTx>,
    author_addresses: &HashSet<String>,
    puzzle_status: &str,
) -> Vec<Transaction> {
    let mut result = Vec::new();

    let mut sorted_txs = txs;
    sorted_txs.sort_by_key(|tx| tx.status.block_time.unwrap_or(i64::MAX));

    let mut has_funding = false;

    for tx in &sorted_txs {
        let amount_to_puzzle: u64 = tx
            .vout
            .iter()
            .filter(|o| o.scriptpubkey_address.as_deref() == Some(puzzle_address))
            .map(|o| o.value)
            .sum();

        let puzzle_is_sender = tx.vin.iter().any(|i| {
            i.prevout
                .as_ref()
                .and_then(|p| p.scriptpubkey_address.as_deref())
                == Some(puzzle_address)
        });

        let amount_from_puzzle: u64 = tx
            .vin
            .iter()
            .filter_map(|i| i.prevout.as_ref())
            .filter(|p| p.scriptpubkey_address.as_deref() == Some(puzzle_address))
            .map(|p| p.value)
            .sum();

        let amount_to_author: u64 = tx
            .vout
            .iter()
            .filter(|o| {
                o.scriptpubkey_address
                    .as_ref()
                    .is_some_and(|addr| author_addresses.contains(addr))
            })
            .map(|o| o.value)
            .sum();

        let block_time = tx.status.block_time.unwrap_or(0);

        if amount_to_puzzle > 0 {
            let tx_type = match has_funding {
                false => "funding",
                true => "increase",
            };

            result.push(Transaction {
                tx_type: tx_type.to_string(),
                txid: tx.txid.clone(),
                date: Some(timestamp_to_date(block_time)),
                amount: Some(sats_to_btc(amount_to_puzzle)),
            });
            has_funding = true;
        }

        if puzzle_is_sender && amount_to_author > 0 {
            result.push(Transaction {
                tx_type: "decrease".to_string(),
                txid: tx.txid.clone(),
                date: Some(timestamp_to_date(block_time)),
                amount: Some(sats_to_btc(amount_to_author)),
            });
        }

        if puzzle_is_sender && amount_from_puzzle > 0 && amount_from_puzzle <= DUST_THRESHOLD {
            result.push(Transaction {
                tx_type: "pubkey_reveal".to_string(),
                txid: tx.txid.clone(),
                date: Some(timestamp_to_date(block_time)),
                amount: Some(sats_to_btc(amount_from_puzzle)),
            });
        } else if puzzle_is_sender && amount_from_puzzle > DUST_THRESHOLD && amount_to_author == 0 {
            let tx_type = if puzzle_status == "swept" {
                "sweep"
            } else {
                "claim"
            };
            result.push(Transaction {
                tx_type: tx_type.to_string(),
                txid: tx.txid.clone(),
                date: Some(timestamp_to_date(block_time)),
                amount: Some(sats_to_btc(amount_from_puzzle)),
            });
        }
    }

    result
}
