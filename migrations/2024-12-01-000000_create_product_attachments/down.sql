-- Drop stock audit tables
DROP INDEX IF EXISTS idx_stock_audit_items_product_id;
DROP INDEX IF EXISTS idx_stock_audit_items_audit_id;
DROP INDEX IF EXISTS idx_stock_audits_status;
DROP INDEX IF EXISTS idx_stock_audits_date;

DROP TABLE IF EXISTS stock_audit_items;
DROP TABLE IF EXISTS stock_audits;

-- Drop product attachment tables
DROP INDEX IF EXISTS idx_product_attachments_primary;
DROP INDEX IF EXISTS idx_product_attachments_type;
DROP INDEX IF EXISTS idx_product_attachments_product_id;

DROP TABLE IF EXISTS product_attachments;