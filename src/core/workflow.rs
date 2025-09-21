use crate::core::{auth::AuthenticatedUser, error::CLIERPError, result::CLIERPResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a workflow step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub name: String,
    pub description: String,
    pub required_role: Option<String>,
    pub auto_execute: bool,
}

/// Represents a workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
    pub current_step: usize,
    pub status: WorkflowStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Draft,
    Active,
    Paused,
    Completed,
    Failed,
}

/// Workflow execution context
#[derive(Debug)]
pub struct WorkflowContext {
    pub user: Option<AuthenticatedUser>,
    pub data: HashMap<String, serde_json::Value>,
}

/// Trait for workflow actions
pub trait WorkflowAction {
    fn execute(&self, context: &mut WorkflowContext) -> CLIERPResult<()>;
    fn can_execute(&self, context: &WorkflowContext) -> bool;
    fn name(&self) -> &str;
}

/// Workflow engine for managing and executing workflows
pub struct WorkflowEngine {
    workflows: HashMap<String, Workflow>,
    actions: HashMap<String, Box<dyn WorkflowAction>>,
}

impl WorkflowEngine {
    pub fn new() -> Self {
        Self {
            workflows: HashMap::new(),
            actions: HashMap::new(),
        }
    }

    /// Register a workflow
    pub fn register_workflow(&mut self, workflow: Workflow) {
        self.workflows.insert(workflow.id.clone(), workflow);
    }

