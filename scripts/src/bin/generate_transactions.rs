mod utils {
    include!("../utils/mod.rs");
}

use serde_json::Value;
use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;
use utils::{
    cache_path, dcrdata, etherscan, extract_author_addresses, extract_existing_transactions,
    mempool, merge_transactions, transactions_to_array,
};

async fn fetch_and_cache_btc(
    client: &reqwest::Client,
    address: &str,
    collection: &str,
    name: &str,
    force: bool,
) -> Result<bool, Box<dyn std::error::Error>> {
    let cache_exists = cache_path(collection, address).exists();
    if cache_exists && !force {
        println!("    Skipping {} ({}) - cached", name, address);
        return Ok(false);
    }

    println!("    Fetching {} ({})", name, address);

    match mempool::fetch_transactions(client, address).await {
        Ok(txs) => {
            mempool::save_to_cache(collection, address, &txs)?;
            Ok(true)
        }
        Err(e) => {
            eprintln!("    Error fetching: {}", e);
            Ok(false)
        }
    }
}

async fn fetch_and_cache_eth(
    client: &reqwest::Client,
    address: &str,
    collection: &str,
    name: &str,
    api_key: &str,
    force: bool,
) -> Result<bool, Box<dyn std::error::Error>> {
    let cache_exists = cache_path(collection, address).exists();
    if cache_exists && !force {
        println!("    Skipping {} ({}) - cached", name, address);
        return Ok(false);
    }

    println!("    Fetching {} ({})", name, address);

    match etherscan::fetch_transactions(client, address, api_key).await {
        Ok(txs) => {
            etherscan::save_to_cache(collection, address, &txs)?;
            Ok(true)
        }
        Err(e) => {
            eprintln!("    Error fetching: {}", e);
            Ok(false)
        }
    }
}

fn strip_jsonc_comments(content: &str) -> String {
    let mut result = String::new();
    let mut in_string = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut chars = content.chars().peekable();

    while let Some(c) = chars.next() {
        if in_line_comment {
            if c == '\n' {
                in_line_comment = false;
                result.push(c);
            }
            continue;
        }

        if in_block_comment {
            if c == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_block_comment = false;
            }
            continue;
        }

        if in_string {
            result.push(c);
            if c == '\\' {
                if let Some(next) = chars.next() {
                    result.push(next);
                }
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }

        if c == '"' {
            in_string = true;
            result.push(c);
            continue;
        }

        if c == '/' {
            match chars.peek() {
                Some('/') => {
                    chars.next();
                    in_line_comment = true;
                    continue;
                }
                Some('*') => {
                    chars.next();
                    in_block_comment = true;
                    continue;
                }
                _ => {}
            }
        }

        result.push(c);
    }

    result
}

fn process_cached_btc(
    puzzle: &mut Value,
    address: &str,
    collection: &str,
    author_addresses: &HashSet<String>,
) -> bool {
    let txs = match mempool::load_from_cache(collection, address) {
        Some(t) => t,
        None => return false,
    };

    let status = puzzle
        .get("status")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let existing = extract_existing_transactions(puzzle);
    let new_transactions = mempool::categorize_transactions(address, txs, author_addresses, &status);
    let merged = merge_transactions(existing, new_transactions);

    if !merged.is_empty() {
        puzzle["transactions"] = transactions_to_array(&merged);
        return true;
    }

    false
}

fn process_cached_eth(
    puzzle: &mut Value,
    address: &str,
    collection: &str,
    author_addresses: &HashSet<String>,
) -> bool {
    let txs = match etherscan::load_from_cache(collection, address) {
        Some(t) => t,
        None => return false,
    };

    let status = puzzle
        .get("status")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let existing = extract_existing_transactions(puzzle);
    let new_transactions =
        etherscan::categorize_transactions(address, txs, author_addresses, &status);
    let merged = merge_transactions(existing, new_transactions);

    if !merged.is_empty() {
        puzzle["transactions"] = transactions_to_array(&merged);
        return true;
    }

    false
}

