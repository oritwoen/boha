use boha_scripts::types::{strip_jsonc_comments, Collection, Puzzle};
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

fn update_puzzles_with_script_hash(puzzles: &mut [Puzzle]) -> usize {
    let mut count = 0;

    for puzzle in puzzles.iter_mut() {
        // Skip if script_hash already exists
        if puzzle
            .address
            .redeem_script
            .as_ref()
            .and_then(|rs| rs.hash.as_ref())
            .is_some()
        {
            continue;
        }

        if let Some(redeem_script) = &puzzle.address.redeem_script {
            let script = &redeem_script.script;
            match redeem_script_to_script_hash(script) {
                Ok(script_hash) => {
                    if let Some(rs) = puzzle.address.redeem_script.as_mut() {
                        rs.hash = Some(script_hash);
                        count += 1;
                    }
                }
                Err(e) => {
                    eprintln!("  Error processing redeem_script {}: {}", script, e);
                }
            }
        }
    }

    count
}

fn process_jsonc_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Processing: {}", path.display());

    let content = std::fs::read_to_string(path)?;
    let json_content = strip_jsonc_comments(&content);
    let mut collection: Collection = serde_json::from_str(&json_content)?;

    let count = if let Some(ref mut puzzles) = collection.puzzles {
        update_puzzles_with_script_hash(puzzles)
    } else {
        0
    };

    if count > 0 {
        std::fs::write(path, serde_json::to_string_pretty(&collection)?)?;
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
