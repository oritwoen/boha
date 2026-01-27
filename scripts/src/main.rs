use chrono::{DateTime, TimeZone, Utc};
use serde::Deserialize;
use serde_json::Value;
use std::path::Path;
use std::time::Duration;

pub mod types;
use types::{Collection, strip_jsonc_comments};

#[derive(Debug, Deserialize)]
struct TxStatus {
    block_time: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct Transaction {
    status: TxStatus,
}

async fn fetch_first_tx_date(
    client: &reqwest::Client,
    address: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let url = format!("https://mempool.space/api/address/{}/txs", address);

    let mut retries = 3;
    loop {
        let response = client.get(&url).send().await?;

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            if retries > 0 {
                retries -= 1;
                tokio::time::sleep(Duration::from_secs(2)).await;
                continue;
            }
            return Err("Rate limited after retries".into());
        }

        if !response.status().is_success() {
            return Err(format!("API returned {}", response.status()).into());
        }

        let txs: Vec<Transaction> = response.json().await?;

        if txs.is_empty() {
            return Ok(None);
        }

        let oldest_time = txs
            .iter()
            .filter_map(|tx| tx.status.block_time)
            .min();

        return match oldest_time {
            Some(timestamp) => {
                let dt: DateTime<Utc> = Utc.timestamp_opt(timestamp, 0).unwrap();
                Ok(Some(dt.format("%Y-%m-%d").to_string()))
            }
            None => Ok(None),
        };
    }
}

fn update_jsonc_with_dates(
    doc: &mut Value,
    dates: &[(usize, String)],
) {
    if let Some(puzzles) = doc.get_mut("puzzles") {
        if let Some(array) = puzzles.as_array_mut() {
            for (idx, date) in dates {
                if let Some(puzzle) = array.get_mut(*idx) {
                    if let Some(obj) = puzzle.as_object_mut() {
                        obj.insert("start_date".to_string(), Value::String(date.clone()));
                    }
                }
            }
        }
    }
}

async fn process_jsonc_file(
    client: &reqwest::Client,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Processing: {}", path.display());

    let content = std::fs::read_to_string(path)?;
    let mut doc: Value = jsonc_parser::parse_to_serde_value(&content, &Default::default())?
        .ok_or_else(|| "Failed to parse JSONC")?;

    let addresses: Vec<(usize, String)> = {
        let stripped = strip_jsonc_comments(&content);
        let collection: Collection = serde_json::from_str(&stripped)?;
        
        let puzzles = collection.puzzles
            .ok_or("No puzzles array found")?;

        puzzles
            .iter()
            .enumerate()
            .filter_map(|(idx, puzzle)| {
                if puzzle.start_date.is_some() {
                    return None;
                }
                
                Some((idx, puzzle.address.value.clone()))
            })
            .collect()
    };

    println!("  Found {} puzzles without start_date", addresses.len());

    let mut dates_to_update: Vec<(usize, String)> = Vec::new();

    for (i, (idx, address)) in addresses.iter().enumerate() {
        print!("  [{}/{}] {} ... ", i + 1, addresses.len(), address);

        match fetch_first_tx_date(client, address).await {
            Ok(Some(date)) => {
                println!("{}", date);
                dates_to_update.push((*idx, date));
            }
            Ok(None) => {
                println!("no transactions");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    if !dates_to_update.is_empty() {
        update_jsonc_with_dates(&mut doc, &dates_to_update);
        std::fs::write(path, serde_json::to_string_pretty(&doc)?)?;
        println!("  Updated {} entries", dates_to_update.len());
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .user_agent("boha-fetch-start-dates/0.1")
        .timeout(Duration::from_secs(30))
        .build()?;

    let data_dir = Path::new("../data");

    let files = ["b1000.jsonc", "hash_collision.jsonc"];

    for file in &files {
        let path = data_dir.join(file);
        if path.exists() {
            process_jsonc_file(&client, &path).await?;
        } else {
            eprintln!("File not found: {}", path.display());
        }
    }

    println!("\nDone!");
    Ok(())
}
