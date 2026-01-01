use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;
use toml_edit::{Array, DocumentMut, InlineTable, Item, Value};

const RATE_LIMIT_DELAY: Duration = Duration::from_secs(10);
const RETRY_DELAY: Duration = Duration::from_secs(60);

fn extract_author_addresses(doc: &DocumentMut) -> HashSet<String> {
    let mut addresses = HashSet::new();
    
    if let Some(author) = doc.get("author") {
        if let Some(table) = author.as_table() {
            if let Some(addrs) = table.get("addresses") {
                if let Some(arr) = addrs.as_array() {
                    for addr in arr.iter() {
                        if let Some(s) = addr.as_str() {
                            addresses.insert(s.to_string());
                        }
                    }
                }
            }
        }
    }
    
    addresses
}

// Esplora API structures (blockstream.info / mempool.space)
#[derive(Debug, Deserialize)]
struct EsploraTx {
    txid: String,
    status: EsploraStatus,
    vin: Vec<EsploraVin>,
    vout: Vec<EsploraVout>,
}

#[derive(Debug, Deserialize)]
struct EsploraStatus {
    block_time: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct EsploraVin {
    prevout: Option<EsploraPrevout>,
}

#[derive(Debug, Deserialize)]
struct EsploraPrevout {
    scriptpubkey_address: Option<String>,
    value: u64,
}

#[derive(Debug, Deserialize)]
struct EsploraVout {
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

async fn fetch_with_retry(
    client: &reqwest::Client,
    url: &str,
) -> Result<Vec<EsploraTx>, Box<dyn std::error::Error>> {
    for attempt in 0..5 {
        if attempt > 0 {
            let delay = RETRY_DELAY * (1 << attempt.min(3));
            eprintln!("    Retry {}/5, waiting {}s...", attempt + 1, delay.as_secs());
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

async fn fetch_transactions(
    client: &reqwest::Client,
    address: &str,
) -> Result<Vec<EsploraTx>, Box<dyn std::error::Error>> {
    let mut all_txs: Vec<EsploraTx> = Vec::new();
    let mut last_txid: Option<String> = None;

    loop {
        let url = match &last_txid {
            Some(txid) => format!(
                "https://blockstream.info/api/address/{}/txs/chain/{}",
                address, txid
            ),
            None => format!("https://blockstream.info/api/address/{}/txs", address),
        };

        tokio::time::sleep(RATE_LIMIT_DELAY).await;

        let txs = fetch_with_retry(client, &url).await?;

        if txs.is_empty() {
            break;
        }

        last_txid = txs.last().map(|tx| tx.txid.clone());
        all_txs.extend(txs);
    }

    all_txs.sort_by_key(|tx| tx.status.block_time.unwrap_or(0));

    Ok(all_txs)
}

fn categorize_transactions(
    puzzle_address: &str,
    txs: Vec<EsploraTx>,
    author_addresses: &HashSet<String>,
) -> Vec<Transaction> {
    let mut result = Vec::new();

    let mut sorted_txs = txs;
    sorted_txs.sort_by_key(|tx| tx.status.block_time.unwrap_or(0));

    let mut has_funding = false;

    for tx in &sorted_txs {
        let author_is_sender = tx.vin.iter().any(|i| {
            i.prevout
                .as_ref()
                .and_then(|p| p.scriptpubkey_address.as_ref())
                .is_some_and(|addr| author_addresses.contains(addr))
        });

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

        let amount_to_solver: u64 = tx
            .vout
            .iter()
            .filter(|o| {
                o.scriptpubkey_address.as_ref().is_some_and(|addr| {
                    addr != puzzle_address && !author_addresses.contains(addr)
                })
            })
            .map(|o| o.value)
            .sum();

        let block_time = tx.status.block_time.unwrap_or(0);

        if author_is_sender && amount_to_puzzle > 0 {
            let tx_type = if !has_funding { "funding" } else { "increase" };
            has_funding = true;

            result.push(Transaction {
                tx_type: tx_type.to_string(),
                txid: tx.txid.clone(),
                date: Some(timestamp_to_date(block_time)),
                amount: Some(sats_to_btc(amount_to_puzzle)),
            });
        }

        if puzzle_is_sender && amount_to_author > 0 {
            result.push(Transaction {
                tx_type: "decrease".to_string(),
                txid: tx.txid.clone(),
                date: Some(timestamp_to_date(block_time)),
                amount: Some(sats_to_btc(amount_to_author)),
            });
        }

        if puzzle_is_sender && amount_to_solver > 0 && amount_to_author == 0 {
            result.push(Transaction {
                tx_type: "claim".to_string(),
                txid: tx.txid.clone(),
                date: Some(timestamp_to_date(block_time)),
                amount: Some(sats_to_btc(amount_to_solver)),
            });
        }
    }

    result
}

fn extract_existing_transactions(table: &toml_edit::Table) -> Vec<Transaction> {
    let mut result = Vec::new();

    if let Some(txs) = table.get("transactions") {
        if let Some(arr) = txs.as_array() {
            for item in arr.iter() {
                if let Some(inline) = item.as_inline_table() {
                    let tx_type = inline
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let txid = inline
                        .get("txid")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let date = inline.get("date").and_then(|v| v.as_str()).map(String::from);
                    let amount = inline.get("amount").and_then(|v| v.as_float());

                    if !txid.is_empty() {
                        result.push(Transaction {
                            tx_type,
                            txid,
                            date,
                            amount,
                        });
                    }
                }
            }
        }
    }

    result
}

fn merge_transactions(existing: Vec<Transaction>, new: Vec<Transaction>) -> Vec<Transaction> {
    let existing_txids: HashSet<String> = existing.iter().map(|t| t.txid.clone()).collect();

    let mut merged = existing;
    for tx in new {
        if !existing_txids.contains(&tx.txid) {
            merged.push(tx);
        }
    }

    merged.sort_by(|a, b| a.date.cmp(&b.date));
    merged
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
    author_addresses: &HashSet<String>,
    filter_puzzle: Option<i64>,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut count = 0;

    if let Some(puzzles) = doc.get_mut("puzzles") {
        if let Some(array) = puzzles.as_array_of_tables_mut() {
            for (idx, table) in array.iter_mut().enumerate() {
                let bits = table.get("bits").and_then(|b| b.as_integer()).unwrap_or(0);

                if let Some(filter) = filter_puzzle {
                    if bits != filter {
                        continue;
                    }
                }

                let address = table
                    .get("address")
                    .and_then(|a| a.as_str())
                    .unwrap_or("")
                    .to_string();

                println!("  [{}/256] Processing puzzle {} ({})", idx + 1, bits, address);

                tokio::time::sleep(RATE_LIMIT_DELAY).await;

                let existing = extract_existing_transactions(table);

                match fetch_transactions(client, &address).await {
                    Ok(txs) => {
                        let new_transactions =
                            categorize_transactions(&address, txs, author_addresses);
                        let merged = merge_transactions(existing, new_transactions);
                        if !merged.is_empty() {
                            table.insert(
                                "transactions",
                                Item::Value(Value::Array(transactions_to_array(&merged))),
                            );
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

async fn process_collection(
    client: &reqwest::Client,
    doc: &mut DocumentMut,
    author_addresses: &HashSet<String>,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut count = 0;

    if let Some(puzzles) = doc.get_mut("puzzles") {
        if let Some(array) = puzzles.as_array_of_tables_mut() {
            let total = array.len();
            for (idx, table) in array.iter_mut().enumerate() {
                let address = table
                    .get("address")
                    .and_then(|a| a.as_str())
                    .unwrap_or("")
                    .to_string();
                let name = table
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                println!("  [{}/{}] Processing {} ({})", idx + 1, total, name, address);

                tokio::time::sleep(RATE_LIMIT_DELAY).await;

                let existing = extract_existing_transactions(table);

                match fetch_transactions(client, &address).await {
                    Ok(txs) => {
                        let new_transactions =
                            categorize_transactions(&address, txs, author_addresses);
                        let merged = merge_transactions(existing, new_transactions);
                        if !merged.is_empty() {
                            table.insert(
                                "transactions",
                                Item::Value(Value::Array(transactions_to_array(&merged))),
                            );
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
    author_addresses: &HashSet<String>,
) -> Result<usize, Box<dyn std::error::Error>> {
    if let Some(puzzle) = doc.get_mut("puzzle") {
        if let Some(table) = puzzle.as_table_mut() {
            let address = table
                .get("address")
                .and_then(|a| a.as_str())
                .unwrap_or("")
                .to_string();

            println!("  Processing gsmg ({})", address);

            let existing = extract_existing_transactions(table);

            tokio::time::sleep(RATE_LIMIT_DELAY).await;

            match fetch_transactions(client, &address).await {
                Ok(txs) => {
                    let new_transactions =
                        categorize_transactions(&address, txs, author_addresses);
                    let merged = merge_transactions(existing, new_transactions);
                    if !merged.is_empty() {
                        table.insert(
                            "transactions",
                            Item::Value(Value::Array(transactions_to_array(&merged))),
                        );
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
    filter_puzzle: Option<i64>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Processing: {}", path.display());

    let content = std::fs::read_to_string(path)?;
    let mut doc: DocumentMut = content.parse()?;

    let author_addresses = extract_author_addresses(&doc);
    if author_addresses.is_empty() {
        println!("  Warning: No author addresses found, skipping");
        return Ok(());
    }

    let count = match collection {
        "b1000" => process_b1000(client, &mut doc, &author_addresses, filter_puzzle).await?,
        "gsmg" => process_gsmg(client, &mut doc, &author_addresses).await?,
        _ => process_collection(client, &mut doc, &author_addresses).await?,
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

    let mut collections: Vec<String> = Vec::new();
    let mut filter_puzzle: Option<i64> = None;
    let mut i = 1;

    while i < args.len() {
        if args[i] == "--puzzle" && i + 1 < args.len() {
            filter_puzzle = args[i + 1].parse().ok();
            i += 2;
        } else {
            collections.push(args[i].clone());
            i += 1;
        }
    }

    if collections.is_empty() {
        collections = vec!["b1000".to_string(), "gsmg".to_string(), "hash_collision".to_string()];
    }

    let client = reqwest::Client::builder()
        .user_agent("boha-scripts/0.1")
        .timeout(Duration::from_secs(30))
        .build()?;

    let data_dir = Path::new("../data");

    for collection in &collections {
        let filename = format!("{}.toml", collection);
        let path = data_dir.join(&filename);

        if path.exists() {
            process_toml_file(&client, &path, collection, filter_puzzle).await?;
        } else {
            eprintln!("File not found: {}", path.display());
        }
    }

    println!("\nDone!");
    Ok(())
}
