use chrono::NaiveDate;
use std::path::Path;
use toml_edit::{DocumentMut, Item, Value};

fn calculate_solve_time(
    start_date: &str,
    solve_date: &str,
) -> Result<u32, Box<dyn std::error::Error>> {
    let start = NaiveDate::parse_from_str(start_date, "%Y-%m-%d")?;
    let solve = NaiveDate::parse_from_str(solve_date, "%Y-%m-%d")?;
    let days = (solve - start).num_days();
    if days < 0 {
        return Err(format!(
            "solve_date {} is before start_date {}",
            solve_date, start_date
        )
        .into());
    }
    Ok(days as u32)
}

fn update_puzzles_with_solve_time(doc: &mut DocumentMut) -> usize {
    let mut count = 0;

    if let Some(puzzles) = doc.get_mut("puzzles") {
        if let Some(array) = puzzles.as_array_of_tables_mut() {
            for table in array.iter_mut() {
                let start_date = table.get("start_date").and_then(|v| v.as_str());
                let solve_date = table.get("solve_date").and_then(|v| v.as_str());

                if let (Some(start), Some(solve)) = (start_date, solve_date) {
                    match calculate_solve_time(start, solve) {
                        Ok(days) => {
                            table.insert("solve_time", Item::Value(Value::from(days as i64)));
                            count += 1;
                        }
                        Err(e) => {
                            let name = table.get("bits").or(table.get("name"));
                            eprintln!("  Error processing {:?}: {}", name, e);
                        }
                    }
                }
            }
        }
    }

    count
}

fn update_single_puzzle_with_solve_time(doc: &mut DocumentMut) -> usize {
    if let Some(puzzle) = doc.get_mut("puzzle") {
        if let Some(table) = puzzle.as_table_mut() {
            let start_date = table.get("start_date").and_then(|v| v.as_str());
            let solve_date = table.get("solve_date").and_then(|v| v.as_str());

            if let (Some(start), Some(solve)) = (start_date, solve_date) {
                match calculate_solve_time(start, solve) {
                    Ok(days) => {
                        table.insert("solve_time", Item::Value(Value::from(days as i64)));
                        return 1;
                    }
                    Err(e) => {
                        eprintln!("  Error processing gsmg puzzle: {}", e);
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
        update_puzzles_with_solve_time(&mut doc)
    } else if doc.get("puzzle").is_some() {
        update_single_puzzle_with_solve_time(&mut doc)
    } else {
        println!("  No puzzles found");
        return Ok(());
    };

    if count > 0 {
        std::fs::write(path, doc.to_string())?;
        println!("  Updated {} entries with solve_time", count);
    } else {
        println!("  No updates needed (no puzzles with both start_date and solve_date)");
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = Path::new("../data");

    let files = ["b1000.toml", "gsmg.toml", "hash_collision.toml"];

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
