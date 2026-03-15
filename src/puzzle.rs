//! Core puzzle types and structures.

use num_bigint::BigUint;
use num_traits::One;
use serde::Serialize;
use std::fmt;
use std::ops::RangeInclusive;
use std::str::FromStr;

/// Blockchain network for a puzzle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Chain {
    Bitcoin,
    Ethereum,
    Litecoin,
    Monero,
    Decred,
    Arweave,
}

impl Chain {
    pub const ALL: [Chain; 6] = [
        Chain::Bitcoin,
        Chain::Ethereum,
        Chain::Litecoin,
        Chain::Monero,
        Chain::Decred,
        Chain::Arweave,
    ];

    /// Currency symbol (e.g., "BTC", "ETH").
    pub fn symbol(&self) -> &'static str {
        match self {
            Chain::Bitcoin => "BTC",
            Chain::Ethereum => "ETH",
            Chain::Litecoin => "LTC",
            Chain::Monero => "XMR",
            Chain::Decred => "DCR",
            Chain::Arweave => "AR",
        }
    }

    /// Full chain name.
    pub fn name(&self) -> &'static str {
        match self {
            Chain::Bitcoin => "Bitcoin",
            Chain::Ethereum => "Ethereum",
            Chain::Litecoin => "Litecoin",
            Chain::Monero => "Monero",
            Chain::Decred => "Decred",
            Chain::Arweave => "Arweave",
        }
    }

    pub fn tx_explorer_url(&self, txid: &str) -> String {
        match self {
            Chain::Bitcoin => format!("https://mempool.space/tx/{}", txid),
            Chain::Ethereum => format!("https://etherscan.io/tx/{}", txid),
            Chain::Litecoin => format!("https://blockchair.com/litecoin/transaction/{}", txid),
            Chain::Monero => format!("https://xmrchain.net/tx/{}", txid),
            Chain::Decred => format!("https://dcrdata.decred.org/tx/{}", txid),
            Chain::Arweave => format!("https://viewblock.io/arweave/tx/{}", txid),
        }
    }

    pub fn address_explorer_url(&self, address: &str) -> String {
        match self {
            Chain::Bitcoin => format!("https://mempool.space/address/{}", address),
            Chain::Ethereum => format!("https://etherscan.io/address/{}", address),
            Chain::Litecoin => format!("https://blockchair.com/litecoin/address/{}", address),
            Chain::Monero => format!("https://xmrchain.net/search?value={}", address),
            Chain::Decred => format!("https://dcrdata.decred.org/address/{}", address),
            Chain::Arweave => format!("https://viewblock.io/arweave/address/{}", address),
        }
    }

    pub fn is_valid_txid(&self, txid: &str) -> bool {
        fn is_hex64(s: &str) -> bool {
            s.len() == 64 && s.as_bytes().iter().all(|b: &u8| b.is_ascii_hexdigit())
        }

        fn is_base64url_43(s: &str) -> bool {
            s.len() == 43
                && s.as_bytes()
                    .iter()
                    .all(|b| matches!(b, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_'))
        }

        match self {
            Chain::Ethereum => txid.starts_with("0x") && txid.len() == 66 && is_hex64(&txid[2..]),
            // Current chains use hex-encoded 256-bit hashes.
            Chain::Bitcoin | Chain::Litecoin | Chain::Monero | Chain::Decred => is_hex64(txid),
            Chain::Arweave => is_base64url_43(txid),
        }
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Chain::Bitcoin => "bitcoin",
            Chain::Ethereum => "ethereum",
            Chain::Litecoin => "litecoin",
            Chain::Monero => "monero",
            Chain::Decred => "decred",
            Chain::Arweave => "arweave",
        })
    }
}

