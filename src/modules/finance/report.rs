use diesel::prelude::*;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::core::result::CLIERPResult;
use crate::core::error::CLIERPError;
use crate::database::models::{Account, Transaction};
use crate::database::schema::{accounts, transactions};
use super::account::AccountService;
use super::transaction::TransactionService;

pub struct ReportService;

impl ReportService {
    pub fn new() -> Self {
        Self
    }

    /// Generate Income Statement (Profit & Loss)
    pub fn generate_income_statement(&self, conn: &mut SqliteConnection, from_date: NaiveDate, to_date: NaiveDate) -> CLIERPResult<IncomeStatement> {
        let account_service = AccountService::new();
        let transaction_service = TransactionService::new();

        // Get revenue accounts
        let revenue_accounts = account_service.list_accounts_by_type(conn, "revenue")?;
        let mut revenue_items = Vec::new();
        let mut total_revenue = 0;

        for account in revenue_accounts {
            let transactions = transaction_service.get_account_transactions(conn, account.id, Some(from_date), Some(to_date))?;
            let period_balance = transactions.iter()
                .map(|t| match t.debit_credit.as_str() {
                    "credit" => t.amount,
                    "debit" => -t.amount,
                    _ => 0,
                })
                .sum::<i32>();

            if period_balance != 0 {
                revenue_items.push(IncomeStatementItem {
                    account_code: account.account_code,
                    account_name: account.account_name,
                    amount: period_balance,
                });
                total_revenue += period_balance;
            }
        }

        // Get expense accounts
        let expense_accounts = account_service.list_accounts_by_type(conn, "expense")?;
        let mut expense_items = Vec::new();
        let mut total_expenses = 0;

        for account in expense_accounts {
            let transactions = transaction_service.get_account_transactions(conn, account.id, Some(from_date), Some(to_date))?;
            let period_balance = transactions.iter()
                .map(|t| match t.debit_credit.as_str() {
                    "debit" => t.amount,
                    "credit" => -t.amount,
                    _ => 0,
                })
                .sum::<i32>();

            if period_balance != 0 {
                expense_items.push(IncomeStatementItem {
                    account_code: account.account_code,
                    account_name: account.account_name,
                    amount: period_balance,
                });
                total_expenses += period_balance;
            }
        }

        let net_income = total_revenue - total_expenses;

        Ok(IncomeStatement {
            from_date,
            to_date,
            revenue_items,
            total_revenue,
            expense_items,
            total_expenses,
            net_income,
        })
    }

    /// Generate Balance Sheet
    pub fn generate_balance_sheet(&self, conn: &mut SqliteConnection, as_of_date: NaiveDate) -> CLIERPResult<BalanceSheet> {
        let account_service = AccountService::new();
        let transaction_service = TransactionService::new();

        // Get asset accounts
        let asset_accounts = account_service.list_accounts_by_type(conn, "asset")?;
        let mut asset_items = Vec::new();
        let mut total_assets = 0;

        for account in asset_accounts {
            let transactions = transaction_service.get_account_transactions(conn, account.id, None, Some(as_of_date))?;
            let balance = transactions.iter()
                .map(|t| match t.debit_credit.as_str() {
                    "debit" => t.amount,
                    "credit" => -t.amount,
                    _ => 0,
                })
                .sum::<i32>();

            if balance != 0 {
                asset_items.push(BalanceSheetItem {
                    account_code: account.account_code,
                    account_name: account.account_name,
                    amount: balance,
                });
                total_assets += balance;
            }
        }

        // Get liability accounts
        let liability_accounts = account_service.list_accounts_by_type(conn, "liability")?;
        let mut liability_items = Vec::new();
        let mut total_liabilities = 0;

        for account in liability_accounts {
            let transactions = transaction_service.get_account_transactions(conn, account.id, None, Some(as_of_date))?;
            let balance = transactions.iter()
                .map(|t| match t.debit_credit.as_str() {
                    "credit" => t.amount,
                    "debit" => -t.amount,
                    _ => 0,
                })
                .sum::<i32>();

            if balance != 0 {
                liability_items.push(BalanceSheetItem {
                    account_code: account.account_code,
                    account_name: account.account_name,
                    amount: balance,
                });
                total_liabilities += balance;
            }
        }

        // Get equity accounts
        let equity_accounts = account_service.list_accounts_by_type(conn, "equity")?;
        let mut equity_items = Vec::new();
        let mut total_equity = 0;

        for account in equity_accounts {
            let transactions = transaction_service.get_account_transactions(conn, account.id, None, Some(as_of_date))?;
            let balance = transactions.iter()
                .map(|t| match t.debit_credit.as_str() {
                    "credit" => t.amount,
                    "debit" => -t.amount,
                    _ => 0,
                })
                .sum::<i32>();

            if balance != 0 {
                equity_items.push(BalanceSheetItem {
                    account_code: account.account_code,
                    account_name: account.account_name,
                    amount: balance,
                });
                total_equity += balance;
            }
        }

        let total_liabilities_and_equity = total_liabilities + total_equity;
        let is_balanced = total_assets == total_liabilities_and_equity;

        Ok(BalanceSheet {
            as_of_date,
            asset_items,
            total_assets,
            liability_items,
            total_liabilities,
            equity_items,
            total_equity,
            total_liabilities_and_equity,
            is_balanced,
        })
    }

