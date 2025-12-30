use boha::{b1000, hash_collision, AddressType, Status};

#[test]
fn b1000_has_256_puzzles() {
    assert_eq!(b1000::all().count(), 256);
}

#[test]
fn b1000_puzzles_have_sequential_bits() {
    let bits: Vec<u16> = b1000::all().filter_map(|p| p.bits).collect();

    for i in 1u16..=256 {
        assert!(bits.contains(&i), "Missing puzzle with bits={}", i);
    }
}

#[test]
fn b1000_get_returns_correct_puzzle() {
    let p1 = b1000::get(1).unwrap();
    assert_eq!(p1.bits, Some(1));
    assert_eq!(p1.address, "1BgGZ9tcN4rm9KBzDn7KprQz87SZ26SAMH");
    assert_eq!(p1.status, Status::Solved);

    let p66 = b1000::get(66).unwrap();
    assert_eq!(p66.bits, Some(66));
    assert_eq!(p66.address, "13zb1hQbWVsc2S7ZTZnP2G4undNNpdh5so");

    let p256 = b1000::get(256).unwrap();
    assert_eq!(p256.bits, Some(256));
}

#[test]
fn b1000_get_accepts_multiple_types() {
    let by_u32 = b1000::get(66_u32).unwrap();
    let by_str = b1000::get("66").unwrap();
    let by_usize = b1000::get(66_usize).unwrap();

    assert_eq!(by_u32.id, by_str.id);
    assert_eq!(by_u32.id, by_usize.id);

    assert!(b1000::get("abc").is_err());
    assert!(b1000::get(-1_i32).is_err());
}

#[test]
fn b1000_key_range_valid() {
    let range = b1000::key_range(1).unwrap();
    assert_eq!(*range.start(), 1);
    assert_eq!(*range.end(), 1);

    let range = b1000::key_range(8).unwrap();
    assert_eq!(*range.start(), 128);
    assert_eq!(*range.end(), 255);

    let range = b1000::key_range(66).unwrap();
    assert_eq!(*range.start(), 1u128 << 65);
    assert_eq!(*range.end(), (1u128 << 66) - 1);

    assert!(b1000::key_range(0).is_none());
    assert!(b1000::key_range(129).is_none());
}

#[test]
fn b1000_all_addresses_start_with_1() {
    for puzzle in b1000::all() {
        assert!(
            puzzle.address.starts_with('1'),
            "BTC1000 address should start with 1: {}",
            puzzle.address
        );
        assert!(
            bs58::decode(puzzle.address).into_vec().is_ok(),
            "Invalid base58: {}",
            puzzle.address
        );
    }
}

#[test]
fn b1000_solved_have_private_keys() {
    for puzzle in b1000::solved() {
        assert!(
            puzzle.private_key.is_some(),
            "Solved puzzle {} missing private key",
            puzzle.id
        );
    }
}

#[test]
fn hash_collision_count() {
    assert_eq!(hash_collision::all().count(), 6);
}

#[test]
fn hash_collision_get_by_name() {
    let sha1 = hash_collision::get("sha1").unwrap();
    assert_eq!(sha1.address, "37k7toV1Nv4DfmQbmZ8KuZDQCYK9x5KpzP");
    assert_eq!(sha1.status, Status::Claimed);

    let sha256 = hash_collision::get("sha256").unwrap();
    assert_eq!(sha256.status, Status::Unsolved);
}

#[test]
fn hash_collision_all_p2sh() {
    for puzzle in hash_collision::all() {
        assert_eq!(puzzle.address_type, AddressType::P2SH);
        assert!(puzzle.redeem_script.is_some());
    }
}

#[test]
fn universal_get_works() {
    assert!(boha::get("b1000/66").is_ok());
    assert!(boha::get("hash_collision/sha256").is_ok());
    assert!(boha::get("peter_todd/sha256").is_ok());
}

#[test]
fn stats_are_reasonable() {
    let stats = boha::stats();
    assert!(stats.total > 250);
    assert!(stats.solved > 50);
    assert!(stats.unsolved > 50);
    assert!(stats.swept > 90);
    assert!(stats.total_btc > 100.0);
}

#[test]
fn all_puzzles_have_start_date() {
    for puzzle in boha::all() {
        assert!(
            puzzle.start_date.is_some(),
            "Puzzle {} missing start_date",
            puzzle.id
        );
    }
}

#[test]
fn start_date_format_valid() {
    let date_regex = regex::Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
    for puzzle in boha::all() {
        if let Some(date) = puzzle.start_date {
            assert!(
                date_regex.is_match(date),
                "Invalid start_date format for {}: {}",
                puzzle.id,
                date
            );
        }
    }
}

#[test]
fn start_date_before_solve_date() {
    for puzzle in boha::all() {
        if let (Some(start), Some(solve)) = (puzzle.start_date, puzzle.solve_date) {
            assert!(
                start <= solve,
                "Puzzle {} has start_date {} after solve_date {}",
                puzzle.id,
                start,
                solve
            );
        }
    }
}
