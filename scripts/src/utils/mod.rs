pub mod esplora;
pub mod etherscan;

use chrono::{DateTime, Utc};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;
use toml_edit::{Array, InlineTable, Value};

pub const RATE_LIMIT_DELAY: Duration = Duration::from_secs(3);
pub const RETRY_DELAY: Duration = Duration::from_secs(60);

#[derive(Debug, Clone)]
pub struct Transaction {
    pub tx_type: String,
    pub txid: String,
    pub date: Option<String>,
    pub amount: Option<f64>,
}

pub fn timestamp_to_date(timestamp: i64) -> String {
    let dt = DateTime::<Utc>::from_timestamp(timestamp, 0).unwrap_or_default();
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn cache_path(collection: &str, address: &str) -> PathBuf {
    Path::new("../data/cache")
        .join(collection)
        .join(format!("{}.json", address))
}

pub fn tx_type_sort_priority(tx_type: &str) -> u8 {
    match tx_type {
        "funding" => 0,
        "increase" => 1,
        "decrease" => 2,
        "pubkey_reveal" => 3,
        "claim" | "sweep" => 4,
        _ => 5,
    }
}

pub fn is_terminal_tx_type(tx_type: &str) -> bool {
    matches!(tx_type, "claim" | "sweep")
}

fn normalize_txid(txid: &str) -> String {
    txid.to_lowercase().trim_start_matches("0x").to_string()
}

pub fn merge_transactions(
    existing: Vec<Transaction>,
    fresh_from_api: Vec<Transaction>,
) -> Vec<Transaction> {
    use std::collections::HashMap;

    let mut transactions_by_txid: HashMap<String, Transaction> = HashMap::new();

    for tx in existing {
        transactions_by_txid.insert(normalize_txid(&tx.txid), tx);
    }

    for tx in fresh_from_api {
        transactions_by_txid.insert(normalize_txid(&tx.txid), tx);
    }

    let mut merged: Vec<Transaction> = transactions_by_txid.into_values().collect();

    merged.sort_by(|a, b| match a.date.cmp(&b.date) {
        std::cmp::Ordering::Equal => {
            tx_type_sort_priority(&a.tx_type).cmp(&tx_type_sort_priority(&b.tx_type))
        }
        other => other,
    });

    if let Some(terminal_idx) = merged.iter().position(|t| is_terminal_tx_type(&t.tx_type)) {
        merged.truncate(terminal_idx + 1);
    }

    merged
}

pub fn transaction_to_inline_table(tx: &Transaction) -> InlineTable {
    let mut table = InlineTable::new();
    table.insert("type", Value::from(tx.tx_type.as_str()));
    table.insert("txid", Value::from(tx.txid.as_str()));
    if let Some(date) = &tx.date {
        table.insert("date", Value::from(date.as_str()));
    }
    if let Some(amount) = tx.amount {
        table.insert("amount", Value::from(amount));
    }
    table
}

pub fn transactions_to_array(transactions: &[Transaction]) -> Array {
    let mut array = Array::new();
    for tx in transactions {
        array.push(Value::InlineTable(transaction_to_inline_table(tx)));
    }
    array
}

pub fn extract_existing_transactions(table: &toml_edit::Table) -> Vec<Transaction> {
    let mut result = Vec::new();

    if let Some(txs) = table.get("transactions") {
        if let Some(arr) = txs.as_array() {
            for item in arr.iter() {
                if let Some(inline) = item.as_inline_table() {
                    let tx_type = inline
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let txid = inline
                        .get("txid")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let date = inline.get("date").and_then(|v| v.as_str()).map(String::from);
                    let amount = inline.get("amount").and_then(|v| v.as_float());

                    if !txid.is_empty() {
                        result.push(Transaction {
                            tx_type,
                            txid,
                            date,
                            amount,
                        });
                    }
                }
            }
        }
    }

    result
}

pub fn extract_author_addresses(doc: &toml_edit::DocumentMut) -> HashSet<String> {
    let mut addresses = HashSet::new();

    if let Some(author) = doc.get("author") {
        if let Some(table) = author.as_table() {
            if let Some(addrs) = table.get("addresses") {
                if let Some(arr) = addrs.as_array() {
                    for addr in arr.iter() {
                        if let Some(s) = addr.as_str() {
                            addresses.insert(s.to_string());
                        }
                    }
                }
            }
        }
    }

    addresses
}
