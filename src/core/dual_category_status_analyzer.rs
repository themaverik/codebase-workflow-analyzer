use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::core::documentation_claims_extractor::{DocumentationClaimsExtractor, DocumentationClaimsResult, DocumentationClaim};
use crate::core::code_reality_analyzer::{CodeRealityAnalyzer, CodeRealityResult, ImplementationEvidence, ImplementationLevel};
use crate::core::conflict_resolution_engine::{ConflictResolutionEngine, ConflictResolutionResult, Conflict, ResolutionStrategy};
use crate::core::todo_scanner::{TodoScanner, TodoScanResult};
use crate::core::status_inference_engine::{StatusInferenceEngine, StatusAnalysisResult};

/// Dual-category status analysis combining documentation claims with code reality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualCategoryStatusResult {
    pub explicit_status: ExplicitStatusAnalysis,
    pub inferred_status: InferredStatusAnalysis,
    pub merged_status: MergedStatusAnalysis,
    pub consistency_analysis: ConsistencyAnalysis,
    pub metadata: DualCategoryAnalysisMetadata,
}

/// Explicit status from direct code analysis (TODOs, placeholders)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplicitStatusAnalysis {
    pub todo_analysis: TodoScanResult,
    pub placeholder_implementations: Vec<PlaceholderImplementation>,
    pub explicit_todos_by_category: HashMap<String, Vec<ExplicitTodo>>,
    pub completion_score: f32,
}

/// Inferred status from documentation vs code comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferredStatusAnalysis {
    pub documentation_claims: DocumentationClaimsResult,
    pub code_reality: CodeRealityResult,
    pub conflicts: ConflictResolutionResult,
    pub incomplete_features: Vec<IncompleteFeature>,
    pub missing_implementations: Vec<MissingImplementation>,
    pub partial_implementations: Vec<PartialImplementation>,
    pub inferred_completion_score: f32,
}

/// Merged status combining explicit and inferred analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergedStatusAnalysis {
    pub comprehensive_feature_status: Vec<FeatureStatus>,
    pub implementation_priorities: Vec<ImplementationPriority>,
    pub overall_completion_score: f32,
    pub confidence_score: f32,
    pub recommended_next_steps: Vec<RecommendedAction>,
}

/// Consistency analysis between documentation and code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyAnalysis {
    pub documentation_accuracy: f32,
    pub implementation_coverage: f32,
    pub consistency_score: f32,
    pub major_discrepancies: Vec<MajorDiscrepancy>,
    pub recommendations: Vec<ConsistencyRecommendation>,
}

/// Metadata for dual-category analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualCategoryAnalysisMetadata {
    pub total_analysis_time_ms: u64,
    pub files_analyzed: usize,
    pub claims_processed: usize,
    pub implementations_found: usize,
    pub conflicts_resolved: usize,
    pub confidence_level: f32,
}

/// A placeholder implementation found in code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceholderImplementation {
    pub feature_name: String,
    pub file_path: String,
    pub line_number: usize,
    pub placeholder_type: String, // "TODO", "NotImplementedError", etc.
    pub description: String,
    pub urgency_score: f32,
}

/// Explicit TODO item with categorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplicitTodo {
    pub description: String,
    pub file_path: String,
    pub line_number: usize,
    pub category: String,
    pub priority_score: f32,
    pub estimated_effort: EffortLevel,
}

/// Incomplete feature identified through analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncompleteFeature {
    pub feature_name: String,
    pub claimed_capabilities: Vec<String>,
    pub actual_implementation: Option<String>,
    pub completion_percentage: f32,
    pub missing_components: Vec<String>,
    pub evidence: Vec<String>,
}

/// Missing implementation identified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingImplementation {
    pub feature_name: String,
    pub documentation_source: String,
    pub expected_implementation: String,
    pub suggested_location: Option<String>,
    pub priority_score: f32,
    pub dependencies: Vec<String>,
}

/// Partially implemented feature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialImplementation {
    pub feature_name: String,
    pub implemented_parts: Vec<String>,
    pub missing_parts: Vec<String>,
    pub implementation_quality: ImplementationQuality,
    pub completion_estimate: f32,
}

/// Comprehensive status of a feature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureStatus {
    pub feature_name: String,
    pub documentation_claim: Option<DocumentationClaim>,
    pub implementation_evidence: Option<ImplementationEvidence>,
    pub status_category: StatusCategory,
    pub completion_level: CompletionLevel,
    pub confidence: f32,
    pub next_actions: Vec<String>,
}

/// Implementation priority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationPriority {
    pub feature_name: String,
    pub priority_score: f32,
    pub business_impact: BusinessImpact,
    pub technical_complexity: TechnicalComplexity,
    pub dependencies: Vec<String>,
    pub estimated_effort: EffortLevel,
    pub recommended_timeline: String,
}

/// Recommended action for implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedAction {
    pub action_type: ActionType,
    pub description: String,
    pub target_feature: String,
    pub priority: ActionPriority,
    pub estimated_effort: EffortLevel,
    pub prerequisites: Vec<String>,
}

/// Major discrepancy between documentation and code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MajorDiscrepancy {
    pub discrepancy_type: String,
    pub description: String,
    pub severity: DiscrepancySeverity,
    pub documentation_claim: Option<String>,
    pub actual_implementation: Option<String>,
    pub impact_assessment: String,
    pub resolution_recommendation: String,
}

/// Consistency recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyRecommendation {
    pub recommendation_type: RecommendationType,
    pub description: String,
    pub priority: RecommendationPriority,
    pub affected_components: Vec<String>,
    pub estimated_impact: String,
}

