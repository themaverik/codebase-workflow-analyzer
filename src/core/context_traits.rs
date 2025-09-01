use async_trait::async_trait;
use anyhow::Result;
use std::path::Path;
use crate::core::context_types::*;
use crate::core::types::AstSegment;
use crate::core::project_analyzer::ProjectMetadata;

#[async_trait]
pub trait ContextProvider: Send + Sync {
    async fn initialize(&mut self, project_path: &Path) -> Result<()>;
    async fn get_project_context(&self) -> Result<ProjectContext>;
    fn get_context_confidence(&self) -> f32;
}

#[async_trait]
pub trait FileContextProvider: Send + Sync {
    async fn extract_file_context(&self, file_path: &Path, project_context: &ProjectContext) -> Result<FileContext>;
    async fn batch_extract_file_contexts(&self, file_paths: &[&Path], project_context: &ProjectContext) -> Result<Vec<FileContext>>;
    fn classify_file_type(&self, file_path: &Path) -> FileType;
    fn determine_file_role(&self, file_path: &Path, project_context: &ProjectContext) -> FileRole;
}

#[async_trait]
pub trait SegmentContextProvider: Send + Sync {
    async fn extract_segment_context(&self, segment: &AstSegment, file_context: &FileContext) -> Result<SegmentContext>;
    async fn batch_extract_segment_contexts(&self, segments: &[AstSegment], file_context: &FileContext) -> Result<Vec<SegmentContext>>;
    fn classify_segment_type(&self, segment: &AstSegment) -> SegmentType;
    fn calculate_business_relevance(&self, segment: &AstSegment, file_context: &FileContext) -> f32;
}

#[async_trait]
pub trait CrossReferenceProvider: Send + Sync {
    async fn build_cross_references(&self, segment_contexts: &[SegmentContext]) -> Result<CrossReferenceMap>;
    async fn find_functional_dependencies(&self, segment: &SegmentContext, all_segments: &[SegmentContext]) -> Result<Vec<SegmentId>>;
    async fn trace_data_flow(&self, segment: &SegmentContext, all_segments: &[SegmentContext]) -> Result<Vec<SegmentId>>;
    async fn identify_architectural_relationships(&self, segment: &SegmentContext, all_segments: &[SegmentContext]) -> Result<Vec<SegmentId>>;
}

#[async_trait]
pub trait ContextEnhancer: Send + Sync {
    async fn enhance_segment_context(&self, segment_context: SegmentContext, project_context: &ProjectContext, cross_references: &CrossReferenceMap) -> Result<EnhancedSegmentContext>;
    async fn extract_business_hints(&self, segment_context: &SegmentContext, project_context: &ProjectContext) -> Result<Vec<String>>;
    async fn determine_architectural_context(&self, segment_context: &SegmentContext, project_context: &ProjectContext) -> Result<ArchitecturalContext>;
}

#[async_trait]
pub trait ContextValidator: Send + Sync {
    async fn validate_project_context(&self, context: &ProjectContext) -> Result<bool>;
    async fn validate_file_context(&self, context: &FileContext, project_context: &ProjectContext) -> Result<bool>;
    async fn validate_segment_context(&self, context: &SegmentContext, file_context: &FileContext) -> Result<bool>;
    async fn validate_cross_references(&self, cross_refs: &CrossReferenceMap, segment_contexts: &[SegmentContext]) -> Result<bool>;
}

pub trait ContextCache: Send + Sync {
    fn get_project_context(&self, project_id: &ProjectId) -> Option<ProjectContext>;
    fn set_project_context(&mut self, project_id: ProjectId, context: ProjectContext);
    fn get_file_context(&self, file_path: &FileId) -> Option<FileContext>;
    fn set_file_context(&mut self, file_path: FileId, context: FileContext);
    fn get_segment_context(&self, segment_id: &SegmentId) -> Option<SegmentContext>;
    fn set_segment_context(&mut self, segment_id: SegmentId, context: SegmentContext);
    fn invalidate_project(&mut self, project_id: &ProjectId);
    fn clear_expired(&mut self);
}