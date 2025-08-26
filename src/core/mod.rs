use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebaseAnalysis {
    pub project_name: String,
    pub project_type: ProjectType,
    pub components: Vec<ComponentInfo>,
    pub user_stories: Vec<UserStory>,
    pub prd: ProductRequirementDocument,
    pub tasks: Vec<Task>,
    pub analysis_metadata: AnalysisMetadata,
    // pub intelligent_insights: Option<crate::intelligence::IntelligentAnalysis>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectType {
    React,
    SpringBoot,
    Django,
    Flask,
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