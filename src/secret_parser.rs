use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::BufReader;

use crate::SECRET;

#[derive(Serialize, Deserialize, Debug)]
pub struct Secret {
    pub test: String,
    pub k_addr: String,
    pub k_un: String,
    pub k_pw: String,
    pub hp_path: String,
    pub p_addr: String,
    pub p_auth: String,
    pub neutrino: String,
    pub key: String,
}

pub fn read_secret() -> Secret {
    let file = File::open(SECRET).expect("can't open file");
    let reader = BufReader::new(file);
    let mut contents: Secret = serde_json::from_reader(reader).expect("can't read file");

    let k_pw = format!("{:?}", contents.k_pw);
    let mut cleaned_k_pw = k_pw.strip_prefix("\"").expect("parse error");
    cleaned_k_pw = cleaned_k_pw.strip_suffix("\"").expect("parse error");
    contents.k_pw = cleaned_k_pw.to_string();

    contents
}
