#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::connection::DatabaseManager;
    use crate::database::models::{NewEmployee, NewDepartment};
    use crate::database::schema::{employees, departments};
    use chrono::{NaiveDate, Local};
    use diesel::prelude::*;

    fn setup_test_data(conn: &mut SqliteConnection) -> (i32, i32) {
        // Create test department
        let new_dept = NewDepartment {
            name: "Test Department".to_string(),
            description: Some("Test description".to_string()),
            manager_id: None,
        };

        let department = diesel::insert_into(departments::table)
            .values(&new_dept)
            .get_result::<crate::database::models::Department>(conn)
            .expect("Failed to create test department");

        // Create test employee
        let new_employee = NewEmployee {
            employee_code: "EMP001".to_string(),
            name: "Test Employee".to_string(),
            email: Some("test@example.com".to_string()),
            phone: None,
            department_id: department.id,
            position: "Test Position".to_string(),
            hire_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            salary: 50000,
            status: "active".to_string(),
        };

        let employee = diesel::insert_into(employees::table)
            .values(&new_employee)
            .get_result::<crate::database::models::Employee>(conn)
            .expect("Failed to create test employee");

        (department.id, employee.id)
    }

    #[test]
    fn test_check_in_new_employee() {
        let db_manager = DatabaseManager::new().expect("Failed to create DB manager");
        let mut conn = db_manager.get_connection().expect("Failed to get connection");

        // Setup test data
        let (_dept_id, employee_id) = setup_test_data(&mut conn);

        let attendance_service = AttendanceService::new();

        // Test check-in
        let result = attendance_service.check_in(&mut conn, employee_id);
        assert!(result.is_ok());

        let attendance = result.unwrap();
        assert_eq!(attendance.employee_id, employee_id);
        assert!(attendance.check_in.is_some());
        assert!(attendance.check_out.is_none());
        assert!(attendance.status == "present" || attendance.status == "late");
    }

    #[test]
    fn test_check_out_after_check_in() {
        let db_manager = DatabaseManager::new().expect("Failed to create DB manager");
        let mut conn = db_manager.get_connection().expect("Failed to get connection");

        let (_dept_id, employee_id) = setup_test_data(&mut conn);
        let attendance_service = AttendanceService::new();

        // Check in first
        attendance_service.check_in(&mut conn, employee_id)
            .expect("Failed to check in");

        // Test check-out
        let result = attendance_service.check_out(&mut conn, employee_id);
        assert!(result.is_ok());

        let attendance = result.unwrap();
        assert_eq!(attendance.employee_id, employee_id);
        assert!(attendance.check_in.is_some());
        assert!(attendance.check_out.is_some());
    }

    #[test]
    fn test_mark_absent() {
        let db_manager = DatabaseManager::new().expect("Failed to create DB manager");
        let mut conn = db_manager.get_connection().expect("Failed to get connection");

        let (_dept_id, employee_id) = setup_test_data(&mut conn);
        let attendance_service = AttendanceService::new();

        let date = Local::now().date_naive();
        let result = attendance_service.mark_absent(&mut conn, employee_id, date, Some("Sick leave".to_string()));

        assert!(result.is_ok());
        let attendance = result.unwrap();
        assert_eq!(attendance.employee_id, employee_id);
        assert_eq!(attendance.status, "absent");
        assert_eq!(attendance.notes, Some("Sick leave".to_string()));
    }

    #[test]
    fn test_get_monthly_stats() {
        let db_manager = DatabaseManager::new().expect("Failed to create DB manager");
        let mut conn = db_manager.get_connection().expect("Failed to get connection");

        let (_dept_id, employee_id) = setup_test_data(&mut conn);
        let attendance_service = AttendanceService::new();

        // Create some test attendance records
        let today = Local::now().date_naive();
        attendance_service.check_in(&mut conn, employee_id).expect("Failed to check in");

        // Get monthly stats
        let result = attendance_service.get_monthly_stats(&mut conn, employee_id, today.year(), today.month());
        assert!(result.is_ok());

        let stats = result.unwrap();
        assert_eq!(stats.total_days, 1);
        assert_eq!(stats.present_days, 1);
        assert_eq!(stats.absent_days, 0);
    }
}