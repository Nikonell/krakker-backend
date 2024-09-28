use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct EmailList {
    pub id: u64,
    pub title: String,
}

#[derive(Deserialize, Debug)]
pub struct EmailListResponse {
    pub result: Vec<EmailList>,
}

#[derive(Deserialize, Debug)]
pub struct NewEmailListResult {
    pub id: u64,
}

#[derive(Deserialize, Debug)]
pub struct NewEmailMessageResult {
    #[allow(unused)]
    pub email_id: String,
}

#[derive(Deserialize, Debug)]
pub struct NewEmailListResponse {
    pub result: NewEmailListResult,
}

#[derive(Deserialize, Debug)]
pub struct NewEmailSentResponse {
    #[allow(unused)]
    pub result: NewEmailMessageResult,
}
