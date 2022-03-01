use std::{process, env, error::Error};
use pay_engine::{
    account::Account,
    actor::ActorHandle,
    account_registry::AccountRegistryHandle,
    transaction_registry::TransactionRegistryHandle,
};

fn fail(error: Box<dyn Error>) {
    eprintln!("{}", error);
    process::exit(1);
}

fn success(accounts: Vec<Account>) {
    let mut header_line = String::new();
    for header in vec!["client", "available", "held", "total", "locked"] {
        header_line.push_str(format!("{},", header).as_str());
    }
    header_line.pop();
    let header_line = header_line;
    println!("{}", header_line);
    for account in accounts.into_iter() {
        println!("{}", account);
    }
}

fn usage(program_name: &str) {
    eprintln!("usage: {} input_file", program_name);
    process::exit(2);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if let 1 = args.len() {
        usage(&args[0]);
    }

    // "Server"
    let result = tokio::runtime::Runtime::new().unwrap().block_on(async {
        // "Database"
        // (ideally we'd change the signature of `new()` to require
        // users to supply their own log and persist closures for writing
        // data and logging actions)
        let accounts = AccountRegistryHandle::new();
        let transactions = TransactionRegistryHandle::new();

        // Ideally this would be an actor processing streams sequentially
        // from multiple concurrent requests. For now we assume this current
        // "request" is next in queue
        match pay_engine::process(
            args[1].clone(),
            accounts.clone(),
            transactions.clone()).await {
            Ok(()) => Ok(accounts.query().await),
            Err(err) => Err(err),
        }
    });

    match result {
        Ok(accounts) => success(accounts),
        Err(err) => fail(err),
    }
}
