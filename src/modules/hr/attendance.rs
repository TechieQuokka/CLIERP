use diesel::prelude::*;
use chrono::{NaiveDate, NaiveTime, Local, Timelike};
use serde::{Deserialize, Serialize};

use crate::core::result::CLIERPResult;
use crate::core::error::CLIERPError;
use crate::database::models::{Attendance, NewAttendance, AttendanceStatus, Employee};
use crate::database::schema::{attendances, employees};

pub struct AttendanceService;

impl AttendanceService {
    pub fn new() -> Self {
        Self
    }

    /// Check in an employee for today
    pub fn check_in(&self, conn: &mut SqliteConnection, employee_id: i32) -> CLIERPResult<Attendance> {
        let today = Local::now().date_naive();
        let now = Local::now().time();

        // Check if already checked in today
        let existing_attendance = attendances::table
            .filter(attendances::employee_id.eq(employee_id))
            .filter(attendances::date.eq(today))
            .first::<Attendance>(conn)
            .optional()?;

        if let Some(mut attendance) = existing_attendance {
            if attendance.check_in.is_some() {
                return Err(CLIERPError::ValidationError(
                    "Employee already checked in today".to_string()
                ));
            }

            // Update existing record with check-in time
            let updated_attendance = diesel::update(attendances::table)
                .filter(attendances::id.eq(attendance.id))
                .set((
                    attendances::check_in.eq(Some(now)),
                    attendances::status.eq(if now.hour() > 9 { "late" } else { "present" }),
                ))
                .get_result::<Attendance>(conn)?;

            Ok(updated_attendance)
        } else {
            // Create new attendance record
            let new_attendance = NewAttendance {
                employee_id,
                date: today,
                check_in: Some(now),
                check_out: None,
                break_time: 0,
                overtime_hours: 0.0,
                status: if now.hour() > 9 { "late".to_string() } else { "present".to_string() },
                notes: None,
            };

            let attendance = diesel::insert_into(attendances::table)
                .values(&new_attendance)
                .get_result::<Attendance>(conn)?;

            Ok(attendance)
        }
    }

    /// Check out an employee for today
    pub fn check_out(&self, conn: &mut SqliteConnection, employee_id: i32) -> CLIERPResult<Attendance> {
        let today = Local::now().date_naive();
        let now = Local::now().time();

        let attendance = attendances::table
            .filter(attendances::employee_id.eq(employee_id))
            .filter(attendances::date.eq(today))
            .first::<Attendance>(conn)
            .optional()?
            .ok_or_else(|| CLIERPError::NotFound("No check-in record found for today".to_string()))?;

        if attendance.check_out.is_some() {
            return Err(CLIERPError::ValidationError(
                "Employee already checked out today".to_string()
            ));
        }

        if attendance.check_in.is_none() {
            return Err(CLIERPError::ValidationError(
                "Employee must check in before checking out".to_string()
            ));
        }

        // Calculate overtime hours if applicable
        let check_in_time = attendance.check_in.unwrap();
        let work_hours = Self::calculate_work_hours(check_in_time, now, attendance.break_time);
        let overtime_hours = if work_hours > 8.0 { work_hours - 8.0 } else { 0.0 };

        let updated_attendance = diesel::update(attendances::table)
            .filter(attendances::id.eq(attendance.id))
            .set((
                attendances::check_out.eq(Some(now)),
                attendances::overtime_hours.eq(overtime_hours),
                attendances::status.eq(if now.hour() < 17 { "early_leave" } else { "present" }),
            ))
            .get_result::<Attendance>(conn)?;

        Ok(updated_attendance)
    }

    /// Get attendance for a specific date
    pub fn get_attendance_by_date(&self, conn: &mut SqliteConnection, employee_id: i32, date: NaiveDate) -> CLIERPResult<Option<AttendanceWithEmployee>> {
        let result = attendances::table
            .inner_join(employees::table)
            .filter(attendances::employee_id.eq(employee_id))
            .filter(attendances::date.eq(date))
            .select((Attendance::as_select(), Employee::as_select()))
            .first::<(Attendance, Employee)>(conn)
            .optional()?;

        Ok(result.map(|(attendance, employee)| AttendanceWithEmployee {
            attendance,
            employee,
        }))
    }

    /// Get today's attendance for all employees
    pub fn get_today_attendance(&self, conn: &mut SqliteConnection) -> CLIERPResult<Vec<AttendanceWithEmployee>> {
        let today = Local::now().date_naive();

        let results = attendances::table
            .inner_join(employees::table)
            .filter(attendances::date.eq(today))
            .select((Attendance::as_select(), Employee::as_select()))
            .load::<(Attendance, Employee)>(conn)?;

        Ok(results.into_iter().map(|(attendance, employee)| AttendanceWithEmployee {
            attendance,
            employee,
        }).collect())
    }

