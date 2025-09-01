use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessDomainConfig {
    pub business_domains: Vec<BusinessDomain>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessDomain {
    pub id: String,
    pub name: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub patterns: Vec<DetectionPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionPattern {
    #[serde(rename = "type")]
    pub pattern_type: String,
    pub patterns: Vec<String>,
    pub confidence_weight: f32,
}

impl BusinessDomainConfig {
    pub fn load_from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: BusinessDomainConfig = serde_json::from_str(&content)?;
        Ok(config)
    }
    
    pub fn get_domain_by_id(&self, id: &str) -> Option<&BusinessDomain> {
        self.business_domains.iter().find(|d| d.id == id)
    }
}

impl BusinessDomain {
    pub fn calculate_match_score(&self, 
        api_endpoints: &[String], 
        dependencies: &[String], 
        file_names: &[String]) -> f32 {
        
        let mut total_score = 0.0;
        
        for pattern in &self.patterns {
            let pattern_score = match pattern.pattern_type.as_str() {
                // API/Route patterns
                "route_pattern" => {
                    let matches = api_endpoints.iter()
                        .filter(|endpoint| pattern.patterns.iter()
                            .any(|p| endpoint.to_lowercase().contains(&p.to_lowercase())))
                        .count() as f32;
                    matches * pattern.confidence_weight
                },
                // File/Component patterns
                "file_pattern" | "component_pattern" | "database_pattern" 
                | "analysis_pattern" | "ast_pattern" | "tool_pattern" => {
                    let matches = file_names.iter()
                        .filter(|file| pattern.patterns.iter()
                            .any(|p| file.to_lowercase().contains(&p.to_lowercase())))
                        .count() as f32;
                    matches * pattern.confidence_weight
                },
                // Dependency patterns  
                "service_integration" | "data_framework" | "search_engine" | "monitoring_tool" => {
                    let matches = dependencies.iter()
                        .filter(|dep| pattern.patterns.iter()
                            .any(|p| dep.to_lowercase().contains(&p.to_lowercase())))
                        .count() as f32;
                    matches * pattern.confidence_weight
                },
                // Generic patterns - check across all sources
                "crud_pattern" | "method_pattern" | "websocket_pattern" | "infrastructure_pattern"
                | "rate_limiting_pattern" | "pipeline_pattern" | "workflow_pattern" 
                | "scheduler_pattern" | "logging_pattern" => {
                    let endpoint_matches = api_endpoints.iter()
                        .filter(|endpoint| pattern.patterns.iter()
                            .any(|p| endpoint.to_lowercase().contains(&p.to_lowercase())))
                        .count() as f32;
                    let file_matches = file_names.iter()
                        .filter(|file| pattern.patterns.iter()
                            .any(|p| file.to_lowercase().contains(&p.to_lowercase())))
                        .count() as f32;
                    let dep_matches = dependencies.iter()
                        .filter(|dep| pattern.patterns.iter()
                            .any(|p| dep.to_lowercase().contains(&p.to_lowercase())))
                        .count() as f32;
                    (endpoint_matches + file_matches + dep_matches) * pattern.confidence_weight
                },
                _ => 0.0
            };
            total_score += pattern_score;
        }
        
        total_score
    }
}