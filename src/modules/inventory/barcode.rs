use std::io::Cursor;
use image::{ImageBuffer, Rgb, RgbImage};
use qrcode::{QrCode, EcLevel};
use qrcode::render::unicode;

use crate::core::result::CLIERPResult;
use crate::database::models::Product;

#[derive(Debug, Clone)]
pub struct BarcodeService;

impl BarcodeService {
    pub fn new() -> Self {
        Self
    }

    /// Generate QR code containing product information as JSON
    pub fn generate_product_qr_code(&self, product: &Product) -> CLIERPResult<String> {
        // Create JSON data for QR code
        let qr_data = serde_json::json!({
            "id": product.id,
            "sku": product.sku,
            "name": product.name,
            "price": product.price,
            "type": "product"
        });

        let qr_string = qr_data.to_string();

        // Generate QR code
        let qr_code = QrCode::with_error_correction_level(&qr_string, EcLevel::M)
            .map_err(|e| crate::core::error::CLIERPError::ValidationError(
                format!("Failed to generate QR code: {}", e)
            ))?;

        // Render as Unicode string for terminal display
        let unicode_string = qr_code
            .render::<unicode::Dense1x2>()
            .dark_color(unicode::Dense1x2::Light)
            .light_color(unicode::Dense1x2::Dark)
            .build();

        Ok(unicode_string)
    }

    /// Generate QR code as PNG image data
    pub fn generate_product_qr_code_image(&self, product: &Product, size: u32) -> CLIERPResult<Vec<u8>> {
        // Create JSON data for QR code
        let qr_data = serde_json::json!({
            "id": product.id,
            "sku": product.sku,
            "name": product.name,
            "price": product.price,
            "type": "product"
        });

        let qr_string = qr_data.to_string();

        // Generate QR code
        let qr_code = QrCode::with_error_correction_level(&qr_string, EcLevel::M)
            .map_err(|e| crate::core::error::CLIERPError::ValidationError(
                format!("Failed to generate QR code: {}", e)
            ))?;

        // Calculate module size based on desired image size
        let qr_width = qr_code.width() as u32;
        let module_size = size / qr_width;
        let actual_size = qr_width * module_size;

        // Create image buffer
        let mut img: RgbImage = ImageBuffer::new(actual_size, actual_size);

        // Fill with white background
        for pixel in img.pixels_mut() {
            *pixel = Rgb([255, 255, 255]);
        }

        // Draw QR code modules
        for y in 0..qr_width {
            for x in 0..qr_width {
                if qr_code[(x as usize, y as usize)] == qrcode::Color::Dark {
                    // Draw dark module
                    for dy in 0..module_size {
                        for dx in 0..module_size {
                            let img_x = x * module_size + dx;
                            let img_y = y * module_size + dy;
                            if img_x < actual_size && img_y < actual_size {
                                img.put_pixel(img_x, img_y, Rgb([0, 0, 0]));
                            }
                        }
                    }
                }
            }
        }

        // Encode as PNG
        let mut png_data = Vec::new();
        {
            let mut cursor = Cursor::new(&mut png_data);
            img.write_to(&mut cursor, image::ImageOutputFormat::Png)
                .map_err(|e| crate::core::error::CLIERPError::ValidationError(
                    format!("Failed to encode PNG: {}", e)
                ))?;
        }

        Ok(png_data)
    }

    /// Generate simple barcode string for SKU (Code 128 format simulation)
    pub fn generate_sku_barcode(&self, sku: &str) -> CLIERPResult<String> {
        // Simple barcode representation using ASCII characters
        // In a real implementation, you'd use a proper barcode library
        let mut barcode = String::new();

        // Start guard
        barcode.push_str("||  ");

        // Encode SKU characters
        for ch in sku.chars() {
            match ch {
                'A'..='Z' => barcode.push_str("| || "),
                'a'..='z' => barcode.push_str("|| | "),
                '0'..='9' => barcode.push_str("|  ||"),
                _ => barcode.push_str("|   |"),
            }
            barcode.push(' ');
        }

        // End guard
        barcode.push_str("  ||");

        Ok(barcode)
    }

