-- Create product_attachments table for images and files
CREATE TABLE product_attachments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    product_id INTEGER NOT NULL,
    attachment_type TEXT NOT NULL CHECK (attachment_type IN ('image', 'document', 'manual', 'certificate')),
    file_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    mime_type TEXT,
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (product_id) REFERENCES products (id) ON DELETE CASCADE
);

-- Create index for efficient queries
CREATE INDEX idx_product_attachments_product_id ON product_attachments(product_id);
CREATE INDEX idx_product_attachments_type ON product_attachments(attachment_type);
CREATE INDEX idx_product_attachments_primary ON product_attachments(is_primary);

-- Create stock_audits table for stock audit functionality
CREATE TABLE stock_audits (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    audit_name TEXT NOT NULL,
    audit_date DATE NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'in_progress', 'completed', 'cancelled')),
    conducted_by INTEGER,
    notes TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (conducted_by) REFERENCES users (id)
);

-- Create stock_audit_items table for individual audit records
CREATE TABLE stock_audit_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    audit_id INTEGER NOT NULL,
    product_id INTEGER NOT NULL,
    expected_quantity INTEGER NOT NULL,
    actual_quantity INTEGER,
    variance INTEGER,
    notes TEXT,
    audited_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (audit_id) REFERENCES stock_audits (id) ON DELETE CASCADE,
    FOREIGN KEY (product_id) REFERENCES products (id),
    UNIQUE(audit_id, product_id)
);

-- Create indexes for stock audit tables
CREATE INDEX idx_stock_audits_date ON stock_audits(audit_date);
CREATE INDEX idx_stock_audits_status ON stock_audits(status);
CREATE INDEX idx_stock_audit_items_audit_id ON stock_audit_items(audit_id);
CREATE INDEX idx_stock_audit_items_product_id ON stock_audit_items(product_id);