    /// Get attendance history for an employee
    pub fn get_employee_attendance_history(&self, conn: &mut SqliteConnection, employee_id: i32, from_date: Option<NaiveDate>, to_date: Option<NaiveDate>) -> CLIERPResult<Vec<Attendance>> {
        let mut query = attendances::table
            .filter(attendances::employee_id.eq(employee_id))
            .into_boxed();

        if let Some(from) = from_date {
            query = query.filter(attendances::date.ge(from));
        }

        if let Some(to) = to_date {
            query = query.filter(attendances::date.le(to));
        }

        let attendances = query
            .order(attendances::date.desc())
            .load::<Attendance>(conn)?;

        Ok(attendances)
    }

    /// Get monthly attendance statistics for an employee
    pub fn get_monthly_stats(&self, conn: &mut SqliteConnection, employee_id: i32, year: i32, month: u32) -> CLIERPResult<AttendanceStats> {
        let start_date = NaiveDate::from_ymd_opt(year, month, 1)
            .ok_or_else(|| CLIERPError::ValidationError("Invalid date".to_string()))?;

        let end_date = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
                .ok_or_else(|| CLIERPError::ValidationError("Invalid date".to_string()))?
                .pred_opt()
                .ok_or_else(|| CLIERPError::ValidationError("Invalid date".to_string()))?
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
                .ok_or_else(|| CLIERPError::ValidationError("Invalid date".to_string()))?
                .pred_opt()
                .ok_or_else(|| CLIERPError::ValidationError("Invalid date".to_string()))?
        };

        let attendances = attendances::table
            .filter(attendances::employee_id.eq(employee_id))
            .filter(attendances::date.ge(start_date))
            .filter(attendances::date.le(end_date))
            .load::<Attendance>(conn)?;

        let total_days = attendances.len() as i32;
        let present_days = attendances.iter().filter(|a| a.status == "present" || a.status == "late").count() as i32;
        let late_days = attendances.iter().filter(|a| a.status == "late").count() as i32;
        let absent_days = attendances.iter().filter(|a| a.status == "absent").count() as i32;
        let total_overtime = attendances.iter().map(|a| a.overtime_hours).sum::<f32>();

        Ok(AttendanceStats {
            total_days,
            present_days,
            late_days,
            absent_days,
            total_overtime_hours: total_overtime,
        })
    }

    /// Mark employee as absent for a specific date
    pub fn mark_absent(&self, conn: &mut SqliteConnection, employee_id: i32, date: NaiveDate, notes: Option<String>) -> CLIERPResult<Attendance> {
        // Check if attendance already exists
        let existing = attendances::table
            .filter(attendances::employee_id.eq(employee_id))
            .filter(attendances::date.eq(date))
            .first::<Attendance>(conn)
            .optional()?;

        if let Some(attendance) = existing {
            // Update existing record
            let updated = diesel::update(attendances::table)
                .filter(attendances::id.eq(attendance.id))
                .set((
                    attendances::status.eq("absent"),
                    attendances::notes.eq(notes),
                ))
                .get_result::<Attendance>(conn)?;
            Ok(updated)
        } else {
            // Create new absence record
            let new_attendance = NewAttendance {
                employee_id,
                date,
                check_in: None,
                check_out: None,
                break_time: 0,
                overtime_hours: 0.0,
                status: "absent".to_string(),
                notes,
            };

            let attendance = diesel::insert_into(attendances::table)
                .values(&new_attendance)
                .get_result::<Attendance>(conn)?;

            Ok(attendance)
        }
    }

    /// Calculate work hours between check-in and check-out
    fn calculate_work_hours(check_in: NaiveTime, check_out: NaiveTime, break_minutes: i32) -> f32 {
        let total_minutes = (check_out.signed_duration_since(check_in)).num_minutes() - break_minutes as i64;
        total_minutes as f32 / 60.0
    }
}

impl Default for AttendanceService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceWithEmployee {
    pub attendance: Attendance,
    pub employee: Employee,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceStats {
    pub total_days: i32,
    pub present_days: i32,
    pub late_days: i32,
    pub absent_days: i32,
    pub total_overtime_hours: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckInRequest {
    pub employee_id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckOutRequest {
    pub employee_id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarkAbsentRequest {
    pub employee_id: i32,
    pub date: NaiveDate,
    pub notes: Option<String>,
}