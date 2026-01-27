use boha_scripts::types::{strip_jsonc_comments, Collection, Puzzle};
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

fn process_puzzles(puzzles: Vec<Puzzle>) -> usize {
    let mut count = 0;

    for mut puzzle in puzzles {
        // Skip if hash160 already exists
        if puzzle.address.hash160.is_some() {
            continue;
        }

        let address_str = &puzzle.address.value;
        // Only process P2PKH addresses (start with '1')
        if address_str.starts_with('1') {
            match address_to_h160(address_str) {
                Ok(h160) => {
                    puzzle.address.hash160 = Some(h160);
                    count += 1;
                }
                Err(e) => {
                    eprintln!("  Error processing {}: {}", address_str, e);
                }
            }
        }
    }

    count
}

fn process_single_puzzle(mut puzzle: Puzzle) -> usize {
    // Skip if hash160 already exists
    if puzzle.address.hash160.is_some() {
        return 0;
    }

    let address_str = &puzzle.address.value;
    if address_str.starts_with('1') {
        match address_to_h160(address_str) {
            Ok(h160) => {
                puzzle.address.hash160 = Some(h160);
                return 1;
            }
            Err(e) => {
                eprintln!("  Error processing {}: {}", address_str, e);
            }
        }
    }
    0
}

fn process_jsonc_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Processing: {}", path.display());

    let content = std::fs::read_to_string(path)?;
    let json_content = strip_jsonc_comments(&content);
    let mut collection: Collection = serde_json::from_str(&json_content)?;

    let count = if let Some(puzzles) = collection.puzzles.take() {
        process_puzzles(puzzles)
    } else if let Some(puzzle) = collection.puzzle.take() {
        process_single_puzzle(puzzle)
    } else {
        println!("  No puzzles found");
        return Ok(());
    };

    if count > 0 {
        std::fs::write(path, serde_json::to_string_pretty(&collection)?)?;
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
