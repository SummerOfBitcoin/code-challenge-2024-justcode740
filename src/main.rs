use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::Path;

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
    let mut transactions = Vec::new();
    let mut total_files = 0;
    let mut failed_parses = 0;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            total_files += 1;
            match fs::read_to_string(&path) {
                Ok(data) => match serde_json::from_str::<Transaction>(&data) {
                    Ok(transaction) => transactions.push(transaction),
                    Err(_) => {
                        failed_parses += 1;
                    }
                },
                Err(_) => {}
            }
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
    // let mut res = vec!{};
    // for i in 0..100 {
    //     res.push(txs[i]);
    // }
    // res
    txs[0..100].to_vec()
}

fn main() {
    let txs = get_tx();

    let mut valid = select_tx_for_block(txs);
    let total_fees = valid.iter().fold(0, |acc, x| acc + x.fee());

    let br = 6_250_000_000;
    let cb_tx = create_coinbase_transaction(br, total_fees);
    let mut valid_tx = vec![cb_tx];
    valid_tx.append(&mut valid);

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
