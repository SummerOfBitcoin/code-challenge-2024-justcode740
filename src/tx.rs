use std::io::Write;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::block::double_sha256;

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

    // Calculate the double SHA256 hash of the transaction
    pub fn calculate_txid(&self) -> Result<String, String> {
        let mut data = Vec::new();

        // Version
        data.write_all(&self.version.to_le_bytes()).unwrap();

        // Input count
        let input_count = self.vin.len() as u64;
        data.write_all(&serialize_varint(input_count)).unwrap();

        // Inputs
        for input in &self.vin {
            // Previous TXID (little-endian)
            let prev_txid = hex::decode(&input.txid)
                .map_err(|_| format!("Invalid previous TXID: {}", input.txid))?;
            data.write_all(&prev_txid.iter().rev().copied().collect::<Vec<u8>>())
                .unwrap();

            // Previous output index (little-endian)
            data.write_all(&input.vout.to_le_bytes()).unwrap();

            // Script length and script
            if input.scriptsig.is_empty() {
                data.write_all(&[0x00]).unwrap(); // Empty script
            } else {
                let script = hex::decode(&input.scriptsig)
                    .map_err(|_| format!("Invalid script: {}", input.scriptsig))?;
                data.write_all(&serialize_varint(script.len() as u64))
                    .unwrap();
                data.write_all(&script).unwrap();
            }

            // Sequence (little-endian)
            data.write_all(&input.sequence.to_le_bytes()).unwrap();
        }

        // Output count
        let output_count = self.vout.len() as u64;
        data.write_all(&serialize_varint(output_count)).unwrap();

        // Outputs
        for output in &self.vout {
            // Value (little-endian)
            data.write_all(&output.value.to_le_bytes()).unwrap();

            // Script length and script
            let script = hex::decode(&output.scriptpubkey)
                .map_err(|_| format!("Invalid script: {}", output.scriptpubkey))?;
            data.write_all(&serialize_varint(script.len() as u64))
                .unwrap();
            data.write_all(&script).unwrap();
        }

        // Locktime (little-endian)
        data.write_all(&self.locktime.to_le_bytes()).unwrap();

        // Double SHA256 hash and reverse
        let txid = double_sha256(&data);
        Ok(hex::encode(txid.iter().rev().copied().collect::<Vec<u8>>()))
    }

    pub fn calculate_wtxid(&self) -> Result<String, String> {
        if self.is_coinbase() {
            return Ok(
                "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            );
        }

        let mut data = Vec::new();

        // Transaction version
        data.extend_from_slice(&self.version.to_le_bytes());

        // Marker
        data.push(0x00);

        // Flag
        data.push(0x01);

        // Inputs
        data.extend_from_slice(&serialize_varint(self.vin.len() as u64));
        for input in &self.vin {
            // Previous TXID (reversed)
            let prev_txid = hex::decode(&input.txid).map_err(|_| "Invalid TXID".to_string())?;
            data.extend(prev_txid.iter().rev());

            // Output index
            data.extend_from_slice(&input.vout.to_le_bytes());

            // ScriptSig
            let script = hex::decode(&input.scriptsig).map_err(|_| "Invalid script".to_string())?;
            data.extend_from_slice(&serialize_varint(script.len() as u64));
            data.extend(script);

            // Sequence
            data.extend_from_slice(&input.sequence.to_le_bytes());
        }

        // Outputs
        data.extend_from_slice(&serialize_varint(self.vout.len() as u64));
        for output in &self.vout {
            // Value
            data.extend_from_slice(&output.value.to_le_bytes());

            // ScriptPubKey
            let script =
                hex::decode(&output.scriptpubkey).map_err(|_| "Invalid script".to_string())?;
            data.extend_from_slice(&serialize_varint(script.len() as u64));
            data.extend(script);
        }

        // Witness data
        for input in &self.vin {
            if let Some(witness) = &input.witness {
                data.extend_from_slice(&serialize_varint(witness.len() as u64));
                for item in witness {
                    let witness_data =
                        hex::decode(item).map_err(|_| "Invalid witness data".to_string())?;
                    data.extend_from_slice(&serialize_varint(witness_data.len() as u64));
                    data.extend(witness_data);
                }
            } else {
                data.push(0x00);
            }
        }

        // Locktime
        data.extend_from_slice(&self.locktime.to_le_bytes());

        // Double SHA-256 to get the wtxid
        let hash = Sha256::digest(&Sha256::digest(&data));
        Ok(hex::encode(hash.iter().rev().copied().collect::<Vec<u8>>()))
    }

    // Helper to determine if the transaction is a coinbase transaction
    // fn is_coinbase(&self) -> bool {
    //     self.vin.len() == 1
    //         && self.vin[0].txid == "0000000000000000000000000000000000000000000000000000000000000000"
    //         && self.vin[0].vout == 0xffffffff
    // }
    // Helper to determine if the transaction is a coinbase transaction
    pub fn is_coinbase(&self) -> bool {
        self.vin.len() == 1
            && self.vin[0].txid
                == "0000000000000000000000000000000000000000000000000000000000000000"
            && self.vin[0].vout == 0xffffffff
    }

    // Calculate the transaction weight
    pub fn weight(&self) -> usize {
        let base_size = self.base_size();
        let total_size = self.total_size();
        (base_size * 3) + total_size
    }

    // Calculate the base size of the transaction (size without witness data)
    fn base_size(&self) -> usize {
        let mut data = Vec::new();

        // Transaction version
        data.extend_from_slice(&self.version.to_le_bytes());

        // Inputs
        data.extend_from_slice(&serialize_varint(self.vin.len() as u64));
        for input in &self.vin {
            let prev_txid = hex::decode(&input.txid).unwrap();
            data.extend(prev_txid.iter().rev());
            data.extend_from_slice(&input.vout.to_le_bytes());
            let script = hex::decode(&input.scriptsig).unwrap();
            data.extend_from_slice(&serialize_varint(script.len() as u64));
            data.extend(script);
            data.extend_from_slice(&input.sequence.to_le_bytes());
        }

        // Outputs
        data.extend_from_slice(&serialize_varint(self.vout.len() as u64));
        for output in &self.vout {
            data.extend_from_slice(&output.value.to_le_bytes());
            let script = hex::decode(&output.scriptpubkey).unwrap();
            data.extend_from_slice(&serialize_varint(script.len() as u64));
            data.extend(script);
        }

        // Locktime
        data.extend_from_slice(&self.locktime.to_le_bytes());

        data.len()
    }

    // Calculate the total size of the transaction (size with witness data)
    fn total_size(&self) -> usize {
        let mut data = Vec::new();

        // Transaction version
        data.extend_from_slice(&self.version.to_le_bytes());

        // Marker and Flag
        if self.has_witness() {
            data.push(0x00); // Marker
            data.push(0x01); // Flag
        }

        // Inputs
        data.extend_from_slice(&serialize_varint(self.vin.len() as u64));
        for input in &self.vin {
            let prev_txid = hex::decode(&input.txid).unwrap();
            data.extend(prev_txid.iter().rev());
            data.extend_from_slice(&input.vout.to_le_bytes());
            let script = hex::decode(&input.scriptsig).unwrap();
            data.extend_from_slice(&serialize_varint(script.len() as u64));
            data.extend(script);
            data.extend_from_slice(&input.sequence.to_le_bytes());
        }

        // Outputs
        data.extend_from_slice(&serialize_varint(self.vout.len() as u64));
        for output in &self.vout {
            data.extend_from_slice(&output.value.to_le_bytes());
            let script = hex::decode(&output.scriptpubkey).unwrap();
            data.extend_from_slice(&serialize_varint(script.len() as u64));
            data.extend(script);
        }

        // Witness data
        if self.has_witness() {
            for input in &self.vin {
                if let Some(witness) = &input.witness {
                    data.extend_from_slice(&serialize_varint(witness.len() as u64));
                    for item in witness {
                        let witness_data = hex::decode(item).unwrap();
                        data.extend_from_slice(&serialize_varint(witness_data.len() as u64));
                        data.extend(witness_data);
                    }
                } else {
                    data.push(0x00); // No witness data
                }
            }
        }

        // Locktime
        data.extend_from_slice(&self.locktime.to_le_bytes());

        data.len()
    }
    // Check if the transaction has witness data
    fn has_witness(&self) -> bool {
        self.vin.iter().any(|input| input.witness.is_some())
    }
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

// "90b22ecd1aec05105687e856b863ff3fbf8720d8436c013ec0db3dcc478794b4"

// 000cb561188c762c81f76976f816829424e2af9e0e491c617b7bf41038df3d35.json
