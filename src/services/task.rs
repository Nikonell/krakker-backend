use std::str::FromStr;
use std::vec;

use chrono::DateTime;
use prisma_client_rust::{Direction, QueryError};

use crate::models::{
    task::{CreateTaskRequest, SelectTask, SelectTaskRequest, UpdateTaskRequest},
    user::SelectUser,
};
use crate::models::project::SelectProject;
use crate::models::task::TaskStatus;
use crate::prisma::{project, task, user};
use crate::prisma::task::Data;
use crate::services::common::create_prisma_client;
use crate::services::user::{is_project_member, is_project_owner, user_data_to_response};

pub async fn task_data_to_response(task_item: &Data) -> Result<SelectTask, String> {
    // Converts from ORM model to response model
    Ok(SelectTask {
        id: task_item.id as u64,
        created_at: task_item.created_at.timestamp() as u64,
        name: task_item.name.clone(),
        description: task_item.description.clone(),
        status: TaskStatus::from_str(&task_item.status).unwrap(),
        due_date: match task_item.due_date {
            Some(date) => Some(date.timestamp() as u64),
            None => None,
        },
        attached_to: match task_item.clone().attached_to {
            Some(users) => users,
            None => return Err("Failed to fetch attached users".to_string()),
        }
            .into_iter()
            .map(|user| SelectUser {
                id: user.id as u64,
                created_at: user.created_at.timestamp() as u64,
                last_seen: user.last_seen.timestamp() as u64,
                email: user.email.clone(),
                username: user.username.clone(),
                first_name: user.first_name.clone(),
                last_name: user.last_name,
            })
            .collect(),
        assigned_issue: task_item.assigned_issue.map(|issue| issue as u64),
        project: match task_item.clone().project {
            Some(project) => SelectProject {
                id: project.id as u64,
                created_at: project.created_at.timestamp() as u64,
                name: project.name.clone(),
                description: project.description.clone(),
                owner: match project.owner {
                    Some(owner) => SelectUser {
                        id: owner.id as u64,
                        created_at: owner.created_at.timestamp() as u64,
                        last_seen: owner.last_seen.timestamp() as u64,
                        email: owner.email.clone(),
                        username: owner.username.clone(),
                        first_name: owner.first_name.clone(),
                        last_name: owner.last_name,
                    },
                    None => return Err("Failed to fetch project owner".to_string()),
                },
                members: vec![],
                tasks: vec![],
                repository_id: project.repo_id.clone(),
            },
            None => return Err("Failed to fetch project".to_string()),
        },
    })
}

pub async fn task_entity_to_response(task: prisma_client_rust::Result<Data>) -> Result<Option<Vec<SelectUser>>, String> {
    match task {
        Ok(task) => {
            let attached_to = task.attached_to.unwrap_or(vec![]);
            let users = attached_to
                .into_iter()
                .map(|user| user_data_to_response(&user))
                .collect();
            Ok(Some(users))
        }
        Err(err) => Err(err.to_string()),
    }
}

pub async fn task_result_to_response(
    task: Result<Option<Data>, QueryError>,
) -> Result<Option<SelectTask>, String> {
    match task {
        Ok(Some(task)) => {
            let response = task_data_to_response(&task).await?;
            Ok(Some(response))
        }
        Ok(None) => Ok(None),
        Err(err) => Err(err.to_string()),
    }
}

pub async fn require_project_participant(user_id: u64, project_id: u64) -> Result<(), String> {
    let is_member = is_project_member(user_id, project_id).await?;
    let is_owner = is_project_owner(user_id, project_id).await?;
    if !is_member && !is_owner {
        return Err("User is not a member or owner of the project".to_string());
    }
    Ok(())
}
pub async fn check_member_from_task(user_id: u64, task_id: u64) -> Result<(), String> {
    let client = create_prisma_client().await?;
    let project_id = match client
        .task()
        .find_first(vec![task::id::equals(task_id as i32)])
        .exec()
        .await
    {
        Ok(Some(task)) => task.project_id as u64,
        Ok(None) => return Err("Task not found".to_string()),
        Err(err) => return Err(err.to_string()),
    };
    require_project_participant(user_id, project_id).await
}
pub async fn get_user_tasks(
    user_id: u64,
    filters: &SelectTaskRequest,
) -> Result<Vec<SelectTask>, String> {
    let client = create_prisma_client().await?;
    let mut query_filters = vec![task::attached_to::some(vec![user::id::equals(
        user_id as i32,
    )])];
    if filters.project_id.is_some() {
        query_filters.push(task::attached_to::some(vec![user::id::equals(
            filters.project_id.unwrap() as i32,
        )]));
    }
    let tasks_query = client
        .task()
        .find_many(query_filters)
        .with(task::attached_to::fetch(vec![]))
        .order_by(task::due_date::order(Direction::Asc))
        .with(task::project::fetch().with(project::owner::fetch()))
        .exec()
        .await;

    match tasks_query {
        Ok(fetched_tasks) => {
            let mut rendered_tasks: Vec<SelectTask> = vec![];
            for task in fetched_tasks.iter() {
                rendered_tasks.push(task_data_to_response(task).await?);
            }
            Ok(rendered_tasks)
        }
        Err(err) => Err(err.to_string()),
    }
}

