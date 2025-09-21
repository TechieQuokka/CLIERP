use chrono::Utc;
use diesel::prelude::*;

use crate::core::result::CLIERPResult;
use crate::database::connection::get_connection;
use crate::database::models::{Category, NewCategory};
use crate::database::schema::categories;
use crate::utils::pagination::{PaginationParams, PaginationResult};
use crate::utils::validation::{validate_required_string, ValidationResult};

#[derive(Debug, Clone)]
pub struct CategoryService;

impl CategoryService {
    pub fn new() -> Self {
        Self
    }

    pub fn create_category(
        &self,
        name: &str,
        description: Option<&str>,
        parent_id: Option<i32>,
    ) -> CLIERPResult<Category> {
        // Validate inputs
        validate_required_string(name, "Category name")?;

        // Check if parent category exists
        if let Some(parent_id) = parent_id {
            self.get_category_by_id(parent_id)?;
        }

        let mut connection = get_connection()?;

        // Check for duplicate name
        let existing = categories::table
            .filter(categories::name.eq(name))
            .first::<Category>(&mut connection)
            .optional()?;

        if existing.is_some() {
            return Err(crate::core::error::CLIERPError::ValidationError(
                "Category name already exists".to_string(),
            ));
        }

        let new_category = NewCategory {
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            parent_id,
            is_active: true,
        };

        diesel::insert_into(categories::table)
            .values(&new_category)
            .execute(&mut connection)?;

        let category = categories::table
            .order(categories::id.desc())
            .first::<Category>(&mut connection)?;

        tracing::info!("Created category: {}", category.name);
        Ok(category)
    }

    pub fn get_category_by_id(&self, id: i32) -> CLIERPResult<Category> {
        let mut connection = get_connection()?;

        let category = categories::table
            .find(id)
            .first::<Category>(&mut connection)?;

        Ok(category)
    }

    pub fn get_category_by_name(&self, name: &str) -> CLIERPResult<Option<Category>> {
        let mut connection = get_connection()?;

        let category = categories::table
            .filter(categories::name.eq(name))
            .first::<Category>(&mut connection)
            .optional()?;

        Ok(category)
    }

    pub fn list_categories(
        &self,
        pagination: &PaginationParams,
        parent_id: Option<i32>,
        active_only: bool,
    ) -> CLIERPResult<PaginationResult<Category>> {
        let mut connection = get_connection()?;

        let mut query = categories::table.into_boxed();

        // Filter by parent_id
        if let Some(parent_id) = parent_id {
            query = query.filter(categories::parent_id.eq(parent_id));
        }

        // Filter by active status
        if active_only {
            query = query.filter(categories::is_active.eq(true));
        }

        // Get total count
        let total_count = {
            let mut count_query = categories::table.into_boxed();
            if let Some(parent_id) = parent_id {
                count_query = count_query.filter(categories::parent_id.eq(parent_id));
            }
            if active_only {
                count_query = count_query.filter(categories::is_active.eq(true));
            }
            count_query.count().get_result::<i64>(&mut connection)? as usize
        };

        // Apply pagination and ordering
        let categories = query
            .order_by(categories::name.asc())
            .offset(pagination.offset())
            .limit(pagination.limit())
            .load::<Category>(&mut connection)?;

        Ok(PaginationResult::new_simple(categories, total_count, pagination))
    }

    pub fn get_category_tree(&self) -> CLIERPResult<Vec<CategoryTreeNode>> {
        let mut connection = get_connection()?;

        let all_categories = categories::table
            .filter(categories::is_active.eq(true))
            .order_by(categories::name.asc())
            .load::<Category>(&mut connection)?;

        let tree = self.build_category_tree(&all_categories, None);
        Ok(tree)
    }

