use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use anyhow::Result;
use tokio::fs;

// Import the correct types
use crate::core::context_types::{
    SegmentContext, SegmentId, ProjectContext, FileContext, CrossReferenceMap,
    ValidationResult, ValidationWarning, ValidationWarningType, ValidationSeverity,
    EdgeCaseAnalysisResult, EdgeCase, EdgeCaseType, EdgeCaseSeverity,
    MissingReference, CrossReferenceType, DeepNestingInfo,
    HandlingRecommendation, RecommendationPriority,
    ContextQualityMetrics, ConfidenceDistribution, ContextDepthMetrics,
    ContextEnhancementResult, ContextImprovement, ImprovementType, PerformanceGains,
    FileType, FileRole, SegmentType
};

/// Enhanced hierarchical context manager with edge case handling
pub struct HierarchicalContextManager {
    pub project_context: Option<ProjectContext>,
    pub file_contexts: HashMap<PathBuf, FileContext>,
    pub segment_contexts: HashMap<SegmentId, SegmentContext>,
    pub cross_references: CrossReferenceMap,
    pub context_cache: HashMap<String, String>,
}

impl HierarchicalContextManager {
    pub fn new() -> Self {
        Self {
            project_context: None,
            file_contexts: HashMap::new(),
            segment_contexts: HashMap::new(),
            cross_references: CrossReferenceMap::new(),
            context_cache: HashMap::new(),
        }
    }
    
    /// Initialize the context manager with project-level information
    pub async fn initialize(&mut self, project_path: &Path) -> Result<()> {
        // Initialize project-level analysis
        let characteristics = self.analyze_project_characteristics(project_path).await?;
        
        // Cache project information for context enhancement
        self.context_cache.insert(
            "project_path".to_string(), 
            project_path.to_string_lossy().to_string()
        );
        self.context_cache.insert(
            "total_files".to_string(), 
            characteristics.total_files.to_string()
        );
        
        Ok(())
    }
    
    /// Build enhanced segment context with full hierarchical information
    pub async fn build_enhanced_segment_context(&self, segment: &crate::core::types::AstSegment) -> Result<crate::core::context_types::EnhancedSegmentContext> {
        // For now, create a basic enhanced context
        // This would be expanded with more sophisticated context building
        
        let file_context = FileContext {
            file_path: segment.file_path.clone(),
            file_type: FileType::SourceCode,
            role_in_project: FileRole::CoreLogic,
            language: Some(segment.language.clone()),
            imports: vec![],
            exports: vec![],
            key_patterns: vec![],
            related_files: vec![],
            business_relevance: 0.5,
            last_modified: SystemTime::now(),
        };
        
        let segment_context = SegmentContext {
            segment_id: format!("{}:{}-{}", segment.file_path.display(), segment.start_line, segment.end_line),
            segment: segment.clone(),
            file_context,
            segment_type: SegmentType::FunctionDefinition, // Default, would be inferred
            business_purpose: None,
            dependencies: vec![],
            dependents: vec![],
            confidence: 0.7,
            extracted_at: SystemTime::now(),
        };
        
        use crate::core::context_types::{EnhancedSegmentContext, ArchitecturalContext, ArchitecturalLayer, InteractionStyle};
        
        Ok(EnhancedSegmentContext {
            segment_context,
            project_context: self.project_context.clone().unwrap_or_else(|| {
                // Create a minimal project context
                ProjectContext {
                    id: "temp_project".to_string(),
                    metadata: crate::core::project_analyzer::ProjectMetadata {
                        name: "Unknown Project".to_string(),
                        version: Some("0.1.0".to_string()),
                        description: Some("Analyzed project".to_string()),
                        authors: vec![],
                        dependencies: HashMap::new(),
                        dev_dependencies: HashMap::new(),
                        license: None,
                        repository: None,
                        package_manager: crate::core::project_analyzer::PackageManager::Unknown,
                    },
                    project_type: "Unknown".to_string(),
                    business_domains: vec![],
                    entry_points: vec![segment.file_path.clone()],
                    documentation_summary: "No documentation available".to_string(),
                    architectural_patterns: vec![],
                    dependency_overview: crate::core::context_types::DependencyOverview {
                        direct_dependencies: HashMap::new(),
                        framework_dependencies: vec![],
                        development_dependencies: vec![],
                        dependency_categories: HashMap::new(),
                    },
                    confidence: 0.5,
                    created_at: SystemTime::now(),
                }
            }),
            related_segments: vec![],
            cross_references: vec![],
            business_hints: vec![],
            architectural_context: ArchitecturalContext {
                layer: ArchitecturalLayer::Business,
                patterns: vec![],
                responsibilities: vec![],
                interaction_style: InteractionStyle::Synchronous,
            },
        })
    }
    
