use assert_cmd::Command;
use predicates::prelude::*;

fn boha() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_boha"));
    cmd.env("NO_COLOR", "1");
    cmd
}

mod stats {
    use super::*;

    #[test]
    fn default_format() {
        boha()
            .arg("stats")
            .assert()
            .success()
            .stdout(predicate::str::contains("Total puzzles"))
            .stdout(predicate::str::contains("Solved"))
            .stdout(predicate::str::contains("Unsolved"));
    }

    #[test]
    fn json_format() {
        boha()
            .args(["--output", "json", "stats"])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"total\":"))
            .stdout(predicate::str::contains("\"solved\":"))
            .stdout(predicate::str::contains("\"unsolved\":"));
    }

    #[test]
    fn yaml_format() {
        boha()
            .args(["--output", "yaml", "stats"])
            .assert()
            .success()
            .stdout(predicate::str::contains("total:"))
            .stdout(predicate::str::contains("solved:"));
    }

    #[test]
    fn csv_format() {
        boha()
            .args(["--output", "csv", "stats"])
            .assert()
            .success()
            .stdout(predicate::str::contains("total,"));
    }

    #[test]
    fn jsonl_format() {
        boha()
            .args(["--output", "jsonl", "stats"])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"total\":"));
    }
}

mod list {
    use super::*;

    #[test]
    fn all_collections() {
        boha()
            .arg("list")
            .assert()
            .success()
            .stdout(predicate::str::contains("b1000/"))
            .stdout(predicate::str::contains("gsmg"))
            .stdout(predicate::str::contains("hash_collision/"));
    }

    #[test]
    fn b1000_collection() {
        boha()
            .args(["list", "b1000"])
            .assert()
            .success()
            .stdout(predicate::str::contains("b1000/1"))
            .stdout(predicate::str::contains("b1000/66"))
            .stdout(predicate::str::contains("b1000/256"));
    }

    #[test]
    fn gsmg_collection() {
        boha()
            .args(["list", "gsmg"])
            .assert()
            .success()
            .stdout(predicate::str::contains("gsmg"))
            .stdout(predicate::str::contains("1GSMG"));
    }

    #[test]
    fn hash_collision_collection() {
        boha()
            .args(["list", "hash_collision"])
            .assert()
            .success()
            .stdout(predicate::str::contains("hash_collision/sha1"))
            .stdout(predicate::str::contains("hash_collision/sha256"));
    }

    #[test]
    fn peter_todd_alias() {
        boha()
            .args(["list", "peter_todd"])
            .assert()
            .success()
            .stdout(predicate::str::contains("hash_collision/"));
    }

    #[test]
    fn unsolved_filter() {
        boha()
            .args(["list", "b1000", "--unsolved"])
            .assert()
            .success()
            .stdout(predicate::str::contains("b1000/71"))
            .stdout(predicate::str::is_match(r"unsolved").unwrap());
    }

    #[test]
    fn solved_filter() {
        boha()
            .args(["list", "b1000", "--solved"])
            .assert()
            .success()
            .stdout(predicate::str::contains("b1000/66"))
            .stdout(predicate::str::is_match(r"solved").unwrap());
    }

    #[test]
    fn with_pubkey_filter() {
        boha()
            .args(["list", "b1000", "--with-pubkey"])
            .assert()
            .success()
            .stdout(predicate::str::contains("b1000/"));
    }

    #[test]
    fn json_format() {
        boha()
            .args(["--output", "json", "list", "gsmg"])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"id\": \"gsmg\""))
            .stdout(predicate::str::contains("\"address\":"));
    }
}

mod show {
    use super::*;

    #[test]
    fn b1000_puzzle() {
        boha()
            .args(["show", "b1000/66"])
            .assert()
            .success()
            .stdout(predicate::str::contains("b1000/66"))
            .stdout(predicate::str::contains("13zb1hQbWVsc2S7"))
            .stdout(predicate::str::contains("Solved"));
    }

    #[test]
    fn gsmg_puzzle() {
        boha()
            .args(["show", "gsmg"])
            .assert()
            .success()
            .stdout(predicate::str::contains("gsmg"))
            .stdout(predicate::str::contains(
                "1GSMG1JC9wtdSwfwApgj2xcmJPAwx7prBe",
            ));
    }

