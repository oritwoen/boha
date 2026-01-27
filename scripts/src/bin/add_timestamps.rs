use chrono::{DateTime, TimeZone, Utc};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use boha_scripts::types::{Address, Puzzle};

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
    value: u64,
}

#[derive(Debug, Deserialize)]
struct TxVout {
    scriptpubkey_address: Option<String>,
}

const DUST_THRESHOLD: u64 = 10_000;

struct AddressTimestamps {
    funding_time: Option<i64>,
    claim_time: Option<i64>,
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

fn analyze_transactions(address: &str, txs: &[CachedTx]) -> AddressTimestamps {
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

        let amount_from_puzzle: u64 = tx
            .vin
            .iter()
            .filter_map(|vin| vin.prevout.as_ref())
            .filter(|p| p.scriptpubkey_address.as_deref() == Some(address))
            .map(|p| p.value)
            .sum();

        let is_claim = amount_from_puzzle > DUST_THRESHOLD;

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

fn timestamp_to_datetime(ts: i64) -> String {
    let dt: DateTime<Utc> = Utc.timestamp_opt(ts, 0).unwrap();
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn has_time(date_str: &str) -> bool {
    date_str.len() > 10 && date_str.contains(' ')
}

fn parse_datetime_to_timestamp(s: &str) -> Option<i64> {
    let parts: Vec<&str> = s.split(&['-', ' ', ':'][..]).collect();
    if parts.len() != 6 {
        return None;
    }
    let year: i64 = parts[0].parse().ok()?;
    let month: i64 = parts[1].parse().ok()?;
    let day: i64 = parts[2].parse().ok()?;
    let hour: i64 = parts[3].parse().ok()?;
    let min: i64 = parts[4].parse().ok()?;
    let sec: i64 = parts[5].parse().ok()?;

    fn days_in_month(year: i64, month: i64) -> i64 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                    29
                } else {
                    28
                }
            }
            _ => 0,
        }
    }

    let mut days: i64 = 0;
    for y in 1970..year {
        days += if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
            366
        } else {
            365
        };
    }
    for m in 1..month {
        days += days_in_month(year, m);
    }
    days += day - 1;

    Some(days * 86400 + hour * 3600 + min * 60 + sec)
}

fn calculate_solve_time(start_date: &str, solve_date: &str) -> Option<u64> {
    let start_ts = parse_datetime_to_timestamp(start_date)?;
    let solve_ts = parse_datetime_to_timestamp(solve_date)?;
    Some((solve_ts - start_ts) as u64)
}