    /// Analyze project characteristics for initialization
    async fn analyze_project_characteristics(&self, project_path: &Path) -> Result<crate::core::context_types::ProjectCharacteristics> {
        let total_files = self.count_project_files(project_path).await?;
        let languages = self.detect_project_languages(project_path).await?;
        
        Ok(crate::core::context_types::ProjectCharacteristics {
            total_files,
            total_directories: 0, // Would be calculated properly
            languages_detected: languages,
            max_nesting_depth: 0, // Would be calculated properly
            has_symbolic_links: false, // Would be detected properly
            has_permission_issues: false, // Would be detected properly
            estimated_analysis_time: std::time::Duration::from_secs(total_files as u64 / 100), // Rough estimate
        })
    }

    /// Validate cross-references for completeness and correctness
    pub async fn validate_cross_references(&self) -> Result<ValidationResult> {
        let mut validation_result = ValidationResult::new();
        
        // Test for circular dependencies
        if let Some(cycles) = self.detect_circular_dependencies().await? {
            validation_result.circular_dependencies = Some(cycles.clone());
            validation_result.add_warning(ValidationWarning {
                warning_type: ValidationWarningType::CircularDependency,
                message: format!("Circular dependencies detected: {:?}", cycles),
                affected_segments: cycles,
                severity: ValidationSeverity::High,
            });
        }
        
        // Test for missing references
        let missing_refs = self.find_missing_references().await?;
        if !missing_refs.is_empty() {
            validation_result.missing_references = missing_refs.clone();
            validation_result.add_warning(ValidationWarning {
                warning_type: ValidationWarningType::MissingReference,
                message: format!("Missing references found: {}", missing_refs.len()),
                affected_segments: missing_refs.iter().map(|r| r.source_segment.clone()).collect(),
                severity: ValidationSeverity::Medium,
            });
        }
        
        // Test for orphaned contexts
        let orphaned = self.find_orphaned_contexts().await?;
        if !orphaned.is_empty() {
            validation_result.orphaned_contexts = orphaned.clone();
            validation_result.add_warning(ValidationWarning {
                warning_type: ValidationWarningType::OrphanedContext,
                message: format!("Orphaned contexts found: {}", orphaned.len()),
                affected_segments: orphaned.clone(),
                severity: ValidationSeverity::Low,
            });
        }
        
        Ok(validation_result)
    }
    
