# Payments engine

 Payments engine that reads a series of transactions
from a CSV input file, updates client accounts, handles disputes and chargebacks, and then outputs the
state of clients accounts as a CSV to standard output.

## Design choices and assumptions

A withdraw request will not do anything if the available money in the account is less than the requested amount.
If the input file contains unsupported or unrecognized transaction type then that line will be skipped.
A dispute request will be ignored if there is a pending dispute for the same transaction.
A request to resolve a dispute will be ignored if the dispute was already resolved.

## How to run unit tests
cargo test

## How to test using the sample transactions file
cargo run -- transactions.csv > accounts.csv