use std::fs;
use std::io;
use std::path::Path;

mod tx;
mod validate;
use tx::Transaction;
use validate::validate_transaction;

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
                Ok(data) => {
                    match serde_json::from_str::<Transaction>(&data) {
                        Ok(transaction) => transactions.push(transaction),
                        Err(e) => { // Capture the parse error
                            failed_parses += 1;
                            println!("Failed to parse file: {:?}, Reason: {}", path.display(), e);
                            // Prints out the specific reason for the parse failure
                        },
                    }
                },
                Err(e) => { // Capture the file read error
                    println!("Failed to read file: {:?}, Reason: {}", path.display(), e);
                    // This case handles file read errors
                }
            }
        }
    }
    for tx in &transactions {
        // println!("{:?}", tx.locktime);
        if tx.locktime != 0 {
            println!("wow {:?}", tx.locktime);
        }
    }

    println!("Total files processed: {}", total_files);
    println!("Failed to parse: {}", failed_parses);
    Ok((transactions, total_files, failed_parses))
}

fn main() {
    let dir = Path::new("../mempool");
    let txs = match read_transactions_from_dir(dir) {
        Ok((transactions, total_files, failed_parses)) => {
            println!("Successfully parsed transactions: {}", transactions.len());
            println!("Total files: {}", total_files);
            println!("Failed parses: {}", failed_parses);
            transactions
        },
        Err(e) => panic!("Error reading transactions: {}", e),
    };
    let mut res = 0;
    for tx in &txs {
        if let Err(e) = validate_transaction(tx) {
            println!("{:?}", tx.);
            res += 1;
            println!("error: {:?}", e);

        }
        
    }
    println!("{:?}", res);
    
}