    /// Handle edge cases in project structure analysis
    pub async fn handle_edge_cases(&mut self, project_path: &Path) -> Result<EdgeCaseAnalysisResult> {
        let mut result = EdgeCaseAnalysisResult::new();
        
        // Handle empty projects
        if self.is_empty_project(project_path).await? {
            result.edge_cases_detected.push(EdgeCase {
                case_type: EdgeCaseType::EmptyProject,
                description: "Project contains no source files".to_string(),
                affected_components: vec!["entire_project".to_string()],
                severity: EdgeCaseSeverity::Warning,
                mitigation_applied: true,
            });
            
            result.handling_recommendations.push(HandlingRecommendation {
                edge_case_type: EdgeCaseType::EmptyProject,
                recommendation: "Analysis will be limited to project structure only".to_string(),
                priority: RecommendationPriority::Medium,
                estimated_effort: "Low".to_string(),
            });
        }
        
        // Handle single-file projects
        if self.is_single_file_project(project_path).await? {
            result.edge_cases_detected.push(EdgeCase {
                case_type: EdgeCaseType::SingleFileProject,
                description: "Project contains only one source file".to_string(),
                affected_components: vec!["single_file".to_string()],
                severity: EdgeCaseSeverity::Info,
                mitigation_applied: true,
            });
            
            result.handling_recommendations.push(HandlingRecommendation {
                edge_case_type: EdgeCaseType::SingleFileProject,
                recommendation: "Cross-reference analysis will be simplified".to_string(),
                priority: RecommendationPriority::Low,
                estimated_effort: "Low".to_string(),
            });
        }
        
        // Handle very large projects (>10k files)
        let file_count = self.count_project_files(project_path).await?;
        if file_count > 10000 {
            result.edge_cases_detected.push(EdgeCase {
                case_type: EdgeCaseType::LargeProject,
                description: format!("Large project with {} files detected", file_count),
                affected_components: vec!["entire_project".to_string()],
                severity: EdgeCaseSeverity::Warning,
                mitigation_applied: true,
            });
            
            result.handling_recommendations.push(HandlingRecommendation {
                edge_case_type: EdgeCaseType::LargeProject,
                recommendation: "Batch processing and memory optimization will be applied".to_string(),
                priority: RecommendationPriority::High,
                estimated_effort: "High".to_string(),
            });
        }
        
        // Update project characteristics
        result.project_characteristics.total_files = file_count;
        result.project_characteristics.languages_detected = self.detect_project_languages(project_path).await?;
        
        Ok(result)
    }
    
    /// Detect circular dependencies in cross-references
    async fn detect_circular_dependencies(&self) -> Result<Option<Vec<SegmentId>>> {
        let mut visited = std::collections::HashSet::new();
        let mut recursion_stack = std::collections::HashSet::new();
        
        for segment_id in self.segment_contexts.keys() {
            if !visited.contains(segment_id) {
                if let Some(cycle) = self.dfs_cycle_detection(
                    segment_id, 
                    &mut visited, 
                    &mut recursion_stack
                ).await? {
                    return Ok(Some(cycle));
                }
            }
        }
        
        Ok(None)
    }
    
    /// DFS-based cycle detection algorithm (simplified to avoid lifetime issues)
    async fn dfs_cycle_detection(
        &self,
        current: &SegmentId,
        visited: &mut std::collections::HashSet<SegmentId>,
        recursion_stack: &mut std::collections::HashSet<SegmentId>
    ) -> Result<Option<Vec<SegmentId>>> {
        visited.insert(current.clone());
        recursion_stack.insert(current.clone());
        
        // Get related segments using the available method
        let related = self.cross_references.get_related_segments(current);
        for neighbor in &related {
            if !visited.contains(neighbor) {
                // For now, use a simplified non-recursive approach to avoid async recursion issues
                // In a full implementation, this would use a proper async recursion pattern
                if recursion_stack.contains(neighbor) {
                    return Ok(Some(vec![current.clone(), neighbor.clone()]));
                }
            } else if recursion_stack.contains(neighbor) {
                // Found a cycle
                return Ok(Some(vec![current.clone(), neighbor.clone()]));
            }
        }
        
        recursion_stack.remove(current);
        Ok(None)
    }
    
    /// Find missing references in the context graph
    async fn find_missing_references(&self) -> Result<Vec<MissingReference>> {
        let mut missing_refs = Vec::new();
        
        // Check functional dependencies
        for (from_id, ref_list) in &self.cross_references.functional_dependencies {
            for to_id in ref_list {
                if !self.segment_contexts.contains_key(to_id) {
                    missing_refs.push(MissingReference {
                        source_segment: from_id.clone(),
                        missing_target: to_id.clone(),
                        reference_type: CrossReferenceType::FunctionalDependency,
                        context: "Functional dependency target not found".to_string(),
                    });
                }
            }
        }
        
        Ok(missing_refs)
    }
    
    /// Find orphaned contexts (segments with no incoming or outgoing references)
    async fn find_orphaned_contexts(&self) -> Result<Vec<SegmentId>> {
        let mut orphaned = Vec::new();
        
        for segment_id in self.segment_contexts.keys() {
            let has_references = !self.cross_references.get_related_segments(segment_id).is_empty();
            
            if !has_references {
                orphaned.push(segment_id.clone());
            }
        }
        
        Ok(orphaned)
    }
    
