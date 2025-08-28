use super::{DocumentGenerator, DocumentType};
use crate::core::CodebaseAnalysis;
use crate::intelligence::IntelligentAnalysis;
use anyhow::Result;
use serde_yaml;

pub struct YamlAnalysisGenerator;

impl YamlAnalysisGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl DocumentGenerator for YamlAnalysisGenerator {
    fn generate(&self, analysis: &CodebaseAnalysis, intelligent_analysis: Option<&IntelligentAnalysis>) -> Result<String> {
        // Create structured analysis output as specified in enhanced CLAUDE.md
        let yaml_output = AnalysisOutput {
            metadata: AnalysisMetadata {
                analyzer_version: analysis.analysis_metadata.analyzer_version.clone(),
                analysis_date: analysis.analysis_metadata.analyzed_at.clone(),
                project_path: "analyzed_project_path".to_string(), // This should come from analysis context
                frameworks_detected: analysis.framework_analysis.detected_frameworks.iter()
                    .map(|f| f.name.clone())
                    .collect(),
                confidence_scores: analysis.framework_analysis.confidence_scores.clone(),
            },
            business_context: BusinessContext {
                inferred_product_type: analysis.business_context.inferred_product_type.clone(),
                confidence: analysis.business_context.confidence,
                evidence: analysis.business_context.evidence.clone(),
                primary_user_personas: analysis.business_context.primary_user_personas.clone(),
                user_journeys_discovered: analysis.business_context.user_journeys_discovered.clone(),
            },
            implementation_analysis: ImplementationAnalysis {
                user_stories: analysis.user_stories.iter().map(|story| UserStoryYaml {
                    id: story.id.clone(),
                    title: story.title.clone(),
                    description: story.description.clone(),
                    status: format!("{:?}", story.status),
                    priority: format!("{:?}", story.priority),
                    complexity: format!("{:?}", story.complexity),
                    acceptance_criteria: story.acceptance_criteria.clone(),
                    evidence: story.inferred_from.clone(),
                }).collect(),
                components: analysis.components.iter().map(|comp| ComponentYaml {
                    name: comp.name.clone(),
                    type_name: format!("{:?}", comp.component_type),
                    purpose: comp.purpose.clone(),
                    file_path: comp.file_path.clone(),
                    status: format!("{:?}", comp.implementation_status),
                    complexity_score: comp.complexity_score,
                    dependencies: comp.dependencies.clone(),
                    api_calls: comp.api_calls.iter().map(|api| format!("{} {}", api.method, api.endpoint)).collect(),
                }).collect(),
                api_endpoints: analysis.implementation_analysis.api_endpoints.iter().map(|endpoint| ApiEndpointYaml {
                    path: endpoint.path.clone(),
                    method: endpoint.method.clone(),
                    purpose: endpoint.purpose.clone(),
                    status: format!("{:?}", endpoint.status),
                    controller: endpoint.controller.clone(),
                }).collect(),
                database_entities: analysis.implementation_analysis.database_entities.iter().map(|entity| DatabaseEntityYaml {
                    name: entity.name.clone(),
                    purpose: entity.purpose.clone(),
                    status: format!("{:?}", entity.status),
                    fields: entity.fields.iter().map(|f| format!("{}: {}", f.name, f.field_type)).collect(),
                }).collect(),
            },
            status_intelligence: StatusIntelligence {
                completed_features: analysis.status_intelligence.completed_features.iter()
                    .map(|f| f.name.clone()).collect(),
                in_progress_features: analysis.status_intelligence.in_progress_features.iter()
                    .map(|f| f.name.clone()).collect(),
                todo_features: analysis.status_intelligence.todo_features.iter()
                    .map(|f| f.name.clone()).collect(),
                technical_debt: analysis.status_intelligence.technical_debt.iter()
                    .map(|debt| format!("{}: {}", debt.severity, debt.description)).collect(),
                overall_completion_percentage: analysis.status_intelligence.overall_completion_percentage,
            },
            integration_points: IntegrationPoints {
                external_services: analysis.integration_points.external_services.iter()
                    .map(|svc| format!("{}: {}", svc.name, svc.service_type)).collect(),
                internal_dependencies: analysis.integration_points.internal_dependencies.iter()
                    .map(|dep| format!("{}: {}", dep.name, dep.dependency_type)).collect(),
                configuration_files: analysis.integration_points.configuration_files.iter()
                    .map(|cfg| format!("{}: {}", cfg.file_path, cfg.purpose)).collect(),
                environment_variables: analysis.integration_points.environment_variables.clone(),
            },
        };

        // Add intelligent analysis if available
        let mut yaml_string = serde_yaml::to_string(&yaml_output)?;
        
        if let Some(intel) = intelligent_analysis {
            yaml_string.push_str("\\n# Enhanced Intelligence Analysis\\n");
            yaml_string.push_str(&format!("quality_metrics:\\n"));
            yaml_string.push_str(&format!("  overall_score: {:.2}\\n", intel.quality_metrics.overall_score));
            yaml_string.push_str(&format!("  maintainability: {:.2}\\n", intel.quality_metrics.maintainability));
            yaml_string.push_str(&format!("  complexity: {:.2}\\n", intel.quality_metrics.complexity));
            yaml_string.push_str(&format!("  technical_debt_score: {:.2}\\n", intel.quality_metrics.technical_debt_score));
        }

        Ok(yaml_string)
    }

