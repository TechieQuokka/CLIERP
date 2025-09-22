mod common;

use common::setup_test_db;
use clierp::modules::inventory::{ProductService, CategoryService, SupplierService, PurchaseOrderService};
use clierp::modules::inventory::{Product, Category, Supplier, PurchaseOrder, PurchaseOrderItem};
use clierp::utils::pagination::PaginationParams;
use chrono::NaiveDate;

#[test]
fn test_category_creation_and_retrieval() {
    setup_test_db();

    let category_service = CategoryService::new();

    // Test creating a new category
    let result = category_service.create_category(
        "Electronics",
        Some("Electronic devices and components"),
        None,
    );

    assert!(result.is_ok());
    let category = result.unwrap();
    assert_eq!(category.name, "Electronics");
    assert_eq!(category.description, Some("Electronic devices and components".to_string()));
    assert_eq!(category.parent_id, None);

    // Test retrieving the category
    let retrieved = category_service.get_category_by_id(category.id);
    assert!(retrieved.is_ok());
    let retrieved_category = retrieved.unwrap();
    assert_eq!(retrieved_category.name, category.name);
    assert_eq!(retrieved_category.id, category.id);
}

#[test]
fn test_category_hierarchy() {
    setup_test_db();

    let category_service = CategoryService::new();

    // Create parent category
    let parent = category_service.create_category(
        "Parent Category",
        Some("Parent category for testing"),
        None,
    ).unwrap();

    // Create child category
    let child = category_service.create_category(
        "Child Category",
        Some("Child category for testing"),
        Some(parent.id),
    ).unwrap();

    assert_eq!(child.parent_id, Some(parent.id));

    // Test listing categories
    let pagination = PaginationParams::new(1, 10);
    let categories = category_service.list_categories(&pagination);
    assert!(categories.is_ok());
    let result = categories.unwrap();
    assert!(result.data.len() >= 2); // At least our parent and child
}

#[test]
fn test_product_creation_and_validation() {
    setup_test_db();

    let product_service = ProductService::new();
    let category_service = CategoryService::new();

    // Create a category first
    let category = category_service.create_category(
        "Test Products",
        Some("Category for product testing"),
        None,
    ).unwrap();

    // Test creating a valid product
    let result = product_service.create_product(
        "PROD001",
        "Test Product",
        Some("A test product for unit testing"),
        category.id,
        10000, // 100.00 in cents
        8000,  // 80.00 in cents
        50,    // Initial stock
        10,    // Min stock level
        Some(100), // Max stock level
        "EA",
        Some("1234567890"),
    );

    assert!(result.is_ok());
    let product = result.unwrap();
    assert_eq!(product.sku, "PROD001");
    assert_eq!(product.name, "Test Product");
    assert_eq!(product.price, 10000);
    assert_eq!(product.current_stock, 50);

    // Test duplicate SKU validation
    let duplicate_result = product_service.create_product(
        "PROD001", // Same SKU
        "Another Product",
        None,
        category.id,
        5000,
        4000,
        20,
        5,
        None,
        "EA",
        None,
    );

    assert!(duplicate_result.is_err());
    assert!(duplicate_result.unwrap_err().to_string().contains("SKU already exists"));
}

#[test]
fn test_product_stock_management() {
    setup_test_db();

    let product_service = ProductService::new();
    let category_service = CategoryService::new();

    // Create category and product
    let category = category_service.create_category("Stock Test", None, None).unwrap();
    let product = product_service.create_product(
        "STOCK001",
        "Stock Test Product",
        None,
        category.id,
        5000,
        4000,
        10, // Initial stock
        5,  // Min level
        Some(50),
        "EA",
        None,
    ).unwrap();

    // Test stock increase
    let result = product_service.update_stock(
        product.id,
        20, // Add 20 units
        "in",
        Some(4000),
        Some("purchase"),
        None,
        Some("Restocking"),
        Some(1), // Test user
    );

    assert!(result.is_ok());
    let updated_product = result.unwrap();
    assert_eq!(updated_product.current_stock, 30); // 10 + 20

    // Test stock decrease
    let result = product_service.update_stock(
        product.id,
        -5, // Remove 5 units
        "out",
        None,
        Some("sale"),
        None,
        Some("Sale transaction"),
        Some(1),
    );

    assert!(result.is_ok());
    let updated_product = result.unwrap();
    assert_eq!(updated_product.current_stock, 25); // 30 - 5

    // Test negative stock validation
    let negative_result = product_service.update_stock(
        product.id,
        -50, // Too much to remove
        "out",
        None,
        None,
        None,
        None,
        None,
    );

    assert!(negative_result.is_err());
    assert!(negative_result.unwrap_err().to_string().contains("cannot be negative"));
}

