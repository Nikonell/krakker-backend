use actix_web::web::{Data, Json, Path, ReqData};
use apistos::api_operation;
use garde::Validate;

use crate::{
    models::{
        project::{
            CreateProjectRequest,
            SelectProject,
            UpdateProjectRequest
        }, user::SelectUser
    },
    services::{notifications::create_notification, project::{
        add_project_member,
        get_project_by_id,
        get_user_projects,
        remove_project_member
    }},
    utils::{app_data::AppData, response::{ErrorResponse, SuccessResponse}}
};

#[api_operation(
    summary = "Get my projects",
    description = "Get all projects of the current user",
    tag = "Projects",
    error_code = "401"
)]
pub async fn get_my(user_id: ReqData<u64>) -> Result<Json<SuccessResponse<Vec<SelectProject>>>, ErrorResponse> {
    let projects = get_user_projects(*user_id).await
        .map_err(|e| ErrorResponse::InternalServerError(e.to_string()))?;

    Ok(Json(SuccessResponse::new(projects)))
}

#[api_operation(
    summary = "Get project by id",
    description = "Get project by id",
    tag = "Projects",
    error_code = "401",
    error_code = "404"
)]
pub async fn get_by_id(owner_id: ReqData<u64>, project_id: Path<u64>) -> Result<Json<SuccessResponse<SelectProject>>, ErrorResponse> {
    let project = get_project_by_id(*owner_id, *project_id).await
        .map_err(|e| ErrorResponse::InternalServerError(e.to_string()))?
        .ok_or_else(|| ErrorResponse::NotFound("Project not found".to_string()))?;

    Ok(Json(SuccessResponse::new(project)))
}

#[api_operation(
    summary = "Create project",
    description = "Create a new project",
    tag = "Projects",
    error_code = "400",
    error_code = "401"
)]
pub async fn create_project(
    app_data: Data<AppData>,
    owner_id: ReqData<u64>,
    body: Json<CreateProjectRequest>
) -> Result<Json<SuccessResponse<SelectProject>>, ErrorResponse> {
    body.validate().map_err(|error| ErrorResponse::BadRequest(error.to_string()))?;

    let project = crate::services::project::create_project(*owner_id, &*body).await
        .map_err(|e| ErrorResponse::InternalServerError(e.to_string()))?;

    create_notification(
        "Новый проект".to_string(),
        format!("Уважаемый пользователь, вы только что создали новый проект {} на сервисе Krakker.", project.name).to_string(),
        project.owner.id,
        &app_data.mailer
    ).await;

    Ok(Json(SuccessResponse::new(project)))
}

#[api_operation(
    summary = "Update project",
    description = "Update project by id",
    tag = "Projects",
    error_code = "400",
    error_code = "401",
    error_code = "404"
)]
pub async fn update_project(owner_id: ReqData<u64>, project_id: Path<u64>, body: Json<UpdateProjectRequest>) -> Result<Json<SuccessResponse<SelectProject>>, ErrorResponse> {
    body.validate().map_err(|error| ErrorResponse::BadRequest(error.to_string()))?;

    let project = crate::services::project::update_project(*owner_id, *project_id, &*body).await
        .map_err(|e| ErrorResponse::InternalServerError(e.to_string()))?
        .ok_or_else(|| ErrorResponse::NotFound("Project not found".to_string()))?;

    Ok(Json(SuccessResponse::new(project)))
}

#[api_operation(
    summary = "Delete project",
    description = "Delete project by id",
    tag = "Projects",
    error_code = "401",
    error_code = "404"
)]
pub async fn delete_project(_app_data: Data<AppData>, owner_id: ReqData<u64>, project_id: Path<u64>) -> Result<Json<SuccessResponse<()>>, ErrorResponse> {
    crate::services::project::delete_project(*owner_id, *project_id).await
        .map_err(|e| ErrorResponse::InternalServerError(e.to_string()))?;

    Ok(Json(SuccessResponse::new(())))
}

#[api_operation(
    summary = "Add project member",
    description = "Add member to project",
    tag = "Projects",
    error_code = "401",
    error_code = "404"
)]
pub async fn add_member(
    app_data: Data<AppData>,
    owner_id: ReqData<u64>,
    path: Path<(u64, u64)>
) -> Result<Json<SuccessResponse<Vec<SelectUser>>>, ErrorResponse> {
    let (project_id, member_id) = path.into_inner();

    if *owner_id == member_id {
        return Err(ErrorResponse::BadRequest("You can't add yourself as a member".to_string()));
    }

    let members = add_project_member(*owner_id, project_id, member_id).await
        .map_err(|e| ErrorResponse::InternalServerError(e.to_string()))?;

    if let Some(member) = members.iter().find(|m| m.id == member_id) {
        if let Ok(Some(project)) = crate::services::project::get_project_by_id(member_id, project_id).await {
            create_notification(
                "Участник добавлен".to_string(),
                format!("Вы только что добавили участника {} {} в проект {}.", member.first_name, member.last_name, project.name).to_string(),
                project.owner.id,
                &app_data.mailer
            ).await;
            create_notification(
                "Участник добавлен".to_string(),
                format!("Вы только что были добавлены в проект {} в качестве участника.", project.name).to_string(),
                member.id,
                &app_data.mailer
            ).await;
        }
    }

    Ok(Json(SuccessResponse::new(members)))
}

#[api_operation(
    summary = "Remove project member",
    description = "Remove member from project",
    tag = "Projects",
    error_code = "401",
    error_code = "404"
)]
pub async fn remove_member(
    app_data: Data<AppData>,
    owner_id: ReqData<u64>,
    path: Path<(u64, u64)>
) -> Result<Json<SuccessResponse<Vec<SelectUser>>>, ErrorResponse> {
    let (project_id, member_id) = path.into_inner();

    if *owner_id == member_id {
        return Err(ErrorResponse::BadRequest("You can't remove yourself as a member".to_string()));
    }

    let members = remove_project_member(*owner_id, project_id, member_id).await
        .map_err(|e| ErrorResponse::InternalServerError(e.to_string()))?;

    if let Ok(Some(member)) = crate::services::user::get_user(member_id).await {
        if let Ok(Some(project)) = crate::services::project::get_project_by_id(member_id, project_id).await {
            create_notification(
                "Участник удален".to_string(),
                format!("Вы только что удалили участника {} {} из проекта {}.", member.first_name, member.last_name, project.name).to_string(),
                project.owner.id,
                &app_data.mailer
            ).await;
            create_notification(
                "Участник удален".to_string(),
                format!("Вы только что были удалены из проекта {}.", project.name).to_string(),
                member.id,
                &app_data.mailer
            ).await;
        }
    }

    Ok(Json(SuccessResponse::new(members)))
}
