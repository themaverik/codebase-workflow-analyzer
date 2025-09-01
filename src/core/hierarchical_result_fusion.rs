use std::collections::HashMap;
use anyhow::Result;
use serde::{Serialize, Deserialize};

use crate::core::enhanced_framework_detector::EnhancedFrameworkDetectionResult;
use crate::core::context_aware_ast_analyzer::{FusedAnalysisResult, ContextAwareSegmentExtractionResult};
use crate::core::business_context_grounding::BusinessContextGroundingResult;
use crate::core::config::Config;
use crate::core::types::{Framework, BusinessDomain};
use crate::core::UsageExtent;

#[derive(Debug, Clone)]
pub struct HierarchicalResultFusionEngine {
    fusion_strategy: FusionStrategy,
    confidence_weights: ConfidenceWeights,
    quality_thresholds: QualityThresholds,
    config: Config,
}

impl HierarchicalResultFusionEngine {
    pub fn new() -> Result<Self> {
        Ok(Self {
            fusion_strategy: FusionStrategy::WeightedConsensus,
            confidence_weights: ConfidenceWeights::default(),
            quality_thresholds: QualityThresholds::default(),
            config: Config::instance(),
        })
    }

    pub fn with_strategy(mut self, strategy: FusionStrategy) -> Self {
        self.fusion_strategy = strategy;
        self
    }

    pub fn with_weights(mut self, weights: ConfidenceWeights) -> Self {
        self.confidence_weights = weights;
        self
    }

    pub async fn fuse_hierarchical_analysis(
        &self,
        traditional_analysis: &EnhancedFrameworkDetectionResult,
        segment_extraction: &ContextAwareSegmentExtractionResult,
        fused_analysis: &FusedAnalysisResult,
        business_grounding: Option<&BusinessContextGroundingResult>,
    ) -> Result<HierarchicalFusionResult> {
        let start_time = std::time::Instant::now();

        // Starting hierarchical result fusion

        // Step 1: Extract tier-specific insights
        // Extracting tier-specific insights
        let tier1_insights = self.extract_tier1_insights(traditional_analysis);
        let tier2_insights = self.extract_tier2_insights(segment_extraction, fused_analysis);
        let tier3_insights = self.extract_tier3_insights(business_grounding);

        // Step 2: Perform confidence-weighted fusion
        // Performing confidence-weighted fusion
        let fusion_analysis = self.perform_weighted_fusion(
            &tier1_insights,
            &tier2_insights,
            &tier3_insights,
        )?;

        // Step 3: Apply consensus-based validation
        // Applying consensus-based validation
        let validated_results = self.apply_consensus_validation(&fusion_analysis)?;

        // Step 4: Generate final consolidated results
        // Generating final consolidated results
        let consolidated_results = self.generate_consolidated_results(
            &validated_results,
            &tier1_insights,
            &tier2_insights,
            &tier3_insights,
        )?;

        // Step 5: Calculate fusion quality metrics
        // Calculating fusion quality metrics
        let quality_metrics = self.calculate_fusion_quality_metrics(
            &consolidated_results,
            &tier1_insights,
            &tier2_insights,
            &tier3_insights,
        );

        let fusion_time = start_time.elapsed();

        Ok(HierarchicalFusionResult {
            consolidated_results,
            tier1_insights,
            tier2_insights,
            tier3_insights,
            fusion_analysis,
            validated_results,
            quality_metrics,
            fusion_metadata: FusionMetadata {
                total_fusion_time_ms: fusion_time.as_millis() as u64,
                strategy_used: self.fusion_strategy.clone(),
                confidence_weights: self.confidence_weights.clone(),
                tiers_processed: self.count_active_tiers(business_grounding),
            },
        })
    }

    fn extract_tier1_insights(&self, traditional_analysis: &EnhancedFrameworkDetectionResult) -> Tier1Insights {
        let mut framework_confidence = HashMap::new();
        let mut detected_frameworks = Vec::new();

        // Extract framework detection insights
        for framework in &traditional_analysis.detected_frameworks {
            framework_confidence.insert(framework.framework.clone(), framework.confidence);
            detected_frameworks.push(FusedFrameworkDetection {
                framework: framework.framework.clone(),
                confidence: framework.confidence,
                evidence_strength: framework.evidence.len() as f32 / 10.0, // Normalize evidence count
                usage_extent: framework.usage_extent.clone(),
            });
        }

        let primary_ecosystem = traditional_analysis.primary_ecosystem.clone();
        
        // Calculate overall tier confidence
        let tier_confidence = if !framework_confidence.is_empty() {
            framework_confidence.values().sum::<f32>() / framework_confidence.len() as f32
        } else {
            0.5
        };

        Tier1Insights {
            primary_ecosystem,
            detected_frameworks,
            framework_confidence,
            architecture_patterns: self.extract_architecture_patterns(traditional_analysis),
            tier_confidence,
            evidence_quality: self.assess_tier1_evidence_quality(traditional_analysis),
        }
    }

