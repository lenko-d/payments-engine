# Payments engine

 Payments engine that reads a series of transactions
from a CSV input file, updates client accounts, handles disputes and chargebacks, and then outputs the
state of clients accounts as a CSV to standard output.

## Design choices and assumptions

A withdraw request will not do anything if the available money in the account is less than the requested amount.
All types of requests(deposit, withdraw...etc) will be ignored if the account is locked(frozen).
If the input file contains unsupported or unrecognized transaction type then that line will be skipped.
A dispute request will be ignored if there is a pending dispute for the same transaction.
A request to resolve a dispute will be ignored if the dispute was already resolved.
A chargeback will be ignored if the transaction is not under a dispute or doesn't exist.
After a successful chargeback the transaction is considered to no longer be under a dispute.

## How to run unit tests
cargo test

## How to test using the sample transactions file
cargo run -- transactions.csv > accounts.csv