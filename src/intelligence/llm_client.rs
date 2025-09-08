use std::collections::HashMap;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

use crate::core::ast_analyzer::CodeSegment;
use crate::core::context_types::EnhancedSegmentContext;
use crate::core::project_analyzer::{ProjectContext, ProjectMetadata};
use crate::core::project_classifier::ProjectType;

#[derive(Debug, Clone)]
pub struct LocalLLMManager {
    client: reqwest::Client,
    config: ModelConfig,
    prompt_templates: PromptTemplateEngine,
}

#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub model_name: String,
    pub ollama_url: String,
    pub context_window: usize,
    pub temperature: f32,
    pub max_tokens: usize,
    pub timeout_seconds: u64,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_name: "llama3.2:3b-instruct-q4_K_M".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            context_window: 128_000, // 128K tokens
            temperature: 0.1,        // Low temperature for consistent analysis
            max_tokens: 4096,        // Max response tokens
            timeout_seconds: 180,    // 3 minute timeout per request
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaOptions {
    temperature: f32,
    num_predict: i32,
    top_p: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaResponse {
    pub model: String,
    pub response: String,
    pub done: bool,
    pub context: Option<Vec<i64>>,
    pub total_duration: Option<u64>,
    pub eval_count: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AnalysisType {
    BusinessDomain,
    FrameworkValidation,
    CodeQuality,
    ArchitecturePattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentAnalysis {
    pub segment_id: String,
    pub primary_domain: Option<String>,
    pub confidence: f32,
    pub evidence: Vec<String>,
    pub secondary_domains: Vec<String>,
    pub quality_score: Option<f32>,
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAnalysisResult {
    pub segments: Vec<SegmentAnalysis>,
    pub summary: AnalysisSummary,
    pub project_analysis: Option<ProjectAnalysis>,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAnalysis {
    pub primary_business_domain: String,
    pub project_type: String,
    pub functional_requirements: RequirementCategory,
    pub non_functional_requirements: RequirementCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementCategory {
    pub description: String,
    pub domains: HashMap<String, DomainAnalysis>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainAnalysis {
    pub description: String,
    pub segment_ids: Vec<String>,
    pub confidence: f32,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSummary {
    pub total_segments: usize,
    pub domain_distribution: HashMap<String, usize>,
    pub average_confidence: f32,
    pub key_patterns: Vec<String>,
}

/// Context-aware prompt structure for LLM analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContextualPrompt {
    pub project_type: Option<ProjectType>,
    pub project_metadata: ProjectMetadata,
    pub business_domain_hints: Vec<String>,
    pub segment_context: EnhancedSegmentContext,
    pub hierarchical_context: String,
}

/// Context-aware analysis result with project-level understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAwareAnalysisResult {
    pub project_classification: ProjectClassification,
    pub business_domain_analysis: BusinessDomainAnalysis,
    pub segment_analyses: Vec<SegmentAnalysis>,
    pub confidence_metrics: ConfidenceMetrics,
    pub processing_metadata: ProcessingMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectClassification {
    pub inferred_project_type: ProjectType,
    pub project_type_confidence: f32,
    pub classification_evidence: Vec<String>,
    pub project_purpose_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessDomainAnalysis {
    pub primary_domain: String,
    pub secondary_domains: Vec<String>,
    pub domain_confidence: f32,
    pub business_context: String,
    pub feature_analysis: Vec<FeatureAnalysis>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureAnalysis {
    pub feature_name: String,
    pub implementation_status: String,
    pub confidence: f32,
    pub evidence_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceMetrics {
    pub overall_confidence: f32,
    pub context_coverage: f32,
    pub classification_certainty: f32,
    pub analysis_completeness: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingMetadata {
    pub analysis_time_ms: u64,
    pub context_injection_successful: bool,
    pub model_used: String,
    pub prompt_strategy: String,
}

impl LocalLLMManager {
    pub async fn new(config: Option<ModelConfig>) -> Result<Self> {
        let config = config.unwrap_or_default();
        
        // Create HTTP client with timeout
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .context("Failed to create HTTP client")?;

        // Test connectivity
        let health_url = format!("{}/api/tags", config.ollama_url);
        let _response = client
            .get(&health_url)
            .send()
            .await
            .context("Failed to connect to Ollama. Is it running?")?;

        Ok(Self {
            client,
            config,
            prompt_templates: PromptTemplateEngine::new(),
        })
    }

    pub async fn ensure_model_ready(&self) -> Result<()> {
        println!("🔍 Checking if model {} is available...", self.config.model_name);
        
        // Test with a simple prompt
        let test_request = OllamaRequest {
            model: self.config.model_name.clone(),
            prompt: "Hello".to_string(),
            stream: false,
            options: Some(OllamaOptions {
                temperature: 0.1,
                num_predict: 5,
                top_p: 0.9,
            }),
        };

        let url = format!("{}/api/generate", self.config.ollama_url);
        let response = self.client
            .post(&url)
            .json(&test_request)
            .send()
            .await
            .context("Failed to test model")?;

        if response.status().is_success() {
            println!("  ✅ Model {} is ready", self.config.model_name);
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Model test failed: {}", error_text);
        }
    }

    pub async fn analyze_code_segments(
        &self,
        segments: &[CodeSegment],
        analysis_type: AnalysisType,
    ) -> Result<BatchAnalysisResult> {
        let start_time = std::time::Instant::now();
        
        println!("Analyzing {} code segments with LLM...", segments.len());

        // Get appropriate prompt template
        let template = self.prompt_templates.get_template(&analysis_type);
        
        // Process segments in batches to stay within context window
        let batch_size = self.calculate_batch_size(segments);
        let mut all_analyses = Vec::new();

        for (batch_idx, batch) in segments.chunks(batch_size).enumerate() {
            println!("  Processing batch {} ({} segments)", batch_idx + 1, batch.len());
            
            let batch_prompt = template.render_batch(batch)?;
            
            // Retry logic for failed requests
            let mut retries = 0;
            let max_retries = 2;
            
            let batch_result = loop {
                match self.send_analysis_request(&batch_prompt, &analysis_type).await {
                    Ok(result) => break result,
                    Err(e) if retries < max_retries => {
                        retries += 1;
                        println!("    Batch {} failed, retrying ({}/{})...", batch_idx + 1, retries, max_retries);
                        tokio::time::sleep(Duration::from_secs(5)).await; // Wait 5 seconds before retry
                        continue;
                    }
                    Err(e) => {
                        println!("    Batch {} failed after {} retries: {}", batch_idx + 1, max_retries, e);
                        // Continue with remaining batches instead of failing completely
                        continue;
                    }
                }
            };
            
            // Parse the response
            match self.parse_batch_response(&batch_result, batch) {
                Ok(batch_analyses) => {
                    let count = batch_analyses.len();
                    all_analyses.extend(batch_analyses);
                    println!("    Batch {} completed ({} analyses)", batch_idx + 1, count);
                }
                Err(e) => {
                    println!("    Failed to parse batch {} response: {}", batch_idx + 1, e);
                    // Continue with remaining batches
                }
            }
        }

        let processing_time = start_time.elapsed().as_millis() as u64;

        // Create summary
        let summary = self.create_analysis_summary(&all_analyses);

        Ok(BatchAnalysisResult {
            segments: all_analyses,
            summary,
            project_analysis: None, // TODO: Extract project analysis from LLM response
            processing_time_ms: processing_time,
        })
    }

    /// Enhanced method that analyzes segments with full project context
    pub async fn analyze_enhanced_segments(
        &self,
        enhanced_segments: &[EnhancedSegmentContext],
        analysis_type: AnalysisType,
    ) -> Result<BatchAnalysisResult> {
        let start_time = std::time::Instant::now();
        
        println!("Analyzing {} enhanced segments with project context...", enhanced_segments.len());

        // Get appropriate prompt template
        let template = self.prompt_templates.get_template(&analysis_type);
        
        // Process segments in batches to stay within context window
        let batch_size = self.calculate_enhanced_batch_size(enhanced_segments);
        let mut all_analyses = Vec::new();

        for (batch_idx, batch) in enhanced_segments.chunks(batch_size).enumerate() {
            println!("  Processing enhanced batch {} ({} segments)", batch_idx + 1, batch.len());
            
            let batch_prompt = template.render_enhanced_batch(batch)?;
            
            // Retry logic for failed requests
            let mut retries = 0;
            let max_retries = 2;
            
            let batch_result = loop {
                match self.send_analysis_request(&batch_prompt, &analysis_type).await {
                    Ok(result) => break result,
                    Err(e) if retries < max_retries => {
                        retries += 1;
                        println!("    Enhanced batch {} failed, retrying ({}/{})...", batch_idx + 1, retries, max_retries);
                        tokio::time::sleep(Duration::from_secs(5)).await; // Wait 5 seconds before retry
                        continue;
                    }
                    Err(e) => {
                        println!("    Enhanced batch {} failed after {} retries: {}", batch_idx + 1, max_retries, e);
                        // Continue with remaining batches instead of failing completely
                        continue;
                    }
                }
            };
            
            // Parse the response with enhanced context
            match self.parse_enhanced_batch_response(&batch_result, batch) {
                Ok(batch_analyses) => {
                    let count = batch_analyses.len();
                    all_analyses.extend(batch_analyses);
                    println!("    Enhanced batch {} completed ({} analyses)", batch_idx + 1, count);
                }
                Err(e) => {
                    println!("    Failed to parse enhanced batch {} response: {}", batch_idx + 1, e);
                    // Continue with remaining batches
                }
            }
        }

        let processing_time = start_time.elapsed().as_millis() as u64;

        // Create summary with enhanced context information
        let summary = self.create_enhanced_analysis_summary(&all_analyses, enhanced_segments);

        // Extract project analysis from enhanced segments
        let project_analysis = if !enhanced_segments.is_empty() {
            self.extract_project_analysis(&enhanced_segments[0].project_context)
        } else {
            None
        };

        Ok(BatchAnalysisResult {
            segments: all_analyses,
            summary,
            project_analysis,
            processing_time_ms: processing_time,
        })
    }

    async fn send_analysis_request(
        &self,
        prompt: &str,
        analysis_type: &AnalysisType,
    ) -> Result<String> {
        let request = OllamaRequest {
            model: self.config.model_name.clone(),
            prompt: prompt.to_string(),
            stream: false,
            options: Some(OllamaOptions {
                temperature: self.config.temperature,
                num_predict: self.config.max_tokens as i32,
                top_p: 0.9,
            }),
        };

        let url = format!("{}/api/generate", self.config.ollama_url);
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send analysis request")?;

        if response.status().is_success() {
            let ollama_response: OllamaResponse = response
                .json()
                .await
                .context("Failed to parse Ollama response")?;
            
            Ok(ollama_response.response)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("LLM analysis request failed: {}", error_text);
        }
    }

    fn calculate_batch_size(&self, segments: &[CodeSegment]) -> usize {
        // Estimate tokens per segment (rough approximation)
        let avg_tokens_per_segment = 200;
        let template_tokens = 500; // Overhead for prompt template
        let response_tokens = 1000; // Reserve space for response
        
        let available_tokens = self.config.context_window
            .saturating_sub(template_tokens)
            .saturating_sub(response_tokens);

        let max_segments = available_tokens / avg_tokens_per_segment;
        
        // For large codebases (>100 segments), use smaller batches to reduce processing time
        let batch_limit = if segments.len() > 100 {
            10 // Smaller batches for large projects
        } else {
            20 // Normal batch size for smaller projects
        };
        
        // Ensure we process at least 1 segment, but respect batch limits
        std::cmp::max(1, std::cmp::min(max_segments, batch_limit))
    }

    fn calculate_enhanced_batch_size(&self, enhanced_segments: &[EnhancedSegmentContext]) -> usize {
        // Enhanced segments include more context, so reduce batch size
        let avg_tokens_per_segment = 400; // More tokens due to project context
        let template_tokens = 800; // More overhead for enhanced templates
        let response_tokens = 1000; // Reserve space for response
        
        let available_tokens = self.config.context_window
            .saturating_sub(template_tokens)
            .saturating_sub(response_tokens);

        let max_segments = available_tokens / avg_tokens_per_segment;
        
        // Smaller batches for enhanced segments
        let batch_limit = if enhanced_segments.len() > 50 {
            5 // Very small batches for large projects with context
        } else {
            10 // Reduced batch size for enhanced segments
        };
        
        // Ensure we process at least 1 segment, but respect batch limits
        std::cmp::max(1, std::cmp::min(max_segments, batch_limit))
    }

    fn parse_batch_response(
        &self,
        response: &str,
        batch: &[CodeSegment],
    ) -> Result<Vec<SegmentAnalysis>> {
        println!("\n🔍 Full LLM Response ({} chars):", response.len());
        println!("{}", response);
        println!("🔍 End of LLM Response\n");
        
        // Try to parse as JSON first
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(response) {
            println!("✅ LLM returned valid JSON, parsing structured response");
            return self.parse_json_response(&parsed, batch);
        }

        // Try to extract JSON from mixed text response
        if let Some(json_str) = self.extract_json_from_text(response) {
            println!("✅ Extracted JSON from mixed text response");
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_str) {
                return self.parse_json_response(&parsed, batch);
            }
        }

        println!("⚠️ LLM returned non-JSON text, falling back to keyword extraction");
        
        // Fallback to text parsing
        self.parse_text_response(response, batch)
    }

    fn parse_json_response(
        &self,
        json: &serde_json::Value,
        batch: &[CodeSegment],
    ) -> Result<Vec<SegmentAnalysis>> {
        let mut analyses = Vec::new();
        
        // Try to parse the new business-focused structure first
        if let Some(functional_reqs) = json.get("functional_requirements").and_then(|f| f.get("domains")) {
            analyses.extend(self.parse_domain_requirements(functional_reqs, "Functional")?);
        }
        
        if let Some(non_functional_reqs) = json.get("non_functional_requirements").and_then(|f| f.get("domains")) {
            analyses.extend(self.parse_domain_requirements(non_functional_reqs, "Non-Functional")?);
        }
        
        // Fallback to old structure if new structure not found
        if analyses.is_empty() {
            if let Some(segments) = json.get("segments").and_then(|s| s.as_array()) {
                for (idx, segment_json) in segments.iter().enumerate() {
                    if idx >= batch.len() {
                        break;
                    }

                    let analysis = SegmentAnalysis {
                        segment_id: format!("segment_{}", idx),
                        primary_domain: segment_json
                            .get("primary_domain")
                            .and_then(|d| d.as_str())
                            .map(|s| s.to_string()),
                        confidence: segment_json
                            .get("confidence")
                            .and_then(|c| c.as_f64())
                            .unwrap_or(0.0) as f32,
                        evidence: segment_json
                            .get("evidence")
                            .and_then(|e| e.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str())
                                    .map(|s| s.to_string())
                                    .collect()
                            })
                            .unwrap_or_default(),
                        secondary_domains: segment_json
                            .get("secondary_domains")
                            .and_then(|e| e.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str())
                                    .map(|s| s.to_string())
                                    .collect()
                            })
                            .unwrap_or_default(),
                        quality_score: segment_json
                            .get("quality_score")
                            .and_then(|q| q.as_f64())
                            .map(|q| q as f32),
                        patterns: Vec::new(),
                    };

                    analyses.push(analysis);
                }
            }
        }

        Ok(analyses)
    }
    
    fn parse_domain_requirements(
        &self,
        domains: &serde_json::Value,
        requirement_type: &str
    ) -> Result<Vec<SegmentAnalysis>> {
        let mut analyses = Vec::new();
        
        if let Some(domains_obj) = domains.as_object() {
            for (domain_name, domain_data) in domains_obj {
                let empty_vec = vec![];
                let segment_ids = domain_data
                    .get("segment_ids")
                    .and_then(|s| s.as_array())
                    .unwrap_or(&empty_vec);
                
                let confidence = domain_data
                    .get("confidence")
                    .and_then(|c| c.as_f64())
                    .unwrap_or(0.0) as f32;
                
                let evidence: Vec<String> = domain_data
                    .get("evidence")
                    .and_then(|e| e.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string())
                            .collect()
                    })
                    .unwrap_or_default();
                
                // Create analysis entries for each segment ID
                for segment_id in segment_ids {
                    if let Some(seg_id_str) = segment_id.as_str() {
                        analyses.push(SegmentAnalysis {
                            segment_id: seg_id_str.to_string(),
                            primary_domain: Some(format!("{} ({})", domain_name, requirement_type)),
                            confidence,
                            evidence: evidence.clone(),
                            secondary_domains: vec![requirement_type.to_string()],
                            quality_score: None,
                            patterns: Vec::new(),
                        });
                    }
                }
            }
        }
        
        Ok(analyses)
    }

    fn parse_text_response(
        &self,
        response: &str,
        batch: &[CodeSegment],
    ) -> Result<Vec<SegmentAnalysis>> {
        // Simple text parsing as fallback
        let mut analyses = Vec::new();
        
        // For each segment in the batch, create a basic analysis
        for (idx, _segment) in batch.iter().enumerate() {
            let analysis = SegmentAnalysis {
                segment_id: format!("segment_{}", idx),
                primary_domain: self.extract_domain_from_text(response),
                confidence: 0.5, // Default confidence for text parsing
                evidence: vec![response.chars().take(100).collect()],
                secondary_domains: Vec::new(),
                quality_score: None,
                patterns: Vec::new(),
            };
            analyses.push(analysis);
        }

        Ok(analyses)
    }

    fn extract_json_from_text(&self, text: &str) -> Option<String> {
        // Try to find JSON block in the response
        // Look for the pattern: { ... } spanning multiple lines
        let text = text.trim();
        
        // Find the first { and last }
        if let Some(start) = text.find('{') {
            if let Some(end) = text.rfind('}') {
                if end > start {
                    let json_candidate = &text[start..=end];
                    // Validate that this looks like proper JSON
                    if json_candidate.contains("\"project_analysis\"") || json_candidate.contains("\"segments\"") {
                        return Some(json_candidate.to_string());
                    }
                }
            }
        }
        
        None
    }

    fn extract_domain_from_text(&self, text: &str) -> Option<String> {
        let text_lower = text.to_lowercase();
        
        let domain_keywords = [
            ("authentication", "Authentication"),
            ("auth", "Authentication"),
            ("user", "User Management"),
            ("notification", "Notification"),
            ("payment", "Payment"),
            ("analytics", "Analytics"),
            ("e-commerce", "E-commerce"),
            ("ecommerce", "E-commerce"),
        ];

        for (keyword, domain) in &domain_keywords {
            if text_lower.contains(keyword) {
                return Some(domain.to_string());
            }
        }

        None
    }

    fn create_analysis_summary(&self, analyses: &[SegmentAnalysis]) -> AnalysisSummary {
        let mut domain_distribution = HashMap::new();
        let mut total_confidence = 0.0;
        let mut confidence_count = 0;

        for analysis in analyses {
            if let Some(domain) = &analysis.primary_domain {
                *domain_distribution.entry(domain.clone()).or_insert(0) += 1;
            }
            
            if analysis.confidence > 0.0 {
                total_confidence += analysis.confidence;
                confidence_count += 1;
            }
        }

        let average_confidence = if confidence_count > 0 {
            total_confidence / confidence_count as f32
        } else {
            0.0
        };

        // Extract key patterns from all analyses
        let mut pattern_counts = HashMap::new();
        for analysis in analyses {
            for pattern in &analysis.patterns {
                *pattern_counts.entry(pattern.clone()).or_insert(0) += 1;
            }
        }

        let key_patterns: Vec<String> = pattern_counts
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .map(|(pattern, _)| pattern)
            .collect();

        AnalysisSummary {
            total_segments: analyses.len(),
            domain_distribution,
            average_confidence,
            key_patterns,
        }
    }

    pub async fn test_connection(&self) -> Result<bool> {
        match self.ensure_model_ready().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    fn parse_enhanced_batch_response(
        &self,
        response: &str,
        batch: &[EnhancedSegmentContext],
    ) -> Result<Vec<SegmentAnalysis>> {
        println!("\nFull LLM Response ({} chars):", response.len());
        println!("{}", response);
        println!("End of LLM Response\n");
        
        // Try to parse as JSON first
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(response) {
            println!("LLM returned valid JSON, parsing structured response");
            return self.parse_enhanced_json_response(&parsed, batch);
        }

        // Try to extract JSON from mixed text response
        if let Some(json_str) = self.extract_json_from_text(response) {
            println!("Extracted JSON from mixed text response");
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_str) {
                return self.parse_enhanced_json_response(&parsed, batch);
            }
        }

        println!("LLM returned non-JSON text, falling back to enhanced keyword extraction");
        
        // Fallback to enhanced text parsing with context
        self.parse_enhanced_text_response(response, batch)
    }

    fn parse_enhanced_json_response(
        &self,
        json: &serde_json::Value,
        batch: &[EnhancedSegmentContext],
    ) -> Result<Vec<SegmentAnalysis>> {
        let mut analyses = Vec::new();
        
        // Try to parse the new business-focused structure first
        if let Some(functional_reqs) = json.get("functional_requirements").and_then(|f| f.get("domains")) {
            analyses.extend(self.parse_domain_requirements(functional_reqs, "Functional")?);
        }
        
        if let Some(non_functional_reqs) = json.get("non_functional_requirements").and_then(|f| f.get("domains")) {
            analyses.extend(self.parse_domain_requirements(non_functional_reqs, "Non-Functional")?);
        }
        
        // Fallback to old structure if new structure not found
        if analyses.is_empty() {
            if let Some(segments) = json.get("segments").and_then(|s| s.as_array()) {
                for (idx, segment_json) in segments.iter().enumerate() {
                    if idx >= batch.len() {
                        break;
                    }

                    let analysis = SegmentAnalysis {
                        segment_id: format!("enhanced_segment_{}", idx),
                        primary_domain: segment_json
                            .get("primary_domain")
                            .and_then(|d| d.as_str())
                            .map(|s| s.to_string()),
                        confidence: segment_json
                            .get("confidence")
                            .and_then(|c| c.as_f64())
                            .unwrap_or(0.0) as f32,
                        evidence: segment_json
                            .get("evidence")
                            .and_then(|e| e.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str())
                                    .map(|s| s.to_string())
                                    .collect()
                            })
                            .unwrap_or_default(),
                        secondary_domains: segment_json
                            .get("secondary_domains")
                            .and_then(|e| e.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str())
                                    .map(|s| s.to_string())
                                    .collect()
                            })
                            .unwrap_or_default(),
                        quality_score: segment_json
                            .get("quality_score")
                            .and_then(|q| q.as_f64())
                            .map(|q| q as f32),
                        patterns: Vec::new(),
                    };

                    analyses.push(analysis);
                }
            }
        }

        Ok(analyses)
    }

    fn parse_enhanced_text_response(
        &self,
        response: &str,
        batch: &[EnhancedSegmentContext],
    ) -> Result<Vec<SegmentAnalysis>> {
        // Enhanced text parsing with project context
        let mut analyses = Vec::new();
        
        // For each segment in the batch, create an enhanced analysis
        for (idx, enhanced_segment) in batch.iter().enumerate() {
            let primary_domain = self.extract_enhanced_domain_from_text(response, enhanced_segment);
            
            let analysis = SegmentAnalysis {
                segment_id: format!("enhanced_segment_{}", idx),
                primary_domain,
                confidence: 0.6, // Higher confidence for enhanced analysis
                evidence: vec![
                    format!("Project type: {}", enhanced_segment.project_context.project_type),
                    format!("File: {}", enhanced_segment.segment_context.file_context.file_path.display()),
                    response.chars().take(100).collect(),
                ],
                secondary_domains: enhanced_segment.business_hints.clone(),
                quality_score: Some(enhanced_segment.segment_context.confidence),
                patterns: enhanced_segment.architectural_context.patterns.clone(),
            };
            analyses.push(analysis);
        }

        Ok(analyses)
    }

    fn extract_enhanced_domain_from_text(&self, text: &str, enhanced_segment: &EnhancedSegmentContext) -> Option<String> {
        let text_lower = text.to_lowercase();
        
        // First check against project business domains
        for domain in &enhanced_segment.project_context.business_domains {
            if text_lower.contains(&domain.name.to_lowercase()) {
                return Some(domain.name.clone());
            }
        }

        // Then check business hints
        for hint in &enhanced_segment.business_hints {
            if text_lower.contains(&hint.to_lowercase()) {
                return Some(hint.clone());
            }
        }

        // Fallback to standard domain keywords
        self.extract_domain_from_text(text)
    }

    fn create_enhanced_analysis_summary(&self, analyses: &[SegmentAnalysis], enhanced_segments: &[EnhancedSegmentContext]) -> AnalysisSummary {
        let mut domain_distribution = HashMap::new();
        let mut total_confidence = 0.0;
        let mut confidence_count = 0;

        for analysis in analyses {
            if let Some(domain) = &analysis.primary_domain {
                *domain_distribution.entry(domain.clone()).or_insert(0) += 1;
            }
            
            if analysis.confidence > 0.0 {
                total_confidence += analysis.confidence;
                confidence_count += 1;
            }
        }

        let average_confidence = if confidence_count > 0 {
            total_confidence / confidence_count as f32
        } else {
            0.0
        };

        // Extract key patterns from enhanced segments
        let mut pattern_counts = HashMap::new();
        for enhanced_segment in enhanced_segments {
            for pattern in &enhanced_segment.architectural_context.patterns {
                *pattern_counts.entry(pattern.clone()).or_insert(0) += 1;
            }
            for hint in &enhanced_segment.business_hints {
                *pattern_counts.entry(format!("business_hint:{}", hint)).or_insert(0) += 1;
            }
        }

        let key_patterns: Vec<String> = pattern_counts
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .map(|(pattern, _)| pattern)
            .collect();

        AnalysisSummary {
            total_segments: analyses.len(),
            domain_distribution,
            average_confidence,
            key_patterns,
        }
    }

    fn extract_project_analysis(&self, project_context: &crate::core::context_types::ProjectContext) -> Option<ProjectAnalysis> {
        Some(ProjectAnalysis {
            primary_business_domain: project_context.business_domains
                .first()
                .map(|d| d.name.clone())
                .unwrap_or_else(|| "Unknown".to_string()),
            project_type: project_context.project_type.clone(),
            functional_requirements: RequirementCategory {
                description: "Core business functionalities".to_string(),
                domains: HashMap::new(), // Will be populated by analysis results
            },
            non_functional_requirements: RequirementCategory {
                description: "Supporting infrastructure and cross-cutting concerns".to_string(),
                domains: HashMap::new(), // Will be populated by analysis results
            },
        })
    }
}