    fn extract_tier2_insights(
        &self, 
        segment_extraction: &ContextAwareSegmentExtractionResult,
        fused_analysis: &FusedAnalysisResult
    ) -> Tier2Insights {
        let mut segment_qualities = HashMap::new();
        let mut context_coverage = HashMap::new();
        let mut architectural_layers = HashMap::new();

        // Extract segment-level insights
        for segment in &segment_extraction.enhanced_segments {
            segment_qualities.insert(
                segment.segment_context.segment_id.clone(),
                segment.segment_context.confidence
            );

            // Track architectural layers
            let layer = format!("{:?}", segment.architectural_context.layer);
            *architectural_layers.entry(layer).or_insert(0) += 1;

            // Track context coverage
            if !segment.business_hints.is_empty() {
                *context_coverage.entry("business_context".to_string()).or_insert(0) += 1;
            }
            if !segment.cross_references.is_empty() {
                *context_coverage.entry("cross_references".to_string()).or_insert(0) += 1;
            }
        }

        // Extract fused analysis insights
        let mut fused_segments = Vec::new();
        for fused_segment in &fused_analysis.fused_segments {
            fused_segments.push(FusedSegmentInsight {
                segment_id: fused_segment.segment_id.clone(),
                fused_confidence: fused_segment.fused_confidence,
                quality_score: fused_segment.quality_score,
                business_domains: fused_segment.business_domains.clone(),
                architectural_patterns: fused_segment.architectural_patterns.clone(),
            });
        }

        let tier_confidence = fused_analysis.confidence_breakdown.calculate_average_confidence();

        Tier2Insights {
            total_segments_processed: segment_extraction.enhanced_segments.len(),
            context_awareness_score: segment_extraction.extraction_metadata.context_awareness_score,
            segment_qualities,
            context_coverage,
            architectural_layers,
            fused_segments,
            tier_confidence,
            fusion_efficiency: fused_analysis.fusion_metadata.segments_processed as f32 / 
                             (fused_analysis.fusion_metadata.fusion_time_ms as f32 / 1000.0),
        }
    }

    fn extract_tier3_insights(&self, business_grounding: Option<&BusinessContextGroundingResult>) -> Option<Tier3Insights> {
        business_grounding.map(|grounding| {
            let mut domain_confidence = HashMap::new();
            let mut business_capabilities = Vec::new();
            let mut implementation_roadmap = Vec::new();

            // Extract grounded business context
            for domain in &grounding.grounded_context.final_domains {
                domain_confidence.insert(domain.name.clone(), domain.confidence);
            }

            business_capabilities = grounding.grounded_context.business_capabilities.clone();
            
            for step in &grounding.grounded_context.implementation_roadmap {
                implementation_roadmap.push(FusedImplementationStep {
                    step_number: step.step_number,
                    domain_focus: step.domain_focus.clone(),
                    business_value: step.business_value,
                    estimated_effort: step.estimated_effort.clone(),
                });
            }

            let tier_confidence = grounding.grounded_context.overall_confidence;

            Tier3Insights {
                grounded_domains: grounding.grounded_context.final_domains.clone(),
                domain_confidence,
                business_capabilities,
                cross_domain_relationships: grounding.grounded_context.cross_domain_relationships.clone(),
                implementation_roadmap,
                tier_confidence,
                validation_quality: grounding.validation_result.overall_confidence,
                evidence_strength: grounding.evidence_summary.evidence_quality_score,
            }
        })
    }

