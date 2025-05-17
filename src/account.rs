
use serde::Serialize;

use crate::transactions::Transaction;

#[derive(Debug, Serialize)]
pub struct Account {
    pub client: u16,
    pub available: f64,
    pub held: f64,
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