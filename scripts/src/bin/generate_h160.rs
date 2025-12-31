use std::path::Path;
use toml_edit::{DocumentMut, Item, Value};

fn address_to_h160(address: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Decode base58 (addresses in TOML are already validated)
    let decoded = bs58::decode(address).into_vec()?;
    // Bitcoin address: 1 byte version + 20 bytes H160 + 4 bytes checksum = 25 bytes
    if decoded.len() != 25 {
        return Err(format!("Invalid address length: {} (expected 25)", decoded.len()).into());
    }
    // Skip version byte (first byte), take 20 bytes of H160 (skip last 4 checksum bytes)
    let h160 = &decoded[1..21];
    Ok(hex::encode(h160))
}

fn update_puzzles_with_h160(doc: &mut DocumentMut) -> usize {
    let mut count = 0;

    if let Some(puzzles) = doc.get_mut("puzzles") {
        if let Some(array) = puzzles.as_array_of_tables_mut() {
            for table in array.iter_mut() {
                // Skip if h160 already exists
                if table.get("h160").is_some() {
                    continue;
                }

                if let Some(address) = table.get("address").and_then(|a| a.as_str()) {
                    // Only process P2PKH addresses (start with '1')
                    if address.starts_with('1') {
                        match address_to_h160(address) {
                            Ok(h160) => {
                                table.insert("h160", Item::Value(Value::from(h160)));
                                count += 1;
                            }
                            Err(e) => {
                                eprintln!("  Error processing {}: {}", address, e);
                            }
                        }
                    }
                }
            }
        }
    }

    count
}

fn update_single_puzzle_with_h160(doc: &mut DocumentMut) -> usize {
    if let Some(puzzle) = doc.get_mut("puzzle") {
        if let Some(table) = puzzle.as_table_mut() {
            // Skip if h160 already exists
            if table.get("h160").is_some() {
                return 0;
            }

            if let Some(address) = table.get("address").and_then(|a| a.as_str()) {
                if address.starts_with('1') {
                    match address_to_h160(address) {
                        Ok(h160) => {
                            table.insert("h160", Item::Value(Value::from(h160.as_str())));
                            return 1;
                        }
                        Err(e) => {
                            eprintln!("  Error processing {}: {}", address, e);
                        }
                    }
                }
            }
        }
    }
    0
}

fn process_toml_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Processing: {}", path.display());

    let content = std::fs::read_to_string(path)?;
    let mut doc: DocumentMut = content.parse()?;

    let count = if doc.get("puzzles").is_some() {
        update_puzzles_with_h160(&mut doc)
    } else if doc.get("puzzle").is_some() {
        update_single_puzzle_with_h160(&mut doc)
    } else {
        println!("  No puzzles found");
        return Ok(());
    };

    if count > 0 {
        std::fs::write(path, doc.to_string())?;
        println!("  Updated {} entries with h160", count);
    } else {
        println!("  No updates needed");
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = Path::new("../data");

    let files = ["b1000.toml", "gsmg.toml"];

    for file in &files {
        let path = data_dir.join(file);
        if path.exists() {
            process_toml_file(&path)?;
        } else {
            eprintln!("File not found: {}", path.display());
        }
    }

    println!("\nDone!");
    Ok(())
}
