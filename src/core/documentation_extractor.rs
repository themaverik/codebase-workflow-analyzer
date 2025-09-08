use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
// Using standard println for logging - could be upgraded to proper logging later

use crate::core::config::get_config;

/// Types of documentation sections that can be extracted
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocumentationSection {
    Description,
    Installation,
    Usage,
    Api,
    Architecture,
    Contributing,
    Changelog,
    License,
    Examples,
    Configuration,
    Troubleshooting,
}

impl DocumentationSection {
    /// Convert section to lowercase string for matching
    fn as_str(&self) -> &str {
        match self {
            Self::Description => "description",
            Self::Installation => "installation",
            Self::Usage => "usage",
            Self::Api => "api",
            Self::Architecture => "architecture",
            Self::Contributing => "contributing",
            Self::Changelog => "changelog",
            Self::License => "license",
            Self::Examples => "examples",
            Self::Configuration => "configuration",
            Self::Troubleshooting => "troubleshooting",
        }
    }
}

/// Information extracted from project documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedDocumentationInfo {
    pub project_description: Option<String>,
    pub installation_instructions: Vec<String>,
    pub usage_examples: Vec<String>,
    pub api_documentation: Vec<String>,
    pub architecture_info: Vec<String>,
    pub contributing_guidelines: Vec<String>,
    pub technologies: Vec<TechnologyInfo>,
    pub setup_commands: Vec<String>,
    pub validation_conflicts: Vec<ValidationConflict>,
    pub confidence_score: f32,
    pub documentation_coverage: DocumentationCoverage,
}

/// Detailed technology information found in documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnologyInfo {
    pub name: String,
    pub version: Option<String>,
    pub purpose: String,
    pub installation_method: Option<String>,
    pub configuration_notes: Vec<String>,
    pub confidence: f32,
}

/// Conflicts between documentation claims and code analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConflict {
    pub conflict_type: ConflictType,
    pub documentation_claim: String,
    pub code_reality: String,
    pub severity: ConflictSeverity,
    pub file_location: Option<String>,
    pub recommendation: String,
}

/// Types of conflicts between docs and code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    FrameworkMismatch,
    VersionMismatch,
    MissingDependency,
    ObsoleteInstructions,
    IncorrectApiUsage,
    MissingConfiguration,
}

/// Severity of validation conflicts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictSeverity {
    Low,      // Minor inconsistencies
    Medium,   // Important but not blocking
    High,     // Likely to cause issues
    Critical, // Blocking or dangerous
}

/// Documentation coverage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationCoverage {
    pub sections_found: Vec<DocumentationSection>,
    pub sections_missing: Vec<DocumentationSection>,
    pub completeness_score: f32,
    pub quality_indicators: Vec<String>,
    pub improvement_suggestions: Vec<String>,
}

/// Configuration for documentation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationAnalysisConfig {
    pub enable_readme_extraction: bool,
    pub enable_docs_scanning: bool,
    pub validate_against_code: bool,
    pub priority_code_over_docs: bool,
    pub scan_patterns: Vec<String>,
    pub max_file_size_kb: u64,
    pub extract_sections: Vec<String>,
}

impl Default for DocumentationAnalysisConfig {
    fn default() -> Self {
        Self {
            enable_readme_extraction: true,
            enable_docs_scanning: true,
            validate_against_code: true,
            priority_code_over_docs: true,
            scan_patterns: vec![
                "README.md".to_string(),
                "README.rst".to_string(),
                "docs/**/*.md".to_string(),
                "CONTRIBUTING.md".to_string(),
                "API.md".to_string(),
                "CHANGELOG.md".to_string(),
            ],
            max_file_size_kb: 500,
            extract_sections: vec![
                "description".to_string(),
                "installation".to_string(),
                "usage".to_string(),
                "api".to_string(),
                "architecture".to_string(),
                "contributing".to_string(),
            ],
        }
    }
}

/// Main documentation extractor with configurable analysis
pub struct DocumentationExtractor {
    config: DocumentationAnalysisConfig,
    section_patterns: HashMap<DocumentationSection, Regex>,
    technology_patterns: HashMap<String, Regex>,
    setup_patterns: Vec<Regex>,
}

