use serde::{Deserialize, Serialize};
use serde_json;
use sha2::{Digest, Sha256};
use std::{fs::File, io::Write, vec};

use crate::tx::Transaction;

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockHeader {
    pub version: i32,
    pub previous_block_hash: String,
    pub merkle_root: String,
    pub time: u32,
    pub bits: u32,
    pub nonce: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn mine(&mut self, difficulty_target: &str) {
        loop {
            let header = serde_json::to_string(&self.header).unwrap();
            let hash = Sha256::digest(header.as_bytes());
            let hash_hex = format!("{:x}", hash);

            if hash_hex < difficulty_target.to_owned() {
                println!("Block mined: {}", hash_hex);
                break;
            }

            self.header.nonce += 1;
        }
    }

    pub fn generate_output(&self) {
        let mut output = File::create("output.txt").unwrap();

        // Write block header
        writeln!(output, "{:?}", self.header).unwrap();

        // Serialize and write coinbase transaction
        let coinbase_tx = serde_json::to_string(&self.transactions[0]).unwrap();
        writeln!(output, "{}", coinbase_tx).unwrap();

        // Write transaction IDs
        for tx in &self.transactions {
            let tx_json = serde_json::to_string(tx).unwrap();
            let txid = Sha256::digest(tx_json.as_bytes());
            writeln!(output, "{:x}", txid).unwrap();
        }
    }
}
