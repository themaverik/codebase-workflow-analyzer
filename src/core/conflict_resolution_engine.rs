use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::core::documentation_claims_extractor::{DocumentationClaim, ClaimType, ClaimPriority};
use crate::core::code_reality_analyzer::{ImplementationEvidence, RealityType, ImplementationLevel};

/// Types of conflicts between documentation and code
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConflictType {
    ClaimedButNotImplemented,    // Documentation claims feature but code doesn't have it
    ImplementedButNotClaimed,    // Code has feature but documentation doesn't mention it
    ImplementationMismatch,      // Different level of implementation than claimed
    TechnologyMismatch,          // Different technology stack than claimed
    SecurityMismatch,           // Security claims don't match actual implementation
    PerformanceMismatch,        // Performance claims don't match actual capabilities
}

/// Severity level for conflicts
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConflictSeverity {
    Critical,    // Major functionality mismatch
    High,        // Important feature discrepancy
    Medium,      // Minor inconsistency
    Low,         // Documentation lag or over-specification
}

/// A conflict between documentation claims and code reality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub conflict_type: ConflictType,
    pub severity: ConflictSeverity,
    pub description: String,
    pub documentation_claim: Option<DocumentationClaim>,
    pub implementation_evidence: Option<ImplementationEvidence>,
    pub confidence: f32,
    pub resolution_strategy: ResolutionStrategy,
    pub recommended_action: String,
}

/// Strategy for resolving conflicts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    PreferCode,              // Code analysis takes precedence (default)
    PreferDocumentation,     // Documentation claim takes precedence 
    Merge,                   // Combine both sources of information
    FlagAsInconsistent,      // Mark as conflict requiring manual review
}

/// Results from conflict resolution analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolutionResult {
    pub conflicts: Vec<Conflict>,
    pub resolution_summary: ResolutionSummary,
    pub metadata: ConflictAnalysisMetadata,
}

/// Summary statistics for conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionSummary {
    pub total_conflicts: usize,
    pub conflicts_by_type: HashMap<ConflictType, usize>,
    pub conflicts_by_severity: HashMap<ConflictSeverity, usize>,
    pub critical_conflicts: usize,
    pub resolved_conflicts: usize,
    pub flagged_for_review: usize,
    pub overall_consistency_score: f32,
}

/// Metadata about conflict analysis process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictAnalysisMetadata {
    pub claims_analyzed: usize,
    pub implementations_analyzed: usize,
    pub analysis_time_ms: u64,
    pub matching_pairs_found: usize,
    pub unmatched_claims: usize,
    pub unmatched_implementations: usize,
}

/// Configuration for conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolutionConfig {
    pub enable_conflict_resolution: bool,
    pub default_resolution_strategy: ResolutionStrategy,
    pub confidence_threshold: f32,
    pub claim_type_mappings: HashMap<ClaimType, RealityType>,
    pub severity_thresholds: SeverityThresholds,
    pub resolution_preferences: ResolutionPreferences,
}

/// Thresholds for determining conflict severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityThresholds {
    pub critical_threshold: f32,      // 0.9+ = Critical
    pub high_threshold: f32,          // 0.7+ = High
    pub medium_threshold: f32,        // 0.5+ = Medium
    // Below medium_threshold = Low
}

/// Preferences for conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionPreferences {
    pub prefer_code_for_security: bool,
    pub prefer_code_for_performance: bool,
    pub prefer_documentation_for_features: bool,
    pub always_merge_technology_info: bool,
    pub flag_critical_mismatches: bool,
}

