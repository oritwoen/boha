//! Extract public keys from claim/sweep transactions for BTC/LTC/DCR/ETH
//!
//! This script fetches transaction details from blockchain APIs and extracts
//! the public key used to sign the claim/sweep transaction.
//!
//! Usage:
//!   cargo run -p scripts --bin extract-pubkey              # Dry-run (shows what would change)
//!   cargo run -p scripts --bin extract-pubkey --apply      # Actually update JSONC files
//!   cargo run -p scripts --bin extract-pubkey --collection zden  # Filter by collection

use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

const RATE_LIMIT_DELAY: Duration = Duration::from_millis(500);

fn cache_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/cache")
}

fn read_cache<T: DeserializeOwned>(cache_key: &str) -> Option<T> {
    let cache_path = cache_dir().join(format!("{}.json", cache_key));
    let content = std::fs::read_to_string(&cache_path).ok()?;
    serde_json::from_str(&content).ok()
}

fn write_cache<T: Serialize>(cache_key: &str, data: &T) {
    let cache_dir = cache_dir();
    let _ = std::fs::create_dir_all(&cache_dir);
    let cache_path = cache_dir.join(format!("{}.json", cache_key));
    let _ = std::fs::write(&cache_path, serde_json::to_string_pretty(data).unwrap_or_default());
}

// ============================================================================
// API Response Types
// ============================================================================

#[derive(Debug, Deserialize, Serialize)]
struct MempoolTxResponse {
    vin: Vec<MempoolVin>,
}

#[derive(Debug, Deserialize, Serialize)]
struct MempoolVin {
    scriptsig: Option<String>,
    witness: Option<Vec<String>>,
    prevout: Option<MempoolPrevout>,
}