    /// Generate Trial Balance
    pub fn generate_trial_balance(&self, conn: &mut SqliteConnection, as_of_date: NaiveDate) -> CLIERPResult<TrialBalanceReport> {
        let account_service = AccountService::new();
        let transaction_service = TransactionService::new();

        let all_accounts = account_service.list_accounts(conn)?;
        let mut trial_balance_items = Vec::new();
        let mut total_debits = 0;
        let mut total_credits = 0;

        for account in all_accounts {
            let transactions = transaction_service.get_account_transactions(conn, account.id, None, Some(as_of_date))?;

            let debit_balance = transactions.iter()
                .filter(|t| t.debit_credit == "debit")
                .map(|t| t.amount)
                .sum::<i32>();

            let credit_balance = transactions.iter()
                .filter(|t| t.debit_credit == "credit")
                .map(|t| t.amount)
                .sum::<i32>();

            let net_balance = debit_balance - credit_balance;

            if net_balance != 0 {
                let (debit_amount, credit_amount) = if net_balance > 0 {
                    (net_balance, 0)
                } else {
                    (0, -net_balance)
                };

                trial_balance_items.push(TrialBalanceItem {
                    account_code: account.account_code,
                    account_name: account.account_name,
                    account_type: account.account_type,
                    debit_amount,
                    credit_amount,
                });

                total_debits += debit_amount;
                total_credits += credit_amount;
            }
        }

        let is_balanced = total_debits == total_credits;

        Ok(TrialBalanceReport {
            as_of_date,
            items: trial_balance_items,
            total_debits,
            total_credits,
            difference: total_debits - total_credits,
            is_balanced,
        })
    }

    /// Generate General Ledger Report
    pub fn generate_general_ledger_report(&self, conn: &mut SqliteConnection, account_id: Option<i32>, from_date: Option<NaiveDate>, to_date: Option<NaiveDate>) -> CLIERPResult<GeneralLedgerReport> {
        let account_service = AccountService::new();
        let transaction_service = TransactionService::new();

        let accounts = if let Some(account_id) = account_id {
            vec![account_service.get_account_by_id(conn, account_id)?
                .ok_or_else(|| CLIERPError::NotFound("Account not found".to_string()))?]
        } else {
            account_service.list_accounts(conn)?
        };

        let mut ledger_accounts = Vec::new();

        for account in accounts {
            let general_ledger = transaction_service.get_general_ledger(conn, account.id, from_date, to_date)?;
            if !general_ledger.entries.is_empty() {
                ledger_accounts.push(general_ledger);
            }
        }

        Ok(GeneralLedgerReport {
            from_date,
            to_date,
            accounts: ledger_accounts,
        })
    }

    /// Generate Cash Flow Statement (simplified)
    pub fn generate_cash_flow_statement(&self, conn: &mut SqliteConnection, from_date: NaiveDate, to_date: NaiveDate) -> CLIERPResult<CashFlowStatement> {
        let transaction_service = TransactionService::new();

        // Find cash accounts (assuming account codes starting with "1000" are cash)
        let cash_accounts: Vec<Account> = accounts::table
            .filter(accounts::account_type.eq("asset"))
            .filter(accounts::account_code.like("1000%"))
            .filter(accounts::is_active.eq(true))
            .load::<Account>(conn)?;

        let mut cash_inflows = 0;
        let mut cash_outflows = 0;
        let mut cash_flow_items = Vec::new();

        for account in cash_accounts {
            let transactions = transaction_service.get_account_transactions(conn, account.id, Some(from_date), Some(to_date))?;

            for transaction in transactions {
                let item = CashFlowItem {
                    date: transaction.transaction_date,
                    description: transaction.description.clone(),
                    account_name: account.account_name.clone(),
                    amount: if transaction.debit_credit == "debit" { transaction.amount } else { -transaction.amount },
                };

                if item.amount > 0 {
                    cash_inflows += item.amount;
                } else {
                    cash_outflows += -item.amount;
                }

                cash_flow_items.push(item);
            }
        }

        let net_cash_flow = cash_inflows - cash_outflows;

        Ok(CashFlowStatement {
            from_date,
            to_date,
            cash_inflows,
            cash_outflows,
            net_cash_flow,
            items: cash_flow_items,
        })
    }
}

impl Default for ReportService {
    fn default() -> Self {
        Self::new()
    }
}

// Report Data Structures

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomeStatement {
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    pub revenue_items: Vec<IncomeStatementItem>,
    pub total_revenue: i32,
    pub expense_items: Vec<IncomeStatementItem>,
    pub total_expenses: i32,
    pub net_income: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomeStatementItem {
    pub account_code: String,
    pub account_name: String,
    pub amount: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSheet {
    pub as_of_date: NaiveDate,
    pub asset_items: Vec<BalanceSheetItem>,
    pub total_assets: i32,
    pub liability_items: Vec<BalanceSheetItem>,
    pub total_liabilities: i32,
    pub equity_items: Vec<BalanceSheetItem>,
    pub total_equity: i32,
    pub total_liabilities_and_equity: i32,
    pub is_balanced: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSheetItem {
    pub account_code: String,
    pub account_name: String,
    pub amount: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialBalanceReport {
    pub as_of_date: NaiveDate,
    pub items: Vec<TrialBalanceItem>,
    pub total_debits: i32,
    pub total_credits: i32,
    pub difference: i32,
    pub is_balanced: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialBalanceItem {
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    pub debit_amount: i32,
    pub credit_amount: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralLedgerReport {
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
    pub accounts: Vec<super::transaction::GeneralLedger>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashFlowStatement {
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    pub cash_inflows: i32,
    pub cash_outflows: i32,
    pub net_cash_flow: i32,
    pub items: Vec<CashFlowItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashFlowItem {
    pub date: NaiveDate,
    pub description: String,
    pub account_name: String,
    pub amount: i32,
}