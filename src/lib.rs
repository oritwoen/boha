mod collections;
mod puzzle;

#[cfg(feature = "balance")]
pub mod balance;

#[cfg(feature = "cli")]
pub mod verify;

pub mod version {
    include!(concat!(env!("OUT_DIR"), "/data_version.rs"));
}

pub use collections::{arweave, b1000, ballet, bitaps, bitimage, gsmg, hash_collision, zden};
pub use puzzle::{
    Address, Assets, Author, Chain, Entropy, EntropySource, IntoPuzzleNum, Key, Passphrase,
    Profile, Pubkey, PubkeyFormat, Puzzle, RedeemScript, Seed, Share, Shares, Solver, Status,
    Transaction, TransactionType, Wif,
};

use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Puzzle not found: {0}")]
    NotFound(String),
    #[error("Invalid puzzle number: {0}")]
    InvalidNumber(u32),
    #[error("Invalid collection: {0}")]
    InvalidCollection(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Collection {
    Arweave,
    B1000,
    Ballet,
    Bitaps,
    Bitimage,
    Gsmg,
    HashCollision,
    Zden,
}

impl Collection {
    pub const ALL: [Self; 8] = [
        Self::Arweave,
        Self::B1000,
        Self::Ballet,
        Self::Bitaps,
        Self::Bitimage,
        Self::Gsmg,
        Self::HashCollision,
        Self::Zden,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Arweave => "arweave",
            Self::B1000 => "b1000",
            Self::Ballet => "ballet",
            Self::Bitaps => "bitaps",
            Self::Bitimage => "bitimage",
            Self::Gsmg => "gsmg",
            Self::HashCollision => "hash_collision",
            Self::Zden => "zden",
        }
    }

    pub fn parse(name: &str) -> Result<Self> {
        match name {
            "arweave" => Ok(Self::Arweave),
            "b1000" => Ok(Self::B1000),
            "ballet" => Ok(Self::Ballet),
            "bitaps" => Ok(Self::Bitaps),
            "bitimage" => Ok(Self::Bitimage),
            "gsmg" => Ok(Self::Gsmg),
            "hash_collision" | "peter_todd" => Ok(Self::HashCollision),
            "zden" => Ok(Self::Zden),
            _ => Err(Error::InvalidCollection(name.to_string())),
        }
    }

    pub fn slice(self) -> &'static [Puzzle] {
        match self {
            Self::Arweave => arweave::slice(),
            Self::B1000 => b1000::slice(),
            Self::Ballet => ballet::slice(),
            Self::Bitaps => bitaps::slice(),
            Self::Bitimage => bitimage::slice(),
            Self::Gsmg => gsmg::slice(),
            Self::HashCollision => hash_collision::slice(),
            Self::Zden => zden::slice(),
        }
    }

    pub fn all(self) -> std::slice::Iter<'static, Puzzle> {
        self.slice().iter()
    }

    pub fn author(self) -> &'static Author {
        match self {
            Self::Arweave => arweave::author(),
            Self::B1000 => b1000::author(),
            Self::Ballet => ballet::author(),
            Self::Bitaps => bitaps::author(),
            Self::Bitimage => bitimage::author(),
            Self::Gsmg => gsmg::author(),
            Self::HashCollision => hash_collision::author(),
            Self::Zden => zden::author(),
        }
    }

    pub fn get(self, name: &str) -> Result<&'static Puzzle> {
        match self {
            Self::Arweave => arweave::get(name),
            Self::B1000 => {
                let num = name
                    .parse::<u32>()
                    .map_err(|_| Error::NotFound(format!("{}/{}", self.name(), name)))?;
                b1000::get(num)
            }
            Self::Ballet => ballet::get(name),
            Self::Bitaps => {
                if name.is_empty() {
                    Ok(bitaps::get())
                } else {
                    Err(Error::NotFound(format!("{}/{}", self.name(), name)))
                }
            }
            Self::Bitimage => bitimage::get(name),
            Self::Gsmg => {
                if name.is_empty() {
                    Ok(gsmg::get())
                } else {
                    Err(Error::NotFound(format!("{}/{}", self.name(), name)))
                }
            }
            Self::HashCollision => hash_collision::get(name),
            Self::Zden => zden::get(name),
        }
    }
}

