use std::vec;

use crate::mailer::mailer::Mailer;
use crate::models::notification::SelectNotification;
use crate::prisma::{notification, user};
use crate::prisma::notification::Data;
use crate::services::common::create_prisma_client;

const LOG_TAG: &'static str = "NotificationsService";

pub fn notification_to_response(notification: &Data) -> SelectNotification {
    SelectNotification {
        id: notification.id as u64,
        created_at: notification.created_at.timestamp() as u64,
        title: notification.title.clone(),
        description: notification.description.clone(),
    }
}

pub async fn get_user_notifications(user_id: u64) -> Result<Vec<SelectNotification>, String> {
    let client = create_prisma_client().await?;
    let notifications = client
        .notification()
        .find_many(vec![notification::user_id::equals(user_id as i32)])
        .exec()
        .await;

    match notifications {
        Ok(notifications) => {
            let notifications = notifications
                .into_iter()
                .map(|notification| notification_to_response(&notification))
                .collect();
            Ok(notifications)
        }
        Err(err) => Err(format!("Failed to get notifications {err}").to_string()),
    }
}

pub async fn create_notification(
    title: String,
    description: String,
    user_id: u64,
    mailer: &Mailer,
) {
    let client = match create_prisma_client().await {
        Ok(client) => client,
        Err(err) => {
            log::error!(target: LOG_TAG, "Failed to create prisma client {err}");
            return;
        }
    };

    let notification = client
        .notification()
        .create(
            title.clone(),
            description.clone(),
            user::id::equals(user_id as i32),
            vec![],
        )
        .exec()
        .await;
    match notification {
        Ok(notification) => notification_to_response(&notification),
        Err(err) => {
            log::error!(target: LOG_TAG, "Failed to create notification {err}");
            return;
        }
    };
    match send_email_notification(title, description, user_id, mailer).await {
        Ok(email) => {
            log::info!(target: LOG_TAG, "Email notification sent to ({user_id}) {email}");
        }
        Err(err) => {
            log::error!(target: LOG_TAG, "Failed to send email notification to: {user_id}. Details: {err}");
        }
    }
}

pub async fn send_email_notification(
    title: String,
    description: String,
    user_id: u64,
    mailer: &Mailer,
) -> Result<String, String> {
    let user = create_prisma_client()
        .await?
        .user()
        .find_unique(user::id::equals(user_id as i32))
        .exec()
        .await;
    match user {
        Ok(Some(found_user)) => {
            mailer
                .send_email_message(&found_user.email, &title, &title, &description)
                .await?;
            Ok(found_user.email)
        }
        Ok(None) => {
            return Err("User not found".to_string());
        }
        Err(err) => {
            return Err(format!("Failed to get user {err}").to_string());
        }
    }
}