impl Default for ConflictResolutionConfig {
    fn default() -> Self {
        let mut claim_type_mappings = HashMap::new();
        
        // Map documentation claim types to implementation reality types
        claim_type_mappings.insert(ClaimType::Feature, RealityType::AuthenticationImplemented);
        claim_type_mappings.insert(ClaimType::Api, RealityType::ApiEndpointsImplemented);
        claim_type_mappings.insert(ClaimType::Integration, RealityType::IntegrationImplemented);
        claim_type_mappings.insert(ClaimType::Security, RealityType::SecurityImplemented);
        claim_type_mappings.insert(ClaimType::Performance, RealityType::PerformanceOptimized);
        claim_type_mappings.insert(ClaimType::Technology, RealityType::DatabaseIntegration);
        claim_type_mappings.insert(ClaimType::Deployment, RealityType::DeploymentReady);
        
        Self {
            enable_conflict_resolution: true,
            default_resolution_strategy: ResolutionStrategy::PreferCode,
            confidence_threshold: 0.6,
            claim_type_mappings,
            severity_thresholds: SeverityThresholds {
                critical_threshold: 0.9,
                high_threshold: 0.7,
                medium_threshold: 0.5,
            },
            resolution_preferences: ResolutionPreferences {
                prefer_code_for_security: true,
                prefer_code_for_performance: true,
                prefer_documentation_for_features: false,
                always_merge_technology_info: true,
                flag_critical_mismatches: true,
            },
        }
    }
}

/// Engine for resolving conflicts between documentation claims and code reality
pub struct ConflictResolutionEngine {
    config: ConflictResolutionConfig,
}

impl ConflictResolutionEngine {
    /// Create new conflict resolution engine with default configuration
    pub fn new() -> Self {
        Self::with_config(ConflictResolutionConfig::default())
    }
    
    /// Create conflict resolution engine with custom configuration
    pub fn with_config(config: ConflictResolutionConfig) -> Self {
        println!("Initialized ConflictResolutionEngine with {} claim type mappings", 
                 config.claim_type_mappings.len());
        
        Self { config }
    }
    
    /// Resolve conflicts between documentation claims and code reality
    pub fn resolve_conflicts(
        &self,
        claims: &[DocumentationClaim],
        implementations: &[ImplementationEvidence]
    ) -> Result<ConflictResolutionResult> {
        if !self.config.enable_conflict_resolution {
            return Ok(ConflictResolutionResult {
                conflicts: Vec::new(),
                resolution_summary: ResolutionSummary {
                    total_conflicts: 0,
                    conflicts_by_type: HashMap::new(),
                    conflicts_by_severity: HashMap::new(),
                    critical_conflicts: 0,
                    resolved_conflicts: 0,
                    flagged_for_review: 0,
                    overall_consistency_score: 1.0,
                },
                metadata: ConflictAnalysisMetadata {
                    claims_analyzed: 0,
                    implementations_analyzed: 0,
                    analysis_time_ms: 0,
                    matching_pairs_found: 0,
                    unmatched_claims: 0,
                    unmatched_implementations: 0,
                },
            });
        }
        
        let start_time = std::time::Instant::now();
        
        println!("Starting conflict resolution: {} claims vs {} implementations", 
                 claims.len(), implementations.len());
        
        // Find matching pairs and conflicts
        let mut conflicts = Vec::new();
        let mut matching_pairs = 0;
        let mut processed_claims = std::collections::HashSet::new();
        let mut processed_implementations = std::collections::HashSet::new();
        
        // 1. Find direct matches and implementation mismatches
        for (claim_idx, claim) in claims.iter().enumerate() {
            for (impl_idx, implementation) in implementations.iter().enumerate() {
                if self.claims_match_implementation(claim, implementation) {
                    matching_pairs += 1;
                    processed_claims.insert(claim_idx);
                    processed_implementations.insert(impl_idx);
                    
                    // Check for implementation level mismatches
                    if let Some(conflict) = self.check_implementation_mismatch(claim, implementation) {
                        conflicts.push(conflict);
                    }
                    
                    break; // Only match each claim to one implementation
                }
            }
        }
        
        // 2. Find claimed but not implemented features
        for (claim_idx, claim) in claims.iter().enumerate() {
            if !processed_claims.contains(&claim_idx) {
                if let Some(conflict) = self.create_claimed_but_not_implemented_conflict(claim) {
                    conflicts.push(conflict);
                }
            }
        }
        
        // 3. Find implemented but not claimed features
        for (impl_idx, implementation) in implementations.iter().enumerate() {
            if !processed_implementations.contains(&impl_idx) {
                if let Some(conflict) = self.create_implemented_but_not_claimed_conflict(implementation) {
                    conflicts.push(conflict);
                }
            }
        }
        
        // Apply resolution strategies
        for conflict in &mut conflicts {
            self.apply_resolution_strategy(conflict);
        }
        
        let analysis_time = start_time.elapsed().as_millis() as u64;
        let summary = self.generate_resolution_summary(&conflicts);
        let metadata = ConflictAnalysisMetadata {
            claims_analyzed: claims.len(),
            implementations_analyzed: implementations.len(),
            analysis_time_ms: analysis_time,
            matching_pairs_found: matching_pairs,
            unmatched_claims: claims.len() - processed_claims.len(),
            unmatched_implementations: implementations.len() - processed_implementations.len(),
        };
        
        println!("Conflict resolution completed: {} conflicts found in {}ms",
                 conflicts.len(), analysis_time);
        
        Ok(ConflictResolutionResult {
            conflicts,
            resolution_summary: summary,
            metadata,
        })
    }
    
