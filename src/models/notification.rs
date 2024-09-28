use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema, ApiComponent)]
pub struct SelectNotification {
    pub(crate) id: u64,
    pub(crate) created_at: u64,
    pub(crate) title: String,
    pub(crate) description: String,
}
