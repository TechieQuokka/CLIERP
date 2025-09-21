-- Drop purchase management tables in reverse order
DROP INDEX IF EXISTS idx_purchase_items_product_id;
DROP INDEX IF EXISTS idx_purchase_items_po_id;
DROP INDEX IF EXISTS idx_purchase_orders_date_status;
DROP INDEX IF EXISTS idx_purchase_orders_po_number;
DROP INDEX IF EXISTS idx_purchase_orders_supplier_id;
DROP INDEX IF EXISTS idx_suppliers_status;
DROP INDEX IF EXISTS idx_suppliers_code;

DROP TABLE IF EXISTS purchase_items;
DROP TABLE IF EXISTS purchase_orders;
DROP TABLE IF EXISTS suppliers;