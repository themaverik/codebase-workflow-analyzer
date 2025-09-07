use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::core::config::get_config;

/// Types of claims that can be extracted from documentation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClaimType {
    Feature,           // "Supports OAuth authentication"
    Capability,        // "Real-time notifications" 
    Integration,       // "Integrates with Stripe API"
    Technology,        // "Built with React and Node.js"
    Performance,       // "Handles 10,000 concurrent users"
    Security,          // "End-to-end encryption"
    Deployment,        // "Deploy with Docker"
    Api,              // "REST API with full CRUD operations"
}

/// Priority level for documentation claims
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClaimPriority {
    Critical,    // Core functionality claims
    High,        // Important feature claims
    Medium,      // Standard feature claims
    Low,         // Nice-to-have claims
}

impl ClaimPriority {
    /// Determine priority based on claim type and context
    fn from_context(claim_type: &ClaimType, context: &str) -> Self {
        let context_lower = context.to_lowercase();
        
        match claim_type {
            ClaimType::Security => Self::Critical,
            ClaimType::Api => {
                if context_lower.contains("authentication") || context_lower.contains("authorization") {
                    Self::Critical
                } else if context_lower.contains("crud") || context_lower.contains("rest") {
                    Self::High
                } else {
                    Self::Medium
                }
            },
            ClaimType::Feature => {
                if context_lower.contains("authentication") || context_lower.contains("user management") {
                    Self::Critical
                } else if context_lower.contains("payment") || context_lower.contains("notification") {
                    Self::High
                } else {
                    Self::Medium
                }
            },
            ClaimType::Integration => Self::High,
            ClaimType::Performance => Self::Medium,
            ClaimType::Technology | ClaimType::Capability => Self::Medium,
            ClaimType::Deployment => Self::Low,
        }
    }
}

/// A claim extracted from documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationClaim {
    pub claim_type: ClaimType,
    pub description: String,
    pub source_file: PathBuf,
    pub line_number: usize,
    pub confidence: f32,
    pub priority: ClaimPriority,
    pub keywords: Vec<String>,
    pub context: String,
    pub evidence: Vec<String>,
}

/// Results from extracting claims from documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationClaimsResult {
    pub claims: Vec<DocumentationClaim>,
    pub summary: ClaimsSummary,
    pub metadata: ClaimsExtractionMetadata,
}

/// Summary statistics for extracted claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimsSummary {
    pub total_claims: usize,
    pub claims_by_type: HashMap<ClaimType, usize>,
    pub claims_by_priority: HashMap<ClaimPriority, usize>,
    pub high_confidence_claims: usize,
    pub verification_needed: usize,
}

/// Metadata about the claims extraction process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimsExtractionMetadata {
    pub files_processed: usize,
    pub extraction_time_ms: u64,
    pub patterns_used: usize,
    pub average_confidence: f32,
}

/// Configuration for documentation claims extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimsExtractionConfig {
    pub enable_claims_extraction: bool,
    pub documentation_patterns: Vec<String>,
    pub confidence_threshold: f32,
    pub max_file_size_kb: usize,
    pub claim_patterns: HashMap<ClaimType, Vec<String>>,
    pub keyword_weights: HashMap<String, f32>,
    pub context_window_lines: usize,
}

