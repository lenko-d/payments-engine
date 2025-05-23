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

const DEPOSIT: &str = "deposit";
const WITHDRAWAL: &str = "withdrawal";
const DISPUTE: &str = "dispute";
const RESOLVE: &str = "resolve";
const CHARGEBACK: &str = "chargeback";

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
    let mut disputed_transactions_by_id = HashMap::new();
    for transaction in transactions.iter() {
        if transaction.type_ == DISPUTE {
            let transaction_is_under_dispute = disputed_transactions_by_id.get(&transaction.tx);
            if transaction_is_under_dispute.is_some() {
                // The transaction is already under dispute so we don't do anything.
                // We ignore(skip) the dispute request.
                continue;
            }
        
            disputed_transactions_by_id.insert(transaction.tx, transaction);
        }

        // dont store certain types of transactions in the lookup table
        // because they provide a reference tr id instead of actual current tr id.
        // The reference tr id overrides the actual tr id.
        if transaction.type_ != DISPUTE && transaction.type_ != RESOLVE && transaction.type_!= CHARGEBACK {
            transactions_by_id.insert(transaction.tx, transaction);
        }

        let mut account = Account::new(transaction.client);
        if accounts.contains_key(&transaction.client) {
            account = accounts.get(&transaction.client).unwrap().clone();
        }

        match transaction.type_.as_str() {
            DEPOSIT => account.deposit(transaction.amount),
            WITHDRAWAL => account.withdraw(transaction.amount),
            DISPUTE => account.dispute(transactions_by_id.get(&transaction.tx)),
            RESOLVE => account.resolve(transactions_by_id.get(&transaction.tx), &mut disputed_transactions_by_id),
            CHARGEBACK => account.chargeback(transactions_by_id.get(&transaction.tx), &mut disputed_transactions_by_id),
            _ => () // ignore unsupported or unrecognized transaction types
        }

        accounts.insert(transaction.client,account);
    }
    
    accounts.values().cloned().collect()
}


