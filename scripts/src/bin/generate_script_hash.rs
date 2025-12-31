use ripemd::Ripemd160;
use sha2::{Digest, Sha256};
use std::path::Path;
use toml_edit::{DocumentMut, Item, Value};

fn redeem_script_to_script_hash(redeem_script_hex: &str) -> Result<String, Box<dyn std::error::Error>> {
    let script_bytes = hex::decode(redeem_script_hex)?;
    let sha256_hash = Sha256::digest(&script_bytes);
    let hash160 = Ripemd160::digest(&sha256_hash);
    Ok(hex::encode(hash160))
}

fn update_puzzles_with_script_hash(doc: &mut DocumentMut) -> usize {
    let mut count = 0;

    if let Some(puzzles) = doc.get_mut("puzzles") {
        if let Some(array) = puzzles.as_array_of_tables_mut() {
            for table in array.iter_mut() {
                // Skip if script_hash already exists
                if table.get("script_hash").is_some() {
                    continue;
                }

                if let Some(redeem_script) = table.get("redeem_script").and_then(|r| r.as_str()) {
                    match redeem_script_to_script_hash(redeem_script) {
                        Ok(script_hash) => {
                            table.insert("script_hash", Item::Value(Value::from(script_hash)));
                            count += 1;
                        }
                        Err(e) => {
                            eprintln!("  Error processing redeem_script {}: {}", redeem_script, e);
                        }
                    }
                }
            }
        }
    }

    count
}

fn process_toml_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Processing: {}", path.display());

    let content = std::fs::read_to_string(path)?;
    let mut doc: DocumentMut = content.parse()?;

    let count = update_puzzles_with_script_hash(&mut doc);

    if count > 0 {
        std::fs::write(path, doc.to_string())?;
        println!("  Updated {} entries with script_hash", count);
    } else {
        println!("  No updates needed");
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = Path::new("../data");

    let path = data_dir.join("hash_collision.toml");
    if path.exists() {
        process_toml_file(&path)?;
    } else {
        eprintln!("File not found: {}", path.display());
    }

    println!("\nDone!");
    Ok(())
}
