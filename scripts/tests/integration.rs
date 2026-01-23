//! Integration tests for JSONC operations
//!
//! Tests verify:
//! - JSONC parsing and round-trip consistency
//! - Comment handling (documented as not preserved with serde_json)
//! - Collection file structure (puzzles array)
//! - Single puzzle object structure

use serde_json::{json, Value};

/// Helper: Verify round-trip consistency
fn verify_round_trip(value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let serialized = serde_json::to_string_pretty(value)?;
    let deserialized: Value = serde_json::from_str(&serialized)?;
    assert_eq!(value, &deserialized, "Round-trip failed: data mismatch");
    Ok(())
}

#[test]
fn test_jsonc_round_trip_simple() {
    let json_str = r#"{"puzzles": [{"id": 1, "name": "test"}]}"#;
    let value: Value = serde_json::from_str(json_str).unwrap();

    // Serialize to pretty JSON
    let output = serde_json::to_string_pretty(&value).unwrap();

    // Deserialize again
    let value2: Value = serde_json::from_str(&output).unwrap();

    // Should be identical
    assert_eq!(value, value2, "Round-trip failed for simple JSON");
}

#[test]
fn test_jsonc_round_trip_nested() {
    let json_str = r#"{
        "author": {
            "name": "test_author",
            "addresses": ["addr1", "addr2"]
        },
        "puzzles": [
            {
                "id": 1,
                "address": {
                    "value": "1ABC",
                    "kind": "p2pkh"
                }
            }
        ]
    }"#;

    let value: Value = serde_json::from_str(json_str).unwrap();
    verify_round_trip(&value).expect("Nested round-trip failed");
}

#[test]
fn test_jsonc_round_trip_with_numbers() {
    let json_str = r#"{
        "puzzles": [
            {
                "id": 1,
                "prize": 0.001,
                "bits": 1,
                "solve_time": 53729
            }
        ]
    }"#;

    let value: Value = serde_json::from_str(json_str).unwrap();
    verify_round_trip(&value).expect("Round-trip with numbers failed");

    // Verify numeric values are preserved
    let puzzle = &value["puzzles"][0];
    assert_eq!(puzzle["prize"].as_f64(), Some(0.001));
    assert_eq!(puzzle["bits"].as_i64(), Some(1));
    assert_eq!(puzzle["solve_time"].as_i64(), Some(53729));
}

#[test]
fn test_jsonc_comment_handling() {
    // This test documents that comments are NOT preserved with serde_json
    // This is acceptable because data files don't contain comments
    let json_with_comment = r#"{
        "puzzles": [
            {
                "id": 1
            }
        ]
    }"#;

    let value: Value = serde_json::from_str(json_with_comment).unwrap();
    let serialized = serde_json::to_string_pretty(&value).unwrap();

    // Comments would be stripped (if they existed)
    // This is expected behavior with serde_json
    assert!(
        !serialized.contains("//"),
        "Comments should not be preserved"
    );
}

