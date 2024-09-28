use actix_web::web::{Json, ReqData};
use apistos::api_operation;

use crate::{models::notification::SelectNotification, services::notifications::get_user_notifications, utils::response::{ErrorResponse, SuccessResponse}};

#[api_operation(
    summary = "Get my notifications",
    description = "Get all notifications of the current user",
    tag = "Notifications",
    error_code = "401"
)]
pub async fn get_my(user_id: ReqData<u64>) -> Result<Json<SuccessResponse<Vec<SelectNotification>>>, ErrorResponse> {
    let notifications = get_user_notifications(*user_id).await
        .map_err(|e| ErrorResponse::InternalServerError(e.to_string()))?;

    Ok(Json(SuccessResponse::new(notifications)))
}
