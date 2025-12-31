use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Btc1000Metadata {
    source_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Btc1000File {
    metadata: Option<Btc1000Metadata>,
    puzzles: Vec<Btc1000Puzzle>,
}

#[derive(Debug, Deserialize)]
struct Btc1000Puzzle {
    bits: u16,
    address: String,
    btc: Option<f64>,
    status: String,
    #[allow(dead_code)]
    has_pubkey: Option<bool>,
    private_key: Option<String>,
    public_key: Option<String>,
    start_date: Option<String>,
    solve_date: Option<String>,
    source_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HashCollisionMetadata {
    source_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HashCollisionFile {
    metadata: Option<HashCollisionMetadata>,
    puzzles: Vec<HashCollisionPuzzle>,
}

#[derive(Debug, Deserialize)]
struct HashCollisionPuzzle {
    name: String,
    address: String,
    status: String,
    redeem_script: String,
    btc: Option<f64>,
    start_date: Option<String>,
    solve_date: Option<String>,
    source_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GsmgMetadata {
    source_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GsmgFile {
    metadata: Option<GsmgMetadata>,
    puzzle: GsmgPuzzle,
}

#[derive(Debug, Deserialize)]
struct GsmgPuzzle {
    address: String,
    status: String,
    btc: Option<f64>,
    start_date: Option<String>,
    solve_date: Option<String>,
    source_url: Option<String>,
}
#[derive(Debug, Deserialize)]
struct ArweaveMetadata {
    source_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ArweaveFile {
    metadata: Option<ArweaveMetadata>,
    puzzles: Vec<ArweavePuzzle>,
}

#[derive(Debug, Deserialize)]
struct ArweavePuzzle {
    id: String,
    address: String,
    status: String,
    #[allow(dead_code)]
    prize_ar: Option<u64>,
}

fn main() {
    println!("cargo:rerun-if-changed=data/b1000.toml");
    println!("cargo:rerun-if-changed=data/hash_collision.toml");
    println!("cargo:rerun-if-changed=data/gsmg.toml");
    println!("cargo:rerun-if-changed=data/arweave.toml");
    let out_dir = env::var("OUT_DIR").unwrap();

    generate_b1000(&out_dir);
    generate_hash_collision(&out_dir);
    generate_gsmg(&out_dir);
    generate_arweave(&out_dir);
}

fn generate_b1000(out_dir: &str) {
    let dest_path = Path::new(out_dir).join("b1000_data.rs");

    let toml_content =
        fs::read_to_string("data/b1000.toml").expect("Failed to read data/b1000.toml");

    let data: Btc1000File = toml::from_str(&toml_content).expect("Failed to parse b1000.toml");

    let default_source_url = data.metadata.as_ref().and_then(|m| m.source_url.as_ref());

    let mut output = String::new();
    output.push_str("static PUZZLES: &[Puzzle] = &[\n");

    for puzzle in &data.puzzles {
        let status = match puzzle.status.as_str() {
            "solved" => "Status::Solved",
            "claimed" => "Status::Claimed",
            "swept" => "Status::Swept",
            _ => "Status::Unsolved",
        };

        let pubkey = match &puzzle.public_key {
            Some(pk) => format!("Some(\"{}\")", pk),
            None => "None".to_string(),
        };

        let private_key = match &puzzle.private_key {
            Some(pk) => format!("Some(\"{}\")", pk),
            None => "None".to_string(),
        };

        let prize = match puzzle.btc {
            Some(btc) => format!("Some({:.6})", btc),
            None => "None".to_string(),
        };

        let start_date = match &puzzle.start_date {
            Some(d) => format!("Some(\"{}\")", d),
            None => "None".to_string(),
        };

        let solve_date = match &puzzle.solve_date {
            Some(d) => format!("Some(\"{}\")", d),
            None => "None".to_string(),
        };

        let source_url = puzzle
            .source_url
            .as_ref()
            .or(default_source_url)
            .map(|url| format!("Some(\"{}\")", url))
            .unwrap_or_else(|| "None".to_string());

        output.push_str(&format!(
            r#"    Puzzle {{
        id: "b1000/{}",
        address: "{}",
        address_type: AddressType::P2PKH,
        status: {},
        pubkey: {},
        private_key: {},
        redeem_script: None,
        bits: Some({}),
        prize_btc: {},
        start_date: {},
        solve_date: {},
        source_url: {},
    }},
"#,
            puzzle.bits,
            puzzle.address,
            status,
            pubkey,
            private_key,
            puzzle.bits,
            prize,
            start_date,
            solve_date,
            source_url,
        ));
    }

    output.push_str("];\n");

    fs::write(&dest_path, output).expect("Failed to write b1000_data.rs");
}

fn generate_hash_collision(out_dir: &str) {
    let dest_path = Path::new(out_dir).join("hash_collision_data.rs");

    let toml_content = fs::read_to_string("data/hash_collision.toml")
        .expect("Failed to read data/hash_collision.toml");

    let data: HashCollisionFile =
        toml::from_str(&toml_content).expect("Failed to parse hash_collision.toml");

    let default_source_url = data.metadata.as_ref().and_then(|m| m.source_url.as_ref());

    let mut output = String::new();
    output.push_str("static PUZZLES: &[Puzzle] = &[\n");

    for puzzle in &data.puzzles {
        let status = match puzzle.status.as_str() {
            "solved" => "Status::Solved",
            "claimed" => "Status::Claimed",
            "swept" => "Status::Swept",
            _ => "Status::Unsolved",
        };

        let prize = match puzzle.btc {
            Some(btc) => format!("Some({:.6})", btc),
            None => "None".to_string(),
        };

        let start_date = match &puzzle.start_date {
            Some(d) => format!("Some(\"{}\")", d),
            None => "None".to_string(),
        };

        let solve_date = match &puzzle.solve_date {
            Some(d) => format!("Some(\"{}\")", d),
            None => "None".to_string(),
        };

        let source_url = puzzle
            .source_url
            .as_ref()
            .or(default_source_url)
            .map(|url| format!("Some(\"{}\")", url))
            .unwrap_or_else(|| "None".to_string());

        output.push_str(&format!(
            r#"    Puzzle {{
        id: "hash_collision/{}",
        address: "{}",
        address_type: AddressType::P2SH,
        status: {},
        pubkey: None,
        private_key: None,
        redeem_script: Some("{}"),
        bits: None,
        prize_btc: {},
        start_date: {},
        solve_date: {},
        source_url: {},
    }},
"#,
            puzzle.name,
            puzzle.address,
            status,
            puzzle.redeem_script,
            prize,
            start_date,
            solve_date,
            source_url,
        ));
    }

    output.push_str("];\n");

    fs::write(&dest_path, output).expect("Failed to write hash_collision_data.rs");
}

fn generate_gsmg(out_dir: &str) {
    let dest_path = Path::new(out_dir).join("gsmg_data.rs");

    let toml_content = fs::read_to_string("data/gsmg.toml").expect("Failed to read data/gsmg.toml");

    let data: GsmgFile = toml::from_str(&toml_content).expect("Failed to parse gsmg.toml");

    let puzzle = &data.puzzle;
    let default_source_url = data.metadata.as_ref().and_then(|m| m.source_url.as_ref());

    let status = match puzzle.status.as_str() {
        "solved" => "Status::Solved",
        "claimed" => "Status::Claimed",
        "swept" => "Status::Swept",
        _ => "Status::Unsolved",
    };

    let prize = match puzzle.btc {
        Some(btc) => format!("Some({:.8})", btc),
        None => "None".to_string(),
    };

    let start_date = match &puzzle.start_date {
        Some(d) => format!("Some(\"{}\")", d),
        None => "None".to_string(),
    };

    let solve_date = match &puzzle.solve_date {
        Some(d) => format!("Some(\"{}\")", d),
        None => "None".to_string(),
    };

    let source_url = puzzle
        .source_url
        .as_ref()
        .or(default_source_url)
        .map(|url| format!("Some(\"{}\")", url))
        .unwrap_or_else(|| "None".to_string());

    let output = format!(
        r#"static PUZZLE: Puzzle = Puzzle {{
    id: "gsmg",
    address: "{}",
    address_type: AddressType::P2PKH,
    status: {},
    pubkey: None,
    private_key: None,
    redeem_script: None,
    bits: None,
    prize_btc: {},
    start_date: {},
    solve_date: {},
    source_url: {},
}};
"#,
        puzzle.address, status, prize, start_date, solve_date, source_url,
    );

    fs::write(&dest_path, output).expect("Failed to write gsmg_data.rs");
}

