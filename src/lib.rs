mod collections;
mod puzzle;

#[cfg(feature = "balance")]
pub mod balance;

pub use collections::{b1000, gsmg, hash_collision};
pub use puzzle::{AddressType, Chain, IntoPuzzleNum, Pubkey, PubkeyFormat, Puzzle, Status};

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

pub fn get(id: &str) -> Result<&'static Puzzle> {
    if id == "gsmg" {
        return Ok(gsmg::get());
    }

    let parts: Vec<&str> = id.split('/').collect();
    if parts.len() < 2 {
        return Err(Error::NotFound(id.to_string()));
    }

    match parts[0] {
        "b1000" => {
            let num: u32 = parts[1]
                .parse()
                .map_err(|_| Error::NotFound(id.to_string()))?;
            b1000::get(num)
        }
        "hash_collision" | "peter_todd" => hash_collision::get(parts[1]),
        _ => Err(Error::NotFound(id.to_string())),
    }
}

pub fn all() -> impl Iterator<Item = &'static Puzzle> {
    b1000::all().chain(gsmg::all()).chain(hash_collision::all())
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
