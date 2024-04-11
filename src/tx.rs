use std::clone;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub version: i32,
    pub locktime: u32,
    pub vin: Vec<Input>,
    pub vout: Vec<Output>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Input {
    pub txid: String,
    pub vout: u32,
    pub prevout: PrevOut,
    pub scriptsig: String,
    pub scriptsig_asm: String,
    pub witness: Option<Vec<String>>,
    pub is_coinbase: bool,
    pub sequence: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PrevOut {
    pub scriptpubkey: String,
    pub scriptpubkey_asm: String,
    pub scriptpubkey_type: String,
    pub scriptpubkey_address: String,
    pub value: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Output {
    pub scriptpubkey: String,
    pub scriptpubkey_asm: String,
    pub scriptpubkey_type: String,
    pub scriptpubkey_address: Option<String>,
    pub value: u64,
}

impl Transaction {
    pub fn is_basic_valid(&self) -> bool {
        if self.vin.len() == 0 || self.vout.len() == 0 {
            return false;
        }

        let mut in_value = 0;
        for input in &self.vin {
            in_value += input.prevout.value;
        }
        let mut out_value = 0;
        for output in &self.vout {
            out_value += output.value;
        }
        if in_value < out_value {
            return false;
        }

        true
    }

    pub fn fee(&self) -> u64 {
        let mut in_value = 0;
        for input in &self.vin {
            in_value += input.prevout.value;
        }
        let mut out_value = 0;
        for output in &self.vout {
            out_value += output.value;
        }
        in_value - out_value
    }
}