    pub fn update_category(
        &self,
        id: i32,
        name: Option<&str>,
        description: Option<Option<&str>>,
        parent_id: Option<Option<i32>>,
        is_active: Option<bool>,
    ) -> CLIERPResult<Category> {
        let mut connection = get_connection()?;

        // Check if category exists
        let existing_category = self.get_category_by_id(id)?;

        // Validate name if provided
        if let Some(name) = name {
            validate_required_string(name, "Category name")?;

            // Check for duplicate name (excluding current category)
            let duplicate = categories::table
                .filter(categories::name.eq(name))
                .filter(categories::id.ne(id))
                .first::<Category>(&mut connection)
                .optional()?;

            if duplicate.is_some() {
                return Err(crate::core::error::CLIERPError::ValidationError(
                    "Category name already exists".to_string(),
                ));
            }
        }

        // Check if parent category exists
        if let Some(Some(parent_id)) = parent_id {
            if parent_id == id {
                return Err(crate::core::error::CLIERPError::ValidationError(
                    "Category cannot be its own parent".to_string(),
                ));
            }
            self.get_category_by_id(parent_id)?;
        }

        // Build update changeset
        let mut changeset = CategoryUpdateChangeset::default();

        if let Some(name) = name {
            changeset.name = Some(name.to_string());
        }
        if let Some(description) = description {
            changeset.description = Some(description.map(|s| s.to_string()));
        }
        if let Some(parent_id) = parent_id {
            changeset.parent_id = Some(parent_id);
        }
        if let Some(is_active) = is_active {
            changeset.is_active = Some(is_active);
        }
        changeset.updated_at = Some(Utc::now().naive_utc());

        diesel::update(categories::table.find(id))
            .set(&changeset)
            .execute(&mut connection)?;

        let updated_category = self.get_category_by_id(id)?;

        tracing::info!("Updated category: {}", updated_category.name);
        Ok(updated_category)
    }

    pub fn delete_category(&self, id: i32, force: bool) -> CLIERPResult<()> {
        let mut connection = get_connection()?;

        // Check if category exists
        let category = self.get_category_by_id(id)?;

        // Check for child categories
        let child_count = categories::table
            .filter(categories::parent_id.eq(id))
            .count()
            .get_result::<i64>(&mut connection)?;

        if child_count > 0 && !force {
            return Err(crate::core::error::CLIERPError::ValidationError(
                format!("Category has {} child categories. Use --force to delete anyway.", child_count),
            ));
        }

        // Check for products in this category
        use crate::database::schema::products;
        let product_count = products::table
            .filter(products::category_id.eq(id))
            .count()
            .get_result::<i64>(&mut connection)?;

        if product_count > 0 && !force {
            return Err(crate::core::error::CLIERPError::ValidationError(
                format!("Category has {} products. Use --force to delete anyway.", product_count),
            ));
        }

        // If force delete, update child categories and products
        if force {
            // Set child categories' parent_id to null
            diesel::update(categories::table.filter(categories::parent_id.eq(id)))
                .set(categories::parent_id.eq::<Option<i32>>(None))
                .execute(&mut connection)?;

            // Move products to "기타" category (assuming it exists as default)
            if let Ok(default_category) = self.get_category_by_name("기타") {
                if let Some(default_cat) = default_category {
                    diesel::update(products::table.filter(products::category_id.eq(id)))
                        .set(products::category_id.eq(default_cat.id))
                        .execute(&mut connection)?;
                }
            }
        }

        // Delete the category
        diesel::delete(categories::table.find(id)).execute(&mut connection)?;

        tracing::info!("Deleted category: {}", category.name);
        Ok(())
    }

    fn build_category_tree(
        &self,
        all_categories: &[Category],
        parent_id: Option<i32>,
    ) -> Vec<CategoryTreeNode> {
        all_categories
            .iter()
            .filter(|cat| cat.parent_id == parent_id)
            .map(|cat| {
                let children = self.build_category_tree(all_categories, Some(cat.id));
                CategoryTreeNode {
                    category: cat.clone(),
                    children,
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct CategoryTreeNode {
    pub category: Category,
    pub children: Vec<CategoryTreeNode>,
}

#[derive(AsChangeset, Default)]
#[diesel(table_name = categories)]
struct CategoryUpdateChangeset {
    name: Option<String>,
    description: Option<Option<String>>,
    parent_id: Option<Option<i32>>,
    is_active: Option<bool>,
    updated_at: Option<chrono::NaiveDateTime>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_service_creation() {
        let service = CategoryService::new();
        // Basic instantiation test
        assert!(true);
    }
}