# Payments engine

 Payments engine that reads a series of transactions
from a CSV, updates client accounts, handles disputes and chargebacks, and then outputs the
state of clients accounts as a CSV.

## Design choices and assumptions

A withdraw request will not do anything if the available money in the account is less than the requested amount.

## How to run unit tests
cargo test