use apistos::ApiComponent;
use garde::Validate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{task::SelectTask, user::SelectUser};

#[derive(Debug, Serialize, Deserialize, JsonSchema, ApiComponent)]
pub struct SelectProject {
    pub(crate) id: u64,
    pub(crate) created_at: u64,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) owner: SelectUser,
    pub(crate) members: Vec<SelectUser>,
    pub(crate) tasks: Vec<SelectTask>,
    pub(crate) repository_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, ApiComponent, Validate)]
pub struct CreateProjectRequest {
    #[garde(length(min = 1, max = 256))]
    #[schemars(length(min = 1, max = 256))]
    pub name: String,
    #[garde(length(min = 1, max = 1024))]
    #[schemars(length(min = 1, max = 1024))]
    pub description: String,
    #[garde(skip)]
    pub members: Vec<u64>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, ApiComponent, Validate, Clone)]
pub struct UpdateProjectRequest {
    #[garde(length(min = 1, max = 256))]
    #[schemars(length(min = 1, max = 256))]
    pub name: Option<String>,
    #[garde(length(min = 1, max = 1024))]
    #[schemars(length(min = 1, max = 1024))]
    pub description: Option<String>,
    #[garde(skip)]
    pub owner_id: Option<u64>,
    #[garde(skip)]
    pub repository_id: Option<String>,
}
