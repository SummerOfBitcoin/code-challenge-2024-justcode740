use crate::tx::{Output, Transaction};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_json;
use sha2::{Digest, Sha256};
use std::{fs::File, io::Write, time::SystemTime, vec};

const WITNESS_RESERVED_VALUE: [u8; 32] = [0; 32];

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

        if let Some(first) = level.get(0) {
            // Now `first` is a reference to the first element (&Vec<u8>), not the owned Vec<u8>.
            Some(hex::encode(first))
        } else {
            None
        }
    }

    pub fn generate_output(&self) {
        let mut output = File::create("output.txt").expect("Failed to create output.txt");

        // Convert binary header to hexadecimal string
        let header_hex = hex::encode(self.serialize_header());

        // Write block header in hex format
        writeln!(output, "{}", header_hex).expect("Failed to write header to file");

        // Check if we have at least one transaction and the first is a coinbase transaction
        if let Some(coinbase_tx) = self.transactions.get(0) {
            let witness_commitment = self
                .calculate_witness_commitment()
                .expect("Failed to calculate witness commitment");
            let updated_coinbase_tx =
                self.create_coinbase_transaction(coinbase_tx, &witness_commitment);
            let coinbase_tx_hex =
                hex::encode(Block::serialize_transaction(&updated_coinbase_tx, true));
            writeln!(output, "{}", coinbase_tx_hex)
                .expect("Failed to write coinbase transaction to file");
        }

        for i  in 0..self.transactions.len() {
            // if tx.calculate_wtxid().unwrap() == "35f1e96e0c00a213134b533d93a6b3cf074c24178b640c1fbdecfe0724455e66" {
            //     println!("{:?}", tx);
            //     println!("{:?}", tx.calculate_txid());
            // }
            println!("{} {}", self.txids[i], self.transactions[i].weight());
        }

        // Write transaction IDs
        for txid in &self.txids {
            writeln!(output, "{}", txid).expect("Failed to write txid to file");
        }
    }

    // 7533d87ec9e2f0eda1298c2e2e37141c275358c4884fd90fbb0f87d67e5f0ce0

    // Helper to create an updated coinbase transaction with witness commitment
    fn create_coinbase_transaction(
        &self,
        existing_coinbase: &Transaction,
        witness_commitment: &str,
    ) -> Transaction {
        let mut new_coinbase = existing_coinbase.clone();
        // Modify the coinbase transaction to include the witness commitment
        new_coinbase.vout.push(Output {
            scriptpubkey: format!("6a24aa21a9ed{}", witness_commitment),
            scriptpubkey_asm: format!("OP_RETURN {}", witness_commitment),
            scriptpubkey_type: String::from("nulldata"),
            scriptpubkey_address: None,
            value: 0,
        });
        new_coinbase
    }

    fn serialize_transaction(tx: &Transaction, is_coinbase: bool) -> Vec<u8> {
        let mut data = Vec::new();

        // Transaction version
        data.extend(&tx.version.to_le_bytes());

        // Marker and flag for SegWit transactions
        if is_coinbase || tx.vin.iter().any(|input| input.witness.is_some()) {
            data.push(0x00); // Marker
            data.push(0x01); // Flag
        }

        // Input count
        data.extend(Self::serialize_varint(tx.vin.len() as u64));

        // Inputs
        for (i, input) in tx.vin.iter().enumerate() {
            // Previous TXID (32 bytes, reversed for regular, not altered for coinbase)
            let prev_txid = hex::decode(&input.txid).expect("Invalid previous TXID");
            if i == 0 && is_coinbase {
                data.extend(&prev_txid);
            } else {
                data.extend(prev_txid.iter().rev());
            }

            // Previous output index (4 bytes)
            data.extend(&input.vout.to_le_bytes());

            // ScriptSig (varint followed by the actual script)
            let scriptsig_bytes = input.scriptsig.as_bytes();
            data.extend(Self::serialize_varint(scriptsig_bytes.len() as u64));
            data.extend(scriptsig_bytes);

            // Sequence (4 bytes)
            data.extend(&input.sequence.to_le_bytes());
        }

        // Outputs
        let output_count = tx.vout.len() as u64;
        data.extend(Self::serialize_varint(output_count));
        for output in &tx.vout {
            // Value in satoshis (8 bytes)
            data.extend(&output.value.to_le_bytes());

            // ScriptPubKey
            let scriptpubkey_bytes = hex::decode(&output.scriptpubkey).expect("Invalid script");
            data.extend(Self::serialize_varint(scriptpubkey_bytes.len() as u64));
            data.extend(scriptpubkey_bytes);
        }

        // Witnesses (only if the flag is set)
        if is_coinbase || tx.vin.iter().any(|input| input.witness.is_some()) {
            for (i, input) in tx.vin.iter().enumerate() {
                if i == 0 && is_coinbase {
                    // Handle coinbase transaction's witness reserved value
                    data.extend(Self::serialize_varint(1)); // One element in witness stack
                    data.extend(Self::serialize_varint(WITNESS_RESERVED_VALUE.len() as u64));
                    data.extend(&WITNESS_RESERVED_VALUE);
                } else if let Some(witness) = &input.witness {
                    // Witness stack size
                    data.extend(Self::serialize_varint(witness.len() as u64));
                    // Witness data
                    for item in witness {
                        let witness_data = hex::decode(item).expect("Invalid witness data");
                        data.extend(Self::serialize_varint(witness_data.len() as u64));
                        data.extend(&witness_data);
                    }
                } else {
                    // Normal transaction without witness data
                    data.extend(Self::serialize_varint(0)); // No witness data
                }
            }
        }

        // Locktime (4 bytes)
        data.extend(&tx.locktime.to_le_bytes());

        data
    }

    fn serialize_varint(value: u64) -> Vec<u8> {
        match value {
            0..=0xFC => vec![value as u8],
            0xFD..=0xFFFF => {
                let mut data = vec![0xFD];
                data.extend(&(value as u16).to_le_bytes());
                data
            }
            0x10000..=0xFFFFFFFF => {
                let mut data = vec![0xFE];
                data.extend(&(value as u32).to_le_bytes());
                data
            }
            _ => {
                let mut data = vec![0xFF];
                data.extend(&value.to_le_bytes());
                data
            }
        }
    }
    fn compute_txids(&mut self) {
        if self.txids.is_empty() {
            self.txids = self
                .transactions
                .iter()
                .map(|tx| match tx.calculate_txid() {
                    Ok(txid) => txid,
                    Err(e) => {
                        eprintln!("Error calculating TXID: {}", e);
                        String::new() // Or handle the error in a different way
                    }
                })
                .collect();
        }
    }

    pub fn calculate_witness_commitment(&self) -> Result<String, String> {
        let wtxids = self.compute_wtxids()?;
        let witness_root = self.generate_merkle_root_wtxids(&wtxids)?;
        let witness_reserved_value = hex::encode(WITNESS_RESERVED_VALUE);
        let commitment_input = witness_root + &witness_reserved_value;
        let commitment = Self::hash256(&commitment_input);
        Ok(commitment)
    }

    fn hash256(input: &str) -> String {
        let decoded_input = hex::decode(input).expect("Failed to decode input as hex");
        let h1 = Sha256::digest(&decoded_input);
        let h2 = Sha256::digest(&h1);
        hex::encode(h2)
    }

    // Helper function to generate the Merkle root from wtxids
    fn generate_merkle_root_wtxids(&self, wtxids: &[String]) -> Result<String, String> {
        let mut level = wtxids
            .iter()
            .map(|id| {
                let mut buf = hex::decode(id).unwrap();
                buf.reverse();
                buf
            })
            .collect::<Vec<_>>();

        while level.len() > 1 {
            let mut next_level = Vec::new();
            for chunk in level.chunks(2) {
                let left = &chunk[0];
                let right = chunk.get(1).unwrap_or(left);
                let combined = [left.as_slice(), right.as_slice()].concat();
                let hash = double_sha256(&combined);
                next_level.push(hash);
            }
            level = next_level;
        }
        level
            .first()
            .map(|hash| hex::encode(hash))
            .ok_or_else(|| "Failed to generate Merkle root.".to_string())
    }

    pub fn compute_wtxids(&self) -> Result<Vec<String>, String> {
        let mut wtxids = Vec::new();
        for (i, tx) in self.transactions.iter().enumerate() {
            if i == 0 && tx.is_coinbase() {
                // For coinbase transactions, use a special wtxid of all zeros
                wtxids.push(
                    "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
                );
            } else {
                // Check if the transaction has witness data
                let has_witness = tx.vin.iter().any(|input| input.witness.is_some());
                if has_witness {
                    // For transactions with witness data, compute the wtxid
                    let wtxid = tx.calculate_wtxid()?;
                    wtxids.push(wtxid);
                } else {
                    println!("?? {:?}", self.txids[i].clone());
                    // For transactions without witness data, use the txid
                    wtxids.push(self.txids[i].clone());
                }
            }
        }
        Ok(wtxids)
    }
}

pub fn double_sha256(data: &[u8]) -> Vec<u8> {
    let hash1 = Sha256::digest(data);
    let hash2 = Sha256::digest(&hash1);
    hash2.to_vec()
}
