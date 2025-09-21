use diesel::prelude::*;
use chrono::{NaiveDate, Local, Datelike};
use serde::{Deserialize, Serialize};

use crate::core::result::CLIERPResult;
use crate::core::error::CLIERPError;
use crate::database::models::{Payroll, NewPayroll, PayrollStatus, Employee, Attendance};
use crate::database::schema::{payrolls, employees, attendances};

pub struct PayrollService;

impl PayrollService {
    pub fn new() -> Self {
        Self
    }

    /// Calculate payroll for an employee for a specific period
    pub fn calculate_payroll(&self, conn: &mut SqliteConnection, employee_id: i32, period: String) -> CLIERPResult<PayrollCalculation> {
        let employee = employees::table
            .find(employee_id)
            .first::<Employee>(conn)
            .optional()?
            .ok_or_else(|| CLIERPError::NotFound("Employee not found".to_string()))?;

        // Parse period (YYYY-MM format)
        let parts: Vec<&str> = period.split('-').collect();
        if parts.len() != 2 {
            return Err(CLIERPError::ValidationError("Period must be in YYYY-MM format".to_string()));
        }

        let year: i32 = parts[0].parse()
            .map_err(|_| CLIERPError::ValidationError("Invalid year in period".to_string()))?;
        let month: u32 = parts[1].parse()
            .map_err(|_| CLIERPError::ValidationError("Invalid month in period".to_string()))?;

        if month < 1 || month > 12 {
            return Err(CLIERPError::ValidationError("Month must be between 1 and 12".to_string()));
        }

        // Get attendance data for the period
        let start_date = NaiveDate::from_ymd_opt(year, month, 1)
            .ok_or_else(|| CLIERPError::ValidationError("Invalid date".to_string()))?;

        let end_date = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
        }
        .ok_or_else(|| CLIERPError::ValidationError("Invalid date".to_string()))?
        .pred_opt()
        .ok_or_else(|| CLIERPError::ValidationError("Invalid date".to_string()))?;

        let attendances = attendances::table
            .filter(attendances::employee_id.eq(employee_id))
            .filter(attendances::date.ge(start_date))
            .filter(attendances::date.le(end_date))
            .load::<Attendance>(conn)?;

        // Calculate overtime hours
        let total_overtime_hours: f32 = attendances.iter()
            .map(|a| a.overtime_hours)
            .sum();

        // Calculate overtime pay (assuming 1.5x hourly rate)
        let daily_rate = employee.salary / 30; // Approximate daily rate
        let hourly_rate = daily_rate / 8; // 8 hours per day
        let overtime_pay = (total_overtime_hours * hourly_rate as f32 * 1.5) as i32;

        // Calculate deductions (simplified - could include tax, insurance, etc.)
        let tax_rate = 0.1; // 10% tax
        let tax_deduction = ((employee.salary + overtime_pay) as f32 * tax_rate) as i32;

        let calculation = PayrollCalculation {
            employee_id,
            employee_name: employee.name.clone(),
            period: period.clone(),
            base_salary: employee.salary,
            overtime_hours: total_overtime_hours,
            overtime_pay,
            bonuses: 0, // To be set manually if needed
            tax_deduction,
            other_deductions: 0,
            total_deductions: tax_deduction,
            gross_salary: employee.salary + overtime_pay,
            net_salary: employee.salary + overtime_pay - tax_deduction,
        };

