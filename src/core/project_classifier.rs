use std::collections::HashMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::core::config::{get_config, ProjectTypeConfig};
use crate::core::project_analyzer::{ProjectMetadata, DocumentationAnalysis};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProjectType {
    AnalysisTool,
    WebApplication,
    ApiService,
    Library,
    CliTool,
    Desktop,
    Mobile,
    GameEngine,
    DataPipeline,
    MachineLearning,
    DevOps,
    EmbeddedSystem,
    DatabaseSystem,
    SecurityTool,
    TestingFramework,
    DocumentationSite,
    ConfigurationTool,
    MonitoringSystem,
    BlockchainApp,
    ChatBot,
    MediaProcessor,
    ScientificComputing,
    NetworkingTool,
    Unknown,
}

impl ProjectType {
    pub fn from_config_id(id: &str) -> Self {
        match id {
            "analysis_tool" => ProjectType::AnalysisTool,
            "web_application" => ProjectType::WebApplication,
            "api_service" => ProjectType::ApiService,
            "library" => ProjectType::Library,
            "cli_tool" => ProjectType::CliTool,
            "desktop_application" => ProjectType::Desktop,
            "mobile_application" => ProjectType::Mobile,
            "game_engine" => ProjectType::GameEngine,
            "data_pipeline" => ProjectType::DataPipeline,
            "machine_learning" => ProjectType::MachineLearning,
            "devops_tool" => ProjectType::DevOps,
            "embedded_system" => ProjectType::EmbeddedSystem,
            "database_system" => ProjectType::DatabaseSystem,
            "security_tool" => ProjectType::SecurityTool,
            "testing_framework" => ProjectType::TestingFramework,
            "documentation_site" => ProjectType::DocumentationSite,
            "configuration_tool" => ProjectType::ConfigurationTool,
            "monitoring_system" => ProjectType::MonitoringSystem,
            "blockchain_app" => ProjectType::BlockchainApp,
            "chatbot" => ProjectType::ChatBot,
            "media_processor" => ProjectType::MediaProcessor,
            "scientific_computing" => ProjectType::ScientificComputing,
            "networking_tool" => ProjectType::NetworkingTool,
            _ => ProjectType::Unknown,
        }
    }

