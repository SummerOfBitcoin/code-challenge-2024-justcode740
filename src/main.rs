use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

mod block;
mod coinbase;
mod tx;
mod validate;
use tx::Transaction;

use crate::block::Block;
use crate::block::BlockHeader;
use crate::coinbase::create_coinbase_transaction;

// use validate::validate_transaction;

fn read_transactions_from_dir(dir: &Path) -> io::Result<(Vec<Transaction>, usize, usize)> {
    let mut entries = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|path| path.is_file()) // Ensure it's a file
        .collect::<Vec<PathBuf>>();

    // Sort the entries by their path names
    entries.sort();

    let mut transactions = Vec::new();
    let mut total_files = 0;
    let mut failed_parses = 0;

    for path in entries {
        total_files += 1;
        println!("{:?}", path);
        match fs::read_to_string(&path) {
            Ok(data) => match serde_json::from_str::<Transaction>(&data) {
                Ok(transaction) => transactions.push(transaction),
                Err(_) => failed_parses += 1,
            },
            Err(_) => {}
        }
    }

    Ok((transactions, total_files, failed_parses))
}

fn get_tx() -> Vec<Transaction> {
    let dir = Path::new("./mempool");
    let txs = match read_transactions_from_dir(dir) {
        Ok((transactions, total_files, failed_parses)) => {
            println!("Successfully parsed transactions: {}", transactions.len());
            println!("Total files: {}", total_files);
            println!("Failed parses: {}", failed_parses);
            transactions
        }
        Err(e) => panic!("Error reading transactions: {}", e),
    };

    let mut invalid_transactions = 0;
    let mut fail = 0;
    let mut valid_txs = vec![];
    for tx in txs {
        // if let Err(_) = validate_transaction(tx) {
        //     invalid_transactions += 1;
        // }
        if tx.is_basic_valid() {
            valid_txs.push(tx);
        }
    }
    valid_txs
}

fn select_tx_for_block(txs: Vec<Transaction>) -> Vec<Transaction> {
    const MAX_BLOCK_WEIGHT: usize = 4_000_000 - 1000; // Standard weight units of a block

    let mut selected_txs: Vec<Transaction> = Vec::new();
    let mut total_weight = 0;

    // Sort transactions by their fee rate (fee per weight unit) in descending order
    let mut txs_sorted = txs;
    txs_sorted.sort_by(|a, b| {
        let fee_rate_a = a.fee() as f64 / a.weight() as f64;
        let fee_rate_b = b.fee() as f64 / b.weight() as f64;
        fee_rate_b.partial_cmp(&fee_rate_a).unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut c = 0;

    // Select transactions to maximize fee and fit within block weight
    for tx in txs_sorted {
        let tx_weight = tx.weight();
        if total_weight + tx_weight <= MAX_BLOCK_WEIGHT {
            selected_txs.push(tx);
            c+=1;
            total_weight += tx_weight;
            if c > 2000 {
                break;
            }
        } else {
            // If adding this transaction exceeds the block weight limit, stop adding.
            break;
        }
    }

    println!("Total transactions selected: {}, Total weight: {}", selected_txs.len(), total_weight);
    selected_txs
}

fn main() {
    let txs = get_tx();

    let mut valid = select_tx_for_block(txs);

    let total_fees = valid.iter().fold(0, |acc, x| acc + x.fee());

    let br = 6_250_000_000;
    let cb_tx = create_coinbase_transaction(br, total_fees, "".to_owned());
    println!("cb {}", cb_tx.weight());
    let mut valid_tx = vec![cb_tx];
    valid_tx.append(&mut valid);
    
    // println!("mai{:?}", valid_tx[1].calculate_txid());
    // println!("mai{:?}", valid_tx[1].calculate_wtxid());

    // println!("mai{:?}",     valid_tx[1].vin[0].txid);// 000cb561188c762c81f76976f816829424e2af9e0e491c617b7bf41038df3d35

    // 69074bd90317c507b367c40dee722821c8954eeb84c9e24e29050b0c85d1d422
    // 7533d87ec9e2f0eda1298c2e2e37141c275358c4884fd90fbb0f87d67e5f0ce0
    // 7533d87ec9e2f0eda1298c2e2e37141c275358c4884fd90fbb0f87d67e5f0ce0
    // println!("{:?}", valid_tx[1])

    let difficulty_target = "0000ffff00000000000000000000000000000000000000000000000000000000";

    let mut block = Block {
        header: BlockHeader {
            version: 1,
            previous_block_hash: "".to_string(),
            merkle_root: "".to_string(),
            time: 0,
            bits: 0,
            nonce: 0,
        },
        transactions: valid_tx,
        txids: vec![],
    };

    block.mine(difficulty_target);
    block.generate_output();

    // println!("Invalid transactions: {}", invalid_transactions);
    // println!("Different script types found:");
    // for script_type in script_types {
    //     println!("- {}", script_type);
    // }
}
