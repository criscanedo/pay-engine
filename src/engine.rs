pub mod account;
pub mod transaction;
pub mod messages;
pub mod account_registry;
pub mod transaction_registry;
pub mod actor;
use std::{error::Error, vec::IntoIter};
use csv::ReaderBuilder;
use tokio::{fs::File, task::JoinHandle};
use tokio_stream::{iter, StreamExt};
use crate::{
    transaction::{Transaction, TransactionType},
    account::Account,
    account_registry::AccountRegistryHandle,
    transaction_registry::TransactionRegistryHandle,
};

pub async fn process(file_name: String,
                 accounts: AccountRegistryHandle,
                 transactions: TransactionRegistryHandle) -> Result<(), Box<dyn Error + Send + Sync>> {
    let handle: JoinHandle<Result<(), Box<dyn Error + Send + Sync>>> = tokio::spawn(async move {
        let mut stream = iter(parse_file(&file_name).await?);

        // The following operations should be handled robustly with
        // the use of Result type and error propogation instead of
        // the Option type that is currently returned from the calls
        while let Some(msg) = stream.next().await {
            let account = accounts.lookup(msg.client_id).await.unwrap();

            // The following logic should be greatly improved to handle
            // fallible operations as mentioned in the previous comment above
            if account.locked {
                eprintln!("client account is frozen, \
                          cannot perform the following transaction: {:?}",
                         msg);
            } else {
                // Writing to a data source should be transactional
                match msg.transaction_type {
                    TransactionType::Deposit => {
                        if let Some(_) = accounts
                            .persist(account.apply(msg.amount.unwrap(), msg.transaction_type)).await {
                                let _ = transactions.persist(msg).await;
                            } else {
                                eprintln!("failed to write Account to registry: {:?}", msg);
                            }
                    },
                    TransactionType::Withdrawal => {
                        let amount = msg.amount.unwrap();
                        if amount > account.available {
                            eprintln!("insufficient funds, \
                                      cannot perform the following transaction: {:?}",
                                     msg);
                        } else {
                            if let Some(_) = accounts
                                .persist(account.apply(amount, msg.transaction_type)).await {
                                    let _ = transactions.persist(msg).await;
                                } else {
                                    eprintln!("failed to write Account to registry: {:?}", msg);
                                }
                        }
                    },
                    TransactionType::Dispute => {
                        if let Some(trans) = transactions.lookup(msg.transaction_id).await {
                            if let Some(_) = accounts
                                .persist(account.apply(trans.amount.unwrap(), msg.transaction_type)).await {
                                    let _ = transactions.persist(trans.dispute()).await;
                                };
                        } else {
                            eprintln!("cannot reference transaction that does not exist: {:?}", msg);
                        }
                    },
                    TransactionType::Resolve => {
                        if let Some(disputee) = transactions.lookup(msg.transaction_id).await {
                            if disputee.disputed {
                                if let Some(_) = accounts
                                    .persist(account.apply(disputee.amount.unwrap(), msg.transaction_type)).await {
                                        let _ = transactions.persist(disputee.resolve()).await;
                                    };
                            } else {
                                eprintln!("transaction referred is not disputed: {:?}", msg);
                            }
                        } else {
                            eprintln!("cannot reference transaction that does not exist: {:?}", msg);
                        }
                    },
                    TransactionType::Chargeback => {
                        if let Some(disputee) = transactions.lookup(msg.transaction_id).await {
                            if disputee.disputed {
                                if let Some(_) = accounts
                                    .persist(account.apply(disputee.amount.unwrap(), msg.transaction_type)).await {
                                        let _ = transactions.persist(disputee.resolve()).await;
                                    };
                            } else {
                                eprintln!("transaction referred is not disputed: {:?}", msg);
                            }
                        } else {
                            eprintln!("cannot reference transaction that does not exist: {:?}", msg);
                        }
                    },
                }
            }
        } // end of while
        Ok(())
    });
    handle.await?
}

async fn parse_file(file_name: &str) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
    match ReaderBuilder::new()
        .flexible(true)
        .from_reader(File::open(file_name)
                     .await?
                     .try_into_std()
                     .unwrap())
        .deserialize::<Transaction>()
        .collect() {
            Ok(transactions) => Ok(transactions),
            Err(err) => Err(Box::new(err)),
        }
}
