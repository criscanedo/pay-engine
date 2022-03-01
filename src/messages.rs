use tokio::sync::oneshot;
use crate::{Account, Transaction};
use serde::Deserialize;

pub enum AccountRegistryMsg {
    Lookup {
        id: u16,
        respond_to: oneshot::Sender<Option<Account>>,
    },
    Register {
        id: u16,
        respond_to: oneshot::Sender<Option<Account>>,
    },
    Persist {
        account: Account,
        respond_to: oneshot::Sender<Option<Account>>,
    },
    Query {
        respond_to: oneshot::Sender<Vec<Account>>,
    },
}

pub enum TransactionRegistryMsg {
    Lookup {
        id: u32,
        respond_to: oneshot::Sender<Option<Transaction>>,
    },
    Persist {
        transaction: Transaction,
        respond_to: oneshot::Sender<Option<Transaction>>,
    },
}

// Ideal message enum to send between actors
#[derive(Debug, Deserialize)]
pub enum TransactionMsg {
    Deposit { client: u16, tx: u32, amount: f64 },
    Withdrawal { client: u16, tx: u32, amount: f64 },
    Dispute { client: u16, tx: u32 },
    Resolve { client: u16, tx: u32 },
    Chargeback { client: u16, tx: u32 },
}

trait Message {}
impl Message for AccountRegistryMsg {}
impl Message for TransactionRegistryMsg {}
impl Message for TransactionMsg {}
