-- Drop indices first
DROP INDEX IF EXISTS idx_transactions_created_by;
DROP INDEX IF EXISTS idx_transactions_date;
DROP INDEX IF EXISTS idx_transactions_account_id;

-- Drop table
DROP TABLE IF EXISTS transactions;