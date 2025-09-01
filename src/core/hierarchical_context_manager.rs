use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use anyhow::{Result, Context};
use async_trait::async_trait;
use tokio::fs;

use crate::core::config::Config;
use crate::core::context_types::*;
use crate::core::context_traits::*;
use crate::core::types::{AstSegment, BusinessDomain};
use crate::core::project_analyzer::ProjectMetadata;
use crate::core::project_analyzer::ProjectAnalyzer;

pub struct HierarchicalContextManager {
    project_context: Option<ProjectContext>,
    file_contexts: HashMap<PathBuf, FileContext>,
    segment_contexts: HashMap<SegmentId, SegmentContext>,
    cross_references: CrossReferenceMap,
    
    project_analyzer: ProjectAnalyzer,
    file_context_provider: Box<dyn FileContextProvider>,
    segment_context_provider: Box<dyn SegmentContextProvider>,
    cross_reference_provider: Box<dyn CrossReferenceProvider>,
    context_enhancer: Box<dyn ContextEnhancer>,
    context_cache: Box<dyn ContextCache>,
    
    config: Config,
}

impl HierarchicalContextManager {
    pub fn new() -> Result<Self> {
        let config = Config::instance();
        
        Ok(Self {
            project_context: None,
            file_contexts: HashMap::new(),
            segment_contexts: HashMap::new(),
            cross_references: CrossReferenceMap::new(),
            
            project_analyzer: ProjectAnalyzer::new(),
            file_context_provider: Box::new(DefaultFileContextProvider::new()),
            segment_context_provider: Box::new(DefaultSegmentContextProvider::new()),
            cross_reference_provider: Box::new(DefaultCrossReferenceProvider::new()),
            context_enhancer: Box::new(DefaultContextEnhancer::new()),
            context_cache: Box::new(InMemoryContextCache::new()),
            
            config,
        })
    }

    pub async fn initialize(&mut self, project_path: &Path) -> Result<()> {
        let project_id = self.generate_project_id(project_path);
        
        if let Some(cached_context) = self.context_cache.get_project_context(&project_id) {
            self.project_context = Some(cached_context);
            return Ok(());
        }

        let project_context = self.build_project_context(project_path).await
            .context("Failed to build project context")?;
        
        self.context_cache.set_project_context(project_id.clone(), project_context.clone());
        self.project_context = Some(project_context);
        
        Ok(())
    }

    pub async fn build_enhanced_segment_context(&mut self, segment: &AstSegment) -> Result<EnhancedSegmentContext> {
        let project_context = self.get_project_context()?.clone();
        let file_context = self.get_or_build_file_context(&segment.file_path, &project_context).await?;
        let segment_context = self.get_or_build_segment_context(segment, &file_context).await?;
        
        self.ensure_cross_references_built().await?;
        
        let _related_segments = self.find_related_segments(&segment_context.segment_id);
        let _cross_references = self.get_cross_references_for_segment(&segment_context.segment_id);
        
        let enhanced_context = self.context_enhancer.enhance_segment_context(
            segment_context,
            &project_context,
            &self.cross_references
        ).await?;

        Ok(enhanced_context)
    }

    pub async fn build_file_contexts_batch(&mut self, file_paths: &[PathBuf]) -> Result<HashMap<PathBuf, FileContext>> {
        let project_context = self.get_project_context()?.clone();
        let path_refs: Vec<&Path> = file_paths.iter().map(|p| p.as_path()).collect();
        
        let contexts = self.file_context_provider
            .batch_extract_file_contexts(&path_refs, &project_context)
            .await?;

        let mut result = HashMap::new();
        for (path, context) in file_paths.iter().zip(contexts.into_iter()) {
            self.file_contexts.insert(path.clone(), context.clone());
            self.context_cache.set_file_context(path.clone(), context.clone());
            result.insert(path.clone(), context);
        }

        Ok(result)
    }

