//! RushWallet - Dmitri Kryptokov's 30 video brainwallet puzzles (2014).
//!
//! Each puzzle uses `sha256(passphrase)` as a private-key scalar and the
//! resulting uncompressed P2PKH address as the target. Twenty-eight
//! passphrases have been recovered locally from the contest videos,
//! frame artifacts, audio, and social clue carriers. Wallet #26 is claimed
//! on-chain with the passphrase still unknown; wallet #30 remains unclaimed.

#[allow(unused_imports)]
use crate::{
    Address, Assets, Author, Chain, Entropy, EntropySource, Error, Key, Passphrase, Profile,
    Pubkey, PubkeyFormat, Puzzle, RedeemScript, Result, Seed, Solver, Status, Transaction,
    TransactionType, Wif,
};

include!(concat!(env!("OUT_DIR"), "/rushwallet_data.rs"));

pub fn author() -> &'static Author {
    &AUTHOR
}

pub fn get(name: &str) -> Result<&'static Puzzle> {
    let search_id = if name.contains('/') {
        name.to_string()
    } else {
        format!("rushwallet/{}", name)
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
    PUZZLES.iter().filter(|p| p.status == Status::Solved)
}

pub fn unsolved() -> impl Iterator<Item = &'static Puzzle> {
    PUZZLES.iter().filter(|p| p.status == Status::Unsolved)
}

pub const fn count() -> usize {
    PUZZLES.len()
}
