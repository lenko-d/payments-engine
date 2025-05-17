extern crate csv;

use std::{collections::HashMap, error::Error};
use serde::Deserialize;
use crate::account::{Account};

#[derive(Debug, Deserialize)]
pub struct Transaction {
     #[serde(rename = "type")]
    pub type_: String,
    pub client: u16,
    pub tx: u32,
    pub amount: f64,
}

pub fn from_file(transactions_file: &str) -> Vec<Account>{
    let mut transactions = read_transactions(transactions_file).unwrap();

    process_transactions(&mut transactions)
}

fn read_transactions(file_path: &str) -> Result<Vec<Transaction>, Box<dyn Error>> {
    let mut reader = csv::ReaderBuilder::new().trim(csv::Trim::All).from_path(file_path).unwrap();
    let mut transactions = vec![];

    for transaction in reader.deserialize() {
        let tx: Transaction = transaction.unwrap();
        transactions.push(tx);
    }

    Ok(transactions)
}

fn process_transactions(transactions: &mut Vec<Transaction>) -> Vec<Account> {
    let mut accounts = Vec::new();
    let mut transactions_by_id = HashMap::new();
    for transaction in transactions.iter() {
        transactions_by_id.insert(transaction.tx, transaction);

        let mut acct = Account::new(transaction.client);

        match transaction.type_.as_str() {
            "deposit" => acct.deposit(transaction.amount),
            "withdrawal" => acct.withdraw(transaction.amount),
            "dispute" => acct.dispute(transactions_by_id.get(&transaction.tx)),
            "resolve" => acct.resolve(transactions_by_id.get(&transaction.tx)),
            "chargeback" => acct.chargeback(transactions_by_id.get(&transaction.tx)),
            _ => println!("unhandled TODO")
        }

        accounts.push(acct);
    }
    
    accounts
}

