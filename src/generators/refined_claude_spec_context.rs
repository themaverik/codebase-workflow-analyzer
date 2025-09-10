use crate::core::refinement_structures::{
    RefinedAnalysisResult, TechnicalContext, CodePattern, StyleRequirement, TestingStrategy,
    FeatureStatusIntelligence, CompletedFeature, InProgressFeature, TodoFeature, NewFeature,
};
use anyhow::Result;

pub struct RefinedClaudeSpecContextGenerator;

impl RefinedClaudeSpecContextGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(refined_analysis: &RefinedAnalysisResult) -> Result<String> {
        let mut content = String::new();
        
        // Header with enhanced context information
        content.push_str("# Claude Code Spec Import: Enhanced Development Context\n\n");
        
        Self::add_codebase_context(&mut content, refined_analysis);
        Self::add_implementation_guidance(&mut content, refined_analysis);
        Self::add_feature_status_intelligence(&mut content, refined_analysis);
        Self::add_existing_codebase_patterns(&mut content, refined_analysis);
        Self::add_technical_debt_context(&mut content, refined_analysis);
        Self::add_footer(&mut content, refined_analysis);
        
        Ok(content)
    }

    fn add_codebase_context(content: &mut String, analysis: &RefinedAnalysisResult) {
        content.push_str("## CODEBASE_CONTEXT\n");
        
        content.push_str("### Current Implementation Status\n");
        
        // Extract framework information from original analysis if available
        let framework_stack = Self::extract_framework_stack(&analysis.original_analysis);
        content.push_str(&format!("- **Framework Stack:** {}\n", framework_stack));
        
        let architecture_pattern = Self::extract_architecture_pattern(&analysis.technical_context);
        content.push_str(&format!("- **Architecture Pattern:** {}\n", architecture_pattern));
        
        // Quality metrics from refined analysis
        let quality_score = Self::calculate_quality_score(&analysis.feature_status_intelligence);
        content.push_str(&format!("- **Code Quality:** {}% completion score, {} components analyzed\n", 
            (quality_score * 100.0) as u8,
            Self::count_total_components(&analysis.feature_status_intelligence)));
            
        let development_stage = Self::determine_development_stage(&analysis.feature_status_intelligence);
        content.push_str(&format!("- **Development Stage:** {}\n", development_stage));
        
        content.push_str("\n");
    }

    fn add_implementation_guidance(content: &mut String, analysis: &RefinedAnalysisResult) {
        content.push_str("## IMPLEMENTATION_GUIDANCE\n");
        
        content.push_str("### Code Style Requirements\n");
        if analysis.technical_context.code_style_requirements.is_empty() {
            content.push_str("*Follow existing codebase patterns and conventions.*\n");
        } else {
            for requirement in &analysis.technical_context.code_style_requirements {
                content.push_str(&format!("- **{}**: {} ({})\n", 
                    requirement.language, requirement.requirement, requirement.enforcement_level));
                if !requirement.examples.is_empty() {
                    content.push_str("  Examples:\n");
                    for example in &requirement.examples {
                        content.push_str(&format!("  ```{}\n  {}\n  ```\n", 
                            requirement.language.to_lowercase(), example));
                    }
                }
            }
        }
        content.push_str("\n");

        content.push_str("### Integration Patterns\n");
        if analysis.technical_context.integration_points.is_empty() {
            content.push_str("*No specific integration patterns identified.*\n");
        } else {
            for integration in &analysis.technical_context.integration_points {
                content.push_str(&format!("- **{}**: {}\n", 
                    integration.service_name, integration.current_implementation));
                if !integration.constraints.is_empty() {
                    content.push_str("  Constraints:\n");
                    for constraint in &integration.constraints {
                        content.push_str(&format!("  - {}\n", constraint));
                    }
                }
            }
        }
        content.push_str("\n");

        content.push_str("### Testing Strategy\n");
        let testing = &analysis.technical_context.testing_strategy;
        content.push_str(&format!("- **Framework:** {}\n", testing.framework));
        content.push_str(&format!("- **Coverage Requirements:** {}\n", testing.coverage_requirements));
        if !testing.test_types.is_empty() {
            content.push_str("- **Test Types:**\n");
            for test_type in &testing.test_types {
                content.push_str(&format!("  - {}\n", test_type));
            }
        }
        if !testing.quality_gates.is_empty() {
            content.push_str("- **Quality Gates:**\n");
            for gate in &testing.quality_gates {
                content.push_str(&format!("  - {}\n", gate));
            }
        }
        content.push_str("\n");
    }

    fn add_feature_status_intelligence(content: &mut String, analysis: &RefinedAnalysisResult) {
        content.push_str("## FEATURE_STATUS_INTELLIGENCE\n");
        
        let status = &analysis.feature_status_intelligence;
        
        content.push_str("### Completed Components\n");
        if status.completed_features.is_empty() {
            content.push_str("*No completed components identified.*\n");
        } else {
            for feature in status.completed_features.iter().take(10) {
                Self::format_completed_feature(content, feature);
            }
        }
        content.push_str("\n");

        content.push_str("### In-Progress Components\n");
        if status.in_progress_features.is_empty() {
            content.push_str("*No components currently in development.*\n");
        } else {
            for feature in status.in_progress_features.iter().take(10) {
                Self::format_in_progress_feature(content, feature);
            }
        }
        content.push_str("\n");

        content.push_str("### TODO Components\n");
        if status.todo_features.is_empty() {
            content.push_str("*No pending components identified.*\n");
        } else {
            for feature in status.todo_features.iter().take(10) {
                Self::format_todo_feature(content, feature);
            }
        }
        content.push_str("\n");

        content.push_str("### New Components Needed\n");
        if status.new_features_needed.is_empty() {
            content.push_str("*No new components identified for development.*\n");
        } else {
            for feature in status.new_features_needed.iter().take(10) {
                Self::format_new_feature(content, feature);
            }
        }
        content.push_str("\n");
    }

    fn add_existing_codebase_patterns(content: &mut String, analysis: &RefinedAnalysisResult) {
        content.push_str("## EXISTING_CODEBASE_PATTERNS\n");
        
        if analysis.technical_context.existing_patterns.is_empty() {
            content.push_str("*No specific code patterns documented.*\n\n");
            return;
        }

        // Group patterns by type
        let mut frontend_patterns = Vec::new();
        let mut backend_patterns = Vec::new();
        let mut database_patterns = Vec::new();
        let mut generic_patterns = Vec::new();

        for pattern in &analysis.technical_context.existing_patterns {
            match pattern.pattern_type.to_lowercase().as_str() {
                t if t.contains("frontend") || t.contains("component") || t.contains("ui") => {
                    frontend_patterns.push(pattern);
                },
                t if t.contains("backend") || t.contains("api") || t.contains("service") => {
                    backend_patterns.push(pattern);
                },
                t if t.contains("database") || t.contains("model") || t.contains("entity") => {
                    database_patterns.push(pattern);
                },
                _ => generic_patterns.push(pattern),
            }
        }

        if !frontend_patterns.is_empty() {
            content.push_str("### Frontend Patterns\n");
            for pattern in frontend_patterns {
                Self::format_code_pattern(content, pattern);
            }
            content.push_str("\n");
        }

        if !backend_patterns.is_empty() {
            content.push_str("### Backend Patterns\n");
            for pattern in backend_patterns {
                Self::format_code_pattern(content, pattern);
            }
            content.push_str("\n");
        }

        if !database_patterns.is_empty() {
            content.push_str("### Database Patterns\n");
            for pattern in database_patterns {
                Self::format_code_pattern(content, pattern);
            }
            content.push_str("\n");
        }

        if !generic_patterns.is_empty() {
            content.push_str("### General Patterns\n");
            for pattern in generic_patterns {
                Self::format_code_pattern(content, pattern);
            }
            content.push_str("\n");
        }
    }

    fn add_technical_debt_context(content: &mut String, analysis: &RefinedAnalysisResult) {
        content.push_str("## TECHNICAL_DEBT_CONTEXT\n");
        
        let debt_items = &analysis.feature_status_intelligence.technical_debt_items;
        
        if debt_items.is_empty() {
            content.push_str("*No significant technical debt identified.*\n\n");
            return;
        }

        // Group by category
        let security_debt: Vec<_> = debt_items.iter()
            .filter(|d| d.category.to_lowercase().contains("security"))
            .collect();
        let performance_debt: Vec<_> = debt_items.iter()
            .filter(|d| d.category.to_lowercase().contains("performance"))
            .collect();
        let architecture_debt: Vec<_> = debt_items.iter()
            .filter(|d| d.category.to_lowercase().contains("architecture") || 
                      d.category.to_lowercase().contains("design"))
            .collect();

        if !security_debt.is_empty() {
            content.push_str("### Security Updates Needed\n");
            for debt in security_debt.iter().take(5) {
                content.push_str(&format!("- **{}** ({}): {}\n", 
                    debt.description, debt.location, debt.recommended_resolution));
                if !debt.business_impact.is_empty() {
                    content.push_str(&format!("  Impact: {}\n", debt.business_impact));
                }
            }
            content.push_str("\n");
        }

        if !performance_debt.is_empty() {
            content.push_str("### Performance Optimizations\n");
            for debt in performance_debt.iter().take(5) {
                content.push_str(&format!("- **{}** ({}): {}\n", 
                    debt.description, debt.location, debt.recommended_resolution));
                if !debt.business_impact.is_empty() {
                    content.push_str(&format!("  Impact: {}\n", debt.business_impact));
                }
            }
            content.push_str("\n");
        }

        if !architecture_debt.is_empty() {
            content.push_str("### Architecture Improvements\n");
            for debt in architecture_debt.iter().take(5) {
                content.push_str(&format!("- **{}** ({}): {}\n", 
                    debt.description, debt.location, debt.recommended_resolution));
                if !debt.business_impact.is_empty() {
                    content.push_str(&format!("  Impact: {}\n", debt.business_impact));
                }
            }
            content.push_str("\n");
        }
    }

    fn add_footer(content: &mut String, analysis: &RefinedAnalysisResult) {
        content.push_str("---\n");
        content.push_str("**Ready for Claude Code Spec workflow context-aware development**\n\n");
        
        if analysis.integration_readiness.claude_spec_ready {
            content.push_str("✅ This context has been validated and is ready for Claude Code Spec Workflow integration.\n");
        } else {
            content.push_str("⚠️ This context may require additional validation for optimal Claude Code Spec integration.\n");
        }
        
        content.push_str(&format!("\n**Context Quality Score:** {:.1}/10 based on implementation completeness and pattern identification.\n",
            Self::calculate_context_quality_score(analysis)));
        
        if analysis.metadata.refinement_stakeholders.len() > 0 {
            content.push_str(&format!("**Validation Level:** Human-refined with technical lead validation.\n"));
        } else {
            content.push_str("**Validation Level:** Automated analysis - consider technical review for enhanced context.\n");
        }
    }

    // Helper methods
    fn format_completed_feature(content: &mut String, feature: &CompletedFeature) {
        content.push_str(&format!("- **{}**: {}\n", feature.name, feature.description));
        if let Some(coverage) = feature.test_coverage {
            content.push_str(&format!("  Test Coverage: {:.1}%\n", coverage * 100.0));
        }
        if !feature.evidence.is_empty() {
            content.push_str(&format!("  Evidence: {}\n", feature.evidence.join(", ")));
        }
    }

    fn format_in_progress_feature(content: &mut String, feature: &InProgressFeature) {
        content.push_str(&format!("- **{}**: {} ({:.1}% complete)\n", 
            feature.name, feature.description, feature.completion_percentage));
        if !feature.remaining_work.is_empty() {
            content.push_str("  Remaining:\n");
            for work in &feature.remaining_work {
                content.push_str(&format!("    - {}\n", work));
            }
        }
    }

    fn format_todo_feature(content: &mut String, feature: &TodoFeature) {
        content.push_str(&format!("- **{}**: {} (Priority: {:?})\n", 
            feature.name, feature.description, feature.priority));
        if let Some(effort) = &feature.estimated_effort {
            content.push_str(&format!("  Estimated Effort: {}\n", effort));
        }
    }

    fn format_new_feature(content: &mut String, feature: &NewFeature) {
        content.push_str(&format!("- **{}**: {}\n", feature.name, feature.description));
        content.push_str(&format!("  Business Justification: {}\n", feature.business_justification));
        content.push_str(&format!("  User Impact: {}\n", feature.user_impact));
    }

    fn format_code_pattern(content: &mut String, pattern: &CodePattern) {
        content.push_str(&format!("#### {}\n", pattern.pattern_type));
        content.push_str(&format!("{}\n", pattern.description));
        if !pattern.examples.is_empty() {
            content.push_str("```\n");
            for example in &pattern.examples {
                content.push_str(&format!("{}\n", example));
            }
            content.push_str("```\n");
        }
        content.push_str(&format!("**Usage Guidelines:** {}\n\n", pattern.usage_guidelines));
    }

    // Utility methods
    fn extract_framework_stack(analysis: &crate::core::refinement_structures::AnalysisResult) -> String {
        if analysis.frameworks_detected.is_empty() {
            "Framework detection pending".to_string()
        } else {
            analysis.frameworks_detected.iter()
                .map(|f| format!("{:?}", f))
                .collect::<Vec<_>>()
                .join(", ")
        }
    }

    fn extract_architecture_pattern(context: &TechnicalContext) -> String {
        if context.architecture_patterns.is_empty() {
            "Pattern analysis pending".to_string()
        } else {
            context.architecture_patterns[0].pattern_name.clone()
        }
    }

    fn calculate_quality_score(status: &FeatureStatusIntelligence) -> f64 {
        let total_features = status.completed_features.len() + 
                           status.in_progress_features.len() + 
                           status.todo_features.len() + 
                           status.new_features_needed.len();
        
        if total_features == 0 {
            0.5 // Default score when no features identified
        } else {
            status.completed_features.len() as f64 / total_features as f64
        }
    }

    fn count_total_components(status: &FeatureStatusIntelligence) -> usize {
        status.completed_features.len() + 
        status.in_progress_features.len() + 
        status.todo_features.len() + 
        status.new_features_needed.len()
    }

    fn determine_development_stage(status: &FeatureStatusIntelligence) -> String {
        let completion_ratio = Self::calculate_quality_score(status);
        
        if completion_ratio >= 0.9 {
            "Production-ready with minor enhancements".to_string()
        } else if completion_ratio >= 0.7 {
            "Feature-complete with optimization opportunities".to_string()
        } else if completion_ratio >= 0.5 {
            "Active development with partial implementation".to_string()
        } else if completion_ratio >= 0.2 {
            "Early development stage".to_string()
        } else {
            "Initial analysis or planning stage".to_string()
        }
    }

    fn calculate_context_quality_score(analysis: &RefinedAnalysisResult) -> f64 {
        let mut score = 0.0;
        
        // Feature completeness (30%)
        let completion_score = Self::calculate_quality_score(&analysis.feature_status_intelligence);
        score += completion_score * 3.0;
        
        // Pattern identification (25%)
        let pattern_score = if analysis.technical_context.existing_patterns.len() > 0 { 2.5 } else { 0.0 };
        score += pattern_score;
        
        // Integration readiness (20%)
        let integration_score = if analysis.integration_readiness.claude_spec_ready { 2.0 } else { 1.0 };
        score += integration_score;
        
        // Human validation (15%)
        let validation_score = if analysis.metadata.refinement_stakeholders.len() > 0 { 1.5 } else { 0.5 };
        score += validation_score;
        
        // Technical debt awareness (10%)
        let debt_score = if analysis.feature_status_intelligence.technical_debt_items.len() > 0 { 1.0 } else { 0.5 };
        score += debt_score;
        
        score.min(10.0)
    }
}