use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use walkdir;

use crate::core::UsageExtent;
use crate::core::ast_analyzer::{ASTAnalyzer, CodeSegment, SegmentStatistics};
use crate::intelligence::llm_client::{LocalLLMManager, AnalysisType, BatchAnalysisResult, ModelConfig};
use crate::core::types::{Framework, LanguageEcosystem};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedFrameworkDetectionResult {
    pub primary_ecosystem: LanguageEcosystem,
    pub detected_frameworks: Vec<EnhancedDetectedFramework>,
    pub confidence_summary: HashMap<Framework, f32>,
    pub ast_analysis: Option<ASTAnalysisResult>,
    pub llm_analysis: Option<LLMAnalysisResult>,
    pub code_segments: Vec<CodeSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedDetectedFramework {
    pub framework: Framework,
    pub version: Option<String>,
    pub confidence: f32,
    pub evidence: Vec<DetectionEvidence>,
    pub usage_extent: UsageExtent,
    pub ecosystem: LanguageEcosystem,
    pub ast_evidence: Option<ASTEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASTAnalysisResult {
    pub segment_statistics: SegmentStatistics,
    pub framework_segments: HashMap<Framework, usize>,
    pub business_hints: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMAnalysisResult {
    pub business_domain_analysis: BatchAnalysisResult,
    pub framework_validation: Option<BatchAnalysisResult>,
    pub processing_time_ms: u64,
    pub llm_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASTEvidence {
    pub relevant_segments: usize,
    pub framework_specific_patterns: Vec<String>,
    pub business_domain_hints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionEvidence {
    pub evidence_type: EvidenceType,
    pub source: String,
    pub pattern: String,
    pub confidence_weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceType {
    ConfigFile,     // requirements.txt, package.json, pom.xml
    ImportPattern,  // from flask import, import React
    FileStructure,  // /templates, /src/components
    ContentPattern, // @app.route, @Controller
    ASTAnalysis,    // AST-based semantic evidence
}

pub struct EnhancedFrameworkDetector {
    pub codebase_path: String,
    pub ast_analyzer: Option<ASTAnalyzer>,
    pub llm_manager: Option<LocalLLMManager>,
}

impl EnhancedFrameworkDetector {
    pub fn new(codebase_path: String) -> Result<Self> {
        Ok(Self { 
            codebase_path,
            ast_analyzer: None,
            llm_manager: None,
        })
    }

    pub fn with_ast_analysis(mut self) -> Result<Self> {
        self.ast_analyzer = Some(ASTAnalyzer::new()?);
        Ok(self)
    }

    pub async fn with_llm_analysis(mut self, config: Option<ModelConfig>) -> Result<Self> {
        match LocalLLMManager::new(config).await {
            Ok(manager) => {
                // Test if the model is ready
                if manager.test_connection().await.unwrap_or(false) {
                    println!("🧠 LLM integration ready");
                    self.llm_manager = Some(manager);
                } else {
                    println!("⚠️ LLM not available, continuing without LLM analysis");
                }
            }
            Err(e) => {
                println!("⚠️ Failed to initialize LLM: {}. Continuing without LLM analysis", e);
            }
        }
        Ok(self)
    }

    /// Enhanced main entry point for framework detection with AST and LLM analysis
    pub async fn detect_frameworks_enhanced(&mut self) -> Result<EnhancedFrameworkDetectionResult> {
        println!("🔍 Starting enhanced framework detection...");
        
        // Step 1: Traditional framework detection
        let traditional_result = self.detect_frameworks_traditional()?;
        println!("  ✅ Traditional analysis complete");
        
        // Step 2: AST analysis if enabled
        let (ast_analysis, code_segments) = if let Some(ref mut analyzer) = self.ast_analyzer {
            let segments = analyzer.extract_segments(
                Path::new(&self.codebase_path), 
                &traditional_result.detected_frameworks.iter().map(|f| f.framework.clone()).collect::<Vec<_>>()
            )?;
            
            let ast_stats = analyzer.get_segment_statistics(&segments);
            let ast_analysis = self.create_ast_analysis_result(&segments, &ast_stats);
            println!("  ✅ AST analysis complete ({} segments)", segments.len());
            
            (Some(ast_analysis), segments)
        } else {
            (None, Vec::new())
        };

        // Step 3: LLM analysis if enabled and segments are available
        let llm_analysis = if !code_segments.is_empty() {
            self.run_llm_analysis(&code_segments).await
        } else {
            None
        };

        // Step 4: Enhance framework detection with AST evidence
        let enhanced_frameworks = self.enhance_frameworks_with_ast(
            traditional_result.detected_frameworks,
            &code_segments
        );

        // Step 5: Update confidence scores based on AST and LLM analysis
        let enhanced_confidence = self.recalculate_confidence_with_analysis(
            &traditional_result.confidence_summary,
            &enhanced_frameworks,
            &code_segments,
            &llm_analysis
        );

        println!("  ✅ Enhanced framework detection complete");

        Ok(EnhancedFrameworkDetectionResult {
            primary_ecosystem: traditional_result.primary_ecosystem,
            detected_frameworks: enhanced_frameworks,
            confidence_summary: enhanced_confidence,
            ast_analysis,
            llm_analysis,
            code_segments,
        })
    }

    /// Traditional framework detection (existing logic)
    fn detect_frameworks_traditional(&self) -> Result<TraditionalDetectionResult> {
        let language_ecosystem = self.detect_language_ecosystem()?;
        
        let detected_frameworks = match language_ecosystem {
            LanguageEcosystem::Python => self.detect_python_frameworks()?,
            LanguageEcosystem::JavaScript => self.detect_js_frameworks()?,
            LanguageEcosystem::TypeScript => self.detect_ts_frameworks()?,
            LanguageEcosystem::Java => self.detect_java_frameworks()?,
            LanguageEcosystem::Mixed => self.detect_mixed_frameworks()?,
            LanguageEcosystem::Rust => Vec::new(),
            LanguageEcosystem::Go => Vec::new(),
            LanguageEcosystem::Deno => Vec::new(),
        };

        let confidence_summary = self.create_confidence_summary(&detected_frameworks);

        Ok(TraditionalDetectionResult {
            primary_ecosystem: language_ecosystem,
            detected_frameworks,
            confidence_summary,
        })
    }

    fn detect_language_ecosystem(&self) -> Result<LanguageEcosystem> {
        let mut scores = HashMap::new();
        let file_counts = self.count_files_by_extension()?;
        
        // Language ecosystem scoring based on file extensions
        if *file_counts.get(".py").unwrap_or(&0) > 0 {
            let python_score = *file_counts.get(".py").unwrap_or(&0) * 10;
            scores.insert(LanguageEcosystem::Python, python_score);
        }
        
        if *file_counts.get(".ts").unwrap_or(&0) > 0 || *file_counts.get(".tsx").unwrap_or(&0) > 0 {
            let ts_score = (*file_counts.get(".ts").unwrap_or(&0) + *file_counts.get(".tsx").unwrap_or(&0)) * 10;
            scores.insert(LanguageEcosystem::TypeScript, ts_score);
        }
        
        if *file_counts.get(".js").unwrap_or(&0) > 0 || *file_counts.get(".jsx").unwrap_or(&0) > 0 {
            let js_score = (*file_counts.get(".js").unwrap_or(&0) + *file_counts.get(".jsx").unwrap_or(&0)) * 8;
            scores.insert(LanguageEcosystem::JavaScript, js_score);
        }
        
        if *file_counts.get(".java").unwrap_or(&0) > 0 {
            let java_score = *file_counts.get(".java").unwrap_or(&0) * 10;
            scores.insert(LanguageEcosystem::Java, java_score);
        }

        // Select the ecosystem with the highest score
        if let Some((primary_ecosystem, _)) = scores.into_iter().max_by_key(|(_, score)| *score) {
            Ok(primary_ecosystem)
        } else {
            Ok(LanguageEcosystem::Mixed)
        }
    }

    fn count_files_by_extension(&self) -> Result<HashMap<String, usize>> {
        let mut counts = HashMap::new();
        
        for entry in walkdir::WalkDir::new(&self.codebase_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            
            // Skip filtered directories
            if self.should_skip_path(path) {
                continue;
            }

            if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
                let key = format!(".{}", extension);
                *counts.entry(key).or_insert(0) += 1;
            }
        }
        
        Ok(counts)
    }

    fn should_skip_path(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        
        // Filter out common directories that shouldn't influence language detection
        let skip_patterns = [
            "node_modules", ".git", "target", "dist", "build", ".next",
            "__pycache__", ".venv", "venv", "env", ".env",
            ".idea", ".vscode", "coverage", ".nyc_output"
        ];
        
        skip_patterns.iter().any(|pattern| path_str.contains(pattern))
    }

    // Enhanced framework detection methods
    fn enhance_frameworks_with_ast(
        &self,
        traditional_frameworks: Vec<EnhancedDetectedFramework>, 
        code_segments: &[CodeSegment]
    ) -> Vec<EnhancedDetectedFramework> {
        traditional_frameworks
            .into_iter()
            .map(|mut framework| {
                framework.ast_evidence = self.extract_ast_evidence_for_framework(
                    &framework.framework,
                    code_segments
                );
                framework
            })
            .collect()
    }

    fn extract_ast_evidence_for_framework(
        &self, 
        framework: &Framework, 
        code_segments: &[CodeSegment]
    ) -> Option<ASTEvidence> {
        let relevant_segments: Vec<&CodeSegment> = code_segments
            .iter()
            .filter(|segment| segment.framework_context.as_ref() == Some(framework))
            .collect();

        if relevant_segments.is_empty() {
            return None;
        }

        let framework_patterns: Vec<String> = relevant_segments
            .iter()
            .flat_map(|segment| {
                match framework {
                    Framework::React => self.extract_react_patterns(segment),
                    Framework::NestJS => self.extract_nestjs_patterns(segment),
                    Framework::Flask => self.extract_flask_patterns(segment),
                    Framework::SpringBoot => self.extract_spring_patterns(segment),
                    _ => Vec::new(), // TODO: Implement patterns for other frameworks
                }
            })
            .collect();

        let business_hints: Vec<String> = relevant_segments
            .iter()
            .flat_map(|segment| segment.business_hints.clone())
            .collect();

        Some(ASTEvidence {
            relevant_segments: relevant_segments.len(),
            framework_specific_patterns: framework_patterns,
            business_domain_hints: business_hints,
        })
    }

    fn extract_react_patterns(&self, segment: &CodeSegment) -> Vec<String> {
        let mut patterns = Vec::new();
        let content = &segment.content.to_lowercase();
        
        if content.contains("usestate") { patterns.push("useState hook".to_string()); }
        if content.contains("useeffect") { patterns.push("useEffect hook".to_string()); }
        if content.contains("jsx") || content.contains("tsx") { patterns.push("JSX syntax".to_string()); }
        if content.contains("props") { patterns.push("Props usage".to_string()); }
        
        patterns
    }

    fn extract_nestjs_patterns(&self, segment: &CodeSegment) -> Vec<String> {
        let mut patterns = Vec::new();
        let content = &segment.content.to_lowercase();
        
        if content.contains("@controller") { patterns.push("@Controller decorator".to_string()); }
        if content.contains("@injectable") { patterns.push("@Injectable decorator".to_string()); }
        if content.contains("@get") || content.contains("@post") { patterns.push("HTTP method decorators".to_string()); }
        
        patterns
    }

    fn extract_flask_patterns(&self, segment: &CodeSegment) -> Vec<String> {
        let mut patterns = Vec::new();
        let content = &segment.content.to_lowercase();
        
        if content.contains("@app.route") { patterns.push("Flask route decorator".to_string()); }
        if content.contains("request") { patterns.push("Flask request object".to_string()); }
        if content.contains("jsonify") { patterns.push("Flask jsonify".to_string()); }
        
        patterns
    }

    fn extract_spring_patterns(&self, segment: &CodeSegment) -> Vec<String> {
        let mut patterns = Vec::new();
        let content = &segment.content.to_lowercase();
        
        if content.contains("@restcontroller") { patterns.push("@RestController annotation".to_string()); }
        if content.contains("@requestmapping") { patterns.push("@RequestMapping annotation".to_string()); }
        if content.contains("@service") { patterns.push("@Service annotation".to_string()); }
        if content.contains("@entity") { patterns.push("JPA @Entity annotation".to_string()); }
        
        patterns
    }

    fn create_ast_analysis_result(&self, segments: &[CodeSegment], stats: &SegmentStatistics) -> ASTAnalysisResult {
        let mut framework_segments = HashMap::new();
        let mut business_hints = HashMap::new();

        for segment in segments {
            if let Some(framework) = &segment.framework_context {
                *framework_segments.entry(framework.clone()).or_insert(0) += 1;
            }

            for hint in &segment.business_hints {
                *business_hints.entry(hint.clone()).or_insert(0) += 1;
            }
        }

        ASTAnalysisResult {
            segment_statistics: stats.clone(),
            framework_segments,
            business_hints,
        }
    }

    async fn run_llm_analysis(&self, code_segments: &[CodeSegment]) -> Option<LLMAnalysisResult> {
        if let Some(ref llm_manager) = self.llm_manager {
            let start_time = std::time::Instant::now();
            println!("  🧠 Running LLM business domain analysis...");
            
            match llm_manager.analyze_code_segments(code_segments, AnalysisType::BusinessDomain).await {
                Ok(business_analysis) => {
                    let processing_time = start_time.elapsed().as_millis() as u64;
                    println!("    ✅ LLM analysis complete in {}ms", processing_time);
                    
                    Some(LLMAnalysisResult {
                        business_domain_analysis: business_analysis,
                        framework_validation: None, // TODO: Add framework validation analysis
                        processing_time_ms: processing_time,
                        llm_available: true,
                    })
                }
                Err(e) => {
                    println!("    ⚠️ LLM analysis failed: {}", e);
                    Some(LLMAnalysisResult {
                        business_domain_analysis: self.create_empty_batch_result(),
                        framework_validation: None,
                        processing_time_ms: start_time.elapsed().as_millis() as u64,
                        llm_available: false,
                    })
                }
            }
        } else {
            None
        }
    }

    fn create_empty_batch_result(&self) -> BatchAnalysisResult {
        use crate::intelligence::llm_client::AnalysisSummary;
        use std::collections::HashMap;
        
        BatchAnalysisResult {
            segments: Vec::new(),
            summary: AnalysisSummary {
                total_segments: 0,
                domain_distribution: HashMap::new(),
                average_confidence: 0.0,
                key_patterns: Vec::new(),
            },
            project_analysis: None,
            processing_time_ms: 0,
        }
    }

    fn recalculate_confidence_with_analysis(
        &self,
        traditional_confidence: &HashMap<Framework, f32>,
        enhanced_frameworks: &[EnhancedDetectedFramework],
        _code_segments: &[CodeSegment],
        llm_analysis: &Option<LLMAnalysisResult>
    ) -> HashMap<Framework, f32> {
        let mut enhanced_confidence = traditional_confidence.clone();

        // Apply AST boosts
        for framework in enhanced_frameworks {
            if let Some(ast_evidence) = &framework.ast_evidence {
                let ast_boost = match ast_evidence.relevant_segments {
                    0 => 0.0,
                    1..=3 => 0.1,
                    4..=10 => 0.2,
                    _ => 0.3,
                };

                let current_confidence = enhanced_confidence.get(&framework.framework).unwrap_or(&0.0);
                let new_confidence = (current_confidence + ast_boost).min(1.0);
                enhanced_confidence.insert(framework.framework.clone(), new_confidence);
            }
        }

        // Apply LLM boosts if available
        if let Some(llm_result) = llm_analysis {
            if llm_result.llm_available && llm_result.business_domain_analysis.summary.average_confidence > 0.5 {
                // Apply small boost for successful LLM analysis
                for framework in enhanced_frameworks {
                    let current_confidence = enhanced_confidence.get(&framework.framework).unwrap_or(&0.0);
                    let llm_boost = 0.05; // Small boost for LLM validation
                    let new_confidence = (current_confidence + llm_boost).min(1.0);
                    enhanced_confidence.insert(framework.framework.clone(), new_confidence);
                }
            }
        }

        enhanced_confidence
    }

    fn recalculate_confidence_with_ast(
        &self,
        traditional_confidence: &HashMap<Framework, f32>,
        enhanced_frameworks: &[EnhancedDetectedFramework],
        _code_segments: &[CodeSegment]
    ) -> HashMap<Framework, f32> {
        let mut enhanced_confidence = traditional_confidence.clone();

        for framework in enhanced_frameworks {
            if let Some(ast_evidence) = &framework.ast_evidence {
                let ast_boost = match ast_evidence.relevant_segments {
                    0 => 0.0,
                    1..=3 => 0.1,
                    4..=10 => 0.2,
                    _ => 0.3,
                };

                let current_confidence = enhanced_confidence.get(&framework.framework).unwrap_or(&0.0);
                let new_confidence = (current_confidence + ast_boost).min(1.0);
                enhanced_confidence.insert(framework.framework.clone(), new_confidence);
            }
        }

        enhanced_confidence
    }

    // Traditional detection methods (simplified versions)
    fn detect_python_frameworks(&self) -> Result<Vec<EnhancedDetectedFramework>> {
        let mut frameworks = Vec::new();
        
        if self.has_flask_indicators() {
            frameworks.push(EnhancedDetectedFramework {
                framework: Framework::Flask,
                version: None,
                confidence: 0.8,
                evidence: vec![],
                usage_extent: UsageExtent::Core,
                ecosystem: LanguageEcosystem::Python,
                ast_evidence: None,
            });
        }

        if self.has_fastapi_indicators() {
            frameworks.push(EnhancedDetectedFramework {
                framework: Framework::FastAPI,
                version: None,
                confidence: 0.8,
                evidence: vec![],
                usage_extent: UsageExtent::Core,
                ecosystem: LanguageEcosystem::Python,
                ast_evidence: None,
            });
        }

        Ok(frameworks)
    }

    fn detect_ts_frameworks(&self) -> Result<Vec<EnhancedDetectedFramework>> {
        let mut frameworks = Vec::new();
        
        if self.has_react_indicators() {
            frameworks.push(EnhancedDetectedFramework {
                framework: Framework::React,
                version: None,
                confidence: 0.8,
                evidence: vec![],
                usage_extent: UsageExtent::Core,
                ecosystem: LanguageEcosystem::TypeScript,
                ast_evidence: None,
            });
        }

        if self.has_nestjs_indicators() {
            frameworks.push(EnhancedDetectedFramework {
                framework: Framework::NestJS,
                version: None,
                confidence: 0.8,
                evidence: vec![],
                usage_extent: UsageExtent::Core,
                ecosystem: LanguageEcosystem::TypeScript,
                ast_evidence: None,
            });
        }

        Ok(frameworks)
    }

    fn detect_js_frameworks(&self) -> Result<Vec<EnhancedDetectedFramework>> {
        self.detect_ts_frameworks() // Similar logic
    }

    fn detect_java_frameworks(&self) -> Result<Vec<EnhancedDetectedFramework>> {
        let mut frameworks = Vec::new();
        
        if self.has_spring_indicators() {
            frameworks.push(EnhancedDetectedFramework {
                framework: Framework::SpringBoot,
                version: None,
                confidence: 0.8,
                evidence: vec![],
                usage_extent: UsageExtent::Core,
                ecosystem: LanguageEcosystem::Java,
                ast_evidence: None,
            });
        }

        Ok(frameworks)
    }

    fn detect_mixed_frameworks(&self) -> Result<Vec<EnhancedDetectedFramework>> {
        let mut frameworks = Vec::new();
        frameworks.extend(self.detect_python_frameworks()?);
        frameworks.extend(self.detect_ts_frameworks()?);
        frameworks.extend(self.detect_java_frameworks()?);
        Ok(frameworks)
    }

    // Helper methods for traditional detection
    fn has_flask_indicators(&self) -> bool {
        self.file_contains_pattern("**/*.py", "from flask import") ||
        self.file_contains_pattern("requirements.txt", "Flask") ||
        self.file_contains_pattern("**/*.py", "@app.route")
    }

    fn has_fastapi_indicators(&self) -> bool {
        self.file_contains_pattern("**/*.py", "from fastapi import") ||
        self.file_contains_pattern("requirements.txt", "fastapi") ||
        self.file_contains_pattern("**/*.py", "@app.get")
    }

    fn has_react_indicators(&self) -> bool {
        self.file_contains_pattern("package.json", "react") ||
        self.file_contains_pattern("**/*.tsx", "import React") ||
        self.file_contains_pattern("**/*.jsx", "import React")
    }

    fn has_nestjs_indicators(&self) -> bool {
        self.file_contains_pattern("package.json", "@nestjs/core") ||
        self.file_contains_pattern("**/*.ts", "@Controller") ||
        self.file_contains_pattern("**/*.ts", "@Injectable")
    }

    fn has_spring_indicators(&self) -> bool {
        self.file_contains_pattern("pom.xml", "spring-boot") ||
        self.file_contains_pattern("**/*.java", "@RestController") ||
        self.file_contains_pattern("**/*.java", "@SpringBootApplication")
    }

    fn file_contains_pattern(&self, pattern: &str, search: &str) -> bool {
        // Simplified pattern matching - would use proper glob matching in production
        walkdir::WalkDir::new(&self.codebase_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .any(|entry| {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    content.contains(search)
                } else {
                    false
                }
            })
    }

    fn create_confidence_summary(&self, frameworks: &[EnhancedDetectedFramework]) -> HashMap<Framework, f32> {
        frameworks
            .iter()
            .map(|f| (f.framework.clone(), f.confidence))
            .collect()
    }
}

#[derive(Debug, Clone)]
struct TraditionalDetectionResult {
    pub primary_ecosystem: LanguageEcosystem,
    pub detected_frameworks: Vec<EnhancedDetectedFramework>,
    pub confidence_summary: HashMap<Framework, f32>,
}