fn get_timestamp_from_transactions(obj: &Value, tx_type: &str) -> Option<String> {
    let transactions = obj.get("transactions")?.as_array()?;

    for item in transactions.iter() {
        let item_type = item.get("type")?.as_str()?;

        if item_type == tx_type {
            if let Some(date) = item.get("date").and_then(|d| d.as_str()) {
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

fn process_puzzles_array(doc: &mut Value, collection: &str) -> usize {
    let mut count = 0;

    let puzzles = match doc.get_mut("puzzles") {
        Some(p) => p,
        None => return 0,
    };

    let array = match puzzles.as_array_mut() {
        Some(a) => a,
        None => return 0,
    };

    for table in array.iter_mut() {
        // Deserialize to typed Puzzle to access address.value
        let puzzle_json = serde_json::to_string(table).unwrap_or_default();
        let puzzle: Puzzle = serde_json::from_str(&puzzle_json).unwrap_or_else(|_| Puzzle {
            name: None,
            chain: None,
            address: Address {
                value: String::new(),
                kind: None,
                hash160: None,
                witness_program: None,
                redeem_script: None,
            },
            status: String::new(),
            pubkey: None,
            key: None,
            transactions: None,
            prize: None,
            start_date: None,
            solve_date: None,
        });

        let address = puzzle.address.value.clone();

        let bits = table.get("bits").and_then(|b| b.as_i64());
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
                    table["start_date"] = Value::String(ts);
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
                    table["solve_date"] = Value::String(ts);
                    count += 1;
                } else {
                    eprintln!("  {} solve_date: {} - no timestamp found!", id, solve_date);
                }
            }
        }

        if let (Some(start_date), Some(solve_date)) = (
            table.get("start_date").and_then(|d| d.as_str()),
            table.get("solve_date").and_then(|d| d.as_str()),
        ) {
            if has_time(start_date) && has_time(solve_date) {
                if let Some(calculated) = calculate_solve_time(start_date, solve_date) {
                    let current = table
                        .get("solve_time")
                        .and_then(|v| v.as_i64())
                        .map(|v| v as u64);

                    if current != Some(calculated) {
                        println!("  {} solve_time: {:?} -> {}", id, current, calculated);
                        table["solve_time"] = Value::Number(calculated.into());
                        count += 1;
                    }
                }
            }
        }
    }

    count
}

fn process_single_puzzle(doc: &mut Value, collection: &str) -> usize {
    let mut count = 0;

    let puzzle = match doc.get_mut("puzzle") {
        Some(p) => p,
        None => return 0,
    };

    let puzzle_json = serde_json::to_string(puzzle).unwrap_or_default();
    let typed_puzzle: Puzzle = serde_json::from_str(&puzzle_json).unwrap_or_else(|_| Puzzle {
        name: None,
        chain: None,
        address: Address {
            value: String::new(),
            kind: None,
            hash160: None,
            witness_program: None,
            redeem_script: None,
        },
        status: String::new(),
        pubkey: None,
        key: None,
        transactions: None,
        prize: None,
        start_date: None,
        solve_date: None,
    });

    let address = typed_puzzle.address.value.clone();

    if let Some(start_date) = puzzle.get("start_date").and_then(|d| d.as_str()) {
        if !has_time(start_date) {
            let timestamp = get_timestamp_from_transactions(puzzle, "funding")
                .or_else(|| get_funding_timestamp_from_cache(collection, &address));

            if let Some(ts) = timestamp {
                println!("  {} start_date: {} -> {}", collection, start_date, ts);
                puzzle["start_date"] = Value::String(ts);
                count += 1;
            } else {
                eprintln!(
                    "  {} start_date: {} - no timestamp found!",
                    collection, start_date
                );
            }
        }
    }

    if let Some(solve_date) = puzzle.get("solve_date").and_then(|d| d.as_str()) {
        if !has_time(solve_date) {
            let timestamp = get_timestamp_from_transactions(puzzle, "claim")
                .or_else(|| get_timestamp_from_transactions(puzzle, "sweep"))
                .or_else(|| get_claim_timestamp_from_cache(collection, &address));

            if let Some(ts) = timestamp {
                println!("  {} solve_date: {} -> {}", collection, solve_date, ts);
                puzzle["solve_date"] = Value::String(ts);
                count += 1;
            } else {
                eprintln!(
                    "  {} solve_date: {} - no timestamp found!",
                    collection, solve_date
                );
            }
        }
    }

    if let (Some(start_date), Some(solve_date)) = (
        puzzle.get("start_date").and_then(|d| d.as_str()),
        puzzle.get("solve_date").and_then(|d| d.as_str()),
    ) {
        if has_time(start_date) && has_time(solve_date) {
            if let Some(calculated) = calculate_solve_time(start_date, solve_date) {
                let current = puzzle
                    .get("solve_time")
                    .and_then(|v| v.as_i64())
                    .map(|v| v as u64);

                if current != Some(calculated) {
                    println!(
                        "  {} solve_time: {:?} -> {}",
                        collection, current, calculated
                    );
                    puzzle["solve_time"] = Value::Number(calculated.into());
                    count += 1;
                }
            }
        }
    }

    count
}

fn load_all_timestamps_from_cache(
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

fn collect_solved_puzzle_addresses(doc: &Value, collection: &str) -> Vec<(String, String, String)> {
    let mut addresses = Vec::new();

    if let Some(puzzles) = doc.get("puzzles").and_then(|p| p.as_array()) {
        for table in puzzles.iter() {
            let puzzle_json = serde_json::to_string(table).unwrap_or_default();
            if let Ok(puzzle) = serde_json::from_str::<Puzzle>(&puzzle_json) {
                if matches!(puzzle.status.as_str(), "solved" | "claimed") {
                    let bits = table.get("bits").and_then(|b| b.as_i64());
                    let name = table.get("name").and_then(|n| n.as_str());
                    let address = puzzle.address.value.clone();
                    let id = bits
                        .map(|b| b.to_string())
                        .or_else(|| name.map(|n| n.to_string()))
                        .unwrap_or_else(|| address[..8.min(address.len())].to_string());
                    addresses.push((id, address, collection.to_string()));
                }
            }
        }
    }

    if let Some(puzzle_val) = doc.get("puzzle") {
        let puzzle_json = serde_json::to_string(puzzle_val).unwrap_or_default();
        if let Ok(puzzle) = serde_json::from_str::<Puzzle>(&puzzle_json) {
            if matches!(puzzle.status.as_str(), "solved" | "claimed") {
                addresses.push((
                    "puzzle".to_string(),
                    puzzle.address.value.clone(),
                    collection.to_string(),
                ));
            }
        }
    }

    addresses
}

fn update_puzzles_from_cache(
    doc: &mut Value,
    timestamps: &HashMap<String, AddressTimestamps>,
) -> usize {
    let mut count = 0;

    if let Some(puzzles) = doc.get_mut("puzzles").and_then(|p| p.as_array_mut()) {
        for table in puzzles.iter_mut() {
            let puzzle_json = serde_json::to_string(&*table).unwrap_or_default();
            if let Ok(puzzle) = serde_json::from_str::<Puzzle>(&puzzle_json) {
                if let Some(ts) = timestamps.get(&puzzle.address.value) {
                    if let (Some(funding), Some(claim)) = (ts.funding_time, ts.claim_time) {
                        let start_str = timestamp_to_datetime(funding);
                        let solve_str = timestamp_to_datetime(claim);
                        let solve_time = claim.saturating_sub(funding);

                        table["start_date"] = Value::String(start_str);
                        table["solve_date"] = Value::String(solve_str);
                        table["solve_time"] = Value::Number(solve_time.into());
                        count += 1;
                    }
                }
            }
        }
    }

    count
}

fn update_single_puzzle_from_cache(
    doc: &mut Value,
    timestamps: &HashMap<String, AddressTimestamps>,
) -> usize {
    if let Some(puzzle) = doc.get_mut("puzzle") {
        let puzzle_json = serde_json::to_string(&*puzzle).unwrap_or_default();
        if let Ok(typed_puzzle) = serde_json::from_str::<Puzzle>(&puzzle_json) {
            if let Some(ts) = timestamps.get(&typed_puzzle.address.value) {
                if let (Some(funding), Some(claim)) = (ts.funding_time, ts.claim_time) {
                    let start_str = timestamp_to_datetime(funding);
                    let solve_str = timestamp_to_datetime(claim);
                    let solve_time = claim.saturating_sub(funding);

                    puzzle["start_date"] = Value::String(start_str);
                    puzzle["solve_date"] = Value::String(solve_str);
                    puzzle["solve_time"] = Value::Number(solve_time.into());
                    return 1;
                }
            }
        }
    }
    0
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let recalculate = args.contains(&"--recalculate".to_string());

    let collections: Vec<&str> = args
        .iter()
        .skip(1)
        .filter(|s| !s.starts_with("--"))
        .map(|s| s.as_str())
        .collect::<Vec<_>>();

    let collections = if collections.is_empty() {
        vec!["b1000", "gsmg", "hash_collision"]
    } else {
        collections
    };

    let data_dir = Path::new("../data");

    for collection in collections {
        let path = data_dir.join(format!("{}.jsonc", collection));
        if !path.exists() {
            eprintln!("File not found: {}", path.display());
            continue;
        }

        println!("Processing: {}", path.display());

        let content = std::fs::read_to_string(&path)?;
        let mut doc: Value = jsonc_parser::parse_to_serde_value(&content, &Default::default())?
            .ok_or_else(|| "Failed to parse JSONC")?;

        let count = if recalculate {
            // Mode: recalculate from cache (like generate_solve_time)
            println!("  Mode: RECALCULATE from cache");
            let addresses = collect_solved_puzzle_addresses(&doc, collection);

            if addresses.is_empty() {
                println!("  No solved/claimed puzzles found");
                0
            } else {
                println!("  Found {} solved/claimed puzzles", addresses.len());
                let timestamps = load_all_timestamps_from_cache(&addresses);

                if doc.get("puzzles").is_some() {
                    update_puzzles_from_cache(&mut doc, &timestamps)
                } else if doc.get("puzzle").is_some() {
                    update_single_puzzle_from_cache(&mut doc, &timestamps)
                } else {
                    0
                }
            }
        } else {
            // Mode: incremental update (default, like original add_timestamps)
            println!("  Mode: INCREMENTAL (only missing timestamps)");
            if doc.get("puzzles").is_some() {
                process_puzzles_array(&mut doc, collection)
            } else if doc.get("puzzle").is_some() {
                process_single_puzzle(&mut doc, collection)
            } else {
                0
            }
        };

        if count > 0 {
            std::fs::write(&path, serde_json::to_string_pretty(&doc)?)?;
            println!("  Updated {} entries\n", count);
        } else {
            println!("  No updates made\n");
        }
    }

    println!("Done!");
    Ok(())
}
