//! GSMG.IO 5 BTC Puzzle - Multi-phase cryptographic challenge.

#[allow(unused_imports)]
use crate::{
    Address, Assets, Author, Chain, Entropy, EntropySource, Key, Passphrase, Profile, Pubkey,
    PubkeyFormat, Puzzle, RedeemScript, Seed, Solver, Status, Transaction, TransactionType, Wif,
};

include!(concat!(env!("OUT_DIR"), "/gsmg_data.rs"));

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