    fn perform_weighted_fusion(
        &self,
        tier1: &Tier1Insights,
        tier2: &Tier2Insights,
        tier3: &Option<Tier3Insights>,
    ) -> Result<WeightedFusionAnalysis> {
        let mut framework_consensus = HashMap::new();
        let mut business_domain_consensus = HashMap::new();
        let mut architectural_consensus = HashMap::new();
        let mut overall_confidence_scores = Vec::new();

        // Fuse framework detection results
        for framework_detection in &tier1.detected_frameworks {
            let tier1_weight = self.confidence_weights.tier1_traditional * framework_detection.confidence;
            let mut total_weight = tier1_weight;
            let mut weighted_confidence = tier1_weight;

            // Check for tier2 segment corroboration
            let tier2_support = tier2.fused_segments.iter()
                .filter(|seg| seg.architectural_patterns.iter()
                    .any(|pattern| pattern.to_lowercase().contains(&framework_detection.framework.to_string().to_lowercase())))
                .count() as f32 / tier2.fused_segments.len() as f32;

            if tier2_support > 0.0 {
                let tier2_weight = self.confidence_weights.tier2_context_aware * tier2_support;
                weighted_confidence += tier2_weight;
                total_weight += self.confidence_weights.tier2_context_aware;
            }

            // Check for tier3 business domain alignment
            if let Some(tier3_data) = tier3 {
                let business_alignment = self.calculate_framework_business_alignment(
                    &framework_detection.framework, 
                    tier3_data
                );
                if business_alignment > 0.0 {
                    let tier3_weight = self.confidence_weights.tier3_business_grounding * business_alignment;
                    weighted_confidence += tier3_weight;
                    total_weight += self.confidence_weights.tier3_business_grounding;
                }
            }

            let final_confidence = if total_weight > 0.0 {
                weighted_confidence / total_weight
            } else {
                framework_detection.confidence
            };

            framework_consensus.insert(
                framework_detection.framework.clone(),
                FrameworkConsensus {
                    framework: framework_detection.framework.clone(),
                    tier1_confidence: framework_detection.confidence,
                    tier2_support: tier2_support,
                    tier3_alignment: tier3.as_ref().map(|t3| self.calculate_framework_business_alignment(&framework_detection.framework, t3)).unwrap_or(0.0),
                    weighted_confidence: final_confidence,
                    evidence_strength: framework_detection.evidence_strength,
                }
            );
        }

        // Fuse business domain results
        if let Some(tier3_data) = tier3 {
            for domain in &tier3_data.grounded_domains {
                let mut tier2_corroboration = 0.0;
                
                // Check tier2 segment support for business domain
                for segment in &tier2.fused_segments {
                    if segment.business_domains.contains(&domain.name) {
                        tier2_corroboration += segment.fused_confidence;
                    }
                }
                tier2_corroboration = (tier2_corroboration / tier2.fused_segments.len() as f32).min(1.0);

                let weighted_confidence = (
                    domain.confidence * self.confidence_weights.tier3_business_grounding +
                    tier2_corroboration * self.confidence_weights.tier2_context_aware
                ) / (self.confidence_weights.tier3_business_grounding + 
                     if tier2_corroboration > 0.0 { self.confidence_weights.tier2_context_aware } else { 0.0 });

                business_domain_consensus.insert(
                    domain.name.clone(),
                    BusinessDomainConsensus {
                        domain: BusinessDomain {
                            name: domain.name.clone(),
                            confidence: weighted_confidence,
                        },
                        tier2_corroboration,
                        tier3_confidence: domain.confidence,
                        weighted_confidence,
                        implementation_status: domain.implementation_status.clone(),
                    }
                );
            }
        }

        // Calculate overall confidence scores
        overall_confidence_scores.push(tier1.tier_confidence * self.confidence_weights.tier1_traditional);
        overall_confidence_scores.push(tier2.tier_confidence * self.confidence_weights.tier2_context_aware);
        if let Some(tier3_data) = tier3 {
            overall_confidence_scores.push(tier3_data.tier_confidence * self.confidence_weights.tier3_business_grounding);
        }

        let fusion_confidence = overall_confidence_scores.iter().sum::<f32>() / overall_confidence_scores.len() as f32;

        Ok(WeightedFusionAnalysis {
            framework_consensus,
            business_domain_consensus,
            architectural_consensus,
            fusion_confidence,
            tier_contributions: TierContributions {
                tier1_weight: self.confidence_weights.tier1_traditional,
                tier2_weight: self.confidence_weights.tier2_context_aware,
                tier3_weight: self.confidence_weights.tier3_business_grounding,
                total_active_tiers: if tier3.is_some() { 3 } else { 2 },
            },
        })
    }

    fn apply_consensus_validation(&self, fusion_analysis: &WeightedFusionAnalysis) -> Result<ConsensusValidatedResults> {
        let mut validated_frameworks = Vec::new();
        let mut validated_domains = Vec::new();
        let mut validation_issues = Vec::new();

        // Validate framework consensus
        for (framework, consensus) in &fusion_analysis.framework_consensus {
            if consensus.weighted_confidence >= self.quality_thresholds.min_framework_confidence {
                // Check for multi-tier agreement
                let tier_agreement_score = self.calculate_tier_agreement_score(
                    consensus.tier1_confidence,
                    consensus.tier2_support,
                    consensus.tier3_alignment,
                );

                if tier_agreement_score >= self.quality_thresholds.min_tier_agreement {
                    validated_frameworks.push(ValidatedFramework {
                        framework: framework.clone(),
                        consensus_confidence: consensus.weighted_confidence,
                        tier_agreement_score,
                        validation_status: ValidationStatus::Validated,
                        supporting_evidence: self.gather_framework_evidence(consensus),
                    });
                } else {
                    validation_issues.push(format!(
                        "Framework {} has insufficient tier agreement (score: {:.3})", 
                        framework, tier_agreement_score
                    ));
                }
            } else {
                validation_issues.push(format!(
                    "Framework {} below confidence threshold (confidence: {:.3})", 
                    framework, consensus.weighted_confidence
                ));
            }
        }

        // Validate business domain consensus
        for (domain_name, consensus) in &fusion_analysis.business_domain_consensus {
            if consensus.weighted_confidence >= self.quality_thresholds.min_domain_confidence {
                validated_domains.push(ValidatedBusinessDomain {
                    domain: consensus.domain.clone(),
                    consensus_confidence: consensus.weighted_confidence,
                    tier2_support: consensus.tier2_corroboration,
                    tier3_support: consensus.tier3_confidence,
                    validation_status: ValidationStatus::Validated,
                    implementation_status: consensus.implementation_status.clone(),
                });
            } else {
                validation_issues.push(format!(
                    "Business domain {} below confidence threshold (confidence: {:.3})", 
                    domain_name, consensus.weighted_confidence
                ));
            }
        }

        // Calculate overall validation quality
        let validation_quality = self.calculate_validation_quality(&validated_frameworks, &validated_domains);

        Ok(ConsensusValidatedResults {
            validated_frameworks,
            validated_domains,
            validation_issues,
            validation_quality,
            consensus_strength: fusion_analysis.fusion_confidence,
        })
    }

