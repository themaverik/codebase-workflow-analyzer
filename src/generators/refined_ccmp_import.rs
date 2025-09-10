use crate::core::refinement_structures::{
    RefinedAnalysisResult, BusinessPriority, FeatureStatus, BusinessValueRating,
    ValidatedPersona, RefinedUserStory,
};
use anyhow::Result;

pub struct RefinedCCMPImportGenerator;

impl RefinedCCMPImportGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(refined_analysis: &RefinedAnalysisResult) -> Result<String> {
        let mut content = String::new();
        
        // Header with refinement context
        content.push_str("# CCMP Import: Reverse Engineered Product Requirements\n\n");
        
        Self::add_import_metadata(&mut content, refined_analysis);
        Self::add_discovered_user_stories(&mut content, refined_analysis);
        Self::add_business_context(&mut content, refined_analysis);
        Self::add_technical_constraints(&mut content, refined_analysis);
        Self::add_enhancement_opportunities(&mut content, refined_analysis);
        Self::add_footer(&mut content, refined_analysis);
        
        Ok(content)
    }

    fn add_import_metadata(content: &mut String, analysis: &RefinedAnalysisResult) {
        content.push_str("## IMPORT_METADATA\n");
        content.push_str(&format!("- Source: codebase-workflow-analyzer v{}\n", 
            analysis.metadata.analyzer_version));
        content.push_str(&format!("- Analysis Date: {}\n", 
            analysis.metadata.analysis_date.format("%Y-%m-%d")));
        
        if analysis.metadata.refinement_stakeholders.len() > 0 {
            content.push_str(&format!("- Refinement Quality: {} ({}% confidence improvement)\n", 
                if analysis.metadata.confidence_improvement.overall_improvement > 0.3 { "HIGH" } 
                else if analysis.metadata.confidence_improvement.overall_improvement > 0.15 { "MEDIUM" } 
                else { "LOW" },
                (analysis.metadata.confidence_improvement.overall_improvement * 100.0) as u8));
            content.push_str(&format!("- Stakeholders: {}\n", 
                analysis.metadata.refinement_stakeholders.iter()
                    .map(|s| format!("{:?}", s).replace("_", " "))
                    .collect::<Vec<_>>()
                    .join(", ")));
        } else {
            content.push_str("- Refinement Quality: AUTOMATED (no human validation)\n");
        }
        
        content.push_str(&format!("- Integration Readiness: {}\n", 
            if analysis.integration_readiness.ccmp_import_ready { "READY" } else { "NEEDS_VALIDATION" }));
        
        content.push_str("\n");
    }

    fn add_discovered_user_stories(content: &mut String, analysis: &RefinedAnalysisResult) {
        content.push_str("## DISCOVERED_USER_STORIES\n");
        
        let stories = &analysis.user_stories.stories;
        if stories.is_empty() {
            content.push_str("*No user stories were discovered or validated.*\n\n");
            return;
        }

        // Group by status
        let completed_stories: Vec<_> = stories.iter()
            .filter(|s| matches!(s.status, FeatureStatus::Complete))
            .collect();
        let in_progress_stories: Vec<_> = stories.iter()
            .filter(|s| matches!(s.status, FeatureStatus::InProgress))
            .collect();
        let todo_stories: Vec<_> = stories.iter()
            .filter(|s| matches!(s.status, FeatureStatus::Todo))
            .collect();
        let new_stories: Vec<_> = stories.iter()
            .filter(|s| matches!(s.status, FeatureStatus::New))
            .collect();

        if !completed_stories.is_empty() {
            content.push_str("### Completed Functionality ✅\n");
            for story in completed_stories.iter().take(10) {
                Self::format_refined_user_story(content, story, "IMPLEMENTED");
            }
            content.push_str("\n");
        }

        if !in_progress_stories.is_empty() {
            content.push_str("### Partial Implementation 🔄\n");
            for story in in_progress_stories.iter().take(10) {
                Self::format_refined_user_story(content, story, "IN_PROGRESS");
            }
            content.push_str("\n");
        }

        if !todo_stories.is_empty() {
            content.push_str("### Identified Features (Not Started) 📝\n");
            for story in todo_stories.iter().take(10) {
                Self::format_refined_user_story(content, story, "TODO");
            }
            content.push_str("\n");
        }

        if !new_stories.is_empty() {
            content.push_str("### Missing Features (New) 🆕\n");
            for story in new_stories.iter().take(10) {
                Self::format_refined_user_story(content, story, "NEW");
            }
            content.push_str("\n");
        }
    }

    fn format_refined_user_story(content: &mut String, story: &RefinedUserStory, status_label: &str) {
        content.push_str(&format!("**{} - {}**\n", story.id, story.title));
        content.push_str(&format!("- Status: {} | Business Value: {} | Persona: {}\n", 
            status_label, story.business_value, story.validated_persona.role));
        content.push_str(&format!("- Description: {}\n", story.description));
        
        if !story.evidence.is_empty() {
            content.push_str(&format!("- Evidence: {}\n", story.evidence.join(", ")));
        }
        
        if let Some(gap) = &story.implementation_gap {
            content.push_str(&format!("- Implementation Gap: {}\n", gap));
        }
        
        if !story.acceptance_criteria.is_empty() {
            content.push_str("- Acceptance Criteria:\n");
            for criteria in &story.acceptance_criteria {
                content.push_str(&format!("  - {}\n", criteria));
            }
        }
        
        content.push_str("\n");
    }

    fn add_business_context(content: &mut String, analysis: &RefinedAnalysisResult) {
        content.push_str("## BUSINESS_CONTEXT\n");
        
        content.push_str("### Product Type\n");
        content.push_str(&format!("**{}**\n", analysis.business_intelligence.validated_product_type));
        content.push_str(&format!("- Industry: {}\n", analysis.business_intelligence.validated_industry));
        content.push_str(&format!("- Target Market: {}\n", analysis.business_intelligence.validated_target_market));
        content.push_str(&format!("- Business Model: {}\n", analysis.business_intelligence.validated_business_model));
        
        if !analysis.business_intelligence.strategic_context.is_empty() {
            content.push_str(&format!("- Strategic Context: {}\n", analysis.business_intelligence.strategic_context));
        }
        
        content.push_str("\n");

        content.push_str("### Primary User Personas\n");
        if analysis.business_intelligence.validated_personas.is_empty() {
            content.push_str("*No validated personas available.*\n\n");
        } else {
            for persona in &analysis.business_intelligence.validated_personas {
                Self::format_validated_persona(content, persona);
            }
            content.push_str("\n");
        }

        content.push_str("### Business Metrics & Success Criteria\n");
        if analysis.business_intelligence.business_metrics.is_empty() {
            content.push_str("*No business metrics defined.*\n\n");
        } else {
            for metric in &analysis.business_intelligence.business_metrics {
                content.push_str(&format!("- **{}**: {} (Target: {})\n", 
                    metric.name, metric.description, metric.target_value));
                if let Some(current) = &metric.current_value {
                    content.push_str(&format!("  Current: {}\n", current));
                }
            }
            content.push_str("\n");
        }
    }

    fn format_validated_persona(content: &mut String, persona: &ValidatedPersona) {
        content.push_str(&format!("**{}**\n", persona.role));
        content.push_str(&format!("- Description: {}\n", persona.description));
        content.push_str(&format!("- Context: {}\n", persona.context));
        content.push_str("- Primary Goals:\n");
        for goal in &persona.primary_goals {
            content.push_str(&format!("  - {}\n", goal));
        }
        content.push_str(&format!("- Business Priority: {:?}\n", persona.business_priority));
        content.push_str("\n");
    }

    fn add_technical_constraints(content: &mut String, analysis: &RefinedAnalysisResult) {
        content.push_str("## TECHNICAL_CONSTRAINTS\n");
        
        content.push_str("### Current Architecture\n");
        if analysis.technical_context.architecture_patterns.is_empty() {
            content.push_str("*Architecture patterns not identified.*\n");
        } else {
            for pattern in &analysis.technical_context.architecture_patterns {
                content.push_str(&format!("- **{}**: {}\n", pattern.pattern_name, pattern.description));
            }
        }
        content.push_str("\n");

        content.push_str("### Code Quality & Technical Debt\n");
        if analysis.feature_status_intelligence.technical_debt_items.is_empty() {
            content.push_str("*No significant technical debt identified.*\n");
        } else {
            for debt in analysis.feature_status_intelligence.technical_debt_items.iter().take(5) {
                content.push_str(&format!("- **{:?}**: {} ({})\n", 
                    debt.severity, debt.description, debt.location));
                if !debt.business_impact.is_empty() {
                    content.push_str(&format!("  Business Impact: {}\n", debt.business_impact));
                }
            }
        }
        content.push_str("\n");

        content.push_str("### Integration Dependencies\n");
        if analysis.technical_context.integration_points.is_empty() {
            content.push_str("*No external integrations detected.*\n");
        } else {
            for integration in &analysis.technical_context.integration_points {
                content.push_str(&format!("- **{}**: {} ({})\n", 
                    integration.service_name, integration.integration_type, integration.current_implementation));
            }
        }
        content.push_str("\n");
    }

    fn add_enhancement_opportunities(content: &mut String, analysis: &RefinedAnalysisResult) {
        content.push_str("## ENHANCEMENT_OPPORTUNITIES\n");
        
        content.push_str("### High-Value Features (Missing)\n");
        let high_value_new: Vec<_> = analysis.feature_status_intelligence.new_features_needed.iter()
            .filter(|f| matches!(f.priority, BusinessPriority::Critical | BusinessPriority::High))
            .collect();
            
        if high_value_new.is_empty() {
            content.push_str("*All high-value features appear to be implemented.*\n");
        } else {
            for feature in high_value_new.iter().take(5) {
                content.push_str(&format!("- **{}**: {} (Justification: {})\n", 
                    feature.name, feature.description, feature.business_justification));
            }
        }
        content.push_str("\n");

        content.push_str("### Technical Debt Priorities\n");
        let critical_debt: Vec<_> = analysis.feature_status_intelligence.technical_debt_items.iter()
            .filter(|d| matches!(d.severity, BusinessPriority::Critical | BusinessPriority::High))
            .collect();
            
        if critical_debt.is_empty() {
            content.push_str("*No critical technical debt identified.*\n");
        } else {
            for debt in critical_debt.iter().take(5) {
                content.push_str(&format!("- **{}**: {}\n", debt.description, debt.recommended_resolution));
            }
        }
        content.push_str("\n");

        content.push_str("### Scalability Improvements\n");
        content.push_str("- Performance monitoring and optimization based on current usage patterns\n");
        content.push_str("- Architecture review for scaling bottlenecks\n");
        content.push_str("- Integration enhancement for external service reliability\n");
        content.push_str("\n");
    }

    fn add_footer(content: &mut String, analysis: &RefinedAnalysisResult) {
        content.push_str("---\n");
        content.push_str("**Ready for CCMP import and business formalization**\n\n");
        
        if analysis.integration_readiness.ccmp_import_ready {
            content.push_str("✅ This document has been validated and is ready for CCMP workflow integration.\n");
        } else {
            content.push_str("⚠️ This document may require additional validation before CCMP integration.\n");
        }
        
        if analysis.metadata.refinement_stakeholders.len() > 0 {
            content.push_str(&format!("\n**Validation Level**: Human-refined with {}% confidence improvement through stakeholder input.\n",
                (analysis.metadata.confidence_improvement.overall_improvement * 100.0) as u8));
        } else {
            content.push_str("\n**Validation Level**: Automated analysis - consider human refinement for higher accuracy.\n");
        }
    }
}

