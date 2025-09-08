use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::core::documentation_extractor::{DocumentationExtractor, ExtractedDocumentationInfo};

/// Cross-repository project relationship detection and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossRepositoryAnalyzer {
    project_identifiers: Vec<String>,
    #[serde(skip)]
    semantic_patterns: HashMap<String, regex::Regex>,
    relevance_threshold: f32,
}

/// Result of cross-repository documentation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossRepositoryAnalysisResult {
    pub project_relationships: Vec<ProjectRelationship>,
    pub relevant_documentation: Vec<RelevantDocumentationEntry>,
    pub parent_project_context: Option<ParentProjectContext>,
    pub confidence_scores: HashMap<String, f32>,
    pub analysis_metadata: CrossRepositoryMetadata,
}

/// Relationship between the current project and external documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRelationship {
    pub relationship_type: RelationshipType,
    pub target_project: String,
    pub confidence: f32,
    pub evidence: Vec<String>,
    pub documentation_paths: Vec<PathBuf>,
}

/// Types of relationships between projects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    ParentProject,      // This is a subcomponent of a larger project
    SubProject,         // This project contains the referenced project
    SiblingProject,     // Projects at the same level in hierarchy
    DependencyProject,  // This project depends on the referenced project
    SharedContext,      // Projects share common documentation context
}

/// Documentation entry relevant to the current project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevantDocumentationEntry {
    pub source_path: PathBuf,
    pub content_type: DocumentationContentType,
    pub relevance_score: f32,
    pub matched_patterns: Vec<String>,
    pub extracted_content: String,
    pub project_references: Vec<String>,
}

/// Types of documentation content found in cross-repository analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentationContentType {
    Architecture,
    ApiDocumentation,
    SetupInstructions,
    DeploymentGuide,
    BusinessRequirements,
    TechnicalSpecifications,
    ProjectOverview,
    CrossServiceIntegration,
}

/// Parent project context derived from cross-repository analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentProjectContext {
    pub parent_project_name: String,
    pub parent_project_path: Option<PathBuf>,
    pub project_role: String,
    pub sibling_projects: Vec<String>,
    pub shared_infrastructure: Vec<String>,
    pub parent_documentation: ExtractedDocumentationInfo,
    pub context_confidence: f32,
}

/// Metadata about the cross-repository analysis process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossRepositoryMetadata {
    pub analyzed_paths: Vec<PathBuf>,
    pub total_documents_scanned: usize,
    pub relationships_found: usize,
    pub analysis_duration_ms: u128,
    pub semantic_matches: usize,
}

impl CrossRepositoryAnalyzer {
    /// Create new cross-repository analyzer
    pub fn new(project_path: &Path) -> Result<Self> {
        let project_identifiers = Self::extract_project_identifiers(project_path)?;
        let semantic_patterns = Self::compile_semantic_patterns(&project_identifiers)?;
        
        Ok(Self {
            project_identifiers,
            semantic_patterns,
            relevance_threshold: 0.3,
        })
    }

    /// Analyze cross-repository relationships and relevant documentation
    pub fn analyze_cross_repository_documentation(
        &self,
        project_path: &Path,
        external_docs_paths: &[PathBuf],
    ) -> Result<CrossRepositoryAnalysisResult> {
        println!("Starting cross-repository documentation analysis...");
        let start_time = std::time::Instant::now();

        let mut project_relationships = Vec::new();
        let mut relevant_documentation = Vec::new();
        let mut confidence_scores = HashMap::new();
        let mut total_documents_scanned = 0;
        let mut semantic_matches = 0;

        // Process each external documentation source
        for external_path in external_docs_paths {
            println!("Analyzing external documentation: {}", external_path.display());
            
            let (relationships, docs, matches) = self.analyze_external_source(
                project_path, 
                external_path
            )?;
            
            project_relationships.extend(relationships);
            relevant_documentation.extend(docs);
            semantic_matches += matches;
            
            // Count documents in this source
            if let Ok(doc_count) = self.count_documentation_files(external_path) {
                total_documents_scanned += doc_count;
            }
        }

        // Detect parent project context from relationships
        let parent_project_context = self.detect_parent_project_context(
            &project_relationships,
            &relevant_documentation,
            external_docs_paths,
        )?;

        // Calculate confidence scores
        for relationship in &project_relationships {
            confidence_scores.insert(
                relationship.target_project.clone(),
                relationship.confidence,
            );
        }

        let analysis_duration = start_time.elapsed();
        let relationships_count = project_relationships.len();
        let relevant_docs_count = relevant_documentation.len();
        
        println!("Cross-repository analysis complete:");
        println!("  Relationships found: {}", relationships_count);
        println!("  Relevant documents: {}", relevant_docs_count);
        println!("  Semantic matches: {}", semantic_matches);
        
        Ok(CrossRepositoryAnalysisResult {
            project_relationships,
            relevant_documentation,
            parent_project_context,
            confidence_scores,
            analysis_metadata: CrossRepositoryMetadata {
                analyzed_paths: external_docs_paths.to_vec(),
                total_documents_scanned,
                relationships_found: relationships_count,
                analysis_duration_ms: analysis_duration.as_millis(),
                semantic_matches,
            },
        })
    }