pub fn get(id: &str) -> Result<&'static Puzzle> {
    if id == "gsmg" {
        return Collection::Gsmg.get("");
    }
    if id == "bitaps" {
        return Collection::Bitaps.get("");
    }

    let parts: Vec<&str> = id.split('/').collect();
    if parts.len() != 2 {
        return Err(Error::NotFound(id.to_string()));
    }

    let collection = Collection::parse(parts[0]).map_err(|_| Error::NotFound(id.to_string()))?;

    if matches!(collection, Collection::Gsmg | Collection::Bitaps) {
        return Err(Error::NotFound(id.to_string()));
    }

    collection.get(parts[1])
}

pub fn all() -> impl Iterator<Item = &'static Puzzle> {
    Collection::ALL.into_iter().flat_map(Collection::all)
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct Stats {
    pub total: usize,
    pub solved: usize,
    pub unsolved: usize,
    pub claimed: usize,
    pub swept: usize,
    pub with_pubkey: usize,
    pub total_prize: HashMap<Chain, f64>,
    pub unsolved_prize: HashMap<Chain, f64>,
}

pub fn stats() -> Stats {
    let mut stats = Stats::default();

    for puzzle in all() {
        stats.total += 1;
        match puzzle.status {
            Status::Solved => stats.solved += 1,
            Status::Unsolved => stats.unsolved += 1,
            Status::Claimed => stats.claimed += 1,
            Status::Swept => stats.swept += 1,
        }
        if puzzle.has_pubkey() {
            stats.with_pubkey += 1;
        }
        if let Some(prize) = puzzle.prize {
            *stats.total_prize.entry(puzzle.chain).or_insert(0.0) += prize;
            if puzzle.status == Status::Unsolved {
                *stats.unsolved_prize.entry(puzzle.chain).or_insert(0.0) += prize;
            }
        }
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collection_parse_supports_aliases() {
        assert_eq!(Collection::parse("arweave").unwrap(), Collection::Arweave);
        assert_eq!(
            Collection::parse("hash_collision").unwrap(),
            Collection::HashCollision
        );
        assert_eq!(
            Collection::parse("peter_todd").unwrap(),
            Collection::HashCollision
        );
    }

    #[test]
    fn collection_all_matches_global_iterator() {
        let from_registry: Vec<_> = Collection::ALL
            .into_iter()
            .flat_map(Collection::all)
            .map(|p| p.id)
            .collect();
        let from_global: Vec<_> = all().map(|p| p.id).collect();

        assert_eq!(from_registry, from_global);
    }

    #[test]
    fn collection_get_handles_singletons_and_numbered_puzzles() {
        assert_eq!(
            Collection::parse("gsmg").unwrap().get("").unwrap().id,
            "gsmg"
        );
        assert_eq!(
            Collection::parse("bitaps").unwrap().get("").unwrap().id,
            "bitaps"
        );
        assert_eq!(
            Collection::parse("b1000").unwrap().get("66").unwrap().id,
            "b1000/66"
        );
    }

    #[test]
    fn collection_get_rejects_singleton_suffixes() {
        assert!(matches!(
            Collection::parse("gsmg").unwrap().get("extra"),
            Err(Error::NotFound(id)) if id == "gsmg/extra"
        ));
        assert!(matches!(
            Collection::parse("bitaps").unwrap().get("extra"),
            Err(Error::NotFound(id)) if id == "bitaps/extra"
        ));
    }

    #[test]
    fn global_get_rejects_singleton_slash_ids() {
        assert!(matches!(
            get("gsmg/extra"),
            Err(Error::NotFound(id)) if id == "gsmg/extra"
        ));
        assert!(matches!(
            get("bitaps/extra"),
            Err(Error::NotFound(id)) if id == "bitaps/extra"
        ));
        assert!(matches!(
            get("gsmg/"),
            Err(Error::NotFound(id)) if id == "gsmg/"
        ));
        assert!(matches!(
            get("bitaps/"),
            Err(Error::NotFound(id)) if id == "bitaps/"
        ));
    }

    #[test]
    fn global_get_rejects_extra_path_segments() {
        assert!(matches!(
            get("b1000/66/extra"),
            Err(Error::NotFound(id)) if id == "b1000/66/extra"
        ));
        assert!(matches!(
            get("hash_collision/sha256/extra"),
            Err(Error::NotFound(id)) if id == "hash_collision/sha256/extra"
        ));
    }
}