impl FromStr for Chain {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bitcoin" | "btc" => Ok(Chain::Bitcoin),
            "ethereum" | "eth" => Ok(Chain::Ethereum),
            "litecoin" | "ltc" => Ok(Chain::Litecoin),
            "monero" | "xmr" => Ok(Chain::Monero),
            "decred" | "dcr" => Ok(Chain::Decred),
            "arweave" | "ar" => Ok(Chain::Arweave),
            _ => Err(format!(
                "unknown chain: '{}'. expected: bitcoin, ethereum, litecoin, monero, decred, arweave (or symbol: btc, eth, ltc, xmr, dcr, ar)",
                s
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Solved,
    Unsolved,
    Claimed,
    Swept,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    Funding,
    Increase,
    Decrease,
    Sweep,
    Claim,
    PubkeyReveal,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Transaction {
    pub tx_type: TransactionType,
    pub txid: Option<&'static str>,
    pub date: Option<&'static str>,
    pub amount: Option<f64>,
}

impl Status {
    pub fn is_active(&self) -> bool {
        matches!(self, Status::Unsolved)
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Status::Solved => "solved",
            Status::Unsolved => "unsolved",
            Status::Claimed => "claimed",
            Status::Swept => "swept",
        })
    }
}

impl FromStr for Status {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "solved" => Ok(Status::Solved),
            "unsolved" => Ok(Status::Unsolved),
            "claimed" => Ok(Status::Claimed),
            "swept" => Ok(Status::Swept),
            _ => Err(format!(
                "unknown status: '{}'. expected: solved, unsolved, claimed, swept",
                s
            )),
        }
    }
}

/// Crypto address with chain-specific type information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct Address {
    /// The address string (e.g., "1BgGZ9tcN4rm9KBzDn7KprQz87SZ26SAMH")
    pub value: &'static str,
    /// Blockchain network
    pub chain: Chain,
    /// Address type/kind (e.g., "p2pkh", "p2sh", "p2wpkh", "p2wsh", "p2tr", "standard")
    pub kind: &'static str,
    /// HASH160 of the public key or script (P2PKH, P2SH, P2WPKH)
    pub hash160: Option<&'static str>,
    /// Witness program for SegWit/Taproot addresses (P2WSH: 32-byte SHA256, P2TR: 32-byte x-only pubkey)
    pub witness_program: Option<&'static str>,
    /// P2SH redeem script (only for p2sh addresses)
    pub redeem_script: Option<RedeemScript>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PubkeyFormat {
    Compressed,
    Uncompressed,
}

/// Source of entropy for deterministic seed generation (e.g., file, image).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct EntropySource {
    /// URL to the original entropy source
    pub url: Option<&'static str>,
    /// Human-readable description of the entropy source
    pub description: Option<&'static str>,
}

/// BIP39 passphrase status for entropy-based seeds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum Passphrase {
    /// Passphrase is required but unknown
    Required,
    /// Passphrase is known
    Known(&'static str),
}

/// External entropy used to derive a seed (for bitimage, brainwallet-style puzzles).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct Entropy {
    /// SHA256 hash of the entropy data (for verification)
    pub hash: &'static str,
    /// Source of the entropy (URL, description)
    pub source: Option<EntropySource>,
    /// BIP39 passphrase status
    pub passphrase: Option<Passphrase>,
}

/// BIP39 seed phrase with optional derivation path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct Seed {
    /// Mnemonic phrase (12/15/18/21/24 words), None if unknown
    pub phrase: Option<&'static str>,
    /// HD derivation path (e.g., "m/44'/0'/0'/0/0")
    pub path: Option<&'static str>,
    /// Extended public key (xpub/ypub/zpub)
    pub xpub: Option<&'static str>,
    /// External entropy source (for deterministic seeds like bitimage)
    pub entropy: Option<Entropy>,
}

/// A single share from a secret sharing scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct Share {
    /// Share index (1-based)
    pub index: u8,
    /// Share data (words, hex, or other format depending on scheme)
    pub data: &'static str,
}

/// Secret sharing scheme configuration (e.g., Shamir, SLIP-39).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct Shares {
    /// Number of shares required to reconstruct the secret
    pub threshold: u8,
    /// Total number of shares generated
    pub total: u8,
    /// Published shares
    pub shares: &'static [Share],
}

/// Wallet Import Format (WIF) with optional BIP38 encryption.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct Wif {
    /// BIP38 encrypted WIF (starts with 6P)
    pub encrypted: Option<&'static str>,
    /// Decrypted/standard WIF (starts with 5, K, L)
    pub decrypted: Option<&'static str>,
    /// BIP38 passphrase for decryption
    pub passphrase: Option<&'static str>,
}

/// Private key in various representations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct Key {
    /// Raw hex (64 characters, 32 bytes)
    pub hex: Option<&'static str>,
    /// Wallet Import Format (standard or BIP38 encrypted)
    pub wif: Option<Wif>,
    /// BIP39 seed phrase with optional derivation path
    pub seed: Option<Seed>,
    /// Mini private key format (starts with 'S')
    pub mini: Option<&'static str>,
    /// Bit range constraint: key is in [2^(bits-1), 2^bits - 1]
    pub bits: Option<u16>,
    /// Secret sharing scheme (e.g., Shamir, SLIP-39)
    pub shares: Option<Shares>,
}

