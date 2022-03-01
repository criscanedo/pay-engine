use tokio::sync::{oneshot, mpsc};
use std::collections::HashMap;
use crate::{
    messages::AccountRegistryMsg,
    account::Account,
    actor::{Actor, ActorHandle},
};

struct AccountRegistry {
    receiver: mpsc::Receiver<AccountRegistryMsg>,
    accounts: HashMap<u16, Option<Account>>,
}

impl Actor for AccountRegistry {}

impl AccountRegistry {
    fn new(receiver: mpsc::Receiver<AccountRegistryMsg>) -> Self {
        AccountRegistry {
            receiver,
            accounts: HashMap::new(),
        }
    }

    fn handle_message(&mut self, msg: AccountRegistryMsg) {
        // Patterns should instead call a closure registered with the struct
        // upon initialization to allow for different users to specify their
        // own lookup/persist functionality to their own sources
        match msg {
            AccountRegistryMsg::Lookup { id, respond_to } => {
                let _ = respond_to.send(match self.accounts.get(&id) {
                    Some(account) => *account,
                    None => None,
                });
            },
            AccountRegistryMsg::Register { id, respond_to } => {
                let _ = respond_to.send(match self.accounts.get(&id) {
                    Some(acc) => *acc,
                    None => {
                        let acc = Some(Account::new(id));
                        self.accounts.insert(id, acc);
                        acc
                    }
                });
            },
            AccountRegistryMsg::Persist { account, respond_to } => {
                let _ = respond_to.send(match self.accounts.get_mut(&account.client_id) {
                    Some(entry) => {
                        // Could use previous Account state for logging
                        let _ = entry.replace(account);
                        *entry
                    },
                    None => {
                        let acc = Some(account);
                        self.accounts.insert(acc.unwrap().client_id, acc);
                        acc
                    },
                });
            },
            AccountRegistryMsg::Query { respond_to } => {
                let _ = respond_to.send(self.accounts.values()
                                        .filter_map(|x| *x)
                                        .collect());
            },
        }
    }
}

async fn start_registry(mut registry: AccountRegistry) {
    while let Some(msg) = registry.receiver.recv().await {
        registry.handle_message(msg);
    }
}

#[derive(Clone)]
pub struct AccountRegistryHandle {
    sender: mpsc::Sender<AccountRegistryMsg>,
}

impl ActorHandle for AccountRegistryHandle {
    fn new() -> Self {
        // Channel capacity should be set by a server configuration
        let (sender, receiver) = mpsc::channel(5);
        let registry = AccountRegistry::new(receiver);
        tokio::spawn(start_registry(registry));
        Self { sender }
    }
}

impl AccountRegistryHandle {
    // These functions should instead respond to Result types returned
    // by the actor in order to perform logging accordingly
    pub async fn lookup(&self, client_id: u16) -> Option<Account> {
        let (send, recv) = oneshot::channel();
        let _ = self.sender.send(AccountRegistryMsg::Lookup {
            id: client_id, respond_to: send
        }).await;

        // If the lookup fails, the client account does not exist,
        // therefore send a subsequent `Register` message. Keeping this logic
        // in the handle allows for future modifications such as retriving the
        // client account from data source on the calling thread rather than in
        // the actor thread which would block
        match recv.await.expect("AccountRegistry task killed") {
            Some(account) => Some(account),
            None => {
                let (send, recv) = oneshot::channel();
                let _ = self.sender.send(AccountRegistryMsg::Register {
                    id: client_id,
                    respond_to: send,
                }).await;
                recv.await.expect("AccountRegistry task killed")
            }
        }
    }

    pub async fn persist(&self, account: Account) -> Option<Account> {
        let (send, recv) = oneshot::channel();
        let _ = self.sender.send(AccountRegistryMsg::Persist {
            account,
            respond_to: send
        }).await;
        recv.await.expect("AccountRegistry task killed")
    }

    pub async fn query(&self) -> Vec<Account> {
        let (send, recv) = oneshot::channel();
        let _ = self.sender.send(AccountRegistryMsg::Query {
            respond_to: send
        }).await;
        recv.await.expect("AccountRegistry task killed")
    }
}
