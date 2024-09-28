use std::option::Option;

use reqwest::StatusCode;
use tera::Tera;

use crate::mailer::models::*;
use crate::mailer::template::MESSAGE_TEMPLATE;

#[derive(Clone)]
pub struct Mailer {
    api_key: String,
    sender_name: String,
    sender_email: String,
    lang: String,
    main_list: Option<EmailList>,
}

/* Example of usage:
#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Trace).unwrap();
    let config = Config::from_env();
    let mut mailer = Mailer::new(
        &config.unisender_api_key,
        &config.unisender_sender_name,
        &config.unisender_sender_email,
        "ru",
    );
    mailer.initialize_mail_list().await;
    mailer
        .send_email_message(
            "noreply@kkracker.org",
            "Kuzbass",
            "Kuzbass",
            "<b>Hi from Kuzbass</b>",
        )
        .await
        .unwrap();
}
*/

impl Mailer {
    const UNISENDER_API_URL: &'static str = "https://api.unisender.com/ru/api";
    const UNISENDER_DEFAULT_LIST_NAME: &'static str = "main";

    pub fn new(api_key: &str, sender_name: &str, sender_email: &str, lang: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            sender_name: sender_name.to_string(),
            sender_email: sender_email.to_string(),
            lang: lang.to_string(),
            main_list: None,
        }
    }

    pub async fn send_email(
        &self,
        target_email: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), String> {
        let list = match self.main_list.clone() {
            Some(id) => id,
            None => {
                log::error!(
                    target: "Mailer",
                    "Create main email list before call the method! (use .initialize_mail_list())"
                );
                // std::process::exit(1);
                return Err("Create main email list before call the method! (use .initialize_mail_list())".to_string());
            }
        };
        let list_id = &list.id.to_string();
        let query_params: Vec<(&str, &str)> = vec![
            ("api_key", &self.api_key),
            ("format", "json"),
            ("sender_name", &self.sender_name),
            ("sender_email", &self.sender_email),
            ("list_id", list_id),
            ("subject", subject),
            ("body", body),
            ("email", target_email),
            ("lang", &self.lang),
        ];
        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/sendEmail", Self::UNISENDER_API_URL))
            .query(&query_params)
            .send()
            .await;
        match response {
            Ok(resp) => {
                if resp.status() == StatusCode::OK {
                    let response_text = resp.text().await.map_err(|e| e.to_string())?;
                    log::info!(target: "Mailer", "Trying to parse unisend response: {}", response_text);
                    let _: NewEmailSentResponse =
                        serde_json::from_str(&response_text).map_err(|e| e.to_string())?;
                    Ok(())
                } else {
                    Err(format!(
                        "Unisender unexpected status code: {}",
                        resp.status()
                    ))
                }
            }
            Err(err) => Err(err.to_string()),
        }
    }

    pub async fn get_lists(&self) -> Result<Vec<EmailList>, String> {
        let query_params = [("format", "json"), ("api_key", self.api_key.as_str())];
        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/getLists", Self::UNISENDER_API_URL))
            .query(&query_params)
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status() == StatusCode::OK {
                    let response_text = resp.text().await.map_err(|e| e.to_string())?;
                    let lists: EmailListResponse =
                        serde_json::from_str(&response_text).map_err(|e| e.to_string())?;
                    Ok(lists.result)
                } else {
                    Err(format!(
                        "Unisender unexpected status code: {}",
                        resp.status()
                    ))
                }
            }
            Err(err) => Err(err.to_string()),
        }
    }

    pub async fn create_email_list(&self, title: &str) -> Result<u64, String> {
        // Find or create main email list
        let query_params = [
            ("format", "json"),
            ("api_key", &self.api_key),
            ("title", title),
        ];
        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/createList", Self::UNISENDER_API_URL))
            .query(&query_params)
            .send()
            .await;
        match response {
            Ok(resp) => {
                if resp.status() == StatusCode::OK {
                    let response_text = resp.text().await.map_err(|e| e.to_string())?;
                    let list: NewEmailListResponse =
                        serde_json::from_str(&response_text).map_err(|e| e.to_string())?;
                    Ok(list.result.id)
                } else {
                    Err(format!(
                        "Unisender unexpected status code: {}",
                        resp.status()
                    ))
                }
            }
            Err(err) => Err(err.to_string()),
        }
    }

    pub async fn initialize_mail_list(&mut self) {
        let email_lists = match self.get_lists().await {
            Ok(lists) => lists,
            Err(err) => {
                log::error!(target: "Mailer", "Error occured while getting email lists: {err}");
                // std::process::exit(1);
                return;
            }
        };
        for list in &email_lists {
            if list.title == Self::UNISENDER_DEFAULT_LIST_NAME {
                log::info!(target: "Mailer", "{} email list already exists", list.title);
                self.main_list = Option::from(list.clone());
                return;
            }
        }
        match self
            .create_email_list(Self::UNISENDER_DEFAULT_LIST_NAME)
            .await
        {
            Ok(new_list) => {
                let main_list = EmailList {
                    title: Self::UNISENDER_DEFAULT_LIST_NAME.to_string(),
                    id: new_list,
                };
                self.main_list = Option::from(main_list);
            }
            Err(err) => {
                log::error!(target: "Mailer", "Error occurred while creating main email list: {}", err);
                // std::process::exit(1);
                return;
            }
        };
    }

    pub async fn send_email_message(
        &self,
        target_email: &str,
        subject: &str,
        message_title: &str,
        message_text: &str,
    ) -> Result<(), String> {
        let mut tera = Tera::default();
        tera.add_raw_template("email", MESSAGE_TEMPLATE).unwrap();
        let mut context = tera::Context::new();
        context.insert("message_title", message_title);
        context.insert("message_text", message_text);

        let message = tera.render("email", &context).unwrap();

        match self.send_email(target_email, subject, &message).await {
            Ok(_) => {
                log::info!(target: "Mailer", "Notification to {target_email} sent");
                Ok(())
            }
            Err(err) => {
                log::error!(target: "Mailer", "Failed to send notification to {target_email}");
                Err(err)
            }
        }
    }
}