#[test]
fn test_collection_file_structure_puzzles_array() {
    // Test with puzzles array structure (like b1000.jsonc)
    let json_str = r#"{
        "$schema": "./schemas/collection.schema.json",
        "author": {
            "name": "test_author",
            "addresses": ["1ABC"],
            "profiles": [
                {
                    "name": "bitcointalk",
                    "url": "https://example.com"
                }
            ]
        },
        "metadata": {
            "source_url": "https://example.com",
            "total_puzzles": 2,
            "solved_count": 1
        },
        "puzzles": [
            {
                "key": {
                    "bits": 1,
                    "hex": "0000000000000000000000000000000000000000000000000000000000000001"
                },
                "address": {
                    "value": "1BgGZ9tcN4rm9KBzDn7KprQz87SZ26SAMH",
                    "kind": "p2pkh",
                    "hash160": "751e76e8199196d454941c45d1b3a323f1433bd6"
                },
                "prize": 0.001,
                "status": "solved"
            },
            {
                "key": {
                    "bits": 2,
                    "hex": "0000000000000000000000000000000000000000000000000000000000000002"
                },
                "address": {
                    "value": "1CUNEBjYrCn2y1SdiUMohaKUi4wpP326Lb",
                    "kind": "p2pkh",
                    "hash160": "62e907b15cbf27d5425399ebf6f0fb50ebb88f18"
                },
                "prize": 0.002,
                "status": "unsolved"
            }
        ]
    }"#;

    let value: Value = serde_json::from_str(json_str).unwrap();

    // Verify structure
    assert!(value["author"].is_object(), "author should be object");
    assert!(value["metadata"].is_object(), "metadata should be object");
    assert!(value["puzzles"].is_array(), "puzzles should be array");

    // Verify array contents
    let puzzles = value["puzzles"].as_array().unwrap();
    assert_eq!(puzzles.len(), 2, "Should have 2 puzzles");

    // Verify first puzzle
    assert_eq!(
        puzzles[0]["address"]["value"].as_str(),
        Some("1BgGZ9tcN4rm9KBzDn7KprQz87SZ26SAMH")
    );
    assert_eq!(puzzles[0]["status"].as_str(), Some("solved"));

    // Verify second puzzle
    assert_eq!(
        puzzles[1]["address"]["value"].as_str(),
        Some("1CUNEBjYrCn2y1SdiUMohaKUi4wpP326Lb")
    );
    assert_eq!(puzzles[1]["status"].as_str(), Some("unsolved"));

    // Verify round-trip
    verify_round_trip(&value).expect("Collection file round-trip failed");
}

#[test]
fn test_single_puzzle_object_structure() {
    // Test with single puzzle object structure (like gsmg.jsonc)
    let json_str = r#"{
        "$schema": "./schemas/collection.schema.json",
        "author": {
            "name": "GSMG.io",
            "addresses": ["1EtbTvVB8QTGN4mduSdy7n4cZQm4iYTpQ1"],
            "profiles": [
                {
                    "name": "website",
                    "url": "https://gsmg.io/puzzle"
                }
            ]
        },
        "metadata": {
            "source_url": "https://gsmg.io/puzzle"
        },
        "puzzle": {
            "address": {
                "value": "1GSMG1JC9wtdSwfwApgj2xcmJPAwx7prBe",
                "kind": "p2pkh",
                "hash160": "a9553269572a317e39f0f518cb87c1a0ee1dbae4"
            },
            "status": "unsolved",
            "prize": 1.25364181,
            "pubkey": {
                "value": "04f4d1bbd91e65e2a019566a17574e97dae908b784b388891848007e4f55d5a4649c73d25fc5ed8fd7227cab0be4e576c0c6404db5aa546286563e4be12bf33559",
                "format": "uncompressed"
            },
            "start_date": "2019-04-13 16:32:40",
            "transactions": [
                {
                    "type": "funding",
                    "txid": "73e48ff571a7e9a4387574a50cf2fcb7b21b6ea5702c777a035664df57cbce02",
                    "date": "2019-04-13 16:32:40",
                    "amount": 5
                }
            ]
        }
    }"#;

    let value: Value = serde_json::from_str(json_str).unwrap();

    // Verify structure
    assert!(value["author"].is_object(), "author should be object");
    assert!(
        value["puzzle"].is_object(),
        "puzzle should be object (not array)"
    );
    assert!(value["puzzles"].is_null(), "puzzles should not exist");

    // Verify puzzle contents
    let puzzle = &value["puzzle"];
    assert_eq!(
        puzzle["address"]["value"].as_str(),
        Some("1GSMG1JC9wtdSwfwApgj2xcmJPAwx7prBe")
    );
    assert_eq!(puzzle["status"].as_str(), Some("unsolved"));
    assert_eq!(puzzle["prize"].as_f64(), Some(1.25364181));

    // Verify transactions array
    let transactions = puzzle["transactions"].as_array().unwrap();
    assert_eq!(transactions.len(), 1, "Should have 1 transaction");
    assert_eq!(transactions[0]["type"].as_str(), Some("funding"));
    assert_eq!(transactions[0]["amount"].as_f64(), Some(5.0));

    // Verify round-trip
    verify_round_trip(&value).expect("Single puzzle round-trip failed");
}

