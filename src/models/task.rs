use apistos::ApiComponent;
use garde::Validate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;
use super::user::SelectUser;

#[derive(Debug, Serialize, Deserialize, Clone, strum_macros::Display, JsonSchema, ApiComponent, EnumString, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum TaskStatus {
    Todo,
    InProgress,
    InReview,
    Done,
    Blocked,
    Cancelled
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, ApiComponent)]
pub struct SelectTask {
    pub id: u64,
    pub created_at: u64,
    pub name: String,
    pub description: String,
    pub attached_to: Vec<SelectUser>,
    pub status: TaskStatus,
    pub due_date: Option<u64>,
    pub assigned_issue: Option<u64>
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, ApiComponent)]
pub struct SelectTaskRequest {
    pub project_id: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, ApiComponent, Validate)]
pub struct CreateTaskRequest {
    #[garde(length(min = 3, max = 255))]
    #[schemars(length(min = 3, max = 255))]
    pub name: String,
    #[garde(length(min = 0, max = 1024))]
    #[schemars(length(min = 0, max = 1024))]
    pub description: String,
    #[garde(skip)]
    pub project_id: u64,
    #[garde(skip)]
    pub attached_to: Vec<u64>,
    #[garde(skip)]
    pub due_date: Option<u64>,
    #[garde(skip)]
    pub assigned_issue: Option<u64>
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, ApiComponent, Validate)]
pub struct UpdateTaskRequest {
    #[garde(length(min = 3, max = 255))]
    #[schemars(length(min = 3, max = 255))]
    pub name: Option<String>,
    #[garde(length(min = 0, max = 1024))]
    #[schemars(length(min = 0, max = 1024))]
    pub description: Option<String>,
    #[garde(skip)]
    pub status: Option<TaskStatus>,
    #[garde(skip)]
    pub due_date: Option<u64>,
    #[garde(skip)]
    pub assigned_issue: Option<u64>
}
