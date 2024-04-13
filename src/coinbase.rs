use crate::tx::{Input, Output, PrevOut, Transaction};
use ripemd::{Digest, Ripemd160};
use sha2::Sha256;

pub fn create_coinbase_transaction(
    block_reward: u64,
    total_fees: u64,
    miner_address: String,
) -> Transaction {
    // The output value of the coinbase transaction is the sum of block reward and total fees
    let output_value = block_reward + total_fees;

    // Convert the miner's address to its public key hash (PKH)
    let pkh = hash_public_key(&miner_address);

    // Create the P2PKH script using the public key hash
    let script = format!("76a914{}88ac", hex::encode(pkh));

    // Create the witness commitment output
    // let witness_commitment_script = create_witness_commitment_script();

    Transaction {
        version: 1,  // Version of the transaction format
        locktime: 0, // Typically 0 for coinbase transactions
        vin: vec![Input {
            // Coinbase transactions have a single input
            txid: String::from("0000000000000000000000000000000000000000000000000000000000000000"), // All zeros for coinbase tx
            vout: 0xffffffff, // Maximum value as it's not referencing a real output
            prevout: PrevOut {
                // Dummy prevout for coinbase tx
                scriptpubkey: String::new(), // Could be used to include miner-specific data
                scriptpubkey_asm: String::new(),
                scriptpubkey_type: String::from("coinbase"),
                scriptpubkey_address: String::new(),
                value: 0, // No input value
            },
            scriptsig: String::from("1600140f1c83b7ea9e7fefd2b10aac8c680ede85e3d50f"), // Miners can include arbitrary data here
            scriptsig_asm: String::new(),
            witness: Some(vec![String::from("00")]), // Witness reserved value
            is_coinbase: true,
            sequence: 0xffffffff, // Full sequence
        }],
        vout: vec![Output {
            // The output sending the reward to the miner's address
            scriptpubkey: script.clone(),
            scriptpubkey_asm: script,
            scriptpubkey_type: String::from("p2pkh"), // Pay to public key hash
            scriptpubkey_address: Some(miner_address),
            value: output_value,
        }],
    }
}

fn hash_public_key(public_key: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(public_key.as_bytes());
    let sha256_hash = hasher.finalize();

    let mut hasher = Ripemd160::new();
    hasher.update(sha256_hash);
    let pkh = hasher.finalize();

    pkh.to_vec()
}
