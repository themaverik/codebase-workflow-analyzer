use std::collections::HashMap;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

use crate::core::ast_analyzer::CodeSegment;

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
        
        println!("🧠 Analyzing {} code segments with LLM...", segments.len());

        // Get appropriate prompt template
        let template = self.prompt_templates.get_template(&analysis_type);
        
        // Process segments in batches to stay within context window
        let batch_size = self.calculate_batch_size(segments);
        let mut all_analyses = Vec::new();

        for (batch_idx, batch) in segments.chunks(batch_size).enumerate() {
            println!("  📦 Processing batch {} ({} segments)", batch_idx + 1, batch.len());
            
            let batch_prompt = template.render_batch(batch)?;
            
            // Retry logic for failed requests
            let mut retries = 0;
            let max_retries = 2;
            
            let batch_result = loop {
                match self.send_analysis_request(&batch_prompt, &analysis_type).await {
                    Ok(result) => break result,
                    Err(e) if retries < max_retries => {
                        retries += 1;
                        println!("    ⚠️  Batch {} failed, retrying ({}/{})...", batch_idx + 1, retries, max_retries);
                        tokio::time::sleep(Duration::from_secs(5)).await; // Wait 5 seconds before retry
                        continue;
                    }
                    Err(e) => {
                        println!("    ❌ Batch {} failed after {} retries: {}", batch_idx + 1, max_retries, e);
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
                    println!("    ✅ Batch {} completed ({} analyses)", batch_idx + 1, count);
                }
                Err(e) => {
                    println!("    ⚠️  Failed to parse batch {} response: {}", batch_idx + 1, e);
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
}