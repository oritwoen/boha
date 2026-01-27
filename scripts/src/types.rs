use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Address {
    pub value: String,
    pub kind: Option<String>,
    pub hash160: Option<String>,
    pub witness_program: Option<String>,
    pub redeem_script: Option<RedeemScript>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RedeemScript {
    pub script: String,
    pub hash: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Pubkey {
    pub value: String,
    pub format: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Key {
    pub bits: Option<u32>,
    pub hex: Option<String>,
    pub wif: Option<Wif>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Wif {
    pub decrypted: Option<String>,
    pub encrypted: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub tx_type: String,
    pub txid: String,
    pub date: Option<String>,
    pub amount: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Puzzle {
    pub name: Option<String>,
    pub chain: Option<String>,
    pub address: Address,
    pub status: String,
    pub pubkey: Option<Pubkey>,
    pub key: Option<Key>,
    pub transactions: Option<Vec<Transaction>>,
    pub prize: Option<f64>,
    pub start_date: Option<String>,
    pub solve_date: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Collection {
    pub puzzles: Option<Vec<Puzzle>>,
    pub puzzle: Option<Puzzle>,
}

/// Strip JSONC comments (// and /* */) from content while preserving strings
pub fn strip_jsonc_comments(content: &str) -> String {
    let mut result = String::new();
    let mut in_string = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut chars = content.chars().peekable();

    while let Some(c) = chars.next() {
        if in_line_comment {
            if c == '\n' {
                in_line_comment = false;
                result.push(c);
            }
            continue;
        }

        if in_block_comment {
            if c == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_block_comment = false;
            }
            continue;
        }

        if in_string {
            result.push(c);
            if c == '\\' {
                if let Some(next) = chars.next() {
                    result.push(next);
                }
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }

        if c == '"' {
            in_string = true;
            result.push(c);
            continue;
        }

        if c == '/' {
            match chars.peek() {
                Some('/') => {
                    chars.next();
                    in_line_comment = true;
                    continue;
                }
                Some('*') => {
                    chars.next();
                    in_block_comment = true;
                    continue;
                }
                _ => {}
            }
        }

        result.push(c);
    }

    result
}
