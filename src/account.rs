
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
        self.available += amount;
        self.total += amount;
    }

    pub fn withdraw(&mut self, amount: Decimal) {
        self.available -= amount;
        self.total -= amount;
    }

    pub fn dispute(&mut self, disputed_transaction: Option<&&Transaction>) {
        if disputed_transaction.is_some() {
            self.available -= disputed_transaction.unwrap().amount;
            self.held += disputed_transaction.unwrap().amount;
        }
    }

    pub fn resolve(&mut self, disputed_transaction: Option<&&Transaction>) {
        if disputed_transaction.is_some() {
            self.available += disputed_transaction.unwrap().amount;
            self.held -= disputed_transaction.unwrap().amount;
        }
    }

    pub fn chargeback(&mut self, disputed_transaction: Option<&&Transaction>) {
        if disputed_transaction.is_some() {
            self.held -= disputed_transaction.unwrap().amount;
            self.total -= disputed_transaction.unwrap().amount;
            self.locked = true;
        }
    }
}

fn round_four_digits<S>(x: &Decimal, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&x.round_dp(4).to_string())
}
