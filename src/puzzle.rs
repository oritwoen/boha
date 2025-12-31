//! Core puzzle types and structures.

use serde::Serialize;

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Solved,
    Unsolved,
    Claimed,
    Swept,
}

impl Status {
    pub fn is_active(&self) -> bool {
        matches!(self, Status::Unsolved)
    }
}

/// Crypto address type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AddressType {
    /// Pay to Public Key Hash (1...)
    P2PKH,
    /// Pay to Script Hash (3...)
    P2SH,
    /// Pay to Witness Public Key Hash (bc1q...)
    P2WPKH,
}

impl AddressType {
    /// Detect address type from address string.
    pub fn from_address(address: &str) -> Option<Self> {
        if address.starts_with('1') {
            Some(AddressType::P2PKH)
        } else if address.starts_with('3') {
            Some(AddressType::P2SH)
        } else if address.starts_with("bc1q") {
            Some(AddressType::P2WPKH)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Puzzle {
    pub id: &'static str,
    pub chain: Chain,
    pub address: &'static str,
    pub address_type: Option<AddressType>,
    pub status: Status,
    pub pubkey: Option<&'static str>,
    pub private_key: Option<&'static str>,
    pub redeem_script: Option<&'static str>,
    pub bits: Option<u16>,
    pub prize: Option<f64>,
    pub start_date: Option<&'static str>,
    pub solve_date: Option<&'static str>,
    pub source_url: Option<&'static str>,
}

impl Puzzle {
    pub fn has_pubkey(&self) -> bool {
        self.pubkey.is_some()
    }

    pub fn has_private_key(&self) -> bool {
        self.private_key.is_some()
    }

    pub fn collection(&self) -> &str {
        self.id.split('/').next().unwrap_or(self.id)
    }

    pub fn name(&self) -> &str {
        self.id.split('/').nth(1).unwrap_or("")
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
    fn test_address_type_detection() {
        assert_eq!(
            AddressType::from_address("1BgGZ9tcN4rm9KBzDn7KprQz87SZ26SAMH"),
            Some(AddressType::P2PKH)
        );
        assert_eq!(
            AddressType::from_address("3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy"),
            Some(AddressType::P2SH)
        );
        assert_eq!(
            AddressType::from_address("bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq"),
            Some(AddressType::P2WPKH)
        );
    }

    #[test]
    fn test_status_is_active() {
        assert!(Status::Unsolved.is_active());
        assert!(!Status::Solved.is_active());
        assert!(!Status::Claimed.is_active());
        assert!(!Status::Swept.is_active());
    }
}
