use serde::{Serialize};
use std::fmt;
use crate::transaction::TransactionType;

#[derive(Copy, Clone, Debug, Serialize)]
pub struct Account {
    #[serde(rename = "client")]
    pub client_id: u16,
    pub available: f64,
    pub held: f64,
    pub total: f64,
    pub locked: bool,
}

impl Account {
    pub fn new(client_id: u16) -> Self {
        Account {
            client_id,
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false,
        }
    }

    pub fn apply(&self, amount: f64, event: TransactionType) -> Self {
        match event {
            TransactionType::Deposit => self.credit(amount),
            TransactionType::Withdrawal => self.debit(amount),
            TransactionType::Dispute => self.dispute(amount),
            TransactionType::Resolve => self.resolve(amount),
            TransactionType::Chargeback => self.chargeback(amount),
        }
    }

    fn debit(&self, amount: f64) -> Self {
        self.credit(-amount)
    }

    fn credit(&self, amount: f64) -> Self {
        Account {
            available: self.available + amount,
            total: self.total + amount,
            ..*self
        }
    }

    fn dispute(&self, amount: f64) -> Self {
        self.resolve(-amount)
    }

    fn resolve(&self, amount: f64) -> Self {
        Account {
            available: self.available + amount,
            held: self.held - amount,
            ..*self
        }
    }

    fn chargeback(&self, amount: f64) -> Self {
        Account {
            held: self.held - amount,
            total: self.total - amount,
            locked: true,
            ..*self
        }
    }
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{},{},{},{}", self.client_id, self.available, self.held,
               self.total, self.locked)
    }
}
