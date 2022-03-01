use tokio::sync::{oneshot, mpsc};
use std::collections::HashMap;
use crate::{
    messages::TransactionRegistryMsg,
    transaction::Transaction,
    actor::{Actor, ActorHandle},
};

struct TransactionRegistry {
    receiver: mpsc::Receiver<TransactionRegistryMsg>,
    transactions: HashMap<u32, Option<Transaction>>,
}

impl Actor for TransactionRegistry {}

impl TransactionRegistry {
    fn new(receiver: mpsc::Receiver<TransactionRegistryMsg>) -> Self {
        TransactionRegistry {
            receiver,
            transactions: HashMap::new(),
        }
    }

    fn handle_message(&mut self, msg: TransactionRegistryMsg) {
        // Patterns should instead call a closure registered with the struct
        // upon initialization to allow for different users to specify their
        // own lookup/persist functionality to their own sources
        match msg {
            TransactionRegistryMsg::Lookup { id, respond_to } => {
                let _ = respond_to.send(match self.transactions.get(&id) {
                    Some(transaction) => *transaction,
                    None => None,
                });
            },
            TransactionRegistryMsg::Persist { transaction, respond_to } => {
                let _ = respond_to.send(match self.transactions.get_mut(&transaction.transaction_id) {
                    Some(entry) => {
                        // Could use previous Transaction state for logging
                        let _ = entry.replace(transaction);
                        *entry
                    },
                    None => {
                        let tran = Some(transaction);
                        self.transactions.insert(tran.unwrap().transaction_id, tran);
                        tran
                    }
                });
            },
        }
    }
}

async fn start_registry(mut registry: TransactionRegistry) {
    while let Some(msg) = registry.receiver.recv().await {
        registry.handle_message(msg);
    }
}

#[derive(Clone)]
pub struct TransactionRegistryHandle {
    sender: mpsc::Sender<TransactionRegistryMsg>,
}

impl ActorHandle for TransactionRegistryHandle {
    fn new() -> Self {
        // Actor configuration when firing up a server should be handled by
        // a server configuration file
        let (sender, receiver) = mpsc::channel(5);
        let registry = TransactionRegistry::new(receiver);
        tokio::spawn(start_registry(registry));
        Self { sender }
    }
}

impl TransactionRegistryHandle {
    // These functions should instead respond to Result types returned
    // by the actor in order to perform logging accordingly and return
    // Result types themselves
    pub async fn persist(&self, transaction: Transaction) -> Option<Transaction> {
        let (send, recv) = oneshot::channel();
        let _ = self.sender
            .send(TransactionRegistryMsg::Persist {
                transaction,
                respond_to: send,
            }).await;
        recv.await.expect("TransactionRegistry task killed")
    }

    pub async fn lookup(&self, transaction_id: u32) -> Option<Transaction> {
        let (send, recv) = oneshot::channel();
        let _ = self.sender
            .send(TransactionRegistryMsg::Lookup {
                id: transaction_id, respond_to: send
            }).await;
        recv.await.expect("TransactionRegistry task killed")
    }
}
