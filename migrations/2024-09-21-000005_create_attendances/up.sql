-- Create attendances table
CREATE TABLE attendances (
    id INTEGER PRIMARY KEY NOT NULL,
    employee_id INTEGER NOT NULL,
    date DATE NOT NULL,
    check_in TIME,
    check_out TIME,
    break_time INTEGER DEFAULT 0, -- minutes
    overtime_hours REAL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'present' CHECK (status IN ('present', 'absent', 'late', 'early_leave', 'holiday')),
    notes TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (employee_id) REFERENCES employees (id),
    UNIQUE(employee_id, date)
);

-- Create index for faster queries
CREATE INDEX idx_attendances_employee_date ON attendances(employee_id, date);
CREATE INDEX idx_attendances_date ON attendances(date);
CREATE INDEX idx_attendances_status ON attendances(status);