impl Default for ClaimsExtractionConfig {
    fn default() -> Self {
        let mut claim_patterns = HashMap::new();
        
        // Feature claim patterns
        claim_patterns.insert(ClaimType::Feature, vec![
            r"(?i)\b(supports?|provides?|includes?|features?)\s+([^.!?]+)".to_string(),
            r"(?i)\b(authentication|authorization|oauth|sso|login)\b".to_string(),
            r"(?i)\b(user\s+management|user\s+registration|user\s+profiles?)\b".to_string(),
            r"(?i)\b(real[\s-]?time|notifications?|messaging)\b".to_string(),
            r"(?i)\b(payments?|billing|subscriptions?)\b".to_string(),
        ]);
        
        // API claim patterns
        claim_patterns.insert(ClaimType::Api, vec![
            r"(?i)\b(rest\s+api|restful|api\s+endpoints?)\b".to_string(),
            r"(?i)\b(graphql|grpc|websocket)\b".to_string(),
            r"(?i)\b(crud\s+operations?|create|read|update|delete)\b".to_string(),
            r"(?i)\b(json\s+api|xml\s+api)\b".to_string(),
        ]);
        
        // Integration claim patterns
        claim_patterns.insert(ClaimType::Integration, vec![
            r"(?i)\b(integrates?\s+with|connects?\s+to)\s+([^.!?]+)".to_string(),
            r"(?i)\b(stripe|paypal|twilio|sendgrid|mailchimp)\b".to_string(),
            r"(?i)\b(social\s+media|facebook|twitter|google|github)\b".to_string(),
            r"(?i)\b(third[\s-]?party|external\s+services?)\b".to_string(),
        ]);
        
        // Technology claim patterns
        claim_patterns.insert(ClaimType::Technology, vec![
            r"(?i)\b(built\s+with|using|powered\s+by)\s+([^.!?]+)".to_string(),
            r"(?i)\b(react|vue|angular|node\.js|express|fastify)\b".to_string(),
            r"(?i)\b(mongodb|postgresql|mysql|redis|elasticsearch)\b".to_string(),
            r"(?i)\b(docker|kubernetes|aws|azure|gcp)\b".to_string(),
        ]);
        
        // Security claim patterns
        claim_patterns.insert(ClaimType::Security, vec![
            r"(?i)\b(secure|security|encrypted?|encryption)\b".to_string(),
            r"(?i)\b(ssl|tls|https|certificates?)\b".to_string(),
            r"(?i)\b(end[\s-]?to[\s-]?end|e2e|zero[\s-]?trust)\b".to_string(),
            r"(?i)\b(gdpr|hipaa|compliance|audit)\b".to_string(),
        ]);
        
        // Performance claim patterns
        claim_patterns.insert(ClaimType::Performance, vec![
            r"(?i)\b(fast|performance|scalable?|scalability)\b".to_string(),
            r"(?i)\b(\d+)\s+(concurrent\s+)?(users?|requests?|connections?)\b".to_string(),
            r"(?i)\b(caching|cdn|optimization|load\s+balancing)\b".to_string(),
        ]);
        
        // Capability claim patterns
        claim_patterns.insert(ClaimType::Capability, vec![
            r"(?i)\b(can|able\s+to|capability|capabilities)\s+([^.!?]+)".to_string(),
            r"(?i)\b(mobile[\s-]?friendly|responsive|cross[\s-]?platform)\b".to_string(),
            r"(?i)\b(offline\s+support|progressive\s+web\s+app|pwa)\b".to_string(),
        ]);
        
        // Deployment claim patterns
        claim_patterns.insert(ClaimType::Deployment, vec![
            r"(?i)\b(deploy|deployment|hosting|hosted)\b".to_string(),
            r"(?i)\b(docker|kubernetes|heroku|vercel|netlify)\b".to_string(),
            r"(?i)\b(ci/cd|continuous\s+integration|continuous\s+deployment)\b".to_string(),
        ]);
        
        let mut keyword_weights = HashMap::new();
        keyword_weights.insert("authentication".to_string(), 1.0);
        keyword_weights.insert("oauth".to_string(), 0.9);
        keyword_weights.insert("api".to_string(), 0.8);
        keyword_weights.insert("crud".to_string(), 0.8);
        keyword_weights.insert("secure".to_string(), 0.9);
        keyword_weights.insert("integration".to_string(), 0.7);
        keyword_weights.insert("real-time".to_string(), 0.8);
        
        Self {
            enable_claims_extraction: true,
            documentation_patterns: vec![
                "README.md".to_string(),
                "README.rst".to_string(),
                "docs/**/*.md".to_string(),
                "API.md".to_string(),
                "FEATURES.md".to_string(),
                "CHANGELOG.md".to_string(),
                "package.json".to_string(),
                "Cargo.toml".to_string(),
            ],
            confidence_threshold: 0.6,
            max_file_size_kb: 500,
            claim_patterns,
            keyword_weights,
            context_window_lines: 3,
        }
    }
}