    pub async fn build_segment_contexts_batch(&mut self, segments: &[AstSegment]) -> Result<HashMap<SegmentId, SegmentContext>> {
        let mut result = HashMap::new();
        
        let mut segments_by_file: HashMap<PathBuf, Vec<&AstSegment>> = HashMap::new();
        for segment in segments {
            segments_by_file.entry(segment.file_path.clone())
                .or_insert_with(Vec::new)
                .push(segment);
        }

        for (file_path, file_segments) in segments_by_file {
            let project_context = self.get_project_context()?.clone();
            let file_context = self.get_or_build_file_context(&file_path, &project_context).await?;
            
            let segment_refs: Vec<AstSegment> = file_segments.into_iter().cloned().collect();
            let contexts = self.segment_context_provider
                .batch_extract_segment_contexts(&segment_refs, &file_context)
                .await?;

            for context in contexts {
                let segment_id = context.segment_id.clone();
                self.segment_contexts.insert(segment_id.clone(), context.clone());
                self.context_cache.set_segment_context(segment_id.clone(), context.clone());
                result.insert(segment_id, context);
            }
        }

        Ok(result)
    }

    async fn build_project_context(&self, project_path: &Path) -> Result<ProjectContext> {
        let project_context = self.project_analyzer.analyze_project_context(project_path).await?;
        
        let metadata = project_context.metadata.clone();
        let dev_dependencies: Vec<String> = metadata.dev_dependencies.keys().cloned().collect();
        
        Ok(ProjectContext {
            id: self.generate_project_id(project_path),
            metadata: project_context.metadata,
            project_type: project_context.project_type
                .as_ref()
                .map(|t| t.display_name().to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
            business_domains: project_context.business_domain_hints
                .into_iter()
                .map(|hint| BusinessDomain { name: hint, confidence: 0.7 })
                .collect(),
            entry_points: project_context.entry_points
                .into_iter()
                .map(|ep| ep.file_path)
                .collect(),
            documentation_summary: project_context.documentation_analysis.readme_content
                .unwrap_or_else(|| "No documentation found".to_string()),
            architectural_patterns: Vec::new(),
            dependency_overview: DependencyOverview {
                direct_dependencies: metadata.dependencies,
                framework_dependencies: Vec::new(),
                development_dependencies: dev_dependencies,
                dependency_categories: HashMap::new(),
            },
            confidence: project_context.project_type_confidence,
            created_at: SystemTime::now(),
        })
    }

    async fn get_or_build_file_context(&mut self, file_path: &Path, project_context: &ProjectContext) -> Result<FileContext> {
        if let Some(context) = self.file_contexts.get(file_path) {
            return Ok(context.clone());
        }

        if let Some(cached_context) = self.context_cache.get_file_context(&file_path.to_path_buf()) {
            self.file_contexts.insert(file_path.to_path_buf(), cached_context.clone());
            return Ok(cached_context);
        }

        let context = self.file_context_provider
            .extract_file_context(file_path, project_context)
            .await?;

        self.file_contexts.insert(file_path.to_path_buf(), context.clone());
        self.context_cache.set_file_context(file_path.to_path_buf(), context.clone());

        Ok(context)
    }

    async fn get_or_build_segment_context(&mut self, segment: &AstSegment, file_context: &FileContext) -> Result<SegmentContext> {
        let segment_id = self.generate_segment_id(segment);
        
        if let Some(context) = self.segment_contexts.get(&segment_id) {
            return Ok(context.clone());
        }

        if let Some(cached_context) = self.context_cache.get_segment_context(&segment_id) {
            self.segment_contexts.insert(segment_id.clone(), cached_context.clone());
            return Ok(cached_context);
        }

        let context = self.segment_context_provider
            .extract_segment_context(segment, file_context)
            .await?;

        self.segment_contexts.insert(segment_id.clone(), context.clone());
        self.context_cache.set_segment_context(segment_id, context.clone());

        Ok(context)
    }

    async fn ensure_cross_references_built(&mut self) -> Result<()> {
        if !self.cross_references.functional_dependencies.is_empty() {
            return Ok(());
        }

        let all_segments: Vec<SegmentContext> = self.segment_contexts.values().cloned().collect();
        if all_segments.is_empty() {
            return Ok(());
        }

        self.cross_references = self.cross_reference_provider
            .build_cross_references(&all_segments)
            .await?;

        Ok(())
    }

    fn find_related_segments(&self, segment_id: &SegmentId) -> Vec<SegmentContext> {
        let related_ids = self.cross_references.get_related_segments(segment_id);
        related_ids.into_iter()
            .filter_map(|id| self.segment_contexts.get(&id))
            .cloned()
            .collect()
    }

    fn get_cross_references_for_segment(&self, segment_id: &SegmentId) -> Vec<CrossReference> {
        let mut references = Vec::new();
        
        if let Some(func_deps) = self.cross_references.functional_dependencies.get(segment_id) {
            for target in func_deps {
                references.push(CrossReference {
                    source_segment: segment_id.clone(),
                    target_segment: target.clone(),
                    reference_type: CrossReferenceType::FunctionalDependency,
                    strength: 0.8,
                    description: "Functional dependency".to_string(),
                });
            }
        }

        if let Some(data_flows) = self.cross_references.data_flow.get(segment_id) {
            for target in data_flows {
                references.push(CrossReference {
                    source_segment: segment_id.clone(),
                    target_segment: target.clone(),
                    reference_type: CrossReferenceType::DataFlow,
                    strength: 0.7,
                    description: "Data flow relationship".to_string(),
                });
            }
        }

        references
    }

    fn get_project_context(&self) -> Result<&ProjectContext> {
        self.project_context.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Project context not initialized"))
    }

    fn generate_project_id(&self, project_path: &Path) -> ProjectId {
        format!("project_{}", project_path.file_name()
            .unwrap_or_default()
            .to_string_lossy())
    }

    fn generate_segment_id(&self, segment: &AstSegment) -> SegmentId {
        format!("{}:{}:{}:{}", 
            segment.file_path.display(),
            segment.start_line,
            segment.end_line,
            segment.segment_type)
    }
}

pub struct DefaultFileContextProvider {
    config: Config,
}

impl DefaultFileContextProvider {
    pub fn new() -> Self {
        Self {
            config: Config::instance(),
        }
    }
}

#[async_trait]
impl FileContextProvider for DefaultFileContextProvider {
    async fn extract_file_context(&self, file_path: &Path, project_context: &ProjectContext) -> Result<FileContext> {
        let file_type = self.classify_file_type(file_path);
        let file_role = self.determine_file_role(file_path, project_context);
        
        let language = self.detect_language(file_path);
        let (imports, exports) = self.extract_imports_exports(file_path).await?;
        let key_patterns = self.extract_key_patterns(file_path).await?;
        let related_files = self.find_related_files(file_path).await?;
        
        let business_relevance = self.calculate_business_relevance(&file_type, &file_role);
        
        let metadata = fs::metadata(file_path).await?;
        let last_modified = metadata.modified()?;

        Ok(FileContext {
            file_path: file_path.to_path_buf(),
            file_type,
            role_in_project: file_role,
            language,
            imports,
            exports,
            key_patterns,
            related_files,
            business_relevance,
            last_modified,
        })
    }