#[derive(Debug, Deserialize, Serialize)]
struct MempoolPrevout {
    #[allow(dead_code)]
    scriptpubkey_type: Option<String>,
    scriptpubkey_address: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct DcrdataTxResponse {
    vin: Vec<DcrdataVin>,
}

#[derive(Debug, Deserialize, Serialize)]
struct DcrdataVin {
    #[serde(rename = "scriptSig")]
    script_sig: Option<DcrdataScriptSig>,
}

#[derive(Debug, Deserialize, Serialize)]
struct DcrdataScriptSig {
    hex: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct EtherscanTxResponse {
    result: EtherscanTxResult,
}

#[derive(Debug, Deserialize, Serialize)]
struct EtherscanTxResult {
    from: String,
    #[serde(rename = "type")]
    tx_type: Option<String>,
    nonce: String,
    #[serde(rename = "gasPrice")]
    gas_price: String,
    gas: String,
    to: Option<String>,
    value: String,
    input: String,
    v: String,
    r: String,
    s: String,
}

// ============================================================================
// JSONC Types (for parsing puzzle data)
// ============================================================================

#[derive(Debug, Deserialize)]
struct Collection {
    puzzles: Option<Vec<Puzzle>>,
    puzzle: Option<Puzzle>,
}

#[derive(Debug, Clone, Deserialize)]
struct Puzzle {
    name: Option<String>,
    chain: Option<String>,
    address: Address,
    status: String,
    pubkey: Option<Pubkey>,
    transactions: Option<Vec<Transaction>>,
    key: Option<PuzzleKey>,
}

#[derive(Debug, Clone, Deserialize)]
struct PuzzleKey {
    bits: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
struct Address {
    value: String,
    kind: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Pubkey {
    value: String,
    format: String,
}

#[derive(Debug, Clone, Deserialize)]
struct Transaction {
    #[serde(rename = "type")]
    tx_type: String,
    txid: String,
}

// ============================================================================
// Pubkey Extraction
// ============================================================================

fn extract_pubkey_from_scriptsig(scriptsig: &str) -> Option<(String, String)> {
    let bytes = hex::decode(scriptsig).ok()?;
    if bytes.len() < 34 {
        return None;
    }

    // Try compressed pubkey (33 bytes, prefix 0x02 or 0x03)
    if bytes.len() >= 34 {
        let potential_len_pos = bytes.len() - 34;
        if bytes[potential_len_pos] == 0x21 {
            let pubkey = &bytes[potential_len_pos + 1..];
            if pubkey.len() == 33 && (pubkey[0] == 0x02 || pubkey[0] == 0x03) {
                return Some((hex::encode(pubkey), "compressed".to_string()));
            }
        }
    }

    // Try uncompressed pubkey (65 bytes, prefix 0x04)
    if bytes.len() >= 66 {
        let potential_len_pos = bytes.len() - 66;
        if bytes[potential_len_pos] == 0x41 {
            let pubkey = &bytes[potential_len_pos + 1..];
            if pubkey.len() == 65 && pubkey[0] == 0x04 {
                return Some((hex::encode(pubkey), "uncompressed".to_string()));
            }
        }
    }

    None
}

/// Extract pubkey from SegWit witness data
/// For P2WPKH, witness = [signature, pubkey]
fn extract_pubkey_from_witness(witness: &[String]) -> Option<(String, String)> {
    if witness.len() < 2 {
        return None;
    }

    let pubkey_hex = &witness[1];
    let pubkey_bytes = hex::decode(pubkey_hex).ok()?;

    let format = match pubkey_bytes.len() {
        33 if pubkey_bytes[0] == 0x02 || pubkey_bytes[0] == 0x03 => "compressed",
        65 if pubkey_bytes[0] == 0x04 => "uncompressed",
        _ => return None,
    };

    Some((pubkey_hex.clone(), format.to_string()))
}

async fn fetch_btc_pubkey(
    client: &Client,
    txid: &str,
    address: &str,
    chain: &str,
) -> Result<Option<(String, String)>, Box<dyn std::error::Error>> {
    let cache_key = format!("{}-{}", chain, txid);
    
    let response: MempoolTxResponse = if let Some(cached) = read_cache(&cache_key) {
        cached
    } else {
        let base_url = match chain {
            "litecoin" => "https://litecoinspace.org/api/tx",
            _ => "https://mempool.space/api/tx",
        };
        let url = format!("{}/{}", base_url, txid);
        tokio::time::sleep(RATE_LIMIT_DELAY).await;
        let data: MempoolTxResponse = client.get(&url).send().await?.json().await?;
        write_cache(&cache_key, &data);
        data
    };

    // Find the vin that spends from our address
    for vin in &response.vin {
        let is_our_input = vin
            .prevout
            .as_ref()
            .and_then(|p| p.scriptpubkey_address.as_ref())
            .map(|a| a == address)
            .unwrap_or(false);

        if !is_our_input {
            continue;
        }

        // Check witness first (SegWit)
        if let Some(witness) = &vin.witness {
            if !witness.is_empty() && !witness[0].is_empty() {
                if let Some(result) = extract_pubkey_from_witness(witness) {
                    return Ok(Some(result));
                }
            }
        }

        // Then check scriptsig (legacy P2PKH)
        if let Some(scriptsig) = &vin.scriptsig {
            if !scriptsig.is_empty() {
                if let Some(result) = extract_pubkey_from_scriptsig(scriptsig) {
                    return Ok(Some(result));
                }
            }
        }
    }

    Ok(None)
}

async fn fetch_dcr_pubkey(
    client: &Client,
    txid: &str,
) -> Result<Option<(String, String)>, Box<dyn std::error::Error>> {
    let cache_key = format!("decred-{}", txid);
    
    let response: DcrdataTxResponse = if let Some(cached) = read_cache(&cache_key) {
        cached
    } else {
        let url = format!("https://dcrdata.decred.org/api/tx/{}", txid);
        tokio::time::sleep(RATE_LIMIT_DELAY).await;
        let data: DcrdataTxResponse = client.get(&url).send().await?.json().await?;
        write_cache(&cache_key, &data);
        data
    };

    // DCR uses same scriptsig format as BTC
    for vin in &response.vin {
        if let Some(script_sig) = &vin.script_sig {
            if let Some(result) = extract_pubkey_from_scriptsig(&script_sig.hex) {
                return Ok(Some(result));
            }
        }
    }

    Ok(None)
}

fn rlp_encode_length(len: usize, offset: u8) -> Vec<u8> {
    if len < 56 {
        vec![offset + len as u8]
    } else {
        let len_bytes = len.to_be_bytes();
        let len_bytes = len_bytes.iter().skip_while(|&&b| b == 0).copied().collect::<Vec<_>>();
        let mut result = vec![offset + 55 + len_bytes.len() as u8];
        result.extend(len_bytes);
        result
    }
}

fn rlp_encode_bytes(data: &[u8]) -> Vec<u8> {
    if data.len() == 1 && data[0] < 0x80 {
        data.to_vec()
    } else {
        let mut result = rlp_encode_length(data.len(), 0x80);
        result.extend(data);
        result
    }
}

fn rlp_encode_list(items: &[Vec<u8>]) -> Vec<u8> {
    let payload: Vec<u8> = items.iter().flat_map(|i| i.clone()).collect();
    let mut result = rlp_encode_length(payload.len(), 0xc0);
    result.extend(payload);
    result
}

fn parse_hex_u64(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap_or(0)
}

fn hex_to_bytes(s: &str) -> Vec<u8> {
    let s = s.trim_start_matches("0x");
    let s = if s.len() % 2 == 1 {
        format!("0{}", s)
    } else {
        s.to_string()
    };
    hex::decode(&s).unwrap_or_default()
}

fn encode_eth_signing_hash(tx: &EtherscanTxResult, chain_id: Option<u64>) -> [u8; 32] {
    use sha3::{Digest, Keccak256};

    let nonce = parse_hex_u64(&tx.nonce);
    let gas_price = hex_to_bytes(&tx.gas_price);
    let gas_limit = parse_hex_u64(&tx.gas);
    let to = tx.to.as_ref().map(|s| hex_to_bytes(s)).unwrap_or_default();
    let value = hex_to_bytes(&tx.value);
    let data = hex_to_bytes(&tx.input);

    let nonce_bytes = if nonce == 0 {
        vec![]
    } else {
        let bytes = nonce.to_be_bytes();
        bytes.iter().skip_while(|&&b| b == 0).copied().collect()
    };

    let gas_limit_bytes = if gas_limit == 0 {
        vec![]
    } else {
        let bytes = gas_limit.to_be_bytes();
        bytes.iter().skip_while(|&&b| b == 0).copied().collect()
    };

    let gas_price_trimmed: Vec<u8> = gas_price.iter().skip_while(|&&b| b == 0).copied().collect();
    let value_trimmed: Vec<u8> = value.iter().skip_while(|&&b| b == 0).copied().collect();

    let mut items = vec![
        rlp_encode_bytes(&nonce_bytes),
        rlp_encode_bytes(&gas_price_trimmed),
        rlp_encode_bytes(&gas_limit_bytes),
        rlp_encode_bytes(&to),
        rlp_encode_bytes(&value_trimmed),
        rlp_encode_bytes(&data),
    ];

    // EIP-155: append [chainId, 0, 0] for replay protection
    // Legacy (v=27/28): no chain_id in signing hash
    if let Some(chain_id) = chain_id {
        let chain_id_bytes = {
            let bytes = chain_id.to_be_bytes();
            bytes
                .iter()
                .skip_while(|&&b| b == 0)
                .copied()
                .collect::<Vec<_>>()
        };
        items.push(rlp_encode_bytes(&chain_id_bytes));
        items.push(rlp_encode_bytes(&[]));
        items.push(rlp_encode_bytes(&[]));
    }

    let rlp = rlp_encode_list(&items);
    let hash: [u8; 32] = Keccak256::digest(&rlp).into();
    hash
}

async fn fetch_eth_pubkey(
    client: &Client,
    txid: &str,
    api_key: &str,
    expected_address: &str,
) -> Result<Option<(String, String)>, Box<dyn std::error::Error>> {
    use secp256k1::{ecdsa::{RecoverableSignature, RecoveryId}, Message, Secp256k1};

    let secp = Secp256k1::new();
    let cache_key = format!("ethereum-{}", txid);

    let response: EtherscanTxResponse = if let Some(cached) = read_cache(&cache_key) {
        cached
    } else {
        let url = format!(
            "https://api.etherscan.io/v2/api?chainid=1&module=proxy&action=eth_getTransactionByHash&txhash={}&apikey={}",
            txid, api_key
        );
        tokio::time::sleep(RATE_LIMIT_DELAY).await;
        let data: EtherscanTxResponse = client.get(&url).send().await?.json().await?;
        write_cache(&cache_key, &data);
        data
    };
    let tx = &response.result;

    // Check if transaction is legacy (type "0x0" or absent)
    let tx_type = tx.tx_type.as_deref().unwrap_or("0x0");
    if tx_type != "0x0" {
        eprintln!("    Skipping non-legacy ETH tx type {}", tx_type);
        return Ok(None);
    }

    let v = parse_hex_u64(&tx.v);
    let r = hex_to_bytes(&tx.r);
    let s = hex_to_bytes(&tx.s);

    let (chain_id, recovery_id) = if v >= 35 {
        // EIP-155: v = chainId * 2 + 35 + {0,1}
        let chain_id = (v - 35) / 2;
        let recovery_id = ((v - 35) % 2) as i32;
        (Some(chain_id), recovery_id)
    } else {
        // Legacy: v = 27 or 28
        let recovery_id = (v - 27) as i32;
        (None, recovery_id)
    };

    let signing_hash = encode_eth_signing_hash(tx, chain_id);

    let mut sig_bytes = [0u8; 64];
    let r_padded = if r.len() < 32 {
        let mut padded = vec![0u8; 32 - r.len()];
        padded.extend(&r);
        padded
    } else {
        r[r.len() - 32..].to_vec()
    };
    let s_padded = if s.len() < 32 {
        let mut padded = vec![0u8; 32 - s.len()];
        padded.extend(&s);
        padded
    } else {
        s[s.len() - 32..].to_vec()
    };
    sig_bytes[..32].copy_from_slice(&r_padded);
    sig_bytes[32..].copy_from_slice(&s_padded);

    let rec_id = RecoveryId::from_i32(recovery_id)?;
    let sig = RecoverableSignature::from_compact(&sig_bytes, rec_id)?;
    let msg = Message::from_digest(signing_hash);

    let pubkey = secp.recover_ecdsa(&msg, &sig)?;
    let pubkey_bytes = pubkey.serialize_uncompressed();

    use sha3::{Digest, Keccak256};
    let pubkey_hash = Keccak256::digest(&pubkey_bytes[1..]);
    let derived_address = format!("0x{}", hex::encode(&pubkey_hash[12..]));
    let expected_lower = expected_address.to_lowercase();

    if derived_address != expected_lower {
        eprintln!("    Error: derived address {} != expected {}", derived_address, expected_lower);
        return Ok(None);
    }

    Ok(Some((hex::encode(pubkey_bytes), "uncompressed".to_string())))
}

// ============================================================================
// JSONC File Processing
// ============================================================================

fn strip_jsonc_comments(content: &str) -> String {
    let mut result = String::new();
    let mut in_string = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut chars = content.chars().peekable();

    while let Some(c) = chars.next() {
        if in_line_comment {
            if c == '\n' {
                in_line_comment = false;
                result.push(c);
            }
            continue;
        }

        if in_block_comment {
            if c == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_block_comment = false;
            }
            continue;
        }

        if in_string {
            result.push(c);
            if c == '\\' {
                if let Some(next) = chars.next() {
                    result.push(next);
                }
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }

        if c == '"' {
            in_string = true;
            result.push(c);
            continue;
        }

        if c == '/' {
            match chars.peek() {
                Some('/') => {
                    chars.next();
                    in_line_comment = true;
                    continue;
                }
                Some('*') => {
                    chars.next();
                    in_block_comment = true;
                    continue;
                }
                _ => {}
            }
        }

        result.push(c);
    }

    result
}

fn update_jsonc_with_pubkey(
    content: &str,
    identifier: &PuzzleIdentifier,
    pubkey_value: &str,
    pubkey_format: &str,
) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut result_lines: Vec<String> = Vec::new();
    let mut in_target_puzzle = false;
    let mut puzzle_indent = 0;
    let mut inserted = false;

