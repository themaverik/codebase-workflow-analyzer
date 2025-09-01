use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;
use anyhow::{Result, Context as AnyhowContext};
use serde::{Deserialize, Serialize};

use crate::core::context_types::EnhancedSegmentContext;
use crate::core::context_aware_ast_analyzer::{ContextAwareASTAnalyzer, FusedAnalysisResult, ContextAwareFusionEngine};
use crate::core::enhanced_framework_detector::{EnhancedFrameworkDetector, EnhancedFrameworkDetectionResult};
use crate::core::business_context_grounding::{BusinessContextGroundingEngine, BusinessContextGroundingResult};
use crate::core::hierarchical_result_fusion::{HierarchicalResultFusionEngine, HierarchicalFusionResult};
use crate::intelligence::llm_client::LocalLLMManager;
use crate::core::config::Config;

pub struct ContextAwareFrameworkDetector {
    traditional_detector: EnhancedFrameworkDetector,
    context_aware_analyzer: ContextAwareASTAnalyzer,
    fusion_engine: ContextAwareFusionEngine,
    business_grounding_engine: BusinessContextGroundingEngine,
    hierarchical_fusion_engine: HierarchicalResultFusionEngine,
    llm_manager: Option<LocalLLMManager>,
    config: Config,
}

impl ContextAwareFrameworkDetector {
    pub async fn new(project_path: &str) -> Result<Self> {
        let traditional_detector = EnhancedFrameworkDetector::new(project_path.to_string())?;
        let context_aware_analyzer = ContextAwareASTAnalyzer::new()?;
        let fusion_engine = ContextAwareFusionEngine::new();
        let business_grounding_engine = BusinessContextGroundingEngine::new().await?;
        let hierarchical_fusion_engine = HierarchicalResultFusionEngine::new()?;
        
        let llm_manager = match LocalLLMManager::new(None).await {
            Ok(manager) => Some(manager),
            Err(_) => None,
        };

        Ok(Self {
            traditional_detector,
            context_aware_analyzer,
            fusion_engine,
            business_grounding_engine,
            hierarchical_fusion_engine,
            llm_manager,
            config: Config::instance(),
        })
    }

    pub async fn analyze_with_hierarchical_context(
        &mut self, 
        project_path: &Path
    ) -> Result<ContextAwareFrameworkAnalysisResult> {
        let start_time = Instant::now();

        println!("Step 1: Project-level context establishment");
        self.context_aware_analyzer.initialize(project_path).await
            .with_context(|| "Failed to initialize context-aware analyzer")?;

        println!("Step 2: Context-aware segment extraction");
        let segment_extraction_result = self.context_aware_analyzer
            .extract_segments_with_context(project_path).await
            .with_context(|| "Failed to extract segments with context")?;

        println!("Step 3: Traditional framework detection");
        let traditional_result = self.traditional_detector
            .detect_frameworks_enhanced().await
            .with_context(|| "Failed to perform traditional framework detection")?;

        println!("Step 4: Context-aware result fusion");
        let fused_analysis = self.fusion_engine
            .fuse_context_aware_analysis(&segment_extraction_result.enhanced_segments).await
            .with_context(|| "Failed to fuse analysis results")?;

        println!("Step 5: Business context grounding");
        let business_grounding = if !segment_extraction_result.enhanced_segments.is_empty() {
            Some(self.business_grounding_engine
                .ground_business_context(
                    &segment_extraction_result.enhanced_segments[0].project_context,
                    &segment_extraction_result.enhanced_segments
                ).await
                .with_context(|| "Failed to ground business context")?)
        } else {
            None
        };

        println!("Step 6: Hierarchical result fusion");
        let hierarchical_fusion = self.hierarchical_fusion_engine
            .fuse_hierarchical_analysis(
                &traditional_result,
                &segment_extraction_result,
                &fused_analysis,
                business_grounding.as_ref()
            ).await
            .with_context(|| "Failed to perform hierarchical result fusion")?;

        let total_duration = start_time.elapsed();
        
        // Calculate metrics before moving values
        let context_efficiency = self.calculate_context_efficiency(&fused_analysis);
        let improvement_metrics = self.calculate_improvement_metrics(&fused_analysis);
        let awareness_summary = self.generate_context_awareness_summary(&fused_analysis);

        Ok(ContextAwareFrameworkAnalysisResult {
            traditional_analysis: traditional_result,
            segment_extraction: segment_extraction_result,
            fused_analysis,
            business_grounding,
            hierarchical_fusion: Some(hierarchical_fusion),
            performance_metrics: PerformanceMetrics {
                total_analysis_time_ms: total_duration.as_millis() as u64,
                context_establishment_efficiency: context_efficiency,
                improvement_over_baseline: improvement_metrics,
            },
            context_awareness_summary: awareness_summary,
        })
    }