    async fn batch_extract_file_contexts(&self, file_paths: &[&Path], project_context: &ProjectContext) -> Result<Vec<FileContext>> {
        let mut contexts = Vec::new();
        
        for &file_path in file_paths {
            let context = self.extract_file_context(file_path, project_context).await?;
            contexts.push(context);
        }
        
        Ok(contexts)
    }

    fn classify_file_type(&self, file_path: &Path) -> FileType {
        let file_name = file_path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
            
        let extension = file_path.extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();

        let context_config = self.config.get_context_management_config();
        let patterns = &context_config["file_context_patterns"];

        if self.matches_patterns(&file_name, &patterns["configuration_files"]) {
            FileType::Configuration
        } else if self.matches_patterns(&file_name, &patterns["documentation_files"]) {
            FileType::Documentation
        } else if self.matches_patterns(&file_name, &patterns["test_files"]) {
            FileType::Test
        } else if matches!(extension.as_str(), "rs" | "ts" | "js" | "py" | "java") {
            FileType::SourceCode
        } else {
            FileType::Asset
        }
    }

    fn determine_file_role(&self, file_path: &Path, project_context: &ProjectContext) -> FileRole {
        let file_name = file_path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();

        if project_context.entry_points.contains(&file_path.to_path_buf()) {
            return FileRole::EntryPoint;
        }

        let context_config = self.config.get_context_management_config();
        let patterns = &context_config["file_context_patterns"];

        if self.matches_patterns(&file_name, &patterns["configuration_files"]) {
            FileRole::Configuration
        } else if self.matches_patterns(&file_name, &patterns["documentation_files"]) {
            FileRole::Documentation  
        } else if self.matches_patterns(&file_name, &patterns["test_files"]) {
            FileRole::Testing
        } else if self.matches_patterns(&file_name, &patterns["entry_point_files"]) {
            FileRole::EntryPoint
        } else {
            FileRole::CoreLogic
        }
    }
}

impl DefaultFileContextProvider {
    fn matches_patterns(&self, file_name: &str, patterns: &serde_json::Value) -> bool {
        if let Some(pattern_array) = patterns.as_array() {
            for pattern in pattern_array {
                if let Some(pattern_str) = pattern.as_str() {
                    if pattern_str.contains('*') {
                        let regex_pattern = pattern_str.replace('*', ".*");
                        if let Ok(regex) = regex::Regex::new(&regex_pattern) {
                            if regex.is_match(file_name) {
                                return true;
                            }
                        }
                    } else if file_name.contains(pattern_str) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn detect_language(&self, file_path: &Path) -> Option<String> {
        let extension = file_path.extension()?.to_str()?;
        match extension.to_lowercase().as_str() {
            "rs" => Some("Rust".to_string()),
            "ts" | "tsx" => Some("TypeScript".to_string()),
            "js" | "jsx" => Some("JavaScript".to_string()),
            "py" => Some("Python".to_string()),
            "java" => Some("Java".to_string()),
            _ => None,
        }
    }

    async fn extract_imports_exports(&self, _file_path: &Path) -> Result<(Vec<String>, Vec<String>)> {
        Ok((Vec::new(), Vec::new()))
    }

    async fn extract_key_patterns(&self, _file_path: &Path) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    async fn find_related_files(&self, _file_path: &Path) -> Result<Vec<PathBuf>> {
        Ok(Vec::new())
    }

    fn calculate_business_relevance(&self, file_type: &FileType, file_role: &FileRole) -> f32 {
        match (file_type, file_role) {
            (FileType::SourceCode, FileRole::CoreLogic) => 0.9,
            (FileType::SourceCode, FileRole::EntryPoint) => 0.8,
            (FileType::Configuration, _) => 0.6,
            (FileType::Documentation, _) => 0.4,
            (FileType::Test, _) => 0.3,
            _ => 0.2,
        }
    }
}

pub struct DefaultSegmentContextProvider;

impl DefaultSegmentContextProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl SegmentContextProvider for DefaultSegmentContextProvider {
    async fn extract_segment_context(&self, segment: &AstSegment, file_context: &FileContext) -> Result<SegmentContext> {
        let segment_id = format!("{}:{}:{}:{}", 
            segment.file_path.display(),
            segment.start_line,
            segment.end_line,
            segment.segment_type);

        let segment_type = self.classify_segment_type(segment);
        let business_purpose = self.infer_business_purpose(segment, file_context);
        let business_relevance = self.calculate_business_relevance(segment, file_context);

        Ok(SegmentContext {
            segment_id,
            segment: segment.clone(),
            file_context: file_context.clone(),
            segment_type,
            business_purpose,
            dependencies: Vec::new(),
            dependents: Vec::new(),
            confidence: business_relevance,
            extracted_at: SystemTime::now(),
        })
    }

    async fn batch_extract_segment_contexts(&self, segments: &[AstSegment], file_context: &FileContext) -> Result<Vec<SegmentContext>> {
        let mut contexts = Vec::new();
        
        for segment in segments {
            let context = self.extract_segment_context(segment, file_context).await?;
            contexts.push(context);
        }
        
        Ok(contexts)
    }

    fn classify_segment_type(&self, segment: &AstSegment) -> SegmentType {
        match segment.segment_type.as_str() {
            "function" => SegmentType::FunctionDefinition,
            "class" => SegmentType::ClassDefinition,
            "interface" => SegmentType::InterfaceDefinition,
            "import" => SegmentType::ModuleImport,
            "config" => SegmentType::ConfigurationBlock,
            "endpoint" => SegmentType::ApiEndpoint,
            _ => SegmentType::BusinessLogic,
        }
    }

    fn calculate_business_relevance(&self, segment: &AstSegment, file_context: &FileContext) -> f32 {
        let base_relevance = match self.classify_segment_type(segment) {
            SegmentType::ApiEndpoint => 0.9,
            SegmentType::BusinessLogic => 0.8,
            SegmentType::ClassDefinition => 0.7,
            SegmentType::FunctionDefinition => 0.6,
            SegmentType::InterfaceDefinition => 0.5,
            _ => 0.3,
        };

        base_relevance * file_context.business_relevance
    }
}

impl DefaultSegmentContextProvider {
    fn infer_business_purpose(&self, segment: &AstSegment, _file_context: &FileContext) -> Option<String> {
        if segment.content.contains("api") || segment.content.contains("endpoint") {
            Some("API endpoint handling".to_string())
        } else if segment.content.contains("business") || segment.content.contains("logic") {
            Some("Business logic implementation".to_string())
        } else if segment.content.contains("data") || segment.content.contains("model") {
            Some("Data handling and modeling".to_string())
        } else {
            None
        }
    }
}

pub struct DefaultCrossReferenceProvider;

impl DefaultCrossReferenceProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CrossReferenceProvider for DefaultCrossReferenceProvider {
    async fn build_cross_references(&self, segment_contexts: &[SegmentContext]) -> Result<CrossReferenceMap> {
        let mut cross_refs = CrossReferenceMap::new();
        
        for segment in segment_contexts {
            let func_deps = self.find_functional_dependencies(segment, segment_contexts).await?;
            for dep in func_deps {
                cross_refs.add_functional_dependency(segment.segment_id.clone(), dep);
            }
            
            let data_flows = self.trace_data_flow(segment, segment_contexts).await?;
            for flow in data_flows {
                cross_refs.add_data_flow(segment.segment_id.clone(), flow);
            }
        }
        
        Ok(cross_refs)
    }

    async fn find_functional_dependencies(&self, segment: &SegmentContext, all_segments: &[SegmentContext]) -> Result<Vec<SegmentId>> {
        let mut dependencies = Vec::new();
        
        for other_segment in all_segments {
            if segment.segment_id != other_segment.segment_id {
                if self.has_functional_dependency(segment, other_segment) {
                    dependencies.push(other_segment.segment_id.clone());
                }
            }
        }
        
        Ok(dependencies)
    }

    async fn trace_data_flow(&self, segment: &SegmentContext, all_segments: &[SegmentContext]) -> Result<Vec<SegmentId>> {
        let mut flows = Vec::new();
        
        for other_segment in all_segments {
            if segment.segment_id != other_segment.segment_id {
                if self.has_data_flow(segment, other_segment) {
                    flows.push(other_segment.segment_id.clone());
                }
            }
        }
        
        Ok(flows)
    }

    async fn identify_architectural_relationships(&self, segment: &SegmentContext, all_segments: &[SegmentContext]) -> Result<Vec<SegmentId>> {
        let mut relationships = Vec::new();
        
        for other_segment in all_segments {
            if segment.segment_id != other_segment.segment_id {
                if self.has_architectural_relationship(segment, other_segment) {
                    relationships.push(other_segment.segment_id.clone());
                }
            }
        }
        
        Ok(relationships)
    }
}

impl DefaultCrossReferenceProvider {
    fn has_functional_dependency(&self, source: &SegmentContext, target: &SegmentContext) -> bool {
        source.segment.content.contains(&self.extract_function_name(&target.segment))
    }

    fn has_data_flow(&self, source: &SegmentContext, target: &SegmentContext) -> bool {
        let source_variables = self.extract_variables(&source.segment);
        let target_variables = self.extract_variables(&target.segment);
        
        source_variables.iter().any(|var| target_variables.contains(var))
    }

    fn has_architectural_relationship(&self, source: &SegmentContext, target: &SegmentContext) -> bool {
        source.file_context.file_path.parent() == target.file_context.file_path.parent()
    }

    fn extract_function_name(&self, segment: &AstSegment) -> String {
        if let Some(start) = segment.content.find("fn ") {
            if let Some(end) = segment.content[start + 3..].find('(') {
                return segment.content[start + 3..start + 3 + end].trim().to_string();
            }
        }
        "unknown".to_string()
    }

    fn extract_variables(&self, _segment: &AstSegment) -> Vec<String> {
        Vec::new()
    }
}

pub struct DefaultContextEnhancer;

impl DefaultContextEnhancer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ContextEnhancer for DefaultContextEnhancer {
    async fn enhance_segment_context(&self, segment_context: SegmentContext, project_context: &ProjectContext, cross_references: &CrossReferenceMap) -> Result<EnhancedSegmentContext> {
        let related_segments = cross_references.get_related_segments(&segment_context.segment_id)
            .into_iter()
            .map(|_| segment_context.clone()) // Simplified for now
            .collect();

        let cross_refs = vec![];
        let business_hints = self.extract_business_hints(&segment_context, project_context).await?;
        let architectural_context = self.determine_architectural_context(&segment_context, project_context).await?;

        Ok(EnhancedSegmentContext {
            segment_context,
            project_context: project_context.clone(),
            related_segments,
            cross_references: cross_refs,
            business_hints,
            architectural_context,
        })
    }

    async fn extract_business_hints(&self, segment_context: &SegmentContext, project_context: &ProjectContext) -> Result<Vec<String>> {
        let mut hints = Vec::new();
        
        for domain in &project_context.business_domains {
            if segment_context.segment.content.to_lowercase().contains(&domain.name.to_lowercase()) {
                hints.push(format!("Related to {} domain", domain.name));
            }
        }
        
        if let Some(purpose) = &segment_context.business_purpose {
            hints.push(purpose.clone());
        }
        
        Ok(hints)
    }

    async fn determine_architectural_context(&self, segment_context: &SegmentContext, _project_context: &ProjectContext) -> Result<ArchitecturalContext> {
        let layer = match segment_context.segment_type {
            SegmentType::ApiEndpoint => ArchitecturalLayer::Presentation,
            SegmentType::BusinessLogic => ArchitecturalLayer::Business,
            SegmentType::DataStructure => ArchitecturalLayer::Data,
            _ => ArchitecturalLayer::Cross,
        };

        Ok(ArchitecturalContext {
            layer,
            patterns: Vec::new(),
            responsibilities: Vec::new(),
            interaction_style: InteractionStyle::Synchronous,
        })
    }
}

pub struct InMemoryContextCache {
    project_contexts: HashMap<ProjectId, ProjectContext>,
    file_contexts: HashMap<FileId, FileContext>,
    segment_contexts: HashMap<SegmentId, SegmentContext>,
}

impl InMemoryContextCache {
    pub fn new() -> Self {
        Self {
            project_contexts: HashMap::new(),
            file_contexts: HashMap::new(),
            segment_contexts: HashMap::new(),
        }
    }
}

impl ContextCache for InMemoryContextCache {
    fn get_project_context(&self, project_id: &ProjectId) -> Option<ProjectContext> {
        self.project_contexts.get(project_id).cloned()
    }

    fn set_project_context(&mut self, project_id: ProjectId, context: ProjectContext) {
        self.project_contexts.insert(project_id, context);
    }

    fn get_file_context(&self, file_path: &FileId) -> Option<FileContext> {
        self.file_contexts.get(file_path).cloned()
    }

    fn set_file_context(&mut self, file_path: FileId, context: FileContext) {
        self.file_contexts.insert(file_path, context);
    }

    fn get_segment_context(&self, segment_id: &SegmentId) -> Option<SegmentContext> {
        self.segment_contexts.get(segment_id).cloned()
    }

    fn set_segment_context(&mut self, segment_id: SegmentId, context: SegmentContext) {
        self.segment_contexts.insert(segment_id, context);
    }

    fn invalidate_project(&mut self, project_id: &ProjectId) {
        self.project_contexts.remove(project_id);
        self.file_contexts.clear();
        self.segment_contexts.clear();
    }

    fn clear_expired(&mut self) {
        // Implementation would check timestamps and remove expired entries
        // Simplified for now
    }
}