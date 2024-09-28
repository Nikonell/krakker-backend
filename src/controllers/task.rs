use actix_web::web::{Data, Json, Path, Query, ReqData};
use apistos::api_operation;
use garde::Validate;

use crate::{
    models::{
        task::{CreateTaskRequest, SelectTask, SelectTaskRequest, UpdateTaskRequest}, user::SelectUser},
    services::{notifications::create_notification, task::{add_assigned_user, get_task_by_id, get_user_tasks, remove_assigned_user}},
    utils::{app_data::AppData, response::{ErrorResponse, SuccessResponse}}
};

#[api_operation(
    summary = "Get my tasks",
    description = "Get all tasks assigned to the current user",
    tag = "Tasks",
    error_code = "401"
)]
pub async fn get_my(user_id: ReqData<u64>, filters: Query<SelectTaskRequest>) -> Result<Json<SuccessResponse<Vec<SelectTask>>>, ErrorResponse> {
    let projects = get_user_tasks(*user_id, &*filters).await
        .map_err(|e| ErrorResponse::InternalServerError(e.to_string()))?;

    Ok(Json(SuccessResponse::new(projects)))
}

#[api_operation(
    summary = "Get task by id",
    description = "Get a task by its id",
    tag = "Tasks",
    error_code = "401",
    error_code = "404"
)]
pub async fn get_by_id(_user_id: ReqData<u64>, task_id: Path<u64>) -> Result<Json<SuccessResponse<SelectTask>>, ErrorResponse> {
    let task = get_task_by_id(*task_id).await
        .map_err(|e| ErrorResponse::InternalServerError(e.to_string()))?
        .ok_or_else(|| ErrorResponse::NotFound("Task not found".to_string()))?;

    Ok(Json(SuccessResponse::new(task)))
}

#[api_operation(
    summary = "Create task",
    description = "Create a new task",
    tag = "Tasks",
    error_code = "400",
    error_code = "401"
)]
pub async fn create_task(
    app_data: Data<AppData>,
    user_id: ReqData<u64>,
    task: Json<CreateTaskRequest>,
) -> Result<Json<SuccessResponse<SelectTask>>, ErrorResponse> {
    task.validate()
        .map_err(|error| ErrorResponse::BadRequest(error.to_string()))?;

    let task = crate::services::task::create_task(*user_id, &*task)
        .await
        .map_err(|e| ErrorResponse::InternalServerError(e.to_string()))?;

    for user in &task.attached_to {
        create_notification(
            "Вас назначили на задачу".to_string(),
            format!("Вы были назначены на задачу {}. Думаю, вам стоит проверить ваш личный кабинет", task.name).to_string(),
            user.id,
            &app_data.mailer
        ).await;
    }

    Ok(Json(SuccessResponse::new(task)))
}

#[api_operation(
    summary = "Update task",
    description = "Update task by id",
    tag = "Tasks",
    error_code = "400",
    error_code = "401",
    error_code = "404"
)]
pub async fn update_task(user_id: ReqData<u64>, task_id: Path<u64>, task: Json<UpdateTaskRequest>) -> Result<Json<SuccessResponse<SelectTask>>, ErrorResponse> {
    task.validate().map_err(|error| ErrorResponse::BadRequest(error.to_string()))?;

    let task = crate::services::task::update_task(*user_id, *task_id, &*task).await
        .map_err(|e| ErrorResponse::InternalServerError(e.to_string()))?
        .ok_or_else(|| ErrorResponse::NotFound("Task not found".to_string()))?;

    Ok(Json(SuccessResponse::new(task)))
}

#[api_operation(
    summary = "Delete task",
    description = "Delete task by id",
    tag = "Tasks",
    error_code = "401",
    error_code = "404"
)]
pub async fn delete_task(user_id: ReqData<u64>, task_id: Path<u64>) -> Result<Json<SuccessResponse<()>>, ErrorResponse> {
    crate::services::task::delete_task(*user_id, *task_id).await
        .map_err(|e| ErrorResponse::InternalServerError(e.to_string()))?;

    Ok(Json(SuccessResponse::new(())))
}

#[api_operation(
    summary = "Add assignee",
    description = "Add an assignee from a task",
    tag = "Tasks",
    error_code = "401",
    error_code = "404"
)]
pub async fn add_assignee(
    app_data: Data<AppData>,
    user_id: ReqData<u64>,
    path: Path<(u64, u64)>
) -> Result<Json<SuccessResponse<Vec<SelectUser>>>, ErrorResponse> {
    let users = add_assigned_user(*user_id, path.0, path.1).await
        .map_err(|e| ErrorResponse::InternalServerError(e.to_string()))?
        .ok_or_else(|| ErrorResponse::NotFound("Task not found".to_string()))?;

    if let Ok(Some(task)) = get_task_by_id(path.0).await {
        create_notification(
            "Вас назначили на задачу".to_string(),
            format!("Вы были назначены на задачу {}. Думаю, вам стоит проверить ваш личный кабинет", task.name).to_string(),
            path.1,
            &app_data.mailer
        ).await;
    }

    Ok(Json(SuccessResponse::new(users)))
}

#[api_operation(
    summary = "Remove assignee",
    description = "Remove an assignee from a task",
    tag = "Tasks",
    error_code = "401",
    error_code = "404"
)]
pub async fn remove_assignee(
    app_data: Data<AppData>,
    user_id: ReqData<u64>,
    path: Path<(u64, u64)>
) -> Result<Json<SuccessResponse<Vec<SelectUser>>>, ErrorResponse> {
    let users = remove_assigned_user(*user_id, path.0, path.1).await
        .map_err(|e| ErrorResponse::InternalServerError(e.to_string()))?
        .ok_or_else(|| ErrorResponse::NotFound("Task not found".to_string()))?;

    if let Ok(Some(task)) = get_task_by_id(path.0).await {
        create_notification(
            "Удаление с задачи".to_string(),
            format!("Вас удалили с задачи {}. Вы можете расслабиться.", task.name).to_string(),
            path.1,
            &app_data.mailer
        ).await;
    }

    Ok(Json(SuccessResponse::new(users)))
}