    for line in lines.iter() {
        let is_target_line = match identifier {
            PuzzleIdentifier::Name(name) => line.contains(&format!("\"name\": \"{}\"", name)),
            PuzzleIdentifier::Bits(bits) => line.contains(&format!("\"bits\": {}", bits)),
            PuzzleIdentifier::SinglePuzzle => line.contains("\"puzzle\":"),
        };

        if is_target_line && !in_target_puzzle {
            in_target_puzzle = true;
            puzzle_indent = line.len() - line.trim_start().len();
            if matches!(identifier, PuzzleIdentifier::SinglePuzzle) {
                puzzle_indent += 2;
            }
            result_lines.push(line.to_string());
            continue;
        }

        if in_target_puzzle && !inserted {
            if line.trim().starts_with("\"pubkey\"") {
                continue;
            }

            if line.trim().starts_with("\"status\"") {
                let indent = " ".repeat(puzzle_indent);
                result_lines.push(format!(
                    "{}\"pubkey\": {{ \"value\": \"{}\", \"format\": \"{}\" }},",
                    indent, pubkey_value, pubkey_format
                ));
                inserted = true;
                in_target_puzzle = false;
            }
        }

        result_lines.push(line.to_string());
    }

    if inserted {
        Some(result_lines.join("\n"))
    } else {
        None
    }
}

// ============================================================================
// Main Processing Logic
// ============================================================================

/// Identifier type for different collection formats
#[derive(Debug, Clone)]
enum PuzzleIdentifier {
    /// For zden/bitimage: "name": "puzzle_name"
    Name(String),
    /// For b1000: "bits": N
    Bits(u32),
    /// For gsmg/bitaps: single puzzle collection, insert at collection level
    SinglePuzzle,
}

struct PuzzleToProcess {
    collection: String,
    identifier: PuzzleIdentifier,
    chain: String,
    address: String,
    claim_txid: String,
}

fn find_puzzles_needing_pubkey(data_dir: &Path) -> Vec<PuzzleToProcess> {
    let mut puzzles = Vec::new();

    let collections = ["zden", "bitimage", "b1000", "gsmg", "bitaps", "ballet"];

    for collection_name in &collections {
        let path = data_dir.join(format!("{}.jsonc", collection_name));
        if !path.exists() {
            continue;
        }

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let json_content = strip_jsonc_comments(&content);
        let collection: Collection = match serde_json::from_str(&json_content) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error parsing {}: {}", collection_name, e);
                continue;
            }
        };

