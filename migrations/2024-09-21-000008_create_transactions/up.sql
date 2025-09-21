-- Create transactions table for accounting entries
CREATE TABLE transactions (
    id INTEGER PRIMARY KEY NOT NULL,
    account_id INTEGER NOT NULL,
    transaction_date DATE NOT NULL,
    amount INTEGER NOT NULL,
    debit_credit TEXT NOT NULL CHECK (debit_credit IN ('debit', 'credit')),
    description TEXT NOT NULL,
    reference TEXT,
    created_by INTEGER,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (account_id) REFERENCES accounts (id),
    FOREIGN KEY (created_by) REFERENCES users (id)
);

-- Create indices for performance
CREATE INDEX idx_transactions_account_id ON transactions(account_id);
CREATE INDEX idx_transactions_date ON transactions(transaction_date);
CREATE INDEX idx_transactions_created_by ON transactions(created_by);