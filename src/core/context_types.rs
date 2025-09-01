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