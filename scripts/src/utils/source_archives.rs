use std::error::Error;
use toml_edit::{Array, DocumentMut, Item, Value};

/// Path to a puzzle within a collection's TOML file
#[derive(Debug, Clone)]
pub struct PuzzlePath {
    pub collection: String,
    pub puzzle_name: String,
    pub array_index: usize,
}

/// Extract X/Twitter status URLs from a TOML document
///
/// Returns a list of (PuzzlePath, url) tuples for all found X/Twitter status URLs.
/// Ignores profile URLs (without /status/ in path).
///
/// Checks three locations:
/// - `[metadata] source_url`
/// - `[puzzles.assets] source_url`
/// - `[puzzles.key.seed.entropy.source] url`
pub fn extract_twitter_urls(
    doc: &DocumentMut,
    collection: &str,
) -> Result<Vec<(PuzzlePath, String)>, Box<dyn Error>> {
    let mut results = Vec::new();

    // Check [metadata] source_url
    if let Some(metadata) = doc.get("metadata") {
        if let Some(table) = metadata.as_table() {
            if let Some(source_url) = table.get("source_url") {
                if let Some(url_str) = source_url.as_str() {
                    if is_twitter_status_url(url_str) {
                        // Metadata-level URLs don't belong to a specific puzzle
                        // Store with empty puzzle_name and index 0
                        results.push((
                            PuzzlePath {
                                collection: collection.to_string(),
                                puzzle_name: String::new(),
                                array_index: 0,
                            },
                            url_str.to_string(),
                        ));
                    }
                }
            }
        }
    }

    // Check [[puzzles]] array
    if let Some(puzzles) = doc.get("puzzles") {
        if let Some(array) = puzzles.as_array_of_tables() {
            for (idx, table) in array.iter().enumerate() {
                let puzzle_name = table
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string();

                // Check [puzzles.assets] source_url
                if let Some(assets) = table.get("assets") {
                    if let Some(assets_table) = assets.as_table() {
                        if let Some(source_url) = assets_table.get("source_url") {
                            if let Some(url_str) = source_url.as_str() {
                                if is_twitter_status_url(url_str) {
                                    results.push((
                                        PuzzlePath {
                                            collection: collection.to_string(),
                                            puzzle_name: puzzle_name.clone(),
                                            array_index: idx,
                                        },
                                        url_str.to_string(),
                                    ));
                                }
                            }
                        }
                    }
                }

                // Check [puzzles.key.seed.entropy.source] url
                if let Some(key) = table.get("key") {
                    if let Some(key_table) = key.as_table() {
                        if let Some(seed) = key_table.get("seed") {
                            if let Some(seed_table) = seed.as_table() {
                                if let Some(entropy) = seed_table.get("entropy") {
                                    if let Some(entropy_table) = entropy.as_table() {
                                        if let Some(source) = entropy_table.get("source") {
                                            if let Some(source_table) = source.as_table() {
                                                if let Some(url) = source_table.get("url") {
                                                    if let Some(url_str) = url.as_str() {
                                                        if is_twitter_status_url(url_str) {
                                                            results.push((
                                                                PuzzlePath {
                                                                    collection: collection
                                                                        .to_string(),
                                                                    puzzle_name: puzzle_name
                                                                        .clone(),
                                                                    array_index: idx,
                                                                },
                                                                url_str.to_string(),
                                                            ));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(results)
}

/// Check if a URL is an X/Twitter status URL (not a profile URL)
fn is_twitter_status_url(url: &str) -> bool {
    (url.contains("twitter.com") || url.contains("x.com")) && url.contains("/status/")
}

/// Canonicalize an X/Twitter URL
///
/// - Normalizes twitter.com → x.com
/// - Strips tracking params (?s=20, ?ref_src=..., etc.)
pub fn canonicalize_url(url: &str) -> String {
    // Normalize twitter.com → x.com
    let mut canonical = url.replace("twitter.com", "x.com");

    // Strip query parameters
    if let Some(pos) = canonical.find('?') {
        canonical.truncate(pos);
    }

    canonical
}

/// Update source_archives field in a puzzle's assets table
///
/// Navigates to the puzzle at `puzzle_path.array_index` in the `[[puzzles]]` array,
/// gets or creates `[puzzles.assets]` table, gets or creates `source_archives` array,
/// and appends `archive_path` to the array.
///
/// Uses `toml_edit` to preserve TOML formatting.
pub fn update_source_archives(
    doc: &mut DocumentMut,
    puzzle_path: &PuzzlePath,
    archive_path: &str,
) -> Result<(), Box<dyn Error>> {
    // Get puzzles array
    let puzzles = doc.get_mut("puzzles").ok_or("puzzles array not found")?;

    let array = puzzles
        .as_array_of_tables_mut()
        .ok_or("puzzles is not an array of tables")?;

    // Get puzzle at index
    let table = array
        .get_mut(puzzle_path.array_index)
        .ok_or_else(|| format!("puzzle at index {} not found", puzzle_path.array_index))?;

    // Get or create [puzzles.assets] table
    if !table.contains_key("assets") {
        table.insert("assets", Item::Table(toml_edit::Table::new()));
    }

    let assets = table.get_mut("assets").ok_or("failed to get assets")?;

    let assets_table = assets.as_table_mut().ok_or("assets is not a table")?;

    // Get or create source_archives array
    if !assets_table.contains_key("source_archives") {
        assets_table.insert("source_archives", Item::Value(Value::Array(Array::new())));
    }

    let source_archives = assets_table
        .get_mut("source_archives")
        .ok_or("failed to get source_archives")?;

    let archives_array = source_archives
        .as_array_mut()
        .ok_or("source_archives is not an array")?;

    // Check if archive_path already exists
    for item in archives_array.iter() {
        if let Some(existing) = item.as_str() {
            if existing == archive_path {
                // Already exists, don't duplicate
                return Ok(());
            }
        }
    }

    // Append archive_path
    archives_array.push(archive_path);

    Ok(())
}