// Prompt Template Engine
#[derive(Debug, Clone)]
pub struct PromptTemplateEngine {
    templates: HashMap<AnalysisType, PromptTemplate>,
}

#[derive(Debug, Clone)]
struct PromptTemplate {
    system_prompt: String,
    user_prompt_template: String,
}

impl PromptTemplateEngine {
    fn new() -> Self {
        let mut templates = HashMap::new();

        // Business domain classification template
        templates.insert(
            AnalysisType::BusinessDomain,
            PromptTemplate {
                system_prompt: r#"You are a senior software architect and business analyst analyzing a codebase to understand its business objectives and functional architecture.

Your task is to:
1. Identify the PRIMARY BUSINESS DOMAIN/OBJECTIVE of the entire project (what is the main business purpose this project serves?)
2. Classify code segments into FUNCTIONAL REQUIREMENTS (core business logic) vs NON-FUNCTIONAL REQUIREMENTS (supporting infrastructure)
3. Determine domain classifications organically based on the actual code - do not limit yourself to predefined categories

Functional Requirements: Core business logic that directly serves the primary business objective
Non-Functional Requirements: Supporting infrastructure like logging, monitoring, documentation, caching, security layers, etc.

IMPORTANT: You must respond with ONLY valid JSON. No explanatory text, no markdown formatting, no code blocks. Just pure JSON."#.to_string(),
                user_prompt_template: r#"Analyze these code segments to understand the business architecture:

{segments}

Based on the code segments, provide a comprehensive business analysis:

1. Determine the PRIMARY BUSINESS DOMAIN/OBJECTIVE of this project
2. Classify each segment as either Functional or Non-Functional requirement
3. Identify specific domain categories organically from the code

Respond with this exact JSON structure:
{
  "project_analysis": {
    "primary_business_domain": "Brief description of main business objective",
    "project_type": "e.g., Gateway Service, Authentication Service, E-commerce Platform, etc."
  },
  "functional_requirements": {
    "description": "Core business functionalities",
    "domains": {
      "Domain Name 1": {
        "description": "What this domain does",
        "segment_ids": ["segment_0", "segment_3"],
        "confidence": 0.9,
        "evidence": ["key code patterns", "function names"]
      }
    }
  },
  "non_functional_requirements": {
    "description": "Supporting infrastructure and cross-cutting concerns",
    "domains": {
      "Domain Name 2": {
        "description": "What this domain does", 
        "segment_ids": ["segment_1", "segment_2"],
        "confidence": 0.8,
        "evidence": ["logging patterns", "config management"]
      }
    }
  }
}"#.to_string(),
            },
        );

        // Framework validation template
        templates.insert(
            AnalysisType::FrameworkValidation,
            PromptTemplate {
                system_prompt: "You are validating framework detection results by analyzing code segments.".to_string(),
                user_prompt_template: r#"Based on these code segments, validate the detected frameworks:

Detected: {detected_frameworks}
Code segments: {segments}

Confirm or correct the framework detection with confidence scores."#.to_string(),
            },
        );

        Self { templates }
    }

