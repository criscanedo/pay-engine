use serde::Deserialize;

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "tx")]
    pub transaction_id: u32,
    pub amount: Option<f64>,
    #[serde(skip)]
    pub disputed: bool,
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

impl Transaction {
    // Should also record the transaction id of the dispute to associate
    // it with this transaction for persisting and logging
    pub fn dispute(&self) -> Self {
        Transaction {
            disputed: true,
            ..*self
        }
    }
    pub fn resolve(&self) -> Self {
        Transaction {
            disputed: false,
            ..*self
        }
    }
}