/// Extractor for claims from project documentation
pub struct DocumentationClaimsExtractor {
    config: ClaimsExtractionConfig,
    compiled_patterns: HashMap<ClaimType, Vec<Regex>>,
}

impl DocumentationClaimsExtractor {
    /// Create new documentation claims extractor with configuration from global config
    pub fn new() -> Result<Self> {
        // For now, use default configuration until global config integration is complete
        let config = ClaimsExtractionConfig::default();
        Self::with_config(config)
    }
    
    /// Create documentation claims extractor with custom configuration
    pub fn with_config(config: ClaimsExtractionConfig) -> Result<Self> {
        let mut extractor = Self {
            config,
            compiled_patterns: HashMap::new(),
        };
        
        extractor.compile_patterns()
            .context("Failed to compile claims extraction patterns")?;
            
        println!("Initialized DocumentationClaimsExtractor with {} pattern types", 
                 extractor.compiled_patterns.len());
        Ok(extractor)
    }
    
    /// Compile regex patterns for claim detection
    fn compile_patterns(&mut self) -> Result<()> {
        for (claim_type, patterns) in &self.config.claim_patterns {
            let mut compiled_patterns = Vec::new();
            
            for pattern in patterns {
                let regex = Regex::new(pattern)
                    .with_context(|| format!("Failed to compile pattern: {}", pattern))?;
                compiled_patterns.push(regex);
            }
            
            self.compiled_patterns.insert(claim_type.clone(), compiled_patterns);
        }
        
        println!("Compiled {} claim patterns", 
                 self.compiled_patterns.values().map(|v| v.len()).sum::<usize>());
        Ok(())
    }
    
    /// Extract claims from project documentation
    pub fn extract_claims<P: AsRef<Path>>(&self, project_path: P) -> Result<DocumentationClaimsResult> {
        if !self.config.enable_claims_extraction {
            return Ok(DocumentationClaimsResult {
                claims: Vec::new(),
                summary: ClaimsSummary {
                    total_claims: 0,
                    claims_by_type: HashMap::new(),
                    claims_by_priority: HashMap::new(),
                    high_confidence_claims: 0,
                    verification_needed: 0,
                },
                metadata: ClaimsExtractionMetadata {
                    files_processed: 0,
                    extraction_time_ms: 0,
                    patterns_used: 0,
                    average_confidence: 0.0,
                },
            });
        }
        
        let start_time = std::time::Instant::now();
        let project_path = project_path.as_ref();
        
        println!("Starting claims extraction for project: {}", project_path.display());
        
        let mut all_claims = Vec::new();
        let mut files_processed = 0;
        
        // Process documentation files
        for pattern in &self.config.documentation_patterns {
            let files = self.find_documentation_files(project_path, pattern)?;
            
            for file_path in files {
                if let Ok(claims) = self.extract_claims_from_file(&file_path) {
                    all_claims.extend(claims);
                    files_processed += 1;
                }
            }
        }
        
        // Filter by confidence threshold
        all_claims.retain(|claim| claim.confidence >= self.config.confidence_threshold);
        
        let extraction_time = start_time.elapsed().as_millis() as u64;
        let summary = self.generate_claims_summary(&all_claims);
        let metadata = ClaimsExtractionMetadata {
            files_processed,
            extraction_time_ms: extraction_time,
            patterns_used: self.compiled_patterns.len(),
            average_confidence: if all_claims.is_empty() { 
                0.0 
            } else {
                all_claims.iter().map(|c| c.confidence).sum::<f32>() / all_claims.len() as f32
            },
        };
        
        println!("Claims extraction completed: {} claims found in {} files in {}ms",
                 all_claims.len(), files_processed, extraction_time);
                 
        Ok(DocumentationClaimsResult {
            claims: all_claims,
            summary,
            metadata,
        })
    }
    