#[test]
fn test_low_stock_detection() {
    setup_test_db();

    let product_service = ProductService::new();
    let category_service = CategoryService::new();

    // Create category
    let category = category_service.create_category("Low Stock Test", None, None).unwrap();

    // Create product with low stock
    let _low_stock_product = product_service.create_product(
        "LOW001",
        "Low Stock Product",
        None,
        category.id,
        5000,
        4000,
        3, // Below minimum of 5
        5,
        Some(50),
        "EA",
        None,
    ).unwrap();

    // Create product with normal stock
    let _normal_product = product_service.create_product(
        "NORMAL001",
        "Normal Stock Product",
        None,
        category.id,
        5000,
        4000,
        20, // Above minimum of 5
        5,
        Some(50),
        "EA",
        None,
    ).unwrap();

    // Test low stock detection
    let low_stock_products = product_service.get_low_stock_products();
    assert!(low_stock_products.is_ok());
    let products = low_stock_products.unwrap();

    // Should find at least our low stock product
    assert!(!products.is_empty());
    let low_product = products.iter().find(|p| p.product.sku == "LOW001");
    assert!(low_product.is_some());
}

#[test]
fn test_supplier_management() {
    setup_test_db();

    let supplier_service = SupplierService::new();

    // Test creating a supplier
    let result = supplier_service.create_supplier(
        "SUPP001",
        "Test Supplier Co.",
        Some("John Doe"),
        Some("supplier@test.com"),
        Some("555-1234"),
        Some("123 Business St, City, State 12345"),
        Some("Net 30"),
    );

    assert!(result.is_ok());
    let supplier = result.unwrap();
    assert_eq!(supplier.supplier_code, "SUPP001");
    assert_eq!(supplier.name, "Test Supplier Co.");
    assert_eq!(supplier.contact_person, Some("John Doe".to_string()));

    // Test retrieving supplier
    let retrieved = supplier_service.get_supplier_by_id(supplier.id);
    assert!(retrieved.is_ok());

    // Test duplicate supplier code validation
    let duplicate_result = supplier_service.create_supplier(
        "SUPP001", // Same code
        "Another Supplier",
        None,
        None,
        None,
        None,
        None,
    );

    assert!(duplicate_result.is_err());
    assert!(duplicate_result.unwrap_err().to_string().contains("already exists"));
}