/// P2SH redeem script with its hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct RedeemScript {
    /// The redeem script in hex
    pub script: &'static str,
    /// HASH160 of the redeem script
    pub hash: &'static str,
}

/// Puzzle assets (images, hints, solutions).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct Assets {
    /// Relative path to main puzzle image (within collection's assets folder)
    pub puzzle: Option<&'static str>,
    /// Relative path to solution explanation image
    pub solver: Option<&'static str>,
    /// Hint images
    pub hints: &'static [&'static str],
    /// Original source URL for attribution
    pub source_url: Option<&'static str>,
}

/// Social/web profile link.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct Profile {
    /// Platform name (e.g., "github", "twitter", "bitcointalk").
    pub name: &'static str,
    /// URL to the profile.
    pub url: &'static str,
}

/// Author/creator of a puzzle collection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Author {
    /// Author's name or pseudonym (None for anonymous).
    pub name: Option<&'static str>,
    /// Addresses that initially funded the puzzle(s).
    pub addresses: &'static [&'static str],
    /// Profile links.
    pub profiles: &'static [Profile],
}

/// Information about who solved a puzzle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Solver {
    /// Solver's name or pseudonym (if known).
    pub name: Option<&'static str>,
    /// Addresses that claimed the funds.
    pub addresses: &'static [&'static str],
    /// Profile links.
    pub profiles: &'static [Profile],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Pubkey {
    pub value: &'static str,
    pub format: PubkeyFormat,
}

#[derive(Debug, Clone, Serialize)]
pub struct Puzzle {
    pub id: &'static str,
    pub chain: Chain,
    pub address: Address,
    pub status: Status,
    pub pubkey: Option<Pubkey>,
    pub key: Option<Key>,
    pub prize: Option<f64>,
    pub start_date: Option<&'static str>,
    pub solve_date: Option<&'static str>,
    pub solve_time: Option<u64>,
    pub pre_genesis: bool,
    pub source_url: Option<&'static str>,
    pub transactions: &'static [Transaction],
    pub solver: Option<Solver>,
    pub assets: Option<Assets>,
}

fn format_duration_human_readable(seconds: u64) -> String {
    const MINUTE: u64 = 60;
    const HOUR: u64 = 60 * MINUTE;
    const DAY: u64 = 24 * HOUR;
    const MONTH: u64 = 30 * DAY;
    const YEAR: u64 = 365 * DAY;

    let years = seconds / YEAR;
    let remaining = seconds % YEAR;
    let months = remaining / MONTH;
    let remaining = remaining % MONTH;
    let days = remaining / DAY;
    let remaining = remaining % DAY;
    let hours = remaining / HOUR;
    let remaining = remaining % HOUR;
    let minutes = remaining / MINUTE;

    let mut parts = Vec::new();

    if years > 0 {
        parts.push(format!("{}y", years));
    }
    if months > 0 {
        parts.push(format!("{}mo", months));
    }
    if days > 0 {
        parts.push(format!("{}d", days));
    }
    if hours > 0 {
        parts.push(format!("{}h", hours));
    }
    if minutes > 0 {
        parts.push(format!("{}m", minutes));
    }

    if parts.is_empty() {
        format!("{}s", seconds)
    } else {
        parts.join(" ")
    }
}

impl Key {
    pub fn has_hex(&self) -> bool {
        self.hex.is_some()
    }

    pub fn has_seed(&self) -> bool {
        self.seed.is_some()
    }

    pub fn has_shares(&self) -> bool {
        self.shares.is_some()
    }

    pub fn is_known(&self) -> bool {
        self.hex.is_some() || self.wif.is_some() || self.seed.is_some() || self.mini.is_some()
    }

    pub fn range(&self) -> Option<RangeInclusive<u128>> {
        let bits = self.bits?;
        if !(1..=128).contains(&bits) {
            return None;
        }
        let start = 1u128 << (bits - 1);
        let end = if bits == 128 {
            u128::MAX
        } else {
            (1u128 << bits) - 1
        };
        Some(start..=end)
    }

    pub fn range_big(&self) -> Option<(BigUint, BigUint)> {
        let bits = self.bits?;
        if !(1..=256).contains(&bits) {
            return None;
        }
        let start = BigUint::one() << (bits - 1) as usize;
        let end = (BigUint::one() << bits as usize) - 1u32;
        Some((start, end))
    }
}

