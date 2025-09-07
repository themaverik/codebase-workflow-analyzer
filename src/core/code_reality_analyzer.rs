use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::fs;

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::core::config::get_config;
use crate::core::documentation_claims_extractor::{ClaimType, ClaimPriority};

/// Types of implementation reality that can be detected from code
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RealityType {
    AuthenticationImplemented,    // JWT tokens, OAuth flows, sessions
    ApiEndpointsImplemented,      // REST endpoints, GraphQL resolvers
    DatabaseIntegration,          // Models, schemas, queries
    SecurityImplemented,          // Encryption, validation, sanitization
    IntegrationImplemented,       // Third-party API calls, webhooks
    PerformanceOptimized,         // Caching, indexing, async operations
    TestingImplemented,           // Unit tests, integration tests
    DeploymentReady,              // Docker files, CI/CD configs
}

/// Evidence of implementation found in code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationEvidence {
    pub reality_type: RealityType,
    pub description: String,
    pub source_files: Vec<PathBuf>,
    pub line_numbers: Vec<usize>,
    pub confidence: f32,
    pub implementation_level: ImplementationLevel,
    pub code_snippets: Vec<String>,
    pub dependencies: Vec<String>,
    pub patterns_matched: Vec<String>,
}

/// Level of implementation completeness
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImplementationLevel {
    Complete,      // Fully implemented with error handling
    Partial,       // Basic implementation, missing edge cases
    Skeleton,      // Interface/structure only, no logic
    Placeholder,   // TODO/stub/mock implementations
}

/// Results from analyzing code reality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeRealityResult {
    pub implementations: Vec<ImplementationEvidence>,
    pub summary: RealitySummary,
    pub metadata: RealityAnalysisMetadata,
}

/// Summary statistics for code reality analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealitySummary {
    pub total_implementations: usize,
    pub implementations_by_type: HashMap<RealityType, usize>,
    pub implementations_by_level: HashMap<ImplementationLevel, usize>,
    pub fully_implemented_features: usize,
    pub partially_implemented_features: usize,
    pub placeholder_implementations: usize,
    pub overall_implementation_score: f32,
}

/// Metadata about the code reality analysis process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealityAnalysisMetadata {
    pub files_analyzed: usize,
    pub analysis_time_ms: u64,
    pub patterns_matched: usize,
    pub dependencies_discovered: usize,
    pub code_lines_scanned: usize,
}

/// Configuration for code reality analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeRealityConfig {
    pub enable_reality_analysis: bool,
    pub scan_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub max_file_size_kb: usize,
    pub implementation_patterns: HashMap<RealityType, Vec<String>>,
    pub dependency_patterns: HashMap<String, RealityType>,
    pub confidence_weights: HashMap<String, f32>,
    pub implementation_level_thresholds: ImplementationThresholds,
}

/// Thresholds for determining implementation levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationThresholds {
    pub complete_threshold: f32,      // 0.8+ = Complete
    pub partial_threshold: f32,       // 0.5+ = Partial  
    pub skeleton_threshold: f32,      // 0.2+ = Skeleton
    // Below skeleton_threshold = Placeholder
}