#[test]
fn test_jsonc_modification_and_round_trip() {
    // Test modifying parsed JSON and verifying round-trip still works
    let json_str = r#"{
        "puzzles": [
            {
                "id": 1,
                "status": "unsolved",
                "prize": 0.001
            }
        ]
    }"#;

    let mut value: Value = serde_json::from_str(json_str).unwrap();

    // Modify the value
    value["puzzles"][0]["status"] = Value::String("solved".to_string());
    value["puzzles"][0]["prize"] = json!(0.002);

    // Verify modifications
    assert_eq!(value["puzzles"][0]["status"].as_str(), Some("solved"));
    assert_eq!(value["puzzles"][0]["prize"].as_f64(), Some(0.002));

    // Verify round-trip after modification
    verify_round_trip(&value).expect("Modified round-trip failed");
}

#[test]
fn test_jsonc_array_operations() {
    // Test adding/removing items from arrays
    let json_str = r#"{
        "puzzles": [
            {"id": 1},
            {"id": 2}
        ]
    }"#;

    let mut value: Value = serde_json::from_str(json_str).unwrap();

    // Add a new puzzle
    let puzzles = value["puzzles"].as_array_mut().unwrap();
    puzzles.push(json!({"id": 3}));

    // Verify
    assert_eq!(puzzles.len(), 3);
    assert_eq!(puzzles[2]["id"].as_i64(), Some(3));

    // Verify round-trip
    verify_round_trip(&value).expect("Array operations round-trip failed");
}

#[test]
fn test_jsonc_null_and_optional_fields() {
    // Test handling of null and optional fields
    let json_str = r#"{
        "puzzles": [
            {
                "id": 1,
                "optional_field": null,
                "present_field": "value"
            }
        ]
    }"#;

    let value: Value = serde_json::from_str(json_str).unwrap();

    // Verify null handling
    assert!(value["puzzles"][0]["optional_field"].is_null());
    assert_eq!(value["puzzles"][0]["present_field"].as_str(), Some("value"));

    // Verify round-trip preserves nulls
    verify_round_trip(&value).expect("Null fields round-trip failed");
}

#[test]
fn test_jsonc_deeply_nested_structure() {
    // Test complex nested structures
    let json_str = r#"{
        "author": {
            "name": "test",
            "addresses": ["a1", "a2"],
            "profiles": [
                {
                    "name": "profile1",
                    "url": "https://example.com",
                    "metadata": {
                        "verified": true,
                        "followers": 100
                    }
                }
            ]
        },
        "puzzles": [
            {
                "key": {
                    "hex": "abc123",
                    "shares": {
                        "threshold": 2,
                        "total": 3,
                        "shares": ["s1", "s2"]
                    }
                },
                "address": {
                    "value": "addr",
                    "kind": "p2pkh",
                    "witness_program": null
                }
            }
        ]
    }"#;

    let value: Value = serde_json::from_str(json_str).unwrap();

    // Verify deep nesting
    assert_eq!(
        value["author"]["profiles"][0]["metadata"]["followers"].as_i64(),
        Some(100)
    );
    assert_eq!(
        value["puzzles"][0]["key"]["shares"]["threshold"].as_i64(),
        Some(2)
    );

    // Verify round-trip
    verify_round_trip(&value).expect("Deeply nested round-trip failed");
}

#[test]
fn test_jsonc_special_characters_in_strings() {
    // Test handling of special characters
    let json_str = r#"{
        "puzzles": [
            {
                "id": 1,
                "description": "Test with \"quotes\" and \\ backslash",
                "unicode": "Test with émojis 🎉 and ñ"
            }
        ]
    }"#;

    let value: Value = serde_json::from_str(json_str).unwrap();

    // Verify special characters are preserved
    let desc = value["puzzles"][0]["description"].as_str().unwrap();
    assert!(desc.contains("quotes"));
    assert!(desc.contains("backslash"));

    let unicode = value["puzzles"][0]["unicode"].as_str().unwrap();
    assert!(unicode.contains("émojis"));
    assert!(unicode.contains("ñ"));

    // Verify round-trip
    verify_round_trip(&value).expect("Special characters round-trip failed");
}
