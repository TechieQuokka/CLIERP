-- Create accounts table for chart of accounts
CREATE TABLE accounts (
    id INTEGER PRIMARY KEY NOT NULL,
    account_code TEXT NOT NULL UNIQUE,
    account_name TEXT NOT NULL,
    account_type TEXT NOT NULL CHECK (account_type IN ('asset', 'liability', 'equity', 'revenue', 'expense')),
    parent_id INTEGER,
    balance INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (parent_id) REFERENCES accounts (id)
);

-- Create index for performance
CREATE INDEX idx_accounts_type_active ON accounts(account_type, is_active);
CREATE INDEX idx_accounts_parent ON accounts(parent_id);
CREATE INDEX idx_accounts_code ON accounts(account_code);