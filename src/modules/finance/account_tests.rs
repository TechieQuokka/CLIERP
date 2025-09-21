#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::connection::DatabaseManager;
    use diesel::prelude::*;

    #[test]
    fn test_create_account() {
        let db_manager = DatabaseManager::new().expect("Failed to create DB manager");
        let mut conn = db_manager.get_connection().expect("Failed to get connection");

        let account_service = AccountService::new();

        let request = CreateAccountRequest {
            account_code: "1000".to_string(),
            account_name: "Cash".to_string(),
            account_type: "asset".to_string(),
            parent_id: None,
        };

        let result = account_service.create_account(&mut conn, request);
        assert!(result.is_ok());

        let account = result.unwrap();
        assert_eq!(account.account_code, "1000");
        assert_eq!(account.account_name, "Cash");
        assert_eq!(account.account_type, "asset");
        assert_eq!(account.balance, 0);
        assert!(account.is_active);
    }

    #[test]
    fn test_duplicate_account_code() {
        let db_manager = DatabaseManager::new().expect("Failed to create DB manager");
        let mut conn = db_manager.get_connection().expect("Failed to get connection");

        let account_service = AccountService::new();

        let request = CreateAccountRequest {
            account_code: "1001".to_string(),
            account_name: "Cash".to_string(),
            account_type: "asset".to_string(),
            parent_id: None,
        };

        // First creation should succeed
        let result1 = account_service.create_account(&mut conn, request.clone());
        assert!(result1.is_ok());

        // Second creation with same code should fail
        let result2 = account_service.create_account(&mut conn, request);
        assert!(result2.is_err());
    }

    #[test]
    fn test_get_account_by_code() {
        let db_manager = DatabaseManager::new().expect("Failed to create DB manager");
        let mut conn = db_manager.get_connection().expect("Failed to get connection");

        let account_service = AccountService::new();

        let request = CreateAccountRequest {
            account_code: "1002".to_string(),
            account_name: "Bank Account".to_string(),
            account_type: "asset".to_string(),
            parent_id: None,
        };

        let created = account_service.create_account(&mut conn, request).unwrap();

        let result = account_service.get_account_by_code(&mut conn, "1002");
        assert!(result.is_ok());

        let account = result.unwrap().unwrap();
        assert_eq!(account.id, created.id);
        assert_eq!(account.account_code, "1002");
    }

    #[test]
    fn test_list_accounts_by_type() {
        let db_manager = DatabaseManager::new().expect("Failed to create DB manager");
        let mut conn = db_manager.get_connection().expect("Failed to get connection");

        let account_service = AccountService::new();

        // Create asset accounts
        let asset_request = CreateAccountRequest {
            account_code: "1003".to_string(),
            account_name: "Asset Account".to_string(),
            account_type: "asset".to_string(),
            parent_id: None,
        };

        let liability_request = CreateAccountRequest {
            account_code: "2000".to_string(),
            account_name: "Liability Account".to_string(),
            account_type: "liability".to_string(),
            parent_id: None,
        };

        account_service.create_account(&mut conn, asset_request).unwrap();
        account_service.create_account(&mut conn, liability_request).unwrap();

        let asset_accounts = account_service.list_accounts_by_type(&mut conn, "asset").unwrap();
        let liability_accounts = account_service.list_accounts_by_type(&mut conn, "liability").unwrap();

        assert!(asset_accounts.iter().any(|a| a.account_code == "1003"));
        assert!(liability_accounts.iter().any(|a| a.account_code == "2000"));
        assert!(!asset_accounts.iter().any(|a| a.account_code == "2000"));
    }

    #[test]
    fn test_update_account_balance() {
        let db_manager = DatabaseManager::new().expect("Failed to create DB manager");
        let mut conn = db_manager.get_connection().expect("Failed to get connection");

        let account_service = AccountService::new();

        let request = CreateAccountRequest {
            account_code: "1004".to_string(),
            account_name: "Test Account".to_string(),
            account_type: "asset".to_string(),
            parent_id: None,
        };

        let account = account_service.create_account(&mut conn, request).unwrap();
        assert_eq!(account.balance, 0);

        let updated = account_service.update_account_balance(&mut conn, account.id, 1000).unwrap();
        assert_eq!(updated.balance, 1000);
    }
}