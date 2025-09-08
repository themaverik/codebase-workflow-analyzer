use anyhow::Result;
use std::path::PathBuf;

use crate::core::project_analyzer::{ProjectAnalyzer, ProjectContext};
use crate::core::project_classifier::ProjectType;
use crate::core::business_domain_engine::{BusinessDomainEngine, BusinessDomain};
use crate::core::framework_detector::FrameworkDetector;
use crate::core::ast_analyzer::{CodeSegment, ASTAnalyzer, SegmentType, SegmentMetadata, ConfigSegment};
use crate::intelligence::llm_client::{LocalLLMManager, ModelConfig};

/// Self-analysis test to validate context-aware classification
pub struct ContextAwareSelfTest {
    project_path: PathBuf,
}

#[derive(Debug)]
pub struct SelfAnalysisResult {
    pub project_classification: ProjectType,
    pub project_confidence: f32,
    pub business_domain: BusinessDomain,
    pub domain_confidence: f32,
    pub context_aware_success: bool,
    pub classification_evidence: Vec<String>,
}

impl ContextAwareSelfTest {
    pub fn new() -> Self {
        Self {
            project_path: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }

    /// Run the self-analysis test to validate context-aware classification
    pub async fn run_self_analysis_test(&self) -> Result<SelfAnalysisResult> {
        println!("🧪 Starting self-analysis test to validate context-aware classification...");
        println!("  Project path: {}", self.project_path.display());

        // Step 1: Analyze project context
        let project_analyzer = ProjectAnalyzer::new();
        let project_context = project_analyzer.analyze_project_context(&self.project_path).await?;
        
        println!("  ✅ Project context analyzed");
        println!("    Project name: {}", project_context.metadata.name);
        if let Some(ref project_type) = project_context.project_type {
            println!("    Initial classification: {:?} (confidence: {:.1}%)", 
                project_type, project_context.project_type_confidence * 100.0);
        }

        // Step 2: Get framework detection results
        let framework_detector = FrameworkDetector::new(self.project_path.to_string_lossy().to_string());
        let framework_result = framework_detector.detect_frameworks()
            .map_err(|e| anyhow::anyhow!("Framework detection failed: {}", e))?;
        
        println!("  ✅ Framework detection completed");
        println!("    Detected frameworks: {}", framework_result.detected_frameworks.len());

        // Step 3: Get code segments for analysis
        let ast_analyzer = ASTAnalyzer::new()
            .map_err(|e| anyhow::anyhow!("Failed to create AST analyzer: {}", e))?;
        let code_segments = self.get_representative_code_segments(&ast_analyzer)?;
        
        println!("  ✅ Code segments extracted: {}", code_segments.len());

        // Step 4: Test context-aware business domain analysis
        let business_domain_engine = BusinessDomainEngine::new(self.project_path.to_string_lossy().to_string());
        
        // Check if LLM is available
        let llm_available = self.check_llm_availability().await;
        
        let (business_domain, domain_confidence, context_aware_success) = if llm_available {
            println!("  🧠 LLM available - running context-aware analysis...");
            
            let llm_manager = LocalLLMManager::new(None).await?;
            let domain_result = business_domain_engine
                .infer_business_domains_with_context(&framework_result, &project_context, &code_segments, &llm_manager)
                .await
                .map_err(|e| anyhow::anyhow!("LLM domain analysis failed: {}", e))?;
            
            let primary_domain = domain_result.primary_domains
                .first()
                .map(|d| d.domain.clone())
                .unwrap_or(BusinessDomain::Unknown);
            
            let confidence = domain_result.primary_domains
                .first()
                .map(|d| d.confidence)
                .unwrap_or(0.0);
            
            (primary_domain, confidence, true)
        } else {
            println!("  ⚠️  LLM not available - running traditional analysis only...");
            
            let domain_result = business_domain_engine.infer_business_domains(&framework_result)
                .map_err(|e| anyhow::anyhow!("Business domain analysis failed: {}", e))?;
            
            let primary_domain = domain_result.primary_domains
                .first()
                .map(|d| d.domain.clone())
                .unwrap_or(BusinessDomain::Unknown);
            
            let confidence = domain_result.primary_domains
                .first()
                .map(|d| d.confidence)
                .unwrap_or(0.0);
            
            (primary_domain, confidence, false)
        };

        // Step 5: Validate results
        let expected_project_type = ProjectType::AnalysisTool;
        let expected_domain = BusinessDomain::Analytics; // Analytics is the closest match for analysis tools

        let project_classification_correct = project_context.project_type
            .as_ref()
            .map(|pt| *pt == expected_project_type)
            .unwrap_or(false);

        let domain_classification_correct = business_domain == expected_domain;

        println!("  📊 Analysis Results:");
        println!("    Project Type: {:?} (Expected: {:?}) {}", 
            project_context.project_type.as_ref().unwrap_or(&ProjectType::Unknown),
            expected_project_type,
            if project_classification_correct { "✅" } else { "❌" });
        
        println!("    Business Domain: {:?} (Expected: {:?}) {}", 
            business_domain,
            expected_domain,
            if domain_classification_correct { "✅" } else { "❌" });
        
        println!("    Domain Confidence: {:.1}%", domain_confidence * 100.0);
        println!("    Context-Aware Analysis: {}", if context_aware_success { "✅ Used" } else { "⚠️ Not Available" });

        // Generate evidence for classification
        let mut classification_evidence = vec![
            format!("Project name: {}", project_context.metadata.name),
            format!("Entry points detected: {}", project_context.entry_points.len()),
            format!("Business domain hints: {}", project_context.business_domain_hints.join(", ")),
        ];

        if let Some(desc) = &project_context.metadata.description {
            classification_evidence.push(format!("Description: {}", desc));
        }

        let result = SelfAnalysisResult {
            project_classification: project_context.project_type.unwrap_or(ProjectType::Unknown),
            project_confidence: project_context.project_type_confidence,
            business_domain,
            domain_confidence,
            context_aware_success,
            classification_evidence,
        };

        // Print final assessment
        if project_classification_correct && domain_classification_correct {
            println!("  🎉 Self-analysis test PASSED - Context-aware classification working correctly!");
        } else {
            println!("  ❌ Self-analysis test FAILED - Context-aware classification needs improvement");
            if !project_classification_correct {
                println!("    Issue: Project type misclassification");
            }
            if !domain_classification_correct {
                println!("    Issue: Business domain misclassification");
            }
        }

        Ok(result)
    }

    /// Check if LLM is available for testing
    async fn check_llm_availability(&self) -> bool {
        match LocalLLMManager::new(Some(ModelConfig::default())).await {
            Ok(llm) => {
                match llm.ensure_model_ready().await {
                    Ok(_) => true,
                    Err(e) => {
                        println!("    LLM model not ready: {}", e);
                        false
                    }
                }
            }
            Err(e) => {
                println!("    LLM not available: {}", e);
                false
            }
        }
    }

    /// Get representative code segments for analysis
    fn get_representative_code_segments(&self, ast_analyzer: &ASTAnalyzer) -> Result<Vec<CodeSegment>> {
        let mut segments = Vec::new();
        
        // Add key files that represent the analyzer's purpose
        let key_files = vec![
            "src/main.rs",
            "src/lib.rs", 
            "src/cli/mod.rs",
            "src/core/mod.rs",
            "src/intelligence/llm_client.rs",
            "Cargo.toml",
        ];

        for file_path in key_files {
            let full_path = self.project_path.join(file_path);
            if full_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&full_path) {
                    // Create a simplified code segment
                    let segment = CodeSegment {
                        segment_type: SegmentType::Configuration(ConfigSegment {
                            config_type: "project_file".to_string(),
                            properties: std::collections::HashMap::new(),
                        }),
                        content: content.chars().take(2000).collect(), // Limit to first 2000 chars
                        metadata: SegmentMetadata {
                            line_start: 1,
                            line_end: content.lines().count().min(50), // Limit to first 50 lines
                            file_path: full_path.clone(),
                            byte_start: 0,
                            byte_end: content.len().min(2000),
                        },
                        framework_context: None,
                        business_hints: vec![],
                    };
                    segments.push(segment);
                }
            }
        }

