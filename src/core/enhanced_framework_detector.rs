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
use crate::core::context_aware_ast_analyzer::ContextAwareASTAnalyzer;
use crate::core::context_types::EnhancedSegmentContext;

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
    ConfigFile,         // requirements.txt, package.json, pom.xml
    ImportPattern,      // from flask import, import React
    FileStructure,      // /templates, /src/components
    ContentPattern,     // @app.route, @Controller
    ASTAnalysis,        // AST-based semantic evidence
    DecoratorPattern,   // @app.route, @Controller, @Injectable
    AnnotationPattern,  // @SpringBootApplication, @RestController
    FrameworkInstance,  // FastAPI(), NestFactory.create
    HookPattern,        // useState, useEffect
}

pub struct EnhancedFrameworkDetector {
    pub codebase_path: String,
    pub ast_analyzer: Option<ASTAnalyzer>,
    pub context_aware_analyzer: Option<ContextAwareASTAnalyzer>,
    pub llm_manager: Option<LocalLLMManager>,
}

impl EnhancedFrameworkDetector {
    pub fn new(codebase_path: String) -> Result<Self> {
        Ok(Self { 
            codebase_path,
            ast_analyzer: None,
            context_aware_analyzer: None,
            llm_manager: None,
        })
    }

    pub fn with_ast_analysis(mut self) -> Result<Self> {
        self.ast_analyzer = Some(ASTAnalyzer::new()?);
        Ok(self)
    }

