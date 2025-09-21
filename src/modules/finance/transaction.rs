use chrono::{Local, NaiveDate};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use super::account::AccountService;
use crate::core::error::CLIERPError;
use crate::core::result::CLIERPResult;
use crate::database::models::{Account, NewTransaction, Transaction};
use crate::database::schema::{accounts, transactions};

pub struct TransactionService;

impl TransactionService {
    pub fn new() -> Self {
        Self
    }

    /// Create a new transaction
    pub fn create_transaction(
        &self,
        conn: &mut SqliteConnection,
        request: CreateTransactionRequest,
        created_by: Option<i32>,
    ) -> CLIERPResult<Transaction> {
        // Validate account exists
        let account = accounts::table
            .find(request.account_id)
            .first::<Account>(conn)
            .optional()?
            .ok_or_else(|| CLIERPError::NotFound("Account not found".to_string()))?;

        if !account.is_active {
            return Err(CLIERPError::ValidationError(
                "Cannot post to inactive account".to_string(),
            ));
        }

        // Validate amount is not zero
        if request.amount == 0 {
            return Err(CLIERPError::ValidationError(
                "Transaction amount cannot be zero".to_string(),
            ));
        }

        // Validate debit/credit
        if request.debit_credit != "debit" && request.debit_credit != "credit" {
            return Err(CLIERPError::ValidationError(
                "Transaction must be either 'debit' or 'credit'".to_string(),
            ));
        }

        let new_transaction = NewTransaction {
            account_id: request.account_id,
            transaction_date: request.transaction_date,
            amount: request.amount,
            debit_credit: request.debit_credit.clone(),
            description: request.description,
            reference: request.reference,
            created_by,
        };

        diesel::insert_into(transactions::table)
            .values(&new_transaction)
            .execute(conn)?;

        let transaction = transactions::table
            .filter(transactions::account_id.eq(request.account_id))
            .filter(transactions::transaction_date.eq(request.transaction_date))
            .filter(transactions::amount.eq(request.amount))
            .order(transactions::id.desc())
            .first::<Transaction>(conn)?;

        // Update account balance
        self.update_account_balance(conn, &account, &transaction)?;

        Ok(transaction)
    }

    /// Get transaction by ID
    pub fn get_transaction_by_id(
        &self,
        conn: &mut SqliteConnection,
        transaction_id: i32,
    ) -> CLIERPResult<Option<TransactionWithAccount>> {
        let result = transactions::table
            .inner_join(accounts::table)
            .filter(transactions::id.eq(transaction_id))
            .select((Transaction::as_select(), Account::as_select()))
            .first::<(Transaction, Account)>(conn)
            .optional()?;

        Ok(result.map(|(transaction, account)| TransactionWithAccount {
            transaction,
            account,
        }))
    }

    /// List transactions with filters
    pub fn list_transactions(
        &self,
        conn: &mut SqliteConnection,
        filters: TransactionFilters,
    ) -> CLIERPResult<Vec<TransactionWithAccount>> {
        let mut query = transactions::table.inner_join(accounts::table).into_boxed();

        if let Some(account_id) = filters.account_id {
            query = query.filter(transactions::account_id.eq(account_id));
        }

        if let Some(account_type) = filters.account_type {
            query = query.filter(accounts::account_type.eq(account_type));
        }

        if let Some(from_date) = filters.from_date {
            query = query.filter(transactions::transaction_date.ge(from_date));
        }

        if let Some(to_date) = filters.to_date {
            query = query.filter(transactions::transaction_date.le(to_date));
        }

        if let Some(debit_credit) = filters.debit_credit {
            query = query.filter(transactions::debit_credit.eq(debit_credit));
        }

        let results = query
            .select((Transaction::as_select(), Account::as_select()))
            .order(transactions::transaction_date.desc())
            .load::<(Transaction, Account)>(conn)?;

        Ok(results
            .into_iter()
            .map(|(transaction, account)| TransactionWithAccount {
                transaction,
                account,
            })
            .collect())
    }

    /// Get account transactions
    pub fn get_account_transactions(
        &self,
        conn: &mut SqliteConnection,
        account_id: i32,
        from_date: Option<NaiveDate>,
        to_date: Option<NaiveDate>,
    ) -> CLIERPResult<Vec<Transaction>> {
        let mut query = transactions::table
            .filter(transactions::account_id.eq(account_id))
            .into_boxed();

        if let Some(from) = from_date {
            query = query.filter(transactions::transaction_date.ge(from));
        }

        if let Some(to) = to_date {
            query = query.filter(transactions::transaction_date.le(to));
        }

        let transactions = query
            .order(transactions::transaction_date.desc())
            .load::<Transaction>(conn)?;

        Ok(transactions)
    }