impl Default for CodeRealityConfig {
    fn default() -> Self {
        let mut implementation_patterns = HashMap::new();
        
        // Authentication implementation patterns
        implementation_patterns.insert(RealityType::AuthenticationImplemented, vec![
            r"(?i)\b(jwt|jsonwebtoken|passport|oauth|session|cookie|auth|login|logout)\b".to_string(),
            r"(?i)\b(bcrypt|scrypt|argon2|hash|salt|verify)\b".to_string(),
            r"(?i)\b(authenticate|authorize|isAuthenticated|requireAuth)\b".to_string(),
            r"(?i)\b(token|bearer|authorization|credentials)\b".to_string(),
            r"(?i)class\s+\w*Auth\w*|function\s+\w*Auth\w*".to_string(),
            r"(?i)@PreAuthorize|@Secured|@RolesAllowed".to_string(),
        ]);
        
        // API endpoints implementation patterns
        implementation_patterns.insert(RealityType::ApiEndpointsImplemented, vec![
            r"(?i)@(Get|Post|Put|Delete|Patch)Mapping|@RequestMapping".to_string(),
            r"(?i)@(GET|POST|PUT|DELETE|PATCH)\s*\(|@Path\s*\(".to_string(),
            r"(?i)app\.(get|post|put|delete|patch)\s*\(|router\.(get|post|put|delete|patch)\s*\(".to_string(),
            r"(?i)@RestController|@Controller|@Resource".to_string(),
            r"(?i)class\s+\w*Controller|class\s+\w*Resource|class\s+\w*Handler".to_string(),
            r"(?i)def\s+(get|post|put|delete|patch)_\w+|async\s+def\s+(get|post|put|delete|patch)_\w+".to_string(),
        ]);
        
        // Database integration patterns
        implementation_patterns.insert(RealityType::DatabaseIntegration, vec![
            r"(?i)@Entity|@Table|@Column|@Id|@GeneratedValue".to_string(),
            r"(?i)@Repository|@Service|@Transactional".to_string(),
            r"(?i)class\s+\w*(Model|Entity|Schema)\b|def\s+\w*(Model|Entity|Schema)\b".to_string(),
            r"(?i)\b(SELECT|INSERT|UPDATE|DELETE|CREATE TABLE|ALTER TABLE)\b".to_string(),
            r"(?i)\b(save|find|update|delete|query|execute)\s*\(|\.save\(\)|\.find\(\)".to_string(),
            r"(?i)(mongoose|sequelize|typeorm|prisma|sqlalchemy|django\.db)".to_string(),
        ]);
        
        // Security implementation patterns
        implementation_patterns.insert(RealityType::SecurityImplemented, vec![
            r"(?i)\b(encrypt|decrypt|cipher|hash|sanitize|validate|escape)\b".to_string(),
            r"(?i)\b(cors|csrf|xss|injection|sql.*injection)\b".to_string(),
            r"(?i)\b(https|ssl|tls|certificate)\b".to_string(),
            r"(?i)@Valid|@NotNull|@Size|@Pattern|@Email".to_string(),
            r"(?i)(helmet|express-rate-limit|csurf)".to_string(),
            r"(?i)(input.*validation|parameter.*validation|request.*validation)".to_string(),
        ]);
        
        // Integration implementation patterns
        implementation_patterns.insert(RealityType::IntegrationImplemented, vec![
            r"(?i)\b(axios|fetch|http|rest|api)\s*\.|\.get\(|\.post\(|\.put\(|\.delete\(".to_string(),
            r"(?i)(stripe|paypal|twilio|sendgrid|mailchimp|slack|discord)".to_string(),
            r"(?i)@FeignClient|@WebServiceClient|RestTemplate|WebClient".to_string(),
            r"(?i)webhook|callback|integration|external.*service".to_string(),
            r"(?i)(oauth|sso|social.*login|third.*party)".to_string(),
        ]);
        
        // Performance optimization patterns
        implementation_patterns.insert(RealityType::PerformanceOptimized, vec![
            r"(?i)\b(cache|redis|memcache|async|await|promise|thread|pool)\b".to_string(),
            r"(?i)@Cacheable|@CacheEvict|@EnableCaching".to_string(),
            r"(?i)(index|constraint|foreign.*key|primary.*key)".to_string(),
            r"(?i)(lazy.*load|eager.*load|pagination|limit|offset)".to_string(),
            r"(?i)(compress|minify|optimize|performance)".to_string(),
        ]);
        
        // Testing implementation patterns
        implementation_patterns.insert(RealityType::TestingImplemented, vec![
            r"(?i)@Test|@TestCase|@Before|@After|@BeforeEach|@AfterEach".to_string(),
            r"(?i)\b(test|spec|describe|it|should|expect|assert)\b".to_string(),
            r"(?i)(junit|testng|pytest|jest|mocha|jasmine|rspec)".to_string(),
            r"(?i)class\s+\w*Test|def\s+test_\w+|function\s+test\w+".to_string(),
            r"(?i)(mock|stub|spy|fixture|setUp|tearDown)".to_string(),
        ]);
        
        // Deployment readiness patterns
        implementation_patterns.insert(RealityType::DeploymentReady, vec![
            r"(?i)FROM\s+\w+|COPY|RUN|CMD|ENTRYPOINT".to_string(),  // Dockerfile
            r"(?i)(docker|kubernetes|helm|compose)".to_string(),
            r"(?i)(ci|cd|github.*actions?|jenkins|gitlab.*ci)".to_string(),
            r"(?i)(deploy|deployment|infrastructure|terraform)".to_string(),
            r"(?i)(environment|env|config|settings)".to_string(),
        ]);
        
        let mut dependency_patterns = HashMap::new();
        dependency_patterns.insert("spring-security".to_string(), RealityType::AuthenticationImplemented);
        dependency_patterns.insert("passport".to_string(), RealityType::AuthenticationImplemented);
        dependency_patterns.insert("jsonwebtoken".to_string(), RealityType::AuthenticationImplemented);
        dependency_patterns.insert("bcrypt".to_string(), RealityType::AuthenticationImplemented);
        dependency_patterns.insert("express".to_string(), RealityType::ApiEndpointsImplemented);
        dependency_patterns.insert("fastapi".to_string(), RealityType::ApiEndpointsImplemented);
        dependency_patterns.insert("spring-boot-starter-web".to_string(), RealityType::ApiEndpointsImplemented);
        dependency_patterns.insert("mongoose".to_string(), RealityType::DatabaseIntegration);
        dependency_patterns.insert("sequelize".to_string(), RealityType::DatabaseIntegration);
        dependency_patterns.insert("prisma".to_string(), RealityType::DatabaseIntegration);
        dependency_patterns.insert("stripe".to_string(), RealityType::IntegrationImplemented);
        dependency_patterns.insert("twilio".to_string(), RealityType::IntegrationImplemented);
        dependency_patterns.insert("redis".to_string(), RealityType::PerformanceOptimized);
        dependency_patterns.insert("jest".to_string(), RealityType::TestingImplemented);
        dependency_patterns.insert("junit".to_string(), RealityType::TestingImplemented);
        
        let mut confidence_weights = HashMap::new();
        confidence_weights.insert("class_definition".to_string(), 0.3);
        confidence_weights.insert("method_implementation".to_string(), 0.4);
        confidence_weights.insert("dependency_usage".to_string(), 0.2);
        confidence_weights.insert("annotation_usage".to_string(), 0.3);
        confidence_weights.insert("pattern_complexity".to_string(), 0.2);
        
        Self {
            enable_reality_analysis: true,
            scan_patterns: vec![
                "**/*.java".to_string(),
                "**/*.ts".to_string(),
                "**/*.js".to_string(),
                "**/*.py".to_string(),
                "**/*.rs".to_string(),
                "**/*.go".to_string(),
                "**/package.json".to_string(),
                "**/Cargo.toml".to_string(),
                "**/pom.xml".to_string(),
                "**/build.gradle".to_string(),
                "**/requirements.txt".to_string(),
                "**/Dockerfile".to_string(),
                "**/*.yml".to_string(),
                "**/*.yaml".to_string(),
            ],
            exclude_patterns: vec![
                "**/node_modules/**".to_string(),
                "**/target/**".to_string(),
                "**/build/**".to_string(),
                "**/.git/**".to_string(),
                "**/dist/**".to_string(),
                "**/coverage/**".to_string(),
            ],
            max_file_size_kb: 1000,
            implementation_patterns,
            dependency_patterns,
            confidence_weights,
            implementation_level_thresholds: ImplementationThresholds {
                complete_threshold: 0.8,
                partial_threshold: 0.5,
                skeleton_threshold: 0.2,
            },
        }
    }
}

