extern crate csv;

use std::{collections::HashMap, error::Error};
use rust_decimal::Decimal;
use serde::Deserialize;
use crate::account::{Account};

#[derive(Debug, Clone, Deserialize)]
pub struct Transaction {
     #[serde(rename = "type")]
    pub type_: String,
    pub client: u16,
    pub tx: u32,
    pub amount: Decimal,
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


#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use crate::{account::Account, transactions::process_transactions};
    use super::Transaction;

    fn deposit_transactions() -> Vec<Transaction> {
        let t1: Transaction = Transaction{
            type_: "deposit".to_owned(),
            client: 1,
            tx: 1,
            amount: Decimal::new(7, 0)
        };

        let t2: Transaction = Transaction{
            type_: "deposit".to_owned(),
            client: 2,
            tx: 1,
            amount: Decimal::new(17, 0)
        };

        let t3: Transaction = Transaction{
            type_: "deposit".to_owned(),
            client: 2,
            tx: 1,
            amount: Decimal::new(27, 0)
        };

        return vec![t1, t2, t3];
    }

 #[test]
 fn multiple_deposits(){
    let mut transactions = deposit_transactions();

    let accounts = process_transactions(&mut transactions);
 
    let account = get_account_by_client_id(2, accounts).unwrap();
    assert!(account.available == Decimal::from_str_exact("44").unwrap());
 }

 fn get_account_by_client_id(clinet_id: u16, accounts: Vec<Account>) -> Option<Account> {
    for account in accounts {
        if account.client == clinet_id {
            return Some(account);
        }
    }

    return None;
 }
}