    fn get_template(&self, analysis_type: &AnalysisType) -> &PromptTemplate {
        self.templates
            .get(analysis_type)
            .expect("Template not found for analysis type")
    }
}

impl PromptTemplate {
    fn render_batch(&self, segments: &[CodeSegment]) -> Result<String> {
        let segments_text = segments
            .iter()
            .enumerate()
            .map(|(idx, segment)| {
                format!(
                    "Segment {}: {}\nFile: {}\nType: {:?}\n---",
                    idx,
                    segment.content.chars().take(500).collect::<String>(), // Limit to 500 chars
                    segment.metadata.file_path.display(),
                    segment.segment_type
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        let user_prompt = self.user_prompt_template.replace("{segments}", &segments_text);
        
        Ok(format!(
            "{}\n\nUser: {}\n\nAssistant:",
            self.system_prompt, user_prompt
        ))
    }

    fn render_enhanced_batch(&self, enhanced_segments: &[EnhancedSegmentContext]) -> Result<String> {
        // Extract project context from the first segment (all segments share the same project context)
        let project_context = if !enhanced_segments.is_empty() {
            Some(&enhanced_segments[0].project_context)
        } else {
            None
        };

        let segments_text = enhanced_segments
            .iter()
            .enumerate()
            .map(|(idx, enhanced_segment)| {
                let segment = &enhanced_segment.segment_context.segment;
                let business_hints = if enhanced_segment.business_hints.is_empty() {
                    "None".to_string()
                } else {
                    enhanced_segment.business_hints.join(", ")
                };
                
                format!(
                    "Segment {}: {}\nFile: {}\nType: {}\nBusiness Hints: {}\nFile Role: {:?}\nArchitectural Layer: {:?}\n---",
                    idx,
                    segment.content.chars().take(500).collect::<String>(),
                    enhanced_segment.segment_context.file_context.file_path.display(),
                    segment.segment_type,
                    business_hints,
                    enhanced_segment.segment_context.file_context.role_in_project,
                    enhanced_segment.architectural_context.layer
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        // Create enhanced user prompt with project context
        let mut enhanced_user_prompt = if let Some(project_ctx) = project_context {
            let project_info = format!(
                "PROJECT CONTEXT:\n- Project Type: {}\n- Primary Business Domains: {}\n- Overall Confidence: {:.2}\n\nCODE SEGMENTS TO ANALYZE:\n{}",
                project_ctx.project_type,
                project_ctx.business_domains.iter()
                    .map(|d| format!("{} ({:.2})", d.name, d.confidence))
                    .collect::<Vec<_>>()
                    .join(", "),
                project_ctx.confidence,
                segments_text
            );
            
            self.user_prompt_template.replace("{segments}", &project_info)
        } else {
            self.user_prompt_template.replace("{segments}", &segments_text)
        };

        // Add enhanced instruction for context-aware analysis
        enhanced_user_prompt = enhanced_user_prompt.replace(
            "Based on the code segments, provide a comprehensive business analysis:",
            "Based on the PROJECT CONTEXT and code segments below, provide a comprehensive business analysis that considers the overall project purpose:"
        );

        Ok(format!(
            "{}\n\nUser: {}\n\nAssistant:",
            self.system_prompt, enhanced_user_prompt
        ))
    }
}

impl LocalLLMManager {
    /// Context-aware analysis method that integrates project-level context to fix segment myopia
    pub async fn analyze_with_project_context(
        &self,
        segments: &[CodeSegment],
        project_context: &ProjectContext,
        analysis_type: &AnalysisType,
    ) -> Result<ContextAwareAnalysisResult> {
        let start_time = std::time::Instant::now();
        
        println!("🔍 Starting context-aware analysis with project context:");
        println!("  Project: {}", project_context.metadata.name);
        if let Some(ref project_type) = project_context.project_type {
            println!("  Type: {:?} (confidence: {:.1}%)", 
                project_type, project_context.project_type_confidence * 100.0);
        }
        println!("  Segments to analyze: {}", segments.len());

        // Create contextual prompt with project-level information
        let contextual_prompt = self.create_project_contextual_prompt(project_context, segments)?;
        
        // Generate context-aware prompt template
        let prompt = self.create_context_aware_prompt(&contextual_prompt, analysis_type)?;
        
        // Send request to LLM with full project context
        let llm_response = self.send_context_aware_request(&prompt).await?;
        
        // Parse response into structured result
        let analysis_result = self.parse_context_aware_response(
            &llm_response, 
            project_context, 
            segments, 
            start_time.elapsed().as_millis() as u64
        )?;

        println!("✅ Context-aware analysis completed in {}ms", analysis_result.processing_metadata.analysis_time_ms);
        println!("  Project classification: {:?} (confidence: {:.1}%)", 
            analysis_result.project_classification.inferred_project_type,
            analysis_result.project_classification.project_type_confidence * 100.0);
        println!("  Primary domain: {} (confidence: {:.1}%)",
            analysis_result.business_domain_analysis.primary_domain,
            analysis_result.business_domain_analysis.domain_confidence * 100.0);

        Ok(analysis_result)
    }

    fn create_project_contextual_prompt(
        &self,
        project_context: &ProjectContext,
        segments: &[CodeSegment],
    ) -> Result<ProjectContextualPrompt> {
        use crate::core::context_types::{SegmentContext, FileContext, FileType, FileRole, SegmentType, ArchitecturalContext, DependencyOverview, ArchitecturalLayer, InteractionStyle};
        use std::time::SystemTime;
        
        // Create a simplified file context
        let file_context = FileContext {
            file_path: project_context.project_path.clone(),
            file_type: FileType::SourceCode,
            role_in_project: FileRole::CoreLogic,
            language: Some("rust".to_string()),
            imports: vec![],
            exports: vec![],
            key_patterns: vec![],
            related_files: vec![],
            business_relevance: 0.8,
            last_modified: SystemTime::now(),
        };

        // Create a simplified segment context
        let segment_context_inner = SegmentContext {
            segment_id: format!("batch_{}", segments.len()),
            segment: crate::core::types::AstSegment {
                file_path: project_context.project_path.clone(),
                start_line: 0,
                end_line: segments.len(),
                segment_type: "project_overview".to_string(),
                content: "Project analysis batch".to_string(),
                language: "rust".to_string(),
            },
            file_context: file_context.clone(),
            segment_type: SegmentType::FunctionDefinition,
            business_purpose: Some("Project analysis overview".to_string()),
            dependencies: vec![],
            dependents: vec![],
            confidence: 0.8,
            extracted_at: SystemTime::now(),
        };

        // Create a simplified project context for the enhanced segment context
        let project_context_inner = crate::core::context_types::ProjectContext {
            id: project_context.metadata.name.clone(),
            metadata: project_context.metadata.clone(),
            project_type: format!("{:?}", project_context.project_type.as_ref().unwrap_or(&crate::core::project_classifier::ProjectType::Unknown)),
            business_domains: vec![], // Simplified
            entry_points: project_context.entry_points.iter().map(|ep| ep.file_path.clone()).collect(),
            documentation_summary: project_context.purpose_description.clone(),
            architectural_patterns: vec![],
            dependency_overview: DependencyOverview {
                direct_dependencies: project_context.metadata.dependencies.clone(),
                framework_dependencies: vec![],
                development_dependencies: project_context.metadata.dev_dependencies.keys().cloned().collect(),
                dependency_categories: std::collections::HashMap::new(),
            },
            confidence: project_context.project_type_confidence,
            created_at: SystemTime::now(),
        };

        // Create enhanced segment context with correct field structure
        let segment_context = EnhancedSegmentContext {
            segment_context: segment_context_inner,
            project_context: project_context_inner,
            related_segments: vec![],
            cross_references: vec![],
            business_hints: project_context.business_domain_hints.clone(),
            architectural_context: ArchitecturalContext {
                layer: ArchitecturalLayer::Business,
                patterns: vec!["layered".to_string()],
                responsibilities: vec!["Project analysis".to_string(), "Code intelligence".to_string()],
                interaction_style: InteractionStyle::RequestResponse,
            },
        };

        // Build hierarchical context description
        let hierarchical_context = format!(
            "Project Context:\n- Name: {}\n- Type: {:?}\n- Description: {}\n- Domain Hints: {}\n- Entry Points: {}",
            project_context.metadata.name,
            project_context.project_type.as_ref().unwrap_or(&crate::core::project_classifier::ProjectType::Unknown),
            project_context.purpose_description,
            project_context.business_domain_hints.join(", "),
            project_context.entry_points.len()
        );

        Ok(ProjectContextualPrompt {
            project_type: project_context.project_type.clone(),
            project_metadata: project_context.metadata.clone(),
            business_domain_hints: project_context.business_domain_hints.clone(),
            segment_context,
            hierarchical_context,
        })
    }

    fn create_context_aware_prompt(
        &self,
        contextual_prompt: &ProjectContextualPrompt,
        analysis_type: &AnalysisType,
    ) -> Result<String> {
        let project_type_guidance = match contextual_prompt.project_type.as_ref() {
            Some(crate::core::project_classifier::ProjectType::AnalysisTool) => {
                "This is a codebase analysis and intelligence tool. Focus on features like static analysis, code parsing, business intelligence extraction, and development workflow automation."
            },
            Some(crate::core::project_classifier::ProjectType::WebApplication) => {
                "This is a web application. Focus on user interface, authentication, data management, and user experience features."
            },
            Some(crate::core::project_classifier::ProjectType::ApiService) => {
                "This is an API service. Focus on endpoints, data processing, authentication, and integration capabilities."
            },
            Some(crate::core::project_classifier::ProjectType::Library) => {
                "This is a library or framework. Focus on reusable components, APIs, and developer tools."
            },
            Some(crate::core::project_classifier::ProjectType::CliTool) => {
                "This is a command-line tool. Focus on command processing, file operations, and automation features."
            },
            _ => "Analyze the code segments to understand the project's purpose and business domain."
        };

        let domain_hints = if !contextual_prompt.business_domain_hints.is_empty() {
            format!("Expected business domains: {}", contextual_prompt.business_domain_hints.join(", "))
        } else {
            "Infer the business domain from the code patterns.".to_string()
        };

        let prompt = format!(
            r#"You are an expert software architect analyzing a codebase with full project context.

PROJECT CONTEXT:
{}

PROJECT TYPE GUIDANCE:
{}

DOMAIN ANALYSIS GUIDANCE:
{}

ANALYSIS TASK:
Provide a comprehensive analysis in JSON format with the following structure:

{{
  "project_classification": {{
    "inferred_project_type": "AnalysisTool|WebApplication|ApiService|Library|CliTool|Other",
    "project_type_confidence": 0.0-1.0,
    "classification_evidence": ["evidence1", "evidence2"],
    "project_purpose_description": "detailed purpose description"
  }},
  "business_domain_analysis": {{
    "primary_domain": "primary domain name",
    "secondary_domains": ["domain1", "domain2"],
    "domain_confidence": 0.0-1.0,
    "business_context": "business context description",
    "feature_analysis": [
      {{
        "feature_name": "feature name",
        "implementation_status": "complete|partial|planned",
        "confidence": 0.0-1.0,
        "evidence_files": ["file1.rs", "file2.rs"]
      }}
    ]
  }},
  "confidence_metrics": {{
    "overall_confidence": 0.0-1.0,
    "context_coverage": 0.0-1.0,
    "classification_certainty": 0.0-1.0,
    "analysis_completeness": 0.0-1.0
  }}
}}

CODE SEGMENTS TO ANALYZE:
[Segments would be inserted here - simplified for now]

Respond with ONLY the JSON structure, no additional text."#,
            contextual_prompt.hierarchical_context,
            project_type_guidance,
            domain_hints
        );

        Ok(prompt)
    }

    async fn send_context_aware_request(&self, prompt: &str) -> Result<String> {
        let request = OllamaRequest {
            model: self.config.model_name.clone(),
            prompt: prompt.to_string(),
            stream: false,
            options: Some(OllamaOptions {
                temperature: 0.1, // Low temperature for consistent analysis
                num_predict: 2048, // Allow longer responses for detailed analysis
                top_p: 0.9,
            }),
        };

        let url = format!("{}/api/generate", self.config.ollama_url);
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send context-aware analysis request")?;

        if response.status().is_success() {
            let ollama_response: OllamaResponse = response
                .json()
                .await
                .context("Failed to parse Ollama response")?;
            
            Ok(ollama_response.response)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Context-aware analysis request failed: {}", error_text);
        }
    }

    fn parse_context_aware_response(
        &self,
        response: &str,
        project_context: &ProjectContext,
        segments: &[CodeSegment],
        analysis_time_ms: u64,
    ) -> Result<ContextAwareAnalysisResult> {
        println!("🔍 Parsing context-aware response ({} chars)", response.len());

        // Try to parse as JSON
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(response) {
            return self.parse_structured_context_response(&parsed, project_context, segments, analysis_time_ms);
        }

        // Try to extract JSON from mixed response
        if let Some(json_str) = self.extract_json_from_text(response) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_str) {
                return self.parse_structured_context_response(&parsed, project_context, segments, analysis_time_ms);
            }
        }

        // Fallback to creating a structured result from text analysis
        println!("⚠️ Could not parse structured response, creating fallback analysis");
        self.create_fallback_context_result(response, project_context, segments, analysis_time_ms)
    }

    fn parse_structured_context_response(
        &self,
        parsed: &serde_json::Value,
        project_context: &ProjectContext,
        segments: &[CodeSegment],
        analysis_time_ms: u64,
    ) -> Result<ContextAwareAnalysisResult> {
        let project_classification = if let Some(pc) = parsed.get("project_classification") {
            ProjectClassification {
                inferred_project_type: self.parse_project_type(pc.get("inferred_project_type")),
                project_type_confidence: pc.get("project_type_confidence").and_then(|v| v.as_f64()).unwrap_or(0.5) as f32,
                classification_evidence: self.parse_string_array(pc.get("classification_evidence")),
                project_purpose_description: pc.get("project_purpose_description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            }
        } else {
            self.create_default_project_classification(project_context)
        };

        let business_domain_analysis = if let Some(bd) = parsed.get("business_domain_analysis") {
            BusinessDomainAnalysis {
                primary_domain: bd.get("primary_domain").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
                secondary_domains: self.parse_string_array(bd.get("secondary_domains")),
                domain_confidence: bd.get("domain_confidence").and_then(|v| v.as_f64()).unwrap_or(0.5) as f32,
                business_context: bd.get("business_context").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                feature_analysis: self.parse_feature_analysis(bd.get("feature_analysis")),
            }
        } else {
            self.create_default_business_domain_analysis(project_context)
        };

        let confidence_metrics = if let Some(cm) = parsed.get("confidence_metrics") {
            ConfidenceMetrics {
                overall_confidence: cm.get("overall_confidence").and_then(|v| v.as_f64()).unwrap_or(0.5) as f32,
                context_coverage: cm.get("context_coverage").and_then(|v| v.as_f64()).unwrap_or(0.8) as f32,
                classification_certainty: cm.get("classification_certainty").and_then(|v| v.as_f64()).unwrap_or(0.6) as f32,
                analysis_completeness: cm.get("analysis_completeness").and_then(|v| v.as_f64()).unwrap_or(0.7) as f32,
            }
        } else {
            ConfidenceMetrics {
                overall_confidence: 0.6,
                context_coverage: 0.8,
                classification_certainty: 0.7,
                analysis_completeness: 0.7,
            }
        };

        Ok(ContextAwareAnalysisResult {
            project_classification,
            business_domain_analysis,
            segment_analyses: vec![], // Simplified for now
            confidence_metrics,
            processing_metadata: ProcessingMetadata {
                analysis_time_ms,
                context_injection_successful: true,
                model_used: self.config.model_name.clone(),
                prompt_strategy: "hierarchical_context".to_string(),
            },
        })
    }

    fn parse_project_type(&self, value: Option<&serde_json::Value>) -> ProjectType {
        if let Some(v) = value.and_then(|v| v.as_str()) {
            match v.to_lowercase().as_str() {
                "analysistool" | "analysis_tool" => ProjectType::AnalysisTool,
                "webapplication" | "web_application" => ProjectType::WebApplication,
                "apiservice" | "api_service" => ProjectType::ApiService,
                "library" => ProjectType::Library,
                "clitool" | "cli_tool" => ProjectType::CliTool,
                _ => ProjectType::Unknown,
            }
        } else {
            ProjectType::Unknown
        }
    }

    fn parse_string_array(&self, value: Option<&serde_json::Value>) -> Vec<String> {
        if let Some(arr) = value.and_then(|v| v.as_array()) {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        } else {
            vec![]
        }
    }

    fn parse_feature_analysis(&self, value: Option<&serde_json::Value>) -> Vec<FeatureAnalysis> {
        if let Some(arr) = value.and_then(|v| v.as_array()) {
            arr.iter()
                .filter_map(|v| {
                    Some(FeatureAnalysis {
                        feature_name: v.get("feature_name")?.as_str()?.to_string(),
                        implementation_status: v.get("implementation_status")?.as_str()?.to_string(),
                        confidence: v.get("confidence")?.as_f64().unwrap_or(0.5) as f32,
                        evidence_files: self.parse_string_array(v.get("evidence_files")),
                    })
                })
                .collect()
        } else {
            vec![]
        }
    }

    fn create_default_project_classification(&self, project_context: &ProjectContext) -> ProjectClassification {
        ProjectClassification {
            inferred_project_type: project_context.project_type.clone().unwrap_or(ProjectType::Unknown),
            project_type_confidence: project_context.project_type_confidence,
            classification_evidence: vec!["Project metadata analysis".to_string()],
            project_purpose_description: project_context.purpose_description.clone(),
        }
    }

    fn create_default_business_domain_analysis(&self, project_context: &ProjectContext) -> BusinessDomainAnalysis {
        BusinessDomainAnalysis {
            primary_domain: if project_context.business_domain_hints.is_empty() {
                "Unknown".to_string()
            } else {
                project_context.business_domain_hints[0].clone()
            },
            secondary_domains: project_context.business_domain_hints.clone(),
            domain_confidence: 0.5,
            business_context: project_context.purpose_description.clone(),
            feature_analysis: vec![],
        }
    }

    fn create_fallback_context_result(
        &self,
        response: &str,
        project_context: &ProjectContext,
        segments: &[CodeSegment],
        analysis_time_ms: u64,
    ) -> Result<ContextAwareAnalysisResult> {
        // Create a basic analysis result when structured parsing fails
        Ok(ContextAwareAnalysisResult {
            project_classification: self.create_default_project_classification(project_context),
            business_domain_analysis: self.create_default_business_domain_analysis(project_context),
            segment_analyses: vec![],
            confidence_metrics: ConfidenceMetrics {
                overall_confidence: 0.4, // Lower confidence for fallback
                context_coverage: 0.6,
                classification_certainty: 0.5,
                analysis_completeness: 0.5,
            },
            processing_metadata: ProcessingMetadata {
                analysis_time_ms,
                context_injection_successful: false, // Failed to parse structured response
                model_used: self.config.model_name.clone(),
                prompt_strategy: "fallback_analysis".to_string(),
            },
        })
    }
}