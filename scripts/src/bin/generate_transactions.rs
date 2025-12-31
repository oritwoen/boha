use chrono::{DateTime, Utc};
use serde::Deserialize;

use std::path::Path;
use std::time::Duration;
use toml_edit::{Array, DocumentMut, InlineTable, Item, Value};

const RATE_LIMIT_DELAY: Duration = Duration::from_secs(5);

#[derive(Debug, Deserialize)]
struct MempoolTx {
    txid: String,
    status: TxStatus,
    vin: Vec<TxInput>,
    vout: Vec<TxOutput>,
}

#[derive(Debug, Deserialize)]
struct TxStatus {
    confirmed: bool,
    block_time: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct TxInput {
    prevout: Option<Prevout>,
}

#[derive(Debug, Deserialize)]
struct Prevout {
    scriptpubkey_address: Option<String>,
    value: u64,
}

#[derive(Debug, Deserialize)]
struct TxOutput {
    scriptpubkey_address: Option<String>,
    value: u64,
}

#[derive(Debug, Clone)]
struct Transaction {
    tx_type: String,
    txid: String,
    date: Option<String>,
    amount: Option<f64>,
}

fn timestamp_to_date(timestamp: i64) -> String {
    let dt = DateTime::<Utc>::from_timestamp(timestamp, 0).unwrap_or_default();
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn sats_to_btc(sats: u64) -> f64 {
    sats as f64 / 100_000_000.0
}

async fn fetch_transactions(
    client: &reqwest::Client,
    address: &str,
) -> Result<Vec<MempoolTx>, Box<dyn std::error::Error>> {
    let url = format!("https://mempool.space/api/address/{}/txs", address);
    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()).into());
    }

    let txs: Vec<MempoolTx> = response.json().await?;
    Ok(txs)
}

fn categorize_transactions(
    address: &str,
    txs: Vec<MempoolTx>,
    status: &str,
) -> Vec<Transaction> {
    let mut result = Vec::new();

    let mut sorted_txs = txs;
    sorted_txs.sort_by_key(|tx| tx.status.block_time.unwrap_or(i64::MAX));

    let mut is_first_incoming = true;
    let mut last_outgoing: Option<&MempoolTx> = None;

    for tx in &sorted_txs {
        let is_incoming = tx
            .vout
            .iter()
            .any(|o| o.scriptpubkey_address.as_deref() == Some(address));
        let is_outgoing = tx
            .vin
            .iter()
            .any(|i| i.prevout.as_ref().and_then(|p| p.scriptpubkey_address.as_deref()) == Some(address));

        if is_outgoing {
            last_outgoing = Some(tx);
        }

        if is_incoming && is_first_incoming {
            let amount: u64 = tx
                .vout
                .iter()
                .filter(|o| o.scriptpubkey_address.as_deref() == Some(address))
                .map(|o| o.value)
                .sum();

            result.push(Transaction {
                tx_type: "funding".to_string(),
                txid: tx.txid.clone(),
                date: tx.status.block_time.map(timestamp_to_date),
                amount: Some(sats_to_btc(amount)),
            });
            is_first_incoming = false;
        }
    }

    if let Some(tx) = last_outgoing {
        let amount: u64 = tx
            .vin
            .iter()
            .filter_map(|i| i.prevout.as_ref())
            .filter(|p| p.scriptpubkey_address.as_deref() == Some(address))
            .map(|p| p.value)
            .sum();

        let tx_type = match status {
            "solved" | "claimed" => "claim",
            "swept" => "sweep",
            _ => return result,
        };

        result.push(Transaction {
            tx_type: tx_type.to_string(),
            txid: tx.txid.clone(),
            date: tx.status.block_time.map(timestamp_to_date),
            amount: Some(sats_to_btc(amount)),
        });
    }

    result
}

fn transaction_to_inline_table(tx: &Transaction) -> InlineTable {
    let mut table = InlineTable::new();
    table.insert("type", Value::from(tx.tx_type.as_str()));
    table.insert("txid", Value::from(tx.txid.as_str()));
    if let Some(date) = &tx.date {
        table.insert("date", Value::from(date.as_str()));
    }
    if let Some(amount) = tx.amount {
        table.insert("amount", Value::from(amount));
    }
    table
}

fn transactions_to_array(transactions: &[Transaction]) -> Array {
    let mut array = Array::new();
    for tx in transactions {
        array.push(Value::InlineTable(transaction_to_inline_table(tx)));
    }
    array
}