    /// Check if a claim matches an implementation
    fn claims_match_implementation(&self, claim: &DocumentationClaim, implementation: &ImplementationEvidence) -> bool {
        // Check direct type mapping
        if let Some(expected_reality_type) = self.config.claim_type_mappings.get(&claim.claim_type) {
            if implementation.reality_type == *expected_reality_type {
                return true;
            }
        }
        
        // Check semantic similarity between descriptions
        let similarity = self.calculate_semantic_similarity(&claim.description, &implementation.description);
        similarity > 0.7
    }
    
    /// Check for implementation level mismatches
    fn check_implementation_mismatch(&self, claim: &DocumentationClaim, implementation: &ImplementationEvidence) -> Option<Conflict> {
        let expected_level = self.infer_expected_implementation_level(claim);
        
        if implementation.implementation_level != expected_level {
            let severity = self.calculate_mismatch_severity(claim, implementation);
            let confidence = (claim.confidence + implementation.confidence) / 2.0;
            
            Some(Conflict {
                conflict_type: ConflictType::ImplementationMismatch,
                severity,
                description: format!(
                    "Implementation level mismatch: claimed as {} but implemented as {:?}",
                    self.level_to_string(&expected_level),
                    implementation.implementation_level
                ),
                documentation_claim: Some(claim.clone()),
                implementation_evidence: Some(implementation.clone()),
                confidence,
                resolution_strategy: self.config.default_resolution_strategy.clone(),
                recommended_action: "Verify implementation completeness or update documentation".to_string(),
            })
        } else {
            None
        }
    }
    
    /// Create conflict for claimed but not implemented features
    fn create_claimed_but_not_implemented_conflict(&self, claim: &DocumentationClaim) -> Option<Conflict> {
        let severity = self.calculate_claim_severity(claim);
        
        // Skip low-priority claims without implementation
        if matches!(severity, ConflictSeverity::Low) && claim.confidence < 0.8 {
            return None;
        }
        
        Some(Conflict {
            conflict_type: ConflictType::ClaimedButNotImplemented,
            severity,
            description: format!("Feature claimed in documentation but not found in code: {}", claim.description),
            documentation_claim: Some(claim.clone()),
            implementation_evidence: None,
            confidence: claim.confidence,
            resolution_strategy: if claim.confidence > 0.8 {
                ResolutionStrategy::FlagAsInconsistent
            } else {
                ResolutionStrategy::PreferCode
            },
            recommended_action: "Implement missing feature or remove from documentation".to_string(),
        })
    }
    
