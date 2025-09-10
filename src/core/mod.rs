use serde::{Deserialize, Serialize};

pub mod framework_detector;
pub mod enhanced_framework_detector;
pub mod business_domain_engine;
pub mod integration_demo;
pub mod ast_analyzer;
pub mod extractors;
pub mod types;
// pub mod ast_integration_test;
pub mod config;
pub mod template_engine;
pub mod business_purpose_extractor;
pub mod business_context_config;
pub mod llm_business_extractor;
pub mod project_analyzer;
pub mod project_classifier;
pub mod danet_detector;
// pub mod self_analysis_test;
pub mod context_types;
pub mod context_traits;
pub mod hierarchical_context_manager;
pub mod context_aware_ast_analyzer;
pub mod context_aware_framework_detector;
pub mod business_context_grounding;
pub mod hierarchical_result_fusion;
pub mod documentation_extractor;
pub mod documentation_claims_extractor;
pub mod code_reality_analyzer;
pub mod conflict_resolution_engine;
pub mod dual_category_status_analyzer;
pub mod todo_scanner;
pub mod status_inference_engine;
pub mod analyzers;
pub mod cache_manager;
pub mod performance_monitor;
pub mod context_aware_test;
pub mod framework_validation;
pub mod context_integration_tests;
pub mod refinement_structures;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebaseAnalysis {
    pub project_name: String,
    pub project_type: ProjectType,
    pub components: Vec<ComponentInfo>,
    pub user_stories: Vec<UserStory>,
    pub prd: ProductRequirementDocument,
    pub tasks: Vec<Task>,
    pub analysis_metadata: AnalysisMetadata,
    // Enhanced fields for comprehensive analysis
    pub framework_analysis: FrameworkAnalysis,
    pub business_context: BusinessContext,
    pub implementation_analysis: ImplementationAnalysis,
    pub status_intelligence: StatusIntelligence,
    pub integration_points: IntegrationPoints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectType {
    React,
    NextJS,
    ExpressNodeJS,
    NestJS,
    SpringBoot,
    Django,
    Flask,
    FastAPI,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub name: String,
    pub file_path: String,
    pub component_type: ComponentType,
    pub purpose: String,
    pub dependencies: Vec<String>,
    pub props: Vec<PropInfo>,
    pub hooks_used: Vec<String>,
    pub api_calls: Vec<ApiCall>,
    pub complexity_score: u8,
    pub implementation_status: ImplementationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ComponentType {
    Page,
    Layout,
    Form,
    Display,
    Navigation,
    Modal,
    Utility,
    Hook,
    Context,
    Service,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropInfo {
    pub name: String,
    pub prop_type: String,
    pub required: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCall {
    pub endpoint: String,
    pub method: String,
    pub purpose: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationStatus {
    Complete,
    InProgress,
    Todo,
    Incomplete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStory {
    pub id: String,
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub priority: Priority,
    pub complexity: Complexity,
    pub related_components: Vec<String>,
    pub status: ImplementationStatus,
    pub inferred_from: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Complexity {
    Simple,
    Medium,
    Complex,
    Epic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductRequirementDocument {
    pub title: String,
    pub overview: String,
    pub objectives: Vec<String>,
    pub target_users: Vec<String>,
    pub features: Vec<FeatureDescription>,
    pub technical_requirements: Vec<String>,
    pub business_context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDescription {
    pub name: String,
    pub description: String,
    pub user_value: String,
    pub technical_approach: String,
    pub related_stories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub description: String,
    pub task_type: TaskType,
    pub status: ImplementationStatus,
    pub effort_estimate: Option<String>,
    pub priority: Priority,
    pub related_components: Vec<String>,
    pub dependencies: Vec<String>,
    pub acceptance_criteria: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    Feature,
    Bug,
    Refactor,
    Test,
    Documentation,
    Infrastructure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetadata {
    pub analyzed_at: String,
    pub analyzer_version: String,
    pub files_analyzed: u32,
    pub lines_of_code: u32,
    pub confidence_score: f32,
    pub warnings: Vec<String>,
}

pub trait CodebaseAnalyzer {
    fn analyze(&self, project_path: &str) -> anyhow::Result<CodebaseAnalysis>;
    fn supported_extensions(&self) -> Vec<&str>;
    fn can_analyze(&self, project_path: &str) -> bool;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzerConfig {
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub max_file_size: usize,
    pub analyze_dependencies: bool,
    pub generate_prd: bool,
    pub infer_user_stories: bool,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            include_patterns: vec!["**/*.ts".to_string(), "**/*.tsx".to_string(), "**/*.js".to_string(), "**/*.jsx".to_string()],
            exclude_patterns: vec!["**/node_modules/**".to_string(), "**/dist/**".to_string(), "**/*.test.*".to_string()],
            max_file_size: 1024 * 1024, // 1MB
            analyze_dependencies: true,
            generate_prd: true,
            infer_user_stories: true,
        }
    }
}

// Enhanced data structures for comprehensive analysis

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkAnalysis {
    pub detected_frameworks: Vec<DetectedFramework>,
    pub confidence_scores: std::collections::HashMap<String, f32>,
    pub architecture_pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedFramework {
    pub name: String,
    pub version: Option<String>,
    pub confidence: f32,
    pub evidence: Vec<String>,
    pub usage_extent: UsageExtent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UsageExtent {
    Core,        // Fundamental to the project
    Extensive,   // Used throughout the project
    Moderate,    // Used in several places
    Limited,     // Used minimally
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessContext {
    pub inferred_product_type: String,
    pub confidence: f32,
    pub evidence: Vec<String>,
    pub primary_user_personas: Vec<String>,
    pub user_journeys_discovered: Vec<String>,
    pub business_domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationAnalysis {
    pub api_endpoints: Vec<EndpointAnalysis>,
    pub database_entities: Vec<EntityAnalysis>,
    pub component_relationships: Vec<ComponentRelationship>,
    pub data_flow: Vec<DataFlowAnalysis>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusIntelligence {
    pub completed_features: Vec<FeatureStatus>,
    pub in_progress_features: Vec<FeatureStatus>,
    pub todo_features: Vec<FeatureStatus>,
    pub technical_debt: Vec<TechnicalDebt>,
    pub overall_completion_percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationPoints {
    pub external_services: Vec<ExternalService>,
    pub internal_dependencies: Vec<InternalDependency>,
    pub configuration_files: Vec<ConfigFile>,
    pub environment_variables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointAnalysis {
    pub path: String,
    pub method: String,
    pub controller: String,
    pub purpose: String,
    pub request_schema: Option<String>,
    pub response_schema: Option<String>,
    pub authentication_required: bool,
    pub status: ImplementationStatus,
    pub related_frontend_components: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityAnalysis {
    pub name: String,
    pub file_path: String,
    pub fields: Vec<EntityField>,
    pub relationships: Vec<EntityRelationship>,
    pub purpose: String,
    pub status: ImplementationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityField {
    pub name: String,
    pub field_type: String,
    pub nullable: bool,
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRelationship {
    pub relationship_type: String,
    pub target_entity: String,
    pub relationship_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentRelationship {
    pub source: String,
    pub target: String,
    pub relationship_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlowAnalysis {
    pub source: String,
    pub target: String,
    pub data_type: String,
    pub flow_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureStatus {
    pub name: String,
    pub description: String,
    pub evidence: Vec<String>,
    pub confidence: f32,
    pub related_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalDebt {
    pub description: String,
    pub severity: String,
    pub location: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalService {
    pub name: String,
    pub service_type: String,
    pub usage_context: String,
    pub integration_points: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalDependency {
    pub name: String,
    pub dependency_type: String,
    pub usage_locations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    pub file_path: String,
    pub file_type: String,
    pub purpose: String,
    pub key_configurations: Vec<String>,
}