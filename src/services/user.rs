use prisma_client_rust::or;

use crate::models::{auth::RegisterRequest, user::SelectUser};
use crate::models::user::SelectUserQuery;
use crate::prisma;
use crate::prisma::{project, QueryMode, user};
use crate::services::common::create_prisma_client;

const BCRYPT_COST: u32 = 9;

pub fn user_data_to_response(user: &user::Data) -> SelectUser {
    SelectUser {
        id: user.id as u64,
        created_at: user.created_at.timestamp() as u64,
        last_seen: user.last_seen.timestamp() as u64,
        email: user.email.clone(),
        username: user.username.clone(),
        first_name: user.first_name.clone(),
        last_name: user.last_name.clone(),
    }
}

pub async fn get_user_id_by_credentials(
    username: &str,
    password: &str,
) -> Result<Option<u64>, String> {
    let client = create_prisma_client().await?;
    match client
        .user()
        .find_first(vec![or!(
            user::username::equals(username.to_string()),
            user::email::equals(username.to_string())
        )])
        .exec()
        .await
    {
        Ok(Some(user)) => {
            if bcrypt::verify(password, &user.password_hash).unwrap() {
                Ok(Some(user.id as u64))
            } else {
                Ok(None)
            }
        }
        Ok(None) => Ok(None),
        Err(err) => {
            log::error!(target: "Prisma", "Failed to get user by credentials: {:?}", err);
            Ok(None)
        }
    }
}

pub async fn create_user(data: &RegisterRequest) -> Result<u64, String> {
    let client = create_prisma_client().await?;
    let hashed_password = bcrypt::hash(data.password.clone(), BCRYPT_COST).unwrap();
    let user = client
        .user()
        .create(
            data.email.clone(),
            data.username.clone(),
            hashed_password,
            data.first_name.clone(),
            data.last_name.clone(),
            vec![],
        )
        .exec()
        .await;
    match user {
        Ok(user) => Ok(user.id as u64),
        Err(err) => {
            log::error!(target: "Prisma", "Failed to create user: {:?}", err);
            Err(err.to_string())
        }
    }
}

pub async fn get_all_users(filters: SelectUserQuery) -> Result<Vec<SelectUser>, String> {
    let client = create_prisma_client().await?;
    let mut query_filters: Vec<_> = vec![];
    if let Some(project_id) = filters.project_id {
        query_filters.push(
            or!(
                user::projects::some(vec![project::id::equals(project_id as i32)]),
                user::team_projects::some(vec![project::id::equals(project_id as i32)])
            )
        );
    }
    if let Some(username) = filters.username {
        query_filters.push(or!(
            prisma_client_rust::and!(
                user::username::contains(username.clone()),
                user::username::mode(QueryMode::Insensitive)
            ),
            prisma_client_rust::and!(
                user::email::contains(username.clone()),
                user::email::mode(QueryMode::Insensitive)
            )
        ));
    }
    let users = client
        .user()
        .find_many(query_filters)
        .exec()
        .await
        .map_err(|err| {
            log::error!(target: "Prisma", "Failed to get all users: {:?}", err);
            "Failed to get all users".to_string()
        })?;
    Ok(users.into_iter().map(|data| user_data_to_response(&data)).collect())
}

pub async fn get_user(id: u64) -> Result<Option<SelectUser>, String> {
    let client = create_prisma_client().await?;
    update_last_seen(id).await.ok();
    match client
        .user()
        .find_first(vec![user::id::equals(id as i32)])
        .exec()
        .await
    {
        Ok(Some(user)) => Ok(Some(user_data_to_response(&user))),
        Ok(None) => Ok(None),
        Err(err) => {
            log::error!(target: "Prisma", "Failed to get user: {:?}", err);
            Err("Failed to get user".to_string())
        }
    }
}

async fn update_last_seen(id: u64) -> Result<(), String> {
    let client = create_prisma_client().await?;
    match client
        .user()
        .update(
            user::id::equals(id as i32),
            vec![user::last_seen::set(chrono::Utc::now().into())],
        )
        .exec()
        .await
    {
        Ok(_) => Ok(()),
        Err(err) => {
            log::error!(target: "Prisma", "Failed to update last seen: {:?}", err);
            Err("Failed to update last seen".to_string())
        }
    }
}

pub async fn is_project_owner(user_id: u64, project_id: u64) -> Result<bool, String> {
    let client = create_prisma_client().await?;
    match client
        .project()
        .find_first(vec![prisma_client_rust::and!(
            project::id::equals(project_id as i32),
            project::owner_id::equals(user_id as i32)
        )])
        .exec()
        .await
    {
        Ok(Some(_)) => Ok(true),
        Ok(None) => Ok(false),
        Err(err) => {
            log::error!(target: "Prisma", "Failed to check project owner: {:?}", err);
            Err("Failed to check project owner".to_string())
        }
    }
}

pub async fn is_project_member(user_id: u64, project_id: u64) -> Result<bool, String> {
    let client = create_prisma_client().await?;
    match client
        .project()
        .find_first(vec![
            project::id::equals(project_id as i32),
            project::members::some(vec![user::id::equals(user_id as i32)])
        ])
        .exec()
        .await
    {
        Ok(Some(_)) => Ok(true),
        Ok(None) => Ok(false),
        Err(err) => {
            log::error!(target: "Prisma", "Failed to check project member: {:?}", err);
            Err("Failed to check project member".to_string())
        }
    }
}