impl DocumentationExtractor {
    /// Create new documentation extractor with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(DocumentationAnalysisConfig::default())
    }

    /// Create documentation extractor with custom configuration
    pub fn with_config(config: DocumentationAnalysisConfig) -> Result<Self> {
        let mut extractor = Self {
            config,
            section_patterns: HashMap::new(),
            technology_patterns: HashMap::new(),
            setup_patterns: Vec::new(),
        };

        extractor.compile_patterns()
            .context("Failed to compile documentation extraction patterns")?;

        // Debug: Initialized DocumentationExtractor with patterns
        Ok(extractor)
    }

    /// Load configuration from the analyzer config file
    pub fn from_global_config() -> Result<Self> {
        let _global_config = get_config();
        // TODO: Extract documentation analysis config from global config
        // For now, use default config
        Self::new()
    }

    /// Compile regex patterns for efficient section and technology detection
    fn compile_patterns(&mut self) -> Result<()> {
        // Compile section header patterns
        let sections = vec![
            (DocumentationSection::Description, r"(?i)^#+\s*(description|about|overview|what\s+is)"),
            (DocumentationSection::Installation, r"(?i)^#+\s*(installation|install|setup|getting\s+started)"),
            (DocumentationSection::Usage, r"(?i)^#+\s*(usage|how\s+to\s+use|quick\s+start|examples?)"),
            (DocumentationSection::Api, r"(?i)^#+\s*(api|reference|endpoints?)"),
            (DocumentationSection::Architecture, r"(?i)^#+\s*(architecture|design|structure)"),
            (DocumentationSection::Contributing, r"(?i)^#+\s*(contributing|development|contribute)"),
            (DocumentationSection::Changelog, r"(?i)^#+\s*(changelog|changes|history)"),
            (DocumentationSection::License, r"(?i)^#+\s*(license|licensing)"),
            (DocumentationSection::Examples, r"(?i)^#+\s*(examples?|samples?)"),
            (DocumentationSection::Configuration, r"(?i)^#+\s*(configuration|config|settings)"),
            (DocumentationSection::Troubleshooting, r"(?i)^#+\s*(troubleshooting|faq|problems?)"),
        ];

        for (section, pattern) in sections {
            let regex = Regex::new(pattern)
                .context(format!("Failed to compile section pattern for {:?}", section))?;
            self.section_patterns.insert(section, regex);
        }

        // Compile technology detection patterns
        let technologies = vec![
            ("React", r"(?i)\b(react|jsx|reactjs)\b"),
            ("Next.js", r"(?i)\b(next\.?js|nextjs)\b"),
            ("Vue.js", r"(?i)\b(vue\.?js|vuejs)\b"),
            ("Angular", r"(?i)\b(angular|ng-)\b"),
            ("Node.js", r"(?i)\b(node\.?js|nodejs|npm|yarn)\b"),
            ("Express", r"(?i)\b(express\.?js|expressjs)\b"),
            ("NestJS", r"(?i)\b(nest\.?js|nestjs)\b"),
            ("Spring Boot", r"(?i)\b(spring\s+boot|springboot)\b"),
            ("Django", r"(?i)\bdjango\b"),
            ("Flask", r"(?i)\bflask\b"),
            ("FastAPI", r"(?i)\b(fastapi|fast\s+api)\b"),
            ("Docker", r"(?i)\b(docker|dockerfile|docker-compose)\b"),
            ("Kubernetes", r"(?i)\b(kubernetes|k8s|kubectl)\b"),
            ("PostgreSQL", r"(?i)\b(postgresql|postgres|psql)\b"),
            ("MySQL", r"(?i)\bmysql\b"),
            ("MongoDB", r"(?i)\bmongodb\b"),
            ("Redis", r"(?i)\bredis\b"),
            ("GraphQL", r"(?i)\bgraphql\b"),
            ("REST API", r"(?i)\b(rest|restful)\s+(api|service)\b"),
            ("TypeScript", r"(?i)\btypescript\b"),
            ("Python", r"(?i)\bpython\b"),
            ("Java", r"(?i)\bjava\b"),
            ("Go", r"(?i)\b(golang|go\s+lang)\b"),
            ("Rust", r"(?i)\brust\b"),
        ];

        for (tech_name, pattern) in technologies {
            let regex = Regex::new(pattern)
                .context(format!("Failed to compile technology pattern for {}", tech_name))?;
            self.technology_patterns.insert(tech_name.to_string(), regex);
        }

        // Compile setup command patterns
        let setup_patterns = vec![
            r"(?i)\b(npm|yarn)\s+(install|i)\b",
            r"(?i)\bpip\s+install\b",
            r"(?i)\bmvn\s+(clean\s+)?install\b",
            r"(?i)\bgradle\s+build\b",
            r"(?i)\bcargo\s+build\b",
            r"(?i)\bmake\s+(install|build)\b",
            r"(?i)\bdocker\s+(build|run)\b",
            r"(?i)\bgit\s+clone\b",
        ];

        for pattern in setup_patterns {
            let regex = Regex::new(pattern)
                .context(format!("Failed to compile setup pattern: {}", pattern))?;
            self.setup_patterns.push(regex);
        }

        println!("Compiled {} section patterns, {} technology patterns, {} setup patterns",
                 self.section_patterns.len(),
                 self.technology_patterns.len(),
                 self.setup_patterns.len());

        Ok(())
    }

    /// Extract documentation from multiple sources with deduplication
    pub fn extract_multi_source_documentation(&self, project_path: &Path, external_paths: &[PathBuf]) -> Result<ExtractedDocumentationInfo> {
        println!("Starting multi-source documentation extraction...");
        println!("   Primary project: {}", project_path.display());
        println!("   External sources: {}", external_paths.len());

        let mut all_sources = vec![project_path.to_path_buf()];
        all_sources.extend_from_slice(external_paths);
        
        let mut combined_info = ExtractedDocumentationInfo {
            project_description: None,
            installation_instructions: Vec::new(),
            usage_examples: Vec::new(),
            api_documentation: Vec::new(),
            architecture_info: Vec::new(),
            contributing_guidelines: Vec::new(),
            technologies: Vec::new(),
            setup_commands: Vec::new(),
            validation_conflicts: Vec::new(),
            confidence_score: 0.0,
            documentation_coverage: DocumentationCoverage {
                sections_found: Vec::new(),
                sections_missing: Vec::new(),
                completeness_score: 0.0,
                quality_indicators: Vec::new(),
                improvement_suggestions: Vec::new(),
            },
        };

        let mut processed_contents = std::collections::HashSet::new();
        let mut total_confidence = 0.0;
        let mut source_count = 0;

        for (i, source_path) in all_sources.iter().enumerate() {
            println!("   Processing source {}/{}: {}", i + 1, all_sources.len(), source_path.display());
            
            match self.extract_documentation(source_path) {
                Ok(source_info) => {
                    // Merge documentation with deduplication
                    self.merge_documentation_info(&mut combined_info, source_info, &mut processed_contents, i == 0)?;
                    total_confidence += combined_info.confidence_score;
                    source_count += 1;
                },
                Err(e) => {
                    println!("   Warning: Failed to process {}: {}", source_path.display(), e);
                    // Continue with other sources instead of failing completely
                }
            }
        }

        // Calculate final confidence score as weighted average
        combined_info.confidence_score = if source_count > 0 {
            total_confidence / source_count as f32
        } else {
            0.0
        };
        
        println!("Multi-source extraction complete:");
        println!("   Final confidence: {:.1}%", combined_info.confidence_score * 100.0);
        
        Ok(combined_info)
    }

    /// Merge documentation information from multiple sources with deduplication
    fn merge_documentation_info(
        &self,
        combined_info: &mut ExtractedDocumentationInfo,
        source_info: ExtractedDocumentationInfo,
        processed_contents: &mut std::collections::HashSet<String>,
        is_primary_source: bool,
    ) -> Result<()> {
        // Merge project description (prefer primary source or first found)
        if combined_info.project_description.is_none() || is_primary_source {
            if let Some(desc) = source_info.project_description {
                combined_info.project_description = Some(desc);
            }
        }

        // Merge installation instructions with deduplication
        self.merge_string_vectors(
            &mut combined_info.installation_instructions,
            source_info.installation_instructions,
            processed_contents,
        );

        // Merge usage examples with deduplication
        self.merge_string_vectors(
            &mut combined_info.usage_examples,
            source_info.usage_examples,
            processed_contents,
        );

        // Merge API documentation with deduplication
        self.merge_string_vectors(
            &mut combined_info.api_documentation,
            source_info.api_documentation,
            processed_contents,
        );

        // Merge architecture info with deduplication
        self.merge_string_vectors(
            &mut combined_info.architecture_info,
            source_info.architecture_info,
            processed_contents,
        );

        // Merge contributing guidelines with deduplication
        self.merge_string_vectors(
            &mut combined_info.contributing_guidelines,
            source_info.contributing_guidelines,
            processed_contents,
        );

        // Merge technologies (avoid duplicates by name)
        for tech in source_info.technologies {
            if !combined_info.technologies.iter().any(|t| t.name == tech.name) {
                combined_info.technologies.push(tech);
            }
        }

        // Merge setup commands with deduplication
        for cmd in source_info.setup_commands {
            if !combined_info.setup_commands.contains(&cmd) {
                combined_info.setup_commands.push(cmd);
            }
        }

        // Merge validation conflicts (keep all unique conflicts)
        for conflict in source_info.validation_conflicts {
            let is_duplicate = combined_info.validation_conflicts.iter().any(|c| 
                c.documentation_claim == conflict.documentation_claim &&
                c.code_reality == conflict.code_reality
            );
            if !is_duplicate {
                combined_info.validation_conflicts.push(conflict);
            }
        }

        // Merge documentation coverage sections
        for section in source_info.documentation_coverage.sections_found {
            if !combined_info.documentation_coverage.sections_found.contains(&section) {
                combined_info.documentation_coverage.sections_found.push(section);
            }
        }

        // Update coverage score (use highest from all sources)
        if source_info.documentation_coverage.completeness_score > combined_info.documentation_coverage.completeness_score {
            combined_info.documentation_coverage.completeness_score = source_info.documentation_coverage.completeness_score;
        }

        // Merge quality indicators with deduplication
        for indicator in source_info.documentation_coverage.quality_indicators {
            if !combined_info.documentation_coverage.quality_indicators.contains(&indicator) {
                combined_info.documentation_coverage.quality_indicators.push(indicator);
            }
        }

        // Merge improvement suggestions with deduplication
        for suggestion in source_info.documentation_coverage.improvement_suggestions {
            if !combined_info.documentation_coverage.improvement_suggestions.contains(&suggestion) {
                combined_info.documentation_coverage.improvement_suggestions.push(suggestion);
            }
        }

        // Update overall confidence (accumulate for later averaging)
        combined_info.confidence_score += source_info.confidence_score;

        println!("   Merged documentation from source (confidence: {:.1}%)", source_info.confidence_score * 100.0);
        
        Ok(())
    }

    /// Merge string vectors with content-based deduplication
    fn merge_string_vectors(
        &self,
        target: &mut Vec<String>,
        source: Vec<String>,
        processed_contents: &mut std::collections::HashSet<String>,
    ) {
        for item in source {
            // Create a normalized hash key for deduplication
            let normalized = self.normalize_content_for_deduplication(&item);
            
            if !processed_contents.contains(&normalized) {
                processed_contents.insert(normalized);
                target.push(item);
            }
        }
    }

    /// Normalize content for deduplication by removing extra whitespace and lowercasing
    fn normalize_content_for_deduplication(&self, content: &str) -> String {
        // Remove extra whitespace, normalize line endings, and lowercase for comparison
        content
            .trim()
            .replace('\n', " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase()
    }

    /// Extract documentation information from a project directory
    pub fn extract_documentation(&self, project_path: &Path) -> Result<ExtractedDocumentationInfo> {
        println!("Starting documentation extraction for project: {}", project_path.display());

        let mut extracted = ExtractedDocumentationInfo {
            project_description: None,
            installation_instructions: Vec::new(),
            usage_examples: Vec::new(),
            api_documentation: Vec::new(),
            architecture_info: Vec::new(),
            contributing_guidelines: Vec::new(),
            technologies: Vec::new(),
            setup_commands: Vec::new(),
            validation_conflicts: Vec::new(),
            confidence_score: 0.0,
            documentation_coverage: DocumentationCoverage {
                sections_found: Vec::new(),
                sections_missing: Vec::new(),
                completeness_score: 0.0,
                quality_indicators: Vec::new(),
                improvement_suggestions: Vec::new(),
            },
        };

        // Find and process documentation files
        let doc_files = self.find_documentation_files(project_path)?;
        println!("Found {} documentation files to process", doc_files.len());

        for doc_file in &doc_files {
            if let Ok(content) = self.read_documentation_file(doc_file) {
                self.process_documentation_content(&content, &mut extracted)?;
            } else {
                println!("Warning: Failed to read documentation file: {}", doc_file.display());
            }
        }

        // Calculate coverage and confidence
        self.calculate_documentation_coverage(&mut extracted);
        self.calculate_confidence_score(&mut extracted);

        println!("Documentation extraction completed: {} technologies found, {:.1}% coverage",
                 extracted.technologies.len(),
                 extracted.documentation_coverage.completeness_score * 100.0);

        Ok(extracted)
    }

    /// Find all documentation files in the project
    fn find_documentation_files(&self, project_path: &Path) -> Result<Vec<PathBuf>> {
        let mut doc_files = Vec::new();

        // Check for standard documentation files in root
        for pattern in &self.config.scan_patterns {
            // Simple pattern matching - could be enhanced with glob patterns
            let file_path = if pattern.contains('/') {
                // Handle subdirectory patterns like "docs/**/*.md"
                // For now, just check docs directory
                if pattern.starts_with("docs/") {
                    let docs_dir = project_path.join("docs");
                    if docs_dir.exists() {
                        self.scan_docs_directory(&docs_dir)?
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            } else {
                // Handle root file patterns
                let file = project_path.join(pattern);
                if file.exists() && file.is_file() {
                    vec![file]
                } else {
                    Vec::new()
                }
            };

            doc_files.extend(file_path);
        }

        // Sort for consistent processing order
        doc_files.sort();
        Ok(doc_files)
    }

    /// Recursively scan docs directory for markdown files
    fn scan_docs_directory(&self, docs_dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        
        if let Ok(entries) = fs::read_dir(docs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        if extension == "md" || extension == "rst" {
                            // Check file size
                            if let Ok(metadata) = entry.metadata() {
                                let size_kb = metadata.len() / 1024;
                                if size_kb <= self.config.max_file_size_kb {
                                    files.push(path);
                                } else {
                                    // Debug: Skipping large documentation file
                                }
                            }
                        }
                    }
                } else if path.is_dir() {
                    // Recursively scan subdirectories
                    if let Ok(sub_files) = self.scan_docs_directory(&path) {
                        files.extend(sub_files);
                    }
                }
            }
        }

        Ok(files)
    }

    /// Read documentation file content
    fn read_documentation_file(&self, file_path: &Path) -> Result<String> {
        fs::read_to_string(file_path)
            .context(format!("Failed to read documentation file: {}", file_path.display()))
    }

    /// Process documentation content and extract information
    fn process_documentation_content(
        &self,
        content: &str,
        extracted: &mut ExtractedDocumentationInfo,
    ) -> Result<()> {
        // Extract sections
        self.extract_sections(content, extracted);
        
        // Extract technologies
        self.extract_technologies(content, extracted);
        
        // Extract setup commands
        self.extract_setup_commands(content, extracted);

        Ok(())
    }

    /// Extract different sections from documentation content
    fn extract_sections(&self, content: &str, extracted: &mut ExtractedDocumentationInfo) {
        let lines: Vec<&str> = content.lines().collect();
        let mut current_section: Option<DocumentationSection> = None;
        let mut section_content = String::new();

        for (i, line) in lines.iter().enumerate() {
            // Check if this line starts a new section
            let mut found_section = None;
            for (section, regex) in &self.section_patterns {
                if regex.is_match(line) {
                    found_section = Some(section.clone());
                    break;
                }
            }

            // If we found a new section, save the previous one
            if let Some(new_section) = found_section {
                if let Some(prev_section) = current_section.take() {
                    self.save_section_content(prev_section, &section_content, extracted);
                }
                current_section = Some(new_section);
                section_content.clear();
            } else if current_section.is_some() {
                // Collect content for current section
                section_content.push_str(line);
                section_content.push('\n');

                // Stop collecting if we hit another header (simple heuristic)
                if line.starts_with('#') && i > 0 {
                    // This might be another section header we don't specifically match
                    if let Some(section) = current_section.take() {
                        self.save_section_content(section, &section_content, extracted);
                        section_content.clear();
                    }
                }
            }
        }

        // Save final section
        if let Some(section) = current_section {
            self.save_section_content(section, &section_content, extracted);
        }
    }

    /// Save extracted section content to appropriate field
    fn save_section_content(
        &self,
        section: DocumentationSection,
        content: &str,
        extracted: &mut ExtractedDocumentationInfo,
    ) {
        let cleaned_content = content.trim();
        if cleaned_content.is_empty() {
            return;
        }

        // Track that we found this section
        if !extracted.documentation_coverage.sections_found.contains(&section) {
            extracted.documentation_coverage.sections_found.push(section.clone());
        }

        match section {
            DocumentationSection::Description => {
                if extracted.project_description.is_none() {
                    extracted.project_description = Some(cleaned_content.to_string());
                }
            }
            DocumentationSection::Installation => {
                extracted.installation_instructions.push(cleaned_content.to_string());
            }
            DocumentationSection::Usage => {
                extracted.usage_examples.push(cleaned_content.to_string());
            }
            DocumentationSection::Api => {
                extracted.api_documentation.push(cleaned_content.to_string());
            }
            DocumentationSection::Architecture => {
                extracted.architecture_info.push(cleaned_content.to_string());
            }
            DocumentationSection::Contributing => {
                extracted.contributing_guidelines.push(cleaned_content.to_string());
            }
            _ => {
                // Other sections could be handled as needed
            }
        }
    }

    /// Extract technology information from documentation
    fn extract_technologies(&self, content: &str, extracted: &mut ExtractedDocumentationInfo) {
        for (tech_name, regex) in &self.technology_patterns {
            if regex.is_match(content) {
                // Find the specific context where this technology is mentioned
                let context = self.extract_technology_context(content, regex);
                let confidence = self.calculate_technology_confidence(tech_name, &context);

                let tech_info = TechnologyInfo {
                    name: tech_name.clone(),
                    version: self.extract_version_from_context(&context),
                    purpose: self.infer_technology_purpose(tech_name, &context),
                    installation_method: self.extract_installation_method(&context),
                    configuration_notes: self.extract_configuration_notes(&context),
                    confidence,
                };

                // Avoid duplicates
                if !extracted.technologies.iter().any(|t| t.name == *tech_name) {
                    extracted.technologies.push(tech_info);
                }
            }
        }
    }

    /// Extract context around technology mentions
    fn extract_technology_context(&self, content: &str, regex: &Regex) -> String {
        let mut contexts = Vec::new();
        
        for mat in regex.find_iter(content) {
            let start = mat.start().saturating_sub(100);
            let end = (mat.end() + 100).min(content.len());
            contexts.push(&content[start..end]);
        }
        
        contexts.join(" | ")
    }

    /// Calculate confidence score for a technology detection
    fn calculate_technology_confidence(&self, _tech_name: &str, context: &str) -> f32 {
        let mut confidence: f32 = 0.5; // Base confidence
        
        // Increase confidence for installation instructions
        if context.to_lowercase().contains("install") {
            confidence += 0.2;
        }
        
        // Increase confidence for version specifications
        if context.contains("version") || context.contains("v") {
            confidence += 0.1;
        }
        
        // Increase confidence for dependency listings
        if context.contains("dependencies") || context.contains("requirements") {
            confidence += 0.15;
        }

        confidence.min(1.0)
    }

    /// Extract version information from context
    fn extract_version_from_context(&self, context: &str) -> Option<String> {
        // Simple version pattern matching
        let version_patterns = vec![
            r"v?(\d+\.\d+(?:\.\d+)?)",
            r"@(\d+\.\d+(?:\.\d+)?)",
            r"version\s*[:\-]\s*(\d+\.\d+(?:\.\d+)?)",
        ];

        for pattern in version_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(captures) = regex.captures(context) {
                    if let Some(version) = captures.get(1) {
                        return Some(version.as_str().to_string());
                    }
                }
            }
        }

        None
    }

    /// Infer the purpose of a technology from context
    fn infer_technology_purpose(&self, tech_name: &str, context: &str) -> String {
        match tech_name.to_lowercase().as_str() {
            "react" | "vue.js" | "angular" => "Frontend UI Framework".to_string(),
            "next.js" => "Full-stack React Framework".to_string(),
            "node.js" => "JavaScript Runtime".to_string(),
            "express" | "nestjs" => "Backend Web Framework".to_string(),
            "spring boot" => "Java Web Framework".to_string(),
            "django" | "flask" | "fastapi" => "Python Web Framework".to_string(),
            "docker" => "Containerization".to_string(),
            "kubernetes" => "Container Orchestration".to_string(),
            "postgresql" | "mysql" | "mongodb" => "Database".to_string(),
            "redis" => "Caching/Message Broker".to_string(),
            "graphql" => "API Query Language".to_string(),
            "typescript" => "Type-safe JavaScript".to_string(),
            _ => {
                // Try to infer from context
                if context.to_lowercase().contains("database") {
                    "Database Technology".to_string()
                } else if context.to_lowercase().contains("api") {
                    "API Technology".to_string()
                } else {
                    "Development Tool".to_string()
                }
            }
        }
    }

    /// Extract installation method from context
    fn extract_installation_method(&self, context: &str) -> Option<String> {
        let install_patterns = vec![
            (r"npm\s+install", "npm"),
            (r"yarn\s+add", "yarn"),
            (r"pip\s+install", "pip"),
            (r"mvn\s+install", "maven"),
            (r"gradle", "gradle"),
            (r"cargo", "cargo"),
            (r"docker", "docker"),
        ];

        for (pattern, method) in install_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(context) {
                    return Some(method.to_string());
                }
            }
        }

        None
    }

    /// Extract configuration notes from context
    fn extract_configuration_notes(&self, context: &str) -> Vec<String> {
        let mut notes = Vec::new();
        
        // Look for configuration-related sentences
        let config_keywords = vec!["config", "configure", "setup", "environment", "settings"];
        
        for sentence in context.split('.') {
            let sentence = sentence.trim();
            if config_keywords.iter().any(|keyword| sentence.to_lowercase().contains(keyword)) {
                if !sentence.is_empty() && sentence.len() > 10 {
                    notes.push(sentence.to_string());
                }
            }
        }

        notes
    }

    /// Extract setup commands from documentation
    fn extract_setup_commands(&self, content: &str, extracted: &mut ExtractedDocumentationInfo) {
        for line in content.lines() {
            let trimmed = line.trim();
            
            // Skip code block markers and comments
            if trimmed.starts_with("```") || trimmed.starts_with("#") || trimmed.is_empty() {
                continue;
            }

            for pattern in &self.setup_patterns {
                if pattern.is_match(trimmed) {
                    // Clean up the command (remove markdown backticks, etc.)
                    let cleaned = trimmed
                        .trim_start_matches('`')
                        .trim_end_matches('`')
                        .trim_start_matches("$ ")
                        .trim();
                    
                    if !extracted.setup_commands.contains(&cleaned.to_string()) {
                        extracted.setup_commands.push(cleaned.to_string());
                    }
                    break;
                }
            }
        }
    }

    /// Calculate documentation coverage metrics
    fn calculate_documentation_coverage(&self, extracted: &mut ExtractedDocumentationInfo) {
        let all_sections = vec![
            DocumentationSection::Description,
            DocumentationSection::Installation,
            DocumentationSection::Usage,
            DocumentationSection::Api,
            DocumentationSection::Architecture,
            DocumentationSection::Contributing,
            DocumentationSection::Examples,
            DocumentationSection::Configuration,
        ];

        extracted.documentation_coverage.sections_missing = all_sections
            .into_iter()
            .filter(|section| !extracted.documentation_coverage.sections_found.contains(section))
            .collect();

        let found_count = extracted.documentation_coverage.sections_found.len() as f32;
        let total_count = found_count + extracted.documentation_coverage.sections_missing.len() as f32;
        
        extracted.documentation_coverage.completeness_score = if total_count > 0.0 {
            found_count / total_count
        } else {
            0.0
        };

        // Generate quality indicators
        if extracted.project_description.is_some() {
            extracted.documentation_coverage.quality_indicators.push("Has project description".to_string());
        }
        if !extracted.installation_instructions.is_empty() {
            extracted.documentation_coverage.quality_indicators.push("Has installation instructions".to_string());
        }
        if !extracted.usage_examples.is_empty() {
            extracted.documentation_coverage.quality_indicators.push("Has usage examples".to_string());
        }
        if !extracted.setup_commands.is_empty() {
            extracted.documentation_coverage.quality_indicators.push("Has setup commands".to_string());
        }

        // Generate improvement suggestions
        if extracted.project_description.is_none() {
            extracted.documentation_coverage.improvement_suggestions.push("Add project description".to_string());
        }
        if extracted.installation_instructions.is_empty() {
            extracted.documentation_coverage.improvement_suggestions.push("Add installation instructions".to_string());
        }
        if extracted.usage_examples.is_empty() {
            extracted.documentation_coverage.improvement_suggestions.push("Add usage examples".to_string());
        }
    }

    /// Calculate overall confidence score for the extraction
    fn calculate_confidence_score(&self, extracted: &mut ExtractedDocumentationInfo) {
        let mut confidence_factors = Vec::new();

        // Base confidence from coverage
        confidence_factors.push(extracted.documentation_coverage.completeness_score);

        // Technology detection confidence
        if !extracted.technologies.is_empty() {
            let avg_tech_confidence = extracted.technologies
                .iter()
                .map(|t| t.confidence)
                .sum::<f32>() / extracted.technologies.len() as f32;
            confidence_factors.push(avg_tech_confidence);
        }

        // Penalize if very little content
        let has_substantial_content = 
            extracted.project_description.is_some() ||
            !extracted.installation_instructions.is_empty() ||
            !extracted.usage_examples.is_empty();

        if !has_substantial_content {
            confidence_factors.push(0.3); // Low confidence penalty
        }

        // Calculate weighted average
        extracted.confidence_score = if confidence_factors.is_empty() {
            0.0
        } else {
            confidence_factors.iter().sum::<f32>() / confidence_factors.len() as f32
        };
    }

    /// Validate documentation against code analysis results
    pub fn validate_against_code_analysis(
        &self,
        extracted: &mut ExtractedDocumentationInfo,
        _code_frameworks: &[String],
        _code_dependencies: &[String],
    ) -> Result<()> {
        // This is a placeholder for validation logic
        // In a real implementation, you would compare:
        // - Documented frameworks vs detected frameworks
        // - Documented dependencies vs actual dependencies
        // - Installation instructions vs actual package.json/requirements.txt
        // - API documentation vs actual API endpoints

        // Example validation conflict (placeholder)
        if extracted.technologies.iter().any(|t| t.name == "React") && 
           !extracted.installation_instructions.iter().any(|i| i.contains("npm")) {
            extracted.validation_conflicts.push(ValidationConflict {
                conflict_type: ConflictType::MissingDependency,
                documentation_claim: "Uses React".to_string(),
                code_reality: "No npm installation instructions found".to_string(),
                severity: ConflictSeverity::Medium,
                file_location: None,
                recommendation: "Add npm install instructions for React dependencies".to_string(),
            });
        }

        Ok(())
    }
}