        let puzzle_list: Vec<Puzzle> = if let Some(puzzles) = collection.puzzles {
            puzzles
        } else if let Some(puzzle) = collection.puzzle {
            vec![puzzle]
        } else {
            continue;
        };

        for puzzle in puzzle_list {
            // Skip if already has pubkey
            if puzzle.pubkey.is_some() {
                continue;
            }

            // Skip unsolved puzzles
            if puzzle.status == "unsolved" {
                continue;
            }

            if puzzle.address.kind.as_deref()
                .map(|k| k.eq_ignore_ascii_case("p2sh"))
                .unwrap_or(false)
            {
                continue;
            }

            // Find claim/sweep transaction
            let claim_txid = puzzle
                .transactions
                .as_ref()
                .and_then(|txs| {
                    txs.iter()
                        .find(|tx| tx.tx_type == "claim" || tx.tx_type == "sweep")
                        .map(|tx| tx.txid.clone())
                })
                .unwrap_or_default();

            if claim_txid.is_empty() {
                continue;
            }

            let chain = puzzle.chain.unwrap_or_else(|| "bitcoin".to_string());

            let identifier = if let Some(name) = puzzle.name.clone() {
                PuzzleIdentifier::Name(name)
            } else if let Some(bits) = puzzle.key.as_ref().and_then(|k| k.bits) {
                PuzzleIdentifier::Bits(bits)
            } else {
                PuzzleIdentifier::SinglePuzzle
            };

            puzzles.push(PuzzleToProcess {
                collection: collection_name.to_string(),
                identifier,
                chain,
                address: puzzle.address.value.clone(),
                claim_txid,
            });
        }
    }

