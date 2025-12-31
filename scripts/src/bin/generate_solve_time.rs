use chrono::{DateTime, TimeZone, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
use toml_edit::{DocumentMut, Item, Value};

#[derive(Debug, Deserialize)]
struct TxStatus {
    confirmed: bool,
    block_time: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct TxVout {
    scriptpubkey_address: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TxVin {
    prevout: Option<TxVout>,
}

#[derive(Debug, Deserialize)]
struct Transaction {
    txid: String,
    status: TxStatus,
    vin: Vec<TxVin>,
    vout: Vec<TxVout>,
}

struct AddressTimestamps {
    funding_time: Option<u64>,
    claim_time: Option<u64>,
}

async fn fetch_address_transactions(
    client: &Client,
    address: &str,
) -> Result<Vec<Transaction>, Box<dyn std::error::Error>> {
    let url = format!("https://mempool.space/api/address/{}/txs", address);
    let response = client.get(&url).send().await?;
    
    if !response.status().is_success() {
        return Err(format!("API error for {}: {}", address, response.status()).into());
    }
    
    let txs: Vec<Transaction> = response.json().await?;
    Ok(txs)
}

fn analyze_transactions(address: &str, txs: &[Transaction]) -> AddressTimestamps {
    let mut funding_time: Option<u64> = None;
    let mut claim_time: Option<u64> = None;
    
    for tx in txs {
        let block_time = match tx.status.block_time {
            Some(t) if tx.status.confirmed => t,
            _ => continue,
        };
        
        let is_funding = tx.vout.iter().any(|vout| {
            vout.scriptpubkey_address.as_deref() == Some(address)
        });
        
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
                Some(existing) => existing.max(block_time),
                None => block_time,
            });
        }
    }
    
    AddressTimestamps {
        funding_time,
        claim_time,
    }
}

fn timestamp_to_datetime_str(ts: u64) -> String {
    let dt: DateTime<Utc> = Utc.timestamp_opt(ts as i64, 0).unwrap();
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

async fn collect_puzzle_addresses(doc: &DocumentMut) -> Vec<(String, String)> {
    let mut addresses = Vec::new();
    
    if let Some(puzzles) = doc.get("puzzles") {
        if let Some(array) = puzzles.as_array_of_tables() {
            for table in array.iter() {
                let address = table.get("address").and_then(|v| v.as_str());
                let status = table.get("status").and_then(|v| v.as_str());
                let bits = table.get("bits").and_then(|v| v.as_integer());
                let name = table.get("name").and_then(|v| v.as_str());
                
                if let Some(addr) = address {
                    if matches!(status, Some("solved") | Some("claimed")) {
                        let id = bits
                            .map(|b| b.to_string())
                            .or_else(|| name.map(|n| n.to_string()))
                            .unwrap_or_else(|| addr[..8].to_string());
                        addresses.push((id, addr.to_string()));
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
                    addresses.push(("gsmg".to_string(), addr.to_string()));
                }
            }
        }
    }
    
    addresses
}

async fn fetch_all_timestamps(
    addresses: &[(String, String)],
) -> Result<HashMap<String, AddressTimestamps>, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;
    
    let mut results = HashMap::new();
    
    for (id, address) in addresses {
        print!("  Fetching {}... ", id);
        
        match fetch_address_transactions(&client, address).await {
            Ok(txs) => {
                let timestamps = analyze_transactions(address, &txs);
                println!(
                    "funding={:?}, claim={:?}",
                    timestamps.funding_time, timestamps.claim_time
                );
                results.insert(address.clone(), timestamps);
            }
            Err(e) => {
                println!("ERROR: {}", e);
            }
        }
        
        tokio::time::sleep(Duration::from_millis(1500)).await;
    }
    
    Ok(results)
}

fn update_puzzles_with_timestamps(
    doc: &mut DocumentMut,
    timestamps: &HashMap<String, AddressTimestamps>,
) -> usize {
    let mut count = 0;
    
    if let Some(puzzles) = doc.get_mut("puzzles") {
        if let Some(array) = puzzles.as_array_of_tables_mut() {
            for table in array.iter_mut() {
                let address = table.get("address").and_then(|v| v.as_str());
                
                if let Some(addr) = address {
                    if let Some(ts) = timestamps.get(addr) {
                        if let (Some(funding), Some(claim)) = (ts.funding_time, ts.claim_time) {
                            let start_str = timestamp_to_datetime_str(funding);
                            let solve_str = timestamp_to_datetime_str(claim);
                            let solve_time = claim.saturating_sub(funding);
                            
                            table.insert("start_date", Item::Value(Value::from(start_str)));
                            table.insert("solve_date", Item::Value(Value::from(solve_str)));
                            table.insert("solve_time", Item::Value(Value::from(solve_time as i64)));
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
                        table.insert("solve_time", Item::Value(Value::from(solve_time as i64)));
                        return 1;
                    }
                }
            }
        }
    }
    0
}

async fn process_toml_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Processing: {}", path.display());
    
    let content = std::fs::read_to_string(path)?;
    let mut doc: DocumentMut = content.parse()?;
    
    let addresses = collect_puzzle_addresses(&doc).await;
    
    if addresses.is_empty() {
        println!("  No solved/claimed puzzles found");
        return Ok(());
    }
    
    println!("  Found {} solved/claimed puzzles", addresses.len());
    
    let timestamps = fetch_all_timestamps(&addresses).await?;
    
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = Path::new("../data");
    
    let files = ["b1000.toml", "hash_collision.toml", "gsmg.toml"];
    
    for file in &files {
        let path = data_dir.join(file);
        if path.exists() {
            process_toml_file(&path).await?;
            println!();
        } else {
            eprintln!("File not found: {}", path.display());
        }
    }
    
    println!("Done!");
    Ok(())
}
