use std::collections::HashMap;

use serde::{Serialize, Serializer};
use rust_decimal::Decimal;

use crate::transactions::Transaction;

#[derive(Debug, Clone, Serialize)]
pub struct Account {
    pub client: u16,
    #[serde(serialize_with = "round_four_digits")]
    pub available: Decimal,
    #[serde(serialize_with = "round_four_digits")]
    pub held: Decimal,
    #[serde(serialize_with = "round_four_digits")]
    pub total: Decimal,
    pub locked: bool,
}

impl Account {
    pub fn new(client: u16) -> Account {
        Account { 
            client, 
            available: Decimal::ZERO,
            held: Decimal::ZERO,
            total: Decimal::ZERO,
            locked: false,
        }
    }

    pub fn deposit(&mut self, amount: Decimal) {
        if self.locked {
            return;
        }

        self.available += amount;
        self.total += amount;
    }

    pub fn withdraw(&mut self, amount: Decimal) {
        if self.locked {
            return;
        }

        if self.available >= amount {
            self.available -= amount;
            self.total -= amount;
        }
    }

    pub fn dispute(&mut self, transaction: Option<&&Transaction>) {
        if self.locked {
            return;
        }

        if transaction.is_some() {
            self.available -= transaction.unwrap().amount;
            self.held += transaction.unwrap().amount;
        }
    }

    pub fn resolve(&mut self, transaction: Option<&&Transaction>, disputed_transactions_by_id: &mut HashMap<u32, &Transaction>) {
        if self.locked {
            return;
        }

        if transaction.is_some() {
            if !is_under_dispute(transaction.unwrap().tx, disputed_transactions_by_id) {
                // The transaction is not under a dispute so we do nothing.
                return;
            }

            self.available += transaction.unwrap().amount;
            self.held -= transaction.unwrap().amount;
            // Mark the transaction as no longer being under a dispute.
            disputed_transactions_by_id.remove(&transaction.unwrap().tx);
        }
    }

    pub fn chargeback(&mut self, transaction: Option<&&Transaction>, disputed_transactions_by_id: &mut HashMap<u32, &Transaction>) {
        if self.locked {
            return;
        }

        if transaction.is_some() {
            if !is_under_dispute(transaction.unwrap().tx, disputed_transactions_by_id) {
                // The transaction is not under a dispute so we do nothing.
                return;
            }

            self.held -= transaction.unwrap().amount;
            self.total -= transaction.unwrap().amount;
            self.locked = true;
            // Mark the transaction as no longer being under a dispute.
            disputed_transactions_by_id.remove(&transaction.unwrap().tx);
        }
    }
}

fn is_under_dispute(tx_id: u32, disputed_transactions_by_id: &mut HashMap<u32, &Transaction>) -> bool {
    let transaction_is_under_dispute = disputed_transactions_by_id.get(&tx_id);
    if transaction_is_under_dispute.is_some() {
        return true;
    }

    return false;
}

fn round_four_digits<S>(x: &Decimal, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&x.round_dp(4).to_string())
}
