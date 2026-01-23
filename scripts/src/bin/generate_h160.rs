use serde_json::Value;
use std::path::Path;

fn address_to_h160(address: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Decode base58 (addresses in JSONC are already validated)
    let decoded = bs58::decode(address).into_vec()?;
    // Bitcoin address: 1 byte version + 20 bytes H160 + 4 bytes checksum = 25 bytes
    if decoded.len() != 25 {
        return Err(format!("Invalid address length: {} (expected 25)", decoded.len()).into());
    }
    // Skip version byte (first byte), take 20 bytes of H160 (skip last 4 checksum bytes)
    let h160 = &decoded[1..21];
    Ok(hex::encode(h160))
}

fn update_puzzles_with_h160(value: &mut Value) -> usize {
    let mut count = 0;

    if let Some(puzzles) = value.get_mut("puzzles").and_then(|v| v.as_array_mut()) {
        for puzzle in puzzles.iter_mut() {
            if let Some(address_obj) = puzzle.get_mut("address").and_then(|v| v.as_object_mut()) {
                // Skip if hash160 already exists
                if address_obj.contains_key("hash160") {
                    continue;
                }

                if let Some(address_str) = address_obj.get("value").and_then(|v| v.as_str()) {
                    // Only process P2PKH addresses (start with '1')
                    if address_str.starts_with('1') {
                        match address_to_h160(address_str) {
                            Ok(h160) => {
                                address_obj.insert("hash160".to_string(), Value::String(h160));
                                count += 1;
                            }
                            Err(e) => {
                                eprintln!("  Error processing {}: {}", address_str, e);
                            }
                        }
                    }
                }
            }
        }
    }

    count
}

fn update_single_puzzle_with_h160(value: &mut Value) -> usize {
    if let Some(puzzle) = value.get_mut("puzzle").and_then(|v| v.as_object_mut()) {
        if let Some(address_obj) = puzzle.get_mut("address").and_then(|v| v.as_object_mut()) {
            // Skip if hash160 already exists
            if address_obj.contains_key("hash160") {
                return 0;
            }

            if let Some(address_str) = address_obj.get("value").and_then(|v| v.as_str()) {
                if address_str.starts_with('1') {
                    match address_to_h160(address_str) {
                        Ok(h160) => {
                            address_obj.insert("hash160".to_string(), Value::String(h160));
                            return 1;
                        }
                        Err(e) => {
                            eprintln!("  Error processing {}: {}", address_str, e);
                        }
                    }
                }
            }
        }
    }
    0
}

fn process_jsonc_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Processing: {}", path.display());

    let content = std::fs::read_to_string(path)?;
    let mut value: Value = serde_json::from_str(&content)?;

    let count = if value.get("puzzles").is_some() {
        update_puzzles_with_h160(&mut value)
    } else if value.get("puzzle").is_some() {
        update_single_puzzle_with_h160(&mut value)
    } else {
        println!("  No puzzles found");
        return Ok(());
    };

    if count > 0 {
        std::fs::write(path, serde_json::to_string_pretty(&value)?)?;
        println!("  Updated {} entries with hash160", count);
    } else {
        println!("  No updates needed");
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = Path::new("../data");

    let files = ["b1000.jsonc", "gsmg.jsonc"];

    for file in &files {
        let path = data_dir.join(file);
        if path.exists() {
            process_jsonc_file(&path)?;
        } else {
            eprintln!("File not found: {}", path.display());
        }
    }

    println!("\nDone!");
    Ok(())
}
