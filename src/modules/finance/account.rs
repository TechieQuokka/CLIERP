use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::result::CLIERPResult;
use crate::core::error::CLIERPError;
use crate::database::models::{Account, NewAccount, AccountType};
use crate::database::schema::accounts;

pub struct AccountService;

impl AccountService {
    pub fn new() -> Self {
        Self
    }

    /// Create a new account
    pub fn create_account(&self, conn: &mut SqliteConnection, request: CreateAccountRequest) -> CLIERPResult<Account> {
        // Validate account code uniqueness
        let existing = accounts::table
            .filter(accounts::account_code.eq(&request.account_code))
            .first::<Account>(conn)
            .optional()?;

        if existing.is_some() {
            return Err(CLIERPError::ValidationError(
                format!("Account code '{}' already exists", request.account_code)
            ));
        }

        // Validate parent account exists if specified
        if let Some(parent_id) = request.parent_id {
            let parent_exists = accounts::table
                .find(parent_id)
                .first::<Account>(conn)
                .optional()?;

            if parent_exists.is_none() {
                return Err(CLIERPError::ValidationError(
                    format!("Parent account with ID {} not found", parent_id)
                ));
            }
        }

        let new_account = NewAccount {
            account_code: request.account_code,
            account_name: request.account_name,
            account_type: request.account_type,
            parent_id: request.parent_id,
            balance: 0, // New accounts start with zero balance
            is_active: true,
        };

        let account = diesel::insert_into(accounts::table)
            .values(&new_account)
            .get_result::<Account>(conn)?;

        Ok(account)
    }

    /// Get account by ID
    pub fn get_account_by_id(&self, conn: &mut SqliteConnection, account_id: i32) -> CLIERPResult<Option<Account>> {
        let account = accounts::table
            .find(account_id)
            .first::<Account>(conn)
            .optional()?;

        Ok(account)
    }

    /// Get account by code
    pub fn get_account_by_code(&self, conn: &mut SqliteConnection, account_code: &str) -> CLIERPResult<Option<Account>> {
        let account = accounts::table
            .filter(accounts::account_code.eq(account_code))
            .first::<Account>(conn)
            .optional()?;

        Ok(account)
    }

    /// List all accounts
    pub fn list_accounts(&self, conn: &mut SqliteConnection) -> CLIERPResult<Vec<Account>> {
        let accounts = accounts::table
            .filter(accounts::is_active.eq(true))
            .order(accounts::account_code.asc())
            .load::<Account>(conn)?;

        Ok(accounts)
    }

    /// List accounts by type
    pub fn list_accounts_by_type(&self, conn: &mut SqliteConnection, account_type: &str) -> CLIERPResult<Vec<Account>> {
        let accounts = accounts::table
            .filter(accounts::account_type.eq(account_type))
            .filter(accounts::is_active.eq(true))
            .order(accounts::account_code.asc())
            .load::<Account>(conn)?;

        Ok(accounts)
    }

    /// Update account
    pub fn update_account(&self, conn: &mut SqliteConnection, request: UpdateAccountRequest) -> CLIERPResult<Account> {
        // Check if account exists
        let existing = accounts::table
            .find(request.id)
            .first::<Account>(conn)
            .optional()?
            .ok_or_else(|| CLIERPError::NotFound("Account not found".to_string()))?;

        // Validate account code uniqueness if changed
        if let Some(ref new_code) = request.account_code {
            if new_code != &existing.account_code {
                let code_exists = accounts::table
                    .filter(accounts::account_code.eq(new_code))
                    .filter(accounts::id.ne(request.id))
                    .first::<Account>(conn)
                    .optional()?;

                if code_exists.is_some() {
                    return Err(CLIERPError::ValidationError(
                        format!("Account code '{}' already exists", new_code)
                    ));
                }
            }
        }

        // Build update query dynamically
        let account = diesel::update(accounts::table)
            .filter(accounts::id.eq(request.id))
            .set((
                accounts::account_code.eq(request.account_code.unwrap_or(existing.account_code)),
                accounts::account_name.eq(request.account_name.unwrap_or(existing.account_name)),
                accounts::account_type.eq(request.account_type.unwrap_or(existing.account_type)),
                accounts::parent_id.eq(request.parent_id.or(existing.parent_id)),
            ))
            .get_result::<Account>(conn)?;

        Ok(account)
    }

