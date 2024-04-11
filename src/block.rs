use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_json;
use sha2::{Digest, Sha256};
use std::{fs::File, io::Write, time::SystemTime, vec};

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
        // Set up the header with valid values
        self.header.version = 4; // Ensure version is at least 4.
        self.header.bits = 0x1f00ffff; // Set bits to specified value.
        self.header.time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32; // Set time to current timestamp.
        self.header.merkle_root =
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(); // This should be computed based on transactions.
        self.header.previous_block_hash =
            "0000000000000000000000000000000000000000000000000000000000000000".to_owned();

        let difficulty_bytes =
            hex::decode(difficulty_target).expect("Invalid hex in difficulty target");
        let difficulty_target = BigUint::from_bytes_be(&difficulty_bytes);

        loop {
            let header_bytes =
                serde_json::to_vec(&self.header).expect("Failed to serialize header");
            let hash = Sha256::digest(&Sha256::digest(&header_bytes));
            let hash_int = BigUint::from_bytes_be(&hash);

            if hash_int < difficulty_target {
                println!("Block mined with hash: {:x}", hash);
                break;
            }

            self.header.nonce += 1;
        }
    }

    pub fn generate_output(&self) {
        let mut output = File::create("output.txt").expect("Failed to create output.txt");

        // Write block header
        let header_str = serde_json::to_string(&self.header).expect("Failed to serialize header");
        writeln!(output, "{}", header_str).expect("Failed to write header to file");

        // Serialize and write coinbase transaction
        let coinbase_tx = &self.transactions[0];
        let coinbase_tx_str =
            serde_json::to_string(coinbase_tx).expect("Failed to serialize coinbase transaction");
        writeln!(output, "{}", coinbase_tx_str)
            .expect("Failed to write coinbase transaction to file");

        // Write transaction IDs
        for tx in &self.transactions {
            let tx_str = serde_json::to_string(tx).expect("Failed to serialize transaction");
            let txid = Sha256::digest(&Sha256::digest(tx_str.as_bytes()));
            writeln!(output, "{:x}", txid).expect("Failed to write txid to file");
        }
    }
}
