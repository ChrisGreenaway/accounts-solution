use crate::account::ClientAccount;
use crate::transaction::{Transaction, TransactionType};
use std::collections::{HashMap, HashSet};

pub struct TransactionProcessor {
    disputed_transaction_ids: HashSet<u32>,
    disputed_transactions: HashMap<u32, Transaction>,
    accounts: HashMap<u16, ClientAccount>,
}

impl TransactionProcessor {
    pub fn new() -> Self {
        Self {
            disputed_transaction_ids: HashSet::new(),
            disputed_transactions: HashMap::new(),
            accounts: HashMap::new(),
        }
    }

    pub fn preprocess(&mut self, transaction: &Transaction) {
        if transaction.r#type == TransactionType::Dispute {
            self.disputed_transaction_ids.insert(transaction.tx);
        }
    }

    pub fn process(&mut self, transaction: &Transaction) {
        match self.accounts.get_mut(&transaction.client) {
            None => {
                let mut account = ClientAccount::new(transaction.client);
                account.combine(transaction, &self.disputed_transactions);
                self.accounts.insert(transaction.client, account);
            }
            Some(account) => account.combine(transaction, &self.disputed_transactions),
        }

        if is_disputable(transaction) && self.disputed_transaction_ids.contains(&transaction.tx) {
            self.disputed_transactions
                .insert(transaction.tx, *transaction);
        }
    }

    pub fn into_accounts(self) -> HashMap<u16, ClientAccount> {
        self.accounts
    }
}