    fn generate_consolidated_results(
        &self,
        validated_results: &ConsensusValidatedResults,
        tier1: &Tier1Insights,
        tier2: &Tier2Insights,
        tier3: &Option<Tier3Insights>,
    ) -> Result<ConsolidatedAnalysisResults> {
        // Primary framework determination
        let primary_framework = validated_results.validated_frameworks
            .iter()
            .max_by(|a, b| a.consensus_confidence.partial_cmp(&b.consensus_confidence).unwrap())
            .map(|vf| vf.framework.clone());

        // Primary business domain determination
        let primary_business_domain = validated_results.validated_domains
            .iter()
            .max_by(|a, b| a.consensus_confidence.partial_cmp(&b.consensus_confidence).unwrap())
            .map(|vd| vd.domain.clone());

        // Architecture assessment
        let architecture_assessment = self.generate_architecture_assessment(tier1, tier2);

        // Implementation readiness
        let implementation_readiness = self.assess_implementation_readiness(
            &validated_results,
            tier2,
            tier3.as_ref(),
        );

        // Business value assessment
        let business_value_assessment = tier3.as_ref().map(|t3| {
            BusinessValueAssessment {
                overall_business_value: t3.implementation_roadmap.iter()
                    .map(|step| step.business_value)
                    .fold(0.0, |acc, val| acc + val) / t3.implementation_roadmap.len().max(1) as f32,
                high_value_capabilities: t3.business_capabilities.iter()
                    .filter(|cap| self.is_high_value_capability(cap))
                    .cloned()
                    .collect(),
                implementation_priority_score: self.calculate_implementation_priority(t3),
            }
        });

        // Quality assurance metrics
        let quality_assurance = QualityAssuranceMetrics {
            framework_detection_accuracy: self.estimate_framework_accuracy(&validated_results.validated_frameworks),
            business_domain_accuracy: self.estimate_domain_accuracy(&validated_results.validated_domains),
            cross_tier_consistency: self.calculate_cross_tier_consistency(tier1, tier2, tier3.as_ref()),
            overall_analysis_reliability: validated_results.validation_quality,
        };

        Ok(ConsolidatedAnalysisResults {
            primary_framework: primary_framework.clone(),
            secondary_frameworks: validated_results.validated_frameworks.iter()
                .filter(|vf| Some(&vf.framework) != primary_framework.as_ref())
                .map(|vf| vf.framework.clone())
                .collect(),
            primary_business_domain: primary_business_domain.clone(),
            secondary_business_domains: validated_results.validated_domains.iter()
                .filter(|vd| Some(&vd.domain) != primary_business_domain.as_ref())
                .map(|vd| vd.domain.clone())
                .collect(),
            architecture_assessment,
            implementation_readiness,
            business_value_assessment,
            quality_assurance,
            confidence_summary: ConsolidatedConfidenceSummary {
                overall_confidence: validated_results.consensus_strength,
                framework_confidence: primary_framework.as_ref()
                    .and_then(|pf| validated_results.validated_frameworks.iter()
                        .find(|vf| &vf.framework == pf)
                        .map(|vf| vf.consensus_confidence))
                    .unwrap_or(0.0),
                business_domain_confidence: primary_business_domain.as_ref()
                    .and_then(|pd| validated_results.validated_domains.iter()
                        .find(|vd| &vd.domain == pd)
                        .map(|vd| vd.consensus_confidence))
                    .unwrap_or(0.0),
                tier_coverage_completeness: self.calculate_tier_coverage_completeness(tier1, tier2, tier3.as_ref()),
            },
        })
    }

    fn calculate_fusion_quality_metrics(
        &self,
        consolidated_results: &ConsolidatedAnalysisResults,
        tier1: &Tier1Insights,
        tier2: &Tier2Insights,
        tier3: &Option<Tier3Insights>,
    ) -> FusionQualityMetrics {
        let tier_alignment_score = self.calculate_tier_alignment_score(tier1, tier2, tier3.as_ref());
        let confidence_distribution = self.analyze_confidence_distribution(tier1, tier2, tier3.as_ref());
        let consensus_strength = consolidated_results.confidence_summary.overall_confidence;
        let result_completeness = self.calculate_result_completeness(consolidated_results);

        let overall_fusion_quality = (
            tier_alignment_score * 0.3 +
            confidence_distribution * 0.25 +
            consensus_strength * 0.25 +
            result_completeness * 0.2
        );

        FusionQualityMetrics {
            overall_fusion_quality,
            tier_alignment_score,
            confidence_distribution,
            consensus_strength,
            result_completeness,
            improvement_over_single_tier: self.calculate_improvement_over_baseline(consolidated_results, tier1),
        }
    }

    // Helper methods implementation
    fn extract_architecture_patterns(&self, traditional_analysis: &EnhancedFrameworkDetectionResult) -> Vec<String> {
        let mut patterns = Vec::new();
        
        // Extract patterns based on detected frameworks
        for framework in &traditional_analysis.detected_frameworks {
            match framework.framework.to_string().as_str() {
                "React" => patterns.push("Component-Based Architecture".to_string()),
                "NestJS" => patterns.push("Decorator-Based Architecture".to_string()),
                "Spring Boot" => patterns.push("Dependency Injection".to_string()),
                "Django" => patterns.push("MTV Pattern".to_string()),
                "FastAPI" => patterns.push("ASGI Architecture".to_string()),
                _ => patterns.push("Framework-Specific Pattern".to_string()),
            }
        }

        patterns
    }