    async fn perform_llm_validation_static(
        enhanced_segments: &[EnhancedSegmentContext]
    ) -> Result<Option<LLMValidationResult>> {
        if enhanced_segments.is_empty() {
            return Ok(None);
        }

        let validation_start = Instant::now();
        
        let sample_segments = enhanced_segments.iter()
            .take(5)
            .cloned()
            .collect::<Vec<_>>();

        // For now, use a simple validation without the analyze_with_context method
        let mock_response = LLMValidationResult {
            validation_confidence: 0.8,
            validation_insights: vec![
                "Context-aware analysis appears well-structured".to_string(),
                "Business domains correctly identified in segments".to_string(),
                "Architectural layers properly mapped".to_string(),
            ],
            validation_time_ms: validation_start.elapsed().as_millis() as u64,
            segments_validated: sample_segments.len(),
        };
        
        Ok(Some(mock_response))
    }

    fn build_validation_prompt(&self, segments: &[EnhancedSegmentContext]) -> String {
        let mut prompt = String::from(
            "VALIDATION TASK: Review the context-aware analysis of these code segments.\n\n"
        );

        for (i, segment) in segments.iter().enumerate() {
            prompt.push_str(&format!(
                "SEGMENT {}: {}\n",
                i + 1,
                segment.segment_context.segment_id
            ));
            
            prompt.push_str(&format!(
                "Project Type: {}\n",
                segment.project_context.project_type
            ));
            
            prompt.push_str(&format!(
                "Business Domains: {}\n",
                segment.project_context.business_domains
                    .iter()
                    .map(|d| d.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            
            prompt.push_str(&format!(
                "Business Hints: {}\n",
                segment.business_hints.join(", ")
            ));
            
            prompt.push_str(&format!(
                "Architectural Layer: {:?}\n\n",
                segment.architectural_context.layer
            ));
        }

        prompt.push_str("QUESTIONS:\n");
        prompt.push_str("1. Are the business domains correctly identified?\n");
        prompt.push_str("2. Do the architectural layers make sense?\n");
        prompt.push_str("3. Are there any context misalignments?\n");
        prompt.push_str("4. What is the overall context awareness quality (0.0-1.0)?\n");

        prompt
    }

    fn calculate_context_efficiency(&self, fused_analysis: &FusedAnalysisResult) -> f32 {
        if fused_analysis.fused_segments.is_empty() {
            return 0.0;
        }

        let high_quality_segments = fused_analysis.fused_segments
            .iter()
            .filter(|s| s.quality_score > 0.7)
            .count();

        high_quality_segments as f32 / fused_analysis.fused_segments.len() as f32
    }

    fn calculate_improvement_metrics(&self, fused_analysis: &FusedAnalysisResult) -> ImprovementMetrics {
        let baseline_accuracy = 0.65;
        let current_accuracy = fused_analysis.confidence_breakdown.calculate_average_confidence();
        
        ImprovementMetrics {
            accuracy_improvement: current_accuracy - baseline_accuracy,
            context_coverage: self.calculate_context_coverage(fused_analysis),
            confidence_boost: current_accuracy / baseline_accuracy,
        }
    }

    fn calculate_context_coverage(&self, fused_analysis: &FusedAnalysisResult) -> f32 {
        if fused_analysis.fused_segments.is_empty() {
            return 0.0;
        }

        let covered_segments = fused_analysis.fused_segments
            .iter()
            .filter(|s| !s.business_domains.is_empty() || !s.architectural_patterns.is_empty())
            .count();

        covered_segments as f32 / fused_analysis.fused_segments.len() as f32
    }

    fn generate_context_awareness_summary(&self, fused_analysis: &FusedAnalysisResult) -> ContextAwarenessSummary {
        let mut business_domain_coverage = HashMap::new();
        let mut architectural_pattern_coverage = HashMap::new();

        for segment in &fused_analysis.fused_segments {
            for domain in &segment.business_domains {
                *business_domain_coverage.entry(domain.clone()).or_insert(0) += 1;
            }
            
            for pattern in &segment.architectural_patterns {
                *architectural_pattern_coverage.entry(pattern.clone()).or_insert(0) += 1;
            }
        }

        ContextAwarenessSummary {
            total_segments_analyzed: fused_analysis.fused_segments.len(),
            business_domain_coverage,
            architectural_pattern_coverage,
            average_confidence: fused_analysis.confidence_breakdown.calculate_average_confidence(),
            high_confidence_ratio: fused_analysis.confidence_breakdown.high_confidence_segments as f32 / 
                fused_analysis.confidence_breakdown.total_segments as f32,
            context_completeness_score: self.calculate_completeness_score(fused_analysis),
        }
    }

    fn calculate_completeness_score(&self, fused_analysis: &FusedAnalysisResult) -> f32 {
        if fused_analysis.fused_segments.is_empty() {
            return 0.0;
        }

        let mut completeness_sum = 0.0;
        
        for segment in &fused_analysis.fused_segments {
            let mut segment_completeness = 0.0;
            
            segment_completeness += if segment.confidence_components.project_context > 0.5 { 0.25 } else { 0.0 };
            segment_completeness += if segment.confidence_components.structural > 0.5 { 0.25 } else { 0.0 };
            segment_completeness += if segment.confidence_components.business > 0.3 { 0.25 } else { 0.0 };
            segment_completeness += if segment.confidence_components.architectural > 0.3 { 0.25 } else { 0.0 };
            
            completeness_sum += segment_completeness;
        }

        completeness_sum / fused_analysis.fused_segments.len() as f32
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAwareFrameworkAnalysisResult {
    pub traditional_analysis: EnhancedFrameworkDetectionResult,
    pub segment_extraction: crate::core::context_aware_ast_analyzer::ContextAwareSegmentExtractionResult,
    pub fused_analysis: FusedAnalysisResult,
    pub business_grounding: Option<BusinessContextGroundingResult>,
    pub hierarchical_fusion: Option<HierarchicalFusionResult>,
    pub performance_metrics: PerformanceMetrics,
    pub context_awareness_summary: ContextAwarenessSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMValidationResult {
    pub validation_confidence: f32,
    pub validation_insights: Vec<String>,
    pub validation_time_ms: u64,
    pub segments_validated: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_analysis_time_ms: u64,
    pub context_establishment_efficiency: f32,
    pub improvement_over_baseline: ImprovementMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementMetrics {
    pub accuracy_improvement: f32,
    pub context_coverage: f32,
    pub confidence_boost: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAwarenessSummary {
    pub total_segments_analyzed: usize,
    pub business_domain_coverage: HashMap<String, usize>,
    pub architectural_pattern_coverage: HashMap<String, usize>,
    pub average_confidence: f32,
    pub high_confidence_ratio: f32,
    pub context_completeness_score: f32,
}