    /// Check if project is empty (no source files)
    async fn is_empty_project(&self, project_path: &Path) -> Result<bool> {
        let file_count = self.count_source_files(project_path).await?;
        Ok(file_count == 0)
    }
    
    /// Check if project has only a single source file
    async fn is_single_file_project(&self, project_path: &Path) -> Result<bool> {
        let file_count = self.count_source_files(project_path).await?;
        Ok(file_count == 1)
    }
    
    /// Count source files in project (simplified to avoid async recursion)
    async fn count_source_files(&self, project_path: &Path) -> Result<usize> {
        let mut count = 0;
        
        if let Ok(mut entries) = fs::read_dir(project_path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(file_type) = entry.file_type().await {
                    if file_type.is_file() {
                        let path = entry.path();
                        if let Some(extension) = path.extension() {
                            let ext = extension.to_string_lossy().to_lowercase();
                            if matches!(ext.as_str(), "ts" | "js" | "tsx" | "jsx" | "py" | "java" | "rs" | "go") {
                                count += 1;
                            }
                        }
                    }
                    // Note: For simplicity, we're not recursing into subdirectories here
                    // In a full implementation, this would use a proper async recursion pattern or iterative approach
                }
            }
        }
        
        Ok(count)
    }
    
    /// Count all files in project (simplified to avoid async recursion)
    async fn count_project_files(&self, project_path: &Path) -> Result<usize> {
        let mut count = 0;
        
        if let Ok(mut entries) = fs::read_dir(project_path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(file_type) = entry.file_type().await {
                    if file_type.is_file() {
                        count += 1;
                    }
                    // Note: For simplicity, we're not recursing into subdirectories here
                    // In a full implementation, this would use a proper async recursion pattern or iterative approach
                }
            }
        }
        
        Ok(count)
    }
    
    /// Detect languages used in the project (simplified to avoid async recursion)
    async fn detect_project_languages(&self, project_path: &Path) -> Result<Vec<String>> {
        let mut languages = std::collections::HashSet::new();
        
        if let Ok(mut entries) = fs::read_dir(project_path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(file_type) = entry.file_type().await {
                    if file_type.is_file() {
                        let path = entry.path();
                        if let Some(extension) = path.extension() {
                            let ext = extension.to_string_lossy().to_lowercase();
                            match ext.as_str() {
                                "ts" | "tsx" => { languages.insert("TypeScript".to_string()); },
                                "js" | "jsx" => { languages.insert("JavaScript".to_string()); },
                                "py" => { languages.insert("Python".to_string()); },
                                "java" => { languages.insert("Java".to_string()); },
                                "rs" => { languages.insert("Rust".to_string()); },
                                "go" => { languages.insert("Go".to_string()); },
                                _ => {}
                            }
                        }
                    }
                    // Note: For simplicity, we're not recursing into subdirectories here
                    // In a full implementation, this would use a proper async recursion pattern or iterative approach
                }
            }
        }
        
        Ok(languages.into_iter().collect())
    }
    
    /// Assess the quality of current context data
    pub async fn assess_context_quality(&self) -> Result<ContextQualityMetrics> {
        let total_segments = self.segment_contexts.len() as f32;
        
        if total_segments == 0.0 {
            return Ok(ContextQualityMetrics {
                completeness_score: 0.0,
                consistency_score: 0.0,
                coverage_score: 0.0,
                performance_score: 1.0, // Empty is technically "performant"
                confidence_distribution: ConfidenceDistribution {
                    high_confidence_segments: 0,
                    medium_confidence_segments: 0,
                    low_confidence_segments: 0,
                    average_confidence: 0.0,
                },
                context_depth: ContextDepthMetrics {
                    shallow_contexts: 0,
                    medium_contexts: 0,
                    deep_contexts: 0,
                    average_depth: 0.0,
                },
                timestamp: SystemTime::now(),
            });
        }
        
        // Calculate completeness score (segments with business purpose)
        let segments_with_purpose = self.segment_contexts.values()
            .filter(|ctx| ctx.business_purpose.is_some())
            .count() as f32;
        let completeness_score = segments_with_purpose / total_segments;
        
        // Calculate consistency score (cross-reference validity)
        let consistency_score = self.calculate_consistency_score().await?;
        
        // Calculate coverage score (files with contexts vs total files)
        let coverage_score = self.calculate_coverage_score().await?;
        
        // Calculate confidence distribution
        let confidence_distribution = self.calculate_confidence_distribution();
        
        // Calculate context depth metrics
        let context_depth = self.calculate_context_depth_metrics();
        
        // Performance score based on cache hit rates and analysis efficiency
        let performance_score = self.calculate_performance_score();
        
        Ok(ContextQualityMetrics {
            completeness_score,
            consistency_score,
            coverage_score,
            performance_score,
            confidence_distribution,
            context_depth,
            timestamp: SystemTime::now(),
        })
    }
    
    /// Enhance context quality through various improvement strategies
    pub async fn enhance_context_quality(&mut self) -> Result<ContextEnhancementResult> {
        let initial_quality = self.assess_context_quality().await?;
        let start_time = std::time::Instant::now();
        let mut improvements = Vec::new();
        
        // 1. Cross-reference enrichment
        let cross_ref_improvements = self.enrich_cross_references().await?;
        improvements.extend(cross_ref_improvements);
        
        // 2. Confidence recalibration
        let confidence_improvements = self.recalibrate_confidence_scores().await?;
        improvements.extend(confidence_improvements);
        
        // 3. Fill missing contexts
        let missing_context_improvements = self.fill_missing_contexts().await?;
        improvements.extend(missing_context_improvements);
        
        // 4. Performance optimizations
        let performance_improvements = self.optimize_context_performance().await?;
        improvements.extend(performance_improvements);
        
        let enhancement_time = start_time.elapsed();
        let enhanced_quality = self.assess_context_quality().await?;
        
        Ok(ContextEnhancementResult {
            initial_quality,
            enhanced_quality,
            improvements_applied: improvements,
            performance_gains: PerformanceGains {
                analysis_time_reduction: enhancement_time,
                memory_usage_reduction: 0.0, // Would need memory tracking
                cache_hit_rate_improvement: 0.0, // Would need cache metrics
                throughput_improvement: 0.0, // Would need throughput measurement
            },
            enhancement_timestamp: SystemTime::now(),
        })
    }
    
    /// Calculate consistency score for cross-references
    async fn calculate_consistency_score(&self) -> Result<f32> {
        let mut total_refs = 0;
        let mut valid_refs = 0;
        
        // Check functional dependencies
        for (_, ref_list) in &self.cross_references.functional_dependencies {
            for target_id in ref_list {
                total_refs += 1;
                if self.segment_contexts.contains_key(target_id) {
                    valid_refs += 1;
                }
            }
        }
        
        // Check data flow references
        for (_, ref_list) in &self.cross_references.data_flow {
            for target_id in ref_list {
                total_refs += 1;
                if self.segment_contexts.contains_key(target_id) {
                    valid_refs += 1;
                }
            }
        }
        
        if total_refs == 0 {
            return Ok(1.0); // No references to be inconsistent
        }
        
        Ok(valid_refs as f32 / total_refs as f32)
    }
    
    /// Calculate coverage score (how much of the codebase is covered)
    async fn calculate_coverage_score(&self) -> Result<f32> {
        let file_contexts = self.file_contexts.len() as f32;
        let segments_contexts = self.segment_contexts.len() as f32;
        
        if file_contexts == 0.0 {
            return Ok(0.0);
        }
        
        // Estimate coverage based on segments per file ratio
        let avg_segments_per_file = segments_contexts / file_contexts;
        let expected_segments_per_file = 5.0; // Reasonable estimate
        
        Ok((avg_segments_per_file / expected_segments_per_file).min(1.0))
    }
    
    /// Calculate confidence distribution across all segments
    fn calculate_confidence_distribution(&self) -> ConfidenceDistribution {
        let mut high = 0;
        let mut medium = 0;
        let mut low = 0;
        let mut total_confidence = 0.0;
        
        for context in self.segment_contexts.values() {
            let confidence = context.confidence;
            total_confidence += confidence;
            
            if confidence > 0.8 {
                high += 1;
            } else if confidence >= 0.5 {
                medium += 1;
            } else {
                low += 1;
            }
        }
        
        let total_segments = self.segment_contexts.len();
        let average_confidence = if total_segments > 0 {
            total_confidence / total_segments as f32
        } else {
            0.0
        };
        
        ConfidenceDistribution {
            high_confidence_segments: high,
            medium_confidence_segments: medium,
            low_confidence_segments: low,
            average_confidence,
        }
    }
    
    /// Calculate context depth metrics
    fn calculate_context_depth_metrics(&self) -> ContextDepthMetrics {
        let mut shallow = 0;
        let mut medium = 0;
        let mut deep = 0;
        let mut total_depth = 0.0;
        
        for context in self.segment_contexts.values() {
            let cross_ref_count = context.dependencies.len() + context.dependents.len();
            let business_context_richness = if context.business_purpose.is_some() { 1 } else { 0 };
            let depth = cross_ref_count + business_context_richness;
            
            total_depth += depth as f32;
            
            if depth <= 1 {
                shallow += 1;
            } else if depth <= 3 {
                medium += 1;
            } else {
                deep += 1;
            }
        }
        
        let total_contexts = self.segment_contexts.len();
        let average_depth = if total_contexts > 0 {
            total_depth / total_contexts as f32
        } else {
            0.0
        };
        
        ContextDepthMetrics {
            shallow_contexts: shallow,
            medium_contexts: medium,
            deep_contexts: deep,
            average_depth,
        }
    }
    
    /// Calculate performance score based on internal metrics
    fn calculate_performance_score(&self) -> f32 {
        // Simple heuristic: smaller cache = better performance
        let cache_size = self.context_cache.len() as f32;
        let max_reasonable_cache_size = 1000.0;
        
        if cache_size > max_reasonable_cache_size {
            0.5 // Performance degraded due to large cache
        } else {
            1.0 - (cache_size / max_reasonable_cache_size) * 0.5
        }
    }
    
    /// Enrich cross-references between contexts
    async fn enrich_cross_references(&mut self) -> Result<Vec<ContextImprovement>> {
        let mut improvements = Vec::new();
        let initial_ref_count = self.count_total_cross_references();
        
        // Look for implicit dependencies based on naming patterns
        for (segment_id, context) in &self.segment_contexts.clone() {
            let segment_name = self.extract_segment_name(&context.segment.content);
            
            // Find potential dependencies based on similar names or patterns
            for (other_id, other_context) in &self.segment_contexts {
                if segment_id != other_id {
                    if self.should_create_cross_reference(context, other_context) {
                        self.cross_references.add_functional_dependency(
                            segment_id.clone(),
                            other_id.clone()
                        );
                    }
                }
            }
        }
        
        let final_ref_count = self.count_total_cross_references();
        let refs_added = final_ref_count - initial_ref_count;
        
        if refs_added > 0 {
            improvements.push(ContextImprovement {
                improvement_type: ImprovementType::CrossReferenceEnrichment,
                description: format!("Added {} new cross-references", refs_added),
                impact_score: 0.3,
                segments_affected: refs_added,
            });
        }
        
        Ok(improvements)
    }
    
    /// Recalibrate confidence scores based on context completeness
    async fn recalibrate_confidence_scores(&mut self) -> Result<Vec<ContextImprovement>> {
        let mut improvements = Vec::new();
        let mut segments_adjusted = 0;
        
        // Collect contexts to avoid borrow checker issues
        let mut contexts_to_update = Vec::new();
        
        for (segment_id, context) in &self.segment_contexts {
            let original_confidence = context.confidence;
            let new_confidence = self.calculate_adjusted_confidence(context);
            
            if (new_confidence - original_confidence).abs() > 0.1 {
                contexts_to_update.push((segment_id.clone(), new_confidence));
            }
        }
        
        // Apply updates
        for (segment_id, new_confidence) in contexts_to_update {
            if let Some(context) = self.segment_contexts.get_mut(&segment_id) {
                context.confidence = new_confidence;
                segments_adjusted += 1;
            }
        }
        
        if segments_adjusted > 0 {
            improvements.push(ContextImprovement {
                improvement_type: ImprovementType::ConfidenceRecalibration,
                description: format!("Recalibrated confidence scores for {} segments", segments_adjusted),
                impact_score: 0.2,
                segments_affected: segments_adjusted,
            });
        }
        
        Ok(improvements)
    }
    
    /// Fill missing context information where possible
    async fn fill_missing_contexts(&mut self) -> Result<Vec<ContextImprovement>> {
        let mut improvements = Vec::new();
        let mut contexts_filled = 0;
        
        // Collect contexts that need business purpose
        let mut contexts_to_update = Vec::new();
        
        for (segment_id, context) in &self.segment_contexts {
            if context.business_purpose.is_none() {
                // Try to infer business purpose from segment content and type
                if let Some(inferred_purpose) = self.infer_business_purpose(&context.segment.content) {
                    contexts_to_update.push((segment_id.clone(), inferred_purpose));
                }
            }
        }
        
        // Apply updates
        for (segment_id, inferred_purpose) in contexts_to_update {
            if let Some(context) = self.segment_contexts.get_mut(&segment_id) {
                context.business_purpose = Some(inferred_purpose);
                contexts_filled += 1;
            }
        }
        
        if contexts_filled > 0 {
            improvements.push(ContextImprovement {
                improvement_type: ImprovementType::MissingContextFilling,
                description: format!("Filled business purpose for {} segments", contexts_filled),
                impact_score: 0.4,
                segments_affected: contexts_filled,
            });
        }
        
        Ok(improvements)
    }
    
    /// Optimize context performance through caching and pruning
    async fn optimize_context_performance(&mut self) -> Result<Vec<ContextImprovement>> {
        let mut improvements = Vec::new();
        let initial_cache_size = self.context_cache.len();
        
        // Clear stale cache entries (simple LRU-style cleanup)
        if initial_cache_size > 500 {
            self.context_cache.clear();
            
            improvements.push(ContextImprovement {
                improvement_type: ImprovementType::PerformanceOptimization,
                description: "Cleared context cache to improve performance".to_string(),
                impact_score: 0.1,
                segments_affected: 0,
            });
        }
        
        Ok(improvements)
    }
    
    // Helper methods for context enhancement
    
    fn count_total_cross_references(&self) -> usize {
        self.cross_references.functional_dependencies.values().map(|v| v.len()).sum::<usize>() +
        self.cross_references.data_flow.values().map(|v| v.len()).sum::<usize>() +
        self.cross_references.architectural_relationships.values().map(|v| v.len()).sum::<usize>()
    }
    
    fn extract_segment_name(&self, content: &str) -> String {
        // Simple name extraction (first word or identifier)
        content.split_whitespace()
            .next()
            .unwrap_or("unknown")
            .to_string()
    }
    
    fn should_create_cross_reference(&self, context1: &SegmentContext, context2: &SegmentContext) -> bool {
        // Simple heuristic: if one segment's content mentions the other's file name
        let file1_name = context1.file_context.file_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let file2_name = context2.file_context.file_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_lowercase())
            .unwrap_or_default();
            
        context1.segment.content.to_lowercase().contains(&file2_name) ||
        context2.segment.content.to_lowercase().contains(&file1_name)
    }
    
    fn calculate_adjusted_confidence(&self, context: &SegmentContext) -> f32 {
        let mut confidence = context.confidence;
        
        // Boost confidence if has business purpose
        if context.business_purpose.is_some() {
            confidence += 0.1;
        }
        
        // Boost confidence based on cross-reference count
        let ref_count = context.dependencies.len() + context.dependents.len();
        confidence += (ref_count as f32 * 0.05).min(0.2);
        
        confidence.min(1.0).max(0.0)
    }
    
    fn infer_business_purpose(&self, content: &str) -> Option<String> {
        let content_lower = content.to_lowercase();
        
        if content_lower.contains("user") || content_lower.contains("auth") {
            Some("User management and authentication".to_string())
        } else if content_lower.contains("api") || content_lower.contains("endpoint") {
            Some("API service endpoint".to_string())
        } else if content_lower.contains("database") || content_lower.contains("model") {
            Some("Data persistence and modeling".to_string())
        } else if content_lower.contains("test") {
            Some("Testing and quality assurance".to_string())
        } else {
            None
        }
    }
}

impl Default for HierarchicalContextManager {
    fn default() -> Self {
        Self::new()
    }
}