    /// Create conflict for implemented but not claimed features
    fn create_implemented_but_not_claimed_conflict(&self, implementation: &ImplementationEvidence) -> Option<Conflict> {
        // Skip placeholder implementations
        if matches!(implementation.implementation_level, ImplementationLevel::Placeholder) {
            return None;
        }
        
        let severity = match implementation.implementation_level {
            ImplementationLevel::Complete => ConflictSeverity::Medium,
            ImplementationLevel::Partial => ConflictSeverity::Low,
            _ => return None,
        };
        
        Some(Conflict {
            conflict_type: ConflictType::ImplementedButNotClaimed,
            severity,
            description: format!("Feature implemented in code but not documented: {}", implementation.description),
            documentation_claim: None,
            implementation_evidence: Some(implementation.clone()),
            confidence: implementation.confidence,
            resolution_strategy: ResolutionStrategy::Merge,
            recommended_action: "Add feature to documentation or verify if implementation should be removed".to_string(),
        })
    }
    
    /// Apply resolution strategy to a conflict
    fn apply_resolution_strategy(&self, conflict: &mut Conflict) {
        // Override default strategy based on preferences
        match (&conflict.conflict_type, &conflict.documentation_claim) {
            (ConflictType::SecurityMismatch, _) if self.config.resolution_preferences.prefer_code_for_security => {
                conflict.resolution_strategy = ResolutionStrategy::PreferCode;
            },
            (ConflictType::PerformanceMismatch, _) if self.config.resolution_preferences.prefer_code_for_performance => {
                conflict.resolution_strategy = ResolutionStrategy::PreferCode;
            },
            (ConflictType::TechnologyMismatch, _) if self.config.resolution_preferences.always_merge_technology_info => {
                conflict.resolution_strategy = ResolutionStrategy::Merge;
            },
            _ if matches!(conflict.severity, ConflictSeverity::Critical) && self.config.resolution_preferences.flag_critical_mismatches => {
                conflict.resolution_strategy = ResolutionStrategy::FlagAsInconsistent;
            },
            _ => {} // Use default strategy
        }
        
        // Update recommended action based on final strategy
        conflict.recommended_action = match conflict.resolution_strategy {
            ResolutionStrategy::PreferCode => "Use code analysis results; update documentation to match".to_string(),
            ResolutionStrategy::PreferDocumentation => "Use documentation claims; implement missing features".to_string(),
            ResolutionStrategy::Merge => "Combine information from both sources".to_string(),
            ResolutionStrategy::FlagAsInconsistent => "Manual review required - significant discrepancy detected".to_string(),
        };
    }
    
    /// Infer expected implementation level from documentation claim
    fn infer_expected_implementation_level(&self, claim: &DocumentationClaim) -> ImplementationLevel {
        // High-confidence, high-priority claims expect complete implementation
        if claim.confidence > 0.8 && matches!(claim.priority, ClaimPriority::Critical | ClaimPriority::High) {
            return ImplementationLevel::Complete;
        }
        
        // Security and API claims expect higher implementation levels
        if matches!(claim.claim_type, ClaimType::Security | ClaimType::Api) {
            return ImplementationLevel::Complete;
        }
        
        // Performance and integration claims might be partial
        if matches!(claim.claim_type, ClaimType::Performance | ClaimType::Integration) {
            return ImplementationLevel::Partial;
        }
        
        // Default expectation
        ImplementationLevel::Partial
    }
    
    /// Calculate mismatch severity between claim and implementation
    fn calculate_mismatch_severity(&self, claim: &DocumentationClaim, implementation: &ImplementationEvidence) -> ConflictSeverity {
        let priority_weight = match claim.priority {
            ClaimPriority::Critical => 0.4,
            ClaimPriority::High => 0.3,
            ClaimPriority::Medium => 0.2,
            ClaimPriority::Low => 0.1,
        };
        
        let confidence_weight = (claim.confidence + implementation.confidence) / 2.0 * 0.3;
        
        let implementation_gap = match implementation.implementation_level {
            ImplementationLevel::Complete => 0.0,
            ImplementationLevel::Partial => 0.2,
            ImplementationLevel::Skeleton => 0.5,
            ImplementationLevel::Placeholder => 0.8,
        };
        
        let severity_score = priority_weight + confidence_weight + implementation_gap;
        
        if severity_score >= self.config.severity_thresholds.critical_threshold {
            ConflictSeverity::Critical
        } else if severity_score >= self.config.severity_thresholds.high_threshold {
            ConflictSeverity::High
        } else if severity_score >= self.config.severity_thresholds.medium_threshold {
            ConflictSeverity::Medium
        } else {
            ConflictSeverity::Low
        }
    }
    
