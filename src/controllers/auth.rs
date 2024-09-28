use actix_web::web::{Json, Data};
use apistos::api_operation;
use garde::Validate;
use jsonwebtoken::{encode, EncodingKey, Header};

use crate::{
    config::Config,
    models::auth::{AuthResponse, JWTClaims, LoginRequest, RegisterRequest},
    services::{notifications::create_notification, user::get_user_id_by_credentials},
    utils::{app_data::AppData, response::{ErrorResponse, SuccessResponse}}
};

#[api_operation(
    summary = "Login",
    description = "Login to the system",
    tag = "Auth",
    error_code = "403"
)]
pub async fn login(app_data: Data<AppData>, body: Json<LoginRequest>) -> Result<Json<SuccessResponse<AuthResponse>>, ErrorResponse> {
    let user_id = get_user_id_by_credentials(&body.username, &body.password).await
        .map_err(|e| ErrorResponse::InternalServerError(e.to_string()))?
        .ok_or_else(|| ErrorResponse::NotFound("Invalid credentials provided".to_string()))?;

    let token = make_token(user_id)?;

    create_notification(
        "Кто-то вошел в аккаунт".to_string(),
        "Здравствуйте! В ваш аккаунт только что был произведен вход. Если это были не вы, немедленно свяжитесь с нашей службой поддержки для обеспечения безопасности вашего аккаунта.".to_string(),
        user_id,
        &app_data.mailer
    ).await;

    Ok(Json(SuccessResponse::new(AuthResponse { token })))
}

#[api_operation(
    summary = "Register",
    description = "Register a new user",
    tag = "Auth",
    error_code = "404"
)]
pub async fn register(app_data: Data<AppData>, body: Json<RegisterRequest>) -> Result<Json<SuccessResponse<AuthResponse>>, ErrorResponse> {
    body.validate().map_err(|error| ErrorResponse::BadRequest(error.to_string()))?;

    let user_id = crate::services::user::create_user(&body).await
        .map_err(|e| ErrorResponse::InternalServerError(e.to_string()))?;

    let token = make_token(user_id)?;

    create_notification(
        "Добро пожаловать в Krakker".to_string(),
        "Здравствуйте! Благодарим вас за регистрацию на сервисе Krakker. Если у вас возникнут какие-либо вопросы или проблемы, пожалуйста, свяжитесь с нашей службой поддержки. Желаем вам приятного использования нашего сервиса!".to_string(),
        user_id,
        &app_data.mailer
    ).await;

    Ok(Json(SuccessResponse::new(AuthResponse { token })))
}

fn make_token(id: u64) -> Result<String, ErrorResponse> {
    let now = chrono::Utc::now();
    let claims = JWTClaims {
        sub: id,
        exp: (now + chrono::Duration::days(30)).timestamp() as usize,
        nbf: now.timestamp() as usize,
    };

    let jwt_secret = Config::get_env_param("JWT_SECRET");
    encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret.as_ref()))
        .map_err(|e| ErrorResponse::InternalServerError(e.to_string()))
}
