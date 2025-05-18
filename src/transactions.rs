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
        if transaction.type_ != "dispute" {
            transactions_by_id.insert(transaction.tx, transaction);
        }

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
            _ => () // ignore unsupported or unrecognized transaction types
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

    const CLIENT_ID_ONE: u16 = 1;
    const CLIENT_ID_TWO: u16 = 2;

    fn deposit_transactions(tx_id1: u32, tx_id2: u32, tx_id3: u32) -> Vec<Transaction> {
        let t1: Transaction = Transaction{
            type_: "deposit".to_owned(),
            client: CLIENT_ID_ONE,
            tx: tx_id1,
            amount: Decimal::new(7, 0)
        };

        let t2: Transaction = Transaction{
            type_: "deposit".to_owned(),
            client: CLIENT_ID_TWO,
            tx: tx_id2,
            amount: Decimal::new(17, 0)
        };

        let t3: Transaction = Transaction{
            type_: "deposit".to_owned(),
            client: CLIENT_ID_TWO,
            tx: tx_id3,
            amount: Decimal::new(27, 0)
        };

        return vec![t1, t2, t3];
    }

    fn withdrawal_transactions(tx_id1: u32, tx_id2: u32, tx_id3: u32) -> Vec<Transaction> {
        let t1: Transaction = Transaction{
            type_: "withdrawal".to_owned(),
            client: CLIENT_ID_ONE,
            tx: tx_id1,
            amount: Decimal::new(7, 0)
        };

        let t2: Transaction = Transaction{
            type_: "withdrawal".to_owned(),
            client: CLIENT_ID_TWO,
            tx: tx_id2,
            amount: Decimal::new(15, 0)
        };

        let t3: Transaction = Transaction{
            type_: "withdrawal".to_owned(),
            client: CLIENT_ID_TWO,
            tx: tx_id3,
            amount: Decimal::new(25, 0)
        };

        return vec![t1, t2, t3];
    }

 #[test]
 fn multiple_deposits(){
    let mut transactions = deposit_transactions(1,2,3);

    let accounts = process_transactions(&mut transactions);
 
    let account = get_account_by_client_id(CLIENT_ID_TWO, accounts).unwrap();
    assert!(account.available == Decimal::from_str_exact("44").unwrap());
 }

 #[test]
 fn multiple_withdrawals(){
    let mut transactions = deposit_transactions(1,2,3);
    transactions.append(&mut withdrawal_transactions(4,5,6));

    let accounts = process_transactions(&mut transactions);
 
    let account = get_account_by_client_id(CLIENT_ID_TWO, accounts).unwrap();
    assert!(account.available == Decimal::from_str_exact("4").unwrap());
 }

 #[test]
 fn withdraw_more_than_available(){
    let mut transactions = deposit_transactions(1,2,3);
    transactions.append(&mut withdrawal_transactions(4,5,6));
    transactions.append(&mut withdrawal_transactions(7,8,9));

    let accounts = process_transactions(&mut transactions);
 
    let account = get_account_by_client_id(CLIENT_ID_TWO, accounts).unwrap();
    assert!(account.available == Decimal::from_str_exact("4").unwrap());
 }

 #[test]
 fn withdraw_as_much_as_available(){
    let mut transactions = deposit_transactions(1,2,3);
    transactions.append(&mut withdrawal_transactions(4,5,6));
    transactions.append(&mut withdrawal_transactions(7,8,9));

    let t1: Transaction = Transaction{
            type_: "withdrawal".to_owned(),
            client: CLIENT_ID_TWO,
            tx: 10,
            amount: Decimal::new(4, 0)
        };
    let mut tx_withdraw_all_available = vec![t1];
    transactions.append(&mut tx_withdraw_all_available);
    
    let accounts = process_transactions(&mut transactions);
 
    let account = get_account_by_client_id(CLIENT_ID_TWO, accounts).unwrap();
    assert!(account.available == Decimal::from_str_exact("0").unwrap());
 }

 #[test]
 fn dispute(){
    let mut transactions = deposit_transactions(1,2,3);
    let t1: Transaction = Transaction{
            type_: "dispute".to_owned(),
            client: CLIENT_ID_TWO,
            tx: 2,
            amount: Decimal::ZERO,
        };
    let mut tx_dispute = vec![t1];
    transactions.append(&mut tx_dispute);

    let accounts = process_transactions(&mut transactions);
 
    let account = get_account_by_client_id(CLIENT_ID_TWO, accounts).unwrap();
    assert!(account.available == Decimal::from_str_exact("27").unwrap());
    assert!(account.held == Decimal::from_str_exact("17").unwrap());
    assert!(account.total == Decimal::from_str_exact("44").unwrap());
 }

 #[test]
 fn dispute_non_existent_transaction(){
    let mut transactions = deposit_transactions(1,2,3);
    let t1: Transaction = Transaction{
            type_: "dispute".to_owned(),
            client: CLIENT_ID_TWO,
            tx: 7,
            amount: Decimal::ZERO,
        };
    let mut tx_dispute = vec![t1];
    transactions.append(&mut tx_dispute);

    let accounts = process_transactions(&mut transactions);
 
    let account = get_account_by_client_id(CLIENT_ID_TWO, accounts).unwrap();
    assert!(account.available == Decimal::from_str_exact("44").unwrap());
    assert!(account.held == Decimal::from_str_exact("0").unwrap());
    assert!(account.total == Decimal::from_str_exact("44").unwrap());
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