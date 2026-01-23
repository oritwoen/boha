use bip38::Decrypt;
use json_strip_comments::strip;
use k256::ecdsa::SigningKey;
use k256::elliptic_curve::sec1::ToEncodedPoint;
use k256::PublicKey;
use num_bigint::BigUint;
use ripemd::Ripemd160;
use serde::Deserialize;
use serde_json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct WithSchema<T> {
    #[serde(rename = "$schema")]
    #[allow(dead_code)]
    schema: Option<String>,
    #[serde(flatten)]
    inner: T,
}

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

fn hash160(data: &[u8]) -> [u8; 20] {
    let sha = sha256(data);
    let mut hasher = Ripemd160::new();
    hasher.update(sha);
    hasher.finalize().into()
}

fn private_key_to_address(hex_key: &str, compressed: bool) -> Option<String> {
    let key_bytes = hex::decode(hex_key).ok()?;
    if key_bytes.len() != 32 {
        return None;
    }

    let signing_key = SigningKey::from_bytes((&key_bytes[..]).into()).ok()?;
    let public_key = PublicKey::from(signing_key.verifying_key());

    let pubkey_bytes = if compressed {
        public_key.to_sec1_bytes().to_vec()
    } else {
        public_key.to_encoded_point(false).as_bytes().to_vec()
    };

    let hash = hash160(&pubkey_bytes);

    let mut data = vec![0x00];
    data.extend_from_slice(&hash);
    let checksum = &sha256(&sha256(&data))[..4];
    data.extend_from_slice(checksum);

    Some(bs58::encode(data).into_string())
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

/// Validates WIF checksum and panics with detailed error if invalid.
/// This catches typos and corrupted WIF strings at build time.
fn validate_wif_checksum(wif: &str, puzzle_id: &str, wif_type: &str) {
    let decoded = match bs58::decode(wif).into_vec() {
        Ok(d) => d,
        Err(e) => panic!(
            "Puzzle '{}' has invalid Base58 in {} WIF '{}': {}",
            puzzle_id, wif_type, wif, e
        ),
    };

    if decoded.len() < 37 {
        panic!(
            "Puzzle '{}' has {} WIF '{}' that is too short (decoded {} bytes, need at least 37)",
            puzzle_id,
            wif_type,
            wif,
            decoded.len()
        );
    }

    let data = &decoded[..decoded.len() - 4];
    let checksum = &decoded[decoded.len() - 4..];
    let expected_checksum = &sha256(&sha256(data))[..4];

    if checksum != expected_checksum {
        panic!(
            "Puzzle '{}' has {} WIF '{}' with INVALID CHECKSUM\n\
             Expected: {:02x}{:02x}{:02x}{:02x}\n\
             Got:      {:02x}{:02x}{:02x}{:02x}\n\
             This is likely a typo when copying the WIF. Please verify the original source.",
            puzzle_id,
            wif_type,
            wif,
            expected_checksum[0],
            expected_checksum[1],
            expected_checksum[2],
            expected_checksum[3],
            checksum[0],
            checksum[1],
            checksum[2],
            checksum[3]
        );
    }
}

fn validate_wif_derives_address(
    wif: &str,
    expected_address: &str,
    puzzle_id: &str,
    wif_type: &str,
) {
    let hex_key = match wif_to_hex(wif) {
        Some(h) => h,
        None => {
            panic!(
                "Puzzle '{}' has {} WIF '{}' that cannot be decoded to hex",
                puzzle_id, wif_type, wif
            );
        }
    };

    let is_compressed = match wif.chars().next() {
        Some('K') | Some('L') => true,
        Some('5') => false,
        _ => panic!(
            "Puzzle '{}' has {} WIF '{}' with invalid prefix (expected '5', 'K', or 'L')",
            puzzle_id, wif_type, wif
        ),
    };
    let derived_address = match private_key_to_address(&hex_key, is_compressed) {
        Some(addr) => addr,
        None => {
            panic!(
                "Puzzle '{}' has {} WIF '{}' that cannot derive address from hex '{}'",
                puzzle_id, wif_type, wif, hex_key
            );
        }
    };

    if derived_address != expected_address {
        panic!(
            "Puzzle '{}' has {} WIF '{}' that derives WRONG ADDRESS\n\
             Expected: {}\n\
             Derived:  {}\n\
             This means the WIF does not match the puzzle address. Please verify the source.",
            puzzle_id, wif_type, wif, expected_address, derived_address
        );
    }
}

fn validate_encrypted_wif_derives_address(
    encrypted_wif: &str,
    passphrase: &str,
    expected_address: &str,
    puzzle_id: &str,
) {
    let (private_key_bytes, compressed) = match encrypted_wif.decrypt(passphrase) {
        Ok(result) => result,
        Err(e) => {
            panic!(
                "Puzzle '{}' has encrypted WIF '{}' that cannot be decrypted with the provided passphrase: {:?}\n\
                 The passphrase may be incorrect or the encrypted WIF may be invalid.",
                puzzle_id, encrypted_wif, e
            );
        }
    };

    let hex_key = hex::encode(private_key_bytes);
    let derived_address = match private_key_to_address(&hex_key, compressed) {
        Some(addr) => addr,
        None => {
            panic!(
                "Puzzle '{}' has encrypted WIF '{}' that cannot derive address after decryption",
                puzzle_id, encrypted_wif
            );
        }
    };

    if derived_address != expected_address {
        panic!(
            "Puzzle '{}' has encrypted WIF '{}' that derives WRONG ADDRESS after decryption\n\
             Expected: {}\n\
             Derived:  {}\n\
             This means the encrypted WIF or passphrase does not match the puzzle address.",
            puzzle_id, encrypted_wif, expected_address, derived_address
        );
    }
}

fn validate_hex_derives_address(hex_key: &str, expected_address: &str, puzzle_id: &str) {
    // Try both compressed and uncompressed formats
    let compressed_addr = private_key_to_address(hex_key, true);
    let uncompressed_addr = private_key_to_address(hex_key, false);

    let matches = compressed_addr
        .as_ref()
        .is_some_and(|a| a == expected_address)
        || uncompressed_addr
            .as_ref()
            .is_some_and(|a| a == expected_address);

    if !matches {
        let derived_compressed = compressed_addr.unwrap_or_else(|| "ERROR".to_string());
        let derived_uncompressed = uncompressed_addr.unwrap_or_else(|| "ERROR".to_string());
        panic!(
            "Puzzle '{}' has hex key '{}' that derives WRONG ADDRESS\n\
             Expected: {}\n\
             Derived (compressed):   {}\n\
             Derived (uncompressed): {}\n\
             This means the hex key does not match the puzzle address. Please verify the source.",
            puzzle_id, hex_key, expected_address, derived_compressed, derived_uncompressed
        );
    }
}

#[derive(Debug, Deserialize)]
struct TomlProfile {
    name: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct AuthorConfig {
    name: Option<String>,
    #[serde(default)]
    addresses: Vec<String>,
    #[serde(default)]
    profiles: Vec<TomlProfile>,
}

#[derive(Debug, Deserialize)]
struct SolverDefinition {
    name: Option<String>,
    #[serde(default)]
    addresses: Vec<String>,
    #[serde(default)]
    profiles: Vec<TomlProfile>,
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
struct TomlEntropySource {
    url: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TomlPassphrase {
    Known(String),
    Required(bool),
}

#[derive(Debug, Deserialize)]
struct TomlEntropy {
    hash: String,
    source: Option<TomlEntropySource>,
    passphrase: Option<TomlPassphrase>,
}

#[derive(Debug, Deserialize)]
struct TomlSeed {
    phrase: Option<String>,
    path: Option<String>,
    xpub: Option<String>,
    entropy: Option<TomlEntropy>,
}

#[derive(Debug, Deserialize)]
struct TomlShare {
    index: u8,
    data: String,
}

#[derive(Debug, Deserialize)]
struct TomlAssets {
    puzzle: Option<String>,
    solver: Option<String>,
    #[serde(default)]
    hints: Vec<String>,
    source_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TomlPubkey {
    value: String,
    format: String,
}

#[derive(Debug, Deserialize)]
struct TomlShares {
    threshold: u8,
    total: u8,
    #[serde(default)]
    shares: Vec<TomlShare>,
}

#[derive(Debug, Clone, Deserialize)]
struct TomlWif {
    encrypted: Option<String>,
    decrypted: Option<String>,
    passphrase: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TomlKey {
    hex: Option<String>,
    wif: Option<TomlWif>,
    seed: Option<TomlSeed>,
    mini: Option<String>,
    bits: Option<u16>,
    shares: Option<TomlShares>,
}

#[derive(Debug, Deserialize)]
struct Address {
    value: String,
    kind: String,
    hash160: Option<String>,
    witness_program: Option<String>,
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
    pubkey: Option<TomlPubkey>,
    start_date: Option<String>,
    solve_date: Option<String>,
    solve_time: Option<u64>,
    #[serde(default)]
    pre_genesis: bool,
    source_url: Option<String>,
    #[serde(default)]
    transactions: Vec<TomlTransaction>,
    solver: Option<String>,
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
    solver: Option<String>,
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
    pubkey: Option<TomlPubkey>,
    start_date: Option<String>,
    solve_date: Option<String>,
    solve_time: Option<u64>,
    source_url: Option<String>,
    #[serde(default)]
    transactions: Vec<TomlTransaction>,
    solver: Option<String>,
    assets: Option<TomlAssets>,
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
    pubkey: Option<TomlPubkey>,
    key: Option<TomlKey>,
    start_date: Option<String>,
    solve_date: Option<String>,
    solve_time: Option<u64>,
    source_url: Option<String>,
    #[serde(default)]
    transactions: Vec<TomlTransaction>,
    solver: Option<String>,
    assets: Option<TomlAssets>,
}

#[derive(Debug, Deserialize)]
struct BitapsMetadata {
    source_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BitimageMetadata {
    source_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BalletMetadata {
    source_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BitimageFile {
    author: Option<AuthorConfig>,
    metadata: Option<BitimageMetadata>,
    puzzles: Vec<BitimagePuzzle>,
}

#[derive(Debug, Deserialize)]
struct BitimagePuzzle {
    name: String,
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
    solver: Option<String>,
    assets: Option<TomlAssets>,
}

#[derive(Debug, Deserialize)]
struct BalletFile {
    author: Option<AuthorConfig>,
    metadata: Option<BalletMetadata>,
    puzzles: Vec<BalletPuzzle>,
}

#[derive(Debug, Deserialize)]
struct BalletPuzzle {
    name: String,
    address: Address,
    pubkey: Option<TomlPubkey>,
    status: String,
    prize: Option<f64>,
    key: Option<TomlKey>,
    start_date: Option<String>,
    solve_date: Option<String>,
    solve_time: Option<u64>,
    source_url: Option<String>,
    #[serde(default)]
    transactions: Vec<TomlTransaction>,
    solver: Option<String>,
    assets: Option<TomlAssets>,
}

#[derive(Debug, Deserialize)]
struct BitapsFile {
    author: Option<AuthorConfig>,
    metadata: Option<BitapsMetadata>,
    puzzle: BitapsPuzzle,
}

#[derive(Debug, Deserialize)]
struct BitapsPuzzle {
    address: Address,
    status: String,
    prize: Option<f64>,
    pubkey: Option<TomlPubkey>,
    key: Option<TomlKey>,
    start_date: Option<String>,
    solve_date: Option<String>,
    solve_time: Option<u64>,
    source_url: Option<String>,
    #[serde(default)]
    transactions: Vec<TomlTransaction>,
    solver: Option<String>,
}

fn format_hash160(address: &Address, chain: &str, puzzle_id: &str) -> String {
    let requires_hash160 = matches!(address.kind.as_str(), "p2pkh" | "p2wpkh")
        && matches!(chain, "bitcoin" | "litecoin");
    if requires_hash160 && address.hash160.is_none() {
        panic!(
            "Puzzle '{}' ({}) requires hash160 but none provided",
            puzzle_id, address.kind
        );
    }
    match &address.hash160 {
        Some(h) => format!("Some(\"{}\")", h),
        None => "None".to_string(),
    }
}

fn format_pubkey(pubkey: &Option<TomlPubkey>, puzzle_id: &str) -> String {
    match pubkey {
        Some(pk) => {
            let format = match pk.format.as_str() {
                "compressed" => "PubkeyFormat::Compressed",
                "uncompressed" => "PubkeyFormat::Uncompressed",
                _ => panic!(
                    "Invalid pubkey format '{}' for puzzle {}",
                    pk.format, puzzle_id
                ),
            };
            format!(
                "Some(Pubkey {{ value: \"{}\", format: {} }})",
                pk.value, format
            )
        }
        None => "None".to_string(),
    }
}

fn format_witness_program(address: &Address, puzzle_id: &str) -> String {
    if address.kind == "p2wsh" || address.kind == "p2tr" {
        match &address.witness_program {
            Some(wp) => {
                if hex::decode(wp).map(|b| b.len()).unwrap_or(0) != 32 {
                    panic!(
                        "Puzzle '{}' ({}) witness_program must be 64 hex chars (32 bytes), got '{}'",
                        puzzle_id, address.kind, wp
                    );
                }
                format!("Some(\"{}\")", wp)
            }
            None => panic!(
                "Puzzle '{}' ({}) requires witness_program but none provided",
                puzzle_id, address.kind
            ),
        }
    } else {
        "None".to_string()
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

fn generate_profiles_code(profiles: &[TomlProfile]) -> String {
    if profiles.is_empty() {
        "&[]".to_string()
    } else {
        let profs: Vec<String> = profiles
            .iter()
            .map(|p| format!("Profile {{ name: \"{}\", url: \"{}\" }}", p.name, p.url))
            .collect();
        format!("&[{}]", profs.join(", "))
    }
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
            let profiles = generate_profiles_code(&a.profiles);
            format!(
                "static AUTHOR: Author = Author {{\n    name: {},\n    addresses: {},\n    profiles: {},\n}};\n",
                name, addresses, profiles
            )
        }
        None => {
            "static AUTHOR: Author = Author {\n    name: None,\n    addresses: &[],\n    profiles: &[],\n};\n".to_string()
        }
    }
}

fn generate_solver_code(
    solver_id: &Option<String>,
    solvers: &HashMap<String, SolverDefinition>,
) -> String {
    match solver_id {
        Some(id) => {
            let solver = solvers
                .get(id)
                .unwrap_or_else(|| panic!("Unknown solver: {}", id));
            let name = match &solver.name {
                Some(n) => format!("Some(\"{}\")", n),
                None => "None".to_string(),
            };
            let addresses = if solver.addresses.is_empty() {
                "&[]".to_string()
            } else {
                let addrs: Vec<String> = solver
                    .addresses
                    .iter()
                    .map(|addr| format!("\"{}\"", addr))
                    .collect();
                format!("&[{}]", addrs.join(", "))
            };
            let profiles = generate_profiles_code(&solver.profiles);
            format!(
                "Some(Solver {{ name: {}, addresses: {}, profiles: {} }})",
                name, addresses, profiles
            )
        }
        None => "None".to_string(),
    }
}

fn generate_entropy_code(entropy: &Option<TomlEntropy>) -> String {
    match entropy {
        Some(e) => {
            let source = match &e.source {
                Some(s) => {
                    let url = match &s.url {
                        Some(u) => format!("Some(\"{}\")", u),
                        None => "None".to_string(),
                    };
                    let description = match &s.description {
                        Some(d) => format!("Some(\"{}\")", d),
                        None => "None".to_string(),
                    };
                    format!(
                        "Some(EntropySource {{ url: {}, description: {} }})",
                        url, description
                    )
                }
                None => "None".to_string(),
            };
            let passphrase = match &e.passphrase {
                Some(TomlPassphrase::Known(s)) => {
                    format!("Some(Passphrase::Known(\"{}\"))", s)
                }
                Some(TomlPassphrase::Required(true)) => "Some(Passphrase::Required)".to_string(),
                Some(TomlPassphrase::Required(false)) | None => "None".to_string(),
            };
            format!(
                "Some(Entropy {{ hash: \"{}\", source: {}, passphrase: {} }})",
                e.hash, source, passphrase
            )
        }
        None => "None".to_string(),
    }
}

fn generate_key_code(key: &Option<TomlKey>, puzzle_id: &str, expected_address: &str) -> String {
    match key {
        Some(k) => generate_key_code_required(k, puzzle_id, expected_address),
        None => "None".to_string(),
    }
}

fn generate_wif_code(wif: &Option<TomlWif>, puzzle_id: &str, expected_address: &str) -> String {
    match wif {
        Some(w) => {
            if let Some(e) = &w.encrypted {
                validate_wif_checksum(e, puzzle_id, "encrypted");
                // Only validate derivation if passphrase is known (unsolved puzzles may lack passphrase)
                if let Some(p) = &w.passphrase {
                    validate_encrypted_wif_derives_address(e, p, expected_address, puzzle_id);
                }
            }
            if let Some(d) = &w.decrypted {
                validate_wif_checksum(d, puzzle_id, "decrypted");
                validate_wif_derives_address(d, expected_address, puzzle_id, "decrypted");
            }

            let encrypted = match &w.encrypted {
                Some(e) => format!("Some(\"{}\")", e),
                None => "None".to_string(),
            };
            let decrypted = match &w.decrypted {
                Some(d) => format!("Some(\"{}\")", d),
                None => "None".to_string(),
            };
            let passphrase = match &w.passphrase {
                Some(p) => format!("Some(\"{}\")", p),
                None => "None".to_string(),
            };
            format!(
                "Some(Wif {{ encrypted: {}, decrypted: {}, passphrase: {} }})",
                encrypted, decrypted, passphrase
            )
        }
        None => "None".to_string(),
    }
}

fn generate_key_code_required(key: &TomlKey, puzzle_id: &str, expected_address: &str) -> String {
    let decrypted_wif = key.wif.as_ref().and_then(|w| w.decrypted.as_ref());

    // Validate: bits is required when hex or decrypted wif exists
    let has_private_key = key.hex.is_some() || decrypted_wif.is_some();
    if has_private_key && key.bits.is_none() {
        panic!("Key has hex or decrypted wif but missing required 'bits' field");
    }

    let (hex_val, derived_decrypted) = match (&key.hex, decrypted_wif) {
        (Some(h), Some(_)) => (Some(h.clone()), None),
        (Some(h), None) => {
            let derived_wif = hex_to_wif(h, true);
            (Some(h.clone()), derived_wif)
        }
        (None, Some(w)) => {
            let derived_hex = wif_to_hex(w);
            (derived_hex, None)
        }
        (None, None) => (None, None),
    };

    let hex = match &hex_val {
        Some(h) => {
            validate_hex_derives_address(h, expected_address, puzzle_id);
            format!("Some(\"{}\")", h)
        }
        None => "None".to_string(),
    };

    let wif_code = if derived_decrypted.is_some() {
        let mut wif_with_derived = key.wif.clone().unwrap_or(TomlWif {
            encrypted: None,
            decrypted: None,
            passphrase: None,
        });
        wif_with_derived.decrypted = derived_decrypted;
        generate_wif_code(&Some(wif_with_derived), puzzle_id, expected_address)
    } else {
        generate_wif_code(&key.wif, puzzle_id, expected_address)
    };

    let seed = match &key.seed {
        Some(s) => {
            let phrase = match &s.phrase {
                Some(p) => format!("Some(\"{}\")", p),
                None => "None".to_string(),
            };
            let path = match &s.path {
                Some(p) => format!("Some(\"{}\")", p),
                None => "None".to_string(),
            };
            let xpub = match &s.xpub {
                Some(x) => format!("Some(\"{}\")", x),
                None => "None".to_string(),
            };
            let entropy = generate_entropy_code(&s.entropy);
            format!(
                "Some(Seed {{ phrase: {}, path: {}, xpub: {}, entropy: {} }})",
                phrase, path, xpub, entropy
            )
        }
        None => "None".to_string(),
    };
    let mini = match &key.mini {
        Some(m) => format!("Some(\"{}\")", m),
        None => "None".to_string(),
    };
    let bits = match key.bits {
        Some(b) => format!("Some({})", b),
        None => "None".to_string(),
    };
    let shares = generate_shares_code(&key.shares);
    format!(
        "Some(Key {{ hex: {}, wif: {}, seed: {}, mini: {}, bits: {}, shares: {} }})",
        hex, wif_code, seed, mini, bits, shares
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

fn generate_shares_code(shares: &Option<TomlShares>) -> String {
    match shares {
        Some(s) => {
            let shares_list: Vec<String> = s
                .shares
                .iter()
                .map(|share| {
                    format!(
                        "Share {{ index: {}, data: \"{}\" }}",
                        share.index, share.data
                    )
                })
                .collect();
            let shares_arr = if shares_list.is_empty() {
                "&[]".to_string()
            } else {
                format!("&[{}]", shares_list.join(", "))
            };
            format!(
                "Some(Shares {{ threshold: {}, total: {}, shares: {} }})",
                s.threshold, s.total, shares_arr
            )
        }
        None => "None".to_string(),
    }
}

fn validate_asset_file(collection: &str, asset_path: &str, puzzle_id: &str) {
    // Skip validation if assets directory doesn't exist (e.g., crates.io package)
    if !Path::new("assets").exists() {
        return;
    }
    let full_path = format!("assets/{}/{}", collection, asset_path);
    if !Path::new(&full_path).exists() {
        panic!(
            "Asset file not found for puzzle '{}': {}",
            puzzle_id, full_path
        );
    }
}

fn generate_assets_code(assets: &Option<TomlAssets>, collection: &str, puzzle_id: &str) -> String {
    match assets {
        Some(a) => {
            if let Some(ref puzzle) = a.puzzle {
                validate_asset_file(collection, puzzle, puzzle_id);
            }
            if let Some(ref solver) = a.solver {
                validate_asset_file(collection, solver, puzzle_id);
            }
            for hint in &a.hints {
                validate_asset_file(collection, hint, puzzle_id);
            }

            let puzzle = match &a.puzzle {
                Some(p) => format!("Some(\"{}\")", p),
                None => "None".to_string(),
            };
            let solver = match &a.solver {
                Some(s) => format!("Some(\"{}\")", s),
                None => "None".to_string(),
            };
            let hints = if a.hints.is_empty() {
                "&[]".to_string()
            } else {
                let hints_list: Vec<String> =
                    a.hints.iter().map(|h| format!("\"{}\"", h)).collect();
                format!("&[{}]", hints_list.join(", "))
            };
            let source_url = match &a.source_url {
                Some(u) => format!("Some(\"{}\")", u),
                None => "None".to_string(),
            };
            format!(
                "Some(Assets {{ puzzle: {}, solver: {}, hints: {}, source_url: {} }})",
                puzzle, solver, hints, source_url
            )
        }
        None => "None".to_string(),
    }
}

fn load_solvers() -> HashMap<String, SolverDefinition> {
    let mut content =
        fs::read_to_string("data/solvers.jsonc").expect("Failed to read data/solvers.jsonc");
    strip(&mut content).expect("Failed to strip comments from solvers.jsonc");
    let mut value: serde_json::Value =
        serde_json::from_str(&content).expect("Failed to parse solvers.jsonc");
    if let Some(obj) = value.as_object_mut() {
        obj.remove("$schema");
    }
    let solvers: HashMap<String, SolverDefinition> =
        serde_json::from_value(value).expect("Failed to deserialize solvers");
    solvers
}

fn generate_data_version(out_dir: &str) {
    let dest_path = Path::new(out_dir).join("data_version.rs");

    let git_hash = std::process::Command::new("git")
        .args(["rev-parse", "--short=7", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let build_date = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let data_files = [
        "data/b1000.jsonc",
        "data/ballet.jsonc",
        "data/bitaps.jsonc",
        "data/bitimage.jsonc",
        "data/gsmg.jsonc",
        "data/hash_collision.jsonc",
        "data/solvers.jsonc",
        "data/zden.jsonc",
    ];

    let mut hasher = Sha256::new();
    for file in &data_files {
        if let Ok(content) = fs::read(file) {
            hasher.update(&content);
        }
    }
    let data_hash = hex::encode(&hasher.finalize()[..4]);

    let pkg_version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "unknown".to_string());
    let full_version = format!(
        "{} (data: {}-{}, {})",
        pkg_version, git_hash, data_hash, build_date
    );

    let output = format!(
        r#"pub const GIT_HASH: &str = "{}";
pub const BUILD_DATE: &str = "{}";
pub const DATA_HASH: &str = "{}";
pub const FULL_VERSION: &str = "{}";
"#,
        git_hash, build_date, data_hash, full_version
    );

    fs::write(&dest_path, output).expect("Failed to write data_version.rs");
}

fn main() {
    println!("cargo:rerun-if-changed=data/b1000.jsonc");
    println!("cargo:rerun-if-changed=data/hash_collision.jsonc");
    println!("cargo:rerun-if-changed=data/gsmg.jsonc");
    println!("cargo:rerun-if-changed=data/zden.jsonc");
    println!("cargo:rerun-if-changed=data/bitaps.jsonc");
    println!("cargo:rerun-if-changed=data/bitimage.jsonc");
    println!("cargo:rerun-if-changed=data/ballet.jsonc");
    println!("cargo:rerun-if-changed=data/solvers.jsonc");
    println!("cargo:rerun-if-changed=assets");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads");

    let out_dir = env::var("OUT_DIR").unwrap();
    let solvers = load_solvers();

    generate_data_version(&out_dir);
    generate_b1000(&out_dir, &solvers);
    generate_hash_collision(&out_dir, &solvers);
    generate_gsmg(&out_dir, &solvers);
    generate_zden(&out_dir, &solvers);
    generate_bitaps(&out_dir, &solvers);
    generate_bitimage(&out_dir, &solvers);
    generate_ballet(&out_dir, &solvers);
}

fn generate_b1000(out_dir: &str, solvers: &HashMap<String, SolverDefinition>) {
    let dest_path = Path::new(out_dir).join("b1000_data.rs");

    let mut content =
        fs::read_to_string("data/b1000.jsonc").expect("Failed to read data/b1000.jsonc");
    strip(&mut content).expect("Failed to strip comments from b1000.jsonc");
    let wrapped: WithSchema<Btc1000File> =
        serde_json::from_str(&content).expect("Failed to parse b1000.jsonc");
    let data = wrapped.inner;

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

        let pubkey = format_pubkey(&puzzle.pubkey, &bits.to_string());

        let puzzle_id = format!("b1000/{}", bits);
        let key = generate_key_code_required(&puzzle.key, &puzzle_id, &puzzle.address.value);

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
        let witness_program = format_witness_program(&puzzle.address, &format!("b1000/{}", bits));
        let redeem_script = generate_redeem_script_code(&puzzle.address.redeem_script);

        let transactions = generate_transactions_code(&puzzle.transactions);
        let solver = generate_solver_code(&puzzle.solver, solvers);

        output.push_str(&format!(
            r#"    Puzzle {{
        id: "b1000/{}",
        chain: Chain::Bitcoin,
        address: Address {{
            value: "{}",
            chain: Chain::Bitcoin,
            kind: "{}",
            hash160: {},
            witness_program: {},
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
        assets: None,
    }},
"#,
            bits,
            puzzle.address.value,
            puzzle.address.kind,
            hash160,
            witness_program,
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

fn generate_hash_collision(out_dir: &str, solvers: &HashMap<String, SolverDefinition>) {
    let dest_path = Path::new(out_dir).join("hash_collision_data.rs");

    let mut content = fs::read_to_string("data/hash_collision.jsonc")
        .expect("Failed to read data/hash_collision.jsonc");
    strip(&mut content).expect("Failed to strip comments from hash_collision.jsonc");
    let wrapped: WithSchema<HashCollisionFile> =
        serde_json::from_str(&content).expect("Failed to parse hash_collision.jsonc");
    let data = wrapped.inner;

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
        let witness_program =
            format_witness_program(&puzzle.address, &format!("hash_collision/{}", puzzle.name));
        let redeem_script = generate_redeem_script_code(&puzzle.address.redeem_script);

        let transactions = generate_transactions_code(&puzzle.transactions);
        let solver = generate_solver_code(&puzzle.solver, solvers);

        output.push_str(&format!(
            r#"    Puzzle {{
        id: "hash_collision/{}",
        chain: Chain::Bitcoin,
        address: Address {{
            value: "{}",
            chain: Chain::Bitcoin,
            kind: "{}",
            hash160: {},
            witness_program: {},
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
        assets: None,
    }},
"#,
            puzzle.name,
            puzzle.address.value,
            puzzle.address.kind,
            hash160,
            witness_program,
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

fn generate_gsmg(out_dir: &str, solvers: &HashMap<String, SolverDefinition>) {
    let dest_path = Path::new(out_dir).join("gsmg_data.rs");

    let mut content =
        fs::read_to_string("data/gsmg.jsonc").expect("Failed to read data/gsmg.jsonc");
    strip(&mut content).expect("Failed to strip comments from gsmg.jsonc");
    let wrapped: WithSchema<GsmgFile> =
        serde_json::from_str(&content).expect("Failed to parse gsmg.jsonc");
    let data = wrapped.inner;

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

    let pubkey = format_pubkey(&puzzle.pubkey, "gsmg");

    let hash160 = format_hash160(&puzzle.address, "bitcoin", "gsmg");
    let witness_program = format_witness_program(&puzzle.address, "gsmg");
    let redeem_script = generate_redeem_script_code(&puzzle.address.redeem_script);

    let transactions = generate_transactions_code(&puzzle.transactions);
    let solver = generate_solver_code(&puzzle.solver, solvers);
    let assets = generate_assets_code(&puzzle.assets, "gsmg", "gsmg");

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
        witness_program: {},
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
    assets: {},
}};
"#,
        puzzle.address.value,
        puzzle.address.kind,
        hash160,
        witness_program,
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
        assets,
    ));

    fs::write(&dest_path, output).expect("Failed to write gsmg_data.rs");
}

fn generate_zden(out_dir: &str, solvers: &HashMap<String, SolverDefinition>) {
    let dest_path = Path::new(out_dir).join("zden_data.rs");

    let mut content =
        fs::read_to_string("data/zden.jsonc").expect("Failed to read data/zden.jsonc");
    strip(&mut content).expect("Failed to strip comments from zden.jsonc");
    let wrapped: WithSchema<ZdenFile> =
        serde_json::from_str(&content).expect("Failed to parse zden.jsonc");
    let data = wrapped.inner;

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
        let witness_program =
            format_witness_program(&puzzle.address, &format!("zden/{}", puzzle.name));
        let redeem_script = generate_redeem_script_code(&puzzle.address.redeem_script);
        let puzzle_id = format!("zden/{}", puzzle.name);
        let key = generate_key_code(&puzzle.key, &puzzle_id, &puzzle.address.value);

        let transactions = generate_transactions_code(&puzzle.transactions);
        let solver = generate_solver_code(&puzzle.solver, solvers);
        let assets = generate_assets_code(&puzzle.assets, "zden", &format!("zden/{}", puzzle.name));

        let pubkey = format_pubkey(&puzzle.pubkey, &puzzle.name);

        output.push_str(&format!(
            r#"    Puzzle {{
        id: "zden/{}",
        chain: {},
        address: Address {{
            value: "{}",
            chain: {},
            kind: "{}",
            hash160: {},
            witness_program: {},
            redeem_script: {},
        }},
        status: {},
        pubkey: {},
        key: {},
        prize: {},
        start_date: {},
        solve_date: {},
        solve_time: {},
        pre_genesis: false,
        source_url: {},
        transactions: {},
        solver: {},
        assets: {},
    }},
"#,
            puzzle.name,
            chain,
            puzzle.address.value,
            chain,
            puzzle.address.kind,
            hash160,
            witness_program,
            redeem_script,
            status,
            pubkey,
            key,
            prize,
            start_date,
            solve_date,
            solve_time,
            source_url,
            transactions,
            solver,
            assets,
        ));
    }

    output.push_str("];\n");

    fs::write(&dest_path, output).expect("Failed to write zden_data.rs");
}

fn generate_bitaps(out_dir: &str, solvers: &HashMap<String, SolverDefinition>) {
    let dest_path = Path::new(out_dir).join("bitaps_data.rs");

    let mut content =
        fs::read_to_string("data/bitaps.jsonc").expect("Failed to read data/bitaps.jsonc");
    strip(&mut content).expect("Failed to strip comments from bitaps.jsonc");
    let wrapped: WithSchema<BitapsFile> =
        serde_json::from_str(&content).expect("Failed to parse bitaps.jsonc");
    let data = wrapped.inner;

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

    let pubkey = format_pubkey(&puzzle.pubkey, "bitaps");

    let hash160 = format_hash160(&puzzle.address, "bitcoin", "bitaps");
    let witness_program = format_witness_program(&puzzle.address, "bitaps");
    let redeem_script = generate_redeem_script_code(&puzzle.address.redeem_script);
    let key = generate_key_code(&puzzle.key, "bitaps", &puzzle.address.value);

    let transactions = generate_transactions_code(&puzzle.transactions);
    let solver = generate_solver_code(&puzzle.solver, solvers);

    let mut output = String::new();
    output.push_str(&generate_author_code(&data.author));
    output.push('\n');
    output.push_str(&format!(
        r#"static PUZZLE: Puzzle = Puzzle {{
    id: "bitaps",
    chain: Chain::Bitcoin,
    address: Address {{
        value: "{}",
        chain: Chain::Bitcoin,
        kind: "{}",
        hash160: {},
        witness_program: {},
        redeem_script: {},
    }},
    status: {},
    pubkey: {},
    key: {},
    prize: {},
    start_date: {},
    solve_date: {},
    solve_time: {},
    pre_genesis: false,
    source_url: {},
    transactions: {},
    solver: {},
    assets: None,
}};
"#,
        puzzle.address.value,
        puzzle.address.kind,
        hash160,
        witness_program,
        redeem_script,
        status,
        pubkey,
        key,
        prize,
        start_date,
        solve_date,
        solve_time,
        source_url,
        transactions,
        solver,
    ));

    fs::write(&dest_path, output).expect("Failed to write bitaps_data.rs");
}

fn generate_bitimage(out_dir: &str, solvers: &HashMap<String, SolverDefinition>) {
    let dest_path = Path::new(out_dir).join("bitimage_data.rs");

    let mut content =
        fs::read_to_string("data/bitimage.jsonc").expect("Failed to read data/bitimage.jsonc");
    strip(&mut content).expect("Failed to strip comments from bitimage.jsonc");
    let wrapped: WithSchema<BitimageFile> =
        serde_json::from_str(&content).expect("Failed to parse bitimage.jsonc");
    let data = wrapped.inner;

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
            "bitcoin",
            &format!("bitimage/{}", puzzle.name),
        );
        let witness_program =
            format_witness_program(&puzzle.address, &format!("bitimage/{}", puzzle.name));
        let redeem_script = generate_redeem_script_code(&puzzle.address.redeem_script);
        let puzzle_id = format!("bitimage/{}", puzzle.name);
        let key = generate_key_code(&puzzle.key, &puzzle_id, &puzzle.address.value);

        let transactions = generate_transactions_code(&puzzle.transactions);
        let solver = generate_solver_code(&puzzle.solver, solvers);
        let assets = generate_assets_code(
            &puzzle.assets,
            "bitimage",
            &format!("bitimage/{}", puzzle.name),
        );

        output.push_str(&format!(
            r#"    Puzzle {{
        id: "bitimage/{}",
        chain: Chain::Bitcoin,
        address: Address {{
            value: "{}",
            chain: Chain::Bitcoin,
            kind: "{}",
            hash160: {},
            witness_program: {},
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
        assets: {},
    }},
"#,
            puzzle.name,
            puzzle.address.value,
            puzzle.address.kind,
            hash160,
            witness_program,
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
            assets,
        ));
    }

    output.push_str("];\n");

    fs::write(&dest_path, output).expect("Failed to write bitimage_data.rs");
}

fn generate_ballet(out_dir: &str, solvers: &HashMap<String, SolverDefinition>) {
    let dest_path = Path::new(out_dir).join("ballet_data.rs");

    let mut content =
        fs::read_to_string("data/ballet.jsonc").expect("Failed to read data/ballet.jsonc");
    strip(&mut content).expect("Failed to strip comments from ballet.jsonc");
    let wrapped: WithSchema<BalletFile> =
        serde_json::from_str(&content).expect("Failed to parse ballet.jsonc");
    let data = wrapped.inner;

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
            "bitcoin",
            &format!("ballet/{}", puzzle.name),
        );
        let witness_program =
            format_witness_program(&puzzle.address, &format!("ballet/{}", puzzle.name));
        let redeem_script = generate_redeem_script_code(&puzzle.address.redeem_script);
        let puzzle_id = format!("ballet/{}", puzzle.name);
        let key = generate_key_code(&puzzle.key, &puzzle_id, &puzzle.address.value);

        let transactions = generate_transactions_code(&puzzle.transactions);
        let solver = generate_solver_code(&puzzle.solver, solvers);
        let assets =
            generate_assets_code(&puzzle.assets, "ballet", &format!("ballet/{}", puzzle.name));

        let pubkey = format_pubkey(&puzzle.pubkey, &puzzle.name);

        output.push_str(&format!(
            r#"    Puzzle {{
        id: "ballet/{}",
        chain: Chain::Bitcoin,
        address: Address {{
            value: "{}",
            chain: Chain::Bitcoin,
            kind: "{}",
            hash160: {},
            witness_program: {},
            redeem_script: {},
        }},
        status: {},
        pubkey: {},
        key: {},
        prize: {},
        start_date: {},
        solve_date: {},
        solve_time: {},
        pre_genesis: false,
        source_url: {},
        transactions: {},
        solver: {},
        assets: {},
    }},
"#,
            puzzle.name,
            puzzle.address.value,
            puzzle.address.kind,
            hash160,
            witness_program,
            redeem_script,
            status,
            pubkey,
            key,
            prize,
            start_date,
            solve_date,
            solve_time,
            source_url,
            transactions,
            solver,
            assets,
        ));
    }

    output.push_str("];\n");

    fs::write(&dest_path, output).expect("Failed to write ballet_data.rs");
}