    pub fn with_context_aware_analysis(mut self) -> Result<Self> {
        self.context_aware_analyzer = Some(ContextAwareASTAnalyzer::new()?);
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
        // Starting enhanced framework detection
        
        // Step 1: Traditional framework detection
        let traditional_result = self.detect_frameworks_traditional()?;
        // Traditional analysis complete
        
        // Step 2: Choose between traditional AST or context-aware analysis
        let (ast_analysis, code_segments, enhanced_segments) = if let Some(ref mut context_analyzer) = self.context_aware_analyzer {
            // Context-aware analysis path
            context_analyzer.initialize(Path::new(&self.codebase_path)).await?;
            let extraction_result = context_analyzer.extract_segments_with_context(Path::new(&self.codebase_path)).await?;
            
            println!("  Context-aware AST analysis complete ({} enhanced segments)", extraction_result.enhanced_segments.len());
            
            // Create empty code segments for compatibility (we'll use enhanced segments for LLM)
            let code_segments: Vec<CodeSegment> = Vec::new();
            
            // Create AST analysis result from enhanced segments
            let ast_analysis = self.create_enhanced_ast_analysis_result(&extraction_result.enhanced_segments);
            
            (Some(ast_analysis), code_segments, extraction_result.enhanced_segments)
            
        } else if let Some(ref mut analyzer) = self.ast_analyzer {
            // Traditional AST analysis path
            let segments = analyzer.extract_segments(
                Path::new(&self.codebase_path), 
                &traditional_result.detected_frameworks.iter().map(|f| f.framework.clone()).collect::<Vec<_>>()
            )?;
            
            let ast_stats = analyzer.get_segment_statistics(&segments);
            let ast_analysis = self.create_ast_analysis_result(&segments, &ast_stats);
            println!("  AST analysis complete ({} segments)", segments.len());
            
            (Some(ast_analysis), segments, Vec::new())
        } else {
            (None, Vec::new(), Vec::new())
        };

        // Step 3: LLM analysis - use enhanced segments if available, otherwise fall back to code segments
        let llm_analysis = if !enhanced_segments.is_empty() {
            self.run_enhanced_llm_analysis(&enhanced_segments).await
        } else if !code_segments.is_empty() {
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

        // Enhanced framework detection complete

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
            LanguageEcosystem::Rust => self.detect_rust_frameworks()?,
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
        
        if *file_counts.get(".rs").unwrap_or(&0) > 0 {
            let rust_score = *file_counts.get(".rs").unwrap_or(&0) * 10;
            scores.insert(LanguageEcosystem::Rust, rust_score);
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
            println!("  Running LLM business domain analysis...");
            
            match llm_manager.analyze_code_segments(code_segments, AnalysisType::BusinessDomain).await {
                Ok(business_analysis) => {
                    let processing_time = start_time.elapsed().as_millis() as u64;
                    println!("    LLM analysis complete in {}ms", processing_time);
                    
                    Some(LLMAnalysisResult {
                        business_domain_analysis: business_analysis,
                        framework_validation: None, // TODO: Add framework validation analysis
                        processing_time_ms: processing_time,
                        llm_available: true,
                    })
                }
                Err(e) => {
                    println!("    LLM analysis failed: {}", e);
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

    async fn run_enhanced_llm_analysis(&self, enhanced_segments: &[EnhancedSegmentContext]) -> Option<LLMAnalysisResult> {
        if let Some(ref llm_manager) = self.llm_manager {
            let start_time = std::time::Instant::now();
            println!("  Running enhanced LLM analysis with project context...");
            
            match llm_manager.analyze_enhanced_segments(enhanced_segments, AnalysisType::BusinessDomain).await {
                Ok(business_analysis) => {
                    let processing_time = start_time.elapsed().as_millis() as u64;
                    println!("    Enhanced LLM analysis complete in {}ms", processing_time);
                    
                    Some(LLMAnalysisResult {
                        business_domain_analysis: business_analysis,
                        framework_validation: None, // TODO: Add framework validation analysis
                        processing_time_ms: processing_time,
                        llm_available: true,
                    })
                }
                Err(e) => {
                    println!("    Enhanced LLM analysis failed: {}", e);
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

    fn create_enhanced_ast_analysis_result(&self, enhanced_segments: &[EnhancedSegmentContext]) -> ASTAnalysisResult {
        let mut business_hints = Vec::new();
        let mut segment_type_distribution = std::collections::HashMap::new();
        let mut language_distribution = std::collections::HashMap::new();
        let mut framework_segments = std::collections::HashMap::new();
        
        // Extract information from enhanced segments
        for enhanced_segment in enhanced_segments {
            // Collect business hints
            business_hints.extend(enhanced_segment.business_hints.clone());
            
            // Count segment types
            let segment_type = &enhanced_segment.segment_context.segment.segment_type;
            *segment_type_distribution.entry(segment_type.clone()).or_insert(0) += 1;
            
            // Count languages
            let language = &enhanced_segment.segment_context.segment.language;
            *language_distribution.entry(language.clone()).or_insert(0) += 1;
        }
        
        // Remove duplicates from business hints
        business_hints.sort();
        business_hints.dedup();

        // Convert Vec<String> to HashMap<String, usize> for business_hints (count occurrences)
        let mut business_hint_counts = std::collections::HashMap::new();
        for hint in &business_hints {
            *business_hint_counts.entry(hint.clone()).or_insert(0) += 1;
        }

        // Create segment statistics
        let segment_statistics = SegmentStatistics {
            total_segments: enhanced_segments.len(),
            function_count: segment_type_distribution.get("function").unwrap_or(&0).clone(),
            class_count: segment_type_distribution.get("class").unwrap_or(&0).clone(),
            interface_count: segment_type_distribution.get("interface").unwrap_or(&0).clone(),
            route_count: segment_type_distribution.get("route").unwrap_or(&0).clone(),
            config_count: segment_type_distribution.get("configuration").unwrap_or(&0).clone(),
            database_count: segment_type_distribution.get("database").unwrap_or(&0).clone(),
            framework_segments: framework_segments.values().sum(),
            total_business_hints: business_hint_counts.len(),
        };

        ASTAnalysisResult {
            segment_statistics,
            framework_segments,
            business_hints: business_hint_counts,
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
        
        // Flask detection with enhanced scoring
        let flask_confidence = self.enhanced_flask_detection();
        if flask_confidence > 0.3 {
            frameworks.push(EnhancedDetectedFramework {
                framework: Framework::Flask,
                version: None,
                confidence: flask_confidence,
                evidence: self.generate_flask_evidence(),
                usage_extent: self.determine_usage_extent(flask_confidence),
                ecosystem: LanguageEcosystem::Python,
                ast_evidence: None,
            });
        }

        // FastAPI detection with enhanced scoring
        let fastapi_confidence = self.enhanced_fastapi_detection();
        if fastapi_confidence > 0.3 {
            frameworks.push(EnhancedDetectedFramework {
                framework: Framework::FastAPI,
                version: None,
                confidence: fastapi_confidence,
                evidence: self.generate_fastapi_evidence(),
                usage_extent: self.determine_usage_extent(fastapi_confidence),
                ecosystem: LanguageEcosystem::Python,
                ast_evidence: None,
            });
        }

        Ok(frameworks)
    }

    fn detect_ts_frameworks(&self) -> Result<Vec<EnhancedDetectedFramework>> {
        let mut frameworks = Vec::new();
        
        // React detection with enhanced scoring
        let react_confidence = self.enhanced_react_detection();
        if react_confidence > 0.3 {
            frameworks.push(EnhancedDetectedFramework {
                framework: Framework::React,
                version: None,
                confidence: react_confidence,
                evidence: self.generate_react_evidence(),
                usage_extent: self.determine_usage_extent(react_confidence),
                ecosystem: LanguageEcosystem::TypeScript,
                ast_evidence: None,
            });
        }

        // NestJS detection with enhanced scoring
        let nestjs_confidence = self.enhanced_nestjs_detection();
        if nestjs_confidence > 0.3 {
            frameworks.push(EnhancedDetectedFramework {
                framework: Framework::NestJS,
                version: None,
                confidence: nestjs_confidence,
                evidence: self.generate_nestjs_evidence(),
                usage_extent: self.determine_usage_extent(nestjs_confidence),
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
        
        // Spring Boot detection with enhanced scoring
        let spring_confidence = self.enhanced_spring_detection();
        if spring_confidence > 0.3 {
            frameworks.push(EnhancedDetectedFramework {
                framework: Framework::SpringBoot,
                version: None,
                confidence: spring_confidence,
                evidence: self.generate_spring_evidence(),
                usage_extent: self.determine_usage_extent(spring_confidence),
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
        frameworks.extend(self.detect_rust_frameworks()?);
        Ok(frameworks)
    }

    // Helper methods for traditional detection
    fn has_flask_indicators(&self) -> bool {
        self.enhanced_flask_detection() > 0.3
    }

    fn has_fastapi_indicators(&self) -> bool {
        self.enhanced_fastapi_detection() > 0.3
    }

    fn has_react_indicators(&self) -> bool {
        self.enhanced_react_detection() > 0.3
    }

    fn has_nestjs_indicators(&self) -> bool {
        self.enhanced_nestjs_detection() > 0.3
    }

    fn has_spring_indicators(&self) -> bool {
        self.enhanced_spring_detection() > 0.3
    }

    /// Enhanced Flask detection with confidence scoring
    fn enhanced_flask_detection(&self) -> f32 {
        let mut confidence: f32 = 0.0;
        let mut evidence_count = 0;

        // Core Flask imports (High confidence)
        if self.file_contains_pattern("**/*.py", "from flask import") {
            confidence += 0.4;
            evidence_count += 1;
        }

        // Flask specific decorators (High confidence)
        if self.file_contains_pattern("**/*.py", "@app.route") {
            confidence += 0.35;
            evidence_count += 1;
        }

        // Flask configuration patterns (Medium confidence)
        if self.file_contains_pattern("**/*.py", "app = Flask(__name__)") ||
           self.file_contains_pattern("**/*.py", "Flask(__name__)") {
            confidence += 0.3;
            evidence_count += 1;
        }

        // Flask blueprints (Medium confidence)
        if self.file_contains_pattern("**/*.py", "Blueprint") ||
           self.file_contains_pattern("**/*.py", "from flask import Blueprint") {
            confidence += 0.25;
            evidence_count += 1;
        }

        // Flask request handling (Medium confidence)
        if self.file_contains_pattern("**/*.py", "from flask import request") ||
           self.file_contains_pattern("**/*.py", "request.json") ||
           self.file_contains_pattern("**/*.py", "request.form") {
            confidence += 0.2;
            evidence_count += 1;
        }

        // Flask template rendering (Medium confidence)
        if self.file_contains_pattern("**/*.py", "render_template") ||
           self.file_contains_pattern("**/*.py", "from flask import render_template") {
            confidence += 0.2;
            evidence_count += 1;
        }

        // Flask extensions (Low-Medium confidence)
        if self.file_contains_pattern("**/*.py", "flask-") ||
           self.file_contains_pattern("requirements.txt", "Flask-") {
            confidence += 0.15;
            evidence_count += 1;
        }

        // Package manager files (Medium confidence)
        if self.file_contains_pattern("requirements.txt", "Flask") {
            confidence += 0.25;
            evidence_count += 1;
        }

        // Flask CLI commands (Low confidence)
        if self.file_contains_pattern("**/*.py", "@app.cli.command") {
            confidence += 0.1;
            evidence_count += 1;
        }

        // Normalize confidence based on evidence diversity
        if evidence_count >= 3 {
            confidence = confidence.min(0.95);
        } else if evidence_count >= 2 {
            confidence = confidence.min(0.8);
        }

        confidence
    }

    /// Enhanced FastAPI detection with confidence scoring
    fn enhanced_fastapi_detection(&self) -> f32 {
        let mut confidence: f32 = 0.0;
        let mut evidence_count = 0;

        // Core FastAPI imports (High confidence)
        if self.file_contains_pattern("**/*.py", "from fastapi import") {
            confidence += 0.4;
            evidence_count += 1;
        }

        // FastAPI app creation (High confidence)
        if self.file_contains_pattern("**/*.py", "FastAPI()") ||
           self.file_contains_pattern("**/*.py", "app = FastAPI") {
            confidence += 0.35;
            evidence_count += 1;
        }

        // FastAPI route decorators (High confidence)
        if self.file_contains_pattern("**/*.py", "@app.get") ||
           self.file_contains_pattern("**/*.py", "@app.post") ||
           self.file_contains_pattern("**/*.py", "@app.put") ||
           self.file_contains_pattern("**/*.py", "@app.delete") {
            confidence += 0.35;
            evidence_count += 1;
        }

        // Pydantic models (Medium-High confidence)
        if self.file_contains_pattern("**/*.py", "from pydantic import") ||
           self.file_contains_pattern("**/*.py", "BaseModel") {
            confidence += 0.3;
            evidence_count += 1;
        }

        // FastAPI dependency injection (Medium confidence)
        if self.file_contains_pattern("**/*.py", "Depends(") ||
           self.file_contains_pattern("**/*.py", "from fastapi import Depends") {
            confidence += 0.25;
            evidence_count += 1;
        }

        // FastAPI middleware (Medium confidence)
        if self.file_contains_pattern("**/*.py", "add_middleware") ||
           self.file_contains_pattern("**/*.py", "@app.middleware") {
            confidence += 0.25;
            evidence_count += 1;
        }

        // FastAPI async patterns (Medium confidence)
        if self.file_contains_pattern("**/*.py", "async def") {
            confidence += 0.2;
            evidence_count += 1;
        }

        // Package manager files (Medium confidence)
        if self.file_contains_pattern("requirements.txt", "fastapi") {
            confidence += 0.25;
            evidence_count += 1;
        }

        // FastAPI response models (Low-Medium confidence)
        if self.file_contains_pattern("**/*.py", "Response") ||
           self.file_contains_pattern("**/*.py", "from fastapi.responses import") {
            confidence += 0.15;
            evidence_count += 1;
        }

        // Normalize confidence based on evidence diversity
        if evidence_count >= 3 {
            confidence = confidence.min(0.95);
        } else if evidence_count >= 2 {
            confidence = confidence.min(0.8);
        }

        confidence
    }

    /// Enhanced React detection with confidence scoring
    fn enhanced_react_detection(&self) -> f32 {
        let mut confidence: f32 = 0.0;
        let mut evidence_count = 0;

        // React dependency in package.json (High confidence)
        if self.file_contains_pattern("package.json", "\"react\":") {
            confidence += 0.4;
            evidence_count += 1;
        }

        // React imports (High confidence)
        if self.file_contains_pattern("**/*.tsx", "import React") ||
           self.file_contains_pattern("**/*.jsx", "import React") {
            confidence += 0.35;
            evidence_count += 1;
        }

        // JSX patterns (High confidence)
        if self.file_contains_pattern("**/*.tsx", "return (") ||
           self.file_contains_pattern("**/*.jsx", "return (") {
            confidence += 0.3;
            evidence_count += 1;
        }

        // React hooks (Medium-High confidence)
        if self.file_contains_pattern("**/*.ts", "useState") ||
           self.file_contains_pattern("**/*.tsx", "useState") ||
           self.file_contains_pattern("**/*.jsx", "useState") ||
           self.file_contains_pattern("**/*.js", "useState") {
            confidence += 0.3;
            evidence_count += 1;
        }

        // React component patterns (Medium confidence)
        if self.file_contains_pattern("**/*.tsx", "function Component") ||
           self.file_contains_pattern("**/*.jsx", "function Component") ||
           self.file_contains_pattern("**/*.tsx", "const ") ||
           self.file_contains_pattern("**/*.jsx", "const ") {
            confidence += 0.25;
            evidence_count += 1;
        }

        // React DOM (Medium confidence)
        if self.file_contains_pattern("package.json", "react-dom") ||
           self.file_contains_pattern("**/*.ts", "ReactDOM") {
            confidence += 0.25;
            evidence_count += 1;
        }

        // React development tools (Low-Medium confidence)
        if self.file_contains_pattern("package.json", "react-scripts") ||
           self.file_contains_pattern("package.json", "@types/react") {
            confidence += 0.2;
            evidence_count += 1;
        }

        // React routing (Low-Medium confidence)
        if self.file_contains_pattern("package.json", "react-router") ||
           self.file_contains_pattern("**/*.tsx", "BrowserRouter") {
            confidence += 0.15;
            evidence_count += 1;
        }

        // Normalize confidence based on evidence diversity
        if evidence_count >= 3 {
            confidence = confidence.min(0.95);
        } else if evidence_count >= 2 {
            confidence = confidence.min(0.8);
        }

        confidence
    }

    /// Enhanced NestJS detection with confidence scoring
    fn enhanced_nestjs_detection(&self) -> f32 {
        let mut confidence: f32 = 0.0;
        let mut evidence_count = 0;

        // Core NestJS dependencies (High confidence)
        if self.file_contains_pattern("package.json", "@nestjs/core") {
            confidence += 0.4;
            evidence_count += 1;
        }

        // NestJS decorators (High confidence)
        if self.file_contains_pattern("**/*.ts", "@Controller") ||
           self.file_contains_pattern("**/*.ts", "@Injectable") ||
           self.file_contains_pattern("**/*.ts", "@Module") {
            confidence += 0.35;
            evidence_count += 1;
        }

        // NestJS main application file (Medium-High confidence)
        if self.file_contains_pattern("**/*.ts", "NestFactory.create") {
            confidence += 0.3;
            evidence_count += 1;
        }

        // NestJS HTTP decorators (Medium confidence)
        if self.file_contains_pattern("**/*.ts", "@Get(") ||
           self.file_contains_pattern("**/*.ts", "@Post(") ||
           self.file_contains_pattern("**/*.ts", "@Put(") ||
           self.file_contains_pattern("**/*.ts", "@Delete(") {
            confidence += 0.3;
            evidence_count += 1;
        }

        // NestJS dependency injection (Medium confidence)
        if self.file_contains_pattern("**/*.ts", "@Inject(") ||
           self.file_contains_pattern("**/*.ts", "constructor(") {
            confidence += 0.25;
            evidence_count += 1;
        }

        // NestJS guards and pipes (Medium confidence)
        if self.file_contains_pattern("**/*.ts", "@UseGuards") ||
           self.file_contains_pattern("**/*.ts", "@UsePipes") {
            confidence += 0.25;
            evidence_count += 1;
        }

        // NestJS testing (Low-Medium confidence)
        if self.file_contains_pattern("**/*.spec.ts", "@nestjs/testing") {
            confidence += 0.2;
            evidence_count += 1;
        }

        // NestJS CLI file (Low-Medium confidence)
        if self.file_contains_pattern("nest-cli.json", "nest") {
            confidence += 0.15;
            evidence_count += 1;
        }

        // Normalize confidence based on evidence diversity
        if evidence_count >= 3 {
            confidence = confidence.min(0.95);
        } else if evidence_count >= 2 {
            confidence = confidence.min(0.8);
        }

        confidence
    }

    /// Enhanced Spring Boot detection with confidence scoring
    fn enhanced_spring_detection(&self) -> f32 {
        let mut confidence: f32 = 0.0;
        let mut evidence_count = 0;

        // Spring Boot dependencies (High confidence)
        if self.file_contains_pattern("pom.xml", "spring-boot-starter") ||
           self.file_contains_pattern("build.gradle", "spring-boot-starter") {
            confidence += 0.4;
            evidence_count += 1;
        }

        // Spring Boot main class (High confidence)
        if self.file_contains_pattern("**/*.java", "@SpringBootApplication") {
            confidence += 0.4;
            evidence_count += 1;
        }

        // Spring MVC annotations (Medium-High confidence)
        if self.file_contains_pattern("**/*.java", "@RestController") ||
           self.file_contains_pattern("**/*.java", "@Controller") {
            confidence += 0.35;
            evidence_count += 1;
        }

        // Spring HTTP mapping annotations (Medium confidence)
        if self.file_contains_pattern("**/*.java", "@GetMapping") ||
           self.file_contains_pattern("**/*.java", "@PostMapping") ||
           self.file_contains_pattern("**/*.java", "@RequestMapping") {
            confidence += 0.3;
            evidence_count += 1;
        }

        // Spring dependency injection (Medium confidence)
        if self.file_contains_pattern("**/*.java", "@Autowired") ||
           self.file_contains_pattern("**/*.java", "@Component") ||
           self.file_contains_pattern("**/*.java", "@Service") {
            confidence += 0.25;
            evidence_count += 1;
        }

        // Spring configuration (Medium confidence)
        if self.file_contains_pattern("**/*.java", "@Configuration") ||
           self.file_contains_pattern("**/*.properties", "spring.") ||
           self.file_contains_pattern("**/*.yml", "spring:") {
            confidence += 0.25;
            evidence_count += 1;
        }

        // Spring Data JPA (Low-Medium confidence)
        if self.file_contains_pattern("**/*.java", "@Entity") ||
           self.file_contains_pattern("**/*.java", "@Repository") ||
           self.file_contains_pattern("pom.xml", "spring-boot-starter-data-jpa") {
            confidence += 0.2;
            evidence_count += 1;
        }

        // Spring Boot testing (Low-Medium confidence)
        if self.file_contains_pattern("**/*.java", "@SpringBootTest") {
            confidence += 0.15;
            evidence_count += 1;
        }

        // Normalize confidence based on evidence diversity
        if evidence_count >= 3 {
            confidence = confidence.min(0.95);
        } else if evidence_count >= 2 {
            confidence = confidence.min(0.8);
        }

        confidence
    }

    fn detect_rust_frameworks(&self) -> Result<Vec<EnhancedDetectedFramework>> {
        let mut frameworks = Vec::new();
        
        if self.has_axum_indicators() {
            frameworks.push(EnhancedDetectedFramework {
                framework: Framework::Axum,
                version: None,
                confidence: 0.95,
                evidence: vec![
                    DetectionEvidence {
                        evidence_type: EvidenceType::ConfigFile,
                        source: "Cargo.toml".to_string(),
                        pattern: "axum".to_string(),
                        confidence_weight: 0.4,
                    },
                    DetectionEvidence {
                        evidence_type: EvidenceType::ImportPattern,
                        source: "*.rs files".to_string(),
                        pattern: "use axum::".to_string(),
                        confidence_weight: 0.6,
                    },
                ],
                usage_extent: UsageExtent::Core,
                ecosystem: LanguageEcosystem::Rust,
                ast_evidence: None,
            });
        }
        
        if self.has_actix_indicators() {
            frameworks.push(EnhancedDetectedFramework {
                framework: Framework::Actix,
                version: None,
                confidence: 0.95,
                evidence: vec![
                    DetectionEvidence {
                        evidence_type: EvidenceType::ConfigFile,
                        source: "Cargo.toml".to_string(),
                        pattern: "actix-web".to_string(),
                        confidence_weight: 0.4,
                    },
                    DetectionEvidence {
                        evidence_type: EvidenceType::ImportPattern,
                        source: "*.rs files".to_string(),
                        pattern: "use actix_web::".to_string(),
                        confidence_weight: 0.6,
                    },
                ],
                usage_extent: UsageExtent::Core,
                ecosystem: LanguageEcosystem::Rust,
                ast_evidence: None,
            });
        }
        
        if self.has_warp_indicators() {
            frameworks.push(EnhancedDetectedFramework {
                framework: Framework::Warp,
                version: None,
                confidence: 0.95,
                evidence: vec![
                    DetectionEvidence {
                        evidence_type: EvidenceType::ConfigFile,
                        source: "Cargo.toml".to_string(),
                        pattern: "warp".to_string(),
                        confidence_weight: 0.4,
                    },
                    DetectionEvidence {
                        evidence_type: EvidenceType::ImportPattern,
                        source: "*.rs files".to_string(),
                        pattern: "use warp::".to_string(),
                        confidence_weight: 0.6,
                    },
                ],
                usage_extent: UsageExtent::Core,
                ecosystem: LanguageEcosystem::Rust,
                ast_evidence: None,
            });
        }
        
        Ok(frameworks)
    }

    fn has_axum_indicators(&self) -> bool {
        self.file_contains_pattern("Cargo.toml", "axum") ||
        self.file_contains_pattern("**/*.rs", "use axum::") ||
        self.file_contains_pattern("**/*.rs", "axum::")
    }

    fn has_actix_indicators(&self) -> bool {
        self.file_contains_pattern("Cargo.toml", "actix-web") ||
        self.file_contains_pattern("**/*.rs", "use actix_web::") ||
        self.file_contains_pattern("**/*.rs", "actix_web::")
    }

    fn has_warp_indicators(&self) -> bool {
        self.file_contains_pattern("Cargo.toml", "warp") ||
        self.file_contains_pattern("**/*.rs", "use warp::") ||
        self.file_contains_pattern("**/*.rs", "warp::")
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

    /// Determine usage extent based on confidence score
    fn determine_usage_extent(&self, confidence: f32) -> UsageExtent {
        if confidence >= 0.9 {
            UsageExtent::Core
        } else if confidence >= 0.7 {
            UsageExtent::Extensive
        } else if confidence >= 0.5 {
            UsageExtent::Moderate
        } else {
            UsageExtent::Limited
        }
    }

    /// Generate evidence list for Flask detection
    fn generate_flask_evidence(&self) -> Vec<DetectionEvidence> {
        let mut evidence = Vec::new();
        
        if self.file_contains_pattern("**/*.py", "from flask import") {
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::ImportPattern,
                source: "Python files".to_string(),
                pattern: "from flask import".to_string(),
                confidence_weight: 0.4,
            });
        }
        
        if self.file_contains_pattern("**/*.py", "@app.route") {
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::DecoratorPattern,
                source: "Python files".to_string(),
                pattern: "@app.route".to_string(),
                confidence_weight: 0.35,
            });
        }
        
        if self.file_contains_pattern("requirements.txt", "Flask") {
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::ConfigFile,
                source: "requirements.txt".to_string(),
                pattern: "Flask dependency".to_string(),
                confidence_weight: 0.25,
            });
        }
        
        evidence
    }

    /// Generate evidence list for FastAPI detection
    fn generate_fastapi_evidence(&self) -> Vec<DetectionEvidence> {
        let mut evidence = Vec::new();
        
        if self.file_contains_pattern("**/*.py", "from fastapi import") {
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::ImportPattern,
                source: "Python files".to_string(),
                pattern: "from fastapi import".to_string(),
                confidence_weight: 0.4,
            });
        }
        
        if self.file_contains_pattern("**/*.py", "@app.get") {
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::DecoratorPattern,
                source: "Python files".to_string(),
                pattern: "FastAPI route decorators".to_string(),
                confidence_weight: 0.35,
            });
        }
        
        if self.file_contains_pattern("**/*.py", "FastAPI()") {
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::FrameworkInstance,
                source: "Python files".to_string(),
                pattern: "FastAPI() instantiation".to_string(),
                confidence_weight: 0.35,
            });
        }
        
        evidence
    }

    /// Generate evidence list for React detection
    fn generate_react_evidence(&self) -> Vec<DetectionEvidence> {
        let mut evidence = Vec::new();
        
        if self.file_contains_pattern("package.json", "\"react\":") {
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::ConfigFile,
                source: "package.json".to_string(),
                pattern: "React dependency".to_string(),
                confidence_weight: 0.4,
            });
        }
        
        if self.file_contains_pattern("**/*.tsx", "import React") {
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::ImportPattern,
                source: "TypeScript files".to_string(),
                pattern: "React imports".to_string(),
                confidence_weight: 0.35,
            });
        }
        
        if self.file_contains_pattern("**/*.tsx", "useState") {
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::HookPattern,
                source: "TypeScript files".to_string(),
                pattern: "React hooks usage".to_string(),
                confidence_weight: 0.3,
            });
        }
        
