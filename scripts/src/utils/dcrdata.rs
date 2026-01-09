use crate::utils::{cache_path, timestamp_to_date, Transaction, RATE_LIMIT_DELAY, RETRY_DELAY};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const PAGE_SIZE: u32 = 50;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DcrdataTx {
    pub txid: String,
    pub time: Option<i64>,
    pub vin: Vec<DcrdataVin>,
    pub vout: Vec<DcrdataVout>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DcrdataVin {
    pub txid: String,
    pub vout: u32,
    pub amountin: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DcrdataVout {
    pub value: f64,
    pub n: u32,
    #[serde(rename = "scriptPubKey")]
    pub script_pub_key: DcrdataScriptPubKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DcrdataScriptPubKey {
    pub addresses: Option<Vec<String>>,
}

pub fn load_from_cache(collection: &str, address: &str) -> Option<Vec<DcrdataTx>> {
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
    txs: &[DcrdataTx],
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
) -> Result<Vec<DcrdataTx>, Box<dyn std::error::Error>> {
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
) -> Result<Vec<DcrdataTx>, Box<dyn std::error::Error>> {
    let mut all_txs: Vec<DcrdataTx> = Vec::new();
    let mut skip: u32 = 0;

    loop {
        let url = format!(
            "https://dcrdata.decred.org/api/address/{}/count/{}/skip/{}/raw",
            address, PAGE_SIZE, skip
        );

        tokio::time::sleep(RATE_LIMIT_DELAY).await;

        let txs = fetch_with_retry(client, &url).await?;

        if txs.is_empty() {
            break;
        }

        let fetched_count = txs.len() as u32;
        all_txs.extend(txs);

        if fetched_count < PAGE_SIZE {
            break;
        }

        skip += fetched_count;
    }

    all_txs.sort_by_key(|tx| tx.time.unwrap_or(i64::MAX));

    Ok(all_txs)
}

pub fn categorize_transactions(
    puzzle_address: &str,
    txs: Vec<DcrdataTx>,
    author_addresses: &HashSet<String>,
    puzzle_status: &str,
) -> Vec<Transaction> {
    let mut result = Vec::new();
    let mut has_funding = false;

    let mut sorted_txs = txs;
    sorted_txs.sort_by_key(|tx| tx.time.unwrap_or(i64::MAX));

    for tx in &sorted_txs {
        let amount_to_puzzle: f64 = tx
            .vout
            .iter()
            .filter(|o| {
                o.script_pub_key
                    .addresses
                    .as_ref()
                    .is_some_and(|addrs| addrs.contains(&puzzle_address.to_string()))
            })
            .map(|o| o.value)
            .sum();

        let puzzle_is_sender = tx.vin.iter().any(|input| {
            sorted_txs.iter().any(|prev_tx| {
                prev_tx.txid == input.txid
                    && prev_tx.vout.get(input.vout as usize).is_some_and(|out| {
                        out.script_pub_key
                            .addresses
                            .as_ref()
                            .is_some_and(|addrs| addrs.contains(&puzzle_address.to_string()))
                    })
            })
        });

        let amount_from_puzzle: f64 =
            tx.vin
                .iter()
                .filter(|input| {
                    sorted_txs.iter().any(|prev_tx| {
                        prev_tx.txid == input.txid
                            && prev_tx.vout.get(input.vout as usize).is_some_and(|out| {
                                out.script_pub_key.addresses.as_ref().is_some_and(|addrs| {
                                    addrs.contains(&puzzle_address.to_string())
                                })
                            })
                    })
                })
                .map(|input| input.amountin)
                .sum();

        let amount_to_author: f64 = tx
            .vout
            .iter()
            .filter(|o| {
                o.script_pub_key
                    .addresses
                    .as_ref()
                    .is_some_and(|addrs| addrs.iter().any(|a| author_addresses.contains(a)))
            })
            .map(|o| o.value)
            .sum();

        let block_time = tx.time.unwrap_or(0);

        if amount_to_puzzle > 0.0 {
            let tx_type = match has_funding {
                false => "funding",
                true => "increase",
            };

            result.push(Transaction {
                tx_type: tx_type.to_string(),
                txid: tx.txid.clone(),
                date: Some(timestamp_to_date(block_time)),
                amount: Some(amount_to_puzzle),
            });
            has_funding = true;
        }

        if puzzle_is_sender && amount_to_author > 0.0 {
            result.push(Transaction {
                tx_type: "decrease".to_string(),
                txid: tx.txid.clone(),
                date: Some(timestamp_to_date(block_time)),
                amount: Some(amount_to_author),
            });
        }

        if puzzle_is_sender && amount_from_puzzle > 0.0 && amount_to_author == 0.0 {
            let tx_type = if puzzle_status == "swept" {
                "sweep"
            } else {
                "claim"
            };
            result.push(Transaction {
                tx_type: tx_type.to_string(),
                txid: tx.txid.clone(),
                date: Some(timestamp_to_date(block_time)),
                amount: Some(amount_from_puzzle),
            });
        }
    }

    result
}
