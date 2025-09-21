-- Drop indices first
DROP INDEX IF EXISTS idx_accounts_code;
DROP INDEX IF EXISTS idx_accounts_parent;
DROP INDEX IF EXISTS idx_accounts_type_active;

-- Drop table
DROP TABLE IF EXISTS accounts;