    fn get_file_extension(&self) -> &str {
        "yaml"
    }

    fn get_document_type(&self) -> DocumentType {
        DocumentType::YamlAnalysis
    }
}

// YAML-specific data structures for serialization
#[derive(serde::Serialize)]
struct AnalysisOutput {
    metadata: AnalysisMetadata,
    business_context: BusinessContext,
    implementation_analysis: ImplementationAnalysis,
    status_intelligence: StatusIntelligence,
    integration_points: IntegrationPoints,
}

#[derive(serde::Serialize)]
struct AnalysisMetadata {
    analyzer_version: String,
    analysis_date: String,
    project_path: String,
    frameworks_detected: Vec<String>,
    confidence_scores: std::collections::HashMap<String, f32>,
}

#[derive(serde::Serialize)]
struct BusinessContext {
    inferred_product_type: String,
    confidence: f32,
    evidence: Vec<String>,
    primary_user_personas: Vec<String>,
    user_journeys_discovered: Vec<String>,
}

#[derive(serde::Serialize)]
struct ImplementationAnalysis {
    user_stories: Vec<UserStoryYaml>,
    components: Vec<ComponentYaml>,
    api_endpoints: Vec<ApiEndpointYaml>,
    database_entities: Vec<DatabaseEntityYaml>,
}

#[derive(serde::Serialize)]
struct StatusIntelligence {
    completed_features: Vec<String>,
    in_progress_features: Vec<String>,
    todo_features: Vec<String>,
    technical_debt: Vec<String>,
    overall_completion_percentage: f32,
}

#[derive(serde::Serialize)]
struct IntegrationPoints {
    external_services: Vec<String>,
    internal_dependencies: Vec<String>,
    configuration_files: Vec<String>,
    environment_variables: Vec<String>,
}

#[derive(serde::Serialize)]
struct UserStoryYaml {
    id: String,
    title: String,
    description: String,
    status: String,
    priority: String,
    complexity: String,
    acceptance_criteria: Vec<String>,
    evidence: Vec<String>,
}

#[derive(serde::Serialize)]
struct ComponentYaml {
    name: String,
    type_name: String,
    purpose: String,
    file_path: String,
    status: String,
    complexity_score: u8,
    dependencies: Vec<String>,
    api_calls: Vec<String>,
}

#[derive(serde::Serialize)]
struct ApiEndpointYaml {
    path: String,
    method: String,
    purpose: String,
    status: String,
    controller: String,
}

#[derive(serde::Serialize)]
struct DatabaseEntityYaml {
    name: String,
    purpose: String,
    status: String,
    fields: Vec<String>,
}