    pub fn to_config_id(&self) -> &'static str {
        match self {
            ProjectType::AnalysisTool => "analysis_tool",
            ProjectType::WebApplication => "web_application",
            ProjectType::ApiService => "api_service",
            ProjectType::Library => "library",
            ProjectType::CliTool => "cli_tool",
            ProjectType::Desktop => "desktop_application",
            ProjectType::Mobile => "mobile_application",
            ProjectType::GameEngine => "game_engine",
            ProjectType::DataPipeline => "data_pipeline",
            ProjectType::MachineLearning => "machine_learning",
            ProjectType::DevOps => "devops_tool",
            ProjectType::EmbeddedSystem => "embedded_system",
            ProjectType::DatabaseSystem => "database_system",
            ProjectType::SecurityTool => "security_tool",
            ProjectType::TestingFramework => "testing_framework",
            ProjectType::DocumentationSite => "documentation_site",
            ProjectType::ConfigurationTool => "configuration_tool",
            ProjectType::MonitoringSystem => "monitoring_system",
            ProjectType::BlockchainApp => "blockchain_app",
            ProjectType::ChatBot => "chatbot",
            ProjectType::MediaProcessor => "media_processor",
            ProjectType::ScientificComputing => "scientific_computing",
            ProjectType::NetworkingTool => "networking_tool",
            ProjectType::Unknown => "unknown",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ProjectType::AnalysisTool => "Analysis Tool",
            ProjectType::WebApplication => "Web Application",
            ProjectType::ApiService => "API Service",
            ProjectType::Library => "Library",
            ProjectType::CliTool => "CLI Tool",
            ProjectType::Desktop => "Desktop Application",
            ProjectType::Mobile => "Mobile Application",
            ProjectType::GameEngine => "Game Engine",
            ProjectType::DataPipeline => "Data Pipeline",
            ProjectType::MachineLearning => "Machine Learning",
            ProjectType::DevOps => "DevOps Tool",
            ProjectType::EmbeddedSystem => "Embedded System",
            ProjectType::DatabaseSystem => "Database System",
            ProjectType::SecurityTool => "Security Tool",
            ProjectType::TestingFramework => "Testing Framework",
            ProjectType::DocumentationSite => "Documentation Site",
            ProjectType::ConfigurationTool => "Configuration Tool",
            ProjectType::MonitoringSystem => "Monitoring System",
            ProjectType::BlockchainApp => "Blockchain Application",
            ProjectType::ChatBot => "Chatbot",
            ProjectType::MediaProcessor => "Media Processor",
            ProjectType::ScientificComputing => "Scientific Computing",
            ProjectType::NetworkingTool => "Networking Tool",
            ProjectType::Unknown => "Unknown",
        }
    }

    pub fn analysis_focus(&self) -> &'static str {
        let config = get_config();
        if let Some(project_config) = config.get_project_type(self.to_config_id()) {
            return &project_config.analysis_focus;
        }

        // Fallback analysis focus
        match self {
            ProjectType::AnalysisTool => "tool capabilities and analysis features",
            ProjectType::WebApplication => "user-facing features and business logic",
            ProjectType::ApiService => "API endpoints and data processing",
            ProjectType::Library => "provided abstractions and APIs",
            ProjectType::CliTool => "command-line interface and automation capabilities",
            _ => "general functionality and architecture",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectClassificationResult {
    pub project_type: ProjectType,
    pub confidence: f32,
    pub evidence: Vec<ClassificationEvidence>,
    pub alternative_classifications: Vec<AlternativeClassification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationEvidence {
    pub evidence_type: EvidenceType,
    pub pattern: String,
    pub confidence_contribution: f32,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceType {
    PackageName,
    DependencyPattern,
    FileStructure,
    ReadmeKeywords,
    BinaryConfiguration,
    FrameworkPattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeClassification {
    pub project_type: ProjectType,
    pub confidence: f32,
    pub reasoning: String,
}

/// Strategy pattern for project classification
pub trait ClassificationStrategy: Send + Sync {
    fn classify(&self, metadata: &ProjectMetadata, documentation: &DocumentationAnalysis) -> Result<ProjectClassificationResult>;
    fn get_name(&self) -> &'static str;
}

/// Comprehensive classification strategy using weighted scoring
pub struct WeightedScoringClassifier {
    weights: ClassificationWeights,
}

#[derive(Debug, Clone)]
struct ClassificationWeights {
    package_name: f32,
    dependencies: f32,
    file_structure: f32,
    readme_content: f32,
    binary_config: f32,
}

impl Default for ClassificationWeights {
    fn default() -> Self {
        Self {
            package_name: 0.3,
            dependencies: 0.25,
            file_structure: 0.15,
            readme_content: 0.2,
            binary_config: 0.1,
        }
    }
}

impl WeightedScoringClassifier {
    pub fn new() -> Self {
        Self {
            weights: ClassificationWeights::default(),
        }
    }

    pub fn with_weights(weights: ClassificationWeights) -> Self {
        Self { weights }
    }

    fn calculate_project_type_score(
        &self,
        project_config: &ProjectTypeConfig,
        metadata: &ProjectMetadata,
        documentation: &DocumentationAnalysis,
    ) -> (f32, Vec<ClassificationEvidence>) {
        let mut total_score = 0.0;
        let mut evidence = Vec::new();

        // Score based on indicators from configuration
        for indicator in &project_config.indicators {
            let (score, mut indicator_evidence) = self.score_indicator(indicator, metadata, documentation);
            total_score += score;
            evidence.append(&mut indicator_evidence);
        }

        (total_score, evidence)
    }

    fn score_indicator(
        &self,
        indicator: &crate::core::config::ProjectIndicator,
        metadata: &ProjectMetadata,
        documentation: &DocumentationAnalysis,
    ) -> (f32, Vec<ClassificationEvidence>) {
        let mut evidence = Vec::new();
        let score = match indicator.r#type.as_str() {
            "cargo_binary_name" => {
                self.score_package_name_patterns(&indicator.patterns, &metadata.name, &mut evidence)
            }
            "dependency_pattern" => {
                self.score_dependency_patterns(&indicator.patterns, metadata, &mut evidence)
            }
            "cli_structure" => {
                self.score_cli_structure_patterns(&indicator.patterns, metadata, &mut evidence)
            }
            "readme_keywords" => {
                self.score_readme_patterns(&indicator.patterns, documentation, &mut evidence)
            }
            "package_dependencies" => {
                self.score_package_dependencies(&indicator.patterns, metadata, &mut evidence)
            }
            "file_structure" => {
                // TODO: Implement file structure analysis
                0.0
            }
            "config_files" => {
                // TODO: Implement config file analysis
                0.0
            }
            _ => 0.0,
        };

        (score, evidence)
    }

    fn score_package_name_patterns(
        &self,
        patterns: &[String],
        package_name: &str,
        evidence: &mut Vec<ClassificationEvidence>,
    ) -> f32 {
        let package_name_lower = package_name.to_lowercase();
        let mut score = 0.0;

        for pattern in patterns {
            if package_name_lower.contains(&pattern.to_lowercase()) {
                score += self.weights.package_name;
                evidence.push(ClassificationEvidence {
                    evidence_type: EvidenceType::PackageName,
                    pattern: pattern.clone(),
                    confidence_contribution: self.weights.package_name,
                    source: format!("Package name: {}", package_name),
                });
            }
        }

        score.min(self.weights.package_name) // Cap at weight limit
    }

    fn score_dependency_patterns(
        &self,
        patterns: &[String],
        metadata: &ProjectMetadata,
        evidence: &mut Vec<ClassificationEvidence>,
    ) -> f32 {
        let all_deps: Vec<_> = metadata
            .dependencies
            .keys()
            .chain(metadata.dev_dependencies.keys())
            .collect();

        let mut score = 0.0;
        let mut pattern_matches = 0;

        for pattern in patterns {
            let pattern_lower = pattern.to_lowercase();
            for dep in &all_deps {
                if dep.to_lowercase().contains(&pattern_lower) {
                    pattern_matches += 1;
                    evidence.push(ClassificationEvidence {
                        evidence_type: EvidenceType::DependencyPattern,
                        pattern: pattern.clone(),
                        confidence_contribution: self.weights.dependencies / patterns.len() as f32,
                        source: format!("Dependency: {}", dep),
                    });
                    break; // Only count each pattern once
                }
            }
        }

        if !patterns.is_empty() {
            score = (pattern_matches as f32 / patterns.len() as f32) * self.weights.dependencies;
        }

        score
    }

    fn score_cli_structure_patterns(
        &self,
        patterns: &[String],
        metadata: &ProjectMetadata,
        evidence: &mut Vec<ClassificationEvidence>,
    ) -> f32 {
        let mut score = 0.0;

        for pattern in patterns {
            let pattern_lower = pattern.to_lowercase();

            // Check in dependencies
            let has_cli_dep = metadata
                .dependencies
                .keys()
                .chain(metadata.dev_dependencies.keys())
                .any(|dep| dep.to_lowercase().contains(&pattern_lower));

            if has_cli_dep {
                score += self.weights.binary_config / patterns.len() as f32;
                evidence.push(ClassificationEvidence {
                    evidence_type: EvidenceType::BinaryConfiguration,
                    pattern: pattern.clone(),
                    confidence_contribution: self.weights.binary_config / patterns.len() as f32,
                    source: "CLI framework dependency".to_string(),
                });
            }
        }

        score
    }

    fn score_readme_patterns(
        &self,
        patterns: &[String],
        documentation: &DocumentationAnalysis,
        evidence: &mut Vec<ClassificationEvidence>,
    ) -> f32 {
        let readme_content = match &documentation.readme_content {
            Some(content) => content.to_lowercase(),
            None => return 0.0,
        };

        let mut score = 0.0;
        let mut pattern_matches = 0;

        for pattern in patterns {
            if readme_content.contains(&pattern.to_lowercase()) {
                pattern_matches += 1;
                evidence.push(ClassificationEvidence {
                    evidence_type: EvidenceType::ReadmeKeywords,
                    pattern: pattern.clone(),
                    confidence_contribution: self.weights.readme_content / patterns.len() as f32,
                    source: "README.md content".to_string(),
                });
            }
        }

        if !patterns.is_empty() {
            score = (pattern_matches as f32 / patterns.len() as f32) * self.weights.readme_content;
        }

        score
    }

    fn score_package_dependencies(
        &self,
        patterns: &[String],
        metadata: &ProjectMetadata,
        evidence: &mut Vec<ClassificationEvidence>,
    ) -> f32 {
        // This is similar to dependency_patterns but with different scoring logic
        self.score_dependency_patterns(patterns, metadata, evidence)
    }
}

impl ClassificationStrategy for WeightedScoringClassifier {
    fn classify(&self, metadata: &ProjectMetadata, documentation: &DocumentationAnalysis) -> Result<ProjectClassificationResult> {
        let config = get_config();
        let mut project_scores: Vec<(ProjectType, f32, Vec<ClassificationEvidence>)> = Vec::new();

        // Score each project type
        for project_config in config.get_all_project_types() {
            let (score, evidence) = self.calculate_project_type_score(project_config, metadata, documentation);
            if score > 0.0 {
                let project_type = ProjectType::from_config_id(&project_config.id);
                project_scores.push((project_type, score, evidence));
            }
        }

        // Sort by score descending
        project_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let (primary_type, confidence, evidence) = project_scores
            .first()
            .cloned()
            .unwrap_or((ProjectType::Unknown, 0.0, vec![]));

        let alternative_classifications: Vec<AlternativeClassification> = project_scores
            .iter()
            .skip(1)
            .take(3) // Top 3 alternatives
            .map(|(project_type, score, _)| AlternativeClassification {
                project_type: project_type.clone(),
                confidence: *score,
                reasoning: format!("Alternative classification with {:.2} confidence", score),
            })
            .collect();

        Ok(ProjectClassificationResult {
            project_type: primary_type,
            confidence,
            evidence,
            alternative_classifications,
        })
    }

    fn get_name(&self) -> &'static str {
        "WeightedScoringClassifier"
    }
}

/// Fallback classification strategy for unknown project types
pub struct HeuristicClassifier;

impl ClassificationStrategy for HeuristicClassifier {
    fn classify(&self, metadata: &ProjectMetadata, documentation: &DocumentationAnalysis) -> Result<ProjectClassificationResult> {
        let mut evidence = Vec::new();

        // Simple heuristic classification
        let project_type = if self.is_web_application(metadata) {
            evidence.push(ClassificationEvidence {
                evidence_type: EvidenceType::DependencyPattern,
                pattern: "web framework".to_string(),
                confidence_contribution: 0.7,
                source: "Web framework dependencies".to_string(),
            });
            ProjectType::WebApplication
        } else if self.is_cli_tool(metadata) {
            evidence.push(ClassificationEvidence {
                evidence_type: EvidenceType::DependencyPattern,
                pattern: "CLI framework".to_string(),
                confidence_contribution: 0.6,
                source: "CLI framework dependencies".to_string(),
            });
            ProjectType::CliTool
        } else if self.is_library(metadata) {
            evidence.push(ClassificationEvidence {
                evidence_type: EvidenceType::PackageName,
                pattern: "library structure".to_string(),
                confidence_contribution: 0.5,
                source: "Library project structure".to_string(),
            });
            ProjectType::Library
        } else {
            ProjectType::Unknown
        };

        let confidence = if project_type == ProjectType::Unknown { 0.0 } else { 0.5 };

        Ok(ProjectClassificationResult {
            project_type,
            confidence,
            evidence,
            alternative_classifications: vec![],
        })
    }

    fn get_name(&self) -> &'static str {
        "HeuristicClassifier"
    }
}

impl HeuristicClassifier {
    fn is_web_application(&self, metadata: &ProjectMetadata) -> bool {
        let web_frameworks = ["react", "vue", "angular", "next", "nuxt", "express", "koa", "django", "flask", "fastapi"];
        metadata.dependencies.keys()
            .any(|dep| web_frameworks.iter().any(|fw| dep.to_lowercase().contains(fw)))
    }

    fn is_cli_tool(&self, metadata: &ProjectMetadata) -> bool {
        let cli_frameworks = ["clap", "structopt", "click", "argparse", "commander"];
        metadata.dependencies.keys()
            .chain(metadata.dev_dependencies.keys())
            .any(|dep| cli_frameworks.iter().any(|fw| dep.to_lowercase().contains(fw)))
    }

    fn is_library(&self, metadata: &ProjectMetadata) -> bool {
        // Simple heuristic: if it's not obviously an application, it might be a library
        !self.is_web_application(metadata) && !self.is_cli_tool(metadata)
    }
}

/// Main project classifier using strategy pattern
pub struct ProjectClassifier {
    strategies: Vec<Box<dyn ClassificationStrategy>>,
}

impl ProjectClassifier {
    pub fn new() -> Self {
        Self {
            strategies: vec![
                Box::new(WeightedScoringClassifier::new()),
                Box::new(HeuristicClassifier),
            ],
        }
    }

    pub fn classify_project(&self, metadata: &ProjectMetadata, documentation: &DocumentationAnalysis) -> Result<ProjectClassificationResult> {
        // Try each strategy in order until we get a confident result
        for strategy in &self.strategies {
            let result = strategy.classify(metadata, documentation)?;
            
            // If we have a confident classification, use it
            if result.confidence > 0.3 {
                return Ok(result);
            }
        }

        // If no strategy is confident, return the best result from the primary strategy
        self.strategies[0].classify(metadata, documentation)
    }

    pub fn classify_from_metadata(metadata: &ProjectMetadata) -> ProjectType {
        let documentation = DocumentationAnalysis {
            readme_content: None,
            purpose_keywords: vec![],
            domain_keywords: vec![],
            setup_instructions: vec![],
            api_documentation: vec![],
        };

        let classifier = Self::new();
        classifier
            .classify_project(metadata, &documentation)
            .map(|result| result.project_type)
            .unwrap_or(ProjectType::Unknown)
    }
}

impl Default for ProjectClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_metadata(name: &str, deps: Vec<(&str, &str)>) -> ProjectMetadata {
        let mut dependencies = HashMap::new();
        for (dep, version) in deps {
            dependencies.insert(dep.to_string(), version.to_string());
        }

        ProjectMetadata {
            name: name.to_string(),
            version: Some("1.0.0".to_string()),
            description: Some("Test project".to_string()),
            authors: vec![],
            dependencies,
            dev_dependencies: HashMap::new(),
            license: None,
            repository: None,
            package_manager: crate::core::project_analyzer::PackageManager::Npm,
        }
    }

    fn create_test_documentation(readme: &str) -> DocumentationAnalysis {
        DocumentationAnalysis {
            readme_content: Some(readme.to_string()),
            purpose_keywords: vec!["test".to_string()],
            domain_keywords: vec![],
            setup_instructions: vec![],
            api_documentation: vec![],
        }
    }

    #[test]
    fn test_project_type_conversion() {
        assert_eq!(ProjectType::from_config_id("analysis_tool"), ProjectType::AnalysisTool);
        assert_eq!(ProjectType::AnalysisTool.to_config_id(), "analysis_tool");
        assert_eq!(ProjectType::AnalysisTool.display_name(), "Analysis Tool");
    }

    #[test]
    fn test_weighted_scoring_classifier() {
        let classifier = WeightedScoringClassifier::new();
        
        // Test analysis tool classification
        let metadata = create_test_metadata("codebase-analyzer", vec![("tree-sitter", "0.20"), ("clap", "4.0")]);
        let documentation = create_test_documentation("This tool analyzes codebases and detects frameworks");
        
        let result = classifier.classify(&metadata, &documentation).unwrap();
        assert_eq!(result.project_type, ProjectType::AnalysisTool);
        assert!(result.confidence > 0.0);
        assert!(!result.evidence.is_empty());
    }

    #[test]
    fn test_heuristic_classifier() {
        let classifier = HeuristicClassifier;
        
        // Test web application classification
        let metadata = create_test_metadata("my-web-app", vec![("react", "18.0"), ("axios", "0.27")]);
        let documentation = create_test_documentation("A web application built with React");
        
        let result = classifier.classify(&metadata, &documentation).unwrap();
        assert_eq!(result.project_type, ProjectType::WebApplication);
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_project_classifier_integration() {
        let classifier = ProjectClassifier::new();
        
        // Test CLI tool classification
        let metadata = create_test_metadata("my-cli-tool", vec![("clap", "4.0"), ("serde", "1.0")]);
        let documentation = create_test_documentation("A command-line utility for developers");
        
        let result = classifier.classify_project(&metadata, &documentation).unwrap();
        assert_eq!(result.project_type, ProjectType::CliTool);
        assert!(result.confidence > 0.3);
    }

    #[test]
    fn test_unknown_project_classification() {
        let classifier = ProjectClassifier::new();
        
        // Test with minimal information
        let metadata = create_test_metadata("unknown-project", vec![]);
        let documentation = create_test_documentation("Some project without clear indicators");
        
        let result = classifier.classify_project(&metadata, &documentation).unwrap();
        // Should still return a result, even if it's Unknown or low confidence
        assert!(result.confidence >= 0.0);
    }
}