impl Default for DocumentationExtractor {
    fn default() -> Self {
        Self::new().expect("Failed to create default DocumentationExtractor")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_documentation_extractor_creation() {
        let extractor = DocumentationExtractor::new();
        assert!(extractor.is_ok());
    }

    #[test]
    fn test_custom_config() {
        let config = DocumentationAnalysisConfig {
            enable_readme_extraction: false,
            enable_docs_scanning: true,
            validate_against_code: false,
            priority_code_over_docs: true,
            scan_patterns: vec!["README.md".to_string()],
            max_file_size_kb: 100,
            extract_sections: vec!["description".to_string()],
        };

        let extractor = DocumentationExtractor::with_config(config);
        assert!(extractor.is_ok());
    }

    #[test]
    fn test_section_patterns() {
        let extractor = DocumentationExtractor::new().unwrap();
        
        // Test that all expected sections have patterns
        assert!(extractor.section_patterns.contains_key(&DocumentationSection::Description));
        assert!(extractor.section_patterns.contains_key(&DocumentationSection::Installation));
        assert!(extractor.section_patterns.contains_key(&DocumentationSection::Usage));
    }

    #[test]
    fn test_technology_patterns() {
        let extractor = DocumentationExtractor::new().unwrap();
        
        // Test that common technologies have patterns
        assert!(extractor.technology_patterns.contains_key("React"));
        assert!(extractor.technology_patterns.contains_key("Node.js"));
        assert!(extractor.technology_patterns.contains_key("Docker"));
    }

    #[test]
    fn test_setup_patterns() {
        let extractor = DocumentationExtractor::new().unwrap();
        assert!(!extractor.setup_patterns.is_empty());
        
        // Test that setup patterns match common commands
        let test_commands = vec![
            "npm install",
            "pip install requirements",
            "docker build .",
            "mvn clean install",
        ];

        for command in test_commands {
            let matches = extractor.setup_patterns.iter().any(|p| p.is_match(command));
            assert!(matches, "No pattern matched command: {}", command);
        }
    }

    #[test]
    fn test_extract_sections() {
        let extractor = DocumentationExtractor::new().unwrap();
        let mut extracted = ExtractedDocumentationInfo {
            project_description: None,
            installation_instructions: Vec::new(),
            usage_examples: Vec::new(),
            api_documentation: Vec::new(),
            architecture_info: Vec::new(),
            contributing_guidelines: Vec::new(),
            technologies: Vec::new(),
            setup_commands: Vec::new(),
            validation_conflicts: Vec::new(),
            confidence_score: 0.0,
            documentation_coverage: DocumentationCoverage {
                sections_found: Vec::new(),
                sections_missing: Vec::new(),
                completeness_score: 0.0,
                quality_indicators: Vec::new(),
                improvement_suggestions: Vec::new(),
            },
        };

        let content = r#"
# My Project

## Description
This is a test project for documentation extraction.

## Installation
Run `npm install` to install dependencies.

## Usage
Use `npm start` to run the application.
"#;

        extractor.extract_sections(content, &mut extracted);

        assert!(extracted.project_description.is_some());
        assert!(!extracted.installation_instructions.is_empty());
        assert!(!extracted.usage_examples.is_empty());
        assert_eq!(extracted.documentation_coverage.sections_found.len(), 3);
    }

    #[test]
    fn test_extract_technologies() {
        let extractor = DocumentationExtractor::new().unwrap();
        let mut extracted = ExtractedDocumentationInfo {
            project_description: None,
            installation_instructions: Vec::new(),
            usage_examples: Vec::new(),
            api_documentation: Vec::new(),
            architecture_info: Vec::new(),
            contributing_guidelines: Vec::new(),
            technologies: Vec::new(),
            setup_commands: Vec::new(),
            validation_conflicts: Vec::new(),
            confidence_score: 0.0,
            documentation_coverage: DocumentationCoverage {
                sections_found: Vec::new(),
                sections_missing: Vec::new(),
                completeness_score: 0.0,
                quality_indicators: Vec::new(),
                improvement_suggestions: Vec::new(),
            },
        };

        let content = r#"
This project uses React v18.2.0 for the frontend.
We also use Node.js for the backend and Docker for deployment.
"#;

        extractor.extract_technologies(content, &mut extracted);

        assert!(!extracted.technologies.is_empty());
        
        let react_tech = extracted.technologies.iter()
            .find(|t| t.name == "React");
        assert!(react_tech.is_some());
        
        let node_tech = extracted.technologies.iter()
            .find(|t| t.name == "Node.js");
        assert!(node_tech.is_some());
        
        let docker_tech = extracted.technologies.iter()
            .find(|t| t.name == "Docker");
        assert!(docker_tech.is_some());
    }

    #[test]
    fn test_extract_setup_commands() {
        let extractor = DocumentationExtractor::new().unwrap();
        let mut extracted = ExtractedDocumentationInfo {
            project_description: None,
            installation_instructions: Vec::new(),
            usage_examples: Vec::new(),
            api_documentation: Vec::new(),
            architecture_info: Vec::new(),
            contributing_guidelines: Vec::new(),
            technologies: Vec::new(),
            setup_commands: Vec::new(),
            validation_conflicts: Vec::new(),
            confidence_score: 0.0,
            documentation_coverage: DocumentationCoverage {
                sections_found: Vec::new(),
                sections_missing: Vec::new(),
                completeness_score: 0.0,
                quality_indicators: Vec::new(),
                improvement_suggestions: Vec::new(),
            },
        };

        let content = r#"
To set up the project:

```bash
npm install
pip install -r requirements.txt
docker build -t myapp .
```
"#;

        extractor.extract_setup_commands(content, &mut extracted);

        assert!(!extracted.setup_commands.is_empty());
        assert!(extracted.setup_commands.iter().any(|c| c.contains("npm install")));
        assert!(extracted.setup_commands.iter().any(|c| c.contains("pip install")));
        assert!(extracted.setup_commands.iter().any(|c| c.contains("docker build")));
    }

    #[test]
    fn test_version_extraction() {
        let extractor = DocumentationExtractor::new().unwrap();
        
        let test_cases = vec![
            ("React v18.2.0", Some("18.2.0")),
            ("Node.js version: 16.14.2", Some("16.14.2")),
            ("@types/node@18.0.0", Some("18.0.0")),
            ("No version here", None),
        ];

        for (input, expected) in test_cases {
            let result = extractor.extract_version_from_context(input);
            match expected {
                Some(exp_version) => {
                    assert!(result.is_some(), "Expected to find version in: {}", input);
                    assert_eq!(result.unwrap(), exp_version);
                }
                None => {
                    assert!(result.is_none(), "Expected no version in: {}", input);
                }
            }
        }
    }

    #[test]
    fn test_calculate_coverage() {
        let extractor = DocumentationExtractor::new().unwrap();
        let mut extracted = ExtractedDocumentationInfo {
            project_description: Some("Test project".to_string()),
            installation_instructions: vec!["npm install".to_string()],
            usage_examples: vec!["npm start".to_string()],
            api_documentation: Vec::new(),
            architecture_info: Vec::new(),
            contributing_guidelines: Vec::new(),
            technologies: Vec::new(),
            setup_commands: Vec::new(),
            validation_conflicts: Vec::new(),
            confidence_score: 0.0,
            documentation_coverage: DocumentationCoverage {
                sections_found: vec![
                    DocumentationSection::Description,
                    DocumentationSection::Installation,
                    DocumentationSection::Usage,
                ],
                sections_missing: Vec::new(),
                completeness_score: 0.0,
                quality_indicators: Vec::new(),
                improvement_suggestions: Vec::new(),
            },
        };

        extractor.calculate_documentation_coverage(&mut extracted);

        assert!(extracted.documentation_coverage.completeness_score > 0.0);
        assert!(!extracted.documentation_coverage.quality_indicators.is_empty());
        assert!(extracted.documentation_coverage.quality_indicators
               .contains(&"Has project description".to_string()));
    }

    #[test]
    fn test_end_to_end_extraction() {
        // Create temporary directory with test documentation
        let temp_dir = TempDir::new().unwrap();
        let readme_path = temp_dir.path().join("README.md");
        
        let readme_content = r#"# Test Project

## Description
This is a comprehensive test project for documentation extraction.

## Installation
To install this project:
```bash
npm install
pip install -r requirements.txt
```

## Usage
Start the application:
```bash
npm start
```

## Architecture
This project uses React for the frontend and Node.js for the backend.
We use Docker for containerization.

## API
REST API endpoints are available at `/api/`.
"#;

        fs::write(&readme_path, readme_content).unwrap();

        let extractor = DocumentationExtractor::new().unwrap();
        let result = extractor.extract_documentation(temp_dir.path());

        assert!(result.is_ok());
        let extracted = result.unwrap();

        // Verify extraction results
        assert!(extracted.project_description.is_some());
        assert!(!extracted.installation_instructions.is_empty());
        assert!(!extracted.usage_examples.is_empty());
        assert!(!extracted.architecture_info.is_empty());
        assert!(!extracted.api_documentation.is_empty());
        assert!(!extracted.technologies.is_empty());
        assert!(!extracted.setup_commands.is_empty());
        assert!(extracted.confidence_score > 0.0);
        assert!(extracted.documentation_coverage.completeness_score > 0.0);

        // Verify specific technologies were detected
        let tech_names: Vec<&String> = extracted.technologies.iter().map(|t| &t.name).collect();
        assert!(tech_names.contains(&&"React".to_string()));
        assert!(tech_names.contains(&&"Node.js".to_string()));
        assert!(tech_names.contains(&&"Docker".to_string()));
    }
}