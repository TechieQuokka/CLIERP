-- Create payrolls table
CREATE TABLE payrolls (
    id INTEGER PRIMARY KEY NOT NULL,
    employee_id INTEGER NOT NULL,
    period TEXT NOT NULL, -- YYYY-MM format
    base_salary INTEGER NOT NULL,
    overtime_pay INTEGER DEFAULT 0,
    bonuses INTEGER DEFAULT 0,
    deductions INTEGER DEFAULT 0,
    net_salary INTEGER NOT NULL,
    payment_date DATE,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'processed', 'paid')),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (employee_id) REFERENCES employees (id),
    UNIQUE(employee_id, period)
);

-- Create indices for performance
CREATE INDEX idx_payrolls_employee_period ON payrolls(employee_id, period);
CREATE INDEX idx_payrolls_period ON payrolls(period);
CREATE INDEX idx_payrolls_status ON payrolls(status);