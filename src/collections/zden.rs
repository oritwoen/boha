#[allow(unused_imports)]
use crate::{
    Address, Assets, Author, Chain, Entropy, EntropySource, Error, Key, Passphrase, Profile,
    Puzzle, RedeemScript, Result, Seed, Solver, Status, Transaction, TransactionType, Wif,
};

include!(concat!(env!("OUT_DIR"), "/zden_data.rs"));

pub fn author() -> &'static Author {
    &AUTHOR
}

pub fn get(name: &str) -> Result<&'static Puzzle> {
    let search_id = if name.contains('/') {
        name.to_string()
    } else {
        format!("zden/{}", name)
    };

    PUZZLES
        .iter()
        .find(|p| p.id == search_id)
        .ok_or(Error::NotFound(search_id))
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

pub const fn count() -> usize {
    15
}
