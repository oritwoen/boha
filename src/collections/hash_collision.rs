//! Peter Todd's hash collision bounties (P2SH).

#[allow(unused_imports)]
use crate::{
    Address, Author, Chain, Entropy, EntropySource, Error, Key, Passphrase, Profile, Puzzle,
    RedeemScript, Result, Seed, Solver, Status, Transaction, TransactionType, Wif,
};

include!(concat!(env!("OUT_DIR"), "/hash_collision_data.rs"));

pub fn author() -> &'static Author {
    &AUTHOR
}

pub fn get(name: &str) -> Result<&'static Puzzle> {
    let search_id = if name.contains('/') {
        name.to_string()
    } else {
        format!("hash_collision/{}", name)
    };

    PUZZLES
        .iter()
        .find(|p| p.id == search_id)
        .ok_or(Error::NotFound(search_id))
}

pub fn slice() -> &'static [Puzzle] {
    PUZZLES
}

pub fn all() -> impl Iterator<Item = &'static Puzzle> {
    slice().iter()
}

pub fn solved() -> impl Iterator<Item = &'static Puzzle> {
    PUZZLES
        .iter()
        .filter(|p| matches!(p.status, Status::Solved | Status::Claimed))
}

pub fn unsolved() -> impl Iterator<Item = &'static Puzzle> {
    PUZZLES.iter().filter(|p| p.status == Status::Unsolved)
}

pub const fn count() -> usize {
    PUZZLES.len()
}
