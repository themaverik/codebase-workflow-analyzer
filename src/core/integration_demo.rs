use crate::core::framework_detector::FrameworkDetector;
use crate::core::business_domain_engine::BusinessDomainEngine;

/// Demonstration of Phase 1: Core Framework Detection and Business Domain Inference
pub struct Phase1Demo {
    pub codebase_path: String,
}

impl Phase1Demo {
    pub fn new(codebase_path: String) -> Self {
        Self { codebase_path }
    }

    /// Run complete Phase 1 analysis
    pub fn run_phase1_analysis(&self) -> Result<Phase1Result, Box<dyn std::error::Error>> {
        println!("🚀 Starting Phase 1: Core Framework Detection");
        
        // Step 1: Framework Detection
        let framework_detector = FrameworkDetector::new(self.codebase_path.clone());
        let framework_result = framework_detector.detect_frameworks()?;
        
        println!("✅ Framework Detection Complete");
        println!("   Primary Ecosystem: {:?}", framework_result.primary_ecosystem);
        println!("   Detected Frameworks: {}", framework_result.detected_frameworks.len());
        
        for framework in &framework_result.detected_frameworks {
            println!("   - {:?} (confidence: {:.2})", framework.framework, framework.confidence);
        }
        
        // Step 2: Business Domain Inference
        println!("\n🧠 Starting Business Domain Inference");
        let domain_engine = BusinessDomainEngine::new(self.codebase_path.clone());
        let domain_result = domain_engine.infer_business_domains(&framework_result)?;
        
        println!("✅ Business Domain Analysis Complete");
        println!("   Primary Domains: {}", domain_result.primary_domains.len());
        println!("   Secondary Domains: {}", domain_result.secondary_domains.len());
        
        for domain in &domain_result.primary_domains {
            println!("   - {:?} (confidence: {:.2}, strategy: {:?})", 
                domain.domain, domain.confidence, domain.story_generation_strategy);
        }

        let summary = self.generate_summary(&framework_result, &domain_result);
        
        Ok(Phase1Result {
            framework_result,
            domain_result,
            summary,
        })
    }

    fn generate_summary(&self, framework_result: &crate::core::framework_detector::FrameworkDetectionResult, 
                       domain_result: &crate::core::business_domain_engine::BusinessDomainAnalysisResult) -> String {
        let mut summary = String::new();
        
        summary.push_str("# Phase 1 Analysis Summary\n\n");
        
        // Framework Summary
        summary.push_str("## Detected Frameworks\n");
        for framework in &framework_result.detected_frameworks {
            summary.push_str(&format!("- **{:?}** ({:?}): Confidence {:.1}%\n", 
                framework.framework, 
                framework.ecosystem, 
                framework.confidence * 100.0
            ));
            summary.push_str(&format!("  - Usage Extent: {:?}\n", framework.usage_extent));
            summary.push_str("  - Evidence:\n");
            for evidence in &framework.evidence {
                summary.push_str(&format!("    - {:?}: {}\n", evidence.evidence_type, evidence.pattern));
            }
            summary.push('\n');
        }
        
        // Domain Summary
        summary.push_str("## Business Domains\n");
        if !domain_result.primary_domains.is_empty() {
            summary.push_str("### Primary Domains\n");
            for domain in &domain_result.primary_domains {
                summary.push_str(&format!("- **{:?}** (Confidence: {:.1}%)\n", 
                    domain.domain, domain.confidence * 100.0));
                summary.push_str(&format!("  - Story Strategy: {:?}\n", domain.story_generation_strategy));
                summary.push_str("  - Evidence:\n");
                for evidence in &domain.evidence {
                    summary.push_str(&format!("    - {:?}: {} (weight: {:.2})\n", 
                        evidence.evidence_type, evidence.pattern, evidence.confidence_weight));
                }
                summary.push('\n');
            }
        }
        
        if !domain_result.secondary_domains.is_empty() {
            summary.push_str("### Secondary Domains\n");
            for domain in &domain_result.secondary_domains {
                summary.push_str(&format!("- **{:?}** (Confidence: {:.1}%)\n", 
                    domain.domain, domain.confidence * 100.0));
            }
            summary.push('\n');
        }

        // Next Steps
        summary.push_str("## Next Steps\n");
        summary.push_str("- [ ] Phase 2: Full Framework Support (React, Next.js, Spring Boot, Danet)\n");
        summary.push_str("- [ ] Advanced business domain relationship analysis\n");
        summary.push_str("- [ ] User story generation based on detected domains\n");
        summary.push_str("- [ ] PRD generation with technical architecture documentation\n");
        
        summary
    }
}

