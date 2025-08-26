use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::core::{UserStory, Priority};
use crate::integrations::CreatedIssue;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIssue {
    pub id: u64,
    pub number: u32,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub labels: Vec<GitHubLabel>,
    pub assignee: Option<GitHubUser>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubLabel {
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
}

#[derive(Debug, Serialize)]
struct CreateIssueRequest {
    title: String,
    body: String,
    labels: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CreateIssueResponse {
    id: u64,
    number: u32,
    html_url: String,
}

pub struct GitHubClient {
    client: Client,
    token: String,
    repo_owner: String,
    repo_name: String,
}

impl GitHubClient {
    pub fn new(token: String) -> Self {
        // Extract repo info from git remote or environment
        let (repo_owner, repo_name) = Self::get_repo_info().unwrap_or(("owner".to_string(), "repo".to_string()));
        
        Self {
            client: Client::new(),
            token,
            repo_owner,
            repo_name,
        }
    }

    pub async fn fetch_issues(&self) -> Result<Vec<GitHubIssue>> {
        let url = format!("https://api.github.com/repos/{}/{}/issues", self.repo_owner, self.repo_name);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("token {}", self.token))
            .header("User-Agent", "codebase-analyzer")
            .send()
            .await?;

        if response.status().is_success() {
            let issues: Vec<GitHubIssue> = response.json().await?;
            Ok(issues)
        } else {
            Ok(Vec::new())
        }
    }

    pub async fn create_issue_from_story(&self, story: &UserStory) -> Result<CreatedIssue> {
        let url = format!("https://api.github.com/repos/{}/{}/issues", self.repo_owner, self.repo_name);
        
        let labels = vec![
            "user-story".to_string(),
            match story.priority {
                Priority::Critical => "priority:critical".to_string(),
                Priority::High => "priority:high".to_string(),
                Priority::Medium => "priority:medium".to_string(),
                Priority::Low => "priority:low".to_string(),
            }
        ];

        let body = self.format_story_body(story);
        
        let request = CreateIssueRequest {
            title: story.title.clone(),
            body,
            labels,
        };

        let response = self.client
            .post(&url)
            .header("Authorization", format!("token {}", self.token))
            .header("User-Agent", "codebase-analyzer")
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let created: CreateIssueResponse = response.json().await?;
            Ok(CreatedIssue {
                platform: "GitHub".to_string(),
                issue_id: created.number.to_string(),
                title: story.title.clone(),
                url: created.html_url,
            })
        } else {
            anyhow::bail!("Failed to create GitHub issue: {}", response.status())
        }
    }

    fn get_repo_info() -> Option<(String, String)> {
        // Try to extract from git remote
        if let Ok(repo) = git2::Repository::open(".") {
            if let Ok(remote) = repo.find_remote("origin") {
                if let Some(url) = remote.url() {
                    return Self::parse_github_url(url);
                }
            }
        }

        // Fallback to environment variables
        if let (Ok(owner), Ok(name)) = (std::env::var("GITHUB_OWNER"), std::env::var("GITHUB_REPO")) {
            return Some((owner, name));
        }

        None
    }

    fn parse_github_url(url: &str) -> Option<(String, String)> {
        // Parse GitHub URLs like:
        // https://github.com/owner/repo.git
        // git@github.com:owner/repo.git
        
        if url.contains("github.com") {
            let parts: Vec<&str> = if url.starts_with("git@") {
                url.split(':').collect()
            } else {
                url.split('/').collect()
            };

            if parts.len() >= 2 {
                let repo_part = parts[parts.len() - 1];
                let owner_part = parts[parts.len() - 2];
                
                let repo_name = repo_part.strip_suffix(".git").unwrap_or(repo_part);
                return Some((owner_part.to_string(), repo_name.to_string()));
            }
        }

        None
    }

    fn format_story_body(&self, story: &UserStory) -> String {
        let mut body = String::new();
        
        body.push_str(&format!("**User Story**: {}\n\n", story.description));
        
        if !story.acceptance_criteria.is_empty() {
            body.push_str("## Acceptance Criteria\n\n");
            for (i, criteria) in story.acceptance_criteria.iter().enumerate() {
                body.push_str(&format!("{}. {}\n", i + 1, criteria));
            }
            body.push_str("\n");
        }

        if !story.related_components.is_empty() {
            body.push_str("## Related Components\n\n");
            for component in &story.related_components {
                body.push_str(&format!("- {}\n", component));
            }
            body.push_str("\n");
        }

        body.push_str(&format!("**Priority**: {:?}\n", story.priority));
        body.push_str(&format!("**Complexity**: {:?}\n", story.complexity));
        body.push_str(&format!("**Status**: {:?}\n", story.status));
        
        body.push_str("\n---\n");
        body.push_str("*Generated by Codebase Analyzer*");

        body
    }
}