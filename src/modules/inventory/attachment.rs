use std::fs;
use std::path::{Path, PathBuf};
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

use crate::core::result::CLIERPResult;
use crate::database::connection::get_connection;
use crate::database::models::{ProductAttachment, NewProductAttachment};
use crate::database::schema::product_attachments;
use crate::utils::validation::{validate_required_string, ValidationResult};

#[derive(Debug, Clone)]
pub struct AttachmentService {
    storage_path: PathBuf,
}

impl AttachmentService {
    pub fn new() -> Self {
        let storage_path = PathBuf::from("./storage/attachments");
        Self { storage_path }
    }

    pub fn with_storage_path(storage_path: PathBuf) -> Self {
        Self { storage_path }
    }

    fn ensure_storage_directory(&self) -> CLIERPResult<()> {
        if !self.storage_path.exists() {
            fs::create_dir_all(&self.storage_path)?;
        }
        Ok(())
    }

    fn get_product_directory(&self, product_id: i32) -> PathBuf {
        self.storage_path.join(format!("product_{}", product_id))
    }

    fn generate_unique_filename(&self, original_filename: &str) -> String {
        let uuid = Uuid::new_v4();
        let extension = Path::new(original_filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        if extension.is_empty() {
            format!("{}", uuid)
        } else {
            format!("{}.{}", uuid, extension)
        }
    }

    fn get_mime_type(&self, file_path: &Path) -> Option<String> {
        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())?
            .to_lowercase();

        match extension.as_str() {
            "jpg" | "jpeg" => Some("image/jpeg".to_string()),
            "png" => Some("image/png".to_string()),
            "gif" => Some("image/gif".to_string()),
            "webp" => Some("image/webp".to_string()),
            "pdf" => Some("application/pdf".to_string()),
            "doc" => Some("application/msword".to_string()),
            "docx" => Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string()),
            "txt" => Some("text/plain".to_string()),
            "csv" => Some("text/csv".to_string()),
            "xlsx" => Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string()),
            _ => None,
        }
    }

    pub fn add_attachment(
        &self,
        product_id: i32,
        attachment_type: &str,
        source_file_path: &Path,
        is_primary: bool,
    ) -> CLIERPResult<ProductAttachment> {
        // Validate inputs
        validate_required_string(attachment_type, "Attachment type")?;

        if !["image", "document", "manual", "certificate"].contains(&attachment_type) {
            return Err(crate::core::error::CLIERPError::ValidationError(
                "Invalid attachment type. Must be one of: image, document, manual, certificate".to_string(),
            ));
        }

        if !source_file_path.exists() {
            return Err(crate::core::error::CLIERPError::ValidationError(
                "Source file does not exist".to_string(),
            ));
        }

        if !source_file_path.is_file() {
            return Err(crate::core::error::CLIERPError::ValidationError(
                "Source path is not a file".to_string(),
            ));
        }

        // Ensure storage directories exist
        self.ensure_storage_directory()?;
        let product_dir = self.get_product_directory(product_id);
        if !product_dir.exists() {
            fs::create_dir_all(&product_dir)?;
        }

        // Generate unique filename and destination path
        let original_filename = source_file_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| crate::core::error::CLIERPError::ValidationError(
                "Invalid source filename".to_string(),
            ))?;

        let unique_filename = self.generate_unique_filename(original_filename);
        let destination_path = product_dir.join(&unique_filename);

        // Get file size and MIME type
        let file_size = source_file_path.metadata()?.len() as i32;
        let mime_type = self.get_mime_type(source_file_path);

        // Copy file to storage
        fs::copy(source_file_path, &destination_path)?;

        let mut connection = get_connection()?;

        // If setting as primary, unset current primary attachment of same type
        if is_primary {
            diesel::update(
                product_attachments::table
                    .filter(product_attachments::product_id.eq(product_id))
                    .filter(product_attachments::attachment_type.eq(attachment_type))
                    .filter(product_attachments::is_primary.eq(true))
            )
            .set(product_attachments::is_primary.eq(false))
            .execute(&mut connection)?;
        }

        // Create attachment record
        let new_attachment = NewProductAttachment {
            product_id,
            attachment_type: attachment_type.to_string(),
            file_name: original_filename.to_string(),
            file_path: destination_path.to_string_lossy().to_string(),
            file_size,
            mime_type,
            is_primary,
        };

        diesel::insert_into(product_attachments::table)
            .values(&new_attachment)
            .execute(&mut connection)?;

        let attachment = product_attachments::table
            .order(product_attachments::id.desc())
            .first::<ProductAttachment>(&mut connection)?;

        tracing::info!(
            "Added attachment {} for product {} ({})",
            attachment.file_name,
            product_id,
            attachment_type
        );

        Ok(attachment)
    }

    pub fn list_attachments(
        &self,
        product_id: i32,
        attachment_type: Option<&str>,
    ) -> CLIERPResult<Vec<ProductAttachment>> {
        let mut connection = get_connection()?;

        let mut query = product_attachments::table
            .filter(product_attachments::product_id.eq(product_id))
            .into_boxed();

        if let Some(attachment_type) = attachment_type {
            query = query.filter(product_attachments::attachment_type.eq(attachment_type));
        }

        let attachments = query
            .order_by((product_attachments::is_primary.desc(), product_attachments::created_at.desc()))
            .load::<ProductAttachment>(&mut connection)?;

        Ok(attachments)
    }

    pub fn get_attachment(&self, id: i32) -> CLIERPResult<ProductAttachment> {
        let mut connection = get_connection()?;

        let attachment = product_attachments::table
            .find(id)
            .first::<ProductAttachment>(&mut connection)?;

        Ok(attachment)
    }

    pub fn delete_attachment(&self, id: i32) -> CLIERPResult<()> {
        let mut connection = get_connection()?;

        // Get attachment info before deletion
        let attachment = self.get_attachment(id)?;

        // Delete file from storage
        let file_path = Path::new(&attachment.file_path);
        if file_path.exists() {
            fs::remove_file(file_path)?;
        }

        // Delete attachment record
        diesel::delete(product_attachments::table.find(id))
            .execute(&mut connection)?;

        tracing::info!(
            "Deleted attachment {} for product {}",
            attachment.file_name,
            attachment.product_id
        );

        Ok(())
    }

    pub fn set_primary_attachment(
        &self,
        id: i32,
    ) -> CLIERPResult<ProductAttachment> {
        let mut connection = get_connection()?;

        let attachment = self.get_attachment(id)?;

        // Unset current primary attachment of same type
        diesel::update(
            product_attachments::table
                .filter(product_attachments::product_id.eq(attachment.product_id))
                .filter(product_attachments::attachment_type.eq(&attachment.attachment_type))
                .filter(product_attachments::is_primary.eq(true))
        )
        .set(product_attachments::is_primary.eq(false))
        .execute(&mut connection)?;

        // Set this attachment as primary
        diesel::update(product_attachments::table.find(id))
            .set((
                product_attachments::is_primary.eq(true),
                product_attachments::updated_at.eq(Utc::now().naive_utc()),
            ))
            .execute(&mut connection)?;

        let updated_attachment = self.get_attachment(id)?;

        tracing::info!(
            "Set attachment {} as primary for product {}",
            updated_attachment.file_name,
            updated_attachment.product_id
        );

        Ok(updated_attachment)
    }

    pub fn get_primary_image(&self, product_id: i32) -> CLIERPResult<Option<ProductAttachment>> {
        let mut connection = get_connection()?;

        let attachment = product_attachments::table
            .filter(product_attachments::product_id.eq(product_id))
            .filter(product_attachments::attachment_type.eq("image"))
            .filter(product_attachments::is_primary.eq(true))
            .first::<ProductAttachment>(&mut connection)
            .optional()?;

        Ok(attachment)
    }

    pub fn get_storage_path(&self) -> &Path {
        &self.storage_path
    }

    pub fn cleanup_orphaned_files(&self) -> CLIERPResult<usize> {
        let mut connection = get_connection()?;
        let mut cleaned_count = 0;

        // Get all attachment records
        let attachments = product_attachments::table
            .load::<ProductAttachment>(&mut connection)?;

        let registered_paths: std::collections::HashSet<String> = attachments
            .iter()
            .map(|a| a.file_path.clone())
            .collect();

        // Walk through storage directory
        if self.storage_path.exists() {
            for entry in fs::read_dir(&self.storage_path)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    // Check product directories
                    for product_entry in fs::read_dir(&path)? {
                        let product_entry = product_entry?;
                        let file_path = product_entry.path();

                        if file_path.is_file() {
                            let file_path_str = file_path.to_string_lossy().to_string();
                            if !registered_paths.contains(&file_path_str) {
                                fs::remove_file(&file_path)?;
                                cleaned_count += 1;
                                tracing::info!("Cleaned orphaned file: {}", file_path_str);
                            }
                        }
                    }
                }
            }
        }

        Ok(cleaned_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_attachment_service_creation() {
        let service = AttachmentService::new();
        assert!(service.storage_path.to_string_lossy().contains("storage/attachments"));
    }

    #[test]
    fn test_unique_filename_generation() {
        let temp_dir = TempDir::new().unwrap();
        let service = AttachmentService::with_storage_path(temp_dir.path().to_path_buf());

        let filename1 = service.generate_unique_filename("test.jpg");
        let filename2 = service.generate_unique_filename("test.jpg");

        assert_ne!(filename1, filename2);
        assert!(filename1.ends_with(".jpg"));
        assert!(filename2.ends_with(".jpg"));
    }

    #[test]
    fn test_mime_type_detection() {
        let temp_dir = TempDir::new().unwrap();
        let service = AttachmentService::with_storage_path(temp_dir.path().to_path_buf());

        assert_eq!(service.get_mime_type(Path::new("test.jpg")), Some("image/jpeg".to_string()));
        assert_eq!(service.get_mime_type(Path::new("test.png")), Some("image/png".to_string()));
        assert_eq!(service.get_mime_type(Path::new("test.pdf")), Some("application/pdf".to_string()));
        assert_eq!(service.get_mime_type(Path::new("test.unknown")), None);
    }
}