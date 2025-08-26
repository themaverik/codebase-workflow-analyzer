use crate::core::{
    CodebaseAnalysis, ComponentInfo, UserStory, Task, Priority, Complexity, 
    ImplementationStatus, TaskType, ComponentType
};
use regex::Regex;
use std::collections::HashMap;

pub struct IntelligenceEngine {
    patterns: PatternDatabase,
    scoring: ScoringEngine,
}

pub struct PatternDatabase {
    business_patterns: Vec<BusinessPattern>,
    technical_patterns: Vec<TechnicalPattern>,
    anti_patterns: Vec<AntiPattern>,
}

pub struct ScoringEngine {
    weights: ScoringWeights,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BusinessPattern {
    pub name: String,
    pub description: String,
    pub indicators: Vec<String>,
    pub business_value: String,
    pub priority_boost: Priority,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TechnicalPattern {
    pub name: String,
    pub description: String,
    pub code_patterns: Vec<String>,
    pub complexity_factor: f32,
    pub quality_impact: QualityImpact,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AntiPattern {
    pub name: String,
    pub description: String,
    pub detection_rules: Vec<String>,
    pub severity: Severity,
    pub recommendations: Vec<String>,
}

#[derive(Clone, Debug)]
pub enum QualityImpact {
    Positive,
    Negative,
    Neutral,
}

#[derive(Clone, Debug)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

pub struct ScoringWeights {
    pub complexity_weight: f32,
    pub business_value_weight: f32,
    pub technical_debt_weight: f32,
    pub implementation_status_weight: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IntelligentAnalysis {
    pub enhanced_stories: Vec<EnhancedUserStory>,
    pub technical_insights: Vec<TechnicalInsight>,
    pub architecture_recommendations: Vec<ArchitectureRecommendation>,
    pub refactoring_opportunities: Vec<RefactoringOpportunity>,
    pub business_insights: Vec<BusinessInsight>,
    pub quality_metrics: QualityMetrics,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnhancedUserStory {
    pub base_story: UserStory,
    pub business_value_score: f32,
    pub implementation_risk: Risk,
    pub dependencies: Vec<String>,
    pub acceptance_tests: Vec<AcceptanceTest>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TechnicalInsight {
    pub category: InsightCategory,
    pub title: String,
    pub description: String,
    pub affected_components: Vec<String>,
    pub severity: Severity,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum InsightCategory {
    Architecture,
    Performance,
    Security,
    Maintainability,
    Testing,
    Documentation,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArchitectureRecommendation {
    pub pattern_name: String,
    pub description: String,
    pub benefits: Vec<String>,
    pub implementation_effort: String,
    pub applicable_components: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RefactoringOpportunity {
    pub component: String,
    pub issue_type: String,
    pub description: String,
    pub impact: QualityImpact,
    pub effort_estimate: String,
    pub expected_benefits: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BusinessInsight {
    pub insight_type: BusinessInsightType,
    pub description: String,
    pub impact: String,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum BusinessInsightType {
    UserExperience,
    FeaturePriority,
    MarketOpportunity,
    RiskAssessment,
    ResourceAllocation,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Risk {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AcceptanceTest {
    pub scenario: String,
    pub given: String,
    pub when: String,
    pub then: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QualityMetrics {
    pub overall_score: f32,
    pub maintainability: f32,
    pub complexity: f32,
    pub test_coverage_estimate: f32,
    pub technical_debt_score: f32,
    pub documentation_score: f32,
}

impl IntelligenceEngine {
    pub fn new() -> Self {
        Self {
            patterns: PatternDatabase::new(),
            scoring: ScoringEngine::new(),
        }
    }

    pub fn enhance_analysis(&self, analysis: &CodebaseAnalysis) -> IntelligentAnalysis {
        let enhanced_stories = self.enhance_user_stories(&analysis.user_stories, &analysis.components);
        let technical_insights = self.analyze_technical_patterns(&analysis.components);
        let architecture_recommendations = self.generate_architecture_recommendations(&analysis.components);
        let refactoring_opportunities = self.identify_refactoring_opportunities(&analysis.components);
        let business_insights = self.extract_business_insights(&analysis.components, &enhanced_stories);
        let quality_metrics = self.calculate_quality_metrics(&analysis.components, &technical_insights);

        IntelligentAnalysis {
            enhanced_stories,
            technical_insights,
            architecture_recommendations,
            refactoring_opportunities,
            business_insights,
            quality_metrics,
        }
    }

    fn enhance_user_stories(&self, stories: &[UserStory], components: &[ComponentInfo]) -> Vec<EnhancedUserStory> {
        stories.iter().map(|story| {
            let business_value_score = self.scoring.calculate_business_value(story, components);
            let implementation_risk = self.assess_implementation_risk(story, components);
            let dependencies = self.identify_story_dependencies(story, components);
            let acceptance_tests = self.generate_acceptance_tests(story);

            EnhancedUserStory {
                base_story: story.clone(),
                business_value_score,
                implementation_risk,
                dependencies,
                acceptance_tests,
            }
        }).collect()
    }

    fn analyze_technical_patterns(&self, components: &[ComponentInfo]) -> Vec<TechnicalInsight> {
        let mut insights = Vec::new();

        // Analyze complexity distribution
        let high_complexity_components: Vec<_> = components.iter()
            .filter(|c| c.complexity_score > 70)
            .collect();

        if !high_complexity_components.is_empty() {
            insights.push(TechnicalInsight {
                category: InsightCategory::Maintainability,
                title: "High Complexity Components Detected".to_string(),
                description: format!("Found {} components with high complexity scores (>70). These may be candidates for refactoring.", 
                    high_complexity_components.len()),
                affected_components: high_complexity_components.iter().map(|c| c.name.clone()).collect(),
                severity: if high_complexity_components.len() > 5 { Severity::High } else { Severity::Medium },
                recommendations: vec![
                    "Consider breaking down complex components into smaller, focused units".to_string(),
                    "Apply Single Responsibility Principle".to_string(),
                    "Add comprehensive unit tests for complex components".to_string(),
                ],
            });
        }

        // Analyze API design patterns
        let api_components: Vec<_> = components.iter()
            .filter(|c| !c.api_calls.is_empty())
            .collect();

        if api_components.len() > 3 {
            insights.push(TechnicalInsight {
                category: InsightCategory::Architecture,
                title: "API-Heavy Architecture Detected".to_string(),
                description: format!("Found {} components with API interactions. Consider implementing API gateway pattern.", 
                    api_components.len()),
                affected_components: api_components.iter().map(|c| c.name.clone()).collect(),
                severity: Severity::Medium,
                recommendations: vec![
                    "Implement API gateway for centralized request handling".to_string(),
                    "Add API versioning strategy".to_string(),
                    "Consider implementing circuit breaker pattern for resilience".to_string(),
                ],
            });
        }

        // Analyze incomplete implementations
        let incomplete_components: Vec<_> = components.iter()
            .filter(|c| matches!(c.implementation_status, ImplementationStatus::Todo | ImplementationStatus::Incomplete))
            .collect();

        if !incomplete_components.is_empty() {
            insights.push(TechnicalInsight {
                category: InsightCategory::Testing,
                title: "Incomplete Implementation Risk".to_string(),
                description: format!("Found {} components with incomplete implementations that may impact system reliability.", 
                    incomplete_components.len()),
                affected_components: incomplete_components.iter().map(|c| c.name.clone()).collect(),
                severity: if incomplete_components.len() > 3 { Severity::High } else { Severity::Medium },
                recommendations: vec![
                    "Prioritize completion of critical components".to_string(),
                    "Add comprehensive testing for completed components".to_string(),
                    "Document known limitations and workarounds".to_string(),
                ],
            });
        }

        insights
    }

    fn generate_architecture_recommendations(&self, components: &[ComponentInfo]) -> Vec<ArchitectureRecommendation> {
        let mut recommendations = Vec::new();

        // Analyze component types for architecture patterns
        let service_count = components.iter().filter(|c| matches!(c.component_type, ComponentType::Service)).count();
        let context_count = components.iter().filter(|c| matches!(c.component_type, ComponentType::Context)).count();

        if service_count > 5 && context_count > 3 {
            recommendations.push(ArchitectureRecommendation {
                pattern_name: "Domain-Driven Design (DDD)".to_string(),
                description: "Consider organizing components into domain-specific modules based on business capabilities".to_string(),
                benefits: vec![
                    "Improved code organization and maintainability".to_string(),
                    "Better alignment with business requirements".to_string(),
                    "Easier to scale individual domains".to_string(),
                ],
                implementation_effort: "Medium".to_string(),
                applicable_components: components.iter()
                    .filter(|c| matches!(c.component_type, ComponentType::Service | ComponentType::Context))
                    .map(|c| c.name.clone())
                    .collect(),
            });
        }

        if service_count > 8 {
            recommendations.push(ArchitectureRecommendation {
                pattern_name: "Microservices Architecture".to_string(),
                description: "Large number of services suggests potential for microservices decomposition".to_string(),
                benefits: vec![
                    "Independent deployment and scaling".to_string(),
                    "Technology diversity and team autonomy".to_string(),
                    "Fault isolation and resilience".to_string(),
                ],
                implementation_effort: "Large".to_string(),
                applicable_components: components.iter()
                    .filter(|c| matches!(c.component_type, ComponentType::Service))
                    .map(|c| c.name.clone())
                    .collect(),
            });
        }

        // Check for caching opportunities
        let high_complexity_services: Vec<_> = components.iter()
            .filter(|c| matches!(c.component_type, ComponentType::Service) && c.complexity_score > 50)
            .collect();

        if !high_complexity_services.is_empty() {
            recommendations.push(ArchitectureRecommendation {
                pattern_name: "Caching Strategy".to_string(),
                description: "Complex services may benefit from caching to improve performance".to_string(),
                benefits: vec![
                    "Reduced response times".to_string(),
                    "Lower resource utilization".to_string(),
                    "Improved user experience".to_string(),
                ],
                implementation_effort: "Small".to_string(),
                applicable_components: high_complexity_services.iter().map(|c| c.name.clone()).collect(),
            });
        }

        recommendations
    }

    fn identify_refactoring_opportunities(&self, components: &[ComponentInfo]) -> Vec<RefactoringOpportunity> {
        let mut opportunities = Vec::new();

        for component in components {
            // High complexity refactoring
            if component.complexity_score > 80 {
                opportunities.push(RefactoringOpportunity {
                    component: component.name.clone(),
                    issue_type: "High Complexity".to_string(),
                    description: format!("Component {} has very high complexity ({}). Consider breaking it down.", 
                        component.name, component.complexity_score),
                    impact: QualityImpact::Positive,
                    effort_estimate: "Medium".to_string(),
                    expected_benefits: vec![
                        "Improved maintainability".to_string(),
                        "Easier testing".to_string(),
                        "Reduced bug likelihood".to_string(),
                    ],
                });
            }

            // Long dependency list
            if component.dependencies.len() > 8 {
                opportunities.push(RefactoringOpportunity {
                    component: component.name.clone(),
                    issue_type: "High Coupling".to_string(),
                    description: format!("Component {} has {} dependencies, indicating high coupling.", 
                        component.name, component.dependencies.len()),
                    impact: QualityImpact::Positive,
                    effort_estimate: "Large".to_string(),
                    expected_benefits: vec![
                        "Reduced coupling".to_string(),
                        "Improved testability".to_string(),
                        "Better separation of concerns".to_string(),
                    ],
                });
            }

            // Incomplete implementation
            if matches!(component.implementation_status, ImplementationStatus::Todo | ImplementationStatus::Incomplete) {
                opportunities.push(RefactoringOpportunity {
                    component: component.name.clone(),
                    issue_type: "Incomplete Implementation".to_string(),
                    description: format!("Component {} is not fully implemented.", component.name),
                    impact: QualityImpact::Positive,
                    effort_estimate: if component.complexity_score > 50 { "Large" } else { "Medium" }.to_string(),
                    expected_benefits: vec![
                        "Complete functionality".to_string(),
                        "Reduced technical debt".to_string(),
                        "System reliability".to_string(),
                    ],
                });
            }
        }

        opportunities
    }

    fn extract_business_insights(&self, components: &[ComponentInfo], enhanced_stories: &[EnhancedUserStory]) -> Vec<BusinessInsight> {
        let mut insights = Vec::new();

        // User experience insights
        let form_components: Vec<_> = components.iter()
            .filter(|c| matches!(c.component_type, ComponentType::Form))
            .collect();

        if !form_components.is_empty() {
            insights.push(BusinessInsight {
                insight_type: BusinessInsightType::UserExperience,
                description: format!("Found {} form components which are critical for user interaction and data collection.", 
                    form_components.len()),
                impact: "High user engagement dependency on form usability and validation".to_string(),
                recommended_actions: vec![
                    "Implement comprehensive form validation".to_string(),
                    "Add user-friendly error messages".to_string(),
                    "Consider progressive disclosure for complex forms".to_string(),
                ],
            });
        }

        // Feature priority insights
        let high_value_stories: Vec<_> = enhanced_stories.iter()
            .filter(|s| s.business_value_score > 0.7)
            .collect();

        if !high_value_stories.is_empty() {
            insights.push(BusinessInsight {
                insight_type: BusinessInsightType::FeaturePriority,
                description: format!("Identified {} high-value user stories that should be prioritized for maximum business impact.", 
                    high_value_stories.len()),
                impact: "Significant business value delivery potential".to_string(),
                recommended_actions: vec![
                    "Prioritize high-value stories in sprint planning".to_string(),
                    "Allocate senior developers to critical features".to_string(),
                    "Consider early user testing for high-impact features".to_string(),
                ],
            });
        }

        // Risk assessment
        let high_risk_stories: Vec<_> = enhanced_stories.iter()
            .filter(|s| matches!(s.implementation_risk, Risk::High | Risk::Critical))
            .collect();

        if !high_risk_stories.is_empty() {
            insights.push(BusinessInsight {
                insight_type: BusinessInsightType::RiskAssessment,
                description: format!("Found {} high-risk user stories that may impact project timeline and success.", 
                    high_risk_stories.len()),
                impact: "Potential project delays and quality issues".to_string(),
                recommended_actions: vec![
                    "Create detailed risk mitigation plans".to_string(),
                    "Consider proof-of-concept implementations".to_string(),
                    "Allocate additional time and resources for high-risk features".to_string(),
                ],
            });
        }

        insights
    }

    fn calculate_quality_metrics(&self, components: &[ComponentInfo], insights: &[TechnicalInsight]) -> QualityMetrics {
        let total_complexity: u32 = components.iter().map(|c| c.complexity_score as u32).sum();
        let avg_complexity = if !components.is_empty() { 
            total_complexity as f32 / components.len() as f32 
        } else { 0.0 };

        let complete_components = components.iter()
            .filter(|c| matches!(c.implementation_status, ImplementationStatus::Complete))
            .count();
        let completion_rate = if !components.is_empty() {
            complete_components as f32 / components.len() as f32
        } else { 0.0 };

        let high_severity_issues = insights.iter()
            .filter(|i| matches!(i.severity, Severity::High | Severity::Critical))
            .count() as f32;

        let maintainability = (1.0 - (avg_complexity / 100.0)).max(0.0);
        let complexity_score = (1.0 - (avg_complexity / 100.0)).max(0.0);
        let technical_debt_score = (1.0 - (high_severity_issues / components.len().max(1) as f32)).max(0.0);

        // Estimate test coverage based on component complexity and completion
        let test_coverage_estimate = completion_rate * 0.6 + (1.0 - avg_complexity / 100.0) * 0.4;

        // Estimate documentation score based on component types and complexity
        let documentation_score = if components.iter().any(|c| matches!(c.component_type, ComponentType::Service)) {
            0.6 // Assume moderate documentation for service-heavy projects
        } else {
            0.4
        };

        let overall_score = (maintainability + complexity_score + technical_debt_score + 
                           test_coverage_estimate + documentation_score) / 5.0;

        QualityMetrics {
            overall_score,
            maintainability,
            complexity: complexity_score,
            test_coverage_estimate,
            technical_debt_score,
            documentation_score,
        }
    }

    fn assess_implementation_risk(&self, story: &UserStory, components: &[ComponentInfo]) -> Risk {
        let related_component = components.iter()
            .find(|c| story.related_components.contains(&c.name));

        if let Some(component) = related_component {
            match component.implementation_status {
                ImplementationStatus::Todo => Risk::High,
                ImplementationStatus::Incomplete => Risk::Medium,
                ImplementationStatus::InProgress => Risk::Medium,
                ImplementationStatus::Complete => {
                    if component.complexity_score > 70 { Risk::Medium } else { Risk::Low }
                }
            }
        } else {
            Risk::Low
        }
    }

    fn identify_story_dependencies(&self, story: &UserStory, components: &[ComponentInfo]) -> Vec<String> {
        let mut dependencies = Vec::new();
        
        for component_name in &story.related_components {
            if let Some(component) = components.iter().find(|c| c.name == *component_name) {
                for dep in &component.dependencies {
                    if !dependencies.contains(dep) && !dep.starts_with("std") && !dep.starts_with("java.") {
                        dependencies.push(dep.clone());
                    }
                }
            }
        }
        
        dependencies
    }

    fn generate_acceptance_tests(&self, story: &UserStory) -> Vec<AcceptanceTest> {
        let mut tests = Vec::new();
        
        for (i, criteria) in story.acceptance_criteria.iter().enumerate() {
            tests.push(AcceptanceTest {
                scenario: format!("Test {} - {}", i + 1, criteria),
                given: format!("Given the user wants to {}", story.description.to_lowercase()),
                when: format!("When they {}", criteria.to_lowercase()),
                then: "Then the system should respond appropriately".to_string(),
            });
        }
        
        tests
    }
}

impl PatternDatabase {
    fn new() -> Self {
        Self {
            business_patterns: Self::load_business_patterns(),
            technical_patterns: Self::load_technical_patterns(),
            anti_patterns: Self::load_anti_patterns(),
        }
    }

    fn load_business_patterns() -> Vec<BusinessPattern> {
        vec![
            BusinessPattern {
                name: "User Authentication".to_string(),
                description: "Login and authentication functionality".to_string(),
                indicators: vec!["login".to_string(), "auth".to_string(), "signin".to_string()],
                business_value: "Critical for user security and personalization".to_string(),
                priority_boost: Priority::Critical,
            },
            BusinessPattern {
                name: "Data Management".to_string(),
                description: "CRUD operations on business entities".to_string(),
                indicators: vec!["create".to_string(), "update".to_string(), "delete".to_string()],
                business_value: "Essential for business operations".to_string(),
                priority_boost: Priority::High,
            },
            BusinessPattern {
                name: "User Dashboard".to_string(),
                description: "Central user interface with key metrics".to_string(),
                indicators: vec!["dashboard".to_string(), "overview".to_string(), "summary".to_string()],
                business_value: "High user engagement and retention".to_string(),
                priority_boost: Priority::High,
            },
        ]
    }

    fn load_technical_patterns() -> Vec<TechnicalPattern> {
        vec![
            TechnicalPattern {
                name: "Repository Pattern".to_string(),
                description: "Data access abstraction".to_string(),
                code_patterns: vec!["Repository".to_string(), "DAO".to_string()],
                complexity_factor: 0.8,
                quality_impact: QualityImpact::Positive,
            },
            TechnicalPattern {
                name: "Service Layer".to_string(),
                description: "Business logic encapsulation".to_string(),
                code_patterns: vec!["Service".to_string(), "Manager".to_string()],
                complexity_factor: 1.0,
                quality_impact: QualityImpact::Positive,
            },
        ]
    }

    fn load_anti_patterns() -> Vec<AntiPattern> {
        vec![
            AntiPattern {
                name: "God Object".to_string(),
                description: "Single class/component doing too much".to_string(),
                detection_rules: vec!["complexity > 90".to_string(), "methods > 20".to_string()],
                severity: Severity::High,
                recommendations: vec![
                    "Break into smaller, focused components".to_string(),
                    "Apply Single Responsibility Principle".to_string(),
                ],
            },
        ]
    }
}

impl ScoringEngine {
    fn new() -> Self {
        Self {
            weights: ScoringWeights {
                complexity_weight: 0.3,
                business_value_weight: 0.4,
                technical_debt_weight: 0.2,
                implementation_status_weight: 0.1,
            },
        }
    }

    fn calculate_business_value(&self, story: &UserStory, components: &[ComponentInfo]) -> f32 {
        let mut score = 0.0;

        // Priority-based scoring
        score += match story.priority {
            Priority::Critical => 1.0,
            Priority::High => 0.8,
            Priority::Medium => 0.6,
            Priority::Low => 0.4,
        };

        // Complexity-based scoring (inverse - simpler = higher business value)
        score += match story.complexity {
            Complexity::Simple => 0.8,
            Complexity::Medium => 0.6,
            Complexity::Complex => 0.4,
            Complexity::Epic => 0.2,
        };

        // Implementation status impact
        score += match story.status {
            ImplementationStatus::Complete => 0.2,
            ImplementationStatus::InProgress => 0.1,
            ImplementationStatus::Todo => 0.0,
            ImplementationStatus::Incomplete => -0.1,
        };

        (score / 2.0_f32).min(1.0).max(0.0)
    }
}

impl Default for IntelligenceEngine {
    fn default() -> Self {
        Self::new()
    }
}