        evidence
    }

    /// Generate evidence list for NestJS detection
    fn generate_nestjs_evidence(&self) -> Vec<DetectionEvidence> {
        let mut evidence = Vec::new();
        
        if self.file_contains_pattern("package.json", "@nestjs/core") {
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::ConfigFile,
                source: "package.json".to_string(),
                pattern: "@nestjs/core dependency".to_string(),
                confidence_weight: 0.4,
            });
        }
        
        if self.file_contains_pattern("**/*.ts", "@Controller") {
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::DecoratorPattern,
                source: "TypeScript files".to_string(),
                pattern: "NestJS decorators".to_string(),
                confidence_weight: 0.35,
            });
        }
        
        if self.file_contains_pattern("**/*.ts", "NestFactory.create") {
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::FrameworkInstance,
                source: "TypeScript files".to_string(),
                pattern: "NestJS application bootstrap".to_string(),
                confidence_weight: 0.3,
            });
        }
        
        evidence
    }

    /// Generate evidence list for Spring Boot detection
    fn generate_spring_evidence(&self) -> Vec<DetectionEvidence> {
        let mut evidence = Vec::new();
        
        if self.file_contains_pattern("pom.xml", "spring-boot-starter") {
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::ConfigFile,
                source: "pom.xml".to_string(),
                pattern: "Spring Boot starter dependencies".to_string(),
                confidence_weight: 0.4,
            });
        }
        
        if self.file_contains_pattern("**/*.java", "@SpringBootApplication") {
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::AnnotationPattern,
                source: "Java files".to_string(),
                pattern: "@SpringBootApplication".to_string(),
                confidence_weight: 0.4,
            });
        }
        
        if self.file_contains_pattern("**/*.java", "@RestController") {
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::AnnotationPattern,
                source: "Java files".to_string(),
                pattern: "Spring MVC annotations".to_string(),
                confidence_weight: 0.35,
            });
        }
        
        evidence
    }
}

#[derive(Debug, Clone)]
struct TraditionalDetectionResult {
    pub primary_ecosystem: LanguageEcosystem,
    pub detected_frameworks: Vec<EnhancedDetectedFramework>,
    pub confidence_summary: HashMap<Framework, f32>,
}