async fn fetch_and_cache_b1000(
    client: &reqwest::Client,
    doc: &Value,
    filter_puzzle: Option<i64>,
    force: bool,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut count = 0;

    if let Some(puzzles) = doc.get("puzzles") {
        if let Some(array) = puzzles.as_array() {
            for (idx, puzzle) in array.iter().enumerate() {
                let bits = puzzle.get("key").and_then(|k| k.get("bits")).and_then(|b| b.as_i64()).unwrap_or(0);

                if let Some(filter) = filter_puzzle {
                    if bits != filter {
                        continue;
                    }
                }

                let address = puzzle
                    .get("address")
                    .and_then(|a| a.get("value"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                print!("  [{}/256]", idx + 1);
                if fetch_and_cache_btc(client, &address, "b1000", &bits.to_string(), force).await? {
                    count += 1;
                }
            }
        }
    }

    Ok(count)
}

fn process_cached_b1000(
    doc: &mut Value,
    author_addresses: &HashSet<String>,
    filter_puzzle: Option<i64>,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut count = 0;

    if let Some(puzzles) = doc.get_mut("puzzles") {
        if let Some(array) = puzzles.as_array_mut() {
            for (idx, puzzle) in array.iter_mut().enumerate() {
                let bits = puzzle.get("key").and_then(|k| k.get("bits")).and_then(|b| b.as_i64()).unwrap_or(0);

                if let Some(filter) = filter_puzzle {
                    if bits != filter {
                        continue;
                    }
                }

                let address = puzzle
                    .get("address")
                    .and_then(|a| a.get("value"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                println!("  [{}/256] Processing puzzle {} ({})", idx + 1, bits, address);

                if process_cached_btc(puzzle, &address, "b1000", author_addresses) {
                    count += 1;
                }
            }
        }
    }

    Ok(count)
}

async fn fetch_and_cache_gsmg(
    client: &reqwest::Client,
    doc: &Value,
    force: bool,
) -> Result<usize, Box<dyn std::error::Error>> {
    if let Some(puzzle) = doc.get("puzzle") {
        let address = puzzle
            .get("address")
            .and_then(|a| a.get("value"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if fetch_and_cache_btc(client, &address, "gsmg", "gsmg", force).await? {
            return Ok(1);
        }
    }

    Ok(0)
}

fn process_cached_gsmg(
    doc: &mut Value,
    author_addresses: &HashSet<String>,
) -> Result<usize, Box<dyn std::error::Error>> {
    if let Some(puzzle) = doc.get_mut("puzzle") {
        let address = puzzle
            .get("address")
            .and_then(|a| a.get("value"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        println!("  Processing gsmg ({})", address);

        if process_cached_btc(puzzle, &address, "gsmg", author_addresses) {
            return Ok(1);
        }
    }

    Ok(0)
}

async fn fetch_and_cache_dcr(
    client: &reqwest::Client,
    address: &str,
    collection: &str,
    name: &str,
    force: bool,
) -> Result<bool, Box<dyn std::error::Error>> {
    let cache_exists = cache_path(collection, address).exists();
    if cache_exists && !force {
        println!("    Skipping {} ({}) - cached", name, address);
        return Ok(false);
    }

    println!("    Fetching {} ({})", name, address);

    match dcrdata::fetch_transactions(client, address).await {
        Ok(txs) => {
            dcrdata::save_to_cache(collection, address, &txs)?;
            Ok(true)
        }
        Err(e) => {
            eprintln!("    Error fetching: {}", e);
            Ok(false)
        }
    }
}

fn process_cached_dcr(
    puzzle: &mut Value,
    address: &str,
    collection: &str,
    author_addresses: &HashSet<String>,
) -> bool {
    let txs = match dcrdata::load_from_cache(collection, address) {
        Some(t) => t,
        None => return false,
    };

    let status = puzzle
        .get("status")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let existing = extract_existing_transactions(puzzle);
    let new_transactions = dcrdata::categorize_transactions(address, txs, author_addresses, &status);
    let merged = merge_transactions(existing, new_transactions);

    if !merged.is_empty() {
        puzzle["transactions"] = transactions_to_array(&merged);
        return true;
    }

    false
}

async fn fetch_and_cache_collection(
    client: &reqwest::Client,
    doc: &Value,
    collection: &str,
    etherscan_api_key: Option<&str>,
    force: bool,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut count = 0;

    if let Some(puzzles) = doc.get("puzzles") {
        if let Some(array) = puzzles.as_array() {
            let total = array.len();
            for (idx, puzzle) in array.iter().enumerate() {
                let address = puzzle
                    .get("address")
                    .and_then(|a| a.get("value"))
                    .and_then(|v| v.as_str())
                    .or_else(|| puzzle.get("address").and_then(|a| a.as_str()))
                    .unwrap_or("")
                    .to_string();

                let name = puzzle
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                let chain = puzzle
                    .get("chain")
                    .and_then(|c| c.as_str())
                    .unwrap_or("bitcoin");

                print!("  [{}/{}]", idx + 1, total);

                let fetched = match chain {
                    "bitcoin" | "litecoin" => {
                        fetch_and_cache_btc(client, &address, collection, &name, force).await?
                    }
                    "ethereum" => {
                        if let Some(api_key) = etherscan_api_key {
                            fetch_and_cache_eth(client, &address, collection, &name, api_key, force)
                                .await?
                        } else {
                            println!("    Skipping {} - no ETHERSCAN_API_KEY", name);
                            false
                        }
                    }
                    "decred" => {
                        fetch_and_cache_dcr(client, &address, collection, &name, force).await?
                    }
                    _ => {
                        println!("    Skipping {} - unsupported chain: {}", name, chain);
                        false
                    }
                };

                if fetched {
                    count += 1;
                }
            }
        }
    }

    Ok(count)
}

fn process_cached_collection(
    doc: &mut Value,
    author_addresses: &HashSet<String>,
    collection: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut count = 0;

    if let Some(puzzles) = doc.get_mut("puzzles") {
        if let Some(array) = puzzles.as_array_mut() {
            let total = array.len();
            for (idx, puzzle) in array.iter_mut().enumerate() {
                let address = puzzle
                    .get("address")
                    .and_then(|a| a.get("value"))
                    .and_then(|v| v.as_str())
                    .or_else(|| puzzle.get("address").and_then(|a| a.as_str()))
                    .unwrap_or("")
                    .to_string();

                let name = puzzle
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                let chain = puzzle
                    .get("chain")
                    .and_then(|c| c.as_str())
                    .unwrap_or("bitcoin");

                println!(
                    "  [{}/{}] Processing {} ({})",
                    idx + 1,
                    total,
                    name,
                    address
                );

                let processed = match chain {
                    "bitcoin" | "litecoin" => {
                        process_cached_btc(puzzle, &address, collection, author_addresses)
                    }
                    "ethereum" => {
                        process_cached_eth(puzzle, &address, collection, author_addresses)
                    }
                    "decred" => {
                        process_cached_dcr(puzzle, &address, collection, author_addresses)
                    }
                    _ => {
                        println!("    Unsupported chain: {}", chain);
                        false
                    }
                };

                if processed {
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
    dotenvy::dotenv().ok();

    let etherscan_api_key = std::env::var("ETHERSCAN_API_KEY").ok();

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
            "zden".to_string(),
        ];
    }

    let client = reqwest::Client::builder()
        .user_agent("boha-scripts/0.1")
        .timeout(Duration::from_secs(30))
        .build()?;

    let data_dir = Path::new("../data");

    for collection in &collections {
        let filename = format!("{}.jsonc", collection);
        let path = data_dir.join(&filename);

        if !path.exists() {
            eprintln!("File not found: {}", path.display());
            continue;
        }

        let content = std::fs::read_to_string(&path)?;
        let json_content = strip_jsonc_comments(&content);
        let mut doc: Value = serde_json::from_str(&json_content)?;
        let author_addresses = extract_author_addresses(&doc);

        if author_addresses.is_empty() {
            println!(
                "Warning: No author addresses found in {}, skipping",
                collection
            );
            continue;
        }

        match mode {
            Mode::Fetch | Mode::Both => {
                println!("Fetching: {}", collection);
                let fetched = match collection.as_str() {
                    "b1000" => fetch_and_cache_b1000(&client, &doc, filter_puzzle, force).await?,
                    "gsmg" => fetch_and_cache_gsmg(&client, &doc, force).await?,
                    _ => {
                        fetch_and_cache_collection(
                            &client,
                            &doc,
                            collection,
                            etherscan_api_key.as_deref(),
                            force,
                        )
                        .await?
                    }
                };
                println!("  Fetched {} addresses\n", fetched);
            }
            Mode::Process => {}
        }

        match mode {
            Mode::Process | Mode::Both => {
                println!("Processing: {}", collection);
                let processed = match collection.as_str() {
                    "b1000" => process_cached_b1000(&mut doc, &author_addresses, filter_puzzle)?,
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
