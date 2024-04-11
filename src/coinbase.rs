use crate::tx::{Input, Output, PrevOut, Transaction};

// Assume these are the calculated values
pub fn create_coinbase_transaction(block_reward: u64, total_fees: u64) -> Transaction {
    // The output value of the coinbase transaction is the sum of block reward and total fees
    let output_value = block_reward + total_fees;

    Transaction {
        version: 1,  // Version of the transaction format
        locktime: 0, // Typically 0 for coinbase transactions
        vin: vec![Input {
            // Coinbase transactions have a single input
            txid: String::new(), // No input transaction (empty string or all zeros)
            vout: 0xffffffff,    // Maximum value as it's not referencing a real output
            prevout: PrevOut {
                // Dummy prevout for coinbase tx
                scriptpubkey: String::new(), // Could be used to include miner-specific data
                scriptpubkey_asm: String::new(),
                scriptpubkey_type: String::from("coinbase"),
                scriptpubkey_address: String::new(),
                value: 0, // No input value
            },
            scriptsig: String::from("arbitrary data"), // Miners can include arbitrary data here
            scriptsig_asm: String::new(),
            witness: None,
            is_coinbase: true,
            sequence: 0xffffffff, // Full sequence
        }],
        vout: vec![Output {
            // The output sending the reward to the miner's address
            scriptpubkey: String::from("miner's address or script"),
            scriptpubkey_asm: String::new(),
            scriptpubkey_type: String::from("pay to script hash or pay to public key hash"),
            scriptpubkey_address: Some(String::from("miner's address")),
            value: output_value,
        }],
    }
}