#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use crate::{account::Account, transactions::{process_transactions, Transaction, DISPUTE, DEPOSIT, WITHDRAWAL, RESOLVE, CHARGEBACK}};

    const CLIENT_ID_ONE: u16 = 1;
    const CLIENT_ID_TWO: u16 = 2;

    fn deposit_transactions(tx_id1: u32, tx_id2: u32, tx_id3: u32) -> Vec<Transaction> {
        let t1: Transaction = Transaction{
            type_: DEPOSIT.to_owned(),
            client: CLIENT_ID_ONE,
            tx: tx_id1,
            amount: Decimal::new(7, 0)
        };

        let t2: Transaction = Transaction{
            type_: DEPOSIT.to_owned(),
            client: CLIENT_ID_TWO,
            tx: tx_id2,
            amount: Decimal::new(17, 0)
        };

        let t3: Transaction = Transaction{
            type_: DEPOSIT.to_owned(),
            client: CLIENT_ID_TWO,
            tx: tx_id3,
            amount: Decimal::new(27, 0)
        };

        return vec![t1, t2, t3];
    }

    fn withdrawal_transactions(tx_id1: u32, tx_id2: u32, tx_id3: u32) -> Vec<Transaction> {
        let t1: Transaction = Transaction{
            type_: WITHDRAWAL.to_owned(),
            client: CLIENT_ID_ONE,
            tx: tx_id1,
            amount: Decimal::new(7, 0)
        };

        let t2: Transaction = Transaction{
            type_: WITHDRAWAL.to_owned(),
            client: CLIENT_ID_TWO,
            tx: tx_id2,
            amount: Decimal::new(15, 0)
        };

        let t3: Transaction = Transaction{
            type_: WITHDRAWAL.to_owned(),
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
    assert!(account.total == Decimal::from_str_exact("44").unwrap());
 }

 #[test]
 fn multiple_withdrawals(){
    let mut transactions = deposit_transactions(1,2,3);
    transactions.append(&mut withdrawal_transactions(4,5,6));

    let accounts = process_transactions(&mut transactions);
 
    let account = get_account_by_client_id(CLIENT_ID_TWO, accounts).unwrap();
    assert!(account.available == Decimal::from_str_exact("4").unwrap());
    assert!(account.total == Decimal::from_str_exact("4").unwrap());
 }

 #[test]
 fn withdraw_more_than_available(){
    let mut transactions = deposit_transactions(1,2,3);
    transactions.append(&mut withdrawal_transactions(4,5,6));
    transactions.append(&mut withdrawal_transactions(7,8,9));

    let accounts = process_transactions(&mut transactions);
 
    let account = get_account_by_client_id(CLIENT_ID_TWO, accounts).unwrap();
    assert!(account.available == Decimal::from_str_exact("4").unwrap());
    assert!(account.total == Decimal::from_str_exact("4").unwrap());
 }

 #[test]
 fn withdraw_as_much_as_available(){
    let mut transactions = deposit_transactions(1,2,3);
    transactions.append(&mut withdrawal_transactions(4,5,6));
    transactions.append(&mut withdrawal_transactions(7,8,9));

    let t1: Transaction = Transaction{
            type_: WITHDRAWAL.to_owned(),
            client: CLIENT_ID_TWO,
            tx: 10,
            amount: Decimal::new(4, 0)
        };
    let mut tx_withdraw_all_available = vec![t1];
    transactions.append(&mut tx_withdraw_all_available);
    
    let accounts = process_transactions(&mut transactions);
 
    let account = get_account_by_client_id(CLIENT_ID_TWO, accounts).unwrap();
    assert!(account.available == Decimal::from_str_exact("0").unwrap());
    assert!(account.total == Decimal::from_str_exact("0").unwrap());
 }

 #[test]
 fn dispute(){
    let mut transactions = deposit_transactions(1,2,3);
    let t1: Transaction = Transaction{
            type_: DISPUTE.to_owned(),
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
 fn dispute_transaction_that_is_already_disputed(){
    let mut transactions = deposit_transactions(1,2,3);
    let t1: Transaction = Transaction{
            type_: DISPUTE.to_owned(),
            client: CLIENT_ID_TWO,
            tx: 2,
            amount: Decimal::ZERO,
        };
    let mut tx_dispute = vec![t1];
    transactions.append(&mut tx_dispute);

    let t2: Transaction = Transaction{
            type_: DISPUTE.to_owned(),
            client: CLIENT_ID_TWO,
            tx: 2,
            amount: Decimal::ZERO,
    };
    let mut tx_dispute_again = vec![t2];
    transactions.append(&mut tx_dispute_again);

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
            type_: DISPUTE.to_owned(),
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

 #[test]
 fn resolve(){
    let mut transactions = deposit_transactions(1,2,3);
    let t1: Transaction = Transaction{
            type_: DISPUTE.to_owned(),
            client: CLIENT_ID_TWO,
            tx: 2,
            amount: Decimal::ZERO,
    };
    let mut tx_dispute = vec![t1];
    transactions.append(&mut tx_dispute);

    let t2: Transaction = Transaction{
            type_: RESOLVE.to_owned(),
            client: CLIENT_ID_TWO,
            tx: 2,
            amount: Decimal::ZERO,
    };
    let mut tx_resolve = vec![t2];
    transactions.append(&mut tx_resolve);

    let accounts = process_transactions(&mut transactions);
 
    let account = get_account_by_client_id(CLIENT_ID_TWO, accounts).unwrap();
    assert!(account.available == Decimal::from_str_exact("44").unwrap());
    assert!(account.held == Decimal::from_str_exact("0").unwrap());
    assert!(account.total == Decimal::from_str_exact("44").unwrap());
 }

 #[test]
 fn resolve_non_existent_transaction(){
    let mut transactions = deposit_transactions(1,2,3);
    let t1: Transaction = Transaction{
            type_: DISPUTE.to_owned(),
            client: CLIENT_ID_TWO,
            tx: 2,
            amount: Decimal::ZERO,
    };
    let mut tx_dispute = vec![t1];
    transactions.append(&mut tx_dispute);

    let t2: Transaction = Transaction{
            type_: RESOLVE.to_owned(),
            client: CLIENT_ID_TWO,
            tx: 7,
            amount: Decimal::ZERO,
    };
    let mut tx_resolve = vec![t2];
    transactions.append(&mut tx_resolve);

    let accounts = process_transactions(&mut transactions);
 
    let account = get_account_by_client_id(CLIENT_ID_TWO, accounts).unwrap();
    assert!(account.available == Decimal::from_str_exact("27").unwrap());
    assert!(account.held == Decimal::from_str_exact("17").unwrap());
    assert!(account.total == Decimal::from_str_exact("44").unwrap());
 }

 
 #[test]
 fn resolve_transaction_that_is_not_under_dispute(){
    let mut transactions = deposit_transactions(1,2,3);

    let t2: Transaction = Transaction{
            type_: RESOLVE.to_owned(),
            client: CLIENT_ID_TWO,
            tx: 2,
            amount: Decimal::ZERO,
    };
    let mut tx_resolve = vec![t2];
    transactions.append(&mut tx_resolve);

    let accounts = process_transactions(&mut transactions);
 
    let account = get_account_by_client_id(CLIENT_ID_TWO, accounts).unwrap();
    assert!(account.available == Decimal::from_str_exact("44").unwrap());
    assert!(account.held == Decimal::from_str_exact("0").unwrap());
    assert!(account.total == Decimal::from_str_exact("44").unwrap());
 }

 #[test]
 fn resolve_transaction_that_was_already_resolved(){
    let mut transactions = deposit_transactions(1,2,3);
    let t1: Transaction = Transaction{
            type_: DISPUTE.to_owned(),
            client: CLIENT_ID_TWO,
            tx: 2,
            amount: Decimal::ZERO,
    };
    let mut tx_dispute = vec![t1];
    transactions.append(&mut tx_dispute);

    let t2: Transaction = Transaction{
            type_: RESOLVE.to_owned(),
            client: CLIENT_ID_TWO,
            tx: 2,
            amount: Decimal::ZERO,
    };
    let mut tx_resolve = vec![t2];
    transactions.append(&mut tx_resolve);

    let t3: Transaction = Transaction{
            type_: RESOLVE.to_owned(),
            client: CLIENT_ID_TWO,
            tx: 2,
            amount: Decimal::ZERO,
    };
    let mut tx_resolve_again = vec![t3];
    transactions.append(&mut tx_resolve_again);

    let accounts = process_transactions(&mut transactions);
 
    let account = get_account_by_client_id(CLIENT_ID_TWO, accounts).unwrap();
    assert!(account.available == Decimal::from_str_exact("44").unwrap());
    assert!(account.held == Decimal::from_str_exact("0").unwrap());
    assert!(account.total == Decimal::from_str_exact("44").unwrap());
 }

  #[test]
 fn chargeback(){
    let mut transactions = deposit_transactions(1,2,3);
    let t1: Transaction = Transaction{
            type_: DISPUTE.to_owned(),
            client: CLIENT_ID_TWO,
            tx: 2,
            amount: Decimal::ZERO,
        };
    let mut tx_dispute = vec![t1];
    transactions.append(&mut tx_dispute);

    let t2: Transaction = Transaction{
            type_: CHARGEBACK.to_owned(),
            client: CLIENT_ID_TWO,
            tx: 2,
            amount: Decimal::ZERO,
        };
    let mut tx_chargeback = vec![t2];
    transactions.append(&mut tx_chargeback);

    let accounts = process_transactions(&mut transactions);
 
    let account = get_account_by_client_id(CLIENT_ID_TWO, accounts).unwrap();
    assert!(account.available == Decimal::from_str_exact("27").unwrap());
    assert!(account.held == Decimal::from_str_exact("0").unwrap());
    assert!(account.total == Decimal::from_str_exact("27").unwrap());
    assert!(account.locked == true);
 }

 #[test]
 fn chargeback_for_transaction_not_under_dispute(){
    let mut transactions = deposit_transactions(1,2,3);

    let t2: Transaction = Transaction{
            type_: CHARGEBACK.to_owned(),
            client: CLIENT_ID_TWO,
            tx: 2,
            amount: Decimal::ZERO,
        };
    let mut tx_chargeback = vec![t2];
    transactions.append(&mut tx_chargeback);

    let accounts = process_transactions(&mut transactions);
 
    let account = get_account_by_client_id(CLIENT_ID_TWO, accounts).unwrap();
    assert!(account.available == Decimal::from_str_exact("44").unwrap());
    assert!(account.held == Decimal::from_str_exact("0").unwrap());
    assert!(account.total == Decimal::from_str_exact("44").unwrap());
    assert!(account.locked == false);
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