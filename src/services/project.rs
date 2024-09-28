use prisma_client_rust::or;
use crate::models::{
    project::{CreateProjectRequest, SelectProject, UpdateProjectRequest},
    user::SelectUser,
};
use crate::prisma::{project, task, user};
use crate::services::common::create_prisma_client;
use crate::services::task::task_data_to_response;
use crate::services::user::{get_user, is_project_member, user_data_to_response};

const LOG_TAG: &'static str = "ProjectService";

pub async fn project_data_to_response(project: &project::Data) -> Result<SelectProject, String> {
    Ok(SelectProject {
        id: project.id as u64,
        created_at: project.created_at.timestamp() as u64,
        name: project.name.clone(),
        description: project.description.clone(),
        owner: match project.owner() {
            Ok(owner) => user_data_to_response(owner),
            Err(err) => return Err(err.to_string()),
        },
        members: match project.members() {
            Ok(project_members) => {
                let mut members = vec![];
                for member in project_members {
                    members.push(user_data_to_response(&member));
                }
                members
            }
            Err(err) => return Err(err.to_string()),
        },
        tasks: match project.tasks() {
            Ok(project_tasks) => {
                let mut tasks = vec![];
                for task in project_tasks {
                    tasks.push(task_data_to_response(task).await?);
                }
                tasks
            }
            Err(err) => return Err(err.to_string()),
        },
        repository_id: project.repo_id.clone(),
    })
}

pub async fn get_all_projects() -> Result<Vec<SelectProject>, String> {
    let client = create_prisma_client().await?;
    let projects = client.project()
        .find_many(vec![])
        .with(project::owner::fetch())
        .with(project::tasks::fetch(vec![]).with(task::attached_to::fetch(vec![])))
        .with(project::members::fetch(vec![]))
        .exec().await;
    match projects {
        Ok(projects) => {
            let mut result = vec![];
            for project in &projects {
                result.push(project_data_to_response(project).await?);
            }
            Ok(result)
        }
        Err(err) => {
            log::error!(target: LOG_TAG, "Failed to get all projects: {:?}", err);
            Err(err.to_string())
        }
    }
}

pub async fn get_user_projects(user_id: u64) -> Result<Vec<SelectProject>, String> {
    let client = create_prisma_client().await?;
    let projects = client
        .project()
        .find_many(vec![or!(
            project::owner_id::equals(user_id as i32),
            project::members::some(vec![user::id::equals(user_id as i32)])
        )])
        .with(project::owner::fetch())
        .with(project::tasks::fetch(vec![]).with(task::attached_to::fetch(vec![])))
        .with(project::members::fetch(vec![]))
        .exec()
        .await;
    match projects {
        Ok(projects) => {
            let mut result = vec![];
            for project in &projects {
                result.push(project_data_to_response(project).await?);
            }
            Ok(result)
        }
        Err(err) => {
            log::error!(target: LOG_TAG, "Failed to get projects: {:?}", err);
            Err(err.to_string())
        }
    }
}

pub async fn get_project_by_id(
    user_id: u64,
    project_id: u64,
) -> Result<Option<SelectProject>, String> {
    let client = create_prisma_client().await?;
    let found_project = client
        .project()
        .find_first(vec![
            project::id::equals(project_id as i32),
            or!(
                project::owner_id::equals(user_id as i32),
                project::members::some(vec![user::id::equals(user_id as i32)])
            ),
        ])
        .with(project::owner::fetch())
        .with(project::tasks::fetch(vec![]).with(task::attached_to::fetch(vec![])))
        .with(project::members::fetch(vec![]))
        .exec()
        .await;
    match found_project {
        Ok(Some(project)) => Ok(Some(project_data_to_response(&project).await?)),
        Ok(None) => Ok(None),
        Err(err) => {
            log::error!(target: LOG_TAG, "Failed to get project by id: {:?}", err);
            Err(err.to_string())
        }
    }
}