pub async fn get_task_by_id(task_id: u64) -> Result<Option<SelectTask>, String> {
    let client = create_prisma_client().await?;
    let task = client
        .task()
        .find_first(vec![task::id::equals(task_id as i32)])
        .with(task::attached_to::fetch(vec![]))
        .with(task::project::fetch().with(project::owner::fetch()))
        .exec()
        .await;
    task_result_to_response(task).await
}

pub async fn create_task(user_id: u64, task: &CreateTaskRequest) -> Result<SelectTask, String> {
    require_project_participant(user_id, task.project_id).await?;

    let client = create_prisma_client().await?;
    let task = client
        .task()
        .create(
            task.name.clone(),
            TaskStatus::Todo.to_string(),
            task.description.clone(),
            project::id::equals(task.project_id as i32),
            vec![task::attached_to::connect(
                task.attached_to
                    .iter()
                    .map(|id| user::id::equals(*id as i32))
                    .collect(),
            ),
                 task::assigned_issue::set(task.assigned_issue.map(|issue| issue as i32)),
            ],
        )
        .with(task::attached_to::fetch(vec![]))
        .with(task::project::fetch().with(project::owner::fetch()))
        .exec()
        .await;

    match task {
        Ok(created_task) => {
            let task = task_data_to_response(&created_task).await?;
            Ok(task)
        }
        Err(err) => Err(err.to_string()),
    }
}

pub async fn update_task(
    user_id: u64,
    task_id: u64,
    task: &UpdateTaskRequest,
) -> Result<Option<SelectTask>, String> {
    check_member_from_task(user_id, task_id).await?;

    let client = create_prisma_client().await?;

    let mut update_properties = vec![];
    if let Some(name) = task.name.clone() {
        update_properties.push(task::name::set(name));
    }
    if let Some(status) = task.status.clone() {
        update_properties.push(task::status::set(status.to_string()));
    }
    if let Some(description) = task.description.clone() {
        update_properties.push(task::description::set(description));
    }
    if let Some(due_date) = task.due_date {
        let date = DateTime::from_timestamp(due_date as i64, 0).unwrap();
        update_properties.push(task::due_date::set(Some(date.into())));
    }
    if let Some(assigned_issue) = task.assigned_issue {
        update_properties.push(task::assigned_issue::set(Some(assigned_issue as i32)));
    } else {
        update_properties.push(task::assigned_issue::set(None));
    }

    let task = client
        .task()
        .update(task::id::equals(task_id as i32), update_properties)
        .with(task::attached_to::fetch(vec![]))
        .with(task::project::fetch().with(project::owner::fetch()))
        .exec()
        .await;
    match task {
        Ok(updated_task) => task_result_to_response(Ok(Some(updated_task))).await,
        Err(err) => Err(err.to_string()),
    }
}

pub async fn delete_task(user_id: u64, task_id: u64) -> Result<(), String> {
    check_member_from_task(user_id, task_id).await?;

    let client = create_prisma_client().await?;
    let task = client
        .task()
        .delete(task::id::equals(task_id as i32))
        .exec()
        .await;
    match task {
        Ok(_) => Ok(()),
        Err(err) => Err(err.to_string()),
    }
}

pub async fn add_assigned_user(
    user_id: u64,
    task_id: u64,
    assigned_user_id: u64,
) -> Result<Option<Vec<SelectUser>>, String> {
    check_member_from_task(user_id, task_id).await?;
    let client = create_prisma_client().await?;
    let task = client
        .task()
        .update(
            task::id::equals(task_id as i32),
            vec![task::attached_to::connect(vec![user::id::equals(
                assigned_user_id as i32,
            )])],
        )
        .with(task::attached_to::fetch(vec![]))
        .with(task::project::fetch().with(project::owner::fetch()))
        .exec()
        .await;
    task_entity_to_response(task).await
}

pub async fn remove_assigned_user(
    user_id: u64,
    task_id: u64,
    assigned_user_id: u64,
) -> Result<Option<Vec<SelectUser>>, String> {
    check_member_from_task(user_id, task_id).await?;
    let client = create_prisma_client().await?;

    let task = client
        .task()
        .update(
            task::id::equals(task_id as i32),
            vec![task::attached_to::disconnect(vec![user::id::equals(
                assigned_user_id as i32,
            )])],
        )
        .with(task::attached_to::fetch(vec![]))
        .with(task::project::fetch().with(project::owner::fetch()))
        .exec()
        .await;
    task_entity_to_response(task).await
}