    /// Register a workflow action
    pub fn register_action<A: WorkflowAction + 'static>(&mut self, action: A) {
        self.actions
            .insert(action.name().to_string(), Box::new(action));
    }

    /// Start a workflow
    pub fn start_workflow(
        &mut self,
        workflow_id: &str,
        _context: WorkflowContext,
    ) -> CLIERPResult<String> {
        let workflow = self.workflows.get_mut(workflow_id).ok_or_else(|| {
            CLIERPError::NotFound(format!("Workflow '{}' not found", workflow_id))
        })?;

        workflow.status = WorkflowStatus::Active;
        workflow.current_step = 0;

        tracing::info!("Started workflow: {} ({})", workflow.name, workflow.id);
        Ok(workflow.id.clone())
    }

    /// Execute the next step in a workflow
    pub fn execute_next_step(
        &mut self,
        workflow_id: &str,
        context: &mut WorkflowContext,
    ) -> CLIERPResult<bool> {
        let workflow = self.workflows.get_mut(workflow_id).ok_or_else(|| {
            CLIERPError::NotFound(format!("Workflow '{}' not found", workflow_id))
        })?;

        if workflow.current_step >= workflow.steps.len() {
            workflow.status = WorkflowStatus::Completed;
            tracing::info!("Workflow completed: {} ({})", workflow.name, workflow.id);
            return Ok(false); // No more steps
        }

        let step = &workflow.steps[workflow.current_step];

        // Check if user has required role for this step
        if let Some(required_role) = &step.required_role {
            if let Some(user) = &context.user {
                if user.role.to_string() != *required_role {
                    return Err(CLIERPError::Authorization(format!(
                        "Step '{}' requires role: {}",
                        step.name, required_role
                    )));
                }
            } else {
                return Err(CLIERPError::Authentication(
                    "Authentication required for this step".to_string(),
                ));
            }
        }

        // Execute step action if available
        if let Some(action) = self.actions.get(&step.id) {
            if action.can_execute(context) {
                action.execute(context)?;
                tracing::info!("Executed workflow step: {} in {}", step.name, workflow.name);
            } else {
                return Err(CLIERPError::Internal(format!(
                    "Cannot execute step '{}' at this time",
                    step.name
                )));
            }
        }

        workflow.current_step += 1;
        Ok(true) // More steps available
    }

    /// Get workflow status
    pub fn get_workflow_status(&self, workflow_id: &str) -> CLIERPResult<&Workflow> {
        self.workflows
            .get(workflow_id)
            .ok_or_else(|| CLIERPError::NotFound(format!("Workflow '{}' not found", workflow_id)))
    }

    /// List all workflows
    pub fn list_workflows(&self) -> Vec<&Workflow> {
        self.workflows.values().collect()
    }

    /// Pause a workflow
    pub fn pause_workflow(&mut self, workflow_id: &str) -> CLIERPResult<()> {
        let workflow = self.workflows.get_mut(workflow_id).ok_or_else(|| {
            CLIERPError::NotFound(format!("Workflow '{}' not found", workflow_id))
        })?;

        workflow.status = WorkflowStatus::Paused;
        tracing::info!("Paused workflow: {} ({})", workflow.name, workflow.id);
        Ok(())
    }

    /// Resume a workflow
    pub fn resume_workflow(&mut self, workflow_id: &str) -> CLIERPResult<()> {
        let workflow = self.workflows.get_mut(workflow_id).ok_or_else(|| {
            CLIERPError::NotFound(format!("Workflow '{}' not found", workflow_id))
        })?;

        workflow.status = WorkflowStatus::Active;
        tracing::info!("Resumed workflow: {} ({})", workflow.name, workflow.id);
        Ok(())
    }
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Create default workflows for common ERP processes
pub fn create_default_workflows() -> Vec<Workflow> {
    vec![
        Workflow {
            id: "employee_onboarding".to_string(),
            name: "Employee Onboarding".to_string(),
            description: "Standard employee onboarding process".to_string(),
            steps: vec![
                WorkflowStep {
                    id: "create_employee_record".to_string(),
                    name: "Create Employee Record".to_string(),
                    description: "Create basic employee information".to_string(),
                    required_role: Some("manager".to_string()),
                    auto_execute: false,
                },
                WorkflowStep {
                    id: "assign_department".to_string(),
                    name: "Assign Department".to_string(),
                    description: "Assign employee to department".to_string(),
                    required_role: Some("manager".to_string()),
                    auto_execute: false,
                },
                WorkflowStep {
                    id: "create_user_account".to_string(),
                    name: "Create User Account".to_string(),
                    description: "Create system user account".to_string(),
                    required_role: Some("admin".to_string()),
                    auto_execute: false,
                },
                WorkflowStep {
                    id: "setup_payroll".to_string(),
                    name: "Setup Payroll".to_string(),
                    description: "Configure payroll settings".to_string(),
                    required_role: Some("manager".to_string()),
                    auto_execute: false,
                },
            ],
            current_step: 0,
            status: WorkflowStatus::Draft,
        },
        Workflow {
            id: "monthly_payroll".to_string(),
            name: "Monthly Payroll Processing".to_string(),
            description: "Monthly payroll calculation and processing".to_string(),
            steps: vec![
                WorkflowStep {
                    id: "calculate_attendance".to_string(),
                    name: "Calculate Attendance".to_string(),
                    description: "Calculate monthly attendance for all employees".to_string(),
                    required_role: Some("manager".to_string()),
                    auto_execute: true,
                },
                WorkflowStep {
                    id: "calculate_payroll".to_string(),
                    name: "Calculate Payroll".to_string(),
                    description: "Calculate monthly payroll".to_string(),
                    required_role: Some("manager".to_string()),
                    auto_execute: true,
                },
                WorkflowStep {
                    id: "review_payroll".to_string(),
                    name: "Review Payroll".to_string(),
                    description: "Review and approve payroll calculations".to_string(),
                    required_role: Some("admin".to_string()),
                    auto_execute: false,
                },
                WorkflowStep {
                    id: "process_payments".to_string(),
                    name: "Process Payments".to_string(),
                    description: "Process salary payments".to_string(),
                    required_role: Some("admin".to_string()),
                    auto_execute: false,
                },
            ],
            current_step: 0,
            status: WorkflowStatus::Draft,
        },
    ]
}
