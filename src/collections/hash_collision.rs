//! Peter Todd's hash collision bounties (P2SH).

use crate::{AddressType, Error, Puzzle, Result, Status};

static PUZZLES: &[Puzzle] = &[
    Puzzle {
        id: "hash_collision/sha1",
        address: "37k7toV1Nv4DfmQbmZ8KuZDQCYK9x5KpzP",
        address_type: AddressType::P2SH,
        status: Status::Claimed,
        pubkey: None,
        private_key: None,
        redeem_script: Some("6e879169a77ca787"),
        bits: None,
        prize_btc: Some(2.48),
        solve_date: Some("2017-02-23"),
    },
    Puzzle {
        id: "hash_collision/sha256",
        address: "35Snmmy3uhaer2gTboc81ayCip4m9DT4ko",
        address_type: AddressType::P2SH,
        status: Status::Unsolved,
        pubkey: None,
        private_key: None,
        redeem_script: Some("6e879169a87ca887"),
        bits: None,
        prize_btc: Some(0.277),
        solve_date: None,
    },
    Puzzle {
        id: "hash_collision/ripemd160",
        address: "3KyiQEGqqdb4nqfhUzGKN6KPhXmQsLNpay",
        address_type: AddressType::P2SH,
        status: Status::Unsolved,
        pubkey: None,
        private_key: None,
        redeem_script: Some("6e879169a67ca687"),
        bits: None,
        prize_btc: Some(0.116),
        solve_date: None,
    },
    Puzzle {
        id: "hash_collision/hash160",
        address: "39VXyuoc6SXYKp9TcAhoiN1mb4ns6z3Yu6",
        address_type: AddressType::P2SH,
        status: Status::Unsolved,
        pubkey: None,
        private_key: None,
        redeem_script: Some("6e879169a97ca987"),
        bits: None,
        prize_btc: Some(0.100),
        solve_date: None,
    },
    Puzzle {
        id: "hash_collision/hash256",
        address: "3DUQQvz4t57Jy7jxE86kyFcNpKtURNf1VW",
        address_type: AddressType::P2SH,
        status: Status::Unsolved,
        pubkey: None,
        private_key: None,
        redeem_script: Some("6e879169aa7caa87"),
        bits: None,
        prize_btc: Some(0.100),
        solve_date: None,
    },
    Puzzle {
        id: "hash_collision/op_abs",
        address: "3QsT6Sast6ghfsjZ9VJj9u8jkM2qTfDgHV",
        address_type: AddressType::P2SH,
        status: Status::Claimed,
        pubkey: None,
        private_key: None,
        redeem_script: Some("6e879169907c9087"),
        bits: None,
        prize_btc: None,
        solve_date: Some("2013-09-13"),
    },
];

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