#[test]
fn test_purchase_order_workflow() {
    setup_test_db();

    let po_service = PurchaseOrderService::new();
    let supplier_service = SupplierService::new();
    let product_service = ProductService::new();
    let category_service = CategoryService::new();

    // Create test data
    let supplier = supplier_service.create_supplier(
        "SUPP001",
        "Test Supplier",
        None,
        None,
        None,
        None,
        None,
    ).unwrap();

    let category = category_service.create_category("PO Test", None, None).unwrap();
    let product = product_service.create_product(
        "PROD001",
        "Test Product",
        None,
        category.id,
        5000,
        4000,
        0, // No initial stock
        10,
        Some(100),
        "EA",
        None,
    ).unwrap();

    // Create purchase order
    let order_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    let expected_date = NaiveDate::from_ymd_opt(2024, 1, 25).unwrap();

    let po_result = po_service.create_purchase_order(
        supplier.id,
        order_date,
        Some(expected_date),
        Some("Test PO notes"),
        vec![(product.id, 50, 4000)], // 50 units at 40.00 each
    );

    assert!(po_result.is_ok());
    let po = po_result.unwrap();
    assert_eq!(po.supplier_id, supplier.id);
    assert_eq!(po.total_amount, 200000); // 50 * 4000 = 200000 cents
    assert_eq!(po.status, "pending");

    // Test approving the purchase order
    let approve_result = po_service.approve_purchase_order(po.id, Some(1));
    assert!(approve_result.is_ok());
    let approved_po = approve_result.unwrap();
    assert_eq!(approved_po.status, "approved");

    // Test receiving the purchase order
    let receive_result = po_service.receive_purchase_order(
        po.id,
        vec![(product.id, 45)], // Received 45 out of 50 ordered
        Some("Partial delivery"),
        Some(1),
    );

    assert!(receive_result.is_ok());

    // Verify stock was updated
    let updated_product = product_service.get_product_by_id(product.id).unwrap();
    assert_eq!(updated_product.current_stock, 45);
}

#[test]
fn test_product_search_and_filtering() {
    setup_test_db();

    let product_service = ProductService::new();
    let category_service = CategoryService::new();

    // Create categories
    let electronics = category_service.create_category("Electronics", None, None).unwrap();
    let books = category_service.create_category("Books", None, None).unwrap();

    // Create products
    let _laptop = product_service.create_product(
        "LAPTOP001",
        "Gaming Laptop",
        Some("High-performance laptop"),
        electronics.id,
        150000,
        120000,
        5,
        2,
        Some(20),
        "EA",
        None,
    ).unwrap();

    let _book = product_service.create_product(
        "BOOK001",
        "Programming Guide",
        Some("Learn Rust programming"),
        books.id,
        5000,
        3000,
        100,
        10,
        Some(500),
        "EA",
        None,
    ).unwrap();

    let pagination = PaginationParams::new(1, 10);

    // Test search by name
    let search_result = product_service.list_products(
        &pagination,
        None,
        true, // active only
        Some("laptop"), // search term
        false,
    );

    assert!(search_result.is_ok());
    let results = search_result.unwrap();
    assert_eq!(results.data.len(), 1);
    assert_eq!(results.data[0].product.name, "Gaming Laptop");

    // Test filter by category
    let category_result = product_service.list_products(
        &pagination,
        Some(books.id), // filter by books category
        true,
        None,
        false,
    );

    assert!(category_result.is_ok());
    let results = category_result.unwrap();
    assert_eq!(results.data.len(), 1);
    assert_eq!(results.data[0].product.name, "Programming Guide");

    // Test low stock filter (none should be low stock)
    let low_stock_result = product_service.list_products(
        &pagination,
        None,
        true,
        None,
        true, // low stock only
    );

    assert!(low_stock_result.is_ok());
    let results = low_stock_result.unwrap();
    assert_eq!(results.data.len(), 0); // No low stock products
}

#[test]
fn test_stock_movement_history() {
    setup_test_db();

    let product_service = ProductService::new();
    let category_service = CategoryService::new();

    // Create test product
    let category = category_service.create_category("Movement Test", None, None).unwrap();
    let product = product_service.create_product(
        "MOVE001",
        "Movement Test Product",
        None,
        category.id,
        5000,
        4000,
        10,
        5,
        Some(50),
        "EA",
        None,
    ).unwrap();

    // Create multiple stock movements
    let _ = product_service.update_stock(
        product.id,
        20,
        "in",
        Some(4000),
        Some("purchase"),
        None,
        Some("Purchase order #1"),
        Some(1),
    );

    let _ = product_service.update_stock(
        product.id,
        -5,
        "out",
        None,
        Some("sale"),
        None,
        Some("Sale #1"),
        Some(1),
    );

    let _ = product_service.update_stock(
        product.id,
        -3,
        "out",
        None,
        Some("sale"),
        None,
        Some("Sale #2"),
        Some(1),
    );

    // Test retrieving stock movements
    let pagination = PaginationParams::new(1, 10);
    let movements_result = product_service.get_stock_movements(product.id, &pagination);

    assert!(movements_result.is_ok());
    let movements = movements_result.unwrap();

    // Should have at least 4 movements (1 initial + 3 we created)
    assert!(movements.data.len() >= 4);

    // Check that movements are in descending order by date
    for i in 1..movements.data.len() {
        assert!(movements.data[i-1].movement_date >= movements.data[i].movement_date);
    }
}

