use super::{DocumentGenerator, DocumentType, format_priority, format_complexity, format_status, format_component_type};
use crate::core::CodebaseAnalysis;
use crate::intelligence::IntelligentAnalysis;
use anyhow::Result;

pub struct MarkdownGenerator;

impl MarkdownGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl DocumentGenerator for MarkdownGenerator {
    fn generate(&self, analysis: &CodebaseAnalysis, intelligent_analysis: Option<&IntelligentAnalysis>) -> Result<String> {
        let mut content = String::new();
        
        // Title and overview
        content.push_str(&format!("#  Codebase Analysis: {}\n\n", analysis.project_name));
        
        // Table of Contents
        content.push_str("##  Table of Contents\n\n");
        content.push_str("- [Project Overview](#project-overview)\n");
        content.push_str("- [Components Analysis](#components-analysis)\n");
        content.push_str("- [User Stories](#user-stories)\n");
        content.push_str("- [Product Requirements](#product-requirements)\n");
        content.push_str("- [Task Breakdown](#task-breakdown)\n");
        if intelligent_analysis.is_some() {
            content.push_str("- [Intelligence Insights](#intelligence-insights)\n");
        }
        content.push_str("- [Analysis Metadata](#analysis-metadata)\n\n");
        
        // Project Overview
        content.push_str("##  Project Overview\n\n");
        content.push_str(&format!("- **Project Type**: {:?}\n", analysis.project_type));
        content.push_str(&format!("- **Files Analyzed**: {}\n", analysis.analysis_metadata.files_analyzed));
        content.push_str(&format!("- **Lines of Code**: {}\n", analysis.analysis_metadata.lines_of_code));
        content.push_str(&format!("- **Components Found**: {}\n", analysis.components.len()));
        content.push_str(&format!("- **User Stories**: {}\n", analysis.user_stories.len()));
        content.push_str(&format!("- **Tasks**: {}\n", analysis.tasks.len()));
        content.push_str(&format!("- **Confidence Score**: {:.1}%\n\n", analysis.analysis_metadata.confidence_score * 100.0));
        
        // Components Analysis
        content.push_str("##  Components Analysis\n\n");
        
        if !analysis.components.is_empty() {
            content.push_str("###  Component Overview\n\n");
            
            // Component type distribution
            let mut type_counts = std::collections::HashMap::new();
            let mut total_complexity = 0u32;
            let mut status_counts = std::collections::HashMap::new();
            
            for component in &analysis.components {
                *type_counts.entry(format!("{:?}", component.component_type)).or_insert(0) += 1;
                total_complexity += component.complexity_score as u32;
                *status_counts.entry(format!("{:?}", component.implementation_status)).or_insert(0) += 1;
            }
            
            content.push_str("#### Component Types\n\n");
            content.push_str("| Type | Count |\n");
            content.push_str("|------|-------|\n");
            for (comp_type, count) in &type_counts {
                content.push_str(&format!("| {} | {} |\n", comp_type, count));
            }
            content.push_str("\n");
            
            content.push_str("#### Implementation Status\n\n");
            content.push_str("| Status | Count |\n");
            content.push_str("|--------|-------|\n");
            for (status, count) in &status_counts {
                content.push_str(&format!("| {} | {} |\n", status, count));
            }
            content.push_str("\n");
            
            let avg_complexity = total_complexity as f32 / analysis.components.len() as f32;
            content.push_str(&format!("**Average Complexity Score**: {:.1}/100\n\n", avg_complexity));
            
            // Detailed component list
            content.push_str("###  Component Details\n\n");
            
            let mut sorted_components = analysis.components.clone();
            sorted_components.sort_by(|a, b| b.complexity_score.cmp(&a.complexity_score));
            
            for component in &sorted_components {
                content.push_str(&format!("#### {} {}\n\n", format_component_type(&component.component_type), component.name));
                content.push_str(&format!("- **Purpose**: {}\n", component.purpose));
                content.push_str(&format!("- **Complexity**: {}/100\n", component.complexity_score));
                content.push_str(&format!("- **Status**: {}\n", format_status(&component.implementation_status)));
                content.push_str(&format!("- **File**: `{}`\n", component.file_path));
                
                if !component.props.is_empty() {
                    content.push_str("- **Props/Fields**:\n");
                    for prop in &component.props {
                        let required = if prop.required { "required" } else { "optional" };
                        content.push_str(&format!("  - `{}` ({}): {}\n", prop.name, prop.prop_type, required));
                    }
                }
                
                if !component.api_calls.is_empty() {
                    content.push_str("- **API Endpoints**:\n");
                    for api in &component.api_calls {
                        content.push_str(&format!("  - `{} {}` - {}\n", api.method, api.endpoint, api.purpose));
                    }
                }
                
                if !component.dependencies.is_empty() {
                    content.push_str("- **Dependencies**:\n");
                    for dep in &component.dependencies {
                        content.push_str(&format!("  - `{}`\n", dep));
                    }
                }
                
                content.push_str("\n");
            }
        }
        
        // User Stories
        content.push_str("##  User Stories\n\n");
        
        if !analysis.user_stories.is_empty() {
            // Priority summary
            let mut priority_counts = std::collections::HashMap::new();
            for story in &analysis.user_stories {
                *priority_counts.entry(format_priority(&story.priority)).or_insert(0) += 1;
            }
            
            content.push_str("### Priority Distribution\n\n");
            content.push_str("| Priority | Count |\n");
            content.push_str("|----------|-------|\n");
            for (priority, count) in &priority_counts {
                content.push_str(&format!("| {} | {} |\n", priority, count));
            }
            content.push_str("\n");
            
            // Detailed stories
            content.push_str("### Story Details\n\n");
            
            for story in &analysis.user_stories {
                content.push_str(&format!("#### {} - {}\n\n", story.id, story.title));
                content.push_str(&format!("**Description**: {}\n\n", story.description));
                content.push_str(&format!("- **Priority**: {}\n", format_priority(&story.priority)));
                content.push_str(&format!("- **Complexity**: {}\n", format_complexity(&story.complexity)));
                content.push_str(&format!("- **Status**: {}\n", format_status(&story.status)));
                
                if !story.related_components.is_empty() {
                    content.push_str(&format!("- **Related Components**: {}\n", story.related_components.join(", ")));
                }
                
                content.push_str("\n**Acceptance Criteria**:\n");
                for (i, criteria) in story.acceptance_criteria.iter().enumerate() {
                    content.push_str(&format!("{}. {}\n", i + 1, criteria));
                }
                content.push_str("\n");
            }
        }
        
        // Product Requirements
        content.push_str("##  Product Requirements\n\n");
        content.push_str(&format!("### {}\n\n", analysis.prd.title));
        content.push_str(&format!("**Overview**: {}\n\n", analysis.prd.overview));
        
        if !analysis.prd.objectives.is_empty() {
            content.push_str("###  Objectives\n\n");
            for objective in &analysis.prd.objectives {
                content.push_str(&format!("- {}\n", objective));
            }
            content.push_str("\n");
        }
        
        if !analysis.prd.target_users.is_empty() {
            content.push_str("### 👥 Target Users\n\n");
            for user in &analysis.prd.target_users {
                content.push_str(&format!("- {}\n", user));
            }
            content.push_str("\n");
        }
        
        if !analysis.prd.features.is_empty() {
            content.push_str("### ✨ Features\n\n");
            for feature in &analysis.prd.features {
                content.push_str(&format!("#### {}\n\n", feature.name));
                content.push_str(&format!("**Description**: {}\n\n", feature.description));
                content.push_str(&format!("**User Value**: {}\n\n", feature.user_value));
                content.push_str(&format!("**Technical Approach**: {}\n\n", feature.technical_approach));
            }
        }
        
        if !analysis.prd.technical_requirements.is_empty() {
            content.push_str("###  Technical Requirements\n\n");
            for req in &analysis.prd.technical_requirements {
                content.push_str(&format!("- {}\n", req));
            }
            content.push_str("\n");
        }
        
        content.push_str(&format!("### 💼 Business Context\n\n{}\n\n", analysis.prd.business_context));
        
        // Task Breakdown
        content.push_str("##  Task Breakdown\n\n");
        
        if !analysis.tasks.is_empty() {
            // Group tasks by status
            let mut status_groups: std::collections::HashMap<String, Vec<_>> = std::collections::HashMap::new();
            for task in &analysis.tasks {
                status_groups.entry(format!("{:?}", task.status))
                    .or_insert_with(Vec::new)
                    .push(task);
            }
            
            for (status, tasks) in &status_groups {
                content.push_str(&format!("### {} Tasks ({})\n\n", status, tasks.len()));
                
                content.push_str("| ID | Title | Type | Effort | Components |\n");
                content.push_str("|----|-------|------|--------|------------|\n");
                
                for task in tasks {
                    let components = task.related_components.join(", ");
                    let effort = task.effort_estimate.as_ref().map(|e| e.as_str()).unwrap_or("Not specified");
                    content.push_str(&format!("| {} | {} | {:?} | {} | {} |\n", 
                        task.id, task.name, task.task_type, effort, components));
                }
                content.push_str("\n");
            }
        }
        
        // Intelligence Insights
        if let Some(intel) = intelligent_analysis {
            content.push_str("## 🧠 Intelligence Insights\n\n");
            
            // Quality metrics
            content.push_str("###  Quality Metrics\n\n");
            content.push_str("| Metric | Score |\n");
            content.push_str("|--------|-------|\n");
            content.push_str(&format!("| Overall Quality | {:.1}% |\n", intel.quality_metrics.overall_score * 100.0));
            content.push_str(&format!("| Maintainability | {:.1}% |\n", intel.quality_metrics.maintainability * 100.0));
            content.push_str(&format!("| Complexity Management | {:.1}% |\n", intel.quality_metrics.complexity * 100.0));
            content.push_str(&format!("| Technical Debt | {:.1}% |\n", intel.quality_metrics.technical_debt_score * 100.0));
            content.push_str(&format!("| Estimated Test Coverage | {:.1}% |\n", intel.quality_metrics.test_coverage_estimate * 100.0));
            content.push_str(&format!("| Documentation | {:.1}% |\n", intel.quality_metrics.documentation_score * 100.0));
            content.push_str("\n");
            
            // Technical insights
            if !intel.technical_insights.is_empty() {
                content.push_str("###  Technical Insights\n\n");
                for insight in &intel.technical_insights {
                    content.push_str(&format!("#### {} - {}\n\n", 
                        format!("{:?}", insight.category), insight.title));
                    content.push_str(&format!("**Severity**: {:?}\n\n", insight.severity));
                    content.push_str(&format!("{}\n\n", insight.description));
                    
                    if !insight.affected_components.is_empty() {
                        content.push_str(&format!("**Affected Components**: {}\n\n", insight.affected_components.join(", ")));
                    }
                    
                    content.push_str("**Recommendations**:\n");
                    for rec in &insight.recommendations {
                        content.push_str(&format!("- {}\n", rec));
                    }
                    content.push_str("\n");
                }
            }
            
            // Architecture recommendations
            if !intel.architecture_recommendations.is_empty() {
                content.push_str("###  Architecture Recommendations\n\n");
                for rec in &intel.architecture_recommendations {
                    content.push_str(&format!("#### {}\n\n", rec.pattern_name));
                    content.push_str(&format!("{}\n\n", rec.description));
                    content.push_str(&format!("**Implementation Effort**: {}\n\n", rec.implementation_effort));
                    
                    content.push_str("**Benefits**:\n");
                    for benefit in &rec.benefits {
                        content.push_str(&format!("- {}\n", benefit));
                    }
                    content.push_str("\n");
                }
            }
            
            // Business insights
            if !intel.business_insights.is_empty() {
                content.push_str("### 💼 Business Insights\n\n");
                for insight in &intel.business_insights {
                    content.push_str(&format!("#### {:?}\n\n", insight.insight_type));
                    content.push_str(&format!("{}\n\n", insight.description));
                    content.push_str(&format!("**Impact**: {}\n\n", insight.impact));
                    
                    content.push_str("**Recommended Actions**:\n");
                    for action in &insight.recommended_actions {
                        content.push_str(&format!("- {}\n", action));
                    }
                    content.push_str("\n");
                }
            }
        }
        
        // Analysis Metadata
        content.push_str("##  Analysis Metadata\n\n");
        content.push_str("| Property | Value |\n");
        content.push_str("|----------|-------|\n");
        content.push_str(&format!("| Analysis Date | {} |\n", analysis.analysis_metadata.analyzed_at));
        content.push_str(&format!("| Analyzer Version | {} |\n", analysis.analysis_metadata.analyzer_version));
        content.push_str(&format!("| Files Processed | {} |\n", analysis.analysis_metadata.files_analyzed));
        content.push_str(&format!("| Total Lines | {} |\n", analysis.analysis_metadata.lines_of_code));
        content.push_str(&format!("| Confidence Score | {:.1}% |\n", analysis.analysis_metadata.confidence_score * 100.0));
        
        if !analysis.analysis_metadata.warnings.is_empty() {
            content.push_str("\n### Warnings\n\n");
            for warning in &analysis.analysis_metadata.warnings {
                content.push_str(&format!("- {}\n", warning));
            }
        }
        
        content.push_str("\n---\n");
        content.push_str("*Generated by Codebase Workflow Analyzer*\n");
        
        Ok(content)
    }

    fn get_file_extension(&self) -> &str {
        "md"
    }

    fn get_document_type(&self) -> DocumentType {
        DocumentType::Markdown
    }
}