/// Analyzer for detecting actual implementations in code
pub struct CodeRealityAnalyzer {
    config: CodeRealityConfig,
    compiled_patterns: HashMap<RealityType, Vec<Regex>>,
}

impl CodeRealityAnalyzer {
    /// Create new code reality analyzer with configuration from global config
    pub fn new() -> Result<Self> {
        // For now, use default configuration until global config integration is complete
        let config = CodeRealityConfig::default();
        Self::with_config(config)
    }
    
    /// Create code reality analyzer with custom configuration
    pub fn with_config(config: CodeRealityConfig) -> Result<Self> {
        let mut analyzer = Self {
            config,
            compiled_patterns: HashMap::new(),
        };
        
        analyzer.compile_patterns()
            .context("Failed to compile code reality patterns")?;
            
        println!("Initialized CodeRealityAnalyzer with {} pattern types", 
                 analyzer.compiled_patterns.len());
        Ok(analyzer)
    }
    
    /// Compile regex patterns for implementation detection
    fn compile_patterns(&mut self) -> Result<()> {
        for (reality_type, patterns) in &self.config.implementation_patterns {
            let mut compiled_patterns = Vec::new();
            
            for pattern in patterns {
                let regex = Regex::new(pattern)
                    .with_context(|| format!("Failed to compile pattern: {}", pattern))?;
                compiled_patterns.push(regex);
            }
            
            self.compiled_patterns.insert(reality_type.clone(), compiled_patterns);
        }
        
        println!("Compiled {} implementation patterns", 
                 self.compiled_patterns.values().map(|v| v.len()).sum::<usize>());
        Ok(())
    }
    