    /// Get general ledger for an account
    pub fn get_general_ledger(
        &self,
        conn: &mut SqliteConnection,
        account_id: i32,
        from_date: Option<NaiveDate>,
        to_date: Option<NaiveDate>,
    ) -> CLIERPResult<GeneralLedger> {
        let account = accounts::table
            .find(account_id)
            .first::<Account>(conn)
            .optional()?
            .ok_or_else(|| CLIERPError::NotFound("Account not found".to_string()))?;

        let transactions = self.get_account_transactions(conn, account_id, from_date, to_date)?;

        // Calculate running balance
        let mut ledger_entries = Vec::new();
        let mut running_balance = 0;

        // Get opening balance (transactions before from_date)
        if let Some(from) = from_date {
            let opening_transactions = transactions::table
                .filter(transactions::account_id.eq(account_id))
                .filter(transactions::transaction_date.lt(from))
                .load::<Transaction>(conn)?;

            for trans in opening_transactions {
                match trans.debit_credit.as_str() {
                    "debit" => running_balance += trans.amount,
                    "credit" => running_balance -= trans.amount,
                    _ => {}
                }
            }
        }

        let opening_balance = running_balance;

        for transaction in transactions {
            match transaction.debit_credit.as_str() {
                "debit" => running_balance += transaction.amount,
                "credit" => running_balance -= transaction.amount,
                _ => {}
            }

            ledger_entries.push(GeneralLedgerEntry {
                transaction,
                running_balance,
            });
        }

        Ok(GeneralLedger {
            account,
            opening_balance,
            closing_balance: running_balance,
            entries: ledger_entries,
        })
    }

    /// Update account balance after transaction
    fn update_account_balance(
        &self,
        conn: &mut SqliteConnection,
        account: &Account,
        transaction: &Transaction,
    ) -> CLIERPResult<()> {
        let account_service = AccountService::new();

        let new_balance = match transaction.debit_credit.as_str() {
            "debit" => account.balance + transaction.amount,
            "credit" => account.balance - transaction.amount,
            _ => account.balance,
        };

        account_service.update_account_balance(conn, account.id, new_balance)?;
        Ok(())
    }

    /// Reverse a transaction
    pub fn reverse_transaction(
        &self,
        conn: &mut SqliteConnection,
        transaction_id: i32,
        reason: String,
        created_by: Option<i32>,
    ) -> CLIERPResult<Transaction> {
        let original_transaction = transactions::table
            .find(transaction_id)
            .first::<Transaction>(conn)
            .optional()?
            .ok_or_else(|| CLIERPError::NotFound("Transaction not found".to_string()))?;

        // Create reverse transaction
        let reverse_debit_credit = match original_transaction.debit_credit.as_str() {
            "debit" => "credit",
            "credit" => "debit",
            _ => {
                return Err(CLIERPError::ValidationError(
                    "Invalid original transaction type".to_string(),
                ))
            }
        };

        let reverse_transaction_request = CreateTransactionRequest {
            account_id: original_transaction.account_id,
            transaction_date: Local::now().date_naive(),
            amount: original_transaction.amount,
            debit_credit: reverse_debit_credit.to_string(),
            description: format!(
                "REVERSAL: {} - {}",
                original_transaction.description, reason
            ),
            reference: Some(format!("REV-{}", original_transaction.id)),
        };

        self.create_transaction(conn, reverse_transaction_request, created_by)
    }

    /// Get transaction summary for a period
    pub fn get_transaction_summary(
        &self,
        conn: &mut SqliteConnection,
        from_date: NaiveDate,
        to_date: NaiveDate,
    ) -> CLIERPResult<TransactionSummary> {
        let transactions = transactions::table
            .filter(transactions::transaction_date.ge(from_date))
            .filter(transactions::transaction_date.le(to_date))
            .load::<Transaction>(conn)?;

        let total_transactions = transactions.len() as i32;
        let total_debits = transactions
            .iter()
            .filter(|t| t.debit_credit == "debit")
            .map(|t| t.amount)
            .sum::<i32>();
        let total_credits = transactions
            .iter()
            .filter(|t| t.debit_credit == "credit")
            .map(|t| t.amount)
            .sum::<i32>();

        Ok(TransactionSummary {
            from_date,
            to_date,
            total_transactions,
            total_debits,
            total_credits,
            net_amount: total_debits - total_credits,
        })
    }
}

impl Default for TransactionService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionWithAccount {
    pub transaction: Transaction,
    pub account: Account,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralLedger {
    pub account: Account,
    pub opening_balance: i32,
    pub closing_balance: i32,
    pub entries: Vec<GeneralLedgerEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralLedgerEntry {
    pub transaction: Transaction,
    pub running_balance: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSummary {
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    pub total_transactions: i32,
    pub total_debits: i32,
    pub total_credits: i32,
    pub net_amount: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTransactionRequest {
    pub account_id: i32,
    pub transaction_date: NaiveDate,
    pub amount: i32,
    pub debit_credit: String,
    pub description: String,
    pub reference: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TransactionFilters {
    pub account_id: Option<i32>,
    pub account_type: Option<String>,
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
    pub debit_credit: Option<String>,
}
