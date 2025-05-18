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
    let mut accounts: HashMap<u16, Account> = HashMap::new();
    let mut transactions_by_id = HashMap::new();
    for transaction in transactions.iter() {
        transactions_by_id.insert(transaction.tx, transaction);

        let mut account = Account::new(transaction.client);
        if accounts.contains_key(&transaction.client) {
            account = accounts.get(&transaction.client).unwrap().clone();
        }

        match transaction.type_.as_str() {
            "deposit" => account.deposit(transaction.amount),
            "withdrawal" => account.withdraw(transaction.amount),
            "dispute" => account.dispute(transactions_by_id.get(&transaction.tx)),
            "resolve" => account.resolve(transactions_by_id.get(&transaction.tx)),
            "chargeback" => account.chargeback(transactions_by_id.get(&transaction.tx)),
            _ => println!("unhandled TODO")
        }

        accounts.insert(transaction.client,account);
    }
    
    accounts.values().cloned().collect()
}

