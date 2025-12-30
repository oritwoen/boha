use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Btc1000File {
    puzzles: Vec<Btc1000Puzzle>,
}

#[derive(Debug, Deserialize)]
struct Btc1000Puzzle {
    bits: u16,
    address: String,
    btc: Option<f64>,
    status: String,
    #[allow(dead_code)]
    has_pubkey: Option<bool>,
    private_key: Option<String>,
    public_key: Option<String>,
    solve_date: Option<String>,
}

fn main() {
    println!("cargo:rerun-if-changed=data/b1000.toml");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("b1000_data.rs");

    let toml_content =
        fs::read_to_string("data/b1000.toml").expect("Failed to read data/b1000.toml");

    let data: Btc1000File = toml::from_str(&toml_content).expect("Failed to parse b1000.toml");

    let mut output = String::new();
    output.push_str("static PUZZLES: &[Puzzle] = &[\n");

    for puzzle in &data.puzzles {
        let status = match puzzle.status.as_str() {
            "solved" => "Status::Solved",
            "claimed" => "Status::Claimed",
            "swept" => "Status::Swept",
            _ => "Status::Unsolved",
        };

        let pubkey = match &puzzle.public_key {
            Some(pk) => format!("Some(\"{}\")", pk),
            None => "None".to_string(),
        };

        let private_key = match &puzzle.private_key {
            Some(pk) => format!("Some(\"{}\")", pk),
            None => "None".to_string(),
        };

        let prize = match puzzle.btc {
            Some(btc) => format!("Some({:.6})", btc),
            None => "None".to_string(),
        };

        let solve_date = match &puzzle.solve_date {
            Some(d) => format!("Some(\"{}\")", d),
            None => "None".to_string(),
        };

        output.push_str(&format!(
            r#"    Puzzle {{
        id: "b1000/{}",
        address: "{}",
        address_type: AddressType::P2PKH,
        status: {},
        pubkey: {},
        private_key: {},
        redeem_script: None,
        bits: Some({}),
        prize_btc: {},
        solve_date: {},
    }},
"#,
            puzzle.bits,
            puzzle.address,
            status,
            pubkey,
            private_key,
            puzzle.bits,
            prize,
            solve_date,
        ));
    }

    output.push_str("];\n");

    fs::write(&dest_path, output).expect("Failed to write b1000_data.rs");
}
