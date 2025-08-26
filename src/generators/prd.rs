use super::{DocumentGenerator, DocumentType};
use crate::core::CodebaseAnalysis;
use crate::intelligence::IntelligentAnalysis;
use anyhow::Result;

pub struct PRDGenerator;

impl PRDGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl DocumentGenerator for PRDGenerator {
    fn generate(&self, analysis: &CodebaseAnalysis, intelligent_analysis: Option<&IntelligentAnalysis>) -> Result<String> {
        let mut content = String::new();
        
        // Document header
        content.push_str(&format!("# Product Requirements Document\n"));
        content.push_str(&format!("## {}\n\n", analysis.project_name));
        
        content.push_str("---\n\n");
        content.push_str("| Property | Value |\n");
        content.push_str("|----------|-------|\n");
        content.push_str(&format!("| **Document Version** | 1.0 |\n"));
        content.push_str(&format!("| **Project Name** | {} |\n", analysis.project_name));
        content.push_str(&format!("| **Project Type** | {:?} |\n", analysis.project_type));
        content.push_str(&format!("| **Last Updated** | {} |\n", analysis.analysis_metadata.analyzed_at.split('T').next().unwrap_or("")));
        content.push_str(&format!("| **Analysis Confidence** | {:.1}% |\n", analysis.analysis_metadata.confidence_score * 100.0));
        content.push_str("\n---\n\n");
        
        // Table of Contents
        content.push_str("## Table of Contents\n\n");
        content.push_str("1. [Executive Summary](#1-executive-summary)\n");
        content.push_str("2. [Product Overview](#2-product-overview)\n");
        content.push_str("3. [Objectives and Goals](#3-objectives-and-goals)\n");
        content.push_str("4. [Target Users and Personas](#4-target-users-and-personas)\n");
        content.push_str("5. [Feature Specifications](#5-feature-specifications)\n");
        content.push_str("6. [Technical Requirements](#6-technical-requirements)\n");
        content.push_str("7. [User Stories and Acceptance Criteria](#7-user-stories-and-acceptance-criteria)\n");
        content.push_str("8. [Success Metrics](#8-success-metrics)\n");
        if intelligent_analysis.is_some() {
            content.push_str("9. [Risk Assessment](#9-risk-assessment)\n");
            content.push_str("10. [Quality Assurance](#10-quality-assurance)\n");
        }
        content.push_str("\n");
        
        // Executive Summary
        content.push_str("## 1. Executive Summary\n\n");
        content.push_str(&format!("### 1.1 Product Overview\n\n"));
        content.push_str(&format!("{}\n\n", analysis.prd.overview));
        
        content.push_str("### 1.2 Business Context\n\n");
        content.push_str(&format!("{}\n\n", analysis.prd.business_context));
        
        content.push_str("### 1.3 Key Statistics\n\n");
        content.push_str(&format!("- **Codebase Size**: {} lines across {} files\n", 
            analysis.analysis_metadata.lines_of_code, analysis.analysis_metadata.files_analyzed));
        content.push_str(&format!("- **Components**: {} identified components\n", analysis.components.len()));
        content.push_str(&format!("- **User Stories**: {} functional requirements\n", analysis.user_stories.len()));
        content.push_str(&format!("- **Features**: {} core features identified\n", analysis.prd.features.len()));
        
        if let Some(intel) = intelligent_analysis {
            content.push_str(&format!("- **Quality Score**: {:.1}% overall system quality\n", 
                intel.quality_metrics.overall_score * 100.0));
        }
        content.push_str("\n");
        
        // Product Overview
        content.push_str("## 2. Product Overview\n\n");
        content.push_str("### 2.1 Product Description\n\n");
        content.push_str(&format!("{}\n\n", analysis.prd.overview));
        
        content.push_str("### 2.2 Value Proposition\n\n");
        let framework_name = match analysis.project_type {
            crate::core::ProjectType::React => "React-based frontend application",
            crate::core::ProjectType::SpringBoot => "Spring Boot backend service",
            crate::core::ProjectType::Django => "Django web application",
            crate::core::ProjectType::Flask => "Flask web service",
            crate::core::ProjectType::Unknown => "Software application",
        };
        
        content.push_str(&format!("This {} provides comprehensive functionality through:\n\n", framework_name));
        
        // Component-based value props
        let service_count = analysis.components.iter()
            .filter(|c| matches!(c.component_type, crate::core::ComponentType::Service))
            .count();
        let form_count = analysis.components.iter()
            .filter(|c| matches!(c.component_type, crate::core::ComponentType::Form))
            .count();
        let data_count = analysis.components.iter()
            .filter(|c| matches!(c.component_type, crate::core::ComponentType::Context))
            .count();
        
        if service_count > 0 {
            content.push_str(&format!("- **Service Layer**: {} business logic components ensuring reliable operations\n", service_count));
        }
        if form_count > 0 {
            content.push_str(&format!("- **User Interface**: {} forms providing intuitive data input\n", form_count));
        }
        if data_count > 0 {
            content.push_str(&format!("- **Data Management**: {} data models ensuring persistent storage\n", data_count));
        }
        content.push_str("\n");
        
        // Objectives and Goals
        content.push_str("## 3. Objectives and Goals\n\n");
        content.push_str("### 3.1 Primary Objectives\n\n");
        for (i, objective) in analysis.prd.objectives.iter().enumerate() {
            content.push_str(&format!("{}. {}\n", i + 1, objective));
        }
        content.push_str("\n");
        
        if let Some(intel) = intelligent_analysis {
            content.push_str("### 3.2 Quality Objectives\n\n");
            let quality_score = intel.quality_metrics.overall_score * 100.0;
            match quality_score {
                score if score >= 80.0 => {
                    content.push_str("**Status**:  **High Quality** - System demonstrates excellent engineering practices\n\n");
                    content.push_str("- Maintain current quality standards\n");
                    content.push_str("- Focus on feature delivery and user value\n");
                },
                score if score >= 60.0 => {
                    content.push_str("**Status**: 🟡 **Good Quality** - System has solid foundation with room for improvement\n\n");
                    content.push_str("- Address technical debt systematically\n");
                    content.push_str("- Improve test coverage and documentation\n");
                },
                _ => {
                    content.push_str("**Status**: 🔴 **Improvement Needed** - System requires quality improvements\n\n");
                    content.push_str("- Prioritize refactoring and code quality\n");
                    content.push_str("- Implement comprehensive testing strategy\n");
                    content.push_str("- Address technical debt before adding new features\n");
                }
            }
            content.push_str("\n");
        }
        
        // Target Users and Personas
        content.push_str("## 4. Target Users and Personas\n\n");
        for (i, user_type) in analysis.prd.target_users.iter().enumerate() {
            content.push_str(&format!("### 4.{} {}\n\n", i + 1, user_type));
            
            // Generate persona details based on user type
            match user_type.to_lowercase() {
                s if s.contains("end user") || s.contains("web") => {
                    content.push_str("**Profile**: Primary application users seeking intuitive and efficient functionality\n\n");
                    content.push_str("**Needs**:\n");
                    content.push_str("- Easy-to-use interface\n");
                    content.push_str("- Fast response times\n");
                    content.push_str("- Reliable functionality\n");
                    content.push_str("- Clear feedback and error messages\n\n");
                },
                s if s.contains("api") || s.contains("client") => {
                    content.push_str("**Profile**: Developers and systems integrating with the application\n\n");
                    content.push_str("**Needs**:\n");
                    content.push_str("- Well-documented APIs\n");
                    content.push_str("- Consistent response formats\n");
                    content.push_str("- Proper error handling\n");
                    content.push_str("- Authentication and authorization\n\n");
                },
                s if s.contains("admin") => {
                    content.push_str("**Profile**: System administrators managing application operations\n\n");
                    content.push_str("**Needs**:\n");
                    content.push_str("- Comprehensive monitoring and logging\n");
                    content.push_str("- Configuration management\n");
                    content.push_str("- Performance optimization tools\n");
                    content.push_str("- Security management capabilities\n\n");
                },
                _ => {
                    content.push_str(&format!("**Profile**: {}\n\n", user_type));
                }
            }
        }
        
        // Feature Specifications
        content.push_str("## 5. Feature Specifications\n\n");
        for (i, feature) in analysis.prd.features.iter().enumerate() {
            content.push_str(&format!("### 5.{} {}\n\n", i + 1, feature.name));
            
            content.push_str(&format!("**Description**: {}\n\n", feature.description));
            content.push_str(&format!("**User Value**: {}\n\n", feature.user_value));
            content.push_str(&format!("**Technical Implementation**: {}\n\n", feature.technical_approach));
            
            // Find related user stories
            let related_stories: Vec<_> = analysis.user_stories.iter()
                .filter(|story| {
                    let story_title_lower = story.title.to_lowercase();
                    let feature_name_lower = feature.name.to_lowercase();
                    story_title_lower.contains(&feature_name_lower) || 
                    feature_name_lower.contains(&story_title_lower)
                })
                .collect();
            
            if !related_stories.is_empty() {
                content.push_str("**Related User Stories**:\n");
                for story in related_stories {
                    content.push_str(&format!("- {} - {}\n", story.id, story.title));
                }
                content.push_str("\n");
            }
        }
        
        // Technical Requirements
        content.push_str("## 6. Technical Requirements\n\n");
        content.push_str("### 6.1 Core Requirements\n\n");
        for (i, requirement) in analysis.prd.technical_requirements.iter().enumerate() {
            content.push_str(&format!("{}. {}\n", i + 1, requirement));
        }
        content.push_str("\n");
        
        content.push_str("### 6.2 System Architecture\n\n");
        let architecture_desc = match analysis.project_type {
            crate::core::ProjectType::React => "Single-page application (SPA) architecture with component-based design",
            crate::core::ProjectType::SpringBoot => "Layered architecture with REST API endpoints and service layer pattern",
            crate::core::ProjectType::Django => "Model-View-Template (MVT) architecture with Django ORM",
            crate::core::ProjectType::Flask => "Microframework architecture with modular blueprint organization",
            crate::core::ProjectType::Unknown => "Modular software architecture",
        };
        content.push_str(&format!("**Architecture Pattern**: {}\n\n", architecture_desc));
        
        // Component breakdown
        let mut component_summary = std::collections::HashMap::new();
        for component in &analysis.components {
            *component_summary.entry(&component.component_type).or_insert(0) += 1;
        }
        
        content.push_str("**Component Distribution**:\n");
        for (comp_type, count) in component_summary {
            let type_desc = match comp_type {
                crate::core::ComponentType::Service => "Business logic and API services",
                crate::core::ComponentType::Context => "Data models and persistence layer",
                crate::core::ComponentType::Form => "User input and validation components",
                crate::core::ComponentType::Page => "Application screens and views",
                crate::core::ComponentType::Navigation => "Navigation and routing components",
                _ => "Utility and supporting components",
            };
            content.push_str(&format!("- **{:?}**: {} components - {}\n", comp_type, count, type_desc));
        }
        content.push_str("\n");
        
        // User Stories and Acceptance Criteria
        content.push_str("## 7. User Stories and Acceptance Criteria\n\n");
        
        // Group stories by priority
        let mut priority_groups: std::collections::HashMap<String, Vec<_>> = std::collections::HashMap::new();
        for story in &analysis.user_stories {
            priority_groups.entry(format!("{:?}", story.priority))
                .or_insert_with(Vec::new)
                .push(story);
        }
        
        let priority_order = ["Critical", "High", "Medium", "Low"];
        for priority in &priority_order {
            if let Some(stories) = priority_groups.get(*priority) {
                content.push_str(&format!("### 7.{} {} Priority Stories\n\n", 
                    priority_order.iter().position(|&x| x == *priority).unwrap() + 1, priority));
                
                for story in stories {
                    content.push_str(&format!("#### {} - {}\n\n", story.id, story.title));
                    content.push_str(&format!("**As a** user, **I want** {}\n\n", story.description));
                    
                    content.push_str("**Acceptance Criteria**:\n");
                    for (i, criteria) in story.acceptance_criteria.iter().enumerate() {
                        content.push_str(&format!("{}. {}\n", i + 1, criteria));
                    }
                    
                    content.push_str(&format!("\n**Complexity**: {:?} | **Status**: {:?}\n\n", 
                        story.complexity, story.status));
                }
            }
        }
        
        // Success Metrics
        content.push_str("## 8. Success Metrics\n\n");
        content.push_str("### 8.1 Development Metrics\n\n");
        
        let complete_components = analysis.components.iter()
            .filter(|c| matches!(c.implementation_status, crate::core::ImplementationStatus::Complete))
            .count();
        let completion_rate = if !analysis.components.is_empty() {
            complete_components as f32 / analysis.components.len() as f32 * 100.0
        } else { 0.0 };
        
        content.push_str(&format!("- **Implementation Progress**: {:.1}% ({}/{} components complete)\n", 
            completion_rate, complete_components, analysis.components.len()));
        
        let complete_stories = analysis.user_stories.iter()
            .filter(|s| matches!(s.status, crate::core::ImplementationStatus::Complete))
            .count();
        let story_completion_rate = if !analysis.user_stories.is_empty() {
            complete_stories as f32 / analysis.user_stories.len() as f32 * 100.0
        } else { 0.0 };
        
        content.push_str(&format!("- **Story Completion**: {:.1}% ({}/{} stories complete)\n", 
            story_completion_rate, complete_stories, analysis.user_stories.len()));
        
        if let Some(intel) = intelligent_analysis {
            content.push_str(&format!("- **Code Quality**: {:.1}% overall quality score\n", 
                intel.quality_metrics.overall_score * 100.0));
            content.push_str(&format!("- **Technical Debt**: {:.1}% debt-free score\n", 
                intel.quality_metrics.technical_debt_score * 100.0));
        }
        content.push_str("\n");
        
        content.push_str("### 8.2 Quality Metrics\n\n");
        content.push_str("- **Performance**: Response times under 200ms for critical operations\n");
        content.push_str("- **Reliability**: 99.9% uptime for production deployment\n");
        content.push_str("- **Security**: Zero critical security vulnerabilities\n");
        content.push_str("- **Maintainability**: Code complexity scores below 70/100\n");
        content.push_str("- **Testing**: Minimum 80% test coverage for critical components\n\n");
        
        // Risk Assessment (if intelligent analysis available)
        if let Some(intel) = intelligent_analysis {
            content.push_str("## 9. Risk Assessment\n\n");
            
            if !intel.technical_insights.is_empty() {
                content.push_str("### 9.1 Technical Risks\n\n");
                for insight in &intel.technical_insights {
                    let risk_level = match insight.severity {
                        crate::intelligence::Severity::Critical => "🔴 **Critical Risk**",
                        crate::intelligence::Severity::High => "🟠 **High Risk**",
                        crate::intelligence::Severity::Medium => "🟡 **Medium Risk**",
                        crate::intelligence::Severity::Low => "🟢 **Low Risk**",
                    };
                    
                    content.push_str(&format!("#### {} - {}\n\n", risk_level, insight.title));
                    content.push_str(&format!("**Description**: {}\n\n", insight.description));
                    
                    if !insight.affected_components.is_empty() {
                        content.push_str(&format!("**Affected Components**: {}\n\n", insight.affected_components.join(", ")));
                    }
                    
                    content.push_str("**Mitigation Strategies**:\n");
                    for rec in &insight.recommendations {
                        content.push_str(&format!("- {}\n", rec));
                    }
                    content.push_str("\n");
                }
            }
            
            // Business risks from business insights
            let high_risk_stories = intel.enhanced_stories.iter()
                .filter(|s| matches!(s.implementation_risk, crate::intelligence::Risk::High | crate::intelligence::Risk::Critical))
                .count();
            
            if high_risk_stories > 0 {
                content.push_str("### 9.2 Business Risks\n\n");
                content.push_str(&format!("#### 🟠 **Implementation Risk** - {} High-Risk Stories\n\n", high_risk_stories));
                content.push_str("**Description**: Multiple user stories have been identified as high-risk for implementation.\n\n");
                content.push_str("**Mitigation Strategies**:\n");
                content.push_str("- Prioritize risk assessment and planning for high-risk stories\n");
                content.push_str("- Allocate senior developers to critical implementations\n");
                content.push_str("- Consider proof-of-concept development for uncertain areas\n");
                content.push_str("- Implement comprehensive testing for high-risk components\n\n");
            }
            
            // Quality Assurance
            content.push_str("## 10. Quality Assurance\n\n");
            content.push_str("### 10.1 Quality Standards\n\n");
            
            let quality_level = if intel.quality_metrics.overall_score >= 0.8 {
                "**Excellent** - System meets high quality standards"
            } else if intel.quality_metrics.overall_score >= 0.6 {
                "**Good** - System has solid quality foundation"
            } else {
                "**Needs Improvement** - System requires quality enhancements"
            };
            
            content.push_str(&format!("**Current Quality Level**: {}\n\n", quality_level));
            
            content.push_str("### 10.2 Quality Metrics Dashboard\n\n");
            content.push_str("| Metric | Current Score | Target | Status |\n");
            content.push_str("|--------|---------------|--------|---------|\n");
            
            let format_status = |score: f32, target: f32| -> &str {
                if score >= target { " Met" } else { " In Progress" }
            };
            
            content.push_str(&format!("| Overall Quality | {:.1}% | 80% | {} |\n", 
                intel.quality_metrics.overall_score * 100.0, format_status(intel.quality_metrics.overall_score, 0.8)));
            content.push_str(&format!("| Maintainability | {:.1}% | 75% | {} |\n", 
                intel.quality_metrics.maintainability * 100.0, format_status(intel.quality_metrics.maintainability, 0.75)));
            content.push_str(&format!("| Technical Debt | {:.1}% | 85% | {} |\n", 
                intel.quality_metrics.technical_debt_score * 100.0, format_status(intel.quality_metrics.technical_debt_score, 0.85)));
            content.push_str(&format!("| Test Coverage | {:.1}% | 80% | {} |\n", 
                intel.quality_metrics.test_coverage_estimate * 100.0, format_status(intel.quality_metrics.test_coverage_estimate, 0.8)));
            content.push_str("\n");
            
            if !intel.refactoring_opportunities.is_empty() {
                content.push_str("### 10.3 Improvement Opportunities\n\n");
                for (i, opportunity) in intel.refactoring_opportunities.iter().take(5).enumerate() {
                    content.push_str(&format!("{}. **{}** in {}\n", i + 1, opportunity.issue_type, opportunity.component));
                    content.push_str(&format!("   - {}\n", opportunity.description));
                    content.push_str(&format!("   - **Effort**: {} | **Impact**: {:?}\n", opportunity.effort_estimate, opportunity.impact));
                }
                content.push_str("\n");
            }
        }
        
        // Document footer
        content.push_str("\n---\n\n");
        content.push_str("**Document Information**\n");
        content.push_str(&format!("- **Generated**: {}\n", analysis.analysis_metadata.analyzed_at));
        content.push_str(&format!("- **Generator**: Codebase Workflow Analyzer v{}\n", analysis.analysis_metadata.analyzer_version));
        content.push_str(&format!("- **Analysis Confidence**: {:.1}%\n", analysis.analysis_metadata.confidence_score * 100.0));
        content.push_str("- **Document Type**: Product Requirements Document (PRD)\n\n");
        
        content.push_str("*This document was automatically generated from codebase analysis. Please review and update as needed for your specific requirements.*\n");
        
        Ok(content)
    }

    fn get_file_extension(&self) -> &str {
        "md"
    }

    fn get_document_type(&self) -> DocumentType {
        DocumentType::ProductRequirementDocument
    }
}