pub async fn create_project(
    owner_id: u64,
    data: &CreateProjectRequest,
) -> Result<SelectProject, String> {
    let client = create_prisma_client().await?;
    let project = client
        .project()
        .create(
            data.name.clone(),
            data.description.clone(),
            user::id::equals(owner_id as i32),
            vec![project::members::connect(
                data.members
                    .iter()
                    .map(|id| user::id::equals(*id as i32))
                    .collect(),
            )],
        )
        .exec()
        .await;

    let owner = get_user(owner_id).await.map_err(|err| {
        log::error!(target: LOG_TAG, "Failed to get current user: {:?}", err);
        err.to_string()
    })?;

    if let Some(owner) = owner {
        project
            .map(|project| SelectProject {
                id: project.id as u64,
                created_at: project.created_at.timestamp() as u64,
                name: project.name.clone(),
                description: project.description.clone(),
                owner,
                members: vec![],
                tasks: vec![],
                repository_id: project.repo_id.clone()
            })
            .map_err(|err| {
                log::error!(target: LOG_TAG, "Failed to create project: {:?}", err);
                err.to_string()
            })
    } else {
        log::error!(target: LOG_TAG, "Current user seems not found");
        Err("Current user seems not found".to_string())
    }
}

pub async fn update_project(
    owner_id: u64,
    project_id: u64,
    data: &UpdateProjectRequest,
) -> Result<Option<SelectProject>, String> {
    let client = create_prisma_client().await?;
    let project = get_project_by_id(owner_id, project_id).await;
    match project {
        Ok(Some(project)) => {
            if project.owner.id != owner_id {
                log::error!(target: LOG_TAG, "User {owner_id} is not the owner of the project {project_id}");
                return Err("User is not the owner of the project".to_string());
            }
        }
        Ok(None) => {
            log::error!(target: LOG_TAG, "Project not found");
            return Err("Project not found".to_string());
        }
        Err(err) => {
            log::error!(target: LOG_TAG, "Failed to get project by id: {:?}", err);
            return Err(err.to_string());
        }
    };

    let mut updates: Vec<_> = vec![];

    if data.name.is_some() {
        updates.push(project::name::set(data.name.clone().unwrap()));
    }
    if data.description.is_some() {
        updates.push(project::description::set(data.description.clone().unwrap()));
    }
    if data.owner_id.is_some() {
        updates.push(project::owner_id::set(data.owner_id.unwrap() as i32));
    }
    if let Some(repo_id) = data.clone().repository_id {
        updates.push(project::repo_id::set(Some(repo_id)));
    } else {
        updates.push(project::repo_id::set(None));
    }

    let new_project = client
        .project()
        .update(project::id::equals(project_id as i32), updates)
        .with(project::owner::fetch())
        .with(project::tasks::fetch(vec![]).with(task::attached_to::fetch(vec![])))
        .with(project::members::fetch(vec![]))
        .exec()
        .await;
    match new_project {
        Ok(project) => Ok(Some(project_data_to_response(&project).await?)),
        Err(e) => {
            log::error!(target: LOG_TAG, "Failed to update project: {:?}", e);
            Err(e.to_string())
        }
    }
    .map_err(|err| {
        log::error!(target: LOG_TAG, "Failed to update project: {:?}", err);
        err.to_string()
    })
}

pub async fn delete_project(owner_id: u64, project_id: u64) -> Result<(), String> {
    let client = create_prisma_client().await?;
    let project = get_project_by_id(owner_id, project_id).await;
    match project {
        Ok(Some(project)) => {
            if project.owner.id != owner_id {
                // remove member if not owner
                match remove_project_member(owner_id, project_id, owner_id).await {
                    Ok(_) => return Ok(()),
                    Err(err) => Err(err),
                }
            } else {
                let query_result = client
                    .project()
                    .delete(project::id::equals(project_id as i32))
                    .exec()
                    .await;
                match query_result {
                    Ok(_) => Ok(()),
                    Err(err) => {
                        log::error!(target: LOG_TAG, "Failed to delete project: {:?}", err);
                        Err(err.to_string())
                    }
                }
            }
        }
        Ok(None) => {
            log::error!(target: LOG_TAG, "Project not found");
            return Err("Project not found".to_string());
        }
        Err(err) => {
            log::error!(target: LOG_TAG, "Failed to get project by id: {:?}", err);
            return Err(err.to_string());
        }
    }
}