#[test]
fn test_product_validation_edge_cases() {
    setup_test_db();

    let product_service = ProductService::new();
    let category_service = CategoryService::new();

    let category = category_service.create_category("Validation Test", None, None).unwrap();

    // Test empty SKU
    let empty_sku_result = product_service.create_product(
        "",
        "Test Product",
        None,
        category.id,
        5000,
        4000,
        10,
        5,
        Some(50),
        "EA",
        None,
    );
    assert!(empty_sku_result.is_err());

    // Test empty name
    let empty_name_result = product_service.create_product(
        "VALID001",
        "",
        None,
        category.id,
        5000,
        4000,
        10,
        5,
        Some(50),
        "EA",
        None,
    );
    assert!(empty_name_result.is_err());

    // Test negative price
    let negative_price_result = product_service.create_product(
        "VALID002",
        "Test Product",
        None,
        category.id,
        -1000, // Negative price
        4000,
        10,
        5,
        Some(50),
        "EA",
        None,
    );
    assert!(negative_price_result.is_err());

    // Test min > max stock level
    let invalid_stock_result = product_service.create_product(
        "VALID003",
        "Test Product",
        None,
        category.id,
        5000,
        4000,
        10,
        50,     // Min level
        Some(20), // Max level < min level
        "EA",
        None,
    );
    assert!(invalid_stock_result.is_err());
}

#[test]
fn test_product_update_functionality() {
    setup_test_db();

    let product_service = ProductService::new();
    let category_service = CategoryService::new();

    // Create test data
    let category1 = category_service.create_category("Category 1", None, None).unwrap();
    let category2 = category_service.create_category("Category 2", None, None).unwrap();

    let product = product_service.create_product(
        "UPDATE001",
        "Original Product",
        Some("Original description"),
        category1.id,
        5000,
        4000,
        10,
        5,
        Some(50),
        "EA",
        None,
    ).unwrap();

    // Test updating various fields
    let update_result = product_service.update_product(
        product.id,
        Some("Updated Product Name"),
        Some(Some("Updated description")),
        Some(category2.id),
        Some(7500), // New price
        Some(6000), // New cost
        Some(8),    // New min level
        Some(Some(75)), // New max level
        Some("PC"),
        Some(Some("9876543210")),
        Some(true),
    );

    assert!(update_result.is_ok());
    let updated_product = update_result.unwrap();

    assert_eq!(updated_product.name, "Updated Product Name");
    assert_eq!(updated_product.description, Some("Updated description".to_string()));
    assert_eq!(updated_product.category_id, category2.id);
    assert_eq!(updated_product.price, 7500);
    assert_eq!(updated_product.cost_price, 6000);
    assert_eq!(updated_product.min_stock_level, 8);
    assert_eq!(updated_product.max_stock_level, Some(75));
    assert_eq!(updated_product.unit, "PC");
    assert_eq!(updated_product.barcode, Some("9876543210".to_string()));

    // Test partial update (only name)
    let partial_update_result = product_service.update_product(
        product.id,
        Some("Partially Updated"),
        None, // Don't change description
        None, // Don't change category
        None, // Don't change price
        None, // Don't change cost
        None, // Don't change min level
        None, // Don't change max level
        None, // Don't change unit
        None, // Don't change barcode
        None, // Don't change active status
    );

    assert!(partial_update_result.is_ok());
    let partially_updated = partial_update_result.unwrap();
    assert_eq!(partially_updated.name, "Partially Updated");
    assert_eq!(partially_updated.category_id, category2.id); // Should remain unchanged
}