    fn assess_tier1_evidence_quality(&self, traditional_analysis: &EnhancedFrameworkDetectionResult) -> f32 {
        if traditional_analysis.detected_frameworks.is_empty() {
            return 0.0;
        }

        let total_evidence = traditional_analysis.detected_frameworks.iter()
            .map(|f| f.evidence.len())
            .sum::<usize>();

        (total_evidence as f32 / traditional_analysis.detected_frameworks.len() as f32 / 10.0).min(1.0)
    }

    fn calculate_framework_business_alignment(&self, framework: &Framework, tier3_data: &Tier3Insights) -> f32 {
        // Simple heuristic for framework-business alignment
        let framework_str = framework.to_string().to_lowercase();
        let mut alignment_score: f32 = 0.0;

        for domain in &tier3_data.grounded_domains {
            let domain_lower = domain.name.to_lowercase();
            match framework_str.as_str() {
                name if name.contains("react") || name.contains("next") => {
                    if domain_lower.contains("web") || domain_lower.contains("frontend") || domain_lower.contains("ui") {
                        alignment_score += 0.3;
                    }
                },
                name if name.contains("spring") || name.contains("nest") => {
                    if domain_lower.contains("api") || domain_lower.contains("service") || domain_lower.contains("backend") {
                        alignment_score += 0.3;
                    }
                },
                name if name.contains("django") || name.contains("flask") || name.contains("fastapi") => {
                    if domain_lower.contains("web") || domain_lower.contains("api") || domain_lower.contains("service") {
                        alignment_score += 0.3;
                    }
                },
                _ => alignment_score += 0.1,
            }
        }

        alignment_score.min(1.0)
    }

