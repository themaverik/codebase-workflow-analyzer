use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::core::{
    types::{BusinessDomain, Framework},
    UserStory,
};

// Placeholder for AnalysisResult - will be integrated with existing analysis structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub business_context: BusinessDomain,
    pub frameworks_detected: Vec<Framework>,
    pub confidence_scores: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefinementSession {
    pub session_id: String,
    pub analysis_input: AnalysisResult,
    pub refinement_corrections: RefinementCorrections,
    pub validation_status: ValidationStatus,
    pub stakeholder_inputs: Vec<StakeholderInput>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefinementCorrections {
    pub product_type_correction: Option<ProductTypeCorrection>,
    pub persona_corrections: Vec<PersonaCorrection>,
    pub feature_priority_corrections: Vec<PriorityCorrection>,
    pub business_context_enhancement: BusinessContextEnhancement,
    pub user_story_corrections: Vec<UserStoryCorrection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductTypeCorrection {
    pub original: String,
    pub corrected: String,
    pub rationale: String,
    pub industry: Option<String>,
    pub target_market: Option<String>,
    pub business_model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaCorrection {
    pub original_persona: String,
    pub corrected_persona: ValidatedPersona,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedPersona {
    pub role: String,
    pub description: String,
    pub primary_goals: Vec<String>,
    pub context: String,
    pub business_priority: BusinessPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityCorrection {
    pub feature_name: String,
    pub original_priority: String,
    pub corrected_priority: BusinessPriority,
    pub rationale: String,
    pub business_impact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessContextEnhancement {
    pub strategic_context: String,
    pub market_positioning: String,
    pub competitive_advantages: Vec<String>,
    pub business_goals: Vec<String>,
    pub success_metrics: Vec<BusinessMetric>,
    pub compliance_requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStoryCorrection {
    pub original_story: UserStory,
    pub corrected_story: RefinedUserStory,
    pub correction_type: CorrectionType,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefinedUserStory {
    pub id: String,
    pub title: String,
    pub description: String, // In EARS format
    pub status: FeatureStatus,
    pub business_value: BusinessValueRating,
    pub evidence: Vec<String>,
    pub acceptance_criteria: Vec<String>,
    pub implementation_gap: Option<String>,
    pub estimated_effort: Option<String>,
    pub validated_persona: ValidatedPersona,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeholderInput {
    pub role: StakeholderRole,
    pub input_timestamp: DateTime<Utc>,
    pub corrections: RefinementCorrections,
    pub feedback: String,
    pub confidence_rating: f64, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStatus {
    pub overall_score: f64,
    pub validation_checks: HashMap<String, ValidationCheck>,
    pub ready_for_generation: bool,
    pub blocking_issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCheck {
    pub passed: bool,
    pub score: f64,
    pub issues: Vec<String>,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessMetric {
    pub name: String,
    pub description: String,
    pub current_value: Option<String>,
    pub target_value: String,
    pub measurement_method: String,
    pub priority: BusinessPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefinedAnalysisResult {
    pub metadata: RefinementMetadata,
    pub business_intelligence: BusinessIntelligence,
    pub feature_status_intelligence: FeatureStatusIntelligence,
    pub technical_context: TechnicalContext,
    pub user_stories: UserStoryCollection,
    pub integration_readiness: IntegrationReadiness,
    pub original_analysis: AnalysisResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefinementMetadata {
    pub analyzer_version: String,
    pub analysis_date: DateTime<Utc>,
    pub refinement_date: DateTime<Utc>,
    pub refinement_stakeholders: Vec<StakeholderRole>,
    pub confidence_improvement: ConfidenceImprovement,
    pub refinement_session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceImprovement {
    pub business_context: (f64, f64), // (before, after)
    pub user_personas: (f64, f64),
    pub feature_priorities: (f64, f64),
    pub overall_improvement: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessIntelligence {
    pub validated_product_type: String,
    pub validated_industry: String,
    pub validated_target_market: String,
    pub validated_business_model: String,
    pub validated_personas: Vec<ValidatedPersona>,
    pub business_metrics: Vec<BusinessMetric>,
    pub success_criteria: Vec<SuccessCriterion>,
    pub strategic_context: String,
    pub market_positioning: String,
    pub compliance_requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureStatusIntelligence {
    pub completed_features: Vec<CompletedFeature>,
    pub in_progress_features: Vec<InProgressFeature>,
    pub todo_features: Vec<TodoFeature>,
    pub new_features_needed: Vec<NewFeature>,
    pub technical_debt_items: Vec<TechnicalDebtItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalContext {
    pub existing_patterns: Vec<CodePattern>,
    pub integration_points: Vec<IntegrationPoint>,
    pub code_style_requirements: Vec<StyleRequirement>,
    pub testing_strategy: TestingStrategy,
    pub implementation_constraints: Vec<Constraint>,
    pub architecture_patterns: Vec<ArchitecturePattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStoryCollection {
    pub stories: Vec<RefinedUserStory>,
    pub epics: Vec<Epic>,
    pub user_journey_maps: Vec<UserJourneyMap>,
    pub acceptance_test_scenarios: Vec<AcceptanceTestScenario>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationReadiness {
    pub ccmp_import_ready: bool,
    pub claude_spec_ready: bool,
    pub notion_integration_ready: bool,
    pub validation_score: f64,
    pub readiness_checks: HashMap<String, ValidationCheck>,
}

// Supporting enums and structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StakeholderRole {
    ProductManager,
    TechLead,
    DomainExpert,
    CustomerSuccess,
    BusinessAnalyst,
    ArchitecturalLead,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BusinessPriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BusinessValueRating {
    FiveStars,
    FourStars,
    ThreeStars,
    TwoStars,
    OneStar,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureStatus {
    Complete,
    InProgress,
    Todo,
    New,
    Deprecated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CorrectionType {
    PersonaUpdate,
    BusinessValueAdjustment,
    AcceptanceCriteriaEnhancement,
    ImplementationGapIdentification,
    StatusCorrection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriterion {
    pub criterion: String,
    pub measurement: String,
    pub target_value: String,
    pub priority: BusinessPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedFeature {
    pub name: String,
    pub description: String,
    pub evidence: Vec<String>,
    pub test_coverage: Option<f64>,
    pub user_stories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InProgressFeature {
    pub name: String,
    pub description: String,
    pub completion_percentage: f64,
    pub blocking_issues: Vec<String>,
    pub remaining_work: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoFeature {
    pub name: String,
    pub description: String,
    pub priority: BusinessPriority,
    pub estimated_effort: Option<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewFeature {
    pub name: String,
    pub description: String,
    pub business_justification: String,
    pub user_impact: String,
    pub priority: BusinessPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalDebtItem {
    pub category: String,
    pub description: String,
    pub severity: BusinessPriority,
    pub location: String,
    pub business_impact: String,
    pub recommended_resolution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePattern {
    pub pattern_type: String,
    pub description: String,
    pub examples: Vec<String>,
    pub usage_guidelines: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationPoint {
    pub service_name: String,
    pub integration_type: String,
    pub current_implementation: String,
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleRequirement {
    pub language: String,
    pub requirement: String,
    pub examples: Vec<String>,
    pub enforcement_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestingStrategy {
    pub framework: String,
    pub coverage_requirements: String,
    pub test_types: Vec<String>,
    pub quality_gates: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub constraint_type: String,
    pub description: String,
    pub impact: String,
    pub workarounds: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturePattern {
    pub pattern_name: String,
    pub description: String,
    pub usage_context: String,
    pub implementation_notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Epic {
    pub id: String,
    pub name: String,
    pub description: String,
    pub user_stories: Vec<String>,
    pub business_value: BusinessValueRating,
    pub estimated_effort: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserJourneyMap {
    pub persona: String,
    pub journey_name: String,
    pub steps: Vec<JourneyStep>,
    pub pain_points: Vec<String>,
    pub opportunities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyStep {
    pub step_name: String,
    pub description: String,
    pub user_actions: Vec<String>,
    pub system_responses: Vec<String>,
    pub touchpoints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceTestScenario {
    pub scenario_name: String,
    pub given: Vec<String>,
    pub when: Vec<String>,
    pub then: Vec<String>,
    pub user_story_id: String,
}

impl Default for BusinessPriority {
    fn default() -> Self {
        BusinessPriority::Medium
    }
}

impl Default for BusinessValueRating {
    fn default() -> Self {
        BusinessValueRating::ThreeStars
    }
}

impl Default for FeatureStatus {
    fn default() -> Self {
        FeatureStatus::Todo
    }
}

impl std::fmt::Display for BusinessValueRating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stars = match self {
            BusinessValueRating::FiveStars => "★★★★★",
            BusinessValueRating::FourStars => "★★★★☆",
            BusinessValueRating::ThreeStars => "★★★☆☆",
            BusinessValueRating::TwoStars => "★★☆☆☆",
            BusinessValueRating::OneStar => "★☆☆☆☆",
        };
        write!(f, "{}", stars)
    }
}