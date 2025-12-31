//! GSMG.IO 5 BTC Puzzle - Multi-phase cryptographic challenge.

use crate::{
    AddressType, Author, Chain, KeySource, Pubkey, PubkeyFormat, Puzzle, Status, Transaction,
    TransactionType,
};

include!(concat!(env!("OUT_DIR"), "/gsmg_data.rs"));

/// Returns the author/creator of the GSMG puzzle.
pub fn author() -> &'static Author {
    &AUTHOR
}

pub fn get() -> &'static Puzzle {
    &PUZZLE
}

pub fn all() -> impl Iterator<Item = &'static Puzzle> {
    std::iter::once(&PUZZLE)
}

pub const fn count() -> usize {
    1
}
