use ripemd::Ripemd160;
use sha2::{Digest, Sha256};
use std::path::Path;

fn redeem_script_to_script_hash(
    redeem_script_hex: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let script_bytes = hex::decode(redeem_script_hex)?;
    let sha256_hash = Sha256::digest(&script_bytes);
    let hash160 = Ripemd160::digest(&sha256_hash);
    Ok(hex::encode(hash160))
}

fn update_puzzles_with_script_hash(doc: &mut serde_json::Value) -> usize {
    let mut count = 0;

    if let Some(puzzles) = doc
        .get_mut("puzzles")
        .and_then(|p: &mut serde_json::Value| p.as_array_mut())
    {
        for puzzle in puzzles.iter_mut() {
            // Skip if script_hash already exists
            if puzzle
                .get("address")
                .and_then(|a: &serde_json::Value| a.get("redeem_script"))
                .and_then(|r: &serde_json::Value| r.get("hash"))
                .is_some()
            {
                continue;
            }

            if let Some(redeem_script) = puzzle
                .get("address")
                .and_then(|a: &serde_json::Value| a.get("redeem_script"))
                .and_then(|r: &serde_json::Value| r.get("script"))
                .and_then(|s: &serde_json::Value| s.as_str())
            {
                match redeem_script_to_script_hash(redeem_script) {
                    Ok(script_hash) => {
                        if let Some(redeem_script_obj) = puzzle
                            .get_mut("address")
                            .and_then(|a: &mut serde_json::Value| a.get_mut("redeem_script"))
                        {
                            redeem_script_obj["hash"] = serde_json::json!(script_hash);
                            count += 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("  Error processing redeem_script {}: {}", redeem_script, e);
                    }
                }
            }
        }
    }

    count
}

fn process_jsonc_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Processing: {}", path.display());

    let content = std::fs::read_to_string(path)?;
    let mut doc: serde_json::Value = serde_json::from_str(&content)?;

    let count = update_puzzles_with_script_hash(&mut doc);

    if count > 0 {
        std::fs::write(path, serde_json::to_string_pretty(&doc)?)?;
        println!("  Updated {} entries with script_hash", count);
    } else {
        println!("  No updates needed");
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = Path::new("../data");

    let path = data_dir.join("hash_collision.jsonc");
    if path.exists() {
        process_jsonc_file(&path)?;
    } else {
        eprintln!("File not found: {}", path.display());
    }

    println!("\nDone!");
    Ok(())
}