    /// Deactivate account (soft delete)
    pub fn deactivate_account(&self, conn: &mut SqliteConnection, account_id: i32) -> CLIERPResult<()> {
        let rows_affected = diesel::update(accounts::table)
            .filter(accounts::id.eq(account_id))
            .set(accounts::is_active.eq(false))
            .execute(conn)?;

        if rows_affected == 0 {
            return Err(CLIERPError::NotFound("Account not found".to_string()));
        }

        Ok(())
    }

    /// Get account balance
    pub fn get_account_balance(&self, conn: &mut SqliteConnection, account_id: i32) -> CLIERPResult<i32> {
        let account = accounts::table
            .find(account_id)
            .first::<Account>(conn)
            .optional()?
            .ok_or_else(|| CLIERPError::NotFound("Account not found".to_string()))?;

        Ok(account.balance)
    }

    /// Update account balance
    pub fn update_account_balance(&self, conn: &mut SqliteConnection, account_id: i32, new_balance: i32) -> CLIERPResult<Account> {
        let account = diesel::update(accounts::table)
            .filter(accounts::id.eq(account_id))
            .set(accounts::balance.eq(new_balance))
            .get_result::<Account>(conn)?;

        Ok(account)
    }

    /// Get chart of accounts (hierarchical structure)
    pub fn get_chart_of_accounts(&self, conn: &mut SqliteConnection) -> CLIERPResult<Vec<AccountNode>> {
        let all_accounts = self.list_accounts(conn)?;
        let mut account_map = std::collections::HashMap::new();
        let mut root_accounts = Vec::new();

        // Create nodes for all accounts
        for account in all_accounts {
            let node = AccountNode {
                account: account.clone(),
                children: Vec::new(),
            };
            account_map.insert(account.id, node);
        }

        // Build hierarchy
        let mut accounts_copy = account_map.clone();
        for (account_id, node) in account_map.iter() {
            if let Some(parent_id) = node.account.parent_id {
                if let Some(parent_node) = accounts_copy.get_mut(&parent_id) {
                    parent_node.children.push(node.clone());
                }
            } else {
                root_accounts.push(node.clone());
            }
        }

        // Sort by account code
        root_accounts.sort_by(|a, b| a.account.account_code.cmp(&b.account.account_code));

        Ok(root_accounts)
    }

    /// Get trial balance
    pub fn get_trial_balance(&self, conn: &mut SqliteConnection) -> CLIERPResult<TrialBalance> {
        let accounts = self.list_accounts(conn)?;

        let mut assets = 0;
        let mut liabilities = 0;
        let mut equity = 0;
        let mut revenue = 0;
        let mut expenses = 0;

        for account in &accounts {
            match account.account_type.as_str() {
                "asset" => assets += account.balance,
                "liability" => liabilities += account.balance,
                "equity" => equity += account.balance,
                "revenue" => revenue += account.balance,
                "expense" => expenses += account.balance,
                _ => {}
            }
        }

        let total_debits = assets + expenses;
        let total_credits = liabilities + equity + revenue;

        Ok(TrialBalance {
            accounts,
            total_assets: assets,
            total_liabilities: liabilities,
            total_equity: equity,
            total_revenue: revenue,
            total_expenses: expenses,
            total_debits,
            total_credits,
            is_balanced: total_debits == total_credits,
        })
    }
}

impl Default for AccountService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountNode {
    pub account: Account,
    pub children: Vec<AccountNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialBalance {
    pub accounts: Vec<Account>,
    pub total_assets: i32,
    pub total_liabilities: i32,
    pub total_equity: i32,
    pub total_revenue: i32,
    pub total_expenses: i32,
    pub total_debits: i32,
    pub total_credits: i32,
    pub is_balanced: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAccountRequest {
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    pub parent_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAccountRequest {
    pub id: i32,
    pub account_code: Option<String>,
    pub account_name: Option<String>,
    pub account_type: Option<String>,
    pub parent_id: Option<i32>,
}