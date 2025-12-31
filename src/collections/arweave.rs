use crate::{AddressType, Puzzle,Status};
include!(concat!(env!("OUT_DIR"), "/arweave_data.rs"));
pub fn all() -> impl Iterator<Item = &'static Puzzle> {
    PUZZLES.iter()
}

pub fn count() -> usize {
    PUZZLES.len()
}
pub fn get(id: &str) -> crate::Result<&'static Puzzle> {
    for puzzle in PUZZLES.iter() {
        if puzzle.id == id {
            return Ok(puzzle);
        }
    }
    Err(crate::Error::NotFound(id.to_string()))
}