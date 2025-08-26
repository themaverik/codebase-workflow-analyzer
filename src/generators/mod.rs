use crate::core::CodebaseAnalysis;
use crate::intelligence::{IntelligentAnalysis, IntelligenceEngine};
use anyhow::Result;

pub mod markdown;
pub mod prd;
pub mod stories;
pub mod technical_docs;
pub mod executive_summary;

pub use markdown::MarkdownGenerator;
pub use prd::PRDGenerator;
pub use stories::UserStoryGenerator;
pub use technical_docs::TechnicalDocumentationGenerator;
pub use executive_summary::ExecutiveSummaryGenerator;

pub trait DocumentGenerator {
    fn generate(&self, analysis: &CodebaseAnalysis, intelligent_analysis: Option<&IntelligentAnalysis>) -> Result<String>;
    fn get_file_extension(&self) -> &str;
    fn get_document_type(&self) -> DocumentType;
}

#[derive(Debug, Clone)]
pub enum DocumentType {
    Markdown,
    ProductRequirementDocument,
    UserStories,
    TechnicalDocumentation,
    ExecutiveSummary,
}

pub struct DocumentGeneratorFactory;

impl DocumentGeneratorFactory {
    pub fn create_generator(doc_type: DocumentType) -> Box<dyn DocumentGenerator> {
        match doc_type {
            DocumentType::Markdown => Box::new(MarkdownGenerator::new()),
            DocumentType::ProductRequirementDocument => Box::new(PRDGenerator::new()),
            DocumentType::UserStories => Box::new(UserStoryGenerator::new()),
            DocumentType::TechnicalDocumentation => Box::new(TechnicalDocumentationGenerator::new()),
            DocumentType::ExecutiveSummary => Box::new(ExecutiveSummaryGenerator::new()),
        }
    }

    pub fn generate_all_documents(
        analysis: &CodebaseAnalysis,
        output_dir: &str,
    ) -> Result<Vec<GeneratedDocument>> {
        let intelligence = IntelligenceEngine::new();
        let intelligent_analysis = intelligence.enhance_analysis(analysis);
        
        let doc_types = vec![
            DocumentType::ExecutiveSummary,
            DocumentType::ProductRequirementDocument,
            DocumentType::UserStories,
            DocumentType::TechnicalDocumentation,
            DocumentType::Markdown,
        ];

        let mut generated_docs = Vec::new();

        for doc_type in doc_types {
            let generator = Self::create_generator(doc_type.clone());
            let content = generator.generate(analysis, Some(&intelligent_analysis))?;
            let filename = Self::generate_filename(&analysis.project_name, &doc_type, generator.get_file_extension());
            let file_path = std::path::Path::new(output_dir).join(&filename);

            std::fs::write(&file_path, &content)?;

            generated_docs.push(GeneratedDocument {
                document_type: doc_type,
                filename,
                file_path: file_path.to_string_lossy().to_string(),
                content,
            });
        }

        Ok(generated_docs)
    }

    fn generate_filename(project_name: &str, doc_type: &DocumentType, extension: &str) -> String {
        let type_name = match doc_type {
            DocumentType::Markdown => "analysis",
            DocumentType::ProductRequirementDocument => "prd",
            DocumentType::UserStories => "user-stories",
            DocumentType::TechnicalDocumentation => "technical-docs",
            DocumentType::ExecutiveSummary => "executive-summary",
        };
        
        let clean_project_name = project_name.to_lowercase()
            .replace(" ", "-")
            .replace("_", "-");
        
        format!("{}-{}.{}", clean_project_name, type_name, extension)
    }
}

pub struct GeneratedDocument {
    pub document_type: DocumentType,
    pub filename: String,
    pub file_path: String,
    pub content: String,
}

// Helper functions for consistent formatting
pub fn format_priority(priority: &crate::core::Priority) -> &str {
    match priority {
        crate::core::Priority::Critical => "🔴 Critical",
        crate::core::Priority::High => "🟠 High",
        crate::core::Priority::Medium => "🟡 Medium",
        crate::core::Priority::Low => "🟢 Low",
    }
}

pub fn format_complexity(complexity: &crate::core::Complexity) -> &str {
    match complexity {
        crate::core::Complexity::Simple => "🟢 Simple",
        crate::core::Complexity::Medium => "🟡 Medium",
        crate::core::Complexity::Complex => "🟠 Complex",
        crate::core::Complexity::Epic => "🔴 Epic",
    }
}

pub fn format_status(status: &crate::core::ImplementationStatus) -> &str {
    match status {
        crate::core::ImplementationStatus::Complete => " Complete",
        crate::core::ImplementationStatus::InProgress => " In Progress",
        crate::core::ImplementationStatus::Todo => " Todo",
        crate::core::ImplementationStatus::Incomplete => "⚠ Incomplete",
    }
}

pub fn format_component_type(component_type: &crate::core::ComponentType) -> &str {
    match component_type {
        crate::core::ComponentType::Page => " Page",
        crate::core::ComponentType::Layout => "📐 Layout",
        crate::core::ComponentType::Form => "📝 Form",
        crate::core::ComponentType::Display => "🖼 Display",
        crate::core::ComponentType::Navigation => "🧭 Navigation",
        crate::core::ComponentType::Modal => "🪟 Modal",
        crate::core::ComponentType::Utility => " Utility",
        crate::core::ComponentType::Hook => "🪝 Hook",
        crate::core::ComponentType::Context => "🗂 Context",
        crate::core::ComponentType::Service => "⚙ Service",
    }
}