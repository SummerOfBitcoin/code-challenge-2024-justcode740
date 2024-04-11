// use crate::tx::Transaction;
// use secp256k1::{Message, PublicKey, Secp256k1, Signature};
// use sha2::{Digest, Sha256};

// pub fn validate_transaction(tx: &Transaction) -> Result<(), String> {
//     let secp = Secp256k1::verification_only();

//     for (i, input) in tx.vin.iter().enumerate() {
//         if input.is_coinbase {
//             continue;
//         }

//         // Validate the script type and extract the public key
//         let prev_output = &input.prevout;
//         let public_key = match prev_output.scriptpubkey_type.as_str() {
//             "v1_p2tr" => {
//                 let pubkey_hex = &prev_output.scriptpubkey_asm[29..];
//                 PublicKey::from_slice(&hex::decode(pubkey_hex).map_err(|_| "Invalid public key")?)
//                     .map_err(|_| "Invalid public key")?
//             }
//             _ => return Err("Unsupported script type".to_string()),
//         };

//         // Verify the signature
//         let signature = Signature::from_der(&hex::decode(&input.witness.as_ref().unwrap()[0]).map_err(|_| "Invalid signature")?)
//             .map_err(|_| "Invalid signature")?;

//         // Assuming the sighash type is `SIGHASH_ALL`
//         let sighash_all: u32 = 1;

//         let mut sig_hash = Sha256::new();
//         sig_hash.update(&tx.version.to_le_bytes());

//         for (j, input) in tx.vin.iter().enumerate() {
//             if i == j {
//                 sig_hash.update(&hex::decode(&input.prevout.scriptpubkey).map_err(|_| "Invalid scriptpubkey")?);
//             } else {
//                 sig_hash.update(&[0; 32]);
//                 sig_hash.update(&[0; 4]);
//             }
//             sig_hash.update(&input.sequence.to_le_bytes());
//         }

//         for output in &tx.vout {
//             sig_hash.update(&output.value.to_le_bytes());
//             sig_hash.update(&hex::decode(&output.scriptpubkey).map_err(|_| "Invalid scriptpubkey")?);
//         }

//         sig_hash.update(&tx.locktime.to_le_bytes());
//         sig_hash.update(&sighash_all.to_le_bytes());

//         let message = Message::from_slice(&sig_hash.finalize()).map_err(|_| "Invalid message")?;
//         secp.verify(&message, &signature, &public_key)
//             .map_err(|_| "Signature verification failed")?;
//     }

//     let total_output_value: u64 = tx.vout.iter().map(|output| output.value).sum();

//     let mut total_input_value = 0;
//     for input in &tx.vin {
//         if !input.is_coinbase {
//             total_input_value += input.prevout.value;
//         }
//     }

//     if total_output_value > total_input_value {
//         return Err("Total output value exceeds total input value".to_string());
//     }

//     Ok(())
// }