    /// Analyze project for actual implementations
    pub fn analyze_reality<P: AsRef<Path>>(&self, project_path: P) -> Result<CodeRealityResult> {
        if !self.config.enable_reality_analysis {
            return Ok(CodeRealityResult {
                implementations: Vec::new(),
                summary: RealitySummary {
                    total_implementations: 0,
                    implementations_by_type: HashMap::new(),
                    implementations_by_level: HashMap::new(),
                    fully_implemented_features: 0,
                    partially_implemented_features: 0,
                    placeholder_implementations: 0,
                    overall_implementation_score: 0.0,
                },
                metadata: RealityAnalysisMetadata {
                    files_analyzed: 0,
                    analysis_time_ms: 0,
                    patterns_matched: 0,
                    dependencies_discovered: 0,
                    code_lines_scanned: 0,
                },
            });
        }
        
        let start_time = std::time::Instant::now();
        let project_path = project_path.as_ref();
        
        println!("Starting code reality analysis for project: {}", project_path.display());
        
        let mut all_implementations = Vec::new();
        let mut files_analyzed = 0;
        let mut code_lines_scanned = 0;
        
        // Find and process source files
        let source_files = self.find_source_files(project_path)?;
        
        for file_path in source_files {
            if let Ok(implementations) = self.analyze_file(&file_path) {
                // Count lines in file
                if let Ok(content) = fs::read_to_string(&file_path) {
                    code_lines_scanned += content.lines().count();
                }
                
                all_implementations.extend(implementations);
                files_analyzed += 1;
            }
        }
        
        // Analyze dependencies
        let dependency_implementations = self.analyze_dependencies(project_path)?;
        all_implementations.extend(dependency_implementations);
        
        // Merge similar implementations
        let merged_implementations = self.merge_similar_implementations(all_implementations);
        
        let analysis_time = start_time.elapsed().as_millis() as u64;
        let summary = self.generate_reality_summary(&merged_implementations);
        let metadata = RealityAnalysisMetadata {
            files_analyzed,
            analysis_time_ms: analysis_time,
            patterns_matched: merged_implementations.iter().map(|i| i.patterns_matched.len()).sum(),
            dependencies_discovered: merged_implementations.iter().map(|i| i.dependencies.len()).sum(),
            code_lines_scanned,
        };
        
        println!("Code reality analysis completed: {} implementations found in {} files in {}ms",
                 merged_implementations.len(), files_analyzed, analysis_time);
                 
        Ok(CodeRealityResult {
            implementations: merged_implementations,
            summary,
            metadata,
        })
    }
    
    /// Find source files matching scan patterns
    fn find_source_files<P: AsRef<Path>>(&self, project_path: P) -> Result<Vec<PathBuf>> {
        let project_path = project_path.as_ref();
        let mut files = Vec::new();
        
        self.scan_directory_recursive(project_path, &mut files)?;
        
        // Filter by scan patterns and exclude patterns
        files.retain(|file| {
            let file_str = file.to_string_lossy();
            
            // Check exclude patterns first
            for exclude_pattern in &self.config.exclude_patterns {
                if self.matches_glob_pattern(&file_str, exclude_pattern) {
                    return false;
                }
            }
            
            // Check scan patterns
            for scan_pattern in &self.config.scan_patterns {
                if self.matches_glob_pattern(&file_str, scan_pattern) {
                    return true;
                }
            }
            
            false
        });
        
        Ok(files)
    }
    
