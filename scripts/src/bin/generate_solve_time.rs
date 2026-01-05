mod utils {
    include!("../utils/mod.rs");
}

use chrono::{DateTime, TimeZone, Utc};
use std::collections::HashMap;
use std::path::Path;
use toml_edit::{DocumentMut, Item, Value};
use utils::mempool::{load_from_cache, MempoolTx};

struct AddressTimestamps {
    funding_time: Option<i64>,
    claim_time: Option<i64>,
}

fn analyze_transactions(address: &str, txs: &[MempoolTx]) -> AddressTimestamps {
    let mut funding_time: Option<i64> = None;
    let mut claim_time: Option<i64> = None;

    for tx in txs {
        let block_time = match tx.status.block_time {
            Some(t) => t,
            None => continue,
        };

        let is_funding = tx
            .vout
            .iter()
            .any(|vout| vout.scriptpubkey_address.as_deref() == Some(address));

        let is_claim = tx.vin.iter().any(|vin| {
            vin.prevout
                .as_ref()
                .and_then(|p| p.scriptpubkey_address.as_deref())
                == Some(address)
        });

        if is_funding {
            funding_time = Some(match funding_time {
                Some(existing) => existing.min(block_time),
                None => block_time,
            });
        }

        if is_claim {
            claim_time = Some(match claim_time {
                Some(existing) => existing.min(block_time),
                None => block_time,
            });
        }
    }

    AddressTimestamps {
        funding_time,
        claim_time,
    }
}

fn timestamp_to_datetime_str(ts: i64) -> String {
    let dt: DateTime<Utc> = Utc.timestamp_opt(ts, 0).unwrap();
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn extract_address(table: &toml_edit::Table) -> Option<String> {
    if let Some(addr) = table.get("address").and_then(|v| v.as_str()) {
        return Some(addr.to_string());
    }

    if let Some(addr_table) = table.get("address").and_then(|v| v.as_inline_table()) {
        if let Some(value) = addr_table.get("value").and_then(|v| v.as_str()) {
            return Some(value.to_string());
        }
    }

    None
}

fn collect_puzzle_addresses(doc: &DocumentMut, collection: &str) -> Vec<(String, String, String)> {
    let mut addresses = Vec::new();

    if let Some(puzzles) = doc.get("puzzles") {
        if let Some(array) = puzzles.as_array_of_tables() {
            for table in array.iter() {
                let address = extract_address(table);
                let status = table.get("status").and_then(|v| v.as_str());
                let bits = table.get("bits").and_then(|v| v.as_integer());
                let name = table.get("name").and_then(|v| v.as_str());

                if let Some(addr) = address {
                    if matches!(status, Some("solved") | Some("claimed")) {
                        let id = bits
                            .map(|b| b.to_string())
                            .or_else(|| name.map(|n| n.to_string()))
                            .unwrap_or_else(|| addr[..8].to_string());
                        addresses.push((id, addr.to_string(), collection.to_string()));
                    }
                }
            }
        }
    }

    if let Some(puzzle) = doc.get("puzzle") {
        if let Some(table) = puzzle.as_table() {
            let address = table.get("address").and_then(|v| v.as_str());
            let status = table.get("status").and_then(|v| v.as_str());

            if let Some(addr) = address {
                if matches!(status, Some("solved") | Some("claimed")) {
                    addresses.push(("gsmg".to_string(), addr.to_string(), collection.to_string()));
                }
            }
        }
    }

    addresses
}

fn load_all_timestamps(
    addresses: &[(String, String, String)],
) -> HashMap<String, AddressTimestamps> {
    let mut results = HashMap::new();

    for (id, address, collection) in addresses {
        print!("  Loading {}... ", id);

        match load_from_cache(collection, address) {
            Some(txs) => {
                let timestamps = analyze_transactions(address, &txs);
                println!(
                    "funding={:?}, claim={:?}",
                    timestamps.funding_time, timestamps.claim_time
                );
                results.insert(address.clone(), timestamps);
            }
            None => {
                println!("NO CACHE");
            }
        }
    }

    results
}

fn update_puzzles_with_timestamps(
    doc: &mut DocumentMut,
    timestamps: &HashMap<String, AddressTimestamps>,
) -> usize {
    let mut count = 0;

    if let Some(puzzles) = doc.get_mut("puzzles") {
        if let Some(array) = puzzles.as_array_of_tables_mut() {
            for table in array.iter_mut() {
                let address = extract_address(table);

                if let Some(addr) = address {
                    if let Some(ts) = timestamps.get(&addr) {
                        if let (Some(funding), Some(claim)) = (ts.funding_time, ts.claim_time) {
                            let start_str = timestamp_to_datetime_str(funding);
                            let solve_str = timestamp_to_datetime_str(claim);
                            let solve_time = claim.saturating_sub(funding);

                            table.insert("start_date", Item::Value(Value::from(start_str)));
                            table.insert("solve_date", Item::Value(Value::from(solve_str)));
                            table.insert("solve_time", Item::Value(Value::from(solve_time)));
                            count += 1;
                        }
                    }
                }
            }
        }
    }

    count
}

fn update_single_puzzle_with_timestamps(
    doc: &mut DocumentMut,
    timestamps: &HashMap<String, AddressTimestamps>,
) -> usize {
    if let Some(puzzle) = doc.get_mut("puzzle") {
        if let Some(table) = puzzle.as_table_mut() {
            let address = table.get("address").and_then(|v| v.as_str());

            if let Some(addr) = address {
                if let Some(ts) = timestamps.get(addr) {
                    if let (Some(funding), Some(claim)) = (ts.funding_time, ts.claim_time) {
                        let start_str = timestamp_to_datetime_str(funding);
                        let solve_str = timestamp_to_datetime_str(claim);
                        let solve_time = claim.saturating_sub(funding);

                        table.insert("start_date", Item::Value(Value::from(start_str)));
                        table.insert("solve_date", Item::Value(Value::from(solve_str)));
                        table.insert("solve_time", Item::Value(Value::from(solve_time)));
                        return 1;
                    }
                }
            }
        }
    }
    0
}

fn process_toml_file(path: &Path, collection: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Processing: {}", path.display());

    let content = std::fs::read_to_string(path)?;
    let mut doc: DocumentMut = content.parse()?;

    let addresses = collect_puzzle_addresses(&doc, collection);

    if addresses.is_empty() {
        println!("  No solved/claimed puzzles found");
        return Ok(());
    }

    println!("  Found {} solved/claimed puzzles", addresses.len());

    let timestamps = load_all_timestamps(&addresses);

    let count = if doc.get("puzzles").is_some() {
        update_puzzles_with_timestamps(&mut doc, &timestamps)
    } else if doc.get("puzzle").is_some() {
        update_single_puzzle_with_timestamps(&mut doc, &timestamps)
    } else {
        0
    };

    if count > 0 {
        std::fs::write(path, doc.to_string())?;
        println!("  Updated {} entries with timestamps", count);
    } else {
        println!("  No updates made");
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = Path::new("../data");

    let files = [
        ("b1000.toml", "b1000"),
        ("hash_collision.toml", "hash_collision"),
        ("gsmg.toml", "gsmg"),
        ("ballet.toml", "ballet"),
        ("zden.toml", "zden"),
    ];

    for (file, collection) in &files {
        let path = data_dir.join(file);
        if path.exists() {
            process_toml_file(&path, collection)?;
            println!();
        } else {
            eprintln!("File not found: {}", path.display());
        }
    }

    println!("Done!");
    Ok(())
}
