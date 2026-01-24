use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::Path;

fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

fn hex_to_wif(hex_key: &str, compressed: bool) -> Option<String> {
    const MAINNET_VERSION: u8 = 0x80;
    const COMPRESSION_FLAG: u8 = 0x01;

    let key_bytes = hex::decode(hex_key).ok()?;
    if key_bytes.len() != 32 {
        return None;
    }

    let mut data = vec![MAINNET_VERSION];
    data.extend_from_slice(&key_bytes);
    if compressed {
        data.push(COMPRESSION_FLAG);
    }

    let checksum = &sha256(&sha256(&data))[..4];
    data.extend_from_slice(checksum);

    Some(bs58::encode(data).into_string())
}

fn needs_wif(key_item: &Value) -> Option<String> {
    let hex = key_item.get("hex")?.as_str()?.to_string();

    let has_decrypted = key_item
        .get("wif")
        .and_then(|w| w.get("decrypted"))
        .is_some();

    if has_decrypted {
        None
    } else {
        Some(hex)
    }
}

fn add_wif_to_key(key_table: &mut Value, wif: &str) {
    if let Some(wif_obj) = key_table.get_mut("wif") {
        if wif_obj.is_object() {
            wif_obj["decrypted"] = Value::String(wif.to_string());
            return;
        }
    }

    // Create wif object if it doesn't exist
    if !key_table.is_object() {
        return;
    }
    key_table["wif"] = json!({ "decrypted": wif });
}

fn update_puzzles_array(doc: &mut Value) -> usize {
    let mut count = 0;

    if let Some(puzzles) = doc.get_mut("puzzles") {
        if let Some(array) = puzzles.as_array_mut() {
            for puzzle in array.iter_mut() {
                if let Some(key_item) = puzzle.get_mut("key") {
                    if let Some(hex) = needs_wif(key_item) {
                        if let Some(wif) = hex_to_wif(&hex, true) {
                            add_wif_to_key(key_item, &wif);
                            count += 1;
                        }
                    }
                }
            }
        }
    }

    count
}

fn update_single_puzzle(doc: &mut Value) -> usize {
    if let Some(puzzle) = doc.get_mut("puzzle") {
        if let Some(key_item) = puzzle.get_mut("key") {
            if let Some(hex) = needs_wif(key_item) {
                if let Some(wif) = hex_to_wif(&hex, true) {
                    add_wif_to_key(key_item, &wif);
                    return 1;
                }
            }
        }
    }
    0
}

fn process_jsonc_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Processing: {}", path.display());

    let content = std::fs::read_to_string(path)?;
    let mut doc: Value = jsonc_parser::parse_to_serde_value(&content, &Default::default())?
        .ok_or_else(|| "Failed to parse JSONC")?;

    let count = if doc.get("puzzles").is_some() {
        update_puzzles_array(&mut doc)
    } else if doc.get("puzzle").is_some() {
        update_single_puzzle(&mut doc)
    } else {
        println!("  No puzzles found");
        return Ok(());
    };

    if count > 0 {
        std::fs::write(path, serde_json::to_string_pretty(&doc)?)?;
        println!("  Updated {} entries with wif.decrypted", count);
    } else {
        println!("  No updates needed");
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = Path::new("../data");

    let files = [
        "b1000.jsonc",
        "ballet.jsonc",
        "zden.jsonc",
        "bitimage.jsonc",
        "gsmg.jsonc",
        "bitaps.jsonc",
        "hash_collision.jsonc",
    ];

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
