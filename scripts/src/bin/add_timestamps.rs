use chrono::{DateTime, TimeZone, Utc};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use toml_edit::{DocumentMut, Item, Value};

#[derive(Debug, Deserialize)]
struct CachedTx {
    status: TxStatus,
    vin: Vec<TxVin>,
    vout: Vec<TxVout>,
}

#[derive(Debug, Deserialize)]
struct TxStatus {
    block_time: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct TxVin {
    prevout: Option<TxPrevout>,
}

#[derive(Debug, Deserialize)]
struct TxPrevout {
    scriptpubkey_address: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TxVout {
    scriptpubkey_address: Option<String>,
}

fn cache_path(collection: &str, address: &str) -> PathBuf {
    Path::new("../data/cache")
        .join(collection)
        .join(format!("{}.json", address))
}

fn load_from_cache(collection: &str, address: &str) -> Option<Vec<CachedTx>> {
    let path = cache_path(collection, address);
    if path.exists() {
        let content = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    } else {
        None
    }
}

fn timestamp_to_datetime(ts: i64) -> String {
    let dt: DateTime<Utc> = Utc.timestamp_opt(ts, 0).unwrap();
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn has_time(date_str: &str) -> bool {
    date_str.len() > 10 && date_str.contains(' ')
}

fn get_timestamp_from_transactions(table: &toml_edit::Table, tx_type: &str) -> Option<String> {
    let transactions = table.get("transactions")?.as_array()?;

    for item in transactions.iter() {
        let inline = item.as_inline_table()?;
        let item_type = inline.get("type")?.as_str()?;

        if item_type == tx_type {
            if let Some(date) = inline.get("date").and_then(|d| d.as_str()) {
                if has_time(date) {
                    return Some(date.to_string());
                }
            }
        }
    }
    None
}

fn get_funding_timestamp_from_cache(collection: &str, address: &str) -> Option<String> {
    let txs = load_from_cache(collection, address)?;

    for tx in &txs {
        let is_funding = tx
            .vout
            .iter()
            .any(|vout| vout.scriptpubkey_address.as_deref() == Some(address));

        if is_funding {
            if let Some(block_time) = tx.status.block_time {
                return Some(timestamp_to_datetime(block_time));
            }
        }
    }
    None
}

fn get_claim_timestamp_from_cache(collection: &str, address: &str) -> Option<String> {
    let txs = load_from_cache(collection, address)?;

    for tx in &txs {
        let is_claim = tx.vin.iter().any(|vin| {
            vin.prevout
                .as_ref()
                .and_then(|p| p.scriptpubkey_address.as_deref())
                == Some(address)
        });

        if is_claim {
            if let Some(block_time) = tx.status.block_time {
                return Some(timestamp_to_datetime(block_time));
            }
        }
    }
    None
}

fn process_puzzles_array(doc: &mut DocumentMut, collection: &str) -> usize {
    let mut count = 0;

    let puzzles = match doc.get_mut("puzzles") {
        Some(p) => p,
        None => return 0,
    };

    let array = match puzzles.as_array_of_tables_mut() {
        Some(a) => a,
        None => return 0,
    };

    for table in array.iter_mut() {
        let address = table
            .get("address")
            .and_then(|a| a.as_str())
            .unwrap_or("")
            .to_string();

        let bits = table.get("bits").and_then(|b| b.as_integer());
        let name = table.get("name").and_then(|n| n.as_str());
        let id = bits
            .map(|b| b.to_string())
            .or_else(|| name.map(|n| n.to_string()))
            .unwrap_or_else(|| address[..8.min(address.len())].to_string());

        if let Some(start_date) = table.get("start_date").and_then(|d| d.as_str()) {
            if !has_time(start_date) {
                let timestamp = get_timestamp_from_transactions(table, "funding")
                    .or_else(|| get_funding_timestamp_from_cache(collection, &address));

                if let Some(ts) = timestamp {
                    println!("  {} start_date: {} -> {}", id, start_date, ts);
                    table.insert("start_date", Item::Value(Value::from(ts)));
                    count += 1;
                } else {
                    eprintln!("  {} start_date: {} - no timestamp found!", id, start_date);
                }
            }
        }

        if let Some(solve_date) = table.get("solve_date").and_then(|d| d.as_str()) {
            if !has_time(solve_date) {
                let timestamp = get_timestamp_from_transactions(table, "claim")
                    .or_else(|| get_timestamp_from_transactions(table, "sweep"))
                    .or_else(|| get_claim_timestamp_from_cache(collection, &address));

                if let Some(ts) = timestamp {
                    println!("  {} solve_date: {} -> {}", id, solve_date, ts);
                    table.insert("solve_date", Item::Value(Value::from(ts)));
                    count += 1;
                } else {
                    eprintln!("  {} solve_date: {} - no timestamp found!", id, solve_date);
                }
            }
        }
    }

    count
}

fn process_single_puzzle(doc: &mut DocumentMut, collection: &str) -> usize {
    let mut count = 0;

    let puzzle = match doc.get_mut("puzzle") {
        Some(p) => p,
        None => return 0,
    };

    let table = match puzzle.as_table_mut() {
        Some(t) => t,
        None => return 0,
    };

    let address = table
        .get("address")
        .and_then(|a| a.as_str())
        .unwrap_or("")
        .to_string();

    if let Some(start_date) = table.get("start_date").and_then(|d| d.as_str()) {
        if !has_time(start_date) {
            let timestamp = get_timestamp_from_transactions(table, "funding")
                .or_else(|| get_funding_timestamp_from_cache(collection, &address));

            if let Some(ts) = timestamp {
                println!("  {} start_date: {} -> {}", collection, start_date, ts);
                table.insert("start_date", Item::Value(Value::from(ts)));
                count += 1;
            } else {
                eprintln!(
                    "  {} start_date: {} - no timestamp found!",
                    collection, start_date
                );
            }
        }
    }

    if let Some(solve_date) = table.get("solve_date").and_then(|d| d.as_str()) {
        if !has_time(solve_date) {
            let timestamp = get_timestamp_from_transactions(table, "claim")
                .or_else(|| get_timestamp_from_transactions(table, "sweep"))
                .or_else(|| get_claim_timestamp_from_cache(collection, &address));

            if let Some(ts) = timestamp {
                println!("  {} solve_date: {} -> {}", collection, solve_date, ts);
                table.insert("solve_date", Item::Value(Value::from(ts)));
                count += 1;
            } else {
                eprintln!(
                    "  {} solve_date: {} - no timestamp found!",
                    collection, solve_date
                );
            }
        }
    }

    count
}

fn process_toml_file(path: &Path, collection: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Processing: {}", path.display());

    let content = std::fs::read_to_string(path)?;
    let mut doc: DocumentMut = content.parse()?;

    let count = if doc.get("puzzles").is_some() {
        process_puzzles_array(&mut doc, collection)
    } else if doc.get("puzzle").is_some() {
        process_single_puzzle(&mut doc, collection)
    } else {
        0
    };

    if count > 0 {
        std::fs::write(path, doc.to_string())?;
        println!("  Updated {} date(s) with timestamps\n", count);
    } else {
        println!("  All dates already have timestamps\n");
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let collections: Vec<&str> = if args.len() > 1 {
        args[1..].iter().map(|s| s.as_str()).collect()
    } else {
        vec!["b1000", "gsmg", "hash_collision"]
    };

    let data_dir = Path::new("../data");

    for collection in collections {
        let path = data_dir.join(format!("{}.toml", collection));
        if path.exists() {
            process_toml_file(&path, collection)?;
        } else {
            eprintln!("File not found: {}", path.display());
        }
    }

    println!("Done!");
    Ok(())
}