    #[test]
    fn hash_collision_puzzle() {
        boha()
            .args(["show", "hash_collision/sha256"])
            .assert()
            .success()
            .stdout(predicate::str::contains("hash_collision/sha256"))
            .stdout(predicate::str::contains("Redeem Script"));
    }

    #[test]
    fn json_format() {
        boha()
            .args(["--output", "json", "show", "b1000/1"])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"id\": \"b1000/1\""))
            .stdout(predicate::str::contains("\"chain\": \"bitcoin\""));
    }

    #[test]
    fn unknown_puzzle_error() {
        boha()
            .args(["show", "b1000/999"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error:"));
    }

    #[test]
    fn invalid_id_error() {
        boha()
            .args(["show", "nonexistent/puzzle"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error:"));
    }
}

mod range {
    use super::*;

    #[test]
    fn puzzle_66() {
        boha()
            .args(["range", "66"])
            .assert()
            .success()
            .stdout(predicate::str::contains("66"))
            .stdout(predicate::str::contains("0x2"))
            .stdout(predicate::str::contains("13zb1hQbWVsc2S7"));
    }

    #[test]
    fn puzzle_1() {
        boha()
            .args(["range", "1"])
            .assert()
            .success()
            .stdout(predicate::str::contains("0x1"))
            .stdout(predicate::str::contains("0x1"));
    }

    #[test]
    fn json_format() {
        boha()
            .args(["--output", "json", "range", "66"])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"puzzle\": 66"))
            .stdout(predicate::str::contains("\"start\":"))
            .stdout(predicate::str::contains("\"end\":"));
    }

    #[test]
    fn invalid_puzzle_error() {
        boha()
            .args(["range", "999"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error:"));
    }
}

mod author {
    use super::*;

    #[test]
    fn b1000_author() {
        boha()
            .args(["author", "b1000"])
            .assert()
            .success()
            .stdout(predicate::str::contains("saatoshi_rising"))
            .stdout(predicate::str::contains(
                "1Czoy8xtddvcGrEhUUCZDQ9QqdRfKh697F",
            ));
    }

    #[test]
    fn gsmg_author() {
        boha()
            .args(["author", "gsmg"])
            .assert()
            .success()
            .stdout(predicate::str::contains("GSMG.io"))
            .stdout(predicate::str::contains("https://gsmg.io/puzzle"));
    }

    #[test]
    fn hash_collision_author() {
        boha()
            .args(["author", "hash_collision"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Peter Todd"));
    }

    #[test]
    fn peter_todd_alias() {
        boha()
            .args(["author", "peter_todd"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Peter Todd"));
    }

    #[test]
    fn json_format() {
        boha()
            .args(["--output", "json", "author", "b1000"])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"name\":"))
            .stdout(predicate::str::contains("\"addresses\":"))
            .stdout(predicate::str::contains("\"profiles\":"));
    }

    #[test]
    fn unknown_collection_error() {
        boha()
            .args(["author", "unknown"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error:"))
            .stderr(predicate::str::contains("Unknown collection"));
    }
}

mod help {
    use super::*;

    #[test]
    fn help_flag() {
        boha()
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("Crypto bounties"))
            .stdout(predicate::str::contains("list"))
            .stdout(predicate::str::contains("show"))
            .stdout(predicate::str::contains("stats"))
            .stdout(predicate::str::contains("author"));
    }

    #[test]
    fn version_flag() {
        boha()
            .arg("--version")
            .assert()
            .success()
            .stdout(predicate::str::contains("boha"));
    }
}

#[cfg(feature = "balance")]
mod balance {
    use super::*;

    #[test]
    fn help_shows_balance_command() {
        boha()
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("balance"));
    }

    #[test]
    fn unknown_puzzle_error() {
        boha()
            .args(["balance", "b1000/999"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error:"));
    }

    #[test]
    fn invalid_collection_error() {
        boha()
            .args(["balance", "nonexistent/puzzle"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error:"));
    }

    #[test]
    fn missing_id_error() {
        boha().arg("balance").assert().failure();
    }

    // Run ignored tests with: cargo test --features cli,balance -- --ignored

    #[test]
    #[ignore]
    fn fetch_solved_puzzle_table_format() {
        boha()
            .args(["balance", "b1000/1"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Address"))
            .stdout(predicate::str::contains("Confirmed"))
            .stdout(predicate::str::contains("Total"));
    }

    #[test]
    #[ignore]
    fn fetch_unsolved_puzzle() {
        boha()
            .args(["balance", "b1000/71"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Address"));
    }

    #[test]
    #[ignore]
    fn json_format() {
        boha()
            .args(["--output", "json", "balance", "b1000/66"])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"address\":"))
            .stdout(predicate::str::contains("\"confirmed\":"))
            .stdout(predicate::str::contains("\"confirmed_btc\":"))
            .stdout(predicate::str::contains("\"unconfirmed\":"))
            .stdout(predicate::str::contains("\"total_btc\":"));
    }

    #[test]
    #[ignore]
    fn jsonl_format() {
        boha()
            .args(["--output", "jsonl", "balance", "b1000/66"])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"address\":"))
            .stdout(predicate::str::contains("\"confirmed\":"));
    }

    #[test]
    #[ignore]
    fn yaml_format() {
        boha()
            .args(["--output", "yaml", "balance", "b1000/66"])
            .assert()
            .success()
            .stdout(predicate::str::contains("address:"))
            .stdout(predicate::str::contains("confirmed:"))
            .stdout(predicate::str::contains("total_btc:"));
    }

    #[test]
    #[ignore]
    fn csv_format() {
        boha()
            .args(["--output", "csv", "balance", "b1000/66"])
            .assert()
            .success()
            .stdout(predicate::str::contains("address,"))
            .stdout(predicate::str::contains("confirmed,"));
    }

    #[test]
    #[ignore]
    fn gsmg_puzzle() {
        boha()
            .args(["balance", "gsmg"])
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "1GSMG1JC9wtdSwfwApgj2xcmJPAwx7prBe",
            ));
    }

    #[test]
    #[ignore]
    fn hash_collision_puzzle() {
        boha()
            .args(["balance", "hash_collision/sha256"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Address"));
    }
}

mod search {
    use super::*;

    #[test]
    fn basic_substring_search() {
        boha()
            .args(["search", "1BgGZ"])
            .assert()
            .success()
            .stdout(predicate::str::contains("b1000/1"))
            .stdout(predicate::str::contains(
                "1BgGZ9tcN4rm9KBzDn7KprQz87SZ26SAMH",
            ));
    }

    #[test]
    fn exact_match() {
        boha()
            .args(["search", "--exact", "b1000/66"])
            .assert()
            .success()
            .stdout(predicate::str::contains("b1000/66"))
            .stdout(predicate::str::contains("b1000/1").not())
            .stdout(predicate::str::contains("b1000/67").not());
    }

    #[test]
    fn case_insensitive_default() {
        boha()
            .args(["search", "gsmg"])
            .assert()
            .success()
            .stdout(predicate::str::contains("gsmg"))
            .stdout(predicate::str::contains(
                "1GSMG1JC9wtdSwfwApgj2xcmJPAwx7prBe",
            ));
    }

    #[test]
    fn case_insensitive_uppercase() {
        boha()
            .args(["search", "GSMG"])
            .assert()
            .success()
            .stdout(predicate::str::contains("gsmg"));
    }

    #[test]
    fn case_sensitive_no_match() {
        boha()
            .args(["search", "--case-sensitive", "GSMG"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("No puzzles found"));
    }

    #[test]
    fn case_sensitive_match() {
        boha()
            .args(["search", "--case-sensitive", "gsmg"])
            .assert()
            .success()
            .stdout(predicate::str::contains("gsmg"));
    }

    #[test]
    fn collection_filter() {
        boha()
            .args(["search", "--collection", "zden", "level"])
            .assert()
            .success()
            .stdout(predicate::str::contains("zden/"))
            .stdout(predicate::str::contains("Level"))
            .stdout(predicate::str::contains("b1000/").not())
            .stdout(predicate::str::contains("hash_collision/").not());
    }

    #[test]
    fn collection_unknown_error() {
        boha()
            .args(["search", "--collection", "nonexistent", "test"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error:"));
    }

    #[test]
    fn limit_results() {
        boha()
            .args(["search", "--limit", "3", "1"])
            .assert()
            .success()
            .stdout(predicate::str::contains("b1000/1"))
            .stdout(predicate::str::contains("b1000/10"))
            .stdout(predicate::str::contains("b1000/11"))
            .stdout(predicate::str::contains("b1000/12").not());
    }

    #[test]
    fn empty_query_error() {
        boha()
            .args(["search", ""])
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error:"));
    }

    #[test]
    fn whitespace_query_error() {
        boha()
            .args(["search", "  "])
            .assert()
            .failure()
            .stderr(predicate::str::contains("Error:"));
    }

    #[test]
    fn json_output_has_matched_fields() {
        boha()
            .args(["-o", "json", "search", "sha256"])
            .assert()
            .success()
            .stdout(predicate::str::contains("matched_fields"));
    }

    #[test]
    fn no_results_table() {
        boha()
            .args(["search", "xyznonexistent123456"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("No puzzles found"));
    }

    #[test]
    fn no_results_json() {
        boha()
            .args(["-o", "json", "search", "xyznonexistent123456"])
            .assert()
            .success()
            .stdout(predicate::str::diff("[]\n"));
    }
}
mod export {
    use super::*;

    #[test]
    fn export_default() {
        boha()
            .args(["export", "-o", "json"])
            .assert()
            .success()
            .stdout(predicate::str::contains("version"));
    }

    #[test]
    fn export_collection_filter() {
        boha()
            .args(["export", "-o", "json", "b1000"])
            .assert()
            .success()
            .stdout(predicate::str::contains("b1000"))
            .stdout(predicate::str::contains("\"name\":\"b1000\""));
    }

    #[test]
    fn export_status_filter() {
        boha()
            .args(["export", "-o", "json", "--unsolved"])
            .assert()
            .success()
            .stdout(predicate::str::contains("version"));
    }

    #[test]
    fn export_csv_error() {
        boha()
            .args(["export", "-o", "csv"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("not supported"))
            .stderr(predicate::str::contains("boha list"));
    }

    #[test]
    fn export_yaml_error() {
        boha()
            .args(["export", "-o", "yaml"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("not supported"))
            .stderr(predicate::str::contains("boha list"));
    }

    #[test]
    fn export_table_maps_to_json() {
        boha()
            .args(["export", "-o", "table"])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"version\""));
    }

    #[test]
    fn export_jsonl() {
        boha()
            .args(["export", "-o", "jsonl"])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"id\""));
    }

    #[test]
    fn export_compact() {
        boha()
            .args(["export", "-o", "json", "--compact"])
            .assert()
            .success()
            .stdout(predicate::str::contains("version"));
    }

    #[test]
    fn export_no_authors() {
        boha()
            .args(["export", "-o", "json", "--no-authors"])
            .assert()
            .success()
            .stdout(predicate::str::contains("version"));
    }

    #[test]
    fn export_no_stats() {
        boha()
            .args(["export", "-o", "json", "--no-stats"])
            .assert()
            .success()
            .stdout(predicate::str::contains("version"));
    }

    #[test]
    fn export_solved_filter() {
        boha()
            .args(["export", "-o", "json", "--solved"])
            .assert()
            .success()
            .stdout(predicate::str::contains("version"));
    }

    #[test]
    fn export_multiple_collections() {
        boha()
            .args(["export", "-o", "json", "b1000", "gsmg"])
            .assert()
            .success()
            .stdout(predicate::str::contains("b1000"))
            .stdout(predicate::str::contains("gsmg"));
    }

    #[test]
    fn export_all_alias() {
        boha()
            .args(["export", "-o", "json", "all"])
            .assert()
            .success()
            .stdout(predicate::str::contains("b1000"))
            .stdout(predicate::str::contains("zden"));
    }

    #[test]
    fn export_peter_todd_alias() {
        boha()
            .args(["export", "-o", "json", "peter_todd"])
            .assert()
            .success()
            .stdout(predicate::str::contains("hash_collision"));
    }
}

mod verify {
    use super::*;

    #[test]
    fn verify_result_success() {
        // RED: This should fail - verify command doesn't exist yet
        boha()
            .args(["verify", "b1000/66"])
            .assert()
            .success()
            .stdout(predicate::str::contains("verified"));
    }

    #[test]
    fn verify_result_no_key() {
        // RED: This should fail - verify command doesn't exist yet
        boha()
            .args(["verify", "b1000/71"])
            .assert()
            .code(2)
            .stderr(predicate::str::contains("no private key"));
    }

    #[test]
    fn verify_result_not_found() {
        // RED: This should fail - verify command doesn't exist yet
        boha().args(["verify", "invalid/id"]).assert().code(1);
    }
}

#[cfg(test)]
mod verify_bitcoin {
    use boha::verify::{verify_bitcoin_address, VerifyError};
    use boha::PubkeyFormat;

    #[test]
    fn verify_bitcoin_p2pkh_success() {
        // b1000/66 - known solved puzzle
        let hex = "000000000000000000000000000000000000000000000002832ed74f2b5e35ee";
        let expected = "13zb1hQbWVsc2S7ZTZnP2G4undNNpdh5so";

        let result = verify_bitcoin_address(hex, expected, PubkeyFormat::Compressed);
        match &result {
            Ok(addr) => println!("Success! Derived: {}", addr),
            Err(e) => println!("Error: {:?}", e),
        }
        assert!(
            result.is_ok(),
            "Verification should succeed for b1000/66: {:?}",
            result
        );
    }

    #[test]
    fn verify_bitcoin_p2wpkh_success() {
        // This test will use a known P2WPKH puzzle when we have hex data
        // For now, placeholder
    }

    #[test]
    fn verify_bitcoin_invalid_key() {
        let hex = "invalid_hex";
        let expected = "13zb1hQbWVsc2S7ZTZnP2G4undNNpdh5so";

        let result = verify_bitcoin_address(hex, expected, PubkeyFormat::Compressed);
        assert!(result.is_err(), "Should fail with invalid hex");
    }
}

#[cfg(test)]
mod verify_multichain {
    use boha::verify::{verify_decred_address, verify_ethereum_address, verify_litecoin_address};
    use boha::PubkeyFormat;

    #[test]
    fn verify_ethereum_success() {
        let hex = "fad9c8855b740a0b7ed4c221dbad0f33a83a49cad6b3fe8d5817ac83d38b6a19";
        let expected = "0x96216849c49358b10257cb55b28ea603c874b05e";

        let result = verify_ethereum_address(hex, expected);
        assert!(
            result.is_ok(),
            "Ethereum verification should succeed: {:?}",
            result
        );
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn verify_ethereum_case_insensitive() {
        let hex = "fad9c8855b740a0b7ed4c221dbad0f33a83a49cad6b3fe8d5817ac83d38b6a19";
        let expected_upper = "0x96216849C49358B10257CB55B28EA603C874B05E";

        let result = verify_ethereum_address(hex, expected_upper);
        assert!(
            result.is_ok(),
            "Ethereum verification should be case-insensitive: {:?}",
            result
        );
    }

    #[test]
    fn verify_ethereum_invalid_key() {
        let hex = "invalid_hex";
        let expected = "0x96216849c49358b10257cb55b28ea603c874b05e";

        let result = verify_ethereum_address(hex, expected);
        assert!(result.is_err(), "Should fail with invalid hex");
    }

    #[test]
    fn verify_litecoin_p2pkh_success() {
        let hex = "0000000000000000000000000000000000000000000000000000000000000001";
        let expected = "LVuDpNCSSj6pQ7t9Pv6d6sUkLKoqDEVUnJ";

        let result = verify_litecoin_address(hex, expected, PubkeyFormat::Compressed);
        assert!(
            result.is_ok(),
            "Litecoin P2PKH verification should succeed: {:?}",
            result
        );
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn verify_litecoin_invalid_key() {
        let hex = "not_a_valid_hex_string";
        let expected = "LVuDpNCSSj6pQ7t9Pv6d6sUkLKoqDEVUnJ";

        let result = verify_litecoin_address(hex, expected, PubkeyFormat::Compressed);
        assert!(result.is_err(), "Should fail with invalid hex");
    }

    #[test]
    fn verify_decred_p2pkh_success() {
        let hex = "0000000000000000000000000000000000000000000000000000000000000001";
        let expected = "DsmcYVbP1Nmag2H4AS17UTvmWXmGeA7nLDx";

        let result = verify_decred_address(hex, expected, PubkeyFormat::Compressed);
        assert!(
            result.is_ok(),
            "Decred P2PKH verification should succeed: {:?}",
            result
        );
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn verify_decred_invalid_key() {
        let hex = "zzz";
        let expected = "DsmcYVbP1Nmag2H4AS17UTvmWXmGeA7nLDx";

        let result = verify_decred_address(hex, expected, PubkeyFormat::Compressed);
        assert!(result.is_err(), "Should fail with invalid hex");
    }
}

#[cfg(test)]
mod verify_wif {
    use boha::verify::verify_wif;

    #[test]
    fn verify_wif_compressed_success() {
        // b1000/66 WIF (compressed)
        let wif = "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qZfFoWMiwBt943V7CQeX";
        let expected = "13zb1hQbWVsc2S7ZTZnP2G4undNNpdh5so";

        let result = verify_wif(wif, expected);
        assert!(
            result.is_ok(),
            "WIF verification should succeed: {:?}",
            result
        );
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn verify_wif_uncompressed_success() {
        // zden Level 1 WIF (uncompressed)
        let wif = "5JMTiDVHj3pj8VfaTe6pDtD9byZr6too3PD3AGBJrXF1hVsitc8";
        let expected = "1cryptommoqPHVNHuxVQG3bzujnRJYB1D";

        let result = verify_wif(wif, expected);
        assert!(
            result.is_ok(),
            "WIF verification should succeed: {:?}",
            result
        );
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn verify_wif_invalid_checksum() {
        let wif = "5HpHagT65TZzG1PH3CSu63k8DbpvD8s5ip4nEB3kEsreAnchuDX"; // Changed last char
        let expected = "1BgGZ9tcN4rm9KBzDn7KprQz87SZ26SAMH";

        let result = verify_wif(wif, expected);
        assert!(result.is_err(), "Should fail with invalid checksum");
    }

    #[test]
    fn verify_wif_invalid_base58() {
        let wif = "invalid_wif_with_bad_chars_0OIl";
        let expected = "1BgGZ9tcN4rm9KBzDn7KprQz87SZ26SAMH";

        let result = verify_wif(wif, expected);
        assert!(result.is_err(), "Should fail with invalid base58");
    }

    #[test]
    fn verify_wif_wrong_network() {
        // Litecoin WIF (starts with T or 6)
        let wif = "T3tFDaAKKmPKvCGEzP6Yk4Gqy3Ry8Qz9Qz9Qz9Qz9Qz9Qz9Qz9";
        let expected = "1BgGZ9tcN4rm9KBzDn7KprQz87SZ26SAMH";

        let result = verify_wif(wif, expected);
        assert!(result.is_err(), "Should fail with wrong network byte");
    }
}

#[cfg(test)]
mod verify_seed {
    use boha::verify::verify_seed;
    use boha::PubkeyFormat;

    #[test]
    fn verify_seed_standard_path() {
        // BIP39 test vector: "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
        // Derivation path: m/44'/0'/0'/0/0 (BIP44 standard)
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let path = "m/44'/0'/0'/0/0";
        let expected = "1LqBGSKuX5yYUonjxT5qGfpUsXKYYWeabA";

        let result = verify_seed(phrase, path, expected, PubkeyFormat::Compressed);
        assert!(
            result.is_ok(),
            "Seed verification should succeed: {:?}",
            result
        );
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn verify_seed_segwit_path() {
        // Same seed, different path: m/84'/0'/0'/0/0 (BIP84 SegWit)
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let path = "m/84'/0'/0'/0/0";
        let expected = "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu";

        let result = verify_seed(phrase, path, expected, PubkeyFormat::Compressed);
        assert!(
            result.is_ok(),
            "Seed verification with SegWit path should succeed: {:?}",
            result
        );
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn verify_seed_invalid_phrase() {
        let phrase = "invalid mnemonic phrase that is not valid";
        let path = "m/44'/0'/0'/0/0";
        let expected = "1LqBGSKuX5yYUonjxT5qGfpUsXKYYWeabA";

        let result = verify_seed(phrase, path, expected, PubkeyFormat::Compressed);
        assert!(result.is_err(), "Should fail with invalid mnemonic");
    }

    #[test]
    fn verify_seed_invalid_path() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let path = "invalid/path/format";
        let expected = "1LqBGSKuX5yYUonjxT5qGfpUsXKYYWeabA";

        let result = verify_seed(phrase, path, expected, PubkeyFormat::Compressed);
        assert!(result.is_err(), "Should fail with invalid derivation path");
    }
}