    /// Extract project identifiers from the current project
    fn extract_project_identifiers(project_path: &Path) -> Result<Vec<String>> {
        let mut identifiers = Vec::new();
        
        // Add directory name
        if let Some(dir_name) = project_path.file_name() {
            if let Some(name) = dir_name.to_str() {
                identifiers.push(name.to_string());
                
                // Add variations (replace hyphens/underscores)
                identifiers.push(name.replace('-', "_"));
                identifiers.push(name.replace('_', "-"));
                
                // Add camelCase and kebab-case variations
                identifiers.push(Self::to_camel_case(name));
                identifiers.push(Self::to_kebab_case(name));
            }
        }

        // Extract from package.json if it exists
        let package_json = project_path.join("package.json");
        if package_json.exists() {
            if let Ok(content) = std::fs::read_to_string(&package_json) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(name) = parsed.get("name").and_then(|n| n.as_str()) {
                        identifiers.push(name.to_string());
                    }
                }
            }
        }

        // Extract from Cargo.toml if it exists
        let cargo_toml = project_path.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                // Simple regex to extract name from Cargo.toml
                if let Ok(re) = regex::Regex::new(r#"name\s*=\s*"([^"]+)""#) {
                    if let Some(captures) = re.captures(&content) {
                        if let Some(name) = captures.get(1) {
                            identifiers.push(name.as_str().to_string());
                        }
                    }
                }
            }
        }

        // Remove duplicates
        identifiers.sort();
        identifiers.dedup();
        
        println!("Extracted project identifiers: {:?}", identifiers);
        Ok(identifiers)
    }

    /// Compile semantic patterns for project matching
    fn compile_semantic_patterns(identifiers: &[String]) -> Result<HashMap<String, regex::Regex>> {
        let mut patterns = HashMap::new();

        for identifier in identifiers {
            // Exact match pattern
            let exact_pattern = format!(r"(?i)\b{}\b", regex::escape(identifier));
            patterns.insert(
                format!("exact_{}", identifier),
                regex::Regex::new(&exact_pattern)?,
            );

            // Related project pattern (common prefixes/suffixes)
            let related_pattern = format!(r"(?i)\b{}-\w+|\b\w+-{}\b", regex::escape(identifier), regex::escape(identifier));
            patterns.insert(
                format!("related_{}", identifier),
                regex::Regex::new(&related_pattern)?,
            );

            // Component pattern (project mentioned as component)
            let component_pattern = format!(r"(?i)(?:component|service|module|project).*\b{}\b|\b{}\b.*(?:component|service|module)", regex::escape(identifier), regex::escape(identifier));
            patterns.insert(
                format!("component_{}", identifier),
                regex::Regex::new(&component_pattern)?,
            );
        }

        // Generic patterns for parent project detection
        patterns.insert(
            "parent_project".to_string(),
            regex::Regex::new(r"(?i)\b(?:parent|main|root|master|core)\s+project\b")?,
        );

        patterns.insert(
            "architecture_overview".to_string(),
            regex::Regex::new(r"(?i)\b(?:architecture|system\s+design|overview|structure)\b")?,
        );

        Ok(patterns)
    }

    /// Analyze a single external documentation source
    fn analyze_external_source(
        &self,
        project_path: &Path,
        external_path: &Path,
    ) -> Result<(Vec<ProjectRelationship>, Vec<RelevantDocumentationEntry>, usize)> {
        let mut relationships = Vec::new();
        let mut relevant_docs = Vec::new();
        let mut semantic_matches = 0;

        // Extract documentation from external source
        let doc_extractor = DocumentationExtractor::new()?;
        let external_docs = doc_extractor.extract_documentation(external_path)?;

        // Analyze project description for relationships
        if let Some(description) = &external_docs.project_description {
            let (desc_relationships, desc_matches) = self.analyze_content_for_relationships(
                description,
                external_path,
                DocumentationContentType::ProjectOverview,
            )?;
            relationships.extend(desc_relationships);
            semantic_matches += desc_matches;
        }

        // Analyze architecture information
        for arch_info in &external_docs.architecture_info {
            let (arch_relationships, arch_matches) = self.analyze_content_for_relationships(
                arch_info,
                external_path,
                DocumentationContentType::Architecture,
            )?;
            relationships.extend(arch_relationships);
            semantic_matches += arch_matches;

            // Check if this is relevant architecture documentation
            let relevance_score = self.calculate_relevance_score(arch_info);
            if relevance_score >= self.relevance_threshold {
                relevant_docs.push(RelevantDocumentationEntry {
                    source_path: external_path.to_path_buf(),
                    content_type: DocumentationContentType::Architecture,
                    relevance_score,
                    matched_patterns: self.get_matched_patterns(arch_info),
                    extracted_content: arch_info.clone(),
                    project_references: self.extract_project_references(arch_info),
                });
            }
        }

        // Analyze API documentation
        for api_doc in &external_docs.api_documentation {
            let relevance_score = self.calculate_relevance_score(api_doc);
            if relevance_score >= self.relevance_threshold {
                relevant_docs.push(RelevantDocumentationEntry {
                    source_path: external_path.to_path_buf(),
                    content_type: DocumentationContentType::ApiDocumentation,
                    relevance_score,
                    matched_patterns: self.get_matched_patterns(api_doc),
                    extracted_content: api_doc.clone(),
                    project_references: self.extract_project_references(api_doc),
                });
            }
        }

        Ok((relationships, relevant_docs, semantic_matches))
    }

    /// Analyze content for project relationships
    fn analyze_content_for_relationships(
        &self,
        content: &str,
        source_path: &Path,
        content_type: DocumentationContentType,
    ) -> Result<(Vec<ProjectRelationship>, usize)> {
        let mut relationships = Vec::new();
        let mut matches = 0;

        // Check for parent project indicators
        if let Some(parent_regex) = self.semantic_patterns.get("parent_project") {
            if parent_regex.is_match(content) {
                matches += 1;
                relationships.push(ProjectRelationship {
                    relationship_type: RelationshipType::ParentProject,
                    target_project: source_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    confidence: 0.8,
                    evidence: vec![format!("Found parent project indicators in {:?}", content_type)],
                    documentation_paths: vec![source_path.to_path_buf()],
                });
            }
        }

        // Check for project-specific matches
        for identifier in &self.project_identifiers {
            if let Some(exact_regex) = self.semantic_patterns.get(&format!("exact_{}", identifier)) {
                if exact_regex.is_match(content) {
                    matches += 1;
                    relationships.push(ProjectRelationship {
                        relationship_type: RelationshipType::SharedContext,
                        target_project: identifier.clone(),
                        confidence: 0.9,
                        evidence: vec![format!("Exact match found for '{}' in {:?}", identifier, content_type)],
                        documentation_paths: vec![source_path.to_path_buf()],
                    });
                }
            }

            if let Some(related_regex) = self.semantic_patterns.get(&format!("related_{}", identifier)) {
                if related_regex.is_match(content) {
                    matches += 1;
                    relationships.push(ProjectRelationship {
                        relationship_type: RelationshipType::SiblingProject,
                        target_project: identifier.clone(),
                        confidence: 0.7,
                        evidence: vec![format!("Related project pattern found for '{}' in {:?}", identifier, content_type)],
                        documentation_paths: vec![source_path.to_path_buf()],
                    });
                }
            }
        }

        Ok((relationships, matches))
    }

    /// Calculate relevance score for documentation content
    fn calculate_relevance_score(&self, content: &str) -> f32 {
        let mut score = 0.0;
        let mut matches = 0;

        // Check against all semantic patterns
        for (pattern_name, regex) in &self.semantic_patterns {
            if regex.is_match(content) {
                matches += 1;
                
                // Weight different pattern types
                let pattern_weight = if pattern_name.starts_with("exact_") {
                    0.4
                } else if pattern_name.starts_with("related_") {
                    0.3
                } else if pattern_name.starts_with("component_") {
                    0.2
                } else {
                    0.1
                };
                
                score += pattern_weight;
            }
        }

        // Normalize score
        if matches > 0 {
            score / matches as f32
        } else {
            0.0
        }
    }

    /// Get matched patterns for content
    fn get_matched_patterns(&self, content: &str) -> Vec<String> {
        let mut matched = Vec::new();
        
        for (pattern_name, regex) in &self.semantic_patterns {
            if regex.is_match(content) {
                matched.push(pattern_name.clone());
            }
        }
        
        matched
    }

    /// Extract project references from content
    fn extract_project_references(&self, content: &str) -> Vec<String> {
        let mut references = Vec::new();
        
        for identifier in &self.project_identifiers {
            if content.to_lowercase().contains(&identifier.to_lowercase()) {
                references.push(identifier.clone());
            }
        }
        
        references
    }

    /// Detect parent project context from relationships
    fn detect_parent_project_context(
        &self,
        relationships: &[ProjectRelationship],
        relevant_docs: &[RelevantDocumentationEntry],
        external_paths: &[PathBuf],
    ) -> Result<Option<ParentProjectContext>> {
        // Look for parent project relationships
        let parent_relationships: Vec<_> = relationships
            .iter()
            .filter(|r| matches!(r.relationship_type, RelationshipType::ParentProject))
            .collect();

        if parent_relationships.is_empty() {
            return Ok(None);
        }

        // Find the most confident parent project
        let best_parent = parent_relationships
            .iter()
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap();

        // Extract sibling projects from relationships
        let sibling_projects: Vec<String> = relationships
            .iter()
            .filter(|r| matches!(r.relationship_type, RelationshipType::SiblingProject))
            .map(|r| r.target_project.clone())
            .collect();

        // Find parent project documentation
        let parent_doc_path = best_parent.documentation_paths.first();
        let parent_documentation = if let Some(path) = parent_doc_path {
            let doc_extractor = DocumentationExtractor::new()?;
            doc_extractor.extract_documentation(path)?
        } else {
            // Create empty documentation info
            ExtractedDocumentationInfo {
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
                documentation_coverage: crate::core::documentation_extractor::DocumentationCoverage {
                    sections_found: Vec::new(),
                    sections_missing: Vec::new(),
                    completeness_score: 0.0,
                    quality_indicators: Vec::new(),
                    improvement_suggestions: Vec::new(),
                },
            }
        };

        Ok(Some(ParentProjectContext {
            parent_project_name: best_parent.target_project.clone(),
            parent_project_path: parent_doc_path.cloned(),
            project_role: self.infer_project_role(relevant_docs),
            sibling_projects,
            shared_infrastructure: self.extract_shared_infrastructure(relevant_docs),
            parent_documentation,
            context_confidence: best_parent.confidence,
        }))
    }

    /// Infer the role of the current project within the parent context
    fn infer_project_role(&self, relevant_docs: &[RelevantDocumentationEntry]) -> String {
        // Analyze documentation content to infer project role
        let mut role_indicators = HashMap::new();
        
        for doc in relevant_docs {
            let content = doc.extracted_content.to_lowercase();
            
            // Check for common role patterns
            if content.contains("frontend") || content.contains("ui") || content.contains("client") {
                *role_indicators.entry("Frontend".to_string()).or_insert(0) += 1;
            }
            if content.contains("backend") || content.contains("api") || content.contains("server") {
                *role_indicators.entry("Backend".to_string()).or_insert(0) += 1;
            }
            if content.contains("database") || content.contains("data") || content.contains("storage") {
                *role_indicators.entry("Data Layer".to_string()).or_insert(0) += 1;
            }
            if content.contains("auth") || content.contains("authentication") || content.contains("security") {
                *role_indicators.entry("Authentication Service".to_string()).or_insert(0) += 1;
            }
            if content.contains("gateway") || content.contains("proxy") || content.contains("load balancer") {
                *role_indicators.entry("Gateway Service".to_string()).or_insert(0) += 1;
            }
        }

        // Return the most common role or default
        role_indicators
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(role, _)| role)
            .unwrap_or_else(|| "Component Service".to_string())
    }

    /// Extract shared infrastructure references
    fn extract_shared_infrastructure(&self, relevant_docs: &[RelevantDocumentationEntry]) -> Vec<String> {
        let mut infrastructure = Vec::new();
        
        for doc in relevant_docs {
            let content = doc.extracted_content.to_lowercase();
            
            // Look for infrastructure keywords
            if content.contains("docker") || content.contains("container") {
                infrastructure.push("Docker".to_string());
            }
            if content.contains("kubernetes") || content.contains("k8s") {
                infrastructure.push("Kubernetes".to_string());
            }
            if content.contains("redis") {
                infrastructure.push("Redis".to_string());
            }
            if content.contains("postgresql") || content.contains("postgres") {
                infrastructure.push("PostgreSQL".to_string());
            }
            if content.contains("mongodb") || content.contains("mongo") {
                infrastructure.push("MongoDB".to_string());
            }
            if content.contains("nginx") {
                infrastructure.push("Nginx".to_string());
            }
        }

        // Remove duplicates
        infrastructure.sort();
        infrastructure.dedup();
        infrastructure
    }

    /// Count documentation files in a directory
    fn count_documentation_files(&self, path: &Path) -> Result<usize> {
        let mut count = 0;
        
        if path.is_dir() {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        if extension == "md" || extension == "rst" || extension == "txt" {
                            count += 1;
                        }
                    }
                } else if path.is_dir() {
                    count += self.count_documentation_files(&path)?;
                }
            }
        } else if path.is_file() {
            count = 1;
        }
        
        Ok(count)
    }

    /// Convert string to camelCase
    fn to_camel_case(s: &str) -> String {
        let parts: Vec<&str> = s.split(&['-', '_'][..]).collect();
        if parts.is_empty() {
            return s.to_string();
        }
        
        let mut result = parts[0].to_lowercase();
        for part in &parts[1..] {
            if !part.is_empty() {
                let mut chars = part.chars();
                if let Some(first) = chars.next() {
                    result.push(first.to_uppercase().next().unwrap_or(first));
                    result.push_str(&chars.as_str().to_lowercase());
                }
            }
        }
        result
    }

    /// Convert string to kebab-case
    fn to_kebab_case(s: &str) -> String {
        s.replace('_', "-").to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_project_identifier_extraction() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("test-project");
        fs::create_dir_all(&project_path).unwrap();

        // Create package.json
        let package_json = r#"{"name": "my-awesome-project"}"#;
        fs::write(project_path.join("package.json"), package_json).unwrap();

        let identifiers = CrossRepositoryAnalyzer::extract_project_identifiers(&project_path).unwrap();
        
        assert!(identifiers.contains(&"test-project".to_string()));
        assert!(identifiers.contains(&"my-awesome-project".to_string()));
        assert!(identifiers.contains(&"test_project".to_string()));
    }

    #[test]
    fn test_semantic_pattern_compilation() {
        let identifiers = vec!["my-project".to_string(), "test-service".to_string()];
        let patterns = CrossRepositoryAnalyzer::compile_semantic_patterns(&identifiers).unwrap();
        
        assert!(patterns.contains_key("exact_my-project"));
        assert!(patterns.contains_key("related_test-service"));
        assert!(patterns.contains_key("parent_project"));
    }

    #[test]
    fn test_relevance_score_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("test-project");
        fs::create_dir_all(&project_path).unwrap();

        let analyzer = CrossRepositoryAnalyzer::new(&project_path).unwrap();
        
        let high_relevance_content = "This is documentation for test-project architecture and design";
        let low_relevance_content = "This is unrelated documentation";
        
        let high_score = analyzer.calculate_relevance_score(high_relevance_content);
        let low_score = analyzer.calculate_relevance_score(low_relevance_content);
        
        assert!(high_score > low_score);
    }

    #[test]
    fn test_case_conversions() {
        assert_eq!(CrossRepositoryAnalyzer::to_camel_case("test-project"), "testProject");
        assert_eq!(CrossRepositoryAnalyzer::to_camel_case("my_service_name"), "myServiceName");
        assert_eq!(CrossRepositoryAnalyzer::to_kebab_case("test_project"), "test-project");
    }
}