    /// Generate inventory label with QR code and product info
    pub fn generate_inventory_label(&self, product: &Product) -> CLIERPResult<String> {
        let qr_code = self.generate_product_qr_code(product)?;
        let barcode = self.generate_sku_barcode(&product.sku)?;

        let label = format!(
            "┌─────────────────────────────────────────┐\n\
             │ INVENTORY LABEL                         │\n\
             ├─────────────────────────────────────────┤\n\
             │ SKU: {:<35} │\n\
             │ Name: {:<33} │\n\
             │ Stock: {:<32} │\n\
             ├─────────────────────────────────────────┤\n\
             │ QR Code:                                │\n\
             {}\n\
             ├─────────────────────────────────────────┤\n\
             │ Barcode:                                │\n\
             │ {}                     │\n\
             └─────────────────────────────────────────┘",
            product.sku,
            if product.name.len() > 33 {
                format!("{}...", &product.name[..30])
            } else {
                product.name.clone()
            },
            format!("{} {}", product.current_stock, product.unit),
            qr_code.lines()
                .map(|line| format!("│ {:<39} │", line))
                .collect::<Vec<_>>()
                .join("\n"),
            if barcode.len() > 35 {
                format!("{}...", &barcode[..32])
            } else {
                barcode
            }
        );

        Ok(label)
    }

    /// Validate and format barcode for storage
    pub fn validate_barcode(&self, barcode: &str) -> CLIERPResult<String> {
        let cleaned = barcode.trim().to_uppercase();

        if cleaned.is_empty() {
            return Err(crate::core::error::CLIERPError::ValidationError(
                "Barcode cannot be empty".to_string(),
            ));
        }

        if cleaned.len() > 50 {
            return Err(crate::core::error::CLIERPError::ValidationError(
                "Barcode cannot be longer than 50 characters".to_string(),
            ));
        }

        // Check for valid barcode characters (alphanumeric and some symbols)
        if !cleaned.chars().all(|c| c.is_alphanumeric() || "-_".contains(c)) {
            return Err(crate::core::error::CLIERPError::ValidationError(
                "Barcode contains invalid characters. Only alphanumeric, dash, and underscore allowed".to_string(),
            ));
        }

        Ok(cleaned)
    }

    /// Generate sequential barcode from SKU
    pub fn generate_sequential_barcode(&self, sku: &str, sequence: u32) -> String {
        format!("{}-{:06}", sku, sequence)
    }

    /// Parse QR code data to extract product information
    pub fn parse_product_qr_data(&self, qr_data: &str) -> CLIERPResult<serde_json::Value> {
        let parsed: serde_json::Value = serde_json::from_str(qr_data)
            .map_err(|e| crate::core::error::CLIERPError::ValidationError(
                format!("Invalid QR code data: {}", e)
            ))?;

        // Validate that it's a product QR code
        if parsed.get("type").and_then(|t| t.as_str()) != Some("product") {
            return Err(crate::core::error::CLIERPError::ValidationError(
                "QR code is not a product code".to_string(),
            ));
        }

        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::models::Product;
    use chrono::Utc;

    fn create_test_product() -> Product {
        Product {
            id: 1,
            sku: "TEST001".to_string(),
            name: "Test Product".to_string(),
            description: Some("Test description".to_string()),
            category_id: 1,
            price: 1000,
            cost_price: 500,
            current_stock: 10,
            min_stock_level: 5,
            max_stock_level: Some(100),
            unit: "ea".to_string(),
            barcode: None,
            is_active: true,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        }
    }

    #[test]
    fn test_qr_code_generation() {
        let service = BarcodeService::new();
        let product = create_test_product();

        let qr_code = service.generate_product_qr_code(&product).unwrap();
        assert!(!qr_code.is_empty());
    }

    #[test]
    fn test_barcode_generation() {
        let service = BarcodeService::new();

        let barcode = service.generate_sku_barcode("TEST001").unwrap();
        assert!(!barcode.is_empty());
        assert!(barcode.contains("||"));
    }

    #[test]
    fn test_barcode_validation() {
        let service = BarcodeService::new();

        assert!(service.validate_barcode("TEST001").is_ok());
        assert!(service.validate_barcode("TEST-001_A").is_ok());
        assert!(service.validate_barcode("").is_err());
        assert!(service.validate_barcode("TEST@001").is_err());
    }

    #[test]
    fn test_sequential_barcode() {
        let service = BarcodeService::new();

        let barcode = service.generate_sequential_barcode("TEST", 123);
        assert_eq!(barcode, "TEST-000123");
    }

    #[test]
    fn test_inventory_label_generation() {
        let service = BarcodeService::new();
        let product = create_test_product();

        let label = service.generate_inventory_label(&product).unwrap();
        assert!(label.contains("INVENTORY LABEL"));
        assert!(label.contains("TEST001"));
        assert!(label.contains("Test Product"));
    }
}