impl Puzzle {
    pub fn has_pubkey(&self) -> bool {
        self.pubkey.is_some()
    }

    pub fn pubkey_str(&self) -> Option<&'static str> {
        self.pubkey.map(|p| p.value)
    }

    pub fn has_private_key(&self) -> bool {
        self.key.is_some_and(|k| k.is_known())
    }

    pub fn solve_time_formatted(&self) -> Option<String> {
        self.solve_time.map(format_duration_human_readable)
    }

    pub fn collection(&self) -> &str {
        self.id.split('/').next().unwrap_or(self.id)
    }

    pub fn name(&self) -> &str {
        self.id.split('/').nth(1).unwrap_or("")
    }

    pub fn funding_tx(&self) -> Option<&Transaction> {
        self.transactions
            .iter()
            .find(|t| t.tx_type == TransactionType::Funding)
    }

    pub fn claim_tx(&self) -> Option<&Transaction> {
        self.transactions
            .iter()
            .find(|t| t.tx_type == TransactionType::Claim)
    }

    pub fn claim_txid(&self) -> Option<&'static str> {
        self.claim_tx().and_then(|tx| tx.txid)
    }

    pub fn funding_txid(&self) -> Option<&'static str> {
        self.funding_tx().and_then(|tx| tx.txid)
    }

    pub fn has_transactions(&self) -> bool {
        !self.transactions.is_empty()
    }

    pub fn transaction_count(&self) -> usize {
        self.transactions.len()
    }

    /// Returns the relative path to the main puzzle asset.
    /// Example: "assets/zden/level_4/puzzle.png"
    pub fn asset_path(&self) -> Option<String> {
        self.assets
            .and_then(|a| a.puzzle)
            .map(|p| format!("assets/{}/{}", self.collection(), p))
    }

    /// Returns the GitHub raw URL for remote access to the main puzzle asset.
    /// Example: "https://raw.githubusercontent.com/oritwoen/boha/main/assets/zden/level_4/puzzle.png"
    pub fn asset_url(&self) -> Option<String> {
        self.asset_path()
            .map(|p| format!("https://raw.githubusercontent.com/oritwoen/boha/main/{}", p))
    }

    pub fn explorer_url(&self) -> String {
        debug_assert_eq!(self.chain, self.address.chain);
        self.address.chain.address_explorer_url(self.address.value)
    }

    pub fn key_range(&self) -> Option<RangeInclusive<u128>> {
        self.key.and_then(|k| k.range())
    }

    pub fn key_range_big(&self) -> Option<(BigUint, BigUint)> {
        self.key.and_then(|k| k.range_big())
    }
}

pub trait IntoPuzzleNum {
    fn into_puzzle_num(self) -> Option<u32>;
}

impl IntoPuzzleNum for u32 {
    fn into_puzzle_num(self) -> Option<u32> {
        Some(self)
    }
}

impl IntoPuzzleNum for i32 {
    fn into_puzzle_num(self) -> Option<u32> {
        if self > 0 {
            Some(self as u32)
        } else {
            None
        }
    }
}

impl IntoPuzzleNum for usize {
    fn into_puzzle_num(self) -> Option<u32> {
        u32::try_from(self).ok()
    }
}

impl IntoPuzzleNum for &str {
    fn into_puzzle_num(self) -> Option<u32> {
        self.parse().ok()
    }
}

