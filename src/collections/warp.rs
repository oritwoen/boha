//! Keybase WarpWallet challenges — deterministic brainwallet security tests.

#[allow(unused_imports)]
use crate::{
    Address, Author, Chain, Entropy, EntropySource, Error, Key, Passphrase, Profile, Pubkey,
    PubkeyFormat, Puzzle, RedeemScript, Result, Seed, Solver, Status, Transaction, TransactionType,
    Wif,
};

include!(concat!(env!("OUT_DIR"), "/warp_data.rs"));

pub fn author() -> &'static Author {
    &AUTHOR
}

pub fn get(name: &str) -> Result<&'static Puzzle> {
    let search_id = if name.contains('/') {
        name.to_string()
    } else {
        format!("warp/{}", name)
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
