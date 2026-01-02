//! Peter Todd's hash collision bounties (P2SH).

#[allow(unused_imports)]
use crate::{
    AddressType, Author, Chain, Error, KeySource, Puzzle, Result, Solver, Status, Transaction,
    TransactionType,
};

include!(concat!(env!("OUT_DIR"), "/hash_collision_data.rs"));

/// Returns the author/creator of the hash collision bounties.
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

pub fn all() -> impl Iterator<Item = &'static Puzzle> {
    PUZZLES.iter()
}

pub fn unsolved() -> impl Iterator<Item = &'static Puzzle> {
    PUZZLES.iter().filter(|p| p.status == Status::Unsolved)
}

pub const fn count() -> usize {
    6
}
