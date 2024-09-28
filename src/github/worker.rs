use anyhow::{Context, Result};
use jsonwebtoken::EncodingKey;
use octocrab::{
    models::{issues::Issue, IssueState},
    Octocrab,
};
use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;

use crate::{
    models::{
        project::SelectProject,
        task::{CreateTaskRequest, SelectTask, TaskStatus, UpdateTaskRequest},
    },
    services::{
        project::get_all_projects,
        task::{create_task, update_task},
    },
};

pub struct GitHubWorker {
    octocrab: Octocrab,
    cancel_token: CancellationToken,
    process_interval: Duration,
}

impl GitHubWorker {
    pub async fn new(
        app_id: u64,
        private_key: &str,
        cancel_token: CancellationToken,
        process_interval: Duration,
    ) -> Result<Self> {
        let access_key = EncodingKey::from_rsa_pem(private_key.as_bytes())
            .context("Failed to create encoding key")?;
        let octocrab = Octocrab::builder()
            .app(app_id.into(), access_key)
            .build()
            .context("Failed to build Octocrab instance")?;

        Ok(Self {
            octocrab,
            cancel_token,
            process_interval,
        })
    }

    pub async fn work(&self) -> Result<()> {
        log::info!("GitHub worker started");
        while !self.cancel_token.is_cancelled() {
            if let Err(e) = self.process_projects().await {
                log::error!("Error processing projects: {}", e);
            }

            tokio::select! {
                _ = sleep(self.process_interval) => {}
                _ = self.cancel_token.cancelled() => {
                    log::info!("Graceful shutdown triggered");
                    break;
                }
            }
        }
        Ok(())
    }
    async fn process_projects(&self) -> Result<()> {
        let projects = get_all_projects().await.map_err(|e| anyhow::anyhow!("Failed to get projects: {}", e))?;

        for project in &projects {
            if project.repository_id.is_none() {
                continue;
            }

            let (owner, repo) = self.parse_repository(project)?;
            let issues = self.fetch_issues(&owner, &repo).await?;

            for issue in issues {
                self.handle_issue(project, &issue).await?;
            }
        }
        Ok(())
    }

    async fn handle_issue(&self, project: &SelectProject, issue: &Issue) -> Result<()> {
        match issue.state {
            IssueState::Open => self.handle_open_issue(project, issue).await,
            IssueState::Closed => self.handle_closed_issue(project, issue).await,
            _ => Ok(()),
        }
    }
    async fn create_task_for_issue(&self, project: &SelectProject, issue: &Issue) -> Result<()> {
        let task = CreateTaskRequest {
            name: issue.title.clone(),
            description: issue.body.clone().unwrap_or_default(),
            project_id: project.id,
            attached_to: Vec::new(),
            assigned_issue: Some(issue.number),
            due_date: None,
        };

        create_task(project.owner.id, &task).await.map_err(|e| anyhow::anyhow!("Failed to create task: {}", e))?;
        Ok(())
    }
    async fn close_task_for_issue(&self, project: &SelectProject, task: &SelectTask, issue: &Issue) -> Result<()> {
        let updated = UpdateTaskRequest {
            assigned_issue: Some(issue.number),
            name: None,
            description: None,
            status: Some(TaskStatus::Done),
            due_date: None,
        };

        update_task(project.owner.id, task.id, &updated).await.map_err(|e| anyhow::anyhow!("Failed to update task: {}", e))?;
        Ok(())
    }
    fn parse_repository(&self, project: &SelectProject) -> Result<(String, String)> {
        let repo = project.repository_id.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Project has no repository_id"))?;
        let (owner, repo) = repo.split_once('/').context("Invalid repository format")?;
        if owner.is_empty() || repo.is_empty() {
            anyhow::bail!("Invalid repository: owner or repo is empty");
        }
        Ok((owner.to_string(), repo.to_string()))
    }

    async fn fetch_issues(&self, owner: &str, repo: &str) -> Result<Vec<Issue>> {
        Ok(self.octocrab
            .issues(owner, repo)
            .list()
            .send()
            .await
            .context("Failed to fetch issues")?
            .items)
    }

    async fn handle_open_issue(&self, project: &SelectProject, issue: &Issue) -> Result<()> {
        let task_exists = project.tasks.iter().any(|task| task.assigned_issue == Some(issue.number));
        if !task_exists {
            self.create_task_for_issue(project, issue).await?;
        }
        Ok(())
    }

    async fn handle_closed_issue(&self, project: &SelectProject, issue: &Issue) -> Result<()> {
        if let Some(task) = project.tasks.iter().find(|task| task.assigned_issue == Some(issue.number)) {
            if task.status != TaskStatus::Done {
                self.close_task_for_issue(project, task, issue).await?;
            }
        }
        Ok(())
    }
}