async fn process_b1000(
    client: &reqwest::Client,
    doc: &mut DocumentMut,
    known_funding_txid: &str,
    known_funding_date: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut count = 0;

    if let Some(puzzles) = doc.get_mut("puzzles") {
        if let Some(array) = puzzles.as_array_of_tables_mut() {
            for (idx, table) in array.iter_mut().enumerate() {
                if table.get("transactions").is_some() {
                    continue;
                }

                let address = table
                    .get("address")
                    .and_then(|a| a.as_str())
                    .unwrap_or("")
                    .to_string();
                let status = table
                    .get("status")
                    .and_then(|s| s.as_str())
                    .unwrap_or("unsolved")
                    .to_string();
                let bits = table.get("bits").and_then(|b| b.as_integer()).unwrap_or(0);
                let prize = table.get("prize").and_then(|p| p.as_float());

                println!("  [{}/256] Processing puzzle {} ({})", idx + 1, bits, address);

                let mut transactions = Vec::new();

                transactions.push(Transaction {
                    tx_type: "funding".to_string(),
                    txid: known_funding_txid.to_string(),
                    date: Some(known_funding_date.to_string()),
                    amount: prize,
                });

                if status == "solved" || status == "claimed" || status == "swept" {
                    tokio::time::sleep(RATE_LIMIT_DELAY).await;

                    match fetch_transactions(client, &address).await {
                        Ok(txs) => {
                            let categorized = categorize_transactions(&address, txs, &status);
                            for tx in categorized {
                                if tx.tx_type != "funding" {
                                    transactions.push(tx);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("    Error fetching transactions: {}", e);
                        }
                    }
                }

                if !transactions.is_empty() {
                    table.insert("transactions", Item::Value(Value::Array(transactions_to_array(&transactions))));
                    count += 1;
                }
            }
        }
    }

    Ok(count)
}

async fn process_collection(
    client: &reqwest::Client,
    doc: &mut DocumentMut,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut count = 0;

    if let Some(puzzles) = doc.get_mut("puzzles") {
        if let Some(array) = puzzles.as_array_of_tables_mut() {
            let total = array.len();
            for (idx, table) in array.iter_mut().enumerate() {
                if table.get("transactions").is_some() {
                    continue;
                }

                let address = table
                    .get("address")
                    .and_then(|a| a.as_str())
                    .unwrap_or("")
                    .to_string();
                let status = table
                    .get("status")
                    .and_then(|s| s.as_str())
                    .unwrap_or("unsolved")
                    .to_string();
                let name = table
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                println!("  [{}/{}] Processing {} ({})", idx + 1, total, name, address);

                tokio::time::sleep(RATE_LIMIT_DELAY).await;

                match fetch_transactions(client, &address).await {
                    Ok(txs) => {
                        let transactions = categorize_transactions(&address, txs, &status);
                        if !transactions.is_empty() {
                            table.insert("transactions", Item::Value(Value::Array(transactions_to_array(&transactions))));
                            count += 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("    Error fetching transactions: {}", e);
                    }
                }
            }
        }
    }

    Ok(count)
}

async fn process_gsmg(
    client: &reqwest::Client,
    doc: &mut DocumentMut,
) -> Result<usize, Box<dyn std::error::Error>> {
    if let Some(puzzle) = doc.get_mut("puzzle") {
        if let Some(table) = puzzle.as_table_mut() {
            if table.get("transactions").is_some() {
                return Ok(0);
            }

            let address = table
                .get("address")
                .and_then(|a| a.as_str())
                .unwrap_or("")
                .to_string();
            let status = table
                .get("status")
                .and_then(|s| s.as_str())
                .unwrap_or("unsolved")
                .to_string();

            println!("  Processing gsmg ({})", address);

            tokio::time::sleep(RATE_LIMIT_DELAY).await;

            match fetch_transactions(client, &address).await {
                Ok(txs) => {
                    let transactions = categorize_transactions(&address, txs, &status);
                    if !transactions.is_empty() {
                        table.insert("transactions", Item::Value(Value::Array(transactions_to_array(&transactions))));
                        return Ok(1);
                    }
                }
                Err(e) => {
                    eprintln!("    Error fetching transactions: {}", e);
                }
            }
        }
    }

    Ok(0)
}

async fn process_toml_file(
    client: &reqwest::Client,
    path: &Path,
    collection: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Processing: {}", path.display());



    let content = std::fs::read_to_string(path)?;
    let mut doc: DocumentMut = content.parse()?;

    let count = match collection {
        "b1000" => {
            process_b1000(
                client,
                &mut doc,
                "08389f34c98c606322740c0be6a7125d9860bb8d5cb182c02f98461e5fa6cd15",
                "2015-01-15 18:07:14",
            )
            .await?
        }
        "gsmg" => process_gsmg(client, &mut doc).await?,
        _ => process_collection(client, &mut doc).await?,
    };

    if count > 0 {
        std::fs::write(path, doc.to_string())?;
        println!("  Updated {} puzzles with transactions", count);
    } else {
        println!("  No updates needed");
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let collections: Vec<&str> = if args.len() > 1 {
        args[1..].iter().map(|s| s.as_str()).collect()
    } else {
        vec!["b1000", "gsmg", "hash_collision"]
    };

    let client = reqwest::Client::builder()
        .user_agent("boha-scripts/0.1")
        .timeout(Duration::from_secs(30))
        .build()?;

    let data_dir = Path::new("../data");

    for collection in collections {
        let filename = format!("{}.toml", collection);
        let path = data_dir.join(&filename);

        if path.exists() {
            process_toml_file(&client, &path, collection).await?;
        } else {
            eprintln!("File not found: {}", path.display());
        }
    }

    println!("\nDone!");
    Ok(())
}
