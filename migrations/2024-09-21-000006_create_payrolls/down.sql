-- Drop indices first
DROP INDEX IF EXISTS idx_payrolls_status;
DROP INDEX IF EXISTS idx_payrolls_period;
DROP INDEX IF EXISTS idx_payrolls_employee_period;

-- Drop table
DROP TABLE IF EXISTS payrolls;