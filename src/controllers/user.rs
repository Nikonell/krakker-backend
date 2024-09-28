use std::fs::{self, File};
use std::io;

use actix_files::NamedFile;
use actix_multipart::form::MultipartForm;
use actix_web::{
    http::header::ContentType,
    web::{Json, Path, Query, ReqData},
    HttpRequest, HttpResponse, Responder,
};

use apistos::api_operation;

use crate::{
    models::user::{ChangeAvatar, SelectUser, SelectUserQuery},
    services::user::{get_all_users, get_user},
    utils::response::{ErrorResponse, SuccessResponse},
};

const AVATARS_PATH_DIR: &str = "/uploads/avatars";
const DEFAULT_AVATAR: &[u8] = include_bytes!("../../static/default_avatar.png");

#[api_operation(
    summary = "Get me",
    description = "Get authenticated user information",
    tag = "Users",
    error_code = "401",
    error_code = "404"
)]
pub async fn get_me(user_id: ReqData<u64>) -> Result<Json<SuccessResponse<SelectUser>>, ErrorResponse> {
    let user = get_user(*user_id)
        .await
        .map_err(|e| ErrorResponse::InternalServerError(e.to_string()))?
        .ok_or_else(|| ErrorResponse::NotFound("User not found".to_string()))?;
    Ok(Json(SuccessResponse::new(user)))
}

#[api_operation(
    summary = "Get all users",
    description = "Get all users",
    tag = "Users",
    error_code = "401",
    error_code = "404"
)]
pub async fn get_all(query: Query<SelectUserQuery>) -> Result<Json<SuccessResponse<Vec<SelectUser>>>, ErrorResponse> {
    let users = get_all_users(query.into_inner())
        .await
        .map_err(|e| ErrorResponse::InternalServerError(e.to_string()))?;
    Ok(Json(SuccessResponse::new(users)))
}

pub async fn get_avatar(req: HttpRequest, user_id: Path<u64>) -> impl Responder {
    let path = format!("{}/{}.png", AVATARS_PATH_DIR, user_id);

    match NamedFile::open(&path) {
        Ok(file) => file.use_last_modified(true).into_response(&req),
        Err(_) => HttpResponse::Ok()
            .content_type(ContentType::png())
            .body(DEFAULT_AVATAR),
    }
}

pub async fn change_avatar(
    user_id: ReqData<u64>,
    body: MultipartForm<ChangeAvatar>,
) -> Result<Json<SuccessResponse<()>>, ErrorResponse> {
    let path = format!("{}/{}.png", AVATARS_PATH_DIR, *user_id);

    fs::create_dir_all(AVATARS_PATH_DIR).map_err(internal_server_error)?;

    let mut file = File::create(path).map_err(internal_server_error)?;
    let mut temp_file = body.file.file.as_file();

    io::copy(&mut temp_file, &mut file).map_err(internal_server_error)?;

    Ok(Json(SuccessResponse::new(())))
}

fn internal_server_error<E: std::error::Error>(e: E) -> ErrorResponse {
    ErrorResponse::InternalServerError(e.to_string())
}
