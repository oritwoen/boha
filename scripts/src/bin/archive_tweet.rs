#[allow(dead_code)]
mod utils {
    include!("../utils/mod.rs");
}

use chrono::Utc;
use clap::Parser;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use toml_edit::DocumentMut;
use utils::playwright::{PlaywrightContext, TweetArchive};
use utils::{source_archives, wayback};

#[derive(Parser, Debug)]
#[command(name = "archive-tweet")]
#[command(about = "Archive X/Twitter source tweets into assets/ + update data/*.toml", long_about = None)]
struct Cli {
    /// Archive a single tweet URL (must be a status URL)
    #[arg(long)]
    url: Option<String>,

    /// Collection name (required when --url is used)
    #[arg(long)]
    collection: Option<String>,

    /// Preview actions without writing files or updating TOML
    #[arg(long)]
    dry_run: bool,

    /// Re-archive even if archive already exists
    #[arg(long)]
    force: bool,

    /// Collection name (positional form)
    #[arg(value_name = "COLLECTION")]
    collection_pos: Option<String>,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn data_path(collection: &str) -> PathBuf {
    repo_root()
        .join("data")
        .join(format!("{}.toml", collection))
}

fn assets_collection_dir(collection: &str) -> PathBuf {
    repo_root().join("assets").join(collection)
}

fn sanitize_dir_name(name: &str) -> String {
    let mut out = String::new();
    let mut prev_underscore = false;

    for ch in name.chars() {
        let mapped = match ch {
            'a'..='z' | '0'..='9' | '_' => Some(ch),
            'A'..='Z' => Some(ch.to_ascii_lowercase()),
            ' ' | '-' => Some('_'),
            _ => None,
        };

        if let Some(c) = mapped {
            if c == '_' {
                if prev_underscore || out.is_empty() {
                    continue;
                }
                prev_underscore = true;
            } else {
                prev_underscore = false;
            }
            out.push(c);
        }
    }

    while out.ends_with('_') {
        out.pop();
    }

    if out.is_empty() {
        "unknown".to_string()
    } else {
        out
    }
}

fn puzzle_table(doc: &DocumentMut, idx: usize) -> Option<&toml_edit::Table> {
    let puzzles = doc.get("puzzles")?;
    let array = puzzles.as_array_of_tables()?;
    array.get(idx)
}

fn puzzle_name_at(doc: &DocumentMut, idx: usize) -> Option<String> {
    puzzle_table(doc, idx)
        .and_then(|t| t.get("name"))
        .and_then(|n| n.as_str())
        .map(|s| s.to_string())
}

fn puzzle_assets_dir(doc: &DocumentMut, idx: usize) -> Option<String> {
    let table = puzzle_table(doc, idx)?;

    if let Some(dir) = table
        .get("assets")
        .and_then(|a| a.as_table())
        .and_then(|assets| assets.get("puzzle"))
        .and_then(|p| p.as_str())
        .and_then(|p| p.split('/').next())
        .filter(|s| !s.trim().is_empty())
    {
        return Some(dir.to_string());
    }

    // Fall back to a deterministic slug based on the puzzle name.
    let name = table
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("unknown");
    Some(sanitize_dir_name(name))
}

fn is_twitter_status_url(url: &str) -> bool {
    (url.contains("twitter.com") || url.contains("x.com")) && url.contains("/status/")
}

fn archive_paths(collection: &str, puzzle_dir: &str) -> (PathBuf, PathBuf, String) {
    let md_path = assets_collection_dir(collection)
        .join(puzzle_dir)
        .join("source_archive.md");
    let png_path = assets_collection_dir(collection)
        .join(puzzle_dir)
        .join("source_archive.png");
    let rel_md = format!("{}/source_archive.md", puzzle_dir);
    (md_path, png_path, rel_md)
}

fn format_markdown(url: &str, archive: &TweetArchive, archived_yyyy_mm_dd: &str) -> String {
    let text = archive.text.trim_end();

    format!(
        "---\nurl: {url}\nauthor: \"{author}\"\ndate: \"{date}\"\narchived: \"{archived}\"\n---\n\n{text}\n\n![screenshot](source_archive.png)\n",
        url = url,
        author = archive.author,
        date = archive.date,
        archived = archived_yyyy_mm_dd,
        text = text,
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let collection = match (&cli.url, &cli.collection, &cli.collection_pos) {
        (Some(_), Some(c), _) => c.clone(),
        (Some(_), None, _) => {
            return Err("--collection is required when using --url".into());
        }
        (None, _, Some(c)) => c.clone(),
        (None, Some(_), None) => {
            return Err("Use positional <collection> (or --url with --collection)".into());
        }
        (None, None, None) => {
            return Err("Missing <collection>. Try --help".into());
        }
    };

    if let Some(url) = &cli.url {
        if !is_twitter_status_url(url) {
            return Err("--url must be an X/Twitter status URL (contains /status/)".into());
        }
    }

    let path = data_path(&collection);
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()).into());
    }

    let content = std::fs::read_to_string(&path)?;
    let mut doc: DocumentMut = content.parse()?;

    let mut extracted = source_archives::extract_twitter_urls(&doc, &collection)?;

    // URL mode: only consider the specific URL (if present in the TOML).
    if let Some(target_url) = &cli.url {
        let target = source_archives::canonicalize_url(target_url);
        extracted.retain(|(_, u)| source_archives::canonicalize_url(u) == target);

        if extracted.is_empty() {
            eprintln!("Warning: URL not found in {}. Skipping.", path.display());
            return Ok(());
        }
    }

    // canonical_url -> all puzzle paths referencing it
    let mut refs_by_url: HashMap<String, Vec<source_archives::PuzzlePath>> = HashMap::new();
    for (p, u) in extracted {
        let canonical = source_archives::canonicalize_url(&u);
        refs_by_url.entry(canonical).or_default().push(p);
    }

    if refs_by_url.is_empty() {
        println!("No X/Twitter status URLs found for {}", collection);
        return Ok(());
    }

    // Stable output ordering.
    let mut ordered: BTreeMap<String, Vec<source_archives::PuzzlePath>> = BTreeMap::new();
    for (k, v) in refs_by_url {
        ordered.insert(k, v);
    }

    let archived_today = Utc::now().format("%Y-%m-%d").to_string();

    let mut ctx: Option<PlaywrightContext> = None;

    let mut any_toml_updates = false;

    for (canonical_url, mut refs) in ordered {
        refs.sort_by_key(|p| p.array_index);

        let non_metadata_refs: Vec<_> = refs.iter().filter(|p| !p.puzzle_name.is_empty()).collect();

        let owner_idx = non_metadata_refs
            .first()
            .map(|p| p.array_index)
            .or_else(|| {
                // Metadata-only reference: fall back to the first puzzle entry.
                // This ensures we still have a deterministic storage location.
                puzzle_table(&doc, 0).map(|_| 0)
            });

        let Some(owner_idx) = owner_idx else {
            eprintln!(
                "Warning: No puzzles found in {}. Skipping {}",
                collection, canonical_url
            );
            continue;
        };

        let owner_name = puzzle_name_at(&doc, owner_idx).unwrap_or_else(|| "unknown".to_string());
        let owner_dir = match puzzle_assets_dir(&doc, owner_idx) {
            Some(d) => d,
            None => {
                eprintln!(
                    "Warning: Could not determine assets dir for puzzle index {}. Skipping {}",
                    owner_idx, canonical_url
                );
                continue;
            }
        };

        let (md_path, png_path, rel_md) = archive_paths(&collection, &owner_dir);
        let archive_exists = md_path.exists() && png_path.exists();

        let ref_names: Vec<String> = non_metadata_refs
            .iter()
            .map(|p| p.puzzle_name.clone())
            .collect();

        println!("URL: {}", canonical_url);
        println!(
            "  Owner: [{}] {} (dir: {})",
            owner_idx, owner_name, owner_dir
        );
        println!("  Archive: {}", md_path.display());
        if !ref_names.is_empty() {
            println!("  Referenced by: {}", ref_names.join(", "));
        }

        if cli.dry_run {
            if archive_exists && !cli.force {
                println!("  Dry-run: would reuse existing archive");
            } else if cli.force {
                println!("  Dry-run: would (re)archive and overwrite files");
            } else {
                println!("  Dry-run: would archive");
            }
            println!("  Dry-run: would set source_archives += \"{}\"\n", rel_md);
            continue;
        }

        let mut refs_to_update: Vec<source_archives::PuzzlePath> = refs
            .iter()
            .filter(|p| !p.puzzle_name.is_empty())
            .cloned()
            .collect();

        if refs_to_update.is_empty() {
            refs_to_update.push(source_archives::PuzzlePath {
                collection: collection.clone(),
                puzzle_name: owner_name.clone(),
                array_index: owner_idx,
            });
        }

        for p in &refs_to_update {
            source_archives::update_source_archives(&mut doc, p, &rel_md)?;
            any_toml_updates = true;
        }

        if archive_exists && !cli.force {
            println!("  Skipping screenshot - archive already exists\n");
            continue;
        }

        let ctx_ref = match &ctx {
            Some(c) => c,
            None => {
                let state_path = repo_root().join("scripts/.playwright-state.json");
                let state_path = state_path
                    .to_str()
                    .ok_or("playwright state path is not valid UTF-8")?;
                let new_ctx = PlaywrightContext::new(state_path).await?;
                ctx.insert(new_ctx)
            }
        };

        let archive = match ctx_ref.screenshot_tweet(&canonical_url).await {
            Ok(a) => a,
            Err(e) => {
                eprintln!("  Live tweet screenshot failed: {e}");
                match wayback::check_availability(&canonical_url).await {
                    Ok(Some(wayback_url)) => {
                        println!("  Falling back to Wayback: {}", wayback_url);
                        match wayback::screenshot_wayback(ctx_ref, &wayback_url).await {
                            Ok(a) => a,
                            Err(e) => {
                                eprintln!("  Wayback screenshot failed: {e}");
                                eprintln!("  Warning: failed to archive {canonical_url}\n");
                                continue;
                            }
                        }
                    }
                    Ok(None) => {
                        eprintln!("  Wayback: no snapshot available");
                        eprintln!("  Warning: failed to archive {canonical_url}\n");
                        continue;
                    }
                    Err(e) => {
                        eprintln!("  Wayback availability check failed: {e}");
                        eprintln!("  Warning: failed to archive {canonical_url}\n");
                        continue;
                    }
                }
            }
        };

        if let Some(parent) = md_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let md = format_markdown(&canonical_url, &archive, &archived_today);
        std::fs::write(&md_path, md)?;
        std::fs::write(&png_path, &archive.png)?;

        println!("  Saved markdown + screenshot\n");
    }

    if !cli.dry_run && any_toml_updates {
        std::fs::write(&path, doc.to_string())?;
        println!("Updated {}", path.display());
    }

    Ok(())
}
