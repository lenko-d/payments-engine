
use serde::{Serialize, Serializer};
use rust_decimal::{prelude::FromPrimitive, Decimal};

use crate::transactions::Transaction;

#[derive(Debug, Serialize)]
pub struct Account {
    pub client: u16,
    #[serde(serialize_with = "decimal_round")]
    pub available: f64,
    #[serde(serialize_with = "decimal_round")]
    pub held: f64,
    #[serde(serialize_with = "decimal_round")]
    pub total: f64,
    pub locked: bool,
}


impl Account {
    pub fn new(client: u16) -> Account {
        Account { 
            client, 
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false,
        }
    }

    pub fn deposit(&mut self, amount: f64) {
        self.available += amount;
        self.total += amount;
    }

    pub fn withdraw(&mut self, amount: f64) {
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

fn decimal_round<S>(x: &f64, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let rounded = Decimal::from_f64(*x).unwrap().round_dp(4);
    s.serialize_str(&rounded.to_string())
}