        // If no segments found, create a basic segment with project metadata
        if segments.is_empty() {
            let basic_content = format!(
                "// Codebase Workflow Analyzer\n// Advanced reverse engineering tool\n// Project path: {}", 
                self.project_path.display()
            );
            
            segments.push(CodeSegment {
                segment_type: SegmentType::Configuration(ConfigSegment {
                    config_type: "synthetic".to_string(),
                    properties: std::collections::HashMap::new(),
                }),
                content: basic_content.clone(),
                metadata: SegmentMetadata {
                    line_start: 1,
                    line_end: 3,
                    file_path: self.project_path.join("synthetic_segment.rs"),
                    byte_start: 0,
                    byte_end: basic_content.len(),
                },
                framework_context: None,
                business_hints: vec![],
            });
        }

        println!("    Representative segments from {} files", segments.len());
        Ok(segments)
    }
}

/// Run a quick self-analysis validation test
pub async fn validate_context_aware_classification() -> Result<()> {
    let test = ContextAwareSelfTest::new();
    let result = test.run_self_analysis_test().await?;
    
    println!("\n📋 Self-Analysis Test Summary:");
    println!("Project Classification: {:?} (confidence: {:.1}%)", 
        result.project_classification, result.project_confidence * 100.0);
    println!("Business Domain: {:?} (confidence: {:.1}%)", 
        result.business_domain, result.domain_confidence * 100.0);
    println!("Context-Aware Analysis Used: {}", result.context_aware_success);
    
    println!("\nClassification Evidence:");
    for evidence in &result.classification_evidence {
        println!("  • {}", evidence);
    }

    if result.project_classification == ProjectType::AnalysisTool && 
       (result.business_domain == BusinessDomain::Analytics || result.business_domain == BusinessDomain::DataProcessing) {
        println!("\n✅ Context-aware classification is working correctly!");
        println!("   The analyzer correctly identifies itself as an AnalysisTool with Analytics/DataProcessing domain.");
    } else {
        println!("\n⚠️  Context-aware classification may need refinement:");
        println!("   Expected: AnalysisTool + Analytics/DataProcessing domain");
        println!("   Actual: {:?} + {:?} domain", result.project_classification, result.business_domain);
    }

    Ok(())
}