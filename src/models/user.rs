use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use apistos::ApiComponent;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema, ApiComponent)]
pub struct SelectUser {
    pub(crate) id: u64,
    pub(crate) created_at: u64,
    pub(crate) last_seen: u64,
    pub(crate) email: String,
    pub(crate) username: String,
    pub(crate) first_name: String,
    pub(crate) last_name: String
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, ApiComponent)]
pub struct SelectUserQuery {
    pub(crate) project_id: Option<u64>,
    pub(crate) username: Option<String>,
}

#[derive(MultipartForm)]
pub struct ChangeAvatar {
    #[multipart(limit = "2mb")]
    pub(crate) file: TempFile
}
