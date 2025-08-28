use super::{DocumentGenerator, DocumentType};
use crate::core::CodebaseAnalysis;
use crate::intelligence::IntelligentAnalysis;
use anyhow::Result;

pub struct CCPMImportGenerator;

impl CCPMImportGenerator {
    pub fn new() -> Self {
        Self
    }

    fn calculate_confidence_level(analysis: &CodebaseAnalysis) -> &str {
        let framework_confidence = analysis.framework_analysis.confidence_scores.values()
            .fold(0.0_f32, |acc, &x| acc.max(x));
        let business_confidence = analysis.business_context.confidence;
        let overall_confidence = (framework_confidence + business_confidence) / 2.0;

        if overall_confidence >= 0.9 {
            "HIGH"
        } else if overall_confidence >= 0.7 {
            "MEDIUM"
        } else {
            "LOW"
        }
    }

    fn format_user_story_for_ccpm(story: &crate::core::UserStory, analysis: &CodebaseAnalysis) -> String {
        let mut formatted = String::new();
        
        formatted.push_str(&format!("### {} - {}\n", story.id, story.title));
        formatted.push_str(&format!("**Status**: {:?} | **Priority**: {:?} | **Complexity**: {:?}\n\n", 
            story.status, story.priority, story.complexity));
        
        formatted.push_str(&format!("**Description**: {}\n\n", story.description));
        
        if !story.acceptance_criteria.is_empty() {
            formatted.push_str("**Acceptance Criteria**:\n");
            for (i, criteria) in story.acceptance_criteria.iter().enumerate() {
                formatted.push_str(&format!("{}. {}\n", i + 1, criteria));
            }
            formatted.push_str("\n");
        }
        
        if !story.inferred_from.is_empty() {
            formatted.push_str("**Evidence**: ");
            formatted.push_str(&story.inferred_from.join(", "));
            formatted.push_str("\n\n");
        }

        // Calculate business value based on priority and complexity
        let business_value = match (story.priority.clone(), story.complexity.clone()) {
            (crate::core::Priority::Critical, _) => "CRITICAL",
            (crate::core::Priority::High, crate::core::Complexity::Simple) => "HIGH",
            (crate::core::Priority::High, _) => "MEDIUM-HIGH",
            (crate::core::Priority::Medium, crate::core::Complexity::Simple) => "MEDIUM",
            (crate::core::Priority::Medium, _) => "MEDIUM-LOW",
            _ => "LOW"
        };
        
        formatted.push_str(&format!("**Business Value**: {}\n", business_value));
        
        // Add related components context
        if !story.related_components.is_empty() {
            formatted.push_str(&format!("**Related Components**: {}\n", 
                story.related_components.join(", ")));
        }
        
        formatted.push_str("\n---\n\n");
        formatted
    }

    fn generate_enhancement_roadmap(analysis: &CodebaseAnalysis, intelligent_analysis: Option<&IntelligentAnalysis>) -> String {
        let mut roadmap = String::new();
        
        roadmap.push_str("### Phase 1: Immediate Actions (0-30 days)\n\n");
        
        // Technical debt items
        if !analysis.status_intelligence.technical_debt.is_empty() {
            for debt in analysis.status_intelligence.technical_debt.iter().take(3) {
                roadmap.push_str(&format!("- **{}**: {}\n", debt.severity, debt.description));
            }
        }
        
        // Incomplete features
        for feature in analysis.status_intelligence.todo_features.iter().take(2) {
            roadmap.push_str(&format!("- **Complete Feature**: {}\n", feature.name));
        }
        
        roadmap.push_str("\n### Phase 2: Core Development (30-90 days)\n\n");
        
        // In-progress features
        for feature in analysis.status_intelligence.in_progress_features.iter() {
            roadmap.push_str(&format!("- **Finish Implementation**: {}\n", feature.name));
        }
        
        // High-priority user stories
        let high_priority_stories: Vec<_> = analysis.user_stories.iter()
            .filter(|s| matches!(s.priority, crate::core::Priority::High | crate::core::Priority::Critical))
            .take(3)
            .collect();
            
        for story in high_priority_stories {
            roadmap.push_str(&format!("- **New Feature**: {}\n", story.title));
        }
        
        roadmap.push_str("\n### Phase 3: Enhancement & Scale (90+ days)\n\n");
        
        // Architecture improvements
        if let Some(intel) = intelligent_analysis {
            if intel.quality_metrics.technical_debt_score > 0.6 {
                roadmap.push_str("- **Architecture Refactoring**: Address technical debt and improve maintainability\n");
            }
            if intel.quality_metrics.test_coverage_estimate < 0.7 {
                roadmap.push_str("- **Testing Enhancement**: Improve test coverage and quality assurance\n");
            }
            if intel.quality_metrics.complexity > 0.8 {
                roadmap.push_str("- **Complexity Reduction**: Simplify complex components and improve code clarity\n");
            }
        }
        
        // Integration opportunities
        if analysis.integration_points.external_services.is_empty() {
            roadmap.push_str("- **Integration Expansion**: Add external service integrations for enhanced functionality\n");
        }
        
        roadmap.push_str("- **Performance Optimization**: Implement caching, monitoring, and scalability improvements\n");
        roadmap.push_str("- **Documentation & Knowledge Transfer**: Comprehensive documentation and team onboarding\n");
        
        roadmap
    }
}

