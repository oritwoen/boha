use sha2::{Digest, Sha256};
use std::path::Path;
use toml_edit::{DocumentMut, Formatted, InlineTable, Item, Table, Value};

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

fn needs_wif(key_item: &Item) -> Option<String> {
    let hex = match key_item {
        Item::Value(Value::InlineTable(t)) => t.get("hex")?.as_str()?.to_string(),
        Item::Table(t) => t.get("hex")?.as_str()?.to_string(),
        _ => return None,
    };

    let has_decrypted = match key_item {
        Item::Value(Value::InlineTable(t)) => t
            .get("wif")
            .and_then(|w| w.as_inline_table())
            .and_then(|w| w.get("decrypted"))
            .is_some(),
        Item::Table(t) => t
            .get("wif")
            .and_then(|w| w.as_table())
            .and_then(|w| w.get("decrypted"))
            .is_some(),
        _ => false,
    };

    if has_decrypted {
        None
    } else {
        Some(hex)
    }
}

fn add_wif_to_inline_table(key_table: &mut InlineTable, wif: &str) {
    if let Some(wif_item) = key_table.get_mut("wif") {
        if let Some(wif_table) = wif_item.as_inline_table_mut() {
            wif_table.insert("decrypted", Value::String(Formatted::new(wif.to_string())));
            return;
        }
    }

    let mut wif_table = InlineTable::new();
    wif_table.insert("decrypted", Value::String(Formatted::new(wif.to_string())));
    key_table.insert("wif", Value::InlineTable(wif_table));
}

fn add_wif_to_table(key_table: &mut Table, wif: &str) {
    if let Some(wif_item) = key_table.get_mut("wif") {
        if let Some(wif_section) = wif_item.as_table_mut() {
            wif_section.insert(
                "decrypted",
                Item::Value(Value::String(Formatted::new(wif.to_string()))),
            );
            return;
        }
    }

    let mut wif_section = Table::new();
    wif_section.insert(
        "decrypted",
        Item::Value(Value::String(Formatted::new(wif.to_string()))),
    );
    key_table.insert("wif", Item::Table(wif_section));
}

fn update_puzzles_array(doc: &mut DocumentMut) -> usize {
    let mut count = 0;

    if let Some(puzzles) = doc.get_mut("puzzles") {
        if let Some(array) = puzzles.as_array_of_tables_mut() {
            for puzzle in array.iter_mut() {
                if let Some(key_item) = puzzle.get_mut("key") {
                    if let Some(hex) = needs_wif(key_item) {
                        if let Some(wif) = hex_to_wif(&hex, true) {
                            match key_item {
                                Item::Value(Value::InlineTable(t)) => {
                                    add_wif_to_inline_table(t, &wif);
                                    count += 1;
                                }
                                Item::Table(t) => {
                                    add_wif_to_table(t, &wif);
                                    count += 1;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    count
}

fn update_single_puzzle(doc: &mut DocumentMut) -> usize {
    if let Some(puzzle) = doc.get_mut("puzzle") {
        if let Some(puzzle_table) = puzzle.as_table_mut() {
            if let Some(key_item) = puzzle_table.get_mut("key") {
                if let Some(hex) = needs_wif(key_item) {
                    if let Some(wif) = hex_to_wif(&hex, true) {
                        match key_item {
                            Item::Value(Value::InlineTable(t)) => {
                                add_wif_to_inline_table(t, &wif);
                                return 1;
                            }
                            Item::Table(t) => {
                                add_wif_to_table(t, &wif);
                                return 1;
                            }
                            _ => {}
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
        update_puzzles_array(&mut doc)
    } else if doc.get("puzzle").is_some() {
        update_single_puzzle(&mut doc)
    } else {
        println!("  No puzzles found");
        return Ok(());
    };

    if count > 0 {
        std::fs::write(path, doc.to_string())?;
        println!("  Updated {} entries with wif.decrypted", count);
    } else {
        println!("  No updates needed");
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = Path::new("../data");

    let files = [
        "b1000.toml",
        "ballet.toml",
        "zden.toml",
        "bitimage.toml",
        "gsmg.toml",
        "bitaps.toml",
        "hash_collision.toml",
    ];

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
