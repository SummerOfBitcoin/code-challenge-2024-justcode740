# Project Documentation

## Design Approach

The project is structured to first validate and filter out invalid transactions. After ensuring the transactions are valid, the process computes fees and sizes. We use a greedy algorithm based on the highest unit fee to optimize the selection of transactions to be included in the next block to be mined.

## Implementation Details

### File Reader

- **Location**: `main.rs`
- **Purpose**: Initializes the reading process for incoming transaction data and directs it to the validation logic.

### Transaction Validation

- **Location**: Defined within the `Transaction` struct in `transaction.rs`
- **Functionality**: Validates transactions based on predefined rules and filters out those that are invalid.

### Fee and Size Calculation

- **Location**: Methods implemented in the `Transaction` struct in `transaction.rs`
- **Details**: Calculates the transaction fees and sizes which are crucial for the transaction selection algorithm.

### Block Construction

- **Location**: `block.rs`
- **Description**: Handles the logic for constructing a block from validated and selected transactions.

## Results and Performance

The implementation achieves a performance score of about 65-75, without extensive optimizations. With further refinement, the system is theoretically capable of including more transactions per block, thereby increasing score.

## Conclusion

This test project successfully implements block building logic for a btc. It includes critical functionalities such as transaction validation, fee and size calculations, and block mining. This foundation allows for further development and optimization in future iterations.