    fn calculate_tier_agreement_score(&self, tier1_conf: f32, tier2_support: f32, tier3_alignment: f32) -> f32 {
        let weights = [0.4, 0.4, 0.2]; // Tier weights for agreement calculation
        let scores = [tier1_conf, tier2_support, tier3_alignment];
        
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;
        
        for (i, &score) in scores.iter().enumerate() {
            if score > 0.0 {
                weighted_sum += score * weights[i];
                total_weight += weights[i];
            }
        }
        
        if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        }
    }

    fn gather_framework_evidence(&self, consensus: &FrameworkConsensus) -> Vec<String> {
        let mut evidence = Vec::new();
        
        if consensus.tier1_confidence > 0.5 {
            evidence.push(format!("Traditional detection confidence: {:.2}", consensus.tier1_confidence));
        }
        if consensus.tier2_support > 0.3 {
            evidence.push(format!("Context-aware segment support: {:.2}", consensus.tier2_support));
        }
        if consensus.tier3_alignment > 0.3 {
            evidence.push(format!("Business domain alignment: {:.2}", consensus.tier3_alignment));
        }
        
        evidence
    }

    fn calculate_validation_quality(&self, frameworks: &[ValidatedFramework], domains: &[ValidatedBusinessDomain]) -> f32 {
        let framework_quality = if !frameworks.is_empty() {
            frameworks.iter().map(|f| f.consensus_confidence).sum::<f32>() / frameworks.len() as f32
        } else {
            0.0
        };

        let domain_quality = if !domains.is_empty() {
            domains.iter().map(|d| d.consensus_confidence).sum::<f32>() / domains.len() as f32
        } else {
            0.0
        };

        (framework_quality + domain_quality) / 2.0
    }

    fn generate_architecture_assessment(&self, tier1: &Tier1Insights, tier2: &Tier2Insights) -> ArchitectureAssessment {
        let mut architectural_patterns = tier1.architecture_patterns.clone();
        
        // Add patterns from tier2 analysis
        for (layer, count) in &tier2.architectural_layers {
            if *count > 0 {
                architectural_patterns.push(format!("{} Layer Implementation", layer));
            }
        }

        let complexity_score = self.calculate_architecture_complexity_score(tier2);
        let maintainability_score = self.calculate_maintainability_score(tier1, tier2);

        ArchitectureAssessment {
            architectural_patterns,
            complexity_score,
            maintainability_score,
            scalability_indicators: self.identify_scalability_indicators(tier1, tier2),
        }
    }

    fn assess_implementation_readiness(
        &self,
        validated_results: &ConsensusValidatedResults,
        tier2: &Tier2Insights,
        tier3: Option<&Tier3Insights>,
    ) -> ImplementationReadinessAssessment {
        let framework_readiness = if !validated_results.validated_frameworks.is_empty() {
            validated_results.validated_frameworks.iter()
                .map(|f| f.consensus_confidence)
                .sum::<f32>() / validated_results.validated_frameworks.len() as f32
        } else {
            0.0
        };

        let context_completeness = tier2.context_awareness_score;

        let business_alignment = tier3.map(|t3| t3.tier_confidence).unwrap_or(0.5);

        let overall_readiness = (framework_readiness * 0.4 + context_completeness * 0.3 + business_alignment * 0.3);

        ImplementationReadinessAssessment {
            overall_readiness,
            framework_readiness,
            context_completeness,
            business_alignment,
            recommended_next_steps: self.generate_next_steps(overall_readiness),
        }
    }

    fn calculate_architecture_complexity_score(&self, tier2: &Tier2Insights) -> f32 {
        // Base complexity on number of architectural layers and segments
        let layer_diversity = tier2.architectural_layers.len() as f32 / 5.0; // Normalize to max 5 layers
        let segment_complexity = (tier2.total_segments_processed as f32 / 100.0).min(1.0); // Cap at 100 segments
        
        (layer_diversity + segment_complexity) / 2.0
    }

    fn calculate_maintainability_score(&self, tier1: &Tier1Insights, tier2: &Tier2Insights) -> f32 {
        // Higher maintainability with clear patterns and good context awareness
        let pattern_clarity = if !tier1.architecture_patterns.is_empty() {
            0.8 // Having clear patterns improves maintainability
        } else {
            0.4
        };

        let context_quality = tier2.context_awareness_score;

        (pattern_clarity + context_quality) / 2.0
    }

    fn identify_scalability_indicators(&self, tier1: &Tier1Insights, tier2: &Tier2Insights) -> Vec<String> {
        let mut indicators = Vec::new();

        // Check for scalable frameworks
        for framework in &tier1.detected_frameworks {
            match framework.framework.to_string().as_str() {
                "React" | "Next.js" => indicators.push("Client-side scalability with component architecture".to_string()),
                "NestJS" | "Spring Boot" => indicators.push("Server-side scalability with microservices support".to_string()),
                "FastAPI" => indicators.push("High-performance async API scalability".to_string()),
                _ => {}
            }
        }

        // Check architectural layer distribution
        if tier2.architectural_layers.len() >= 3 {
            indicators.push("Multi-layer architecture supports horizontal scaling".to_string());
        }

        if indicators.is_empty() {
            indicators.push("Standard scalability patterns detected".to_string());
        }

        indicators
    }

    fn is_high_value_capability(&self, capability: &str) -> bool {
        let high_value_keywords = ["payment", "user", "auth", "api", "data", "analytics", "security"];
        let capability_lower = capability.to_lowercase();
        high_value_keywords.iter().any(|&keyword| capability_lower.contains(keyword))
    }

    fn calculate_implementation_priority(&self, tier3: &Tier3Insights) -> f32 {
        tier3.implementation_roadmap.iter()
            .map(|step| step.business_value)
            .fold(0.0, |acc, val| acc + val) / tier3.implementation_roadmap.len().max(1) as f32
    }

    fn estimate_framework_accuracy(&self, frameworks: &[ValidatedFramework]) -> f32 {
        if frameworks.is_empty() {
            return 0.0;
        }

        frameworks.iter().map(|f| f.consensus_confidence).sum::<f32>() / frameworks.len() as f32
    }

    fn estimate_domain_accuracy(&self, domains: &[ValidatedBusinessDomain]) -> f32 {
        if domains.is_empty() {
            return 0.0;
        }

        domains.iter().map(|d| d.consensus_confidence).sum::<f32>() / domains.len() as f32
    }

    fn calculate_cross_tier_consistency(&self, tier1: &Tier1Insights, tier2: &Tier2Insights, tier3: Option<&Tier3Insights>) -> f32 {
        let mut consistency_scores = Vec::new();

        // Tier1-Tier2 consistency
        let t1_t2_consistency = self.calculate_pairwise_consistency(tier1.tier_confidence, tier2.tier_confidence);
        consistency_scores.push(t1_t2_consistency);

        // Include Tier3 if available
        if let Some(t3) = tier3 {
            let t1_t3_consistency = self.calculate_pairwise_consistency(tier1.tier_confidence, t3.tier_confidence);
            let t2_t3_consistency = self.calculate_pairwise_consistency(tier2.tier_confidence, t3.tier_confidence);
            consistency_scores.push(t1_t3_consistency);
            consistency_scores.push(t2_t3_consistency);
        }

        consistency_scores.iter().sum::<f32>() / consistency_scores.len() as f32
    }

    fn calculate_pairwise_consistency(&self, conf1: f32, conf2: f32) -> f32 {
        // Consistency is higher when confidences are similar
        1.0 - (conf1 - conf2).abs()
    }

    fn calculate_tier_coverage_completeness(&self, tier1: &Tier1Insights, tier2: &Tier2Insights, tier3: Option<&Tier3Insights>) -> f32 {
        let mut coverage_score: f32 = 0.0;
        
        // Tier1 coverage
        if tier1.tier_confidence > 0.5 {
            coverage_score += 0.33;
        }
        
        // Tier2 coverage
        if tier2.context_awareness_score > 0.5 {
            coverage_score += 0.33;
        }
        
        // Tier3 coverage (if available)
        if let Some(t3) = tier3 {
            if t3.tier_confidence > 0.5 {
                coverage_score += 0.34;
            }
        } else {
            // Adjust scoring for 2-tier analysis
            coverage_score = coverage_score * 1.5; // Scale up to full range
        }

        coverage_score.min(1.0)
    }

    fn calculate_tier_alignment_score(&self, tier1: &Tier1Insights, tier2: &Tier2Insights, tier3: Option<&Tier3Insights>) -> f32 {
        // Measure how well the tiers align in their conclusions
        let mut alignment_scores = Vec::new();

        // Check framework-context alignment
        let framework_context_alignment = self.assess_framework_context_alignment(tier1, tier2);
        alignment_scores.push(framework_context_alignment);

        // Check business-context alignment if tier3 available
        if let Some(t3) = tier3 {
            let business_context_alignment = self.assess_business_context_alignment(tier2, t3);
            alignment_scores.push(business_context_alignment);
        }

        alignment_scores.iter().sum::<f32>() / alignment_scores.len() as f32
    }

    fn assess_framework_context_alignment(&self, tier1: &Tier1Insights, tier2: &Tier2Insights) -> f32 {
        // Simple heuristic: if tier1 detects frameworks and tier2 has good context awareness, alignment is higher
        let framework_strength = tier1.tier_confidence;
        let context_strength = tier2.context_awareness_score;
        
        // Alignment is better when both are strong
        (framework_strength * context_strength).sqrt()
    }

    fn assess_business_context_alignment(&self, tier2: &Tier2Insights, tier3: &Tier3Insights) -> f32 {
        // Check if tier2 segments align with tier3 business domains
        let mut alignment_count = 0;
        let mut total_segments = 0;

        for segment in &tier2.fused_segments {
            total_segments += 1;
            for domain in &tier3.grounded_domains {
                if segment.business_domains.contains(&domain.name) {
                    alignment_count += 1;
                    break; // Count each segment once
                }
            }
        }

        if total_segments > 0 {
            alignment_count as f32 / total_segments as f32
        } else {
            0.5
        }
    }

    fn analyze_confidence_distribution(&self, tier1: &Tier1Insights, tier2: &Tier2Insights, tier3: Option<&Tier3Insights>) -> f32 {
        let mut confidences = vec![tier1.tier_confidence, tier2.tier_confidence];
        if let Some(t3) = tier3 {
            confidences.push(t3.tier_confidence);
        }

        // A good distribution has reasonable spread but not too much variance
        let mean = confidences.iter().sum::<f32>() / confidences.len() as f32;
        let variance = confidences.iter()
            .map(|&c| (c - mean).powi(2))
            .sum::<f32>() / confidences.len() as f32;

        // Good distribution: high mean, low variance
        let distribution_quality = mean * (1.0 - variance.sqrt());
        distribution_quality.max(0.0).min(1.0)
    }

    fn calculate_result_completeness(&self, consolidated_results: &ConsolidatedAnalysisResults) -> f32 {
        let mut completeness_score = 0.0;
        
        // Framework completeness
        if consolidated_results.primary_framework.is_some() {
            completeness_score += 0.3;
        }
        if !consolidated_results.secondary_frameworks.is_empty() {
            completeness_score += 0.1;
        }

        // Business domain completeness
        if consolidated_results.primary_business_domain.is_some() {
            completeness_score += 0.3;
        }
        if !consolidated_results.secondary_business_domains.is_empty() {
            completeness_score += 0.1;
        }

        // Architecture assessment completeness
        if !consolidated_results.architecture_assessment.architectural_patterns.is_empty() {
            completeness_score += 0.1;
        }

        // Implementation readiness completeness
        if consolidated_results.implementation_readiness.overall_readiness > 0.5 {
            completeness_score += 0.1;
        }

        completeness_score
    }

    fn calculate_improvement_over_baseline(&self, consolidated_results: &ConsolidatedAnalysisResults, tier1: &Tier1Insights) -> f32 {
        // Compare consolidated confidence vs tier1 only confidence
        let baseline_confidence = tier1.tier_confidence;
        let consolidated_confidence = consolidated_results.confidence_summary.overall_confidence;
        
        if baseline_confidence > 0.0 {
            ((consolidated_confidence - baseline_confidence) / baseline_confidence).max(0.0)
        } else {
            consolidated_confidence
        }
    }

    fn generate_next_steps(&self, readiness_score: f32) -> Vec<String> {
        if readiness_score >= 0.8 {
            vec!["Project is ready for implementation".to_string()]
        } else if readiness_score >= 0.6 {
            vec![
                "Address remaining framework configuration".to_string(),
                "Finalize business requirements".to_string(),
            ]
        } else {
            vec![
                "Improve framework detection confidence".to_string(),
                "Enhance business context understanding".to_string(),
                "Validate architectural decisions".to_string(),
            ]
        }
    }

    fn count_active_tiers(&self, business_grounding: Option<&BusinessContextGroundingResult>) -> usize {
        if business_grounding.is_some() { 3 } else { 2 }
    }
}

