use serde::{Deserialize, Serialize};

use crate::tx::Transaction;

// Assuming the structs Transaction, Input, Output, PrevOut are defined as above

pub fn validate_transaction(tx: &Transaction) -> Result<(), String> {
    if tx.version < 1 || tx.version > 2 {
        return Err("Unsupported transaction version".to_string());
    }

    if tx.vin.is_empty() {
        return Err("Transaction has no inputs".to_string());
    }

    if tx.vout.is_empty() {
        return Err("Transaction has no outputs".to_string());
    }

    for input in &tx.vin {
        if input.is_coinbase && tx.vin.len() > 1 {
            return Err("Coinbase transaction has more than one input".to_string());
        }

        if !input.is_coinbase && input.txid.is_empty() {
            return Err("Input txid is empty".to_string());
        }

        if let Some(witness) = &input.witness {
            if witness.is_empty() {
                return Err("Witness is present but empty".to_string());
            }
        }
    }

    // let total_output_value: u64 = tx.vout.iter().map(|output| output.value).sum();
    // if total_output_value == 0 {
    //     return Err("Total output value is 0".to_string());
    // }

    // Further checks can include scriptsig and scriptpubkey validation, which are complex and require executing the scripts

    Ok(())
}
