use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;
use toml_edit::{Array, DocumentMut, InlineTable, Item, Value};

const RATE_LIMIT_DELAY: Duration = Duration::from_secs(3);
const RETRY_DELAY: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EsploraTx {
    txid: String,
    status: EsploraStatus,
    vin: Vec<EsploraVin>,
    vout: Vec<EsploraVout>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EsploraStatus {
    block_time: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EsploraVin {
    prevout: Option<EsploraPrevout>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EsploraPrevout {
    scriptpubkey_address: Option<String>,
    value: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

fn timestamp_to_date(timestamp: i64) -> String {
    let dt = DateTime::<Utc>::from_timestamp(timestamp, 0).unwrap_or_default();
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn sats_to_btc(sats: u64) -> f64 {
    sats as f64 / 100_000_000.0
}

fn cache_path(collection: &str, address: &str) -> PathBuf {
    Path::new("../data/cache")
        .join(collection)
        .join(format!("{}.json", address))
}

fn load_from_cache(collection: &str, address: &str) -> Option<Vec<EsploraTx>> {
    let path = cache_path(collection, address);
    if path.exists() {
        let content = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    } else {
        None
    }
}

fn save_to_cache(
    collection: &str,
    address: &str,
    txs: &[EsploraTx],
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

fn categorize_transactions(
    puzzle_address: &str,
    txs: Vec<EsploraTx>,
    author_addresses: &HashSet<String>,
) -> Vec<Transaction> {
    let mut result = Vec::new();

    let mut sorted_txs = txs;
    sorted_txs.sort_by_key(|tx| tx.status.block_time.unwrap_or(i64::MAX));

    let mut has_funding = false;
    let mut has_claim = false;

    for tx in &sorted_txs {
        if has_claim {
            break;
        }
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

        let amount_from_puzzle: u64 = tx
            .vin
            .iter()
            .filter_map(|i| i.prevout.as_ref())
            .filter(|p| p.scriptpubkey_address.as_deref() == Some(puzzle_address))
            .map(|p| p.value)
            .sum();

        const DUST_THRESHOLD: u64 = 10_000;

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

        if puzzle_is_sender && amount_from_puzzle > 0 && amount_from_puzzle <= DUST_THRESHOLD {
            result.push(Transaction {
                tx_type: "pubkey_reveal".to_string(),
                txid: tx.txid.clone(),
                date: Some(timestamp_to_date(block_time)),
                amount: Some(sats_to_btc(amount_from_puzzle)),
            });
        } else if puzzle_is_sender && amount_to_solver > 0 && amount_to_author == 0 {
            result.push(Transaction {
                tx_type: "claim".to_string(),
                txid: tx.txid.clone(),
                date: Some(timestamp_to_date(block_time)),
                amount: Some(sats_to_btc(amount_to_solver)),
            });
            has_claim = true;
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

    let claim_idx = merged.iter().position(|t| t.tx_type == "claim");
    if let Some(idx) = claim_idx {
        merged.truncate(idx + 1);
    }

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

async fn fetch_and_cache_b1000(
    client: &reqwest::Client,
    doc: &DocumentMut,
    filter_puzzle: Option<i64>,
    force: bool,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut count = 0;

    if let Some(puzzles) = doc.get("puzzles") {
        if let Some(array) = puzzles.as_array_of_tables() {
            for (idx, table) in array.iter().enumerate() {
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

                let cache_exists = cache_path("b1000", &address).exists();
                if cache_exists && !force {
                    println!(
                        "  [{}/256] Skipping puzzle {} ({}) - cached",
                        idx + 1,
                        bits,
                        address
                    );
                    continue;
                }

                println!(
                    "  [{}/256] Fetching puzzle {} ({})",
                    idx + 1,
                    bits,
                    address
                );

                match fetch_transactions(client, &address).await {
                    Ok(txs) => {
                        save_to_cache("b1000", &address, &txs)?;
                        count += 1;
                    }
                    Err(e) => {
                        eprintln!("    Error fetching: {}", e);
                    }
                }
            }
        }
    }

    Ok(count)
}

fn process_cached_b1000(
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

                let txs = match load_from_cache("b1000", &address) {
                    Some(t) => t,
                    None => {
                        println!(
                            "  [{}/256] No cache for puzzle {} ({})",
                            idx + 1,
                            bits,
                            address
                        );
                        continue;
                    }
                };

                println!(
                    "  [{}/256] Processing puzzle {} ({})",
                    idx + 1,
                    bits,
                    address
                );

                let existing = extract_existing_transactions(table);
                let new_transactions = categorize_transactions(&address, txs, author_addresses);
                let merged = merge_transactions(existing, new_transactions);

                if !merged.is_empty() {
                    table.insert(
                        "transactions",
                        Item::Value(Value::Array(transactions_to_array(&merged))),
                    );
                    count += 1;
                }
            }
        }
    }

    Ok(count)
}

async fn fetch_and_cache_gsmg(
    client: &reqwest::Client,
    doc: &DocumentMut,
    force: bool,
) -> Result<usize, Box<dyn std::error::Error>> {
    if let Some(puzzle) = doc.get("puzzle") {
        if let Some(table) = puzzle.as_table() {
            let address = table
                .get("address")
                .and_then(|a| a.as_str())
                .unwrap_or("")
                .to_string();

            let cache_exists = cache_path("gsmg", &address).exists();
            if cache_exists && !force {
                println!("  Skipping gsmg ({}) - cached", address);
                return Ok(0);
            }

            println!("  Fetching gsmg ({})", address);

            match fetch_transactions(client, &address).await {
                Ok(txs) => {
                    save_to_cache("gsmg", &address, &txs)?;
                    return Ok(1);
                }
                Err(e) => {
                    eprintln!("    Error fetching: {}", e);
                }
            }
        }
    }

    Ok(0)
}

fn process_cached_gsmg(
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

            let txs = match load_from_cache("gsmg", &address) {
                Some(t) => t,
                None => {
                    println!("  No cache for gsmg ({})", address);
                    return Ok(0);
                }
            };

            println!("  Processing gsmg ({})", address);

            let existing = extract_existing_transactions(table);
            let new_transactions = categorize_transactions(&address, txs, author_addresses);
            let merged = merge_transactions(existing, new_transactions);

            if !merged.is_empty() {
                table.insert(
                    "transactions",
                    Item::Value(Value::Array(transactions_to_array(&merged))),
                );
                return Ok(1);
            }
        }
    }

    Ok(0)
}

async fn fetch_and_cache_collection(
    client: &reqwest::Client,
    doc: &DocumentMut,
    collection: &str,
    force: bool,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut count = 0;

    if let Some(puzzles) = doc.get("puzzles") {
        if let Some(array) = puzzles.as_array_of_tables() {
            let total = array.len();
            for (idx, table) in array.iter().enumerate() {
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

                let cache_exists = cache_path(collection, &address).exists();
                if cache_exists && !force {
                    println!(
                        "  [{}/{}] Skipping {} ({}) - cached",
                        idx + 1,
                        total,
                        name,
                        address
                    );
                    continue;
                }

                println!("  [{}/{}] Fetching {} ({})", idx + 1, total, name, address);

                match fetch_transactions(client, &address).await {
                    Ok(txs) => {
                        save_to_cache(collection, &address, &txs)?;
                        count += 1;
                    }
                    Err(e) => {
                        eprintln!("    Error fetching: {}", e);
                    }
                }
            }
        }
    }

    Ok(count)
}

fn process_cached_collection(
    doc: &mut DocumentMut,
    author_addresses: &HashSet<String>,
    collection: &str,
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

                let txs = match load_from_cache(collection, &address) {
                    Some(t) => t,
                    None => {
                        println!(
                            "  [{}/{}] No cache for {} ({})",
                            idx + 1,
                            total,
                            name,
                            address
                        );
                        continue;
                    }
                };

                println!(
                    "  [{}/{}] Processing {} ({})",
                    idx + 1,
                    total,
                    name,
                    address
                );

                let existing = extract_existing_transactions(table);
                let new_transactions = categorize_transactions(&address, txs, author_addresses);
                let merged = merge_transactions(existing, new_transactions);

                if !merged.is_empty() {
                    table.insert(
                        "transactions",
                        Item::Value(Value::Array(transactions_to_array(&merged))),
                    );
                    count += 1;
                }
            }
        }
    }

    Ok(count)
}

enum Mode {
    Fetch,
    Process,
    Both,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let mut collections: Vec<String> = Vec::new();
    let mut filter_puzzle: Option<i64> = None;
    let mut mode = Mode::Both;
    let mut force = false;
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--puzzle" if i + 1 < args.len() => {
                filter_puzzle = args[i + 1].parse().ok();
                i += 2;
            }
            "--fetch" => {
                mode = Mode::Fetch;
                i += 1;
            }
            "--process" => {
                mode = Mode::Process;
                i += 1;
            }
            "--force" => {
                force = true;
                i += 1;
            }
            _ => {
                collections.push(args[i].clone());
                i += 1;
            }
        }
    }

    if collections.is_empty() {
        collections = vec![
            "b1000".to_string(),
            "gsmg".to_string(),
            "hash_collision".to_string(),
        ];
    }

    let client = reqwest::Client::builder()
        .user_agent("boha-scripts/0.1")
        .timeout(Duration::from_secs(30))
        .build()?;

    let data_dir = Path::new("../data");

    for collection in &collections {
        let filename = format!("{}.toml", collection);
        let path = data_dir.join(&filename);

        if !path.exists() {
            eprintln!("File not found: {}", path.display());
            continue;
        }

        let content = std::fs::read_to_string(&path)?;
        let mut doc: DocumentMut = content.parse()?;
        let author_addresses = extract_author_addresses(&doc);

        if author_addresses.is_empty() {
            println!("Warning: No author addresses found in {}, skipping", collection);
            continue;
        }

        match mode {
            Mode::Fetch | Mode::Both => {
                println!("Fetching: {}", collection);
                let fetched = match collection.as_str() {
                    "b1000" => {
                        fetch_and_cache_b1000(&client, &doc, filter_puzzle, force).await?
                    }
                    "gsmg" => fetch_and_cache_gsmg(&client, &doc, force).await?,
                    _ => fetch_and_cache_collection(&client, &doc, collection, force).await?,
                };
                println!("  Fetched {} addresses\n", fetched);
            }
            Mode::Process => {}
        }

        match mode {
            Mode::Process | Mode::Both => {
                println!("Processing: {}", collection);
                let processed = match collection.as_str() {
                    "b1000" => {
                        process_cached_b1000(&mut doc, &author_addresses, filter_puzzle)?
                    }
                    "gsmg" => process_cached_gsmg(&mut doc, &author_addresses)?,
                    _ => process_cached_collection(&mut doc, &author_addresses, collection)?,
                };

                if processed > 0 {
                    std::fs::write(&path, doc.to_string())?;
                    println!("  Updated {} puzzles\n", processed);
                } else {
                    println!("  No updates needed\n");
                }
            }
            Mode::Fetch => {}
        }
    }

    println!("Done!");
    Ok(())
}
