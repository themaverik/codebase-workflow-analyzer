use super::{DocumentGenerator, DocumentType};
use crate::core::CodebaseAnalysis;
use crate::intelligence::IntelligentAnalysis;
use anyhow::Result;

pub struct ExecutiveSummaryGenerator;

impl ExecutiveSummaryGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl DocumentGenerator for ExecutiveSummaryGenerator {
    fn generate(&self, analysis: &CodebaseAnalysis, intelligent_analysis: Option<&IntelligentAnalysis>) -> Result<String> {
        let mut content = String::new();
        
        // Document header
        content.push_str("# Executive Summary\n");
        content.push_str(&format!("## {}\n\n", analysis.project_name));
        
        content.push_str("---\n\n");
        content.push_str("| **Executive Summary** | **Key Metrics** |\n");
        content.push_str("|----------------------|------------------|\n");
        content.push_str(&format!("| **Project Name** | {} |\n", analysis.project_name));
        content.push_str(&format!("| **Technology Stack** | {:?} |\n", analysis.project_type));
        content.push_str(&format!("| **Analysis Date** | {} |\n", analysis.analysis_metadata.analyzed_at.split('T').next().unwrap_or("")));
        
        if let Some(intel) = intelligent_analysis {
            let quality_grade = if intel.quality_metrics.overall_score >= 0.9 { "A" }
                              else if intel.quality_metrics.overall_score >= 0.8 { "B" }
                              else if intel.quality_metrics.overall_score >= 0.7 { "C" }
                              else if intel.quality_metrics.overall_score >= 0.6 { "D" }
                              else { "F" };
            content.push_str(&format!("| **Quality Grade** | {} ({:.0}%) |\n", quality_grade, intel.quality_metrics.overall_score * 100.0));
        }
        
        content.push_str("\n---\n\n");
        
        // Key Findings
        content.push_str("##  Key Findings\n\n");
        
        // Project scale and scope
        content.push_str("### Project Scale & Scope\n\n");
        content.push_str(&format!("**Codebase Size**: {} lines of code across {} files\n\n", 
            analysis.analysis_metadata.lines_of_code, analysis.analysis_metadata.files_analyzed));
        
        content.push_str(&format!("**System Complexity**: {} components with an average complexity of {:.1}/100\n\n", 
            analysis.components.len(),
            if !analysis.components.is_empty() { 
                analysis.components.iter().map(|c| c.complexity_score as u32).sum::<u32>() as f32 / analysis.components.len() as f32 
            } else { 0.0 }
        ));
        
        content.push_str(&format!("**Feature Set**: {} user stories representing core functionality\n\n", analysis.user_stories.len()));
        
        // Business Value Assessment
        content.push_str("### Business Value Assessment\n\n");
        
        let critical_high_stories = analysis.user_stories.iter()
            .filter(|s| matches!(s.priority, crate::core::Priority::Critical | crate::core::Priority::High))
            .count();
        
        content.push_str(&format!("**Critical Business Functions**: {} high-priority user stories identified\n\n", critical_high_stories));
        
        // Calculate completion rates
        let complete_components = analysis.components.iter()
            .filter(|c| matches!(c.implementation_status, crate::core::ImplementationStatus::Complete))
            .count();
        let completion_rate = if !analysis.components.is_empty() {
            complete_components as f32 / analysis.components.len() as f32 * 100.0
        } else { 0.0 };
        
        let completion_status = if completion_rate >= 90.0 { "**Excellent**" }
                              else if completion_rate >= 70.0 { "**Good**" }
                              else if completion_rate >= 50.0 { "**Fair**" }
                              else { "**Poor**" };
        
        content.push_str(&format!("**Implementation Progress**: {:.0}% complete ({})\n\n", completion_rate, completion_status));
        
        // Technology assessment
        let framework_assessment = match analysis.project_type {
            crate::core::ProjectType::React => "Modern frontend framework suitable for scalable user interfaces",
            crate::core::ProjectType::NextJS => "Full-stack React framework with SSR/SSG capabilities for optimal performance",
            crate::core::ProjectType::ExpressNodeJS => "Lightweight backend framework ideal for API development and microservices",
            crate::core::ProjectType::NestJS => "Enterprise-grade TypeScript framework with dependency injection and modular architecture",
            crate::core::ProjectType::SpringBoot => "Enterprise-grade backend framework with robust ecosystem",
            crate::core::ProjectType::Django => "Full-featured web framework with rapid development capabilities",
            crate::core::ProjectType::Flask => "Lightweight and flexible web framework for targeted applications",
            crate::core::ProjectType::FastAPI => "High-performance API framework with modern Python async capabilities",
            crate::core::ProjectType::Unknown => "Mixed or custom technology stack requiring evaluation",
        };
        
        content.push_str(&format!("**Technology Assessment**: {}\n\n", framework_assessment));
        
        // Risk Assessment
        content.push_str("## Risk Assessment\n\n");
        
        let incomplete_components = analysis.components.iter()
            .filter(|c| matches!(c.implementation_status, 
                crate::core::ImplementationStatus::Todo | 
                crate::core::ImplementationStatus::Incomplete))
            .count();
        
        let high_complexity_components = analysis.components.iter()
            .filter(|c| c.complexity_score > 70)
            .count();
        
        // Implementation risk
        let implementation_risk = if incomplete_components == 0 { "**Low**" }
                                else if incomplete_components <= analysis.components.len() / 4 { "**Medium**" }
                                else { "**High**" };
        
        content.push_str(&format!("### Implementation Risk: {}\n\n", implementation_risk));
        
        if incomplete_components > 0 {
            content.push_str(&format!("**Findings**: {} components require completion or have incomplete implementations\n\n", incomplete_components));
            content.push_str("**Recommendation**: Prioritize completion of critical components before adding new features\n\n");
        } else {
            content.push_str("**Findings**: All identified components appear to be implemented\n\n");
            content.push_str("**Recommendation**: Focus on quality improvements and feature enhancements\n\n");
        }
        
        // Technical complexity risk
        let complexity_risk = if high_complexity_components == 0 { "**Low**" }
                             else if high_complexity_components <= 3 { "**Medium**" }
                             else { "**High**" };
        
        content.push_str(&format!("### Technical Complexity Risk: {}\n\n", complexity_risk));
        
        if high_complexity_components > 0 {
            content.push_str(&format!("**Findings**: {} components have high complexity scores (>70/100)\n\n", high_complexity_components));
            content.push_str("**Recommendation**: Consider refactoring complex components to improve maintainability\n\n");
        } else {
            content.push_str("**Findings**: Component complexity is well-managed across the codebase\n\n");
            content.push_str("**Recommendation**: Maintain current development practices\n\n");
        }
        
        // Quality Assessment (if intelligent analysis available)
        if let Some(intel) = intelligent_analysis {
            content.push_str("##  Quality Assessment\n\n");
            
            let overall_grade = if intel.quality_metrics.overall_score >= 0.9 { ("A", "Exceptional") }
                              else if intel.quality_metrics.overall_score >= 0.8 { ("B", "Good") }
                              else if intel.quality_metrics.overall_score >= 0.7 { ("C", "Acceptable") }
                              else if intel.quality_metrics.overall_score >= 0.6 { ("D", "Below Average") }
                              else { ("F", "Poor") };
            
            content.push_str(&format!("### Overall Quality Grade: {} ({})\n\n", overall_grade.0, overall_grade.1));
            
            content.push_str("| Quality Metric | Score | Assessment |\n");
            content.push_str("|----------------|-------|------------|\n");
            
            let format_assessment = |score: f32| -> &str {
                if score >= 0.8 { "Excellent" }
                else if score >= 0.6 { "Good" }
                else if score >= 0.4 { "Fair" }
                else { "Poor" }
            };
            
            let maintainability_text = format_assessment(intel.quality_metrics.maintainability);
            content.push_str(&format!("| Code Maintainability | {:.0}% | {} |\n", 
                intel.quality_metrics.maintainability * 100.0, maintainability_text));
            
            let complexity_text = format_assessment(intel.quality_metrics.complexity);
            content.push_str(&format!("| Complexity Management | {:.0}% | {} |\n", 
                intel.quality_metrics.complexity * 100.0, complexity_text));
            
            let debt_text = format_assessment(intel.quality_metrics.technical_debt_score);
            content.push_str(&format!("| Technical Debt | {:.0}% | {} |\n", 
                intel.quality_metrics.technical_debt_score * 100.0, debt_text));
            
            let test_text = format_assessment(intel.quality_metrics.test_coverage_estimate);
            content.push_str(&format!("| Test Coverage (Est.) | {:.0}% | {} |\n", 
                intel.quality_metrics.test_coverage_estimate * 100.0, test_text));
            
            content.push_str("\n");
            
            // Critical issues
            let critical_issues = intel.technical_insights.iter()
                .filter(|i| matches!(i.severity, crate::intelligence::Severity::Critical | crate::intelligence::Severity::High))
                .count();
            
            if critical_issues > 0 {
                content.push_str(&format!("### Critical Issues Identified: {}\n\n", critical_issues));
                
                for insight in intel.technical_insights.iter()
                    .filter(|i| matches!(i.severity, crate::intelligence::Severity::Critical | crate::intelligence::Severity::High))
                    .take(3) {
                    
                    content.push_str(&format!("**{}**: {}\n\n", insight.title, insight.description));
                }
                
                if critical_issues > 3 {
                    content.push_str(&format!("*...and {} more issues identified in detailed analysis*\n\n", critical_issues - 3));
                }
            }
        }
        
        // Strategic Recommendations
        content.push_str("##  Strategic Recommendations\n\n");
        
        content.push_str("### Immediate Actions (Next 30 Days)\n\n");
        
        if incomplete_components > 0 {
            content.push_str(&format!("1. **Complete Implementation**: Address {} incomplete components\n", incomplete_components));
        }
        
        if let Some(intel) = intelligent_analysis {
            let high_severity_issues = intel.technical_insights.iter()
                .filter(|i| matches!(i.severity, crate::intelligence::Severity::High | crate::intelligence::Severity::Critical))
                .count();
            
            if high_severity_issues > 0 {
                content.push_str(&format!("2. **Address Quality Issues**: Resolve {} high-severity technical issues\n", high_severity_issues));
            }
            
            if intel.quality_metrics.test_coverage_estimate < 0.6 {
                content.push_str("3. **Improve Test Coverage**: Implement comprehensive testing strategy\n");
            }
        }
        
        if critical_high_stories > analysis.user_stories.len() / 2 {
            content.push_str("4. **Feature Prioritization**: Focus on critical business functionality\n");
        }
        content.push_str("\n");
        
        content.push_str("### Medium-Term Goals (Next 90 Days)\n\n");
        
        if high_complexity_components > 0 {
            content.push_str("1. **Refactoring Initiative**: Simplify complex components for better maintainability\n");
        }
        
        content.push_str("2. **Documentation Enhancement**: Improve code documentation and technical guides\n");
        content.push_str("3. **Performance Optimization**: Identify and address performance bottlenecks\n");
        content.push_str("4. **Security Review**: Conduct comprehensive security assessment\n\n");
        
        content.push_str("### Long-Term Vision (Next 6 Months)\n\n");
        
        if let Some(intel) = intelligent_analysis {
            if !intel.architecture_recommendations.is_empty() {
                content.push_str(&format!("1. **Architecture Evolution**: Consider implementing {} recommended patterns\n", 
                    intel.architecture_recommendations.len()));
            }
        }
        
        content.push_str("2. **Scalability Planning**: Prepare system for growth and increased load\n");
        content.push_str("3. **Team Development**: Invest in team skills and development practices\n");
        content.push_str("4. **Automation Enhancement**: Improve CI/CD and development automation\n\n");
        
        // Business Impact
        content.push_str("## 💼 Business Impact\n\n");
        
        content.push_str("### Current State\n\n");
        
        let business_readiness = if completion_rate >= 90.0 {
            "**Production Ready**: System appears ready for production deployment with minimal additional work"
        } else if completion_rate >= 70.0 {
            "**Near Production Ready**: System requires focused completion effort before deployment"
        } else if completion_rate >= 50.0 {
            "**Development Stage**: System requires significant development before production readiness"
        } else {
            "**Early Development**: System is in early development stage with substantial work remaining"
        };
        
        content.push_str(&format!("{}\n\n", business_readiness));
        
        // ROI indicators
        content.push_str("### Return on Investment Indicators\n\n");
        
        let api_endpoints: usize = analysis.components.iter()
            .map(|c| c.api_calls.len())
            .sum();
        
        if api_endpoints > 0 {
            content.push_str(&format!("- **API Economy Ready**: {} endpoints identified for potential monetization or partnership\n", api_endpoints));
        }
        
        let service_components = analysis.components.iter()
            .filter(|c| matches!(c.component_type, crate::core::ComponentType::Service))
            .count();
        
        if service_components > 5 {
            content.push_str("- **Microservices Potential**: Architecture suitable for independent scaling and deployment\n");
        }
        
        let form_components = analysis.components.iter()
            .filter(|c| matches!(c.component_type, crate::core::ComponentType::Form))
            .count();
        
        if form_components > 0 {
            content.push_str(&format!("- **User Engagement**: {} data collection points for user insights and analytics\n", form_components));
        }
        
        content.push_str("\n");
        
        // Cost-Benefit Analysis
        content.push_str("### Cost-Benefit Analysis\n\n");
        
        content.push_str("**Development Costs**:\n");
        let estimated_dev_weeks = if completion_rate >= 90.0 { 2 }
                                 else if completion_rate >= 70.0 { 4 }
                                 else if completion_rate >= 50.0 { 8 }
                                 else { 16 };
        
        content.push_str(&format!("- Estimated {} weeks of development effort to reach production readiness\n", estimated_dev_weeks));
        
        if let Some(intel) = intelligent_analysis {
            let quality_improvement_weeks = if intel.quality_metrics.overall_score >= 0.8 { 1 }
                                          else if intel.quality_metrics.overall_score >= 0.6 { 3 }
                                          else { 6 };
            content.push_str(&format!("- Additional {} weeks for quality improvements and testing\n", quality_improvement_weeks));
        }
        
        content.push_str("\n**Benefits**:\n");
        
        match analysis.project_type {
            crate::core::ProjectType::React => {
                content.push_str("- Modern user interface improving user experience and engagement\n");
                content.push_str("- Scalable frontend architecture supporting business growth\n");
            },
            crate::core::ProjectType::SpringBoot => {
                content.push_str("- Enterprise-grade backend supporting high transaction volumes\n");
                content.push_str("- Robust API infrastructure enabling third-party integrations\n");
            },
            crate::core::ProjectType::Django => {
                content.push_str("- Rapid development framework accelerating time-to-market\n");
                content.push_str("- Built-in admin interface reducing operational overhead\n");
            },
            crate::core::ProjectType::Flask => {
                content.push_str("- Lightweight architecture minimizing infrastructure costs\n");
                content.push_str("- Flexible framework supporting diverse business requirements\n");
            },
            _ => {
                content.push_str("- Custom solution addressing specific business needs\n");
                content.push_str("- Flexible architecture supporting future requirements\n");
            }
        }
        
        content.push_str("- Reduced manual processes and operational efficiency gains\n\n");
        
        // Executive Decision Points
        content.push_str("##  Executive Decision Points\n\n");
        
        content.push_str("### Go/No-Go Recommendation\n\n");
        
        let recommendation = if completion_rate >= 80.0 && 
                               intelligent_analysis.map_or(true, |i| i.quality_metrics.overall_score >= 0.7) {
            "**GO**: Proceed with production deployment planning"
        } else if completion_rate >= 60.0 {
            "**CONDITIONAL GO**: Complete identified issues before production"
        } else {
            "**NO-GO**: Significant development required before production consideration"
        };
        
        content.push_str(&format!("{}\n\n", recommendation));
        
        content.push_str("### Key Success Factors\n\n");
        content.push_str("1. **Team Capability**: Ensure development team has necessary skills\n");
        content.push_str("2. **Quality Standards**: Maintain high code quality throughout development\n");
        content.push_str("3. **Testing Strategy**: Implement comprehensive testing before deployment\n");
        content.push_str("4. **Monitoring & Support**: Establish production monitoring and support processes\n\n");
        
        content.push_str("### Budget Considerations\n\n");
        
        let budget_category = if estimated_dev_weeks <= 4 { "Low" }
                             else if estimated_dev_weeks <= 8 { "Medium" } 
                             else { "High" };
        
        content.push_str(&format!("**Development Budget**: {} (estimated {} weeks of development)\n\n", budget_category, estimated_dev_weeks));
        
        content.push_str("**Ongoing Costs**:\n");
        content.push_str("- Infrastructure and hosting\n");
        content.push_str("- Maintenance and support (15-20% of development cost annually)\n");
        content.push_str("- Security updates and compliance\n");
        content.push_str("- Feature enhancements and scalability improvements\n\n");
        
        // Document footer
        content.push_str("---\n\n");
        content.push_str("**Executive Summary Information**\n");
        content.push_str(&format!("- **Prepared**: {}\n", analysis.analysis_metadata.analyzed_at));
        content.push_str(&format!("- **Analysis Tool**: Codebase Workflow Analyzer v{}\n", analysis.analysis_metadata.analyzer_version));
        content.push_str(&format!("- **Analysis Confidence**: {:.1}%\n", analysis.analysis_metadata.confidence_score * 100.0));
        content.push_str("- **Document Type**: Executive Summary\n\n");
        
        content.push_str("*This executive summary provides a high-level overview based on automated codebase analysis. Detailed technical and business reviews are recommended before making final decisions.*\n");
        
        Ok(content)
    }

    fn get_file_extension(&self) -> &str {
        "md"
    }

    fn get_document_type(&self) -> DocumentType {
        DocumentType::ExecutiveSummary
    }
}