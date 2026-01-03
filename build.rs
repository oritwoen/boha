use num_bigint::BigUint;
use serde::Deserialize;
use sha2::{Digest, Sha256};
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

fn wif_to_hex(wif: &str) -> Option<String> {
    const COMPRESSED_PAYLOAD_LEN: usize = 33;
    const UNCOMPRESSED_PAYLOAD_LEN: usize = 32;
    const COMPRESSION_FLAG: u8 = 0x01;

    let decoded = bs58::decode(wif).into_vec().ok()?;
    if decoded.len() < 37 {
        return None;
    }

    let data = &decoded[..decoded.len() - 4];
    let checksum = &decoded[decoded.len() - 4..];
    let expected_checksum = &sha256(&sha256(data))[..4];
    if checksum != expected_checksum {
        return None;
    }

    let payload = &data[1..];

    let key_bytes = match payload.len() {
        COMPRESSED_PAYLOAD_LEN if payload[32] == COMPRESSION_FLAG => &payload[..32],
        UNCOMPRESSED_PAYLOAD_LEN => payload,
        _ => return None,
    };

    Some(hex::encode(key_bytes))
}

#[derive(Debug, Deserialize)]
struct AuthorConfig {
    name: Option<String>,
    #[serde(default)]
    addresses: Vec<String>,
    profile: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SolverConfig {
    name: Option<String>,
    address: Option<String>,
    #[serde(default)]
    verified: bool,
    source: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TomlTransaction {
    #[serde(rename = "type")]
    tx_type: String,
    txid: Option<String>,
    date: Option<String>,
    amount: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct TomlRedeemScript {
    script: String,
    hash: String,
}

#[derive(Debug, Deserialize)]
struct TomlSeed {
    phrase: String,
    path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TomlKey {
    hex: Option<String>,
    wif: Option<String>,
    seed: Option<TomlSeed>,
    mini: Option<String>,
    passphrase: Option<String>,
    bits: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct Address {
    value: String,
    kind: String,
    hash160: Option<String>,
    redeem_script: Option<TomlRedeemScript>,
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
    address: Address,
    prize: Option<f64>,
    status: String,
    #[allow(dead_code)]
    has_pubkey: Option<bool>,
    key: TomlKey,
    public_key: Option<String>,
    pubkey_format: Option<String>,
    start_date: Option<String>,
    solve_date: Option<String>,
    solve_time: Option<u64>,
    #[serde(default)]
    pre_genesis: bool,
    source_url: Option<String>,
    #[serde(default)]
    transactions: Vec<TomlTransaction>,
    solver: Option<SolverConfig>,
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
    address: Address,
    status: String,
    prize: Option<f64>,
    start_date: Option<String>,
    solve_date: Option<String>,
    solve_time: Option<u64>,
    source_url: Option<String>,
    #[serde(default)]
    transactions: Vec<TomlTransaction>,
    solver: Option<SolverConfig>,
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
    address: Address,
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
    solver: Option<SolverConfig>,
}

#[derive(Debug, Deserialize)]
struct ZdenMetadata {
    source_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ZdenFile {
    author: Option<AuthorConfig>,
    metadata: Option<ZdenMetadata>,
    puzzles: Vec<ZdenPuzzle>,
}

#[derive(Debug, Deserialize)]
struct ZdenPuzzle {
    name: String,
    chain: String,
    address: Address,
    status: String,
    prize: Option<f64>,
    key: Option<TomlKey>,
    start_date: Option<String>,
    solve_date: Option<String>,
    solve_time: Option<u64>,
    source_url: Option<String>,
    #[serde(default)]
    transactions: Vec<TomlTransaction>,
    solver: Option<SolverConfig>,
}

fn format_hash160(address: &Address, chain: &str, puzzle_id: &str) -> String {
    let requires_hash160 = address.kind == "p2pkh" && matches!(chain, "bitcoin" | "litecoin");
    if requires_hash160 && address.hash160.is_none() {
        panic!(
            "Puzzle '{}' (p2pkh) requires hash160 but none provided",
            puzzle_id
        );
    }
    match &address.hash160 {
        Some(h) => format!("Some(\"{}\")", h),
        None => "None".to_string(),
    }
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
                "pubkey_reveal" => "TransactionType::PubkeyReveal",
                other => panic!("Unknown transaction type: {}", other),
            };
            let txid = match &t.txid {
                Some(id) => format!("Some(\"{}\")", id),
                None => "None".to_string(),
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
                "Transaction {{ tx_type: {}, txid: {}, date: {}, amount: {} }}",
                tx_type, txid, date, amount
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

fn generate_solver_code(solver: &Option<SolverConfig>) -> String {
    match solver {
        Some(s) => {
            let name = match &s.name {
                Some(n) => format!("Some(\"{}\")", n),
                None => "None".to_string(),
            };
            let address = match &s.address {
                Some(a) => format!("Some(\"{}\")", a),
                None => "None".to_string(),
            };
            let source = match &s.source {
                Some(src) => format!("Some(\"{}\")", src),
                None => "None".to_string(),
            };
            format!(
                "Some(Solver {{ name: {}, address: {}, verified: {}, source: {} }})",
                name, address, s.verified, source
            )
        }
        None => "None".to_string(),
    }
}

fn generate_key_code(key: &Option<TomlKey>) -> String {
    match key {
        Some(k) => generate_key_code_required(k),
        None => "None".to_string(),
    }
}

fn generate_key_code_required(key: &TomlKey) -> String {
    let (hex_val, wif_val) = match (&key.hex, &key.wif) {
        (Some(h), Some(w)) => (Some(h.clone()), Some(w.clone())),
        (Some(h), None) => {
            let derived_wif = hex_to_wif(h, true);
            (Some(h.clone()), derived_wif)
        }
        (None, Some(w)) => {
            let derived_hex = wif_to_hex(w);
            (derived_hex, Some(w.clone()))
        }
        (None, None) => (None, None),
    };

    let hex = match &hex_val {
        Some(h) => format!("Some(\"{}\")", h),
        None => "None".to_string(),
    };
    let wif = match &wif_val {
        Some(w) => format!("Some(\"{}\")", w),
        None => "None".to_string(),
    };
    let seed = match &key.seed {
        Some(s) => {
            let path = match &s.path {
                Some(p) => format!("Some(\"{}\")", p),
                None => "None".to_string(),
            };
            format!("Some(Seed {{ phrase: \"{}\", path: {} }})", s.phrase, path)
        }
        None => "None".to_string(),
    };
    let mini = match &key.mini {
        Some(m) => format!("Some(\"{}\")", m),
        None => "None".to_string(),
    };
    let passphrase = match &key.passphrase {
        Some(p) => format!("Some(\"{}\")", p),
        None => "None".to_string(),
    };
    let bits = match key.bits {
        Some(b) => format!("Some({})", b),
        None => "None".to_string(),
    };
    format!(
        "Some(Key {{ hex: {}, wif: {}, seed: {}, mini: {}, passphrase: {}, bits: {} }})",
        hex, wif, seed, mini, passphrase, bits
    )
}

fn generate_redeem_script_code(rs: &Option<TomlRedeemScript>) -> String {
    match rs {
        Some(r) => format!(
            "Some(RedeemScript {{ script: \"{}\", hash: \"{}\" }})",
            r.script, r.hash
        ),
        None => "None".to_string(),
    }
}

fn main() {
    println!("cargo:rerun-if-changed=data/b1000.toml");
    println!("cargo:rerun-if-changed=data/hash_collision.toml");
    println!("cargo:rerun-if-changed=data/gsmg.toml");
    println!("cargo:rerun-if-changed=data/zden.toml");

    let out_dir = env::var("OUT_DIR").unwrap();

    generate_b1000(&out_dir);
    generate_hash_collision(&out_dir);
    generate_gsmg(&out_dir);
    generate_zden(&out_dir);
}

fn generate_b1000(out_dir: &str) {
    let dest_path = Path::new(out_dir).join("b1000_data.rs");

    let toml_content =
        fs::read_to_string("data/b1000.toml").expect("Failed to read data/b1000.toml");

    let data: Btc1000File = toml::from_str(&toml_content).expect("Failed to parse b1000.toml");

    for puzzle in &data.puzzles {
        if let Some(pk) = &puzzle.key.hex {
            if let Some(derived_bits) = bits_from_private_key(pk) {
                let declared_bits = puzzle.key.bits.expect("key.bits required for b1000");
                assert_eq!(
                    declared_bits, derived_bits,
                    "b1000/{} declares bits={} but key.hex implies bits={}",
                    declared_bits, declared_bits, derived_bits
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
        let bits = puzzle.key.bits.expect("key.bits required for b1000");

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
                    _ => panic!("Invalid pubkey_format '{}' for puzzle {}", fmt, bits),
                };
                format!("Some(Pubkey {{ key: \"{}\", format: {} }})", pk, format)
            }
            (None, None) => "None".to_string(),
            (Some(_), None) => panic!("Puzzle {} has public_key but no pubkey_format", bits),
            (None, Some(_)) => panic!("Puzzle {} has pubkey_format but no public_key", bits),
        };

        let key = generate_key_code_required(&puzzle.key);

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

        let hash160 = format_hash160(&puzzle.address, "bitcoin", &format!("b1000/{}", bits));
        let redeem_script = generate_redeem_script_code(&puzzle.address.redeem_script);

        let transactions = generate_transactions_code(&puzzle.transactions);
        let solver = generate_solver_code(&puzzle.solver);

        output.push_str(&format!(
            r#"    Puzzle {{
        id: "b1000/{}",
        chain: Chain::Bitcoin,
        address: Address {{
            value: "{}",
            chain: Chain::Bitcoin,
            kind: "{}",
            hash160: {},
            redeem_script: {},
        }},
        status: {},
        pubkey: {},
        key: {},
        prize: {},
        start_date: {},
        solve_date: {},
        solve_time: {},
        pre_genesis: {},
        source_url: {},
        transactions: {},
        solver: {},
    }},
"#,
            bits,
            puzzle.address.value,
            puzzle.address.kind,
            hash160,
            redeem_script,
            status,
            pubkey,
            key,
            prize,
            start_date,
            solve_date,
            solve_time,
            puzzle.pre_genesis,
            source_url,
            transactions,
            solver,
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

        let hash160 = format_hash160(
            &puzzle.address,
            "bitcoin",
            &format!("hash_collision/{}", puzzle.name),
        );
        let redeem_script = generate_redeem_script_code(&puzzle.address.redeem_script);

        let transactions = generate_transactions_code(&puzzle.transactions);
        let solver = generate_solver_code(&puzzle.solver);

        output.push_str(&format!(
            r#"    Puzzle {{
        id: "hash_collision/{}",
        chain: Chain::Bitcoin,
        address: Address {{
            value: "{}",
            chain: Chain::Bitcoin,
            kind: "{}",
            hash160: {},
            redeem_script: {},
        }},
        status: {},
        pubkey: None,
        key: None,
        prize: {},
        start_date: {},
        solve_date: {},
        solve_time: {},
        pre_genesis: false,
        source_url: {},
        transactions: {},
        solver: {},
    }},
"#,
            puzzle.name,
            puzzle.address.value,
            puzzle.address.kind,
            hash160,
            redeem_script,
            status,
            prize,
            start_date,
            solve_date,
            solve_time,
            source_url,
            transactions,
            solver,
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

    let hash160 = format_hash160(&puzzle.address, "bitcoin", "gsmg");
    let redeem_script = generate_redeem_script_code(&puzzle.address.redeem_script);

    let transactions = generate_transactions_code(&puzzle.transactions);
    let solver = generate_solver_code(&puzzle.solver);

    let mut output = String::new();
    output.push_str(&generate_author_code(&data.author));
    output.push('\n');
    output.push_str(&format!(
        r#"static PUZZLE: Puzzle = Puzzle {{
    id: "gsmg",
    chain: Chain::Bitcoin,
    address: Address {{
        value: "{}",
        chain: Chain::Bitcoin,
        kind: "{}",
        hash160: {},
        redeem_script: {},
    }},
    status: {},
    pubkey: {},
    key: None,
    prize: {},
    start_date: {},
    solve_date: {},
    solve_time: {},
    pre_genesis: false,
    source_url: {},
    transactions: {},
    solver: {},
}};
"#,
        puzzle.address.value,
        puzzle.address.kind,
        hash160,
        redeem_script,
        status,
        pubkey,
        prize,
        start_date,
        solve_date,
        solve_time,
        source_url,
        transactions,
        solver,
    ));

    fs::write(&dest_path, output).expect("Failed to write gsmg_data.rs");
}

fn generate_zden(out_dir: &str) {
    let dest_path = Path::new(out_dir).join("zden_data.rs");

    let toml_content = fs::read_to_string("data/zden.toml").expect("Failed to read data/zden.toml");

    let data: ZdenFile = toml::from_str(&toml_content).expect("Failed to parse zden.toml");

    let default_source_url = data.metadata.as_ref().and_then(|m| m.source_url.as_ref());

    let mut output = String::new();
    output.push_str(&generate_author_code(&data.author));
    output.push('\n');
    output.push_str("static PUZZLES: &[Puzzle] = &[\n");

    for puzzle in &data.puzzles {
        let chain = match puzzle.chain.as_str() {
            "bitcoin" => "Chain::Bitcoin",
            "ethereum" => "Chain::Ethereum",
            "litecoin" => "Chain::Litecoin",
            "monero" => "Chain::Monero",
            "decred" => "Chain::Decred",
            other => panic!("Unknown chain '{}' for puzzle {}", other, puzzle.name),
        };

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

        let hash160 = format_hash160(
            &puzzle.address,
            &puzzle.chain,
            &format!("zden/{}", puzzle.name),
        );
        let redeem_script = generate_redeem_script_code(&puzzle.address.redeem_script);
        let key = generate_key_code(&puzzle.key);

        let transactions = generate_transactions_code(&puzzle.transactions);
        let solver = generate_solver_code(&puzzle.solver);

        output.push_str(&format!(
            r#"    Puzzle {{
        id: "zden/{}",
        chain: {},
        address: Address {{
            value: "{}",
            chain: {},
            kind: "{}",
            hash160: {},
            redeem_script: {},
        }},
        status: {},
        pubkey: None,
        key: {},
        prize: {},
        start_date: {},
        solve_date: {},
        solve_time: {},
        pre_genesis: false,
        source_url: {},
        transactions: {},
        solver: {},
    }},
"#,
            puzzle.name,
            chain,
            puzzle.address.value,
            chain,
            puzzle.address.kind,
            hash160,
            redeem_script,
            status,
            key,
            prize,
            start_date,
            solve_date,
            solve_time,
            source_url,
            transactions,
            solver,
        ));
    }

    output.push_str("];\n");

    fs::write(&dest_path, output).expect("Failed to write zden_data.rs");
}