    /// Recursively scan directory for files
    fn scan_directory_recursive<P: AsRef<Path>>(&self, dir_path: P, files: &mut Vec<PathBuf>) -> Result<()> {
        let dir_path = dir_path.as_ref();
        
        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                if path.is_dir() {
                    self.scan_directory_recursive(&path, files)?;
                } else {
                    files.push(path);
                }
            }
        }
        
        Ok(())
    }
    
    /// Simple glob pattern matching
    fn matches_glob_pattern(&self, text: &str, pattern: &str) -> bool {
        if pattern.contains("**") {
            // Handle recursive glob patterns
            let parts: Vec<&str> = pattern.split("**").collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                return text.contains(prefix) && text.ends_with(suffix.trim_start_matches('/'));
            }
        }
        
        if pattern.contains('*') {
            // Handle simple wildcards
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                return text.starts_with(parts[0]) && text.ends_with(parts[1]);
            }
        }
        
        // Exact match
        text.contains(pattern)
    }
    
    /// Analyze a single source file for implementations
    fn analyze_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<ImplementationEvidence>> {
        let file_path = file_path.as_ref();
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;
            
        // Check file size limit
        if content.len() > self.config.max_file_size_kb * 1024 {
            return Ok(Vec::new());
        }
        
        let lines: Vec<&str> = content.lines().collect();
        let mut implementations = Vec::new();
        
        for (reality_type, patterns) in &self.compiled_patterns {
            let mut matched_patterns = Vec::new();
            let mut code_snippets = Vec::new();
            let mut line_numbers = Vec::new();
            
            for (line_num, line) in lines.iter().enumerate() {
                for (pattern_idx, pattern) in patterns.iter().enumerate() {
                    if pattern.is_match(line) {
                        matched_patterns.push(format!("Pattern_{}", pattern_idx));
                        code_snippets.push(line.trim().to_string());
                        line_numbers.push(line_num + 1);
                    }
                }
            }
            
            if !matched_patterns.is_empty() {
                let confidence = self.calculate_implementation_confidence(
                    reality_type, 
                    &matched_patterns, 
                    &code_snippets,
                    file_path
                );
                
                let implementation_level = self.determine_implementation_level(confidence);
                let description = self.generate_implementation_description(reality_type, &code_snippets);
                
                let evidence = ImplementationEvidence {
                    reality_type: reality_type.clone(),
                    description,
                    source_files: vec![file_path.to_path_buf()],
                    line_numbers,
                    confidence,
                    implementation_level,
                    code_snippets,
                    dependencies: Vec::new(), // Will be populated later
                    patterns_matched: matched_patterns,
                };
                
                implementations.push(evidence);
            }
        }
        
        Ok(implementations)
    }
    
    /// Analyze project dependencies for implementation evidence
    fn analyze_dependencies<P: AsRef<Path>>(&self, project_path: P) -> Result<Vec<ImplementationEvidence>> {
        let project_path = project_path.as_ref();
        let mut implementations = Vec::new();
        
        // Check package.json
        if let Ok(package_json) = fs::read_to_string(project_path.join("package.json")) {
            if let Ok(package_data) = serde_json::from_str::<serde_json::Value>(&package_json) {
                let dependencies = self.extract_dependencies(&package_data);
                implementations.extend(self.convert_dependencies_to_implementations(&dependencies, "package.json"));
            }
        }
        
        // Check Cargo.toml
        if let Ok(cargo_toml) = fs::read_to_string(project_path.join("Cargo.toml")) {
            let dependencies = self.extract_cargo_dependencies(&cargo_toml);
            implementations.extend(self.convert_dependencies_to_implementations(&dependencies, "Cargo.toml"));
        }
        
        // Check requirements.txt
        if let Ok(requirements) = fs::read_to_string(project_path.join("requirements.txt")) {
            let dependencies = self.extract_python_dependencies(&requirements);
            implementations.extend(self.convert_dependencies_to_implementations(&dependencies, "requirements.txt"));
        }
        
        Ok(implementations)
    }
    
    /// Extract dependencies from package.json
    fn extract_dependencies(&self, package_data: &serde_json::Value) -> Vec<String> {
        let mut dependencies = Vec::new();
        
        if let Some(deps) = package_data.get("dependencies").and_then(|d| d.as_object()) {
            for (name, _version) in deps {
                dependencies.push(name.clone());
            }
        }
        
        if let Some(dev_deps) = package_data.get("devDependencies").and_then(|d| d.as_object()) {
            for (name, _version) in dev_deps {
                dependencies.push(name.clone());
            }
        }
        
        dependencies
    }
    
    /// Extract dependencies from Cargo.toml (simple implementation)
    fn extract_cargo_dependencies(&self, cargo_content: &str) -> Vec<String> {
        let mut dependencies = Vec::new();
        let mut in_dependencies = false;
        
        for line in cargo_content.lines() {
            let line = line.trim();
            
            if line.starts_with("[dependencies") {
                in_dependencies = true;
                continue;
            } else if line.starts_with('[') && in_dependencies {
                in_dependencies = false;
                continue;
            }
            
            if in_dependencies && line.contains('=') {
                if let Some(dep_name) = line.split('=').next() {
                    dependencies.push(dep_name.trim().to_string());
                }
            }
        }
        
        dependencies
    }
    
    /// Extract dependencies from requirements.txt
    fn extract_python_dependencies(&self, requirements_content: &str) -> Vec<String> {
        let mut dependencies = Vec::new();
        
        for line in requirements_content.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                if let Some(package_name) = line.split(&['=', '>', '<', '!', '~'][..]).next() {
                    dependencies.push(package_name.trim().to_string());
                }
            }
        }
        
        dependencies
    }
    
    /// Convert dependencies to implementation evidence
    fn convert_dependencies_to_implementations(&self, dependencies: &[String], source_file: &str) -> Vec<ImplementationEvidence> {
        let mut implementations = Vec::new();
        
        for dependency in dependencies {
            if let Some(reality_type) = self.config.dependency_patterns.get(dependency) {
                let evidence = ImplementationEvidence {
                    reality_type: reality_type.clone(),
                    description: format!("Uses {} dependency", dependency),
                    source_files: vec![PathBuf::from(source_file)],
                    line_numbers: vec![0],
                    confidence: 0.7, // Dependencies indicate implementation intent
                    implementation_level: ImplementationLevel::Partial,
                    code_snippets: vec![format!("Dependency: {}", dependency)],
                    dependencies: vec![dependency.clone()],
                    patterns_matched: vec!["dependency_analysis".to_string()],
                };
                
                implementations.push(evidence);
            }
        }
        
        implementations
    }
    
    /// Calculate confidence score for implementation evidence
    fn calculate_implementation_confidence(
        &self, 
        reality_type: &RealityType, 
        patterns_matched: &[String],
        code_snippets: &[String],
        file_path: &Path
    ) -> f32 {
        let mut confidence = 0.3; // Base confidence
        
        // Boost confidence based on pattern matches
        confidence += patterns_matched.len() as f32 * 0.1;
        
        // Boost confidence based on code complexity
        let avg_snippet_length = code_snippets.iter().map(|s| s.len()).sum::<usize>() / code_snippets.len().max(1);
        if avg_snippet_length > 50 {
            confidence += 0.2;
        }
        
        // Boost confidence based on file type and context
        let file_ext = file_path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
        match reality_type {
            RealityType::TestingImplemented => {
                if file_path.to_string_lossy().contains("test") || file_ext == "test" {
                    confidence += 0.3;
                }
            },
            RealityType::DeploymentReady => {
                if file_path.file_name().and_then(|n| n.to_str()) == Some("Dockerfile") {
                    confidence += 0.4;
                }
            },
            _ => {}
        }
        
        // Penalty for very simple patterns
        if code_snippets.iter().any(|s| s.len() < 10) {
            confidence -= 0.1;
        }
        
        confidence.min(1.0).max(0.0)
    }
    
    /// Determine implementation level based on confidence
    fn determine_implementation_level(&self, confidence: f32) -> ImplementationLevel {
        if confidence >= self.config.implementation_level_thresholds.complete_threshold {
            ImplementationLevel::Complete
        } else if confidence >= self.config.implementation_level_thresholds.partial_threshold {
            ImplementationLevel::Partial
        } else if confidence >= self.config.implementation_level_thresholds.skeleton_threshold {
            ImplementationLevel::Skeleton
        } else {
            ImplementationLevel::Placeholder
        }
    }
    
    /// Generate implementation description based on reality type and code snippets
    fn generate_implementation_description(&self, reality_type: &RealityType, code_snippets: &[String]) -> String {
        let base_description = match reality_type {
            RealityType::AuthenticationImplemented => "Authentication system with",
            RealityType::ApiEndpointsImplemented => "API endpoints providing",
            RealityType::DatabaseIntegration => "Database integration with",
            RealityType::SecurityImplemented => "Security measures including",
            RealityType::IntegrationImplemented => "Third-party integration with",
            RealityType::PerformanceOptimized => "Performance optimization using",
            RealityType::TestingImplemented => "Testing implementation with",
            RealityType::DeploymentReady => "Deployment configuration for",
        };
        
        // Extract key features from code snippets
        let mut features = HashSet::new();
        for snippet in code_snippets.iter().take(5) { // Limit to first 5 snippets
            let words: Vec<&str> = snippet.split_whitespace().take(3).collect();
            if !words.is_empty() {
                features.insert(words.join(" "));
            }
        }
        
        let features_list: Vec<_> = features.into_iter().take(3).collect();
        
        if features_list.is_empty() {
            format!("{} implementation found", base_description)
        } else {
            format!("{} {}", base_description, features_list.join(", "))
        }
    }
    
    /// Merge similar implementations to avoid duplicates
    fn merge_similar_implementations(&self, implementations: Vec<ImplementationEvidence>) -> Vec<ImplementationEvidence> {
        let mut merged = Vec::new();
        let mut processed = HashSet::new();
        
        for (i, impl1) in implementations.iter().enumerate() {
            if processed.contains(&i) {
                continue;
            }
            
            let mut merged_impl = impl1.clone();
            processed.insert(i);
            
            // Find similar implementations
            for (j, impl2) in implementations.iter().enumerate().skip(i + 1) {
                if processed.contains(&j) {
                    continue;
                }
                
                if impl1.reality_type == impl2.reality_type && 
                   self.calculate_description_similarity(&impl1.description, &impl2.description) > 0.7 {
                    
                    // Merge implementations
                    merged_impl.source_files.extend(impl2.source_files.clone());
                    merged_impl.line_numbers.extend(impl2.line_numbers.clone());
                    merged_impl.code_snippets.extend(impl2.code_snippets.clone());
                    merged_impl.dependencies.extend(impl2.dependencies.clone());
                    merged_impl.patterns_matched.extend(impl2.patterns_matched.clone());
                    
                    // Use higher confidence
                    merged_impl.confidence = merged_impl.confidence.max(impl2.confidence);
                    merged_impl.implementation_level = if matches!(merged_impl.implementation_level, ImplementationLevel::Complete) ||
                                                           matches!(impl2.implementation_level, ImplementationLevel::Complete) {
                        ImplementationLevel::Complete
                    } else if matches!(merged_impl.implementation_level, ImplementationLevel::Partial) ||
                              matches!(impl2.implementation_level, ImplementationLevel::Partial) {
                        ImplementationLevel::Partial
                    } else {
                        merged_impl.implementation_level
                    };
                    
                    processed.insert(j);
                }
            }
            
            // Remove duplicates from merged implementation
            merged_impl.source_files.sort();
            merged_impl.source_files.dedup();
            merged_impl.code_snippets.sort();
            merged_impl.code_snippets.dedup();
            merged_impl.dependencies.sort();
            merged_impl.dependencies.dedup();
            
            merged.push(merged_impl);
        }
        
        merged
    }
    
    /// Calculate similarity between two descriptions
    fn calculate_description_similarity(&self, desc1: &str, desc2: &str) -> f32 {
        if desc1 == desc2 {
            return 1.0;
        }
        
        let words1: HashSet<_> = desc1.split_whitespace().collect();
        let words2: HashSet<_> = desc2.split_whitespace().collect();
        
        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();
        
        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }
    
    /// Generate summary statistics for code reality analysis
    fn generate_reality_summary(&self, implementations: &[ImplementationEvidence]) -> RealitySummary {
        let mut implementations_by_type = HashMap::new();
        let mut implementations_by_level = HashMap::new();
        let mut fully_implemented_features = 0;
        let mut partially_implemented_features = 0;
        let mut placeholder_implementations = 0;
        
        for implementation in implementations {
            *implementations_by_type.entry(implementation.reality_type.clone()).or_insert(0) += 1;
            *implementations_by_level.entry(implementation.implementation_level.clone()).or_insert(0) += 1;
            
            match implementation.implementation_level {
                ImplementationLevel::Complete => fully_implemented_features += 1,
                ImplementationLevel::Partial => partially_implemented_features += 1,
                ImplementationLevel::Skeleton => {},
                ImplementationLevel::Placeholder => placeholder_implementations += 1,
            }
        }
        
        // Calculate overall implementation score
        let total = implementations.len() as f32;
        let overall_implementation_score = if total == 0.0 {
            0.0
        } else {
            (fully_implemented_features as f32 * 1.0 + partially_implemented_features as f32 * 0.5) / total
        };
        
        RealitySummary {
            total_implementations: implementations.len(),
            implementations_by_type,
            implementations_by_level,
            fully_implemented_features,
            partially_implemented_features,
            placeholder_implementations,
            overall_implementation_score,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_implementation_level_determination() {
        let config = CodeRealityConfig::default();
        let analyzer = CodeRealityAnalyzer::with_config(config).unwrap();
        
        assert_eq!(analyzer.determine_implementation_level(0.9), ImplementationLevel::Complete);
        assert_eq!(analyzer.determine_implementation_level(0.6), ImplementationLevel::Partial);
        assert_eq!(analyzer.determine_implementation_level(0.3), ImplementationLevel::Skeleton);
        assert_eq!(analyzer.determine_implementation_level(0.1), ImplementationLevel::Placeholder);
    }
    
    #[test]
    fn test_code_reality_analyzer_creation() {
        let analyzer = CodeRealityAnalyzer::new();
        assert!(analyzer.is_ok());
    }
    
    #[test]
    fn test_dependency_extraction() {
        let analyzer = CodeRealityAnalyzer::new().unwrap();
        
        let package_json = serde_json::json!({
            "dependencies": {
                "express": "^4.18.0",
                "passport": "^0.6.0"
            },
            "devDependencies": {
                "jest": "^28.0.0"
            }
        });
        
        let deps = analyzer.extract_dependencies(&package_json);
        assert!(deps.contains(&"express".to_string()));
        assert!(deps.contains(&"passport".to_string()));
        assert!(deps.contains(&"jest".to_string()));
    }
    
    #[test]
    fn test_reality_analysis_on_sample() {
        let temp_dir = TempDir::new().unwrap();
        
        let sample_code = r#"
import jwt from 'jsonwebtoken';
import bcrypt from 'bcrypt';

class AuthController {
    async login(req, res) {
        const { username, password } = req.body;
        const hashedPassword = await bcrypt.hash(password, 10);
        const token = jwt.sign({ userId: user.id }, 'secret');
        res.json({ token });
    }
}

app.post('/api/login', authController.login);
app.get('/api/users', requireAuth, userController.getUsers);
"#;
        
        std::fs::create_dir(temp_dir.path().join("src")).unwrap();
        std::fs::write(temp_dir.path().join("src/auth.js"), sample_code).unwrap();
        
        let package_json = r#"{
  "dependencies": {
    "express": "^4.18.0",
    "jsonwebtoken": "^8.5.1",
    "bcrypt": "^5.0.1"
  }
}"#;
        std::fs::write(temp_dir.path().join("package.json"), package_json).unwrap();
        
        let analyzer = CodeRealityAnalyzer::new().unwrap();
        let result = analyzer.analyze_reality(temp_dir.path()).unwrap();
        
        assert!(!result.implementations.is_empty());
        
        let auth_implementations: Vec<_> = result.implementations.iter()
            .filter(|i| matches!(i.reality_type, RealityType::AuthenticationImplemented))
            .collect();
        assert!(!auth_implementations.is_empty());
        
        let api_implementations: Vec<_> = result.implementations.iter()
            .filter(|i| matches!(i.reality_type, RealityType::ApiEndpointsImplemented))
            .collect();
        assert!(!api_implementations.is_empty());
    }
}