#[derive(Debug)]
pub struct Phase1Result {
    pub framework_result: crate::core::framework_detector::FrameworkDetectionResult,
    pub domain_result: crate::core::business_domain_engine::BusinessDomainAnalysisResult,
    pub summary: String,
}

impl Phase1Result {
    /// Print a detailed report of the Phase 1 analysis
    pub fn print_detailed_report(&self) {
        println!("\n{}", "=".repeat(80));
        println!("📊 PHASE 1 DETAILED ANALYSIS REPORT");
        println!("{}", "=".repeat(80));
        
        // Framework Analysis Details
        println!("\n🔧 FRAMEWORK ANALYSIS");
        println!("{}", "-".repeat(40));
        println!("Language Ecosystem: {:?}", self.framework_result.primary_ecosystem);
        
        for framework in &self.framework_result.detected_frameworks {
            println!("\n📦 Framework: {:?}", framework.framework);
            println!("   Confidence: {:.1}%", framework.confidence * 100.0);
            println!("   Usage Extent: {:?}", framework.usage_extent);
            println!("   Version: {:?}", framework.version.as_ref().unwrap_or(&"Unknown".to_string()));
            println!("   Evidence ({} items):", framework.evidence.len());
            for evidence in &framework.evidence {
                println!("     - {:?}: {} (weight: {:.2})", 
                    evidence.evidence_type, evidence.pattern, evidence.confidence_weight);
            }
        }
        
        // Business Domain Analysis Details
        println!("\n🏢 BUSINESS DOMAIN ANALYSIS");
        println!("{}", "-".repeat(40));
        
        if !self.domain_result.primary_domains.is_empty() {
            println!("Primary Business Domains:");
            for domain in &self.domain_result.primary_domains {
                println!("\n🎯 Domain: {:?}", domain.domain);
                println!("   Confidence: {:.1}%", domain.confidence * 100.0);
                println!("   Story Strategy: {:?}", domain.story_generation_strategy);
                println!("   Evidence ({} items):", domain.evidence.len());
                for evidence in &domain.evidence {
                    println!("     - {:?}: {} (weight: {:.2})", 
                        evidence.evidence_type, evidence.pattern, evidence.confidence_weight);
                    if let Some(ref framework) = evidence.framework_context {
                        println!("       Framework Context: {:?}", framework);
                    }
                }
            }
        } else {
            println!("No primary business domains detected with high confidence.");
        }
        
        if !self.domain_result.secondary_domains.is_empty() {
            println!("\nSecondary Business Domains:");
            for domain in &self.domain_result.secondary_domains {
                println!("   - {:?} (Confidence: {:.1}%)", 
                    domain.domain, domain.confidence * 100.0);
            }
        }
        
        // Confidence Thresholds
        println!("\n📏 CONFIDENCE THRESHOLDS");
        println!("{}", "-".repeat(40));
        println!("High Confidence: {:.1}% (Comprehensive story generation)", 
            self.domain_result.confidence_thresholds.high_confidence * 100.0);
        println!("Medium Confidence: {:.1}% (Core story generation)", 
            self.domain_result.confidence_thresholds.medium_confidence * 100.0);
        println!("Low Confidence: {:.1}% (Minimal mention)", 
            self.domain_result.confidence_thresholds.low_confidence * 100.0);
        
        println!("\n{}", "=".repeat(80));
        println!("✨ PHASE 1 ANALYSIS COMPLETE");
        println!("{}", "=".repeat(80));
    }

    /// Save the analysis results to files
    pub fn save_results(&self, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs;
        use std::path::Path;
        
        let output_path = Path::new(output_dir);
        fs::create_dir_all(output_path)?;
        
        // Save summary markdown
        let summary_path = output_path.join("phase1_summary.md");
        fs::write(&summary_path, &self.summary)?;
        println!("📄 Saved summary to: {:?}", summary_path);
        
        // Save detailed JSON results
        let framework_json = serde_json::to_string_pretty(&self.framework_result)?;
        let framework_path = output_path.join("framework_analysis.json");
        fs::write(&framework_path, framework_json)?;
        println!("📄 Saved framework analysis to: {:?}", framework_path);
        
        let domain_json = serde_json::to_string_pretty(&self.domain_result)?;
        let domain_path = output_path.join("domain_analysis.json");
        fs::write(&domain_path, domain_json)?;
        println!("📄 Saved domain analysis to: {:?}", domain_path);
        
        Ok(())
    }
}