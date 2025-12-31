use num_bigint::BigUint;
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

fn bits_from_private_key(private_key: &str) -> Option<u16> {
    let bytes = hex::decode(private_key).ok()?;
    let key = BigUint::from_bytes_be(&bytes);
    if key == BigUint::ZERO {
        return None;
    }
    Some(key.bits() as u16)
}

#[derive(Debug, Deserialize)]
struct AuthorConfig {
    name: Option<String>,
    #[serde(default)]
    addresses: Vec<String>,
    profile: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TomlTransaction {
    #[serde(rename = "type")]
    tx_type: String,
    txid: String,
    date: Option<String>,
    amount: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct Btc1000Metadata {
    source_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Btc1000File {
    author: Option<AuthorConfig>,
    metadata: Option<Btc1000Metadata>,
    puzzles: Vec<Btc1000Puzzle>,
}

#[derive(Debug, Deserialize)]
struct Btc1000Puzzle {
    bits: u16,
    address: String,
    h160: Option<String>,
    prize: Option<f64>,
    status: String,
    #[allow(dead_code)]
    has_pubkey: Option<bool>,
    private_key: Option<String>,
    public_key: Option<String>,
    pubkey_format: Option<String>,
    start_date: Option<String>,
    solve_date: Option<String>,
    solve_time: Option<u64>,
    source_url: Option<String>,
    #[serde(default)]
    transactions: Vec<TomlTransaction>,
}

#[derive(Debug, Deserialize)]
struct HashCollisionMetadata {
    source_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HashCollisionFile {
    author: Option<AuthorConfig>,
    metadata: Option<HashCollisionMetadata>,
    puzzles: Vec<HashCollisionPuzzle>,
}

#[derive(Debug, Deserialize)]
struct HashCollisionPuzzle {
    name: String,
    address: String,
    status: String,
    redeem_script: String,
    script_hash: Option<String>,
    prize: Option<f64>,
    start_date: Option<String>,
    solve_date: Option<String>,
    solve_time: Option<u64>,
    source_url: Option<String>,
    #[serde(default)]
    transactions: Vec<TomlTransaction>,
}

#[derive(Debug, Deserialize)]
struct GsmgMetadata {
    source_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GsmgFile {
    author: Option<AuthorConfig>,
    metadata: Option<GsmgMetadata>,
    puzzle: GsmgPuzzle,
}

#[derive(Debug, Deserialize)]
struct GsmgPuzzle {
    address: String,
    h160: Option<String>,
    status: String,
    prize: Option<f64>,
    public_key: Option<String>,
    pubkey_format: Option<String>,
    start_date: Option<String>,
    solve_date: Option<String>,
    solve_time: Option<u64>,
    source_url: Option<String>,
    #[serde(default)]
    transactions: Vec<TomlTransaction>,
}

fn generate_transactions_code(transactions: &[TomlTransaction]) -> String {
    if transactions.is_empty() {
        return "&[]".to_string();
    }

    let tx_list: Vec<String> = transactions
        .iter()
        .map(|t| {
            let tx_type = match t.tx_type.as_str() {
                "funding" => "TransactionType::Funding",
                "increase" => "TransactionType::Increase",
                "decrease" => "TransactionType::Decrease",
                "sweep" => "TransactionType::Sweep",
                "claim" => "TransactionType::Claim",
                other => panic!("Unknown transaction type: {}", other),
            };
            let date = match &t.date {
                Some(d) => format!("Some(\"{}\")", d),
                None => "None".to_string(),
            };
            let amount = match t.amount {
                Some(a) => format!("Some({:.8})", a),
                None => "None".to_string(),
            };
            format!(
                "Transaction {{ tx_type: {}, txid: \"{}\", date: {}, amount: {} }}",
                tx_type, t.txid, date, amount
            )
        })
        .collect();

    format!("&[{}]", tx_list.join(", "))
}

fn generate_author_code(author: &Option<AuthorConfig>) -> String {
    match author {
        Some(a) => {
            let name = match &a.name {
                Some(n) => format!("Some(\"{}\")", n),
                None => "None".to_string(),
            };
            let addresses = if a.addresses.is_empty() {
                "&[]".to_string()
            } else {
                let addrs: Vec<String> = a.addresses.iter().map(|addr| format!("\"{}\"", addr)).collect();
                format!("&[{}]", addrs.join(", "))
            };
            let profile = match &a.profile {
                Some(p) => format!("Some(\"{}\")", p),
                None => "None".to_string(),
            };
            format!(
                "static AUTHOR: Author = Author {{\n    name: {},\n    addresses: {},\n    profile: {},\n}};\n",
                name, addresses, profile
            )
        }
        None => {
            "static AUTHOR: Author = Author {\n    name: None,\n    addresses: &[],\n    profile: None,\n};\n".to_string()
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=data/b1000.toml");
    println!("cargo:rerun-if-changed=data/hash_collision.toml");
    println!("cargo:rerun-if-changed=data/gsmg.toml");

    let out_dir = env::var("OUT_DIR").unwrap();

    generate_b1000(&out_dir);
    generate_hash_collision(&out_dir);
    generate_gsmg(&out_dir);
}

fn generate_b1000(out_dir: &str) {
    let dest_path = Path::new(out_dir).join("b1000_data.rs");

    let toml_content =
        fs::read_to_string("data/b1000.toml").expect("Failed to read data/b1000.toml");

    let data: Btc1000File = toml::from_str(&toml_content).expect("Failed to parse b1000.toml");

    for puzzle in &data.puzzles {
        if let Some(pk) = &puzzle.private_key {
            if let Some(derived_bits) = bits_from_private_key(pk) {
                assert_eq!(
                    puzzle.bits, derived_bits,
                    "Puzzle {} has bits={} but private_key implies bits={}",
                    puzzle.bits, puzzle.bits, derived_bits
                );
            }
        }
    }

    let default_source_url = data.metadata.as_ref().and_then(|m| m.source_url.as_ref());

    let mut output = String::new();
    output.push_str(&generate_author_code(&data.author));
    output.push('\n');
    output.push_str("static PUZZLES: &[Puzzle] = &[\n");

    for puzzle in &data.puzzles {
        let status = match puzzle.status.as_str() {
            "solved" => "Status::Solved",
            "claimed" => "Status::Claimed",
            "swept" => "Status::Swept",
            _ => "Status::Unsolved",
        };

        let pubkey = match (&puzzle.public_key, &puzzle.pubkey_format) {
            (Some(pk), Some(fmt)) => {
                let format = match fmt.as_str() {
                    "compressed" => "PubkeyFormat::Compressed",
                    "uncompressed" => "PubkeyFormat::Uncompressed",
                    _ => panic!("Invalid pubkey_format '{}' for puzzle {}", fmt, puzzle.bits),
                };
                format!("Some(Pubkey {{ key: \"{}\", format: {} }})", pk, format)
            }
            (None, None) => "None".to_string(),
            (Some(_), None) => panic!("Puzzle {} has public_key but no pubkey_format", puzzle.bits),
            (None, Some(_)) => panic!("Puzzle {} has pubkey_format but no public_key", puzzle.bits),
        };

        let private_key = match &puzzle.private_key {
            Some(pk) => format!("Some(\"{}\")", pk),
            None => "None".to_string(),
        };

        let prize = match puzzle.prize {
            Some(p) => format!("Some({:.6})", p),
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

        let solve_time = match puzzle.solve_time {
            Some(t) => format!("Some({})", t),
            None => "None".to_string(),
        };

        let source_url = puzzle
            .source_url
            .as_ref()
            .or(default_source_url)
            .map(|url| format!("Some(\"{}\")", url))
            .unwrap_or_else(|| "None".to_string());

        let h160 = match &puzzle.h160 {
            Some(h) => format!("Some(\"{}\")", h),
            None => "None".to_string(),
        };

        let transactions = generate_transactions_code(&puzzle.transactions);

        output.push_str(&format!(
            r#"    Puzzle {{
        id: "b1000/{}",
        chain: Chain::Bitcoin,
        address: "{}",
        address_type: Some(AddressType::P2PKH),
        h160: {},
        status: {},
        pubkey: {},
        private_key: {},
        key_source: KeySource::Direct {{ bits: {} }},
        prize: {},
        start_date: {},
        solve_date: {},
        solve_time: {},
        source_url: {},
        transactions: {},
    }},
"#,
            puzzle.bits,
            puzzle.address,
            h160,
            status,
            pubkey,
            private_key,
            puzzle.bits,
            prize,
            start_date,
            solve_date,
            solve_time,
            source_url,
            transactions,
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
    output.push_str(&generate_author_code(&data.author));
    output.push('\n');
    output.push_str("static PUZZLES: &[Puzzle] = &[\n");

    for puzzle in &data.puzzles {
        let status = match puzzle.status.as_str() {
            "solved" => "Status::Solved",
            "claimed" => "Status::Claimed",
            "swept" => "Status::Swept",
            _ => "Status::Unsolved",
        };

        let prize = match puzzle.prize {
            Some(p) => format!("Some({:.6})", p),
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

        let solve_time = match puzzle.solve_time {
            Some(t) => format!("Some({})", t),
            None => "None".to_string(),
        };

        let source_url = puzzle
            .source_url
            .as_ref()
            .or(default_source_url)
            .map(|url| format!("Some(\"{}\")", url))
            .unwrap_or_else(|| "None".to_string());

        let script_hash = match &puzzle.script_hash {
            Some(h) => format!("Some(\"{}\")", h),
            None => "None".to_string(),
        };

        let transactions = generate_transactions_code(&puzzle.transactions);

        output.push_str(&format!(
            r#"    Puzzle {{
        id: "hash_collision/{}",
        chain: Chain::Bitcoin,
        address: "{}",
        address_type: Some(AddressType::P2SH),
        h160: None,
        status: {},
        pubkey: None,
        private_key: None,
        key_source: KeySource::Script {{ redeem_script: "{}", script_hash: {} }},
        prize: {},
        start_date: {},
        solve_date: {},
        solve_time: {},
        source_url: {},
        transactions: {},
    }},
"#,
            puzzle.name,
            puzzle.address,
            status,
            puzzle.redeem_script,
            script_hash,
            prize,
            start_date,
            solve_date,
            solve_time,
            source_url,
            transactions,
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

    let prize = match puzzle.prize {
        Some(p) => format!("Some({:.8})", p),
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

    let solve_time = match puzzle.solve_time {
        Some(t) => format!("Some({})", t),
        None => "None".to_string(),
    };

    let source_url = puzzle
        .source_url
        .as_ref()
        .or(default_source_url)
        .map(|url| format!("Some(\"{}\")", url))
        .unwrap_or_else(|| "None".to_string());

    let pubkey = match (&puzzle.public_key, &puzzle.pubkey_format) {
        (Some(pk), Some(fmt)) => {
            let format = match fmt.as_str() {
                "compressed" => "PubkeyFormat::Compressed",
                "uncompressed" => "PubkeyFormat::Uncompressed",
                _ => panic!("Invalid pubkey_format '{}' for gsmg", fmt),
            };
            format!("Some(Pubkey {{ key: \"{}\", format: {} }})", pk, format)
        }
        (None, None) => "None".to_string(),
        (Some(_), None) => panic!("gsmg has public_key but no pubkey_format"),
        (None, Some(_)) => panic!("gsmg has pubkey_format but no public_key"),
    };

    let h160 = match &puzzle.h160 {
        Some(h) => format!("Some(\"{}\")", h),
        None => "None".to_string(),
    };

    let transactions = generate_transactions_code(&puzzle.transactions);

    let mut output = String::new();
    output.push_str(&generate_author_code(&data.author));
    output.push('\n');
    output.push_str(&format!(
        r#"static PUZZLE: Puzzle = Puzzle {{
    id: "gsmg",
    chain: Chain::Bitcoin,
    address: "{}",
    address_type: Some(AddressType::P2PKH),
    h160: {},
    status: {},
    pubkey: {},
    private_key: None,
    key_source: KeySource::Unknown,
    prize: {},
    start_date: {},
    solve_date: {},
    solve_time: {},
    source_url: {},
    transactions: {},
}};
"#,
        puzzle.address,
        h160,
        status,
        pubkey,
        prize,
        start_date,
        solve_date,
        solve_time,
        source_url,
        transactions,
    ));

    fs::write(&dest_path, output).expect("Failed to write gsmg_data.rs");
}