fn is_disputable(transaction: &Transaction) -> bool {
    transaction.r#type == TransactionType::Deposit
        || transaction.r#type == TransactionType::Withdrawal
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{Transaction, TransactionType};
    use rust_decimal::prelude::FromStr;
    use rust_decimal::Decimal;
    use std::collections::HashMap;

    #[test]
    fn one_deposit() {
        let mut test = TransactionTest::new();
        test.deposit(1, 1, "2.0");
        test.expected(1, "2", "0", "2", false);
        test.run();
    }

    #[test]
    fn one_withdrawal() {
        let mut test = TransactionTest::new();
        test.withdrawl(1, 1, "2.0");
        test.expected(1, "-2", "0", "-2", false);
        test.run();
    }

    #[test]
    fn one_dispute() {
        let mut test = TransactionTest::new();
        test.deposit(1, 1, "2.0");
        test.dispute(1, 1);
        test.expected(1, "0", "2", "2", false);
        test.run();
    }

    #[test]
    fn one_resolve() {
        let mut test = TransactionTest::new();
        test.deposit(1, 1, "2.0");
        test.dispute(1, 1);
        test.resolve(1, 1);
        test.expected(1, "2", "0", "2", false);
        test.run();
    }

    #[test]
    fn one_chargeback() {
        let mut test = TransactionTest::new();
        test.deposit(1, 1, "2.0");
        test.dispute(1, 1);
        test.chargeback(1, 1);
        test.expected(1, "0", "0", "0", true);
        test.run();
    }

    #[test]
    fn multiple_deposits_and_withdrawals() {
        let mut test = TransactionTest::new();
        test.deposit(1, 1, "10.0");
        test.withdrawl(1, 2, "20.0");
        test.deposit(1, 3, "30.0");
        test.deposit(1, 4, "40.0");
        test.withdrawl(1, 5, "50.0");
        test.deposit(1, 6, "60.0");
        test.expected(1, "70", "0", "70", false);
        test.run();
    }

    #[test]
    fn dispute_transaction_that_does_not_exist() {
        let mut test = TransactionTest::new();
        test.dispute(1, 1);
        test.expected(1, "0", "0", "0", false);
        test.run();
    }

    #[test]
    fn resolve_dispute_that_does_not_exist() {
        let mut test = TransactionTest::new();
        test.resolve(1, 1);
        test.expected(1, "0", "0", "0", false);
        test.run();
    }

    #[test]
    fn resolve_dispute_that_does_not_exist_but_deposit_does() {
        let mut test = TransactionTest::new();
        test.deposit(1, 1, "2.0");
        test.resolve(1, 1);
        test.expected(1, "2", "0", "2", false);
        test.run();
    }

    #[test]
    fn chargeback_dispute_that_does_not_exist() {
        let mut test = TransactionTest::new();
        test.chargeback(1, 1);
        test.expected(1, "0", "0", "0", false);
        test.run();
    }

    #[test]
    fn chargeback_dispute_that_does_not_exist_but_deposit_does() {
        let mut test = TransactionTest::new();
        test.deposit(1, 1, "2.0");
        test.chargeback(1, 1);
        test.expected(1, "2", "0", "2", false);
        test.run();
    }

    #[test]
    fn multiple_clients() {
        let mut test = TransactionTest::new();
        test.deposit(1, 1, "10.0");
        test.deposit(2, 2, "20.0");
        test.expected(1, "10", "0", "10", false);
        test.expected(2, "20", "0", "20", false);
        test.run();
    }

    #[test]
    fn multiple_clients_with_a_dispute() {
        let mut test = TransactionTest::new();
        test.deposit(1, 1, "10.0");
        test.deposit(2, 2, "20.0");
        test.dispute(1, 1);
        test.expected(1, "0", "10", "10", false);
        test.expected(2, "20", "0", "20", false);
        test.run();
    }

    #[test]
    fn multiple_clients_with_a_resolve() {
        let mut test = TransactionTest::new();
        test.deposit(1, 1, "10.0");
        test.deposit(2, 2, "20.0");
        test.dispute(1, 1);
        test.resolve(1, 1);
        test.expected(1, "10", "0", "10", false);
        test.expected(2, "20", "0", "20", false);
        test.run();
    }

    #[test]
    fn multiple_clients_with_a_chargeback() {
        let mut test = TransactionTest::new();
        test.deposit(1, 1, "10.0");
        test.deposit(2, 2, "20.0");
        test.dispute(1, 1);
        test.chargeback(1, 1);
        test.expected(1, "0", "0", "0", true);
        test.expected(2, "20", "0", "20", false);
        test.run();
    }

    #[test]
    fn multiple_clients_with_a_dispute_incorrect_client() {
        let mut test = TransactionTest::new();
        test.deposit(1, 1, "10.0");
        test.deposit(2, 2, "20.0");
        test.dispute(2, 1);
        test.expected(1, "10", "0", "10", false);
        test.expected(2, "20", "0", "20", false);
        test.run();
    }

    #[test]
    fn multiple_clients_with_a_resolve_incorrect_client() {
        let mut test = TransactionTest::new();
        test.deposit(1, 1, "10.0");
        test.deposit(2, 2, "20.0");
        test.dispute(1, 1);
        test.resolve(2, 1);
        test.expected(1, "0", "10", "10", false);
        test.expected(2, "20", "0", "20", false);
        test.run();
    }

    #[test]
    fn multiple_clients_with_a_chargeback_incorrect_client() {
        let mut test = TransactionTest::new();
        test.deposit(1, 1, "10.0");
        test.deposit(2, 2, "20.0");
        test.dispute(1, 1);
        test.chargeback(2, 1);
        test.expected(1, "0", "10", "10", false);
        test.expected(2, "20", "0", "20", false);
        test.run();
    }

    struct TransactionTest {
        input: Vec<Transaction>,
        expected_output: HashMap<u16, ClientAccount>,
    }

    impl TransactionTest {
        fn new() -> Self {
            Self {
                input: vec![],
                expected_output: HashMap::new(),
            }
        }

        fn deposit(&mut self, client: u16, tx: u32, amount: &str) {
            self.transaction_with_amount(TransactionType::Deposit, client, tx, amount)
        }

        fn withdrawl(&mut self, client: u16, tx: u32, amount: &str) {
            self.transaction_with_amount(TransactionType::Withdrawal, client, tx, amount)
        }

        fn dispute(&mut self, client: u16, tx: u32) {
            self.transaction_no_amount(TransactionType::Dispute, client, tx)
        }

        fn resolve(&mut self, client: u16, tx: u32) {
            self.transaction_no_amount(TransactionType::Resolve, client, tx)
        }

        fn chargeback(&mut self, client: u16, tx: u32) {
            self.transaction_no_amount(TransactionType::Chargeback, client, tx)
        }

        fn transaction_with_amount(
            &mut self,
            transaction_type: TransactionType,
            client: u16,
            tx: u32,
            amount: &str,
        ) {
            let amount = Some(Decimal::from_str(amount).unwrap());
            let transaction = Transaction {
                r#type: transaction_type,
                client,
                tx,
                amount,
            };
            self.input.push(transaction);
        }

        fn transaction_no_amount(
            &mut self,
            transaction_type: TransactionType,
            client: u16,
            tx: u32,
        ) {
            let transaction = Transaction {
                r#type: transaction_type,
                client,
                tx,
                amount: None,
            };
            self.input.push(transaction);
        }

        fn expected(
            &mut self,
            client: u16,
            available: &str,
            held: &str,
            total: &str,
            locked: bool,
        ) {
            let available = Decimal::from_str(available).unwrap();
            let held = Decimal::from_str(held).unwrap();
            let total = Decimal::from_str(total).unwrap();
            let client_account = ClientAccount {
                client,
                available,
                held,
                total,
                locked,
            };
            self.expected_output.insert(client, client_account);
        }

        fn run(self) {
            let mut processor = TransactionProcessor::new();

            for transaction in &self.input {
                processor.preprocess(transaction);
            }
            for transaction in &self.input {
                processor.process(transaction);
            }

            assert_eq!(processor.into_accounts(), self.expected_output);
        }
    }
}
