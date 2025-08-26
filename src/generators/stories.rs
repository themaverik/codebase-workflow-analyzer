use super::{DocumentGenerator, DocumentType, format_priority, format_complexity, format_status};
use crate::core::CodebaseAnalysis;
use crate::intelligence::IntelligentAnalysis;
use anyhow::Result;

pub struct UserStoryGenerator;

impl UserStoryGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl DocumentGenerator for UserStoryGenerator {
    fn generate(&self, analysis: &CodebaseAnalysis, intelligent_analysis: Option<&IntelligentAnalysis>) -> Result<String> {
        let mut content = String::new();
        
        // Document header
        content.push_str("# User Stories Document\n");
        content.push_str(&format!("## {}\n\n", analysis.project_name));
        
        content.push_str("---\n\n");
        content.push_str("| Property | Value |\n");
        content.push_str("|----------|-------|\n");
        content.push_str(&format!("| **Project** | {} |\n", analysis.project_name));
        content.push_str(&format!("| **Total Stories** | {} |\n", analysis.user_stories.len()));
        content.push_str(&format!("| **Generated** | {} |\n", analysis.analysis_metadata.analyzed_at.split('T').next().unwrap_or("")));
        content.push_str("\n---\n\n");
        
        if analysis.user_stories.is_empty() {
            content.push_str("No user stories were identified in the codebase analysis.\n");
            return Ok(content);
        }
        
        // Story summary
        content.push_str("##  Story Summary\n\n");
        
        // Priority distribution
        let mut priority_counts = std::collections::HashMap::new();
        let mut complexity_counts = std::collections::HashMap::new();
        let mut status_counts = std::collections::HashMap::new();
        
        for story in &analysis.user_stories {
            *priority_counts.entry(format!("{:?}", story.priority)).or_insert(0) += 1;
            *complexity_counts.entry(format!("{:?}", story.complexity)).or_insert(0) += 1;
            *status_counts.entry(format!("{:?}", story.status)).or_insert(0) += 1;
        }
        
        content.push_str("### Priority Distribution\n\n");
        content.push_str("| Priority | Count | Percentage |\n");
        content.push_str("|----------|-------|------------|\n");
        for (priority, count) in &priority_counts {
            let percentage = *count as f32 / analysis.user_stories.len() as f32 * 100.0;
            content.push_str(&format!("| {} | {} | {:.1}% |\n", priority, count, percentage));
        }
        content.push_str("\n");
        
        content.push_str("### Complexity Distribution\n\n");
        content.push_str("| Complexity | Count | Percentage |\n");
        content.push_str("|------------|-------|------------|\n");
        for (complexity, count) in &complexity_counts {
            let percentage = *count as f32 / analysis.user_stories.len() as f32 * 100.0;
            content.push_str(&format!("| {} | {} | {:.1}% |\n", complexity, count, percentage));
        }
        content.push_str("\n");
        
        content.push_str("### Implementation Status\n\n");
        content.push_str("| Status | Count | Percentage |\n");
        content.push_str("|--------|-------|------------|\n");
        for (status, count) in &status_counts {
            let percentage = *count as f32 / analysis.user_stories.len() as f32 * 100.0;
            content.push_str(&format!("| {} | {} | {:.1}% |\n", status, count, percentage));
        }
        content.push_str("\n");
        
        // Enhanced stories if available
        if let Some(intel) = intelligent_analysis {
            content.push_str("## 🧠 Enhanced Story Insights\n\n");
            
            let high_value_stories = intel.enhanced_stories.iter()
                .filter(|s| s.business_value_score > 0.7)
                .count();
            let high_risk_stories = intel.enhanced_stories.iter()
                .filter(|s| matches!(s.implementation_risk, crate::intelligence::Risk::High | crate::intelligence::Risk::Critical))
                .count();
            
            content.push_str("### Key Metrics\n\n");
            content.push_str(&format!("- **High Business Value Stories**: {} ({:.1}%)\n", 
                high_value_stories, high_value_stories as f32 / analysis.user_stories.len() as f32 * 100.0));
            content.push_str(&format!("- **High Risk Stories**: {} ({:.1}%)\n", 
                high_risk_stories, high_risk_stories as f32 / analysis.user_stories.len() as f32 * 100.0));
            
            let avg_business_value = intel.enhanced_stories.iter()
                .map(|s| s.business_value_score)
                .sum::<f32>() / intel.enhanced_stories.len() as f32;
            content.push_str(&format!("- **Average Business Value Score**: {:.2}/1.0\n\n", avg_business_value));
            
            // Top value stories
            if high_value_stories > 0 {
                content.push_str("### 💎 High Business Value Stories\n\n");
                let mut high_value_list: Vec<_> = intel.enhanced_stories.iter()
                    .filter(|s| s.business_value_score > 0.7)
                    .collect();
                high_value_list.sort_by(|a, b| b.business_value_score.partial_cmp(&a.business_value_score).unwrap());
                
                for story in high_value_list.iter().take(5) {
                    content.push_str(&format!("- **{}** - {} (Value: {:.2})\n", 
                        story.base_story.id, story.base_story.title, story.business_value_score));
                }
                content.push_str("\n");
            }
            
            // High risk stories
            if high_risk_stories > 0 {
                content.push_str("### ⚠ High Risk Stories\n\n");
                let high_risk_list: Vec<_> = intel.enhanced_stories.iter()
                    .filter(|s| matches!(s.implementation_risk, crate::intelligence::Risk::High | crate::intelligence::Risk::Critical))
                    .collect();
                
                for story in high_risk_list.iter().take(5) {
                    content.push_str(&format!("- **{}** - {} (Risk: {:?})\n", 
                        story.base_story.id, story.base_story.title, story.implementation_risk));
                }
                content.push_str("\n");
            }
        }
        
        // Detailed stories by priority
        content.push_str("##  Detailed User Stories\n\n");
        
        let priority_order = [
            crate::core::Priority::Critical,
            crate::core::Priority::High,
            crate::core::Priority::Medium,
            crate::core::Priority::Low,
        ];
        
        for priority in &priority_order {
            let priority_stories: Vec<_> = analysis.user_stories.iter()
                .filter(|s| std::mem::discriminant(&s.priority) == std::mem::discriminant(priority))
                .collect();
            
            if priority_stories.is_empty() {
                continue;
            }
            
            content.push_str(&format!("### {} Priority Stories\n\n", format_priority(priority)));
            
            for story in priority_stories {
                content.push_str(&format!("#### {} - {}\n\n", story.id, story.title));
                
                // Story details table
                content.push_str("| Attribute | Value |\n");
                content.push_str("|-----------|-------|\n");
                content.push_str(&format!("| **ID** | {} |\n", story.id));
                content.push_str(&format!("| **Priority** | {} |\n", format_priority(&story.priority)));
                content.push_str(&format!("| **Complexity** | {} |\n", format_complexity(&story.complexity)));
                content.push_str(&format!("| **Status** | {} |\n", format_status(&story.status)));
                
                if !story.related_components.is_empty() {
                    content.push_str(&format!("| **Components** | {} |\n", story.related_components.join(", ")));
                }
                
                // Add enhanced information if available
                if let Some(intel) = intelligent_analysis {
                    if let Some(enhanced) = intel.enhanced_stories.iter().find(|s| s.base_story.id == story.id) {
                        content.push_str(&format!("| **Business Value** | {:.2}/1.0 |\n", enhanced.business_value_score));
                        content.push_str(&format!("| **Implementation Risk** | {:?} |\n", enhanced.implementation_risk));
                        
                        if !enhanced.dependencies.is_empty() {
                            content.push_str(&format!("| **Dependencies** | {} |\n", enhanced.dependencies.join(", ")));
                        }
                    }
                }
                content.push_str("\n");
                
                // Story description
                content.push_str("**Story Description:**\n\n");
                content.push_str(&format!("> {}\n\n", story.description));
                
                // Acceptance criteria
                content.push_str("**Acceptance Criteria:**\n\n");
                for (i, criteria) in story.acceptance_criteria.iter().enumerate() {
                    content.push_str(&format!("{}.  {}\n", i + 1, criteria));
                }
                content.push_str("\n");
                
                // Enhanced acceptance tests if available
                if let Some(intel) = intelligent_analysis {
                    if let Some(enhanced) = intel.enhanced_stories.iter().find(|s| s.base_story.id == story.id) {
                        if !enhanced.acceptance_tests.is_empty() {
                            content.push_str("**Acceptance Tests (BDD Format):**\n\n");
                            for (i, test) in enhanced.acceptance_tests.iter().enumerate() {
                                content.push_str(&format!("**Test {}: {}**\n", i + 1, test.scenario));
                                content.push_str(&format!("- **Given**: {}\n", test.given));
                                content.push_str(&format!("- **When**: {}\n", test.when));
                                content.push_str(&format!("- **Then**: {}\n\n", test.then));
                            }
                        }
                    }
                }
                
                // Story source
                if !story.inferred_from.is_empty() {
                    content.push_str("**Inferred From:**\n");
                    for source in &story.inferred_from {
                        content.push_str(&format!("- `{}`\n", source));
                    }
                    content.push_str("\n");
                }
                
                content.push_str("---\n\n");
            }
        }
        
        // Story backlog recommendations
        content.push_str("##  Backlog Recommendations\n\n");
        
        // Sprint planning recommendations
        content.push_str("### Sprint Planning Guidance\n\n");
        
        let critical_high_stories = analysis.user_stories.iter()
            .filter(|s| matches!(s.priority, crate::core::Priority::Critical | crate::core::Priority::High))
            .count();
        let simple_medium_stories = analysis.user_stories.iter()
            .filter(|s| matches!(s.complexity, crate::core::Complexity::Simple | crate::core::Complexity::Medium))
            .count();
        
        content.push_str(&format!("**Immediate Priority**: {} critical/high priority stories should be addressed first\n\n", critical_high_stories));
        content.push_str(&format!("**Quick Wins**: {} simple/medium complexity stories for momentum building\n\n", simple_medium_stories));
        
        // Story estimation
        content.push_str("### Story Estimation Guidelines\n\n");
        content.push_str("| Complexity | Story Points | Estimated Hours | Team Size |\n");
        content.push_str("|------------|--------------|----------------|------------|\n");
        content.push_str("| Simple | 1-2 | 4-8 hours | 1 developer |\n");
        content.push_str("| Medium | 3-5 | 1-2 days | 1-2 developers |\n");
        content.push_str("| Complex | 8-13 | 3-5 days | 2-3 developers |\n");
        content.push_str("| Epic | 20+ | 1-2 weeks | 3+ developers |\n\n");
        
        // Implementation order recommendations
        if let Some(intel) = intelligent_analysis {
            content.push_str("###  Recommended Implementation Order\n\n");
            
            // Sort stories by business value and risk
            let mut prioritized_stories: Vec<_> = intel.enhanced_stories.iter()
                .map(|s| {
                    let priority_score = match s.base_story.priority {
                        crate::core::Priority::Critical => 4.0,
                        crate::core::Priority::High => 3.0,
                        crate::core::Priority::Medium => 2.0,
                        crate::core::Priority::Low => 1.0,
                    };
                    let complexity_factor = match s.base_story.complexity {
                        crate::core::Complexity::Simple => 1.0,
                        crate::core::Complexity::Medium => 0.8,
                        crate::core::Complexity::Complex => 0.6,
                        crate::core::Complexity::Epic => 0.4,
                    };
                    let risk_factor = match s.implementation_risk {
                        crate::intelligence::Risk::Low => 1.0,
                        crate::intelligence::Risk::Medium => 0.8,
                        crate::intelligence::Risk::High => 0.6,
                        crate::intelligence::Risk::Critical => 0.4,
                    };
                    
                    let weighted_score = (s.business_value_score * 0.4) + 
                                       (priority_score / 4.0 * 0.3) + 
                                       (complexity_factor * 0.15) + 
                                       (risk_factor * 0.15);
                    
                    (s, weighted_score)
                })
                .collect();
            
            prioritized_stories.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            
            content.push_str("**Phase 1 - Foundation (Highest Value, Lower Risk)**\n");
            for (story, score) in prioritized_stories.iter().take(5) {
                content.push_str(&format!("1. **{}** - {} (Score: {:.2})\n", 
                    story.base_story.id, story.base_story.title, score));
            }
            content.push_str("\n");
            
            if prioritized_stories.len() > 5 {
                content.push_str("**Phase 2 - Core Features**\n");
                for (story, score) in prioritized_stories.iter().skip(5).take(5) {
                    content.push_str(&format!("1. **{}** - {} (Score: {:.2})\n", 
                        story.base_story.id, story.base_story.title, score));
                }
                content.push_str("\n");
            }
            
            if prioritized_stories.len() > 10 {
                content.push_str("**Phase 3 - Enhancement Features**\n");
                for (story, score) in prioritized_stories.iter().skip(10) {
                    content.push_str(&format!("1. **{}** - {} (Score: {:.2})\n", 
                        story.base_story.id, story.base_story.title, score));
                }
                content.push_str("\n");
            }
        }
        
        // Definition of Done
        content.push_str("##  Definition of Done\n\n");
        content.push_str("For each user story to be considered complete, the following criteria must be met:\n\n");
        content.push_str("### Development Criteria\n");
        content.push_str("- [ ] Code implementation matches acceptance criteria\n");
        content.push_str("- [ ] Code follows project coding standards\n");
        content.push_str("- [ ] Unit tests written and passing (minimum 80% coverage)\n");
        content.push_str("- [ ] Integration tests pass\n");
        content.push_str("- [ ] Code reviewed and approved by team member\n");
        content.push_str("- [ ] No critical or high-severity security issues\n\n");
        
        content.push_str("### Quality Criteria\n");
        content.push_str("- [ ] Functionality works as expected in development environment\n");
        content.push_str("- [ ] Performance meets requirements (if applicable)\n");
        content.push_str("- [ ] Accessibility standards met (WCAG 2.1 AA minimum)\n");
        content.push_str("- [ ] Cross-browser compatibility verified (if frontend)\n");
        content.push_str("- [ ] Error handling implemented and tested\n\n");
        
        content.push_str("### Documentation Criteria\n");
        content.push_str("- [ ] API documentation updated (if applicable)\n");
        content.push_str("- [ ] README or user documentation updated\n");
        content.push_str("- [ ] Inline code comments for complex logic\n");
        content.push_str("- [ ] Database schema changes documented\n\n");
        
        content.push_str("### Deployment Criteria\n");
        content.push_str("- [ ] Feature deployed to staging environment\n");
        content.push_str("- [ ] Stakeholder acceptance received\n");
        content.push_str("- [ ] Monitoring and logging in place\n");
        content.push_str("- [ ] Rollback plan documented\n\n");
        
        // Document footer
        content.push_str("---\n\n");
        content.push_str("**Document Information**\n");
        content.push_str(&format!("- **Generated**: {}\n", analysis.analysis_metadata.analyzed_at));
        content.push_str(&format!("- **Generator**: Codebase Workflow Analyzer v{}\n", analysis.analysis_metadata.analyzer_version));
        content.push_str(&format!("- **Total Stories**: {}\n", analysis.user_stories.len()));
        content.push_str("- **Document Type**: User Stories Document\n\n");
        
        content.push_str("*This document was automatically generated from codebase analysis. Stories should be reviewed, refined, and validated with stakeholders before implementation.*\n");
        
        Ok(content)
    }

    fn get_file_extension(&self) -> &str {
        "md"
    }

    fn get_document_type(&self) -> DocumentType {
        DocumentType::UserStories
    }
}