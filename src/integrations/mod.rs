pub mod git;
pub mod github;
pub mod jira;

use crate::core::{CodebaseAnalysis, Task, UserStory, Priority};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    pub git_enabled: bool,
    pub github_token: Option<String>,
    pub jira_url: Option<String>,
    pub jira_token: Option<String>,
    pub jira_project_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationSummary {
    pub git_info: Option<git::GitInfo>,
    pub github_issues: Vec<github::GitHubIssue>,
    pub jira_issues: Vec<jira::JiraIssue>,
    pub created_issues: Vec<CreatedIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedIssue {
    pub platform: String,
    pub issue_id: String,
    pub title: String,
    pub url: String,
}

pub struct IntegrationEngine {
    config: IntegrationConfig,
}

impl IntegrationEngine {
    pub fn new(config: IntegrationConfig) -> Self {
        Self { config }
    }

    pub async fn integrate_analysis(&self, analysis: &CodebaseAnalysis) -> Result<IntegrationSummary> {
        let mut summary = IntegrationSummary {
            git_info: None,
            github_issues: Vec::new(),
            jira_issues: Vec::new(),
            created_issues: Vec::new(),
        };

        // Git integration
        if self.config.git_enabled {
            if let Ok(git_info) = git::GitAnalyzer::analyze_repository(".") {
                summary.git_info = Some(git_info);
            }
        }

        // GitHub integration
        if let Some(token) = &self.config.github_token {
            if let Ok(issues) = github::GitHubClient::new(token.clone()).fetch_issues().await {
                summary.github_issues = issues;
            }

            // Create GitHub issues for high-priority user stories
            let high_priority_stories: Vec<_> = analysis.user_stories.iter()
                .filter(|s| matches!(s.priority, Priority::High | Priority::Critical))
                .collect();

            for story in high_priority_stories {
                if let Ok(issue) = github::GitHubClient::new(token.clone())
                    .create_issue_from_story(story).await {
                    summary.created_issues.push(issue);
                }
            }
        }

        // Jira integration
        if let (Some(url), Some(token), Some(project_key)) = (
            &self.config.jira_url,
            &self.config.jira_token,
            &self.config.jira_project_key,
        ) {
            if let Ok(issues) = jira::JiraClient::new(url.clone(), token.clone())
                .fetch_project_issues(project_key).await {
                summary.jira_issues = issues;
            }

            // Create Jira tickets for critical tasks
            let critical_tasks: Vec<_> = analysis.tasks.iter()
                .filter(|t| matches!(t.priority, Priority::Critical))
                .collect();

            for task in critical_tasks {
                if let Ok(issue) = jira::JiraClient::new(url.clone(), token.clone())
                    .create_ticket_from_task(task, project_key).await {
                    summary.created_issues.push(issue);
                }
            }
        }

        Ok(summary)
    }
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            git_enabled: true,
            github_token: std::env::var("GITHUB_TOKEN").ok(),
            jira_url: std::env::var("JIRA_URL").ok(),
            jira_token: std::env::var("JIRA_TOKEN").ok(),
            jira_project_key: std::env::var("JIRA_PROJECT_KEY").ok(),
        }
    }
}