// Data structures for hierarchical fusion

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FusionStrategy {
    WeightedConsensus,
    MajorityVote,
    HighestConfidence,
    EvidenceBased,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceWeights {
    pub tier1_traditional: f32,
    pub tier2_context_aware: f32,
    pub tier3_business_grounding: f32,
}

impl Default for ConfidenceWeights {
    fn default() -> Self {
        Self {
            tier1_traditional: 0.3,
            tier2_context_aware: 0.4,
            tier3_business_grounding: 0.3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholds {
    pub min_framework_confidence: f32,
    pub min_domain_confidence: f32,
    pub min_tier_agreement: f32,
}

impl Default for QualityThresholds {
    fn default() -> Self {
        Self {
            min_framework_confidence: 0.6,
            min_domain_confidence: 0.5,
            min_tier_agreement: 0.4,
        }
    }
}

// Tier-specific insight structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tier1Insights {
    pub primary_ecosystem: crate::core::types::LanguageEcosystem,
    pub detected_frameworks: Vec<FusedFrameworkDetection>,
    pub framework_confidence: HashMap<Framework, f32>,
    pub architecture_patterns: Vec<String>,
    pub tier_confidence: f32,
    pub evidence_quality: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tier2Insights {
    pub total_segments_processed: usize,
    pub context_awareness_score: f32,
    pub segment_qualities: HashMap<String, f32>,
    pub context_coverage: HashMap<String, usize>,
    pub architectural_layers: HashMap<String, usize>,
    pub fused_segments: Vec<FusedSegmentInsight>,
    pub tier_confidence: f32,
    pub fusion_efficiency: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tier3Insights {
    pub grounded_domains: Vec<crate::core::business_context_grounding::GroundedDomain>,
    pub domain_confidence: HashMap<String, f32>,
    pub business_capabilities: Vec<String>,
    pub cross_domain_relationships: Vec<crate::core::business_context_grounding::DomainRelationship>,
    pub implementation_roadmap: Vec<FusedImplementationStep>,
    pub tier_confidence: f32,
    pub validation_quality: f32,
    pub evidence_strength: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusedFrameworkDetection {
    pub framework: Framework,
    pub confidence: f32,
    pub evidence_strength: f32,
    pub usage_extent: UsageExtent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusedSegmentInsight {
    pub segment_id: String,
    pub fused_confidence: f32,
    pub quality_score: f32,
    pub business_domains: Vec<String>,
    pub architectural_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusedImplementationStep {
    pub step_number: usize,
    pub domain_focus: String,
    pub business_value: f32,
    pub estimated_effort: String,
}

// Fusion analysis structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightedFusionAnalysis {
    pub framework_consensus: HashMap<Framework, FrameworkConsensus>,
    pub business_domain_consensus: HashMap<String, BusinessDomainConsensus>,
    pub architectural_consensus: HashMap<String, f32>,
    pub fusion_confidence: f32,
    pub tier_contributions: TierContributions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkConsensus {
    pub framework: Framework,
    pub tier1_confidence: f32,
    pub tier2_support: f32,
    pub tier3_alignment: f32,
    pub weighted_confidence: f32,
    pub evidence_strength: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessDomainConsensus {
    pub domain: BusinessDomain,
    pub tier2_corroboration: f32,
    pub tier3_confidence: f32,
    pub weighted_confidence: f32,
    pub implementation_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierContributions {
    pub tier1_weight: f32,
    pub tier2_weight: f32,
    pub tier3_weight: f32,
    pub total_active_tiers: usize,
}

// Validation structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusValidatedResults {
    pub validated_frameworks: Vec<ValidatedFramework>,
    pub validated_domains: Vec<ValidatedBusinessDomain>,
    pub validation_issues: Vec<String>,
    pub validation_quality: f32,
    pub consensus_strength: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedFramework {
    pub framework: Framework,
    pub consensus_confidence: f32,
    pub tier_agreement_score: f32,
    pub validation_status: ValidationStatus,
    pub supporting_evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedBusinessDomain {
    pub domain: BusinessDomain,
    pub consensus_confidence: f32,
    pub tier2_support: f32,
    pub tier3_support: f32,
    pub validation_status: ValidationStatus,
    pub implementation_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationStatus {
    Validated,
    Pending,
    Rejected,
}

// Final consolidated results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidatedAnalysisResults {
    pub primary_framework: Option<Framework>,
    pub secondary_frameworks: Vec<Framework>,
    pub primary_business_domain: Option<BusinessDomain>,
    pub secondary_business_domains: Vec<BusinessDomain>,
    pub architecture_assessment: ArchitectureAssessment,
    pub implementation_readiness: ImplementationReadinessAssessment,
    pub business_value_assessment: Option<BusinessValueAssessment>,
    pub quality_assurance: QualityAssuranceMetrics,
    pub confidence_summary: ConsolidatedConfidenceSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureAssessment {
    pub architectural_patterns: Vec<String>,
    pub complexity_score: f32,
    pub maintainability_score: f32,
    pub scalability_indicators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationReadinessAssessment {
    pub overall_readiness: f32,
    pub framework_readiness: f32,
    pub context_completeness: f32,
    pub business_alignment: f32,
    pub recommended_next_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessValueAssessment {
    pub overall_business_value: f32,
    pub high_value_capabilities: Vec<String>,
    pub implementation_priority_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAssuranceMetrics {
    pub framework_detection_accuracy: f32,
    pub business_domain_accuracy: f32,
    pub cross_tier_consistency: f32,
    pub overall_analysis_reliability: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidatedConfidenceSummary {
    pub overall_confidence: f32,
    pub framework_confidence: f32,
    pub business_domain_confidence: f32,
    pub tier_coverage_completeness: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionQualityMetrics {
    pub overall_fusion_quality: f32,
    pub tier_alignment_score: f32,
    pub confidence_distribution: f32,
    pub consensus_strength: f32,
    pub result_completeness: f32,
    pub improvement_over_single_tier: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionMetadata {
    pub total_fusion_time_ms: u64,
    pub strategy_used: FusionStrategy,
    pub confidence_weights: ConfidenceWeights,
    pub tiers_processed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchicalFusionResult {
    pub consolidated_results: ConsolidatedAnalysisResults,
    pub tier1_insights: Tier1Insights,
    pub tier2_insights: Tier2Insights,
    pub tier3_insights: Option<Tier3Insights>,
    pub fusion_analysis: WeightedFusionAnalysis,
    pub validated_results: ConsensusValidatedResults,
    pub quality_metrics: FusionQualityMetrics,
    pub fusion_metadata: FusionMetadata,
}