impl DocumentGenerator for CCPMImportGenerator {
    fn generate(&self, analysis: &CodebaseAnalysis, intelligent_analysis: Option<&IntelligentAnalysis>) -> Result<String> {
        let mut content = String::new();
        
        // Header
        content.push_str("# CCMP Import: Reverse Engineered Requirements\n\n");
        
        // Import metadata section
        content.push_str("## IMPORT_METADATA\n\n");
        content.push_str(&format!("- **Analyzer**: codebase-workflow-analyzer v{}\n", 
            analysis.analysis_metadata.analyzer_version));
        content.push_str(&format!("- **Analysis Date**: {}\n", 
            analysis.analysis_metadata.analyzed_at));
        content.push_str(&format!("- **Confidence**: {} based on framework detection accuracy\n", 
            Self::calculate_confidence_level(analysis)));
        content.push_str(&format!("- **Project Type**: {}\n", 
            analysis.business_context.inferred_product_type));
        content.push_str(&format!("- **Architecture**: {}\n", 
            analysis.framework_analysis.architecture_pattern));
        
        // Add framework details
        if !analysis.framework_analysis.detected_frameworks.is_empty() {
            content.push_str("- **Detected Frameworks**:\n");
            for framework in &analysis.framework_analysis.detected_frameworks {
                content.push_str(&format!("  - {} v{} ({}% confidence)\n", 
                    framework.name,
                    framework.version.as_deref().unwrap_or("latest"),
                    (framework.confidence * 100.0) as u8));
            }
        }
        
        content.push_str(&format!("- **Implementation Status**: {:.1}% complete\n", 
            analysis.status_intelligence.overall_completion_percentage));
        content.push_str(&format!("- **Components Analyzed**: {}\n", analysis.components.len()));
        content.push_str(&format!("- **User Stories Discovered**: {}\n\n", analysis.user_stories.len()));
        
        // Discovered user stories section
        content.push_str("## DISCOVERED_USER_STORIES\n\n");
        
        if analysis.user_stories.is_empty() {
            content.push_str("*No user stories were automatically inferred from the codebase.*\n\n");
        } else {
            content.push_str(&format!("**Total Stories**: {} | **By Priority**: Critical: {}, High: {}, Medium: {}, Low: {}\n\n",
                analysis.user_stories.len(),
                analysis.user_stories.iter().filter(|s| matches!(s.priority, crate::core::Priority::Critical)).count(),
                analysis.user_stories.iter().filter(|s| matches!(s.priority, crate::core::Priority::High)).count(),
                analysis.user_stories.iter().filter(|s| matches!(s.priority, crate::core::Priority::Medium)).count(),
                analysis.user_stories.iter().filter(|s| matches!(s.priority, crate::core::Priority::Low)).count()
            ));
            
            // Group stories by status
            let mut complete_stories = Vec::new();
            let mut in_progress_stories = Vec::new();
            let mut todo_stories = Vec::new();
            
            for story in &analysis.user_stories {
                match story.status {
                    crate::core::ImplementationStatus::Complete => complete_stories.push(story),
                    crate::core::ImplementationStatus::InProgress => in_progress_stories.push(story),
                    _ => todo_stories.push(story),
                }
            }
            
            if !complete_stories.is_empty() {
                content.push_str("### Implemented Features\n\n");
                for story in complete_stories.iter().take(10) {
                    content.push_str(&Self::format_user_story_for_ccpm(story, analysis));
                }
            }
            
            if !in_progress_stories.is_empty() {
                content.push_str("### Features In Development\n\n");
                for story in in_progress_stories.iter().take(10) {
                    content.push_str(&Self::format_user_story_for_ccpm(story, analysis));
                }
            }
            
            if !todo_stories.is_empty() {
                content.push_str("### Features To Be Implemented\n\n");
                for story in todo_stories.iter().take(10) {
                    content.push_str(&Self::format_user_story_for_ccpm(story, analysis));
                }
            }
        }
        
        // Technical context section
        content.push_str("## TECHNICAL_CONTEXT\n\n");
        content.push_str("### Current Architecture\n\n");
        content.push_str(&format!("**Pattern**: {}\n\n", analysis.framework_analysis.architecture_pattern));
        
        if !analysis.framework_analysis.detected_frameworks.is_empty() {
            content.push_str("**Technology Stack**:\n");
            for framework in &analysis.framework_analysis.detected_frameworks {
                content.push_str(&format!("- **{}**: {}\n", 
                    framework.name, 
                    match framework.usage_extent {
                        crate::core::UsageExtent::Core => "Core framework driving the application architecture",
                        crate::core::UsageExtent::Extensive => "Extensively used throughout the application",
                        crate::core::UsageExtent::Moderate => "Moderately integrated into several components",
                        crate::core::UsageExtent::Limited => "Limited usage in specific areas",
                    }));
            }
            content.push_str("\n");
        }
        
        // Component breakdown
        if !analysis.components.is_empty() {
            content.push_str("**Component Analysis**:\n");
            let mut component_types = std::collections::HashMap::new();
            for component in &analysis.components {
                *component_types.entry(format!("{:?}", component.component_type)).or_insert(0) += 1;
            }
            
            for (comp_type, count) in component_types {
                content.push_str(&format!("- {}: {} components\n", comp_type, count));
            }
            content.push_str("\n");
        }
        
        // Integration points
        if !analysis.integration_points.external_services.is_empty() 
            || !analysis.integration_points.configuration_files.is_empty() {
            content.push_str("**Integration Points**:\n");
            
            for service in &analysis.integration_points.external_services {
                content.push_str(&format!("- External Service: {} ({})\n", 
                    service.name, service.service_type));
            }
            
            for config in analysis.integration_points.configuration_files.iter().take(5) {
                content.push_str(&format!("- Configuration: {} ({})\n", 
                    config.file_path, config.file_type));
            }
            content.push_str("\n");
        }
        
        // Current constraints and opportunities
        content.push_str("### Constraints & Opportunities\n\n");
        
        if !analysis.status_intelligence.technical_debt.is_empty() {
            content.push_str("**Technical Constraints**:\n");
            for debt in analysis.status_intelligence.technical_debt.iter().take(5) {
                content.push_str(&format!("- {} ({}): {}\n", 
                    debt.severity, debt.location, debt.description));
            }
            content.push_str("\n");
        }
        
        content.push_str("**Enhancement Opportunities**:\n");
        if analysis.status_intelligence.overall_completion_percentage < 80.0 {
            content.push_str(&format!("- Feature Completion: {:.1}% of identified features need completion\n", 
                100.0 - analysis.status_intelligence.overall_completion_percentage));
        }
        
        if let Some(intel) = intelligent_analysis {
            if intel.quality_metrics.maintainability < 0.8 {
                content.push_str("- Code Maintainability: Opportunities for refactoring and code quality improvement\n");
            }
            if intel.quality_metrics.test_coverage_estimate < 0.7 {
                content.push_str("- Test Coverage: Expansion of automated testing and quality assurance\n");
            }
        }
        
        if analysis.business_context.primary_user_personas.len() < 3 {
            content.push_str("- User Experience: Opportunity to expand user persona coverage and journey optimization\n");
        }
        
        content.push_str("\n");
        
        // Enhancement roadmap section
        content.push_str("## ENHANCEMENT_ROADMAP\n\n");
        content.push_str(&Self::generate_enhancement_roadmap(analysis, intelligent_analysis));
        
        // Business context summary
        content.push_str("\n## BUSINESS_CONTEXT_SUMMARY\n\n");
        content.push_str(&format!("**Inferred Product Type**: {} ({:.1}% confidence)\n\n", 
            analysis.business_context.inferred_product_type, 
            analysis.business_context.confidence * 100.0));
        
        if !analysis.business_context.primary_user_personas.is_empty() {
            content.push_str("**Target User Personas**:\n");
            for persona in &analysis.business_context.primary_user_personas {
                content.push_str(&format!("- {}\n", persona));
            }
            content.push_str("\n");
        }
        
        if !analysis.business_context.user_journeys_discovered.is_empty() {
            content.push_str("**Key User Journeys**:\n");
            for journey in &analysis.business_context.user_journeys_discovered {
                content.push_str(&format!("- {}\n", journey));
            }
            content.push_str("\n");
        }
        
        // Footer
        content.push_str("---\n\n");
        content.push_str("**Import Instructions**: This document contains reverse-engineered requirements ");
        content.push_str("suitable for import into CCPM systems. All user stories, technical context, and ");
        content.push_str("roadmap items have been automatically inferred from codebase analysis with ");
        content.push_str(&format!("{} confidence level.\n\n", Self::calculate_confidence_level(analysis)));
        
        content.push_str("**Next Steps**: Review and validate the discovered requirements, adjust priorities ");
        content.push_str("based on business objectives, and integrate into your project management workflow.\n");
        
        Ok(content)
    }

    fn get_file_extension(&self) -> &str {
        "md"
    }

    fn get_document_type(&self) -> DocumentType {
        DocumentType::CCPMImport
    }
}