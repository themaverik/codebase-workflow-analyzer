pub mod crud_analyzer;
pub mod cross_repository_analyzer;

pub use crud_analyzer::CrudAnalyzer;
pub use cross_repository_analyzer::{CrossRepositoryAnalyzer, CrossRepositoryAnalysisResult, ProjectRelationship, ParentProjectContext};