    /// Find documentation files matching patterns
    fn find_documentation_files<P: AsRef<Path>>(&self, project_path: P, pattern: &str) -> Result<Vec<PathBuf>> {
        let project_path = project_path.as_ref();
        let mut files = Vec::new();
        
        if pattern.contains("**") {
            // Handle glob patterns like "docs/**/*.md"
            // For simplicity, just check common documentation directories
            let base_dirs = ["docs", "documentation", "wiki"];
            
            for base_dir in &base_dirs {
                let dir_path = project_path.join(base_dir);
                if dir_path.exists() {
                    self.scan_directory_recursive(&dir_path, &mut files, ".md")?;
                }
            }
        } else {
            // Handle specific file patterns
            let file_path = project_path.join(pattern);
            if file_path.exists() {
                files.push(file_path);
            }
        }
        
        Ok(files)
    }
    
    /// Recursively scan directory for files with specific extension
    fn scan_directory_recursive<P: AsRef<Path>>(&self, dir_path: P, files: &mut Vec<PathBuf>, extension: &str) -> Result<()> {
        let dir_path = dir_path.as_ref();
        
        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                if path.is_dir() {
                    self.scan_directory_recursive(&path, files, extension)?;
                } else if path.extension()
                    .and_then(|ext| ext.to_str())
                    .map_or(false, |ext| ext == extension.trim_start_matches('.')) {
                    files.push(path);
                }
            }
        }
        
        Ok(())
    }
    
    /// Extract claims from a single documentation file
    fn extract_claims_from_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<DocumentationClaim>> {
        let file_path = file_path.as_ref();
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;
            
        // Check file size limit
        if content.len() > self.config.max_file_size_kb * 1024 {
            return Ok(Vec::new());
        }
        
        let lines: Vec<&str> = content.lines().collect();
        let mut claims = Vec::new();
        
        for (claim_type, patterns) in &self.compiled_patterns {
            for (line_num, line) in lines.iter().enumerate() {
                for pattern in patterns {
                    if let Some(captures) = pattern.captures(line) {
                        let description = if captures.len() > 1 {
                            captures.get(1)
                                .map(|m| m.as_str())
                                .unwrap_or(line)
                                .trim()
                                .to_string()
                        } else {
                            line.trim().to_string()
                        };
                        
                        let context = self.extract_context(&lines, line_num);
                        let keywords = self.extract_keywords(&description);
                        let confidence = self.calculate_confidence(claim_type, &description, &keywords);
                        let priority = ClaimPriority::from_context(claim_type, &context);
                        
                        let claim = DocumentationClaim {
                            claim_type: claim_type.clone(),
                            description,
                            source_file: file_path.to_path_buf(),
                            line_number: line_num + 1,
                            confidence,
                            priority,
                            keywords,
                            context,
                            evidence: vec![line.trim().to_string()],
                        };
                        
                        claims.push(claim);
                    }
                }
            }
        }
        
        // Remove duplicate claims based on similar descriptions
        claims.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        claims.dedup_by(|a, b| {
            a.claim_type == b.claim_type && 
            self.calculate_similarity(&a.description, &b.description) > 0.8
        });
        
        Ok(claims)
    }
    
    /// Extract context around a claim
    fn extract_context(&self, lines: &[&str], line_num: usize) -> String {
        let start = line_num.saturating_sub(self.config.context_window_lines);
        let end = std::cmp::min(line_num + self.config.context_window_lines + 1, lines.len());
        
        lines[start..end].join(" ").trim().to_string()
    }
    
    /// Extract keywords from claim description
    fn extract_keywords(&self, description: &str) -> Vec<String> {
        let words: Vec<String> = description
            .split_whitespace()
            .map(|word| word.to_lowercase().trim_matches(|c: char| c.is_ascii_punctuation()).to_string())
            .filter(|word| word.len() > 2)
            .collect();
            
        // Filter keywords that appear in our weight map
        words.into_iter()
            .filter(|word| self.config.keyword_weights.contains_key(word))
            .collect()
    }
    
    /// Calculate confidence score for a claim
    fn calculate_confidence(&self, claim_type: &ClaimType, description: &str, keywords: &[String]) -> f32 {
        let mut confidence = 0.5; // Base confidence
        
        // Boost confidence based on claim type
        match claim_type {
            ClaimType::Feature | ClaimType::Api => confidence += 0.2,
            ClaimType::Security => confidence += 0.3,
            ClaimType::Integration => confidence += 0.1,
            _ => confidence += 0.05,
        }
        
        // Boost confidence based on keyword matches
        for keyword in keywords {
            if let Some(weight) = self.config.keyword_weights.get(keyword) {
                confidence += weight * 0.1;
            }
        }
        
        // Penalize very short or very long descriptions
        let desc_len = description.len();
        if desc_len < 10 {
            confidence -= 0.2;
        } else if desc_len > 200 {
            confidence -= 0.1;
        }
        
        confidence.min(1.0).max(0.0)
    }
    
    /// Calculate similarity between two strings (simple implementation)
    fn calculate_similarity(&self, a: &str, b: &str) -> f32 {
        if a == b {
            return 1.0;
        }
        
        let words_a: std::collections::HashSet<_> = a.split_whitespace().collect();
        let words_b: std::collections::HashSet<_> = b.split_whitespace().collect();
        
        let intersection = words_a.intersection(&words_b).count();
        let union = words_a.union(&words_b).count();
        
        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }
    
    /// Generate summary statistics for extracted claims
    fn generate_claims_summary(&self, claims: &[DocumentationClaim]) -> ClaimsSummary {
        let mut claims_by_type = HashMap::new();
        let mut claims_by_priority = HashMap::new();
        let mut high_confidence_claims = 0;
        let mut verification_needed = 0;
        
        for claim in claims {
            *claims_by_type.entry(claim.claim_type.clone()).or_insert(0) += 1;
            *claims_by_priority.entry(claim.priority.clone()).or_insert(0) += 1;
            
            if claim.confidence > 0.8 {
                high_confidence_claims += 1;
            }
            
            if matches!(claim.claim_type, ClaimType::Feature | ClaimType::Api | ClaimType::Security) {
                verification_needed += 1;
            }
        }
        
        ClaimsSummary {
            total_claims: claims.len(),
            claims_by_type,
            claims_by_priority,
            high_confidence_claims,
            verification_needed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_claim_priority_from_context() {
        assert_eq!(
            ClaimPriority::from_context(&ClaimType::Security, "encryption"),
            ClaimPriority::Critical
        );
        
        assert_eq!(
            ClaimPriority::from_context(&ClaimType::Api, "authentication API"),
            ClaimPriority::Critical
        );
        
        assert_eq!(
            ClaimPriority::from_context(&ClaimType::Feature, "user management"),
            ClaimPriority::Critical
        );
    }
    
    #[test]
    fn test_claims_extractor_creation() {
        let extractor = DocumentationClaimsExtractor::new();
        assert!(extractor.is_ok());
    }
    
    #[test]
    fn test_pattern_compilation() {
        let config = ClaimsExtractionConfig::default();
        let extractor = DocumentationClaimsExtractor::with_config(config);
        assert!(extractor.is_ok());
    }
    
    #[test]
    fn test_claims_extraction_from_sample() {
        let temp_dir = TempDir::new().unwrap();
        let readme_content = r#"
# My Project

This project supports OAuth authentication and provides a REST API.
Features include user management and real-time notifications.

## Security
- End-to-end encryption
- HTTPS by default
- GDPR compliant

## Integration
Integrates with Stripe for payments and Twilio for SMS.
"#;
        
        fs::write(temp_dir.path().join("README.md"), readme_content).unwrap();
        
        let extractor = DocumentationClaimsExtractor::new().unwrap();
        let result = extractor.extract_claims(temp_dir.path()).unwrap();
        
        assert!(!result.claims.is_empty());
        assert!(result.claims.iter().any(|c| matches!(c.claim_type, ClaimType::Feature)));
        assert!(result.claims.iter().any(|c| matches!(c.claim_type, ClaimType::Api)));
        assert!(result.claims.iter().any(|c| matches!(c.claim_type, ClaimType::Security)));
    }
}