    /// Calculate severity for unimplemented claims
    fn calculate_claim_severity(&self, claim: &DocumentationClaim) -> ConflictSeverity {
        let base_severity = match claim.priority {
            ClaimPriority::Critical => ConflictSeverity::Critical,
            ClaimPriority::High => ConflictSeverity::High,
            ClaimPriority::Medium => ConflictSeverity::Medium,
            ClaimPriority::Low => ConflictSeverity::Low,
        };
        
        // Upgrade severity for high-confidence security/API claims
        if claim.confidence > 0.8 && matches!(claim.claim_type, ClaimType::Security | ClaimType::Api) {
            match base_severity {
                ConflictSeverity::Low => ConflictSeverity::Medium,
                ConflictSeverity::Medium => ConflictSeverity::High,
                ConflictSeverity::High => ConflictSeverity::Critical,
                ConflictSeverity::Critical => ConflictSeverity::Critical,
            }
        } else {
            base_severity
        }
    }
    
    /// Calculate semantic similarity between two descriptions
    fn calculate_semantic_similarity(&self, desc1: &str, desc2: &str) -> f32 {
        // Simple implementation - can be enhanced with more sophisticated NLP
        let desc1_lower = desc1.to_lowercase();
        let words1: std::collections::HashSet<_> = desc1_lower
            .split_whitespace()
            .filter(|word| word.len() > 2)
            .collect();
            
        let desc2_lower = desc2.to_lowercase();
        let words2: std::collections::HashSet<_> = desc2_lower
            .split_whitespace()
            .filter(|word| word.len() > 2)
            .collect();
        
        if words1.is_empty() && words2.is_empty() {
            return 1.0;
        }
        
        if words1.is_empty() || words2.is_empty() {
            return 0.0;
        }
        
        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();
        
        intersection as f32 / union as f32
    }
    
    /// Convert implementation level to string
    fn level_to_string(&self, level: &ImplementationLevel) -> &str {
        match level {
            ImplementationLevel::Complete => "complete",
            ImplementationLevel::Partial => "partial", 
            ImplementationLevel::Skeleton => "skeleton",
            ImplementationLevel::Placeholder => "placeholder",
        }
    }
    
