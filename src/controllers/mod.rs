use apistos::web;

use crate::middleware::auth::Authentication;

pub mod user;
pub mod auth;
pub mod project;
pub mod task;
pub mod notification;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/login", web::post().to(auth::login))
            .route("/register", web::post().to(auth::register)
        )
    );
    cfg.service(
        web::scope("/users")
            .wrap(Authentication)
            .route("/", web::get().to(user::get_all))
            .route("/me", web::get().to(user::get_me))
    );
    cfg.service(
        web::scope("/projects")
            .wrap(Authentication)
            .route("/my", web::get().to(project::get_my))
            .route("/{project_id}", web::get().to(project::get_by_id))
            .route("/", web::post().to(project::create_project))
            .route("/{project_id}", web::patch().to(project::update_project))
            .route("/{project_id}", web::delete().to(project::delete_project))
            .route("/{project_id}/members/{member_id}", web::post().to(project::add_member))
            .route("/{project_id}/members/{member_id}", web::delete().to(project::remove_member))
    );
    cfg.service(
        web::scope("/tasks")
            .wrap(Authentication)
            .route("/my", web::get().to(task::get_my))
            .route("/{task_id}", web::get().to(task::get_by_id))
            .route("/", web::post().to(task::create_task))
            .route("/{task_id}", web::patch().to(task::update_task))
            .route("/{task_id}", web::delete().to(task::delete_task))
            .route("/{task_id}/assignees/{assignee_id}", web::post().to(task::add_assignee))
            .route("/{task_id}/assignees/{assignee_id}", web::delete().to(task::remove_assignee))
    );
    cfg.service(
        web::scope("/notifications")
            .wrap(Authentication)
            .route("/my", web::get().to(notification::get_my))
    );
}

pub fn init_uploads(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::resource("/avatars/get/{user_id}")
            .get(user::get_avatar)
    );
    cfg.service(
        actix_web::web::resource("/avatars/update/me")
            .wrap(Authentication)
            .post(user::change_avatar)
    );
}
