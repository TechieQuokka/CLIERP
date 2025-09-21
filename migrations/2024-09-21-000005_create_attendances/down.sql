-- Drop indices first
DROP INDEX IF EXISTS idx_attendances_status;
DROP INDEX IF EXISTS idx_attendances_date;
DROP INDEX IF EXISTS idx_attendances_employee_date;

-- Drop table
DROP TABLE IF EXISTS attendances;