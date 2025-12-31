//! B1000 - Bitcoin Puzzle Transaction (256 puzzles).
//!
//! Each puzzle N has a private key k where: 2^(N-1) <= k < 2^N

use crate::{
    AddressType, Chain, Error, IntoPuzzleNum, Pubkey, PubkeyFormat, Puzzle, Result, Status,
};

include!(concat!(env!("OUT_DIR"), "/b1000_data.rs"));

pub fn get(key: impl IntoPuzzleNum) -> Result<&'static Puzzle> {
    let number = key.into_puzzle_num().ok_or(Error::InvalidNumber(0))?;
    if !(1..=256).contains(&number) {
        return Err(Error::InvalidNumber(number));
    }
    PUZZLES
        .iter()
        .find(|p| p.bits == Some(number as u16))
        .ok_or_else(|| Error::NotFound(format!("b1000/{}", number)))
}

pub fn all() -> impl Iterator<Item = &'static Puzzle> {
    PUZZLES.iter()
}

pub fn solved() -> impl Iterator<Item = &'static Puzzle> {
    PUZZLES.iter().filter(|p| p.status == Status::Solved)
}

pub fn unsolved() -> impl Iterator<Item = &'static Puzzle> {
    PUZZLES.iter().filter(|p| p.status == Status::Unsolved)
}

pub fn with_pubkey() -> impl Iterator<Item = &'static Puzzle> {
    PUZZLES.iter().filter(|p| p.pubkey.is_some())
}

pub const fn count() -> usize {
    256
}

pub fn solved_count() -> usize {
    PUZZLES
        .iter()
        .filter(|p| p.status == Status::Solved)
        .count()
}

pub fn unsolved_count() -> usize {
    PUZZLES
        .iter()
        .filter(|p| p.status == Status::Unsolved)
        .count()
}