impl IntoPuzzleNum for String {
    fn into_puzzle_num(self) -> Option<u32> {
        self.parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_is_active() {
        assert!(Status::Unsolved.is_active());
        assert!(!Status::Solved.is_active());
        assert!(!Status::Claimed.is_active());
        assert!(!Status::Swept.is_active());
    }

    #[test]
    fn chain_display_matches_serde() {
        assert_eq!(Chain::Bitcoin.to_string(), "bitcoin");
        assert_eq!(Chain::Ethereum.to_string(), "ethereum");
        assert_eq!(Chain::Litecoin.to_string(), "litecoin");
        assert_eq!(Chain::Monero.to_string(), "monero");
        assert_eq!(Chain::Decred.to_string(), "decred");
        assert_eq!(Chain::Arweave.to_string(), "arweave");
    }

    #[test]
    fn chain_fromstr_by_name() {
        assert_eq!("bitcoin".parse::<Chain>().unwrap(), Chain::Bitcoin);
        assert_eq!("Bitcoin".parse::<Chain>().unwrap(), Chain::Bitcoin);
        assert_eq!("ETHEREUM".parse::<Chain>().unwrap(), Chain::Ethereum);
    }

    #[test]
    fn chain_fromstr_by_symbol() {
        assert_eq!("btc".parse::<Chain>().unwrap(), Chain::Bitcoin);
        assert_eq!("ETH".parse::<Chain>().unwrap(), Chain::Ethereum);
        assert_eq!("ltc".parse::<Chain>().unwrap(), Chain::Litecoin);
        assert_eq!("xmr".parse::<Chain>().unwrap(), Chain::Monero);
        assert_eq!("dcr".parse::<Chain>().unwrap(), Chain::Decred);
        assert_eq!("ar".parse::<Chain>().unwrap(), Chain::Arweave);
    }

    #[test]
    fn chain_fromstr_invalid() {
        assert!("dogecoin".parse::<Chain>().is_err());
        assert!("".parse::<Chain>().is_err());
    }

    #[test]
    fn status_display_matches_serde() {
        assert_eq!(Status::Solved.to_string(), "solved");
        assert_eq!(Status::Unsolved.to_string(), "unsolved");
        assert_eq!(Status::Claimed.to_string(), "claimed");
        assert_eq!(Status::Swept.to_string(), "swept");
    }

    #[test]
    fn status_fromstr() {
        assert_eq!("solved".parse::<Status>().unwrap(), Status::Solved);
        assert_eq!("Unsolved".parse::<Status>().unwrap(), Status::Unsolved);
        assert_eq!("CLAIMED".parse::<Status>().unwrap(), Status::Claimed);
        assert_eq!("swept".parse::<Status>().unwrap(), Status::Swept);
    }

    #[test]
    fn status_fromstr_invalid() {
        assert!("pending".parse::<Status>().is_err());
        assert!("".parse::<Status>().is_err());
    }

    #[test]
    fn chain_roundtrip() {
        for chain in Chain::ALL {
            let s = chain.to_string();
            assert_eq!(s.parse::<Chain>().unwrap(), chain);
        }
    }

    #[test]
    fn test_into_puzzle_num_i32() {
        assert_eq!((-1i32).into_puzzle_num(), None);
        assert_eq!(0i32.into_puzzle_num(), None);
        assert_eq!(1i32.into_puzzle_num(), Some(1));
    }

    #[test]
    fn valid_bitcoin_txid() {
        let txid = "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d";
        assert!(Chain::Bitcoin.is_valid_txid(txid));
    }

    #[test]
    fn bitcoin_txid_accepts_mixed_case_hex() {
        let lower = "a3b5c7d9e1f20000000000000000000000000000000000000000000000000001";
        let upper = "A3B5C7D9E1F20000000000000000000000000000000000000000000000000001";
        let mixed = "a3B5c7D9e1F20000000000000000000000000000000000000000000000000001";

        assert!(Chain::Bitcoin.is_valid_txid(lower));
        assert!(Chain::Bitcoin.is_valid_txid(upper));
        assert!(Chain::Bitcoin.is_valid_txid(mixed));
    }

    #[test]
    fn bitcoin_txid_rejects_wrong_length() {
        assert!(!Chain::Bitcoin.is_valid_txid("abcd"));
        assert!(!Chain::Bitcoin.is_valid_txid(""));
        let too_long = "a".repeat(65);
        assert!(!Chain::Bitcoin.is_valid_txid(&too_long));
    }

    #[test]
    fn bitcoin_txid_rejects_non_hex() {
        let txid = "g1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d";
        assert!(!Chain::Bitcoin.is_valid_txid(txid));
    }

    #[test]
    fn valid_ethereum_txid() {
        let txid = "0xa1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d";
        assert!(Chain::Ethereum.is_valid_txid(txid));
    }

    #[test]
    fn ethereum_txid_requires_0x_prefix() {
        let txid = "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d";
        assert!(!Chain::Ethereum.is_valid_txid(txid));
    }

    #[test]
    fn ethereum_txid_rejects_wrong_length() {
        assert!(!Chain::Ethereum.is_valid_txid("0xabcd"));
        assert!(!Chain::Ethereum.is_valid_txid("0x"));
    }

    #[test]
    fn valid_arweave_txid() {
        let txid = "hKMMPNh_emBf8v_at1tFzNYACisyMQNcKzeeE1QE9p8";
        assert!(Chain::Arweave.is_valid_txid(txid));
    }

    #[test]
    fn arweave_txid_rejects_wrong_length() {
        assert!(!Chain::Arweave.is_valid_txid("too_short"));
        assert!(!Chain::Arweave.is_valid_txid(""));
    }

    #[test]
    fn arweave_txid_rejects_invalid_chars() {
        let txid = "hKMMPNh_emBf8v_at1tFzNYACisyMQNc!@#$%^&*()+=";
        assert!(!Chain::Arweave.is_valid_txid(txid));
    }

    #[test]
    fn litecoin_and_decred_share_bitcoin_format() {
        let txid = "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d";
        assert!(Chain::Litecoin.is_valid_txid(txid));
        assert!(Chain::Decred.is_valid_txid(txid));
        assert!(Chain::Monero.is_valid_txid(txid));
    }

    #[test]
    fn format_duration_zero_seconds() {
        assert_eq!(format_duration_human_readable(0), "0s");
    }

    #[test]
    fn format_duration_under_minute() {
        assert_eq!(format_duration_human_readable(45), "45s");
    }

    #[test]
    fn format_duration_exact_minute() {
        assert_eq!(format_duration_human_readable(60), "1m");
    }

    #[test]
    fn format_duration_hours_and_minutes() {
        assert_eq!(format_duration_human_readable(3661), "1h 1m");
    }

    #[test]
    fn format_duration_days() {
        assert_eq!(format_duration_human_readable(86400), "1d");
    }

    #[test]
    fn format_duration_months() {
        assert_eq!(format_duration_human_readable(30 * 86400), "1mo");
    }

    #[test]
    fn format_duration_years_and_months() {
        let one_year_two_months = 365 * 86400 + 2 * 30 * 86400;
        assert_eq!(
            format_duration_human_readable(one_year_two_months),
            "1y 2mo"
        );
    }

    #[test]
    fn format_duration_all_units() {
        let duration = 365 * 86400 + 30 * 86400 + 86400 + 3600 + 60;
        assert_eq!(format_duration_human_readable(duration), "1y 1mo 1d 1h 1m");
    }

    #[test]
    fn test_address_explorer_url_all_chains() {
        let cases: &[(Chain, &str, &str)] = &[
            (Chain::Bitcoin, "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa", "https://mempool.space/address/1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"),
            (Chain::Ethereum, "0xde0b295669a9fd93d5f28d9ec85e40f4cb697bae", "https://etherscan.io/address/0xde0b295669a9fd93d5f28d9ec85e40f4cb697bae"),
            (Chain::Litecoin, "LVuDpNCSSj6pQ7t9Pv6d6sUkLKoqDEVUnJ", "https://blockchair.com/litecoin/address/LVuDpNCSSj6pQ7t9Pv6d6sUkLKoqDEVUnJ"),
            (Chain::Monero, "44AFFq5kSiGBoZ4NMDwYtN18obc8AemS33DBLWs3H7otXft3XjrpDtQGv7SqSsaBYBb98uNbr2VBBEt7f2wfn3RVGQBEP3A", "https://xmrchain.net/search?value=44AFFq5kSiGBoZ4NMDwYtN18obc8AemS33DBLWs3H7otXft3XjrpDtQGv7SqSsaBYBb98uNbr2VBBEt7f2wfn3RVGQBEP3A"),
            (Chain::Decred, "DsUZxxoHJSty8DCfwfartwTYbuhmVct7tJu", "https://dcrdata.decred.org/address/DsUZxxoHJSty8DCfwfartwTYbuhmVct7tJu"),
            (Chain::Arweave, "vh-NTHVvlKZqRxc8LyyTNok65yQ55a_PJ1zWLb9G2JI", "https://viewblock.io/arweave/address/vh-NTHVvlKZqRxc8LyyTNok65yQ55a_PJ1zWLb9G2JI"),
        ];
        for (chain, addr, expected) in cases {
            assert_eq!(
                chain.address_explorer_url(addr),
                *expected,
                "failed for {:?}",
                chain
            );
        }
    }

    #[test]
    fn test_puzzle_explorer_url_delegates() {
        let puzzle = crate::b1000::get(1).expect("puzzle b1000/1 should exist");
        let expected = puzzle.chain.address_explorer_url(puzzle.address.value);
        assert_eq!(puzzle.explorer_url(), expected);
    }
}
