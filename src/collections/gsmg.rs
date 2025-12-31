use crate::{AddressType, Chain, KeySource, Pubkey, PubkeyFormat, Puzzle, Status};

include!(concat!(env!("OUT_DIR"), "/gsmg_data.rs"));

pub fn get() -> &'static Puzzle {
    &PUZZLE
}

pub fn all() -> impl Iterator<Item = &'static Puzzle> {
    std::iter::once(&PUZZLE)
}

pub const fn count() -> usize {
    1
}
