use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::core::{Task, Priority};
use crate::integrations::CreatedIssue;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraIssue {
    pub id: String,
    pub key: String,
    pub summary: String,
    pub description: Option<String>,
    pub status: JiraStatus,
    pub priority: JiraPriority,
    pub assignee: Option<JiraUser>,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraStatus {
    pub name: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraPriority {
    pub name: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraUser {
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "accountId")]
    pub account_id: String,
}

#[derive(Debug, Serialize)]
struct CreateIssueRequest {
    fields: IssueFields,
}

#[derive(Debug, Serialize)]
struct IssueFields {
    project: ProjectKey,
    summary: String,
    description: String,
    issuetype: IssueType,
    priority: PriorityRef,
}

#[derive(Debug, Serialize)]
struct ProjectKey {
    key: String,
}

#[derive(Debug, Serialize)]
struct IssueType {
    name: String,
}

#[derive(Debug, Serialize)]
struct PriorityRef {
    name: String,
}

#[derive(Debug, Deserialize)]
struct CreateIssueResponse {
    id: String,
    key: String,
    #[serde(rename = "self")]
    self_url: String,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    issues: Vec<JiraIssueResponse>,
}

#[derive(Debug, Deserialize)]
struct JiraIssueResponse {
    id: String,
    key: String,
    fields: JiraIssueFields,
}

#[derive(Debug, Deserialize)]
struct JiraIssueFields {
    summary: String,
    description: Option<String>,
    status: JiraStatus,
    priority: Option<JiraPriority>,
    assignee: Option<JiraUser>,
    created: String,
    updated: String,
}

pub struct JiraClient {
    client: Client,
    base_url: String,
    token: String,
}

impl JiraClient {
    pub fn new(base_url: String, token: String) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            token,
        }
    }

    pub async fn fetch_project_issues(&self, project_key: &str) -> Result<Vec<JiraIssue>> {
        let url = format!("{}/rest/api/3/search", self.base_url);
        
        let jql = format!("project = {}", project_key);
        let params = [("jql", jql.as_str()), ("maxResults", "100")];

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .query(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let search_result: SearchResponse = response.json().await?;
            let issues = search_result.issues.into_iter().map(|issue| {
                JiraIssue {
                    id: issue.id,
                    key: issue.key,
                    summary: issue.fields.summary,
                    description: issue.fields.description,
                    status: issue.fields.status,
                    priority: issue.fields.priority.unwrap_or(JiraPriority {
                        name: "Medium".to_string(),
                        id: "3".to_string(),
                    }),
                    assignee: issue.fields.assignee,
                    created: issue.fields.created,
                    updated: issue.fields.updated,
                }
            }).collect();
            
            Ok(issues)
        } else {
            Ok(Vec::new())
        }
    }

    pub async fn create_ticket_from_task(&self, task: &Task, project_key: &str) -> Result<CreatedIssue> {
        let url = format!("{}/rest/api/3/issue", self.base_url);
        
        let priority_name = match task.priority {
            Priority::Critical => "Highest",
            Priority::High => "High",
            Priority::Medium => "Medium",
            Priority::Low => "Low",
        };

        let description = self.format_task_description(task);
        
        let request = CreateIssueRequest {
            fields: IssueFields {
                project: ProjectKey {
                    key: project_key.to_string(),
                },
                summary: task.name.clone(),
                description,
                issuetype: IssueType {
                    name: "Task".to_string(),
                },
                priority: PriorityRef {
                    name: priority_name.to_string(),
                },
            },
        };

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let created: CreateIssueResponse = response.json().await?;
            let issue_url = format!("{}/browse/{}", self.base_url, created.key);
            
            Ok(CreatedIssue {
                platform: "Jira".to_string(),
                issue_id: created.key.clone(),
                title: task.name.clone(),
                url: issue_url,
            })
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to create Jira issue: {} - {}", status, text)
        }
    }

    fn format_task_description(&self, task: &Task) -> String {
        let mut description = String::new();
        
        description.push_str(&format!("*Task Description*: {}\n\n", task.description));
        
        if !task.acceptance_criteria.is_empty() {
            description.push_str("*Acceptance Criteria*:\n");
            for (i, criteria) in task.acceptance_criteria.iter().enumerate() {
                description.push_str(&format!("# {}\n", criteria));
            }
            description.push_str("\n");
        }

        if !task.dependencies.is_empty() {
            description.push_str("*Dependencies*:\n");
            for dep in &task.dependencies {
                description.push_str(&format!("* {}\n", dep));
            }
            description.push_str("\n");
        }

        description.push_str(&format!("*Priority*: {:?}\n", task.priority));
        description.push_str(&format!("*Task Type*: {:?}\n", task.task_type));
        description.push_str(&format!("*Status*: {:?}\n", task.status));
        
        if let Some(estimate) = &task.effort_estimate {
            description.push_str(&format!("*Effort Estimate*: {}\n", estimate));
        }

        description.push_str("\n----\n");
        description.push_str("_Generated by Codebase Analyzer_");

        description
    }
}