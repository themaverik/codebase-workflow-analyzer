use axum::{
    extract::Query,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::core::{CodebaseAnalysis, ProjectType};
use crate::analyzers::{TypeScriptAnalyzer, JavaAnalyzer, PythonAnalyzer};
use crate::intelligence::IntelligenceEngine;
use crate::integrations::{IntegrationConfig, IntegrationEngine};

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[derive(Deserialize)]
pub struct AnalyzeRequest {
    pub path: String,
    pub analyzer: Option<String>,
    pub integrations: Option<bool>,
}

#[derive(Serialize)]
pub struct AnalyzeResponse {
    pub success: bool,
    pub analysis: Option<CodebaseAnalysis>,
    pub error: Option<String>,
    pub integration_summary: Option<crate::integrations::IntegrationSummary>,
}

#[derive(Serialize)]
pub struct IntegrationStatusResponse {
    pub git_available: bool,
    pub github_configured: bool,
    pub jira_configured: bool,
    pub config: IntegrationConfigStatus,
}

#[derive(Serialize)]
pub struct IntegrationConfigStatus {
    pub git_enabled: bool,
    pub has_github_token: bool,
    pub has_jira_config: bool,
}

pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

pub async fn analyze_codebase(
    Query(params): Query<AnalyzeRequest>
) -> Result<Json<AnalyzeResponse>, StatusCode> {
    // Detect project type
    let project_type = detect_project_type(&params.path);
    
    // Choose analyzer
    let analyzer_type = params.analyzer.as_deref().unwrap_or(match project_type {
        ProjectType::React => "typescript",
        ProjectType::SpringBoot => "java", 
        ProjectType::Django | ProjectType::Flask => "python",
        _ => "typescript",
    });

    // Run analysis
    let analysis_result = match analyzer_type {
        "typescript" => {
            let analyzer = TypeScriptAnalyzer::new();
            analyzer.analyze(&params.path).await
        },
        "java" => {
            let analyzer = JavaAnalyzer::new();
            analyzer.analyze(&params.path).await
        },
        "python" => {
            let analyzer = PythonAnalyzer::new();
            analyzer.analyze(&params.path).await
        },
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    match analysis_result {
        Ok(mut analysis) => {
            // Apply intelligence enhancement
            let intelligence = IntelligenceEngine::new();
            let intelligent_analysis = intelligence.enhance_analysis(&analysis);
            
            // Add intelligent insights to analysis
            analysis.intelligent_insights = Some(intelligent_analysis);

            // Run integrations if requested
            let integration_summary = if params.integrations.unwrap_or(false) {
                let config = IntegrationConfig::default();
                let integration_engine = IntegrationEngine::new(config);
                integration_engine.integrate_analysis(&analysis).await.ok()
            } else {
                None
            };

            Ok(Json(AnalyzeResponse {
                success: true,
                analysis: Some(analysis),
                error: None,
                integration_summary,
            }))
        },
        Err(e) => Ok(Json(AnalyzeResponse {
            success: false,
            analysis: None,
            error: Some(e.to_string()),
            integration_summary: None,
        })),
    }
}

pub async fn integration_status() -> Json<IntegrationStatusResponse> {
    let config = IntegrationConfig::default();
    
    Json(IntegrationStatusResponse {
        git_available: git2::Repository::open(".").is_ok(),
        github_configured: config.github_token.is_some(),
        jira_configured: config.jira_url.is_some() && config.jira_token.is_some(),
        config: IntegrationConfigStatus {
            git_enabled: config.git_enabled,
            has_github_token: config.github_token.is_some(),
            has_jira_config: config.jira_url.is_some() && config.jira_token.is_some(),
        },
    })
}

fn detect_project_type(path: &str) -> ProjectType {
    use std::path::Path;

    let path = Path::new(path);
    
    // Check for package.json (React/TypeScript)
    if path.join("package.json").exists() {
        if let Ok(content) = std::fs::read_to_string(path.join("package.json")) {
            if content.contains("\"react\"") {
                return ProjectType::React;
            }
        }
    }

    // Check for Java Spring Boot
    if path.join("pom.xml").exists() || path.join("build.gradle").exists() {
        // Look for Spring Boot dependencies
        if let Ok(content) = std::fs::read_to_string(path.join("pom.xml")) {
            if content.contains("spring-boot") {
                return ProjectType::SpringBoot;
            }
        }
        if let Ok(content) = std::fs::read_to_string(path.join("build.gradle")) {
            if content.contains("spring-boot") {
                return ProjectType::SpringBoot;
            }
        }
    }

    // Check for Python Django/Flask
    if path.join("manage.py").exists() || path.join("settings.py").exists() {
        return ProjectType::Django;
    }
    
    if path.join("app.py").exists() || path.join("wsgi.py").exists() {
        return ProjectType::Flask;
    }

    // Default fallback
    ProjectType::React
}