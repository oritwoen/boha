//! Core puzzle types and structures.

use num_bigint::BigUint;
use num_traits::One;
use serde::Serialize;
use std::ops::RangeInclusive;

/// Blockchain network for a puzzle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Chain {
    Bitcoin,
    Ethereum,
    Litecoin,
    Monero,
    Decred,
}

impl Chain {
    /// Currency symbol (e.g., "BTC", "ETH").
    pub fn symbol(&self) -> &'static str {
        match self {
            Chain::Bitcoin => "BTC",
            Chain::Ethereum => "ETH",
            Chain::Litecoin => "LTC",
            Chain::Monero => "XMR",
            Chain::Decred => "DCR",
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
        }
    }

    pub fn tx_explorer_url(&self, txid: &str) -> String {
        match self {
            Chain::Bitcoin => format!("https://mempool.space/tx/{}", txid),
            Chain::Ethereum => format!("https://etherscan.io/tx/{}", txid),
            Chain::Litecoin => format!("https://blockchair.com/litecoin/transaction/{}", txid),
            Chain::Monero => format!("https://xmrchain.net/tx/{}", txid),
            Chain::Decred => format!("https://dcrdata.decred.org/tx/{}", txid),
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
}
