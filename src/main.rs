mod account;
mod transactions;

use crate::account::Account;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: \n payments-engine <transactions.csv>");
        return;
    }

    let transactions_file = &args[1];
    
    let accounts = transactions::from_file(transactions_file);

    print(&accounts);
}

fn print(accounts: &Vec<Account>) {
    let mut writer = csv::Writer::from_writer(std::io::stdout());

    for account in accounts.iter() {
            writer
                .serialize(account)
                .ok()
                .expect("Unable to write to output file.");
    }
}