/// Enums for categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatusCategory {
    CompletelyImplemented,
    PartiallyImplemented,
    PlannedNotImplemented,
    UnplannedImplemented,
    Conflicted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompletionLevel {
    Complete,      // 90-100%
    NearComplete,  // 70-89%
    Partial,       // 30-69%
    Minimal,       // 10-29%
    NotStarted,    // 0-9%
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImplementationQuality {
    Production,    // Complete with error handling, tests
    Development,   // Basic implementation, some gaps
    Prototype,     // Proof of concept, many shortcuts
    Skeleton,      // Structure only, no logic
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BusinessImpact {
    Critical,      // Core business functionality
    High,          // Important user features
    Medium,        // Nice-to-have features
    Low,           // Internal tools, optimizations
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TechnicalComplexity {
    High,          // Complex algorithms, integrations
    Medium,        // Standard implementation
    Low,           // Simple feature addition
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffortLevel {
    Large,         // Weeks to months
    Medium,        // Days to weeks  
    Small,         // Hours to days
    Trivial,       // Minutes to hours
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionType {
    Implement,     // Build missing feature
    Complete,      // Finish partial implementation
    Fix,           // Resolve conflicts/bugs
    Document,      // Add missing documentation
    Remove,        // Remove unused/dead code
    Refactor,      // Improve existing code
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionPriority {
    Urgent,        // Critical path blockers
    High,          // Important for next release
    Medium,        // Should be done eventually
    Low,           // Nice to have
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiscrepancySeverity {
    Critical,      // Major functionality claims false
    High,          // Important features misrepresented
    Medium,        // Minor inconsistencies
    Low,           // Documentation lag
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RecommendationType {
    UpdateDocumentation,
    ImplementFeature,
    RemoveClaimedFeature,
    ImproveTesting,
    AddValidation,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Configuration for dual-category status analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualCategoryStatusConfig {
    pub enable_dual_analysis: bool,
    pub completion_score_weights: CompletionScoreWeights,
    pub priority_calculation_weights: PriorityWeights,
    pub confidence_thresholds: ConfidenceThresholds,
    pub action_generation_config: ActionGenerationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionScoreWeights {
    pub explicit_weight: f32,      // Weight for explicit TODO analysis
    pub inferred_weight: f32,      // Weight for inferred status analysis
    pub implementation_weight: f32, // Weight for actual implementation evidence
    pub documentation_weight: f32,  // Weight for documentation claims
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityWeights {
    pub business_impact_weight: f32,
    pub technical_complexity_weight: f32,
    pub dependency_weight: f32,
    pub urgency_weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceThresholds {
    pub high_confidence: f32,
    pub medium_confidence: f32,
    pub low_confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionGenerationConfig {
    pub max_recommended_actions: usize,
    pub prioritize_critical_conflicts: bool,
    pub include_effort_estimates: bool,
    pub generate_timeline_suggestions: bool,
}

impl Default for DualCategoryStatusConfig {
    fn default() -> Self {
        Self {
            enable_dual_analysis: true,
            completion_score_weights: CompletionScoreWeights {
                explicit_weight: 0.4,
                inferred_weight: 0.3,
                implementation_weight: 0.2,
                documentation_weight: 0.1,
            },
            priority_calculation_weights: PriorityWeights {
                business_impact_weight: 0.4,
                technical_complexity_weight: 0.2,
                dependency_weight: 0.2,
                urgency_weight: 0.2,
            },
            confidence_thresholds: ConfidenceThresholds {
                high_confidence: 0.8,
                medium_confidence: 0.6,
                low_confidence: 0.4,
            },
            action_generation_config: ActionGenerationConfig {
                max_recommended_actions: 10,
                prioritize_critical_conflicts: true,
                include_effort_estimates: true,
                generate_timeline_suggestions: true,
            },
        }
    }
}

/// Main analyzer for dual-category status analysis
pub struct DualCategoryStatusAnalyzer {
    config: DualCategoryStatusConfig,
    claims_extractor: DocumentationClaimsExtractor,
    reality_analyzer: CodeRealityAnalyzer,
    conflict_resolver: ConflictResolutionEngine,
    todo_scanner: TodoScanner,
    status_engine: StatusInferenceEngine,
}

impl DualCategoryStatusAnalyzer {
    /// Create new dual-category status analyzer
    pub fn new() -> Result<Self> {
        let config = DualCategoryStatusConfig::default();
        Self::with_config(config)
    }
    
    /// Create analyzer with custom configuration
    pub fn with_config(config: DualCategoryStatusConfig) -> Result<Self> {
        let claims_extractor = DocumentationClaimsExtractor::new()
            .context("Failed to create documentation claims extractor")?;
            
        let reality_analyzer = CodeRealityAnalyzer::new()
            .context("Failed to create code reality analyzer")?;
            
        let conflict_resolver = ConflictResolutionEngine::new();
        
        let todo_scanner = TodoScanner::new()
            .context("Failed to create TODO scanner")?;
            
        let status_engine = StatusInferenceEngine::new()
            .context("Failed to create status inference engine")?;
        
        println!("Initialized DualCategoryStatusAnalyzer with comprehensive analysis pipeline");
        
        Ok(Self {
            config,
            claims_extractor,
            reality_analyzer,
            conflict_resolver,
            todo_scanner,
            status_engine,
        })
    }
    
    /// Perform comprehensive dual-category status analysis
    pub fn analyze<P: AsRef<Path>>(&self, project_path: P) -> Result<DualCategoryStatusResult> {
        if !self.config.enable_dual_analysis {
            return Ok(self.create_empty_result());
        }
        
        let start_time = std::time::Instant::now();
        let project_path = project_path.as_ref();
        
        println!("Starting dual-category status analysis for: {}", project_path.display());
        
        // Phase 1: Extract documentation claims
        let claims_result = self.claims_extractor.extract_claims(project_path)
            .context("Failed to extract documentation claims")?;
        println!("Phase 1 complete: {} claims extracted", claims_result.claims.len());
        
        // Phase 2: Analyze code reality
        let reality_result = self.reality_analyzer.analyze_reality(project_path)
            .context("Failed to analyze code reality")?;
        println!("Phase 2 complete: {} implementations found", reality_result.implementations.len());
        
        // Phase 3: Resolve conflicts
        let conflicts_result = self.conflict_resolver.resolve_conflicts(
            &claims_result.claims, 
            &reality_result.implementations
        ).context("Failed to resolve conflicts")?;
        println!("Phase 3 complete: {} conflicts identified", conflicts_result.conflicts.len());
        
        // Phase 4: Explicit status analysis (TODOs)
        let todo_result = self.todo_scanner.scan_project(project_path)
            .context("Failed to scan TODOs")?;
        println!("Phase 4 complete: {} TODO items found", todo_result.todo_items.len());
        
        // Phase 5: Status inference analysis (create minimal documentation info for now)
        let minimal_docs = crate::core::documentation_extractor::ExtractedDocumentationInfo {
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
        };
        let status_result = self.status_engine.analyze_project_status(project_path, &minimal_docs, &todo_result)
            .context("Failed to analyze project status")?;
        println!("Phase 5 complete: Status inference analysis finished");
        
        // Phase 6: Merge and analyze
        let explicit_status = self.build_explicit_status_analysis(&todo_result)?;
        let inferred_status = self.build_inferred_status_analysis(
            &claims_result, 
            &reality_result, 
            &conflicts_result,
            &status_result
        )?;
        let merged_status = self.build_merged_status_analysis(&explicit_status, &inferred_status)?;
        let consistency_analysis = self.build_consistency_analysis(&inferred_status)?;
        
        let total_time = start_time.elapsed().as_millis() as u64;
        let metadata = DualCategoryAnalysisMetadata {
            total_analysis_time_ms: total_time,
            files_analyzed: inferred_status.code_reality.metadata.files_analyzed,
            claims_processed: inferred_status.documentation_claims.claims.len(),
            implementations_found: inferred_status.code_reality.implementations.len(),
            conflicts_resolved: inferred_status.conflicts.conflicts.len(),
            confidence_level: merged_status.confidence_score,
        };
        
        println!("Dual-category status analysis completed in {}ms", total_time);
        println!("Overall completion score: {:.1}%", merged_status.overall_completion_score * 100.0);
        
        Ok(DualCategoryStatusResult {
            explicit_status,
            inferred_status,
            merged_status,
            consistency_analysis,
            metadata,
        })
    }
    
    /// Create empty result when analysis is disabled
    fn create_empty_result(&self) -> DualCategoryStatusResult {
        DualCategoryStatusResult {
            explicit_status: ExplicitStatusAnalysis {
                todo_analysis: TodoScanResult {
                    todo_items: Vec::new(),
                    summary: crate::core::todo_scanner::TodoSummary {
                        total_items: 0,
                        by_type: HashMap::new(),
                        by_priority: HashMap::new(),
                        by_file_type: HashMap::new(),
                        most_problematic_files: Vec::new(),
                    },
                    scan_metadata: crate::core::todo_scanner::TodoScanMetadata {
                        files_scanned: 0,
                        total_lines_scanned: 0,
                        scan_duration_ms: 0,
                        file_types_scanned: Vec::new(),
                        excluded_patterns: Vec::new(),
                        scan_errors: Vec::new(),
                    },
                },
                placeholder_implementations: Vec::new(),
                explicit_todos_by_category: HashMap::new(),
                completion_score: 1.0,
            },
            inferred_status: InferredStatusAnalysis {
                documentation_claims: crate::core::documentation_claims_extractor::DocumentationClaimsResult {
                    claims: Vec::new(),
                    summary: crate::core::documentation_claims_extractor::ClaimsSummary {
                        total_claims: 0,
                        claims_by_type: HashMap::new(),
                        claims_by_priority: HashMap::new(),
                        high_confidence_claims: 0,
                        verification_needed: 0,
                    },
                    metadata: crate::core::documentation_claims_extractor::ClaimsExtractionMetadata {
                        files_processed: 0,
                        extraction_time_ms: 0,
                        patterns_used: 0,
                        average_confidence: 0.0,
                    },
                },
                code_reality: crate::core::code_reality_analyzer::CodeRealityResult {
                    implementations: Vec::new(),
                    summary: crate::core::code_reality_analyzer::RealitySummary {
                        total_implementations: 0,
                        implementations_by_type: HashMap::new(),
                        implementations_by_level: HashMap::new(),
                        fully_implemented_features: 0,
                        partially_implemented_features: 0,
                        placeholder_implementations: 0,
                        overall_implementation_score: 0.0,
                    },
                    metadata: crate::core::code_reality_analyzer::RealityAnalysisMetadata {
                        files_analyzed: 0,
                        analysis_time_ms: 0,
                        patterns_matched: 0,
                        dependencies_discovered: 0,
                        code_lines_scanned: 0,
                    },
                },
                conflicts: crate::core::conflict_resolution_engine::ConflictResolutionResult {
                    conflicts: Vec::new(),
                    resolution_summary: crate::core::conflict_resolution_engine::ResolutionSummary {
                        total_conflicts: 0,
                        conflicts_by_type: HashMap::new(),
                        conflicts_by_severity: HashMap::new(),
                        critical_conflicts: 0,
                        resolved_conflicts: 0,
                        flagged_for_review: 0,
                        overall_consistency_score: 1.0,
                    },
                    metadata: crate::core::conflict_resolution_engine::ConflictAnalysisMetadata {
                        claims_analyzed: 0,
                        implementations_analyzed: 0,
                        analysis_time_ms: 0,
                        matching_pairs_found: 0,
                        unmatched_claims: 0,
                        unmatched_implementations: 0,
                    },
                },
                incomplete_features: Vec::new(),
                missing_implementations: Vec::new(),
                partial_implementations: Vec::new(),
                inferred_completion_score: 1.0,
            },
            merged_status: MergedStatusAnalysis {
                comprehensive_feature_status: Vec::new(),
                implementation_priorities: Vec::new(),
                overall_completion_score: 1.0,
                confidence_score: 1.0,
                recommended_next_steps: Vec::new(),
            },
            consistency_analysis: ConsistencyAnalysis {
                documentation_accuracy: 1.0,
                implementation_coverage: 1.0,
                consistency_score: 1.0,
                major_discrepancies: Vec::new(),
                recommendations: Vec::new(),
            },
            metadata: DualCategoryAnalysisMetadata {
                total_analysis_time_ms: 0,
                files_analyzed: 0,
                claims_processed: 0,
                implementations_found: 0,
                conflicts_resolved: 0,
                confidence_level: 1.0,
            },
        }
    }
    
    /// Build explicit status analysis from TODO scan results
    fn build_explicit_status_analysis(
        &self, 
        todo_result: &TodoScanResult
    ) -> Result<ExplicitStatusAnalysis> {
        let placeholder_implementations = self.extract_placeholder_implementations(todo_result);
        let explicit_todos_by_category = self.categorize_todos(todo_result);
        
        // Calculate completion score based on TODO density
        let completion_score = if todo_result.summary.total_items == 0 {
            1.0
        } else {
            // Lower score with more TODOs, but don't go below 0.1
            (1.0 - (todo_result.summary.total_items as f32 * 0.01)).max(0.1)
        };
        
        Ok(ExplicitStatusAnalysis {
            todo_analysis: todo_result.clone(),
            placeholder_implementations,
            explicit_todos_by_category,
            completion_score,
        })
    }
    
    /// Build inferred status analysis from claims, reality, and conflicts
    fn build_inferred_status_analysis(
        &self,
        claims_result: &crate::core::documentation_claims_extractor::DocumentationClaimsResult,
        reality_result: &crate::core::code_reality_analyzer::CodeRealityResult,
        conflicts_result: &crate::core::conflict_resolution_engine::ConflictResolutionResult,
        status_result: &crate::core::status_inference_engine::DualCategoryStatusResult,
    ) -> Result<InferredStatusAnalysis> {
        let incomplete_features = self.identify_incomplete_features(claims_result, reality_result, conflicts_result);
        let missing_implementations = self.identify_missing_implementations(conflicts_result);
        let partial_implementations = self.identify_partial_implementations(reality_result, conflicts_result);
        
        // Calculate inferred completion score
        let base_score = reality_result.summary.overall_implementation_score;
        let conflict_penalty = conflicts_result.resolution_summary.critical_conflicts as f32 * 0.1;
        let inferred_completion_score = (base_score - conflict_penalty).max(0.0);
        
        Ok(InferredStatusAnalysis {
            documentation_claims: claims_result.clone(),
            code_reality: reality_result.clone(),
            conflicts: conflicts_result.clone(),
            incomplete_features,
            missing_implementations,
            partial_implementations,
            inferred_completion_score,
        })
    }
    
    /// Build merged status analysis combining explicit and inferred results
    fn build_merged_status_analysis(
        &self,
        explicit: &ExplicitStatusAnalysis,
        inferred: &InferredStatusAnalysis,
    ) -> Result<MergedStatusAnalysis> {
        let comprehensive_feature_status = self.merge_feature_statuses(explicit, inferred);
        let implementation_priorities = self.calculate_implementation_priorities(explicit, inferred);
        
        // Calculate overall completion score using configured weights
        let overall_completion_score = 
            explicit.completion_score * self.config.completion_score_weights.explicit_weight +
            inferred.inferred_completion_score * self.config.completion_score_weights.inferred_weight +
            inferred.code_reality.summary.overall_implementation_score * self.config.completion_score_weights.implementation_weight;
            
        let confidence_score = inferred.conflicts.resolution_summary.overall_consistency_score;
        let recommended_next_steps = self.generate_recommended_actions(explicit, inferred);
        
        Ok(MergedStatusAnalysis {
            comprehensive_feature_status,
            implementation_priorities,
            overall_completion_score,
            confidence_score,
            recommended_next_steps,
        })
    }
    
    /// Build consistency analysis
    fn build_consistency_analysis(
        &self,
        inferred: &InferredStatusAnalysis,
    ) -> Result<ConsistencyAnalysis> {
        let documentation_accuracy = self.calculate_documentation_accuracy(inferred);
        let implementation_coverage = self.calculate_implementation_coverage(inferred);
        let consistency_score = inferred.conflicts.resolution_summary.overall_consistency_score;
        let major_discrepancies = self.identify_major_discrepancies(&inferred.conflicts);
        let recommendations = self.generate_consistency_recommendations(inferred);
        
        Ok(ConsistencyAnalysis {
            documentation_accuracy,
            implementation_coverage,
            consistency_score,
            major_discrepancies,
            recommendations,
        })
    }
    
    // Helper methods for building analysis components
    
    fn extract_placeholder_implementations(&self, todo_result: &TodoScanResult) -> Vec<PlaceholderImplementation> {
        todo_result.todo_items.iter()
            .filter(|todo| matches!(todo.todo_type, crate::core::todo_scanner::TodoType::Todo))
            .map(|todo| PlaceholderImplementation {
                feature_name: todo.description.split_whitespace().take(3).collect::<Vec<_>>().join(" "),
                file_path: todo.file_path.to_string_lossy().to_string(),
                line_number: todo.line_number as usize,
                placeholder_type: todo.todo_type.as_str().to_string(),
                description: todo.description.clone(),
                urgency_score: match todo.priority {
                    crate::core::todo_scanner::TodoPriority::Critical => 1.0,
                    crate::core::todo_scanner::TodoPriority::High => 0.8,
                    crate::core::todo_scanner::TodoPriority::Medium => 0.5,
                    crate::core::todo_scanner::TodoPriority::Low => 0.2,
                },
            })
            .collect()
    }
    
    fn categorize_todos(&self, todo_result: &TodoScanResult) -> HashMap<String, Vec<ExplicitTodo>> {
        let mut categorized = HashMap::new();
        
        for todo in &todo_result.todo_items {
            let category = self.categorize_todo_by_content(&todo.description);
            let explicit_todo = ExplicitTodo {
                description: todo.description.clone(),
                file_path: todo.file_path.to_string_lossy().to_string(),
                line_number: todo.line_number as usize,
                category: category.clone(),
                priority_score: match todo.priority {
                    crate::core::todo_scanner::TodoPriority::Critical => 1.0,
                    crate::core::todo_scanner::TodoPriority::High => 0.8,
                    crate::core::todo_scanner::TodoPriority::Medium => 0.5,
                    crate::core::todo_scanner::TodoPriority::Low => 0.2,
                },
                estimated_effort: self.estimate_effort_from_description(&todo.description),
            };
            
            categorized.entry(category).or_insert_with(Vec::new).push(explicit_todo);
        }
        
        categorized
    }
    
    fn categorize_todo_by_content(&self, description: &str) -> String {
        let desc_lower = description.to_lowercase();
        
        if desc_lower.contains("auth") || desc_lower.contains("login") || desc_lower.contains("security") {
            "Authentication".to_string()
        } else if desc_lower.contains("api") || desc_lower.contains("endpoint") || desc_lower.contains("rest") {
            "API".to_string()
        } else if desc_lower.contains("database") || desc_lower.contains("db") || desc_lower.contains("query") {
            "Database".to_string()
        } else if desc_lower.contains("test") || desc_lower.contains("spec") {
            "Testing".to_string()
        } else if desc_lower.contains("ui") || desc_lower.contains("frontend") || desc_lower.contains("component") {
            "Frontend".to_string()
        } else if desc_lower.contains("performance") || desc_lower.contains("optimize") || desc_lower.contains("cache") {
            "Performance".to_string()
        } else {
            "General".to_string()
        }
    }
    
    fn estimate_effort_from_description(&self, description: &str) -> EffortLevel {
        let desc_lower = description.to_lowercase();
        let word_count = description.split_whitespace().count();
        
        if desc_lower.contains("refactor") || desc_lower.contains("rewrite") || desc_lower.contains("implement") {
            if word_count > 10 {
                EffortLevel::Large
            } else {
                EffortLevel::Medium
            }
        } else if desc_lower.contains("fix") || desc_lower.contains("update") || desc_lower.contains("change") {
            EffortLevel::Small
        } else if desc_lower.contains("add") || desc_lower.contains("remove") {
            EffortLevel::Small
        } else {
            EffortLevel::Medium
        }
    }
    
    fn identify_incomplete_features(
        &self,
        claims: &crate::core::documentation_claims_extractor::DocumentationClaimsResult,
        reality: &crate::core::code_reality_analyzer::CodeRealityResult,
        conflicts: &crate::core::conflict_resolution_engine::ConflictResolutionResult,
    ) -> Vec<IncompleteFeature> {
        let mut incomplete_features = Vec::new();
        
        for conflict in &conflicts.conflicts {
            if let Some(claim) = &conflict.documentation_claim {
                if let Some(implementation) = &conflict.implementation_evidence {
                    if matches!(implementation.implementation_level, ImplementationLevel::Partial | ImplementationLevel::Skeleton) {
                        incomplete_features.push(IncompleteFeature {
                            feature_name: claim.description.clone(),
                            claimed_capabilities: claim.keywords.clone(),
                            actual_implementation: Some(implementation.description.clone()),
                            completion_percentage: match implementation.implementation_level {
                                ImplementationLevel::Partial => 0.5,
                                ImplementationLevel::Skeleton => 0.2,
                                _ => 0.8,
                            },
                            missing_components: self.identify_missing_components(claim, implementation),
                            evidence: implementation.code_snippets.clone(),
                        });
                    }
                }
            }
        }
        
        incomplete_features
    }
    
    fn identify_missing_implementations(
        &self,
        conflicts: &crate::core::conflict_resolution_engine::ConflictResolutionResult,
    ) -> Vec<MissingImplementation> {
        conflicts.conflicts.iter()
            .filter_map(|conflict| {
                if matches!(conflict.conflict_type, crate::core::conflict_resolution_engine::ConflictType::ClaimedButNotImplemented) {
                    if let Some(claim) = &conflict.documentation_claim {
                        Some(MissingImplementation {
                            feature_name: claim.description.clone(),
                            documentation_source: claim.source_file.to_string_lossy().to_string(),
                            expected_implementation: format!("Implementation for {}", claim.claim_type.as_str()),
                            suggested_location: None,
                            priority_score: conflict.confidence,
                            dependencies: Vec::new(),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }
    
    fn identify_partial_implementations(
        &self,
        reality: &crate::core::code_reality_analyzer::CodeRealityResult,
        conflicts: &crate::core::conflict_resolution_engine::ConflictResolutionResult,
    ) -> Vec<PartialImplementation> {
        reality.implementations.iter()
            .filter(|impl_evidence| matches!(impl_evidence.implementation_level, ImplementationLevel::Partial))
            .map(|impl_evidence| PartialImplementation {
                feature_name: impl_evidence.description.clone(),
                implemented_parts: impl_evidence.code_snippets.clone(),
                missing_parts: vec!["Error handling".to_string(), "Input validation".to_string(), "Tests".to_string()],
                implementation_quality: match impl_evidence.confidence {
                    c if c > 0.8 => ImplementationQuality::Development,
                    c if c > 0.5 => ImplementationQuality::Prototype,
                    _ => ImplementationQuality::Skeleton,
                },
                completion_estimate: impl_evidence.confidence,
            })
            .collect()
    }
    
    fn identify_missing_components(&self, claim: &DocumentationClaim, implementation: &ImplementationEvidence) -> Vec<String> {
        let mut missing = Vec::new();
        
        // Simple heuristic based on claim keywords vs implementation evidence
        for keyword in &claim.keywords {
            if !implementation.code_snippets.iter().any(|snippet| 
                snippet.to_lowercase().contains(&keyword.to_lowercase())
            ) {
                missing.push(format!("Missing {}", keyword));
            }
        }
        
        if missing.is_empty() {
            missing.push("Unknown missing components".to_string());
        }
        
        missing
    }
    
    fn merge_feature_statuses(
        &self,
        explicit: &ExplicitStatusAnalysis,
        inferred: &InferredStatusAnalysis,
    ) -> Vec<FeatureStatus> {
        let mut feature_statuses = Vec::new();
        
        // Process claims with implementations
        for claim in &inferred.documentation_claims.claims {
            let matching_impl = inferred.code_reality.implementations.iter()
                .find(|impl_evidence| self.features_match(&claim.description, &impl_evidence.description));
                
            let status_category = match matching_impl {
                Some(impl_evidence) => match impl_evidence.implementation_level {
                    ImplementationLevel::Complete => StatusCategory::CompletelyImplemented,
                    ImplementationLevel::Partial => StatusCategory::PartiallyImplemented,
                    _ => StatusCategory::PlannedNotImplemented,
                },
                None => StatusCategory::PlannedNotImplemented,
            };
            
            let completion_level = match matching_impl {
                Some(impl_evidence) => match impl_evidence.confidence {
                    c if c >= 0.9 => CompletionLevel::Complete,
                    c if c >= 0.7 => CompletionLevel::NearComplete,
                    c if c >= 0.3 => CompletionLevel::Partial,
                    c if c >= 0.1 => CompletionLevel::Minimal,
                    _ => CompletionLevel::NotStarted,
                },
                None => CompletionLevel::NotStarted,
            };
            
            feature_statuses.push(FeatureStatus {
                feature_name: claim.description.clone(),
                documentation_claim: Some(claim.clone()),
                implementation_evidence: matching_impl.cloned(),
                status_category,
                completion_level,
                confidence: matching_impl.map(|i| i.confidence).unwrap_or(claim.confidence),
                next_actions: self.generate_feature_actions(&status_category, &completion_level),
            });
        }
        
        // Process unmatched implementations
        for impl_evidence in &inferred.code_reality.implementations {
            if !feature_statuses.iter().any(|fs| fs.implementation_evidence.as_ref().map(|ie| &ie.description) == Some(&impl_evidence.description)) {
                feature_statuses.push(FeatureStatus {
                    feature_name: impl_evidence.description.clone(),
                    documentation_claim: None,
                    implementation_evidence: Some(impl_evidence.clone()),
                    status_category: StatusCategory::UnplannedImplemented,
                    completion_level: match impl_evidence.implementation_level {
                        ImplementationLevel::Complete => CompletionLevel::Complete,
                        ImplementationLevel::Partial => CompletionLevel::Partial,
                        ImplementationLevel::Skeleton => CompletionLevel::Minimal,
                        ImplementationLevel::Placeholder => CompletionLevel::NotStarted,
                    },
                    confidence: impl_evidence.confidence,
                    next_actions: vec!["Document this feature".to_string()],
                });
            }
        }
        
        feature_statuses
    }
    
    fn features_match(&self, claim_desc: &str, impl_desc: &str) -> bool {
        // Simple similarity check
        let claim_desc_lower = claim_desc.to_lowercase();
        let impl_desc_lower = impl_desc.to_lowercase();
        let claim_words: std::collections::HashSet<_> = claim_desc_lower.split_whitespace().collect();
        let impl_words: std::collections::HashSet<_> = impl_desc_lower.split_whitespace().collect();
        
        let intersection = claim_words.intersection(&impl_words).count();
        let union = claim_words.union(&impl_words).count();
        
        if union == 0 {
            false
        } else {
            intersection as f32 / union as f32 > 0.3
        }
    }
    
    fn generate_feature_actions(&self, status: &StatusCategory, completion: &CompletionLevel) -> Vec<String> {
        match (status, completion) {
            (StatusCategory::PlannedNotImplemented, _) => vec!["Implement this feature".to_string()],
            (StatusCategory::PartiallyImplemented, CompletionLevel::Partial) => vec![
                "Complete the implementation".to_string(),
                "Add error handling".to_string(),
                "Add tests".to_string()
            ],
            (StatusCategory::CompletelyImplemented, _) => vec!["Consider enhancements".to_string()],
            (StatusCategory::UnplannedImplemented, _) => vec!["Add documentation".to_string()],
            (StatusCategory::Conflicted, _) => vec!["Resolve conflicts between docs and code".to_string()],
            _ => vec!["Review implementation status".to_string()],
        }
    }
    
    fn calculate_implementation_priorities(
        &self,
        _explicit: &ExplicitStatusAnalysis,
        inferred: &InferredStatusAnalysis,
    ) -> Vec<ImplementationPriority> {
        let mut priorities = Vec::new();
        
        // High priority for missing critical features
        for conflict in &inferred.conflicts.conflicts {
            if matches!(conflict.conflict_type, crate::core::conflict_resolution_engine::ConflictType::ClaimedButNotImplemented) {
                if let Some(claim) = &conflict.documentation_claim {
                    let business_impact = match claim.priority {
                        crate::core::documentation_claims_extractor::ClaimPriority::Critical => BusinessImpact::Critical,
                        crate::core::documentation_claims_extractor::ClaimPriority::High => BusinessImpact::High,
                        crate::core::documentation_claims_extractor::ClaimPriority::Medium => BusinessImpact::Medium,
                        crate::core::documentation_claims_extractor::ClaimPriority::Low => BusinessImpact::Low,
                    };
                    
                    let technical_complexity = match claim.claim_type {
                        crate::core::documentation_claims_extractor::ClaimType::Security => TechnicalComplexity::High,
                        crate::core::documentation_claims_extractor::ClaimType::Integration => TechnicalComplexity::High,
                        crate::core::documentation_claims_extractor::ClaimType::Api => TechnicalComplexity::Medium,
                        _ => TechnicalComplexity::Low,
                    };
                    
                    let priority_score = self.calculate_priority_score(&business_impact, &technical_complexity, conflict.confidence);
                    
                    priorities.push(ImplementationPriority {
                        feature_name: claim.description.clone(),
                        priority_score,
                        business_impact,
                        technical_complexity,
                        dependencies: Vec::new(),
                        estimated_effort: match technical_complexity {
                            TechnicalComplexity::High => EffortLevel::Large,
                            TechnicalComplexity::Medium => EffortLevel::Medium,
                            TechnicalComplexity::Low => EffortLevel::Small,
                        },
                        recommended_timeline: self.generate_timeline_recommendation(&business_impact, &technical_complexity),
                    });
                }
            }
        }
        
        // Sort by priority score
        priorities.sort_by(|a, b| b.priority_score.partial_cmp(&a.priority_score).unwrap_or(std::cmp::Ordering::Equal));
        
        priorities
    }
    
    fn calculate_priority_score(&self, business_impact: &BusinessImpact, technical_complexity: &TechnicalComplexity, confidence: f32) -> f32 {
        let business_weight = match business_impact {
            BusinessImpact::Critical => 1.0,
            BusinessImpact::High => 0.8,
            BusinessImpact::Medium => 0.5,
            BusinessImpact::Low => 0.2,
        } * self.config.priority_calculation_weights.business_impact_weight;
        
        let complexity_weight = match technical_complexity {
            TechnicalComplexity::Low => 0.8,  // Higher priority for easier tasks
            TechnicalComplexity::Medium => 0.6,
            TechnicalComplexity::High => 0.3,
        } * self.config.priority_calculation_weights.technical_complexity_weight;
        
        let confidence_weight = confidence * 0.2;
        
        business_weight + complexity_weight + confidence_weight
    }
    
    fn generate_timeline_recommendation(&self, business_impact: &BusinessImpact, technical_complexity: &TechnicalComplexity) -> String {
        match (business_impact, technical_complexity) {
            (BusinessImpact::Critical, TechnicalComplexity::Low) => "Next sprint".to_string(),
            (BusinessImpact::Critical, TechnicalComplexity::Medium) => "Within 2 sprints".to_string(),
            (BusinessImpact::Critical, TechnicalComplexity::High) => "Within 1 month".to_string(),
            (BusinessImpact::High, TechnicalComplexity::Low) => "Within 2 sprints".to_string(),
            (BusinessImpact::High, TechnicalComplexity::Medium) => "Within 1 month".to_string(),
            (BusinessImpact::High, TechnicalComplexity::High) => "Within 2 months".to_string(),
            (BusinessImpact::Medium, _) => "Within 3 months".to_string(),
            (BusinessImpact::Low, _) => "When capacity allows".to_string(),
        }
    }
    
    fn generate_recommended_actions(
        &self,
        explicit: &ExplicitStatusAnalysis,
        inferred: &InferredStatusAnalysis,
    ) -> Vec<RecommendedAction> {
        let mut actions = Vec::new();
        
        // Actions from critical conflicts
        for conflict in &inferred.conflicts.conflicts {
            if matches!(conflict.severity, crate::core::conflict_resolution_engine::ConflictSeverity::Critical) {
                let action_type = match conflict.conflict_type {
                    crate::core::conflict_resolution_engine::ConflictType::ClaimedButNotImplemented => ActionType::Implement,
                    crate::core::conflict_resolution_engine::ConflictType::ImplementedButNotClaimed => ActionType::Document,
                    crate::core::conflict_resolution_engine::ConflictType::ImplementationMismatch => ActionType::Complete,
                    _ => ActionType::Fix,
                };
                
                actions.push(RecommendedAction {
                    action_type,
                    description: conflict.recommended_action.clone(),
                    target_feature: conflict.documentation_claim.as_ref()
                        .map(|c| c.description.clone())
                        .or_else(|| conflict.implementation_evidence.as_ref().map(|i| i.description.clone()))
                        .unwrap_or_else(|| "Unknown feature".to_string()),
                    priority: ActionPriority::Urgent,
                    estimated_effort: EffortLevel::Medium,
                    prerequisites: Vec::new(),
                });
            }
        }
        
        // Actions from high-priority TODOs
        for todo in &explicit.todo_analysis.todo_items {
            if matches!(todo.priority, crate::core::todo_scanner::TodoPriority::Critical | crate::core::todo_scanner::TodoPriority::High) {
                actions.push(RecommendedAction {
                    action_type: ActionType::Implement,
                    description: format!("Address TODO: {}", todo.description),
                    target_feature: todo.description.clone(),
                    priority: match todo.priority {
                        crate::core::todo_scanner::TodoPriority::Critical => ActionPriority::Urgent,
                        crate::core::todo_scanner::TodoPriority::High => ActionPriority::High,
                        _ => ActionPriority::Medium,
                    },
                    estimated_effort: self.estimate_effort_from_description(&todo.description),
                    prerequisites: Vec::new(),
                });
            }
        }
        
        // Limit to configured maximum
        actions.truncate(self.config.action_generation_config.max_recommended_actions);
        
        actions
    }
    
    fn calculate_documentation_accuracy(&self, inferred: &InferredStatusAnalysis) -> f32 {
        let total_claims = inferred.documentation_claims.claims.len() as f32;
        if total_claims == 0.0 {
            return 1.0;
        }
        
        let false_claims = inferred.conflicts.conflicts.iter()
            .filter(|c| matches!(c.conflict_type, crate::core::conflict_resolution_engine::ConflictType::ClaimedButNotImplemented))
            .count() as f32;
            
        (1.0 - (false_claims / total_claims)).max(0.0)
    }
    
    fn calculate_implementation_coverage(&self, inferred: &InferredStatusAnalysis) -> f32 {
        let total_implementations = inferred.code_reality.implementations.len() as f32;
        if total_implementations == 0.0 {
            return 0.0;
        }
        
        let documented_implementations = inferred.conflicts.metadata.matching_pairs_found as f32;
        (documented_implementations / total_implementations).min(1.0)
    }
    
    fn identify_major_discrepancies(
        &self,
        conflicts: &crate::core::conflict_resolution_engine::ConflictResolutionResult,
    ) -> Vec<MajorDiscrepancy> {
        conflicts.conflicts.iter()
            .filter(|c| matches!(c.severity, crate::core::conflict_resolution_engine::ConflictSeverity::Critical | crate::core::conflict_resolution_engine::ConflictSeverity::High))
            .map(|conflict| MajorDiscrepancy {
                discrepancy_type: format!("{:?}", conflict.conflict_type),
                description: conflict.description.clone(),
                severity: match conflict.severity {
                    crate::core::conflict_resolution_engine::ConflictSeverity::Critical => DiscrepancySeverity::Critical,
                    crate::core::conflict_resolution_engine::ConflictSeverity::High => DiscrepancySeverity::High,
                    crate::core::conflict_resolution_engine::ConflictSeverity::Medium => DiscrepancySeverity::Medium,
                    crate::core::conflict_resolution_engine::ConflictSeverity::Low => DiscrepancySeverity::Low,
                },
                documentation_claim: conflict.documentation_claim.as_ref().map(|c| c.description.clone()),
                actual_implementation: conflict.implementation_evidence.as_ref().map(|i| i.description.clone()),
                impact_assessment: self.assess_discrepancy_impact(conflict),
                resolution_recommendation: conflict.recommended_action.clone(),
            })
            .collect()
    }
    
    fn assess_discrepancy_impact(&self, conflict: &Conflict) -> String {
        match conflict.conflict_type {
            crate::core::conflict_resolution_engine::ConflictType::ClaimedButNotImplemented => 
                "Users may expect functionality that doesn't exist".to_string(),
            crate::core::conflict_resolution_engine::ConflictType::ImplementedButNotClaimed => 
                "Users may not be aware of available functionality".to_string(),
            crate::core::conflict_resolution_engine::ConflictType::SecurityMismatch => 
                "Security expectations may not match actual protection level".to_string(),
            _ => "May cause confusion for users and developers".to_string(),
        }
    }
    
    fn generate_consistency_recommendations(
        &self,
        inferred: &InferredStatusAnalysis,
    ) -> Vec<ConsistencyRecommendation> {
        let mut recommendations = Vec::new();
        
        let unmatched_claims = inferred.conflicts.metadata.unmatched_claims;
        let unmatched_implementations = inferred.conflicts.metadata.unmatched_implementations;
        
        if unmatched_claims > 0 {
            recommendations.push(ConsistencyRecommendation {
                recommendation_type: RecommendationType::ImplementFeature,
                description: format!("Implement {} features claimed in documentation", unmatched_claims),
                priority: RecommendationPriority::High,
                affected_components: vec!["Core functionality".to_string()],
                estimated_impact: "High user satisfaction improvement".to_string(),
            });
        }
        
        if unmatched_implementations > 0 {
            recommendations.push(ConsistencyRecommendation {
                recommendation_type: RecommendationType::UpdateDocumentation,
                description: format!("Document {} implemented features", unmatched_implementations),
                priority: RecommendationPriority::Medium,
                affected_components: vec!["Documentation".to_string()],
                estimated_impact: "Improved feature discoverability".to_string(),
            });
        }
        
        recommendations
    }
}

// Extension trait to add convenience methods to ClaimType
impl crate::core::documentation_claims_extractor::ClaimType {
    fn as_str(&self) -> &str {
        match self {
            Self::Feature => "feature",
            Self::Capability => "capability", 
            Self::Integration => "integration",
            Self::Technology => "technology",
            Self::Performance => "performance",
            Self::Security => "security",
            Self::Deployment => "deployment",
            Self::Api => "api",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_dual_category_analyzer_creation() {
        let analyzer = DualCategoryStatusAnalyzer::new();
        assert!(analyzer.is_ok());
    }
    
    #[test]
    fn test_effort_estimation() {
        let analyzer = DualCategoryStatusAnalyzer::new().unwrap();
        
        assert_eq!(
            analyzer.estimate_effort_from_description("refactor entire authentication system"),
            EffortLevel::Large
        );
        
        assert_eq!(
            analyzer.estimate_effort_from_description("fix typo"),
            EffortLevel::Small
        );
        
        assert_eq!(
            analyzer.estimate_effort_from_description("add logging"),
            EffortLevel::Small
        );
    }
    
    #[test]
    fn test_todo_categorization() {
        let analyzer = DualCategoryStatusAnalyzer::new().unwrap();
        
        assert_eq!(
            analyzer.categorize_todo_by_content("TODO: implement OAuth authentication"),
            "Authentication"
        );
        
        assert_eq!(
            analyzer.categorize_todo_by_content("TODO: add REST API endpoint"),
            "API"
        );
        
        assert_eq!(
            analyzer.categorize_todo_by_content("TODO: fix database query"),
            "Database"
        );
    }
    
    #[test]
    fn test_feature_matching() {
        let analyzer = DualCategoryStatusAnalyzer::new().unwrap();
        
        assert!(analyzer.features_match(
            "OAuth authentication system",
            "authentication with OAuth provider"
        ));
        
        assert!(!analyzer.features_match(
            "OAuth authentication",
            "database connection pooling"
        ));
    }
}