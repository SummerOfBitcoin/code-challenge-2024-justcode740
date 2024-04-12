use crate::tx::Transaction;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_json;
use sha2::{Digest, Sha256};
use std::{fs::File, io::Write, time::SystemTime, vec};

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockHeader {
    pub version: u32,
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
    pub txids: Vec<String>,
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
        self.header.merkle_root = self.generate_merkle_root().unwrap(); // Compute merkle root based on transactions.
        self.header.previous_block_hash =
            "0000000000000000000000000000000000000000000000000000000000000000".to_owned();

        let difficulty_bytes =
            hex::decode(difficulty_target).expect("Invalid hex in difficulty target");

        loop {
            let header_bin = self.serialize_header();
            let hash = double_sha256(&header_bin);
            let reversed_hash = hash.iter().rev().copied().collect::<Vec<u8>>();

            if reversed_hash < difficulty_bytes {
                // println!("Block mined with hash: {:x}", hex::encode(reversed_hash));
                break;
            }
            self.header.nonce += 1;
        }
    }

    fn serialize_header(&self) -> Vec<u8> {
        let mut header_bin = vec![];
        header_bin.extend(&self.header.version.to_le_bytes()); // Little endian for version
        header_bin.extend(&hex::decode(&self.header.previous_block_hash).unwrap()); // Hex-decoded previous block hash
        header_bin.extend(&hex::decode(&self.header.merkle_root).unwrap()); // Hex-decoded merkle root
        header_bin.extend(&self.header.time.to_le_bytes()); // Little endian for time
        header_bin.extend(&self.header.bits.to_le_bytes()); // Little endian for bits
        header_bin.extend(&self.header.nonce.to_le_bytes()); // Little endian for nonce
        header_bin
    }

    /// Generates the Merkle root from the block's transactions.
    fn generate_merkle_root(&mut self) -> Option<String> {
        self.compute_txids();

        if self.txids.is_empty() {
            return None;
        }

        let mut level = self
            .txids
            .iter()
            .map(|txid| {
                hex::decode(txid)
                    .unwrap()
                    .into_iter()
                    .rev()
                    .collect::<Vec<u8>>()
            })
            .collect::<Vec<_>>();

        while level.len() > 1 {
            let mut next_level = Vec::new();

            for chunk in level.chunks(2) {
                let pair_hash = match chunk {
                    [left] => double_sha256(&[left.clone(), left.clone()].concat()),
                    [left, right] => double_sha256(&[left.clone(), right.clone()].concat()),
                    _ => unreachable!(),
                };
                next_level.push(pair_hash);
            }

            level = next_level;
        }

        Some(hex::encode(
            level[0].iter().rev().copied().collect::<Vec<u8>>(),
        ))
    }

    pub fn generate_output(&self) {
        let mut output = File::create("output.txt").expect("Failed to create output.txt");

        // Convert binary header to hexadecimal string
        let header_hex = hex::encode(self.serialize_header());

        // Write block header in hex format
        writeln!(output, "{}", header_hex).expect("Failed to write header to file");

        // Serialize and write coinbase transaction
        let coinbase_tx = &self.transactions[0];
        let coinbase_tx_str =
            serde_json::to_string(coinbase_tx).expect("Failed to serialize coinbase transaction");
        writeln!(output, "{}", coinbase_tx_str)
            .expect("Failed to write coinbase transaction to file");

        // Write transaction IDs
        for txid in &self.txids {
            writeln!(output, "{}", txid).expect("Failed to write txid to file");
        }
    }

    fn compute_txids(&mut self) {
        if self.txids.is_empty() {
            self.txids = self
                .transactions
                .iter()
                .map(|tx| {
                    let tx_json = serde_json::to_string(tx).expect("Serialization error");
                    hex::encode(double_sha256(tx_json.as_bytes()))
                })
                .collect();
        }
    }
}

fn double_sha256(data: &[u8]) -> Vec<u8> {
    let hash1 = Sha256::digest(data);
    let hash2 = Sha256::digest(&hash1);
    hash2.to_vec()
}