    puzzles
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let args: Vec<String> = std::env::args().collect();

    let mut apply = false;
    let mut collection_filter: Option<String> = None;
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--apply" => {
                apply = true;
                i += 1;
            }
            "--collection" if i + 1 < args.len() => {
                collection_filter = Some(args[i + 1].clone());
                i += 2;
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                i += 1;
            }
        }
    }

    let etherscan_api_key = std::env::var("ETHERSCAN_API_KEY").ok();

    let client = Client::builder()
        .user_agent("boha-scripts/0.1")
        .timeout(Duration::from_secs(30))
        .build()?;

    let data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../data");

    println!("Scanning for puzzles needing pubkey extraction...\n");

    let mut puzzles = find_puzzles_needing_pubkey(&data_dir);

    if let Some(filter) = &collection_filter {
        puzzles.retain(|p| &p.collection == filter);
    }

    if puzzles.is_empty() {
        println!("No puzzles need pubkey extraction.");
        return Ok(());
    }

    let needs_eth = puzzles.iter().any(|p| p.chain == "ethereum");
    if needs_eth && etherscan_api_key.is_none() {
        return Err("ETHERSCAN_API_KEY is required to process ethereum puzzles".into());
    }

    println!("Found {} puzzles needing pubkey:\n", puzzles.len());

    // Group by collection for file updates
    let mut updates_by_collection: HashMap<String, Vec<(PuzzleIdentifier, String, String)>> =
        HashMap::new();

    for puzzle in &puzzles {
        let display_name = match &puzzle.identifier {
            PuzzleIdentifier::Name(n) => n.clone(),
            PuzzleIdentifier::Bits(b) => b.to_string(),
            PuzzleIdentifier::SinglePuzzle => "puzzle".to_string(),
        };

        println!(
            "Processing: {}/{} ({}) - txid: {}",
            puzzle.collection,
            display_name,
            puzzle.chain,
            &puzzle.claim_txid[..16.min(puzzle.claim_txid.len())]
        );

        let fetch_result = match puzzle.chain.as_str() {
            "bitcoin" | "litecoin" => {
                fetch_btc_pubkey(&client, &puzzle.claim_txid, &puzzle.address, &puzzle.chain).await
            }
            "decred" => fetch_dcr_pubkey(&client, &puzzle.claim_txid).await,
            "ethereum" => {
                if let Some(api_key) = &etherscan_api_key {
                    fetch_eth_pubkey(&client, &puzzle.claim_txid, api_key, &puzzle.address).await
                } else {
                    Ok(None)
                }
            }
            _ => {
                eprintln!("    Unsupported chain: {}", puzzle.chain);
                Ok(None)
            }
        };

        let result = match fetch_result {
            Ok(res) => res,
            Err(err) => {
                eprintln!("    Fetch failed for {}:{}: {}", puzzle.chain, puzzle.claim_txid, err);
                continue;
            }
        };

        match result {
            Some((pubkey, format)) => {
                println!("    Found pubkey: {}... ({})", &pubkey[..16], format);

                updates_by_collection
                    .entry(puzzle.collection.clone())
                    .or_default()
                    .push((puzzle.identifier.clone(), pubkey, format));
            }
            None => {
                println!("    Could not extract pubkey");
            }
        }
    }

    println!();

    // Apply updates
    if apply {
        println!("Applying updates...\n");

        for (collection, updates) in &updates_by_collection {
            let path = data_dir.join(format!("{}.jsonc", collection));
            let mut content = std::fs::read_to_string(&path)?;

            for (identifier, pubkey, format) in updates {
                let display_name = match identifier {
                    PuzzleIdentifier::Name(n) => n.clone(),
                    PuzzleIdentifier::Bits(b) => b.to_string(),
                    PuzzleIdentifier::SinglePuzzle => "puzzle".to_string(),
                };

                if let Some(new_content) =
                    update_jsonc_with_pubkey(&content, identifier, pubkey, format)
                {
                    content = new_content;
                    println!("  Updated {}/{}", collection, display_name);
                } else {
                    eprintln!("  Failed to update {}/{}", collection, display_name);
                }
            }

            std::fs::write(&path, content)?;
        }

        println!("\nDone!");
    } else {
        println!("Dry-run mode. Use --apply to update files.");

        for (collection, updates) in &updates_by_collection {
            for (identifier, pubkey, format) in updates {
                let display_name = match identifier {
                    PuzzleIdentifier::Name(n) => n.clone(),
                    PuzzleIdentifier::Bits(b) => b.to_string(),
                    PuzzleIdentifier::SinglePuzzle => "puzzle".to_string(),
                };

                println!(
                    "  Would update {}/{}: pubkey = {}... ({})",
                    collection,
                    display_name,
                    &pubkey[..16.min(pubkey.len())],
                    format
                );
            }
        }
    }

    Ok(())
}
