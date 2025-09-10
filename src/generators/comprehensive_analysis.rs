use crate::core::{
    refinement_structures::RefinedAnalysisResult,
    CodebaseAnalysis,
};
use anyhow::Result;
use serde_json;
use serde_yaml;

pub struct ComprehensiveAnalysisGenerator;

impl ComprehensiveAnalysisGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate all 4 analysis files as specified in workflow requirements
    pub fn generate_all_analysis_files(
        analysis: &CodebaseAnalysis, 
        refined_analysis: Option<&RefinedAnalysisResult>,
        output_dir: &std::path::Path
    ) -> Result<Vec<String>> {
        let mut generated_files = Vec::new();

        // 1. analysis-report.md (Comprehensive analysis)
        let analysis_report = Self::generate_analysis_report(analysis, refined_analysis)?;
        let analysis_path = output_dir.join("analysis-report.md");
        std::fs::write(&analysis_path, analysis_report)?;
        generated_files.push(analysis_path.to_string_lossy().to_string());

        // 2. business-intelligence.yaml (Structured business data)
        let business_yaml = Self::generate_business_intelligence_yaml(analysis, refined_analysis)?;
        let business_path = output_dir.join("business-intelligence.yaml");
        std::fs::write(&business_path, business_yaml)?;
        generated_files.push(business_path.to_string_lossy().to_string());

        // 3. feature-status.json (Implementation status per feature)
        let feature_json = Self::generate_feature_status_json(analysis, refined_analysis)?;
        let feature_path = output_dir.join("feature-status.json");
        std::fs::write(&feature_path, feature_json)?;
        generated_files.push(feature_path.to_string_lossy().to_string());

        // 4. user-stories-discovered.md (Extracted + new user stories)
        let stories_md = Self::generate_user_stories_discovered(analysis, refined_analysis)?;
        let stories_path = output_dir.join("user-stories-discovered.md");
        std::fs::write(&stories_path, stories_md)?;
        generated_files.push(stories_path.to_string_lossy().to_string());

        Ok(generated_files)
    }

    fn generate_analysis_report(
        analysis: &CodebaseAnalysis, 
        refined_analysis: Option<&RefinedAnalysisResult>
    ) -> Result<String> {
        let mut content = String::new();
        
        content.push_str("# Comprehensive Codebase Analysis Report\n\n");
        
        // Metadata section
        content.push_str("## Analysis Metadata\n");
        content.push_str(&format!("- **Analyzer Version**: {}\n", analysis.analysis_metadata.analyzer_version));
        content.push_str(&format!("- **Analysis Date**: {}\n", analysis.analysis_metadata.analyzed_at));
        content.push_str(&format!("- **Project Path**: {}\n", analysis.project_name));
        content.push_str(&format!("- **Files Analyzed**: {}\n", analysis.analysis_metadata.files_analyzed));
        content.push_str(&format!("- **Lines of Code**: {}\n", analysis.analysis_metadata.lines_of_code));
        
        if let Some(refined) = refined_analysis {
            content.push_str(&format!("- **Refinement Applied**: Yes ({}% confidence improvement)\n",
                (refined.metadata.confidence_improvement.overall_improvement * 100.0) as u8));
            if !refined.metadata.refinement_stakeholders.is_empty() {
                content.push_str(&format!("- **Stakeholder Validation**: {}\n",
                    refined.metadata.refinement_stakeholders.iter()
                        .map(|s| format!("{:?}", s))
                        .collect::<Vec<_>>()
                        .join(", ")));
            }
        } else {
            content.push_str("- **Refinement Applied**: No (automated analysis only)\n");
        }
        content.push_str("\n");

        // Framework Analysis
        content.push_str("## Framework Detection Results\n");
        if analysis.framework_analysis.detected_frameworks.is_empty() {
            content.push_str("*No frameworks detected.*\n\n");
        } else {
            for framework in &analysis.framework_analysis.detected_frameworks {
                content.push_str(&format!("### {}\n", framework.name));
                content.push_str(&format!("- **Version**: {}\n", 
                    framework.version.as_deref().unwrap_or("Not specified")));
                content.push_str(&format!("- **Confidence**: {:.1}%\n", framework.confidence * 100.0));
                content.push_str(&format!("- **Usage Extent**: {:?}\n", framework.usage_extent));
                if !framework.evidence.is_empty() {
                    content.push_str("- **Evidence**:\n");
                    for evidence in &framework.evidence {
                        content.push_str(&format!("  - {}\n", evidence));
                    }
                }
                content.push_str("\n");
            }
        }

        // Business Context Analysis  
        content.push_str("## Business Context Intelligence\n");
        if let Some(refined) = refined_analysis {
            content.push_str(&format!("### Product Type: {}\n", refined.business_intelligence.validated_product_type));
            content.push_str(&format!("- **Industry**: {}\n", refined.business_intelligence.validated_industry));
            content.push_str(&format!("- **Target Market**: {}\n", refined.business_intelligence.validated_target_market));
            content.push_str(&format!("- **Business Model**: {}\n", refined.business_intelligence.validated_business_model));
            
            if !refined.business_intelligence.strategic_context.is_empty() {
                content.push_str(&format!("- **Strategic Context**: {}\n", refined.business_intelligence.strategic_context));
            }
            
            content.push_str("\n### Validated User Personas\n");
            if refined.business_intelligence.validated_personas.is_empty() {
                content.push_str("*No validated personas identified.*\n");
            } else {
                for persona in &refined.business_intelligence.validated_personas {
                    content.push_str(&format!("#### {}\n", persona.role));
                    content.push_str(&format!("- **Description**: {}\n", persona.description));
                    content.push_str(&format!("- **Context**: {}\n", persona.context));
                    content.push_str("- **Primary Goals**:\n");
                    for goal in &persona.primary_goals {
                        content.push_str(&format!("  - {}\n", goal));
                    }
                    content.push_str("\n");
                }
            }
        } else {
            content.push_str(&format!("### Inferred Product Type: {}\n", analysis.business_context.inferred_product_type));
            content.push_str(&format!("- **Confidence**: {:.1}%\n", analysis.business_context.confidence * 100.0));
            content.push_str(&format!("- **Business Domain**: {}\n", analysis.business_context.business_domain));
            
            if !analysis.business_context.primary_user_personas.is_empty() {
                content.push_str("- **Identified User Personas**:\n");
                for persona in &analysis.business_context.primary_user_personas {
                    content.push_str(&format!("  - {}\n", persona));
                }
            }
        }
        content.push_str("\n");

        // Feature Status Analysis
        content.push_str("## Implementation Status Analysis\n");
        content.push_str(&format!("- **Overall Completion**: {:.1}%\n", 
            analysis.status_intelligence.overall_completion_percentage));
        
        content.push_str(&format!("- **Completed Features**: {}\n", 
            analysis.status_intelligence.completed_features.len()));
        content.push_str(&format!("- **In-Progress Features**: {}\n", 
            analysis.status_intelligence.in_progress_features.len()));
        content.push_str(&format!("- **Todo Features**: {}\n", 
            analysis.status_intelligence.todo_features.len()));
        content.push_str(&format!("- **Technical Debt Items**: {}\n", 
            analysis.status_intelligence.technical_debt.len()));
        content.push_str("\n");

        // Component Analysis
        content.push_str("## Component Analysis\n");
        content.push_str(&format!("- **Total Components**: {}\n", analysis.components.len()));
        
        if !analysis.components.is_empty() {
            let mut component_by_type = std::collections::HashMap::new();
            for component in &analysis.components {
                *component_by_type.entry(&component.component_type).or_insert(0) += 1;
            }
            
            content.push_str("- **Component Breakdown**:\n");
            for (comp_type, count) in component_by_type {
                content.push_str(&format!("  - {:?}: {} components\n", comp_type, count));
            }
        }
        content.push_str("\n");

        // Integration Analysis
        if !analysis.integration_points.external_services.is_empty() ||
           !analysis.integration_points.internal_dependencies.is_empty() {
            content.push_str("## Integration Points\n");
            
            if !analysis.integration_points.external_services.is_empty() {
                content.push_str("### External Services\n");
                for service in &analysis.integration_points.external_services {
                    content.push_str(&format!("- **{}** ({}): {}\n", 
                        service.name, service.service_type, service.usage_context));
                }
                content.push_str("\n");
            }
            
            if !analysis.integration_points.internal_dependencies.is_empty() {
                content.push_str("### Internal Dependencies\n");
                for dep in &analysis.integration_points.internal_dependencies {
                    content.push_str(&format!("- **{}** ({})\n", dep.name, dep.dependency_type));
                }
                content.push_str("\n");
            }
        }

        // Recommendations
        content.push_str("## Analysis Recommendations\n");
        if let Some(refined) = refined_analysis {
            content.push_str("### Priority Actions (Human-Validated)\n");
            let high_priority_debt: Vec<_> = refined.feature_status_intelligence.technical_debt_items.iter()
                .filter(|d| matches!(d.severity, crate::core::refinement_structures::BusinessPriority::Critical | crate::core::refinement_structures::BusinessPriority::High))
                .collect();
                
            if !high_priority_debt.is_empty() {
                content.push_str("#### Critical Technical Debt\n");
                for debt in high_priority_debt.iter().take(5) {
                    content.push_str(&format!("- **{}**: {}\n", debt.description, debt.recommended_resolution));
                    if !debt.business_impact.is_empty() {
                        content.push_str(&format!("  - Business Impact: {}\n", debt.business_impact));
                    }
                }
                content.push_str("\n");
            }
        } else {
            content.push_str("### Automated Recommendations\n");
            if analysis.status_intelligence.overall_completion_percentage < 80.0 {
                content.push_str(&format!("- **Feature Completion**: {:.1}% of features need completion\n",
                    100.0 - analysis.status_intelligence.overall_completion_percentage));
            }
            
            if !analysis.status_intelligence.technical_debt.is_empty() {
                content.push_str("- **Technical Debt**: Address identified technical debt items\n");
                for debt in analysis.status_intelligence.technical_debt.iter().take(3) {
                    content.push_str(&format!("  - {} ({})\n", debt.description, debt.severity));
                }
            }
            content.push_str("\n");
        }

        // Footer
        content.push_str("---\n");
        content.push_str("*This comprehensive analysis provides the foundation for systematic development planning ");
        content.push_str("and can be used as input for CCMP and Claude Code Spec workflows.*\n");
        
        Ok(content)
    }

    fn generate_business_intelligence_yaml(
        analysis: &CodebaseAnalysis,
        refined_analysis: Option<&RefinedAnalysisResult>
    ) -> Result<String> {
        use serde_json::json;
        
        let business_data = if let Some(refined) = refined_analysis {
            json!({
                "metadata": {
                    "analysis_type": "human_refined",
                    "confidence_improvement": refined.metadata.confidence_improvement.overall_improvement,
                    "stakeholders": refined.metadata.refinement_stakeholders,
                    "refinement_date": refined.metadata.refinement_date
                },
                "product_intelligence": {
                    "validated_product_type": refined.business_intelligence.validated_product_type,
                    "industry": refined.business_intelligence.validated_industry,
                    "target_market": refined.business_intelligence.validated_target_market,
                    "business_model": refined.business_intelligence.validated_business_model,
                    "strategic_context": refined.business_intelligence.strategic_context,
                    "market_positioning": refined.business_intelligence.market_positioning
                },
                "user_personas": refined.business_intelligence.validated_personas.iter().map(|p| json!({
                    "role": p.role,
                    "description": p.description,
                    "context": p.context,
                    "primary_goals": p.primary_goals,
                    "business_priority": p.business_priority
                })).collect::<Vec<_>>(),
                "business_metrics": refined.business_intelligence.business_metrics.iter().map(|m| json!({
                    "name": m.name,
                    "description": m.description,
                    "current_value": m.current_value,
                    "target_value": m.target_value,
                    "measurement_method": m.measurement_method,
                    "priority": m.priority
                })).collect::<Vec<_>>(),
                "success_criteria": refined.business_intelligence.success_criteria.iter().map(|c| json!({
                    "criterion": c.criterion,
                    "measurement": c.measurement,
                    "target_value": c.target_value,
                    "priority": c.priority
                })).collect::<Vec<_>>()
            })
        } else {
            json!({
                "metadata": {
                    "analysis_type": "automated",
                    "confidence_level": analysis.business_context.confidence,
                    "analysis_date": analysis.analysis_metadata.analyzed_at
                },
                "product_intelligence": {
                    "inferred_product_type": analysis.business_context.inferred_product_type,
                    "business_domain": analysis.business_context.business_domain,
                    "confidence": analysis.business_context.confidence
                },
                "user_personas": analysis.business_context.primary_user_personas.iter().map(|p| json!({
                    "persona": p,
                    "source": "automated_inference"
                })).collect::<Vec<_>>(),
                "user_journeys": analysis.business_context.user_journeys_discovered
            })
        };
        
        serde_yaml::to_string(&business_data).map_err(|e| anyhow::anyhow!("YAML serialization error: {}", e))
    }

    fn generate_feature_status_json(
        analysis: &CodebaseAnalysis,
        refined_analysis: Option<&RefinedAnalysisResult>
    ) -> Result<String> {
        use serde_json::json;
        
        let feature_data = if let Some(refined) = refined_analysis {
            json!({
                "metadata": {
                    "analysis_type": "human_refined",
                    "total_features": refined.feature_status_intelligence.completed_features.len() +
                                   refined.feature_status_intelligence.in_progress_features.len() +
                                   refined.feature_status_intelligence.todo_features.len() +
                                   refined.feature_status_intelligence.new_features_needed.len()
                },
                "completed_features": refined.feature_status_intelligence.completed_features.iter().map(|f| json!({
                    "name": f.name,
                    "description": f.description,
                    "evidence": f.evidence,
                    "test_coverage": f.test_coverage,
                    "user_stories": f.user_stories
                })).collect::<Vec<_>>(),
                "in_progress_features": refined.feature_status_intelligence.in_progress_features.iter().map(|f| json!({
                    "name": f.name,
                    "description": f.description,
                    "completion_percentage": f.completion_percentage,
                    "blocking_issues": f.blocking_issues,
                    "remaining_work": f.remaining_work
                })).collect::<Vec<_>>(),
                "todo_features": refined.feature_status_intelligence.todo_features.iter().map(|f| json!({
                    "name": f.name,
                    "description": f.description,
                    "priority": f.priority,
                    "estimated_effort": f.estimated_effort,
                    "dependencies": f.dependencies
                })).collect::<Vec<_>>(),
                "new_features_needed": refined.feature_status_intelligence.new_features_needed.iter().map(|f| json!({
                    "name": f.name,
                    "description": f.description,
                    "business_justification": f.business_justification,
                    "user_impact": f.user_impact,
                    "priority": f.priority
                })).collect::<Vec<_>>(),
                "technical_debt": refined.feature_status_intelligence.technical_debt_items.iter().map(|d| json!({
                    "category": d.category,
                    "description": d.description,
                    "severity": d.severity,
                    "location": d.location,
                    "business_impact": d.business_impact,
                    "recommended_resolution": d.recommended_resolution
                })).collect::<Vec<_>>()
            })
        } else {
            json!({
                "metadata": {
                    "analysis_type": "automated",
                    "overall_completion_percentage": analysis.status_intelligence.overall_completion_percentage
                },
                "completed_features": analysis.status_intelligence.completed_features.iter().map(|f| json!({
                    "name": f.name,
                    "description": f.description,
                    "evidence": f.evidence,
                    "confidence": f.confidence,
                    "related_files": f.related_files
                })).collect::<Vec<_>>(),
                "in_progress_features": analysis.status_intelligence.in_progress_features.iter().map(|f| json!({
                    "name": f.name,
                    "description": f.description,
                    "evidence": f.evidence,
                    "confidence": f.confidence,
                    "related_files": f.related_files
                })).collect::<Vec<_>>(),
                "todo_features": analysis.status_intelligence.todo_features.iter().map(|f| json!({
                    "name": f.name,
                    "description": f.description,
                    "evidence": f.evidence,
                    "confidence": f.confidence,
                    "related_files": f.related_files
                })).collect::<Vec<_>>(),
                "technical_debt": analysis.status_intelligence.technical_debt.iter().map(|d| json!({
                    "description": d.description,
                    "severity": d.severity,
                    "location": d.location,
                    "recommendation": d.recommendation
                })).collect::<Vec<_>>()
            })
        };
        
        serde_json::to_string_pretty(&feature_data).map_err(|e| anyhow::anyhow!("JSON serialization error: {}", e))
    }

    fn generate_user_stories_discovered(
        analysis: &CodebaseAnalysis,
        refined_analysis: Option<&RefinedAnalysisResult>
    ) -> Result<String> {
        let mut content = String::new();
        
        content.push_str("# User Stories Discovered Through Codebase Analysis\n\n");
        
        if let Some(refined) = refined_analysis {
            content.push_str("## Analysis Metadata\n");
            content.push_str("- **Source**: Human-refined codebase analysis\n");
            content.push_str(&format!("- **Total Stories**: {}\n", refined.user_stories.stories.len()));
            content.push_str(&format!("- **Confidence Improvement**: {}%\n", 
                (refined.metadata.confidence_improvement.user_personas.1 * 100.0) as u8));
            content.push_str("\n");

            // Group stories by status
            let completed: Vec<_> = refined.user_stories.stories.iter()
                .filter(|s| matches!(s.status, crate::core::refinement_structures::FeatureStatus::Complete))
                .collect();
            let in_progress: Vec<_> = refined.user_stories.stories.iter()
                .filter(|s| matches!(s.status, crate::core::refinement_structures::FeatureStatus::InProgress))
                .collect();
            let todo: Vec<_> = refined.user_stories.stories.iter()
                .filter(|s| matches!(s.status, crate::core::refinement_structures::FeatureStatus::Todo))
                .collect();
            let new: Vec<_> = refined.user_stories.stories.iter()
                .filter(|s| matches!(s.status, crate::core::refinement_structures::FeatureStatus::New))
                .collect();

            if !completed.is_empty() {
                content.push_str("## Completed User Stories (Implemented Features)\n");
                for story in completed {
                    Self::format_refined_story(&mut content, story);
                }
                content.push_str("\n");
            }

            if !in_progress.is_empty() {
                content.push_str("## In-Progress User Stories (Partial Implementation)\n");
                for story in in_progress {
                    Self::format_refined_story(&mut content, story);
                }
                content.push_str("\n");
            }

            if !todo.is_empty() {
                content.push_str("## Todo User Stories (Identified but Not Started)\n");
                for story in todo {
                    Self::format_refined_story(&mut content, story);
                }
                content.push_str("\n");
            }

            if !new.is_empty() {
                content.push_str("## New User Stories (Gap Analysis)\n");
                content.push_str("*These stories were identified through business analysis as missing functionality.*\n\n");
                for story in new {
                    Self::format_refined_story(&mut content, story);
                }
                content.push_str("\n");
            }

        } else {
            content.push_str("## Analysis Metadata\n");
            content.push_str("- **Source**: Automated codebase analysis\n");
            content.push_str(&format!("- **Total Stories**: {}\n", analysis.user_stories.len()));
            content.push_str(&format!("- **Overall Confidence**: {:.1}%\n", 
                analysis.analysis_metadata.confidence_score * 100.0));
            content.push_str("\n");

            if analysis.user_stories.is_empty() {
                content.push_str("*No user stories were automatically extracted from the codebase.*\n\n");
                content.push_str("**Recommendation**: Consider running with LLM enhancement or human refinement ");
                content.push_str("to improve user story extraction accuracy.\n");
            } else {
                content.push_str("## Extracted User Stories\n");
                for story in &analysis.user_stories {
                    Self::format_basic_story(&mut content, story);
                }
            }
        }

        content.push_str("\n---\n");
        content.push_str("*These user stories can be imported into CCMP for systematic product management ");
        content.push_str("and used as input for Claude Code Spec Workflow development planning.*\n");
        
        Ok(content)
    }

    fn format_refined_story(content: &mut String, story: &crate::core::refinement_structures::RefinedUserStory) {
        content.push_str(&format!("### {} - {}\n", story.id, story.title));
        content.push_str(&format!("**Persona**: {} | **Business Value**: {} | **Status**: {:?}\n\n", 
            story.validated_persona.role, story.business_value, story.status));
        content.push_str(&format!("**Story**: {}\n\n", story.description));
        
        if !story.acceptance_criteria.is_empty() {
            content.push_str("**Acceptance Criteria**:\n");
            for (i, criteria) in story.acceptance_criteria.iter().enumerate() {
                content.push_str(&format!("{}. {}\n", i + 1, criteria));
            }
            content.push_str("\n");
        }
        
        if !story.evidence.is_empty() {
            content.push_str(&format!("**Evidence**: {}\n", story.evidence.join(", ")));
        }
        
        if let Some(gap) = &story.implementation_gap {
            content.push_str(&format!("**Implementation Gap**: {}\n", gap));
        }
        
        if let Some(effort) = &story.estimated_effort {
            content.push_str(&format!("**Estimated Effort**: {}\n", effort));
        }
        
        content.push_str("\n---\n\n");
    }

    fn format_basic_story(content: &mut String, story: &crate::core::UserStory) {
        content.push_str(&format!("### {} - {}\n", story.id, story.title));
        content.push_str(&format!("**Priority**: {:?} | **Complexity**: {:?} | **Status**: {:?}\n\n", 
            story.priority, story.complexity, story.status));
        content.push_str(&format!("**Description**: {}\n\n", story.description));
        
        if !story.acceptance_criteria.is_empty() {
            content.push_str("**Acceptance Criteria**:\n");
            for (i, criteria) in story.acceptance_criteria.iter().enumerate() {
                content.push_str(&format!("{}. {}\n", i + 1, criteria));
            }
            content.push_str("\n");
        }
        
        if !story.inferred_from.is_empty() {
            content.push_str(&format!("**Inferred From**: {}\n", story.inferred_from.join(", ")));
        }
        
        if !story.related_components.is_empty() {
            content.push_str(&format!("**Related Components**: {}\n", story.related_components.join(", ")));
        }
        
        content.push_str("\n---\n\n");
    }
}