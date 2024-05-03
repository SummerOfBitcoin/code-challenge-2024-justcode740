extern crate hex;
extern crate ripemd160;
extern crate secp256k1;
extern crate sha2;

use hex::FromHex;
use ripemd160::Ripemd160;
use secp256k1::{Message, PublicKey, Secp256k1, Signature};
use sha2::{Digest, Sha256};
use std::error::Error;

use crate::tx::Transaction;

// Assuming the rest of your provided code is here

impl Transaction {
    /// Validates all inputs in the transaction based on P2PKH rules.
    pub fn validate_p2pkh_inputs(&self) -> Result<bool, Box<dyn Error>> {
        let secp = Secp256k1::new();

        // Process each input
        for input in &self.vin {
            if !input.is_coinbase {
                let prevout_scriptpubkey = &input.prevout.scriptpubkey;
                // Extract public key hash from scriptPubKey
                // Assumes scriptPubKey is formatted as "OP_DUP OP_HASH160 <pubkeyhash> OP_EQUALVERIFY OP_CHECKSIG"
                let script_parts: Vec<&str> = prevout_scriptpubkey.split_whitespace().collect();
                if script_parts.len() < 5
                    || script_parts[0] != "OP_DUP"
                    || script_parts[1] != "OP_HASH160"
                {
                    return Err(Box::new(std::fmt::Error)); // Invalid scriptPubKey format
                }
                let expected_pubkey_hash = script_parts[2];

                // Extract signature and public key from scriptSig
                // Assumes scriptSig is formatted as "<signature> <pubkey>"
                let script_sig_parts: Vec<&str> = input.scriptsig.split_whitespace().collect();
                if script_sig_parts.len() != 2 {
                    return Err(Box::new(std::fmt::Error)); // Invalid scriptSig format
                }
                let signature_hex = script_sig_parts[0];
                let pubkey_hex = script_sig_parts[1];

                let pubkey_bytes = hex::decode(pubkey_hex)?;
                let signature_bytes = hex::decode(signature_hex)?;

                // Verify public key hash
                let pubkey_hash = Self::hash160(&pubkey_bytes);
                if hex::encode(pubkey_hash) != expected_pubkey_hash {
                    return Ok(false); // Public key hash does not match
                }

                // Verify signature
                let pubkey = PublicKey::from_slice(&pubkey_bytes)?;
                let signature = Signature::from_der(&signature_bytes[..signature_bytes.len() - 1])?; // remove sighash type byte
                let message =
                    Message::from_hashed_data::<Sha256>(&double_sha256(&self.encode_to_vec()?));

                if !secp.verify(&message, &signature, &pubkey).is_ok() {
                    return Ok(false); // Signature does not verify
                }
            }
        }
        Ok(true)
    }

    /// Helper function to hash data using SHA256 followed by RIPEMD-160.
    fn hash160(input: &[u8]) -> Vec<u8> {
        let sha256_result = Sha256::digest(input);
        let ripemd_result = Ripemd160::digest(&sha256_result);
        ripemd_result.to_vec()
    }

    /// Helper function to encode transaction to Vec<u8> for hashing.
    fn encode_to_vec(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        // Simple serialization logic to prepare the transaction data for signing
        // In practice, this should handle all transaction fields properly
        let mut encoded = vec![];
        encoded.extend(&self.version.to_le_bytes());
        encoded.extend(&self.locktime.to_le_bytes());
        // Encoding other parts is necessary for real applications
        Ok(encoded)
    }
}

/// Helper function to compute double SHA256.
fn double_sha256(data: &[u8]) -> Vec<u8> {
    let first_hash = Sha256::digest(data);
    let second_hash = Sha256::digest(&first_hash);
    second_hash.to_vec()
}