        Ok(calculation)
    }

    /// Generate payroll record
    pub fn generate_payroll(&self, conn: &mut SqliteConnection, calculation: PayrollCalculation, bonuses: Option<i32>, additional_deductions: Option<i32>) -> CLIERPResult<Payroll> {
        // Check if payroll already exists for this period
        let existing = payrolls::table
            .filter(payrolls::employee_id.eq(calculation.employee_id))
            .filter(payrolls::period.eq(&calculation.period))
            .first::<Payroll>(conn)
            .optional()?;

        if existing.is_some() {
            return Err(CLIERPError::ValidationError(
                format!("Payroll already exists for employee {} in period {}", calculation.employee_id, calculation.period)
            ));
        }

        let final_bonuses = bonuses.unwrap_or(0);
        let final_additional_deductions = additional_deductions.unwrap_or(0);
        let total_deductions = calculation.total_deductions + final_additional_deductions;
        let net_salary = calculation.gross_salary + final_bonuses - total_deductions;

        let new_payroll = NewPayroll {
            employee_id: calculation.employee_id,
            period: calculation.period,
            base_salary: calculation.base_salary,
            overtime_pay: calculation.overtime_pay,
            bonuses: final_bonuses,
            deductions: total_deductions,
            net_salary,
            payment_date: None,
            status: PayrollStatus::Pending.to_string(),
        };

        let payroll = diesel::insert_into(payrolls::table)
            .values(&new_payroll)
            .get_result::<Payroll>(conn)?;

        Ok(payroll)
    }

    /// Process payroll (mark as processed)
    pub fn process_payroll(&self, conn: &mut SqliteConnection, payroll_id: i32) -> CLIERPResult<Payroll> {
        let payroll = diesel::update(payrolls::table)
            .filter(payrolls::id.eq(payroll_id))
            .set(payrolls::status.eq(PayrollStatus::Processed.to_string()))
            .get_result::<Payroll>(conn)?;

        Ok(payroll)
    }

    /// Pay payroll (mark as paid and set payment date)
    pub fn pay_payroll(&self, conn: &mut SqliteConnection, payroll_id: i32) -> CLIERPResult<Payroll> {
        let today = Local::now().date_naive();

        let payroll = diesel::update(payrolls::table)
            .filter(payrolls::id.eq(payroll_id))
            .set((
                payrolls::status.eq(PayrollStatus::Paid.to_string()),
                payrolls::payment_date.eq(Some(today)),
            ))
            .get_result::<Payroll>(conn)?;

        Ok(payroll)
    }

    /// Get payroll by ID
    pub fn get_payroll_by_id(&self, conn: &mut SqliteConnection, payroll_id: i32) -> CLIERPResult<Option<PayrollWithEmployee>> {
        let result = payrolls::table
            .inner_join(employees::table)
            .filter(payrolls::id.eq(payroll_id))
            .select((Payroll::as_select(), Employee::as_select()))
            .first::<(Payroll, Employee)>(conn)
            .optional()?;

        Ok(result.map(|(payroll, employee)| PayrollWithEmployee {
            payroll,
            employee,
        }))
    }

    /// Get payrolls for a specific period
    pub fn get_payrolls_by_period(&self, conn: &mut SqliteConnection, period: &str) -> CLIERPResult<Vec<PayrollWithEmployee>> {
        let results = payrolls::table
            .inner_join(employees::table)
            .filter(payrolls::period.eq(period))
            .select((Payroll::as_select(), Employee::as_select()))
            .load::<(Payroll, Employee)>(conn)?;

        Ok(results.into_iter().map(|(payroll, employee)| PayrollWithEmployee {
            payroll,
            employee,
        }).collect())
    }

    /// Get payroll history for an employee
    pub fn get_employee_payroll_history(&self, conn: &mut SqliteConnection, employee_id: i32) -> CLIERPResult<Vec<Payroll>> {
        let payrolls = payrolls::table
            .filter(payrolls::employee_id.eq(employee_id))
            .order(payrolls::period.desc())
            .load::<Payroll>(conn)?;

        Ok(payrolls)
    }

    /// Get pending payrolls
    pub fn get_pending_payrolls(&self, conn: &mut SqliteConnection) -> CLIERPResult<Vec<PayrollWithEmployee>> {
        let results = payrolls::table
            .inner_join(employees::table)
            .filter(payrolls::status.eq(PayrollStatus::Pending.to_string()))
            .select((Payroll::as_select(), Employee::as_select()))
            .load::<(Payroll, Employee)>(conn)?;

        Ok(results.into_iter().map(|(payroll, employee)| PayrollWithEmployee {
            payroll,
            employee,
        }).collect())
    }

    /// Calculate payroll for all employees in a period
    pub fn calculate_period_payrolls(&self, conn: &mut SqliteConnection, period: String) -> CLIERPResult<Vec<PayrollCalculation>> {
        let employees = employees::table
            .filter(employees::status.eq("active"))
            .load::<Employee>(conn)?;

        let mut calculations = Vec::new();

        for employee in employees {
            match self.calculate_payroll(conn, employee.id, period.clone()) {
                Ok(calculation) => calculations.push(calculation),
                Err(e) => {
                    eprintln!("Warning: Failed to calculate payroll for employee {}: {}", employee.id, e);
                }
            }
        }

        Ok(calculations)
    }

    /// Generate payslip data
    pub fn generate_payslip(&self, conn: &mut SqliteConnection, payroll_id: i32) -> CLIERPResult<Payslip> {
        let payroll_with_employee = self.get_payroll_by_id(conn, payroll_id)?
            .ok_or_else(|| CLIERPError::NotFound("Payroll not found".to_string()))?;

        let payslip = Payslip {
            payroll_id,
            employee_id: payroll_with_employee.employee.id,
            employee_name: payroll_with_employee.employee.name,
            employee_code: payroll_with_employee.employee.employee_code,
            department: "".to_string(), // TODO: Join with department
            position: payroll_with_employee.employee.position,
            period: payroll_with_employee.payroll.period,
            base_salary: payroll_with_employee.payroll.base_salary,
            overtime_pay: payroll_with_employee.payroll.overtime_pay,
            bonuses: payroll_with_employee.payroll.bonuses,
            gross_salary: payroll_with_employee.payroll.base_salary +
                         payroll_with_employee.payroll.overtime_pay +
                         payroll_with_employee.payroll.bonuses,
            deductions: payroll_with_employee.payroll.deductions,
            net_salary: payroll_with_employee.payroll.net_salary,
            payment_date: payroll_with_employee.payroll.payment_date,
            status: payroll_with_employee.payroll.status,
        };

        Ok(payslip)
    }
}

impl Default for PayrollService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollWithEmployee {
    pub payroll: Payroll,
    pub employee: Employee,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayrollCalculation {
    pub employee_id: i32,
    pub employee_name: String,
    pub period: String,
    pub base_salary: i32,
    pub overtime_hours: f32,
    pub overtime_pay: i32,
    pub bonuses: i32,
    pub tax_deduction: i32,
    pub other_deductions: i32,
    pub total_deductions: i32,
    pub gross_salary: i32,
    pub net_salary: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payslip {
    pub payroll_id: i32,
    pub employee_id: i32,
    pub employee_name: String,
    pub employee_code: String,
    pub department: String,
    pub position: String,
    pub period: String,
    pub base_salary: i32,
    pub overtime_pay: i32,
    pub bonuses: i32,
    pub gross_salary: i32,
    pub deductions: i32,
    pub net_salary: i32,
    pub payment_date: Option<NaiveDate>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneratePayrollRequest {
    pub employee_id: i32,
    pub period: String,
    pub bonuses: Option<i32>,
    pub additional_deductions: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessPayrollRequest {
    pub payroll_id: i32,
}