pub async fn add_project_member(
    owner_id: u64,
    project_id: u64,
    user_id: u64,
) -> Result<Vec<SelectUser>, String> {
    if owner_id == user_id {
        log::error!(target: LOG_TAG, "User {owner_id} is the owner of the project {project_id}");
        return Err("User is the owner of the project".to_string());
    }
    let client = create_prisma_client().await?;

    let project = get_project_by_id(owner_id, project_id).await;
    match project {
        Ok(Some(project)) => {
            if project.owner.id != owner_id {
                log::error!(target: LOG_TAG, "User {owner_id} is not the owner of the project {project_id}");
                return Err("User is not the owner of the project".to_string());
            } else {
                let existing_member = is_project_member(user_id, project_id).await?;
                if existing_member {
                    log::info!(target: LOG_TAG, "User {user_id} is already a member of the project {project_id}");
                    return Err("User is already a member of the project".to_string());
                } else {
                    let _ = client
                        .project()
                        .update(
                            project::id::equals(project_id as i32),
                            vec![project::members::connect(vec![user::id::equals(
                                user_id as i32,
                            )])],
                        )
                        .exec()
                        .await;
                }

                let members = client
                    .user()
                    .find_many(vec![user::team_projects::some(vec![project::id::equals(
                        project_id as i32,
                    )])])
                    .exec()
                    .await;

                match members {
                    Ok(members) => {
                        let result = members
                            .into_iter()
                            .map(|member| user_data_to_response(&member))
                            .collect();
                        Ok(result)
                    }
                    Err(err) => {
                        log::error!(target: LOG_TAG, "Failed to get project members: {:?}", err);
                        Err(err.to_string())
                    }
                }
            }
        }
        Ok(None) => {
            log::error!(target: LOG_TAG, "Project not found");
            Err("Project not found".to_string())
        }
        Err(err) => {
            log::error!(target: LOG_TAG, "Failed to get project by id: {:?}", err);
            Err(err.to_string())
        }
    }
}

pub async fn remove_project_member(
    owner_id: u64,
    project_id: u64,
    user_id: u64,
) -> Result<Vec<SelectUser>, String> {
    let client = create_prisma_client().await?;

    let project = get_project_by_id(owner_id, project_id).await;
    match project {
        Ok(Some(project)) => {
            if project.owner.id != owner_id {
                log::error!(target: LOG_TAG, "User {owner_id} is not the owner of the project {project_id}");
                return Err("User is not the owner of the project".to_string());
            } else {
                let existing_member = is_project_member(user_id, project_id).await?;

                if existing_member {
                    let _ = client
                        .project()
                        .update(
                            project::id::equals(project_id as i32),
                            vec![project::members::disconnect(vec![user::id::equals(
                                user_id as i32,
                            )])],
                        )
                        .exec()
                        .await;
                } else {
                    log::info!(target: LOG_TAG, "User {user_id} is not a member of the project {project_id}");
                    return Err("User is not a member of the project".to_string());
                }

                let members = client
                    .user()
                    .find_many(vec![user::team_projects::some(vec![project::id::equals(
                        project_id as i32,
                    )])])
                    .exec()
                    .await;

                match members {
                    Ok(members) => {
                        let result = members
                            .into_iter()
                            .map(|member| user_data_to_response(&member))
                            .collect();
                        Ok(result)
                    }
                    Err(err) => {
                        log::error!(target: LOG_TAG, "Failed to get project members: {:?}", err);
                        Err(err.to_string())
                    }
                }
            }
        }
        Ok(None) => {
            log::error!(target: LOG_TAG, "Project not found");
            Err("Project not found".to_string())
        }
        Err(err) => {
            log::error!(target: LOG_TAG, "Failed to get project by id: {:?}", err);
            Err(err.to_string())
        }
    }
}
