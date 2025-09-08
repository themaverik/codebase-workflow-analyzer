use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use crate::core::types::{AstSegment, BusinessDomain};
use crate::core::project_analyzer::ProjectMetadata;

pub type SegmentId = String;
pub type FileId = PathBuf;
pub type ProjectId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    pub id: ProjectId,
    pub metadata: ProjectMetadata,
    pub project_type: String,
    pub business_domains: Vec<BusinessDomain>,
    pub entry_points: Vec<PathBuf>,
    pub documentation_summary: String,
    pub architectural_patterns: Vec<String>,
    pub dependency_overview: DependencyOverview,
    pub confidence: f32,
    pub created_at: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContext {
    pub file_path: PathBuf,
    pub file_type: FileType,
    pub role_in_project: FileRole,
    pub language: Option<String>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub key_patterns: Vec<String>,
    pub related_files: Vec<PathBuf>,
    pub business_relevance: f32,
    pub last_modified: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentContext {
    pub segment_id: SegmentId,
    pub segment: AstSegment,
    pub file_context: FileContext,
    pub segment_type: SegmentType,
    pub business_purpose: Option<String>,
    pub dependencies: Vec<SegmentId>,
    pub dependents: Vec<SegmentId>,
    pub confidence: f32,
    pub extracted_at: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSegmentContext {
    pub segment_context: SegmentContext,
    pub project_context: ProjectContext,
    pub related_segments: Vec<SegmentContext>,
    pub cross_references: Vec<CrossReference>,
    pub business_hints: Vec<String>,
    pub architectural_context: ArchitecturalContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossReference {
    pub source_segment: SegmentId,
    pub target_segment: SegmentId,
    pub reference_type: CrossReferenceType,
    pub strength: f32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalContext {
    pub layer: ArchitecturalLayer,
    pub patterns: Vec<String>,
    pub responsibilities: Vec<String>,
    pub interaction_style: InteractionStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyOverview {
    pub direct_dependencies: HashMap<String, String>,
    pub framework_dependencies: Vec<String>,
    pub development_dependencies: Vec<String>,
    pub dependency_categories: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileType {
    SourceCode,
    Configuration,
    Documentation,
    Test,
    Build,
    Asset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileRole {
    EntryPoint,
    CoreLogic,
    Configuration,
    Documentation,
    Testing,
    Utility,
    Integration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SegmentType {
    FunctionDefinition,
    ClassDefinition,
    InterfaceDefinition,
    ModuleImport,
    ConfigurationBlock,
    ApiEndpoint,
    DataStructure,
    BusinessLogic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossReferenceType {
    FunctionalDependency,
    DataFlow,
    ArchitecturalRelationship,
    NamingConvention,
    BusinessRule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchitecturalLayer {
    Presentation,
    Business,
    Data,
    Infrastructure,
    Cross,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionStyle {
    Synchronous,
    Asynchronous,
    EventDriven,
    RequestResponse,
    Pipeline,
}

#[derive(Debug, Clone)]
pub struct CrossReferenceMap {
    pub functional_dependencies: HashMap<SegmentId, Vec<SegmentId>>,
    pub data_flow: HashMap<SegmentId, Vec<SegmentId>>,
    pub architectural_relationships: HashMap<SegmentId, Vec<SegmentId>>,
    pub naming_conventions: HashMap<String, Vec<SegmentId>>,
}

impl CrossReferenceMap {
    pub fn new() -> Self {
        Self {
            functional_dependencies: HashMap::new(),
            data_flow: HashMap::new(),
            architectural_relationships: HashMap::new(),
            naming_conventions: HashMap::new(),
        }
    }

    pub fn add_functional_dependency(&mut self, source: SegmentId, target: SegmentId) {
        self.functional_dependencies
            .entry(source)
            .or_insert_with(Vec::new)
            .push(target);
    }

    pub fn add_data_flow(&mut self, source: SegmentId, target: SegmentId) {
        self.data_flow
            .entry(source)
            .or_insert_with(Vec::new)
            .push(target);
    }

    pub fn get_related_segments(&self, segment_id: &SegmentId) -> Vec<SegmentId> {
        let mut related = Vec::new();
        
        if let Some(deps) = self.functional_dependencies.get(segment_id) {
            related.extend(deps.clone());
        }
        
        if let Some(flows) = self.data_flow.get(segment_id) {
            related.extend(flows.clone());
        }
        
        if let Some(arch) = self.architectural_relationships.get(segment_id) {
            related.extend(arch.clone());
        }
        
        related.sort();
        related.dedup();
        related
    }
}

// Edge case analysis and validation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub warnings: Vec<ValidationWarning>,
    pub circular_dependencies: Option<Vec<SegmentId>>,
    pub missing_references: Vec<MissingReference>,
    pub orphaned_contexts: Vec<SegmentId>,
    pub validation_timestamp: SystemTime,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            warnings: Vec::new(),
            circular_dependencies: None,
            missing_references: Vec::new(),
            orphaned_contexts: Vec::new(),
            validation_timestamp: SystemTime::now(),
        }
    }
    
    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.is_valid = false;
        self.warnings.push(warning);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub warning_type: ValidationWarningType,
    pub message: String,
    pub affected_segments: Vec<SegmentId>,
    pub severity: ValidationSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationWarningType {
    CircularDependency,
    MissingReference,
    OrphanedContext,
    WeakCrossReference,
    IncompleteDependencyGraph,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingReference {
    pub source_segment: SegmentId,
    pub missing_target: String,
    pub reference_type: CrossReferenceType,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeCaseAnalysisResult {
    pub edge_cases_detected: Vec<EdgeCase>,
    pub project_characteristics: ProjectCharacteristics,
    pub handling_recommendations: Vec<HandlingRecommendation>,
    pub performance_impact: PerformanceImpact,
    pub analysis_timestamp: SystemTime,
}

impl EdgeCaseAnalysisResult {
    pub fn new() -> Self {
        Self {
            edge_cases_detected: Vec::new(),
            project_characteristics: ProjectCharacteristics::default(),
            handling_recommendations: Vec::new(),
            performance_impact: PerformanceImpact::default(),
            analysis_timestamp: SystemTime::now(),
        }
    }
}

impl Default for ProjectCharacteristics {
    fn default() -> Self {
        Self {
            total_files: 0,
            total_directories: 0,
            languages_detected: Vec::new(),
            max_nesting_depth: 0,
            has_symbolic_links: false,
            has_permission_issues: false,
            estimated_analysis_time: std::time::Duration::from_secs(0),
        }
    }
}

impl Default for PerformanceImpact {
    fn default() -> Self {
        Self {
            expected_analysis_duration: std::time::Duration::from_secs(0),
            memory_usage_estimate: MemoryUsageEstimate::Low,
            processing_complexity: ProcessingComplexity::Simple,
            parallelization_potential: ParallelizationPotential::Moderate,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeCase {
    pub case_type: EdgeCaseType,
    pub description: String,
    pub affected_components: Vec<String>,
    pub severity: EdgeCaseSeverity,
    pub mitigation_applied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeCaseType {
    EmptyProject,
    SingleFileProject,
    LargeProject,
    MultiLanguageProject,
    SymbolicLinkProject,
    DeepNesting,
    CircularSymlinks,
    BrokenSymlinks,
    PermissionIssues,
    EncodingIssues,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeCaseSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCharacteristics {
    pub total_files: usize,
    pub total_directories: usize,
    pub languages_detected: Vec<String>,
    pub max_nesting_depth: usize,
    pub has_symbolic_links: bool,
    pub has_permission_issues: bool,
    pub estimated_analysis_time: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandlingRecommendation {
    pub edge_case_type: EdgeCaseType,
    pub recommendation: String,
    pub priority: RecommendationPriority,
    pub estimated_effort: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceImpact {
    pub expected_analysis_duration: std::time::Duration,
    pub memory_usage_estimate: MemoryUsageEstimate,
    pub processing_complexity: ProcessingComplexity,
    pub parallelization_potential: ParallelizationPotential,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryUsageEstimate {
    Low,      // < 100MB
    Medium,   // 100MB - 500MB
    High,     // 500MB - 1GB
    VeryHigh, // > 1GB
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingComplexity {
    Simple,    // Linear processing
    Moderate,  // Some branching logic
    Complex,   // Multiple decision points
    VeryComplex, // Recursive or highly branched
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParallelizationPotential {
    None,      // Must be sequential
    Limited,   // Some parts can be parallel
    Moderate,  // Good parallelization opportunities
    High,      // Highly parallelizable
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepNestingInfo {
    pub max_depth: usize,
    pub paths_exceeding_threshold: Vec<PathBuf>,
    pub nesting_threshold: usize,
    pub performance_warning: bool,
}

// Context quality and performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextQualityMetrics {
    pub completeness_score: f32,          // 0.0 to 1.0
    pub consistency_score: f32,           // Cross-reference consistency
    pub coverage_score: f32,              // How well the context covers the codebase
    pub performance_score: f32,           // Analysis efficiency
    pub confidence_distribution: ConfidenceDistribution,
    pub context_depth: ContextDepthMetrics,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceDistribution {
    pub high_confidence_segments: usize,    // > 0.8
    pub medium_confidence_segments: usize,  // 0.5 - 0.8
    pub low_confidence_segments: usize,     // < 0.5
    pub average_confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextDepthMetrics {
    pub shallow_contexts: usize,     // Basic file/segment info only
    pub medium_contexts: usize,      // With some cross-references
    pub deep_contexts: usize,        // Rich cross-references and metadata
    pub average_depth: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEnhancementResult {
    pub initial_quality: ContextQualityMetrics,
    pub enhanced_quality: ContextQualityMetrics,
    pub improvements_applied: Vec<ContextImprovement>,
    pub performance_gains: PerformanceGains,
    pub enhancement_timestamp: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextImprovement {
    pub improvement_type: ImprovementType,
    pub description: String,
    pub impact_score: f32,
    pub segments_affected: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImprovementType {
    CrossReferenceEnrichment,
    ConfidenceRecalibration,
    MissingContextFilling,
    PerformanceOptimization,
    ConsistencyNormalization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceGains {
    pub analysis_time_reduction: std::time::Duration,
    pub memory_usage_reduction: f32,  // Percentage
    pub cache_hit_rate_improvement: f32,
    pub throughput_improvement: f32,  // Segments processed per second
}