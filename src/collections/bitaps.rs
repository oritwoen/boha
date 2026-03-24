//! Bitaps Mnemonic Challenge - Shamir Secret Sharing Scheme puzzle.

#[allow(unused_imports)]
use crate::{
    Address, Author, Chain, Entropy, EntropySource, Key, Passphrase, Profile, Pubkey, PubkeyFormat,
    Puzzle, RedeemScript, Seed, Share, Shares, Solver, Status, Transaction, TransactionType, Wif,
};

include!(concat!(env!("OUT_DIR"), "/bitaps_data.rs"));

pub fn author() -> &'static Author {
    &AUTHOR
}

pub fn get() -> &'static Puzzle {
    &PUZZLE
}

pub fn slice() -> &'static [Puzzle] {
    std::slice::from_ref(&PUZZLE)
}

pub fn all() -> impl Iterator<Item = &'static Puzzle> {
    slice().iter()
}

pub const fn count() -> usize {
    1
}