    /// Generate summary statistics for conflict resolution
    fn generate_resolution_summary(&self, conflicts: &[Conflict]) -> ResolutionSummary {
        let mut conflicts_by_type = HashMap::new();
        let mut conflicts_by_severity = HashMap::new();
        let mut critical_conflicts = 0;
        let mut resolved_conflicts = 0;
        let mut flagged_for_review = 0;
        
        for conflict in conflicts {
            *conflicts_by_type.entry(conflict.conflict_type.clone()).or_insert(0) += 1;
            *conflicts_by_severity.entry(conflict.severity.clone()).or_insert(0) += 1;
            
            if matches!(conflict.severity, ConflictSeverity::Critical) {
                critical_conflicts += 1;
            }
            
            match conflict.resolution_strategy {
                ResolutionStrategy::PreferCode | ResolutionStrategy::PreferDocumentation | ResolutionStrategy::Merge => {
                    resolved_conflicts += 1;
                },
                ResolutionStrategy::FlagAsInconsistent => {
                    flagged_for_review += 1;
                },
            }
        }
        
        // Calculate overall consistency score
        let total_conflicts = conflicts.len() as f32;
        let overall_consistency_score = if total_conflicts == 0.0 {
            1.0
        } else {
            let critical_penalty = critical_conflicts as f32 * 0.3;
            let total_penalty = total_conflicts * 0.1 + critical_penalty;
            (1.0 - total_penalty).max(0.0)
        };
        
        ResolutionSummary {
            total_conflicts: conflicts.len(),
            conflicts_by_type,
            conflicts_by_severity,
            critical_conflicts,
            resolved_conflicts,
            flagged_for_review,
            overall_consistency_score,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::documentation_claims_extractor::{DocumentationClaim, ClaimType, ClaimPriority};
    use crate::core::code_reality_analyzer::{ImplementationEvidence, RealityType, ImplementationLevel};
    use std::path::PathBuf;
    
    #[test]
    fn test_conflict_resolution_engine_creation() {
        let engine = ConflictResolutionEngine::new();
        assert!(!engine.config.claim_type_mappings.is_empty());
    }
    
    #[test]
    fn test_claims_match_implementation() {
        let engine = ConflictResolutionEngine::new();
        
        let claim = DocumentationClaim {
            claim_type: ClaimType::Api,
            description: "REST API endpoints".to_string(),
            source_file: PathBuf::from("README.md"),
            line_number: 1,
            confidence: 0.8,
            priority: ClaimPriority::High,
            keywords: vec!["api".to_string()],
            context: "API section".to_string(),
            evidence: vec!["Supports REST API".to_string()],
        };
        
        let implementation = ImplementationEvidence {
            reality_type: RealityType::ApiEndpointsImplemented,
            description: "API endpoints implementation".to_string(),
            source_files: vec![PathBuf::from("src/api.js")],
            line_numbers: vec![10],
            confidence: 0.9,
            implementation_level: ImplementationLevel::Complete,
            code_snippets: vec!["app.get('/api/users')".to_string()],
            dependencies: vec!["express".to_string()],
            patterns_matched: vec!["api_pattern".to_string()],
        };
        
        assert!(engine.claims_match_implementation(&claim, &implementation));
    }
    
    #[test]
    fn test_conflict_resolution() {
        let engine = ConflictResolutionEngine::new();
        
        // Create a claim without matching implementation
        let claim = DocumentationClaim {
            claim_type: ClaimType::Security,
            description: "OAuth authentication".to_string(),
            source_file: PathBuf::from("README.md"),
            line_number: 5,
            confidence: 0.9,
            priority: ClaimPriority::Critical,
            keywords: vec!["oauth".to_string()],
            context: "Security features".to_string(),
            evidence: vec!["Supports OAuth".to_string()],
        };
        
        // Create implementation without matching claim
        let implementation = ImplementationEvidence {
            reality_type: RealityType::TestingImplemented,
            description: "Unit tests".to_string(),
            source_files: vec![PathBuf::from("src/test.js")],
            line_numbers: vec![1],
            confidence: 0.8,
            implementation_level: ImplementationLevel::Complete,
            code_snippets: vec!["test('should work')".to_string()],
            dependencies: vec!["jest".to_string()],
            patterns_matched: vec!["test_pattern".to_string()],
        };
        
        let result = engine.resolve_conflicts(&[claim], &[implementation]).unwrap();
        
        assert_eq!(result.conflicts.len(), 2); // One for each unmatched item
        assert!(result.conflicts.iter().any(|c| matches!(c.conflict_type, ConflictType::ClaimedButNotImplemented)));
        assert!(result.conflicts.iter().any(|c| matches!(c.conflict_type, ConflictType::ImplementedButNotClaimed)));
    }
    
    #[test]
    fn test_semantic_similarity() {
        let engine = ConflictResolutionEngine::new();
        
        let similarity1 = engine.calculate_semantic_similarity("OAuth authentication", "authentication with OAuth");
        assert!(similarity1 > 0.5);
        
        let similarity2 = engine.calculate_semantic_similarity("REST API", "database connection");
        assert!(similarity2 < 0.3);
    }
}