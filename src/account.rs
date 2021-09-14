use crate::transaction::{Transaction, TransactionType};
use rust_decimal::prelude::Zero;
use rust_decimal::Decimal;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize, Eq, PartialEq)]
pub struct ClientAccount {
    pub client: u16,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}

impl ClientAccount {
    pub fn new(client: u16) -> Self {
        Self {
            client,
            available: Decimal::zero(),
            held: Decimal::zero(),
            total: Decimal::zero(),
            locked: false,
        }
    }

    /// Panics if the client of the Transaction does not match the client of the account
    /// That would indicate a bug that should be fixed
    /// Crash to prevent incorrect processing and detect coding errors sooner
    pub fn combine(&mut self, transaction: &Transaction, transactions: &HashMap<u32, Transaction>) {
        assert_eq!(self.client, transaction.client);

        match transaction.r#type {
            TransactionType::Deposit => self.deposit(transaction.amount),
            TransactionType::Withdrawal => self.withdrawal(transaction.amount),
            TransactionType::Dispute => self.dispute(transaction, transactions),
            TransactionType::Resolve => self.resolve(transaction, transactions),
            TransactionType::Chargeback => self.chargeback(transaction, transactions),
        }

        assert_eq!(self.available + self.held, self.total);
    }

    fn deposit(&mut self, amount: Option<Decimal>) {
        // Ignore deposits with missing amounts
        if let Some(amount) = amount {
            self.available += amount;
            self.total += amount;
        }
    }

    fn withdrawal(&mut self, amount: Option<Decimal>) {
        // Ignore deposits with missing amounts
        if let Some(amount) = amount {
            self.available -= amount;
            self.total -= amount;
        }
    }

    fn dispute(&mut self, transaction: &Transaction, transactions: &HashMap<u32, Transaction>) {
        if let Some(amount) = referenced_transaction_amount(transaction, transactions) {
            self.available -= amount;
            self.held += amount;
        }
    }

    fn resolve(&mut self, transaction: &Transaction, transactions: &HashMap<u32, Transaction>) {
        if let Some(amount) = referenced_transaction_amount(transaction, transactions) {
            self.available += amount;
            self.held -= amount;
        }
    }

    fn chargeback(&mut self, transaction: &Transaction, transactions: &HashMap<u32, Transaction>) {
        if let Some(amount) = referenced_transaction_amount(transaction, transactions) {
            self.held -= amount;
            self.total -= amount;
            self.locked = true;
        }
    }
}

fn referenced_transaction_amount(
    transaction: &Transaction,
    transactions: &HashMap<u32, Transaction>,
) -> Option<Decimal> {
    // Ignore transactions that aren't disputed
    if let Some(referenced_transaction) = transactions.get(&transaction.tx) {
        // Ignore disputes that reference a transaction for another client
        if transaction.client == referenced_transaction.client {
            return referenced_transaction.amount;
        }
    }

    None
}