fn generate_arweave(out_dir: &str) {
    let dest_path = Path::new(out_dir).join("arweave_data.rs");
    let toml_content =
        fs::read_to_string("data/arweave.toml").expect("Failed to read data/arweave.toml");
    let data: ArweaveFile = toml::from_str(&toml_content).expect("Failed to parse arweave.toml");
    let default_source_url = data.metadata.as_ref().and_then(|m| m.source_url.as_ref());
    let mut output = String::new();
    output.push_str("static PUZZLES: &[Puzzle] = &[\n");
    for puzzle in &data.puzzles {
        let status = match puzzle.status.as_str() {
            "solved" => "Status::Solved",
            "claimed" => "Status::Claimed",
            "swept" => "Status::Swept",
            _ => "Status::Unsolved",
        };
        let source_url = default_source_url
            .map(|url| format!("Some(\"{}\")", url))
            .unwrap_or_else(|| "None".to_string());
        output.push_str(&format!(
            r#"    Puzzle {{
        id: "arweave/{}",
        address: "{}",
        address_type: AddressType::Arweave,
        status: {},
        pubkey: None,
        private_key: None,
        redeem_script: None,
        bits: None, 
        prize_btc: None,
        start_date: None,
        solve_date: None,
        source_url: {},
        }},
"#,
            puzzle.id, puzzle.address, status, source_url,
        ));
    }
    output.push_str("];\n");
    fs::write(&dest_path, output).expect("Failed to write arweave_data.rs");
}
