# TODO: SOTA Hybrid Hierarchical Context-Aware Implementation

## Overview
Implement the research-backed hybrid approach to fix the current LLM "segment myopia" issue and achieve 90%+ business domain classification accuracy.

## Phase 1: Project-Level Context Analysis (3-4 days)

### Critical Fix: Eliminate Segment Myopia
**Problem**: Current LLM analysis misclassifies analyzer tool as "User Management Web App"  
**Solution**: Add project-level context understanding before segment analysis

#### Task 1.1: Project Metadata Reader (Day 1)
```rust
// NEW: src/core/project_analyzer.rs
pub struct ProjectAnalyzer {
    metadata_readers: HashMap<ProjectType, MetadataReader>,
    entry_point_detector: EntryPointDetector,
    purpose_inferrer: ProjectPurposeInferrer,
}

impl ProjectAnalyzer {
    pub fn analyze_project_context(path: &Path) -> ProjectContext {
        // 1. Read package.json, Cargo.toml, README.md
        // 2. Identify main entry points (main.rs, index.ts, app.py)  
        // 3. Infer project purpose (CLI tool, web app, library, analyzer)
        // 4. Extract domain hints from project description, documentation including README and other documents (usually markdown files organised in docs folder)
    }
}
```

#### Task 1.2: Context-Aware LLM Prompting (Day 2)
```rust
// ENHANCE: src/intelligence/llm_client.rs
impl LlmClient {
    pub async fn analyze_with_project_context(
        &self,
        segments: &[AstSegment],
        project_context: &ProjectContext // ADD THIS
    ) -> Result<BusinessAnalysis> {
        
        let grounded_prompt = format!(r#"
CRITICAL CONTEXT: You are analyzing a {project_type} in the {domain} domain.

PROJECT PURPOSE: {project_purpose}
PROBLEM CONTEXT: {problem_context}

IMPORTANT: This is a {project_classification}. Apply context-specific analysis:
- If analyzing a CODE ANALYSIS TOOL: Focus on the tool's capabilities, not analyzed content
- If analyzing a WEB APPLICATION: Focus on user-facing features and business logic
- If analyzing a LIBRARY/FRAMEWORK: Focus on provided abstractions and APIs
- If analyzing a CLI TOOL: Focus on command-line interface and automation capabilities
- If analyzing a DESKTOP APPLICATION: Focus on GUI components and desktop-specific features
- If analyzing a MOBILE APPLICATION: Focus on mobile-specific patterns and user interactions
- If analyzing a DATA PIPELINE: Focus on data processing, ETL operations, and data flow
- If analyzing a MACHINE LEARNING project: Focus on model training, inference, and AI capabilities
- If analyzing a DEVOPS TOOL: Focus on deployment, infrastructure, and automation workflows
- If analyzing a SECURITY TOOL: Focus on security analysis, vulnerability detection, and protection
- If analyzing a TESTING FRAMEWORK: Focus on test automation, validation, and quality assurance
- If analyzing a GAME ENGINE: Focus on game mechanics, rendering, and interactive features
- If analyzing a BLOCKCHAIN APP: Focus on smart contracts, decentralized features, and crypto operations
- If analyzing a MONITORING SYSTEM: Focus on observability, metrics collection, and alerting
- If analyzing a MEDIA PROCESSOR: Focus on content processing, transformation, and media handling

Project Metadata:
{project_metadata}

Code Segments to Analyze:
{segments}

Classify business domains within this project context.
"#);
    }
}
```

#### Task 1.3: Project Type Classification (Day 3)
```rust
// NEW: src/core/project_classifier.rs
#[derive(Debug, Clone)]
pub enum ProjectType {
    AnalysisTool,       // Codebase analyzers, linters, formatters, code intelligence tools
    WebApplication,     // Frontend/backend web apps, SPAs, progressive web apps
    ApiService,         // REST/GraphQL API backends, microservices
    Library,            // Reusable code libraries, SDKs, frameworks
    CliTool,            // Command-line utilities, terminal applications
    Desktop,            // Desktop applications (Electron, native GUI apps)
    Mobile,             // Mobile applications (React Native, Flutter, native iOS/Android)
    GameEngine,         // Game development frameworks and engines
    DataPipeline,       // ETL tools, data processing frameworks
    MachineLearning,    // ML/AI model training, inference systems
    DevOps,             // CI/CD tools, deployment automation, infrastructure
    EmbeddedSystem,     // IoT, firmware, embedded device software
    DatabaseSystem,     // Database engines, ORM tools, data storage
    SecurityTool,       // Security analysis, penetration testing, vulnerability scanners
    TestingFramework,   // Testing libraries, test automation, QA tools
    DocumentationSite,  // Static site generators, documentation platforms
    ConfigurationTool,  // Configuration management, environment setup
    MonitoringSystem,   // Logging, metrics, observability platforms
    BlockchainApp,      // Smart contracts, DeFi, blockchain applications
    ChatBot,            // Conversational AI, automated support systems
    MediaProcessor,     // Image/video processing, content management
    ScientificComputing, // Research tools, simulation, mathematical computing
    NetworkingTool,     // Network utilities, protocol implementations
}

pub struct ProjectClassifier;
impl ProjectClassifier {
    pub fn classify_from_metadata(metadata: &ProjectMetadata) -> ProjectType {
        // Logic to distinguish analyzer tools from web apps from CLIs
        if metadata.name.contains("analyzer") || 
           metadata.has_analysis_dependencies() ||
           metadata.has_cli_structure() {
            ProjectType::AnalysisTool
        } else if metadata.has_web_dependencies() {
            ProjectType::WebApplication
        }
        // ... other classifications
    }
}
```

#### Task 1.4: Integration Testing (Day 4)
- Test project context analysis on existing sample projects
- Validate correct classification of analyzer vs web apps
- Ensure LLM now correctly identifies our tool's purpose

## Phase 2: Multi-layered Context Integration (4-5 days)

### Task 2.1: Hierarchical Context Manager (Days 5-6)
```rust
// NEW: src/core/hierarchical_context_manager.rs
pub struct HierarchicalContextManager {
    project_context: ProjectContext,
    file_contexts: HashMap<PathBuf, FileContext>,
    segment_contexts: HashMap<SegmentId, SegmentContext>,
    cross_references: CrossReferenceMap,
}

impl HierarchicalContextManager {
    pub fn build_segment_context(&self, segment: &AstSegment) -> EnhancedSegmentContext {
        EnhancedSegmentContext {
            segment: segment.clone(),
            project_context: self.project_context.clone(),
            file_context: self.file_contexts[&segment.file_path].clone(),
            related_segments: self.find_related_segments(segment),
            business_hints: self.extract_contextual_business_hints(segment),
        }
    }
}
```

### Task 2.2: Context-Aware Segment Analysis (Days 7-8)
```rust
// ENHANCE: src/core/enhanced_framework_detector.rs
impl EnhancedFrameworkDetector {
    pub async fn analyze_with_hierarchical_context(&self) -> Result<ContextAwareAnalysis> {
        // Step 1: Project-level understanding (NEW)
        let project_context = self.project_analyzer.analyze_project_context(&self.path)?;
        
        // Step 2: Context-aware segment extraction  
        let segments = self.ast_analyzer.extract_segments_with_context(&project_context)?;
        
        // Step 3: Hierarchical LLM analysis
        let segment_analyses = self.llm_client.analyze_segments_with_context(
            &segments,
            &project_context
        ).await?;
        
        // Step 4: Context-aware result fusion
        let fused_result = self.fusion_engine.fuse_with_context(
            project_context,
            segment_analyses
        )?;
        
        Ok(fused_result)
    }
}
```

### Task 2.3: Business Context Grounding (Day 9)
```rust
// NEW: src/intelligence/domain_grounding_engine.rs
pub struct DomainGroundingEngine {
    domain_ontology: BusinessDomainOntology,
    context_templates: HashMap<ProjectType, ContextTemplate>,
    evidence_validators: Vec<EvidenceValidator>,
}

impl DomainGroundingEngine {
    pub fn ground_analysis_in_business_context(
        &self,
        analysis: &RawAnalysis,
        project_context: &ProjectContext
    ) -> GroundedAnalysis {
        // Apply project-type-specific domain grounding
        let template = &self.context_templates[&project_context.project_type];
        
        let grounded_domains = analysis.business_domains
            .iter()
            .map(|domain| self.ground_domain(domain, template, project_context))
            .collect();
            
        GroundedAnalysis {
            project_context: project_context.clone(),
            business_domains: grounded_domains,
            confidence_breakdown: self.calculate_grounded_confidence(&grounded_domains),
            validation_evidence: self.collect_validation_evidence(&grounded_domains),
        }
    }
}
```

## Phase 3: Hierarchical Result Fusion (3-4 days)

### Task 3.1: Multi-tier Confidence Scoring (Days 10-11)
```rust
// NEW: src/core/confidence_fusion_engine.rs
pub struct ConfidenceFusionEngine {
    weights: TierWeights,
    validators: Vec<Box<dyn ConfidenceValidator>>,
    calibration_data: CalibrationDataset,
}

pub struct TierWeights {
    project_context: f32,      // 0.4 - High weight for project understanding
    traditional_patterns: f32,  // 0.3 - Our current pattern matching
    ast_structure: f32,        // 0.2 - Structural evidence
    llm_semantic: f32,         // 0.1 - LLM insights (contextual)
}

impl ConfidenceFusionEngine {
    pub fn fuse_multi_tier_analysis(
        &self,
        project_context: &ProjectContext,
        traditional_analysis: &TraditionalAnalysis,
        ast_analysis: &AstAnalysis,
        llm_analysis: &LlmAnalysis
    ) -> FusedAnalysisResult {
        
        // Weight-based confidence fusion
        let fused_domains = BusinessDomain::all_domains()
            .iter()
            .filter_map(|domain| {
                let project_confidence = self.get_project_level_confidence(domain, project_context);
                let traditional_confidence = self.get_traditional_confidence(domain, traditional_analysis);
                let ast_confidence = self.get_ast_confidence(domain, ast_analysis);
                let llm_confidence = self.get_llm_confidence(domain, llm_analysis);
                
                let fused_confidence = 
                    project_confidence * self.weights.project_context +
                    traditional_confidence * self.weights.traditional_patterns +
                    ast_confidence * self.weights.ast_structure +
                    llm_confidence * self.weights.llm_semantic;
                
                if fused_confidence > 0.3 {
                    Some(FusedBusinessDomain {
                        domain: domain.clone(),
                        confidence: fused_confidence,
                        evidence_breakdown: self.collect_evidence_breakdown(domain, &[
                            project_context, traditional_analysis, ast_analysis, llm_analysis
                        ]),
                        validation_status: self.validate_domain_classification(domain, fused_confidence),
                    })
                } else {
                    None
                }
            })
            .collect();
            
        FusedAnalysisResult {
            business_domains: fused_domains,
            confidence_breakdown: self.generate_confidence_breakdown(&fused_domains),
            analysis_metadata: self.generate_analysis_metadata(),
            recommendations: self.generate_improvement_recommendations(&fused_domains),
        }
    }
}
```

### Task 3.2: Validation & Testing (Days 12-13)
```rust
// NEW: src/testing/hierarchical_validation.rs
pub struct HierarchicalValidationSuite {
    test_projects: Vec<TestProject>,
    ground_truth: HashMap<ProjectId, ExpectedResults>,
    accuracy_metrics: AccuracyMetrics,
}

impl HierarchicalValidationSuite {
    pub async fn validate_hierarchical_analysis(&self) -> ValidationReport {
        let mut results = Vec::new();
        
        for test_project in &self.test_projects {
            // Run hierarchical analysis
            let analysis_result = self.analyzer.analyze_with_hierarchical_context(
                &test_project.path
            ).await?;
            
            // Compare with ground truth
            let accuracy = self.compare_with_ground_truth(
                &analysis_result,
                &self.ground_truth[&test_project.id]
            )?;
            
            results.push(TestResult {
                project: test_project.clone(),
                analysis: analysis_result,
                accuracy,
            });
        }
        
        ValidationReport {
            overall_accuracy: self.calculate_overall_accuracy(&results),
            per_domain_accuracy: self.calculate_per_domain_accuracy(&results),
            improvement_over_baseline: self.calculate_improvement(&results),
            failed_cases: self.identify_failed_cases(&results),
            recommendations: self.generate_recommendations(&results),
        }
    }
}
```

## Phase 4: Performance Optimization (2-3 days)

### Task 4.1: Caching & Incremental Analysis (Day 14)
```rust
// NEW: src/core/analysis_cache.rs
pub struct AnalysisCache {
    project_cache: HashMap<PathBuf, CachedProjectAnalysis>,
    segment_cache: HashMap<SegmentHash, CachedSegmentAnalysis>, 
    llm_cache: HashMap<PromptHash, CachedLLMResponse>,
    cache_policy: CachePolicy,
}

impl AnalysisCache {
    pub fn get_or_analyze_project(&mut self, path: &Path) -> Result<ProjectAnalysis> {
        let project_hash = self.calculate_project_hash(path)?;
        
        if let Some(cached) = self.project_cache.get(&path) {
            if cached.is_valid(&project_hash) {
                println!("Using cached project analysis");
                return Ok(cached.analysis.clone());
            }
        }
        
        // Perform fresh analysis
        let analysis = self.perform_fresh_analysis(path)?;
        self.project_cache.insert(path.to_path_buf(), CachedProjectAnalysis {
            hash: project_hash,
            analysis: analysis.clone(),
            timestamp: SystemTime::now(),
        });
        
        Ok(analysis)
    }
}
```

### Task 4.2: Batch Processing Optimization (Day 15)
```rust
// ENHANCE: src/intelligence/llm_client.rs  
impl LlmClient {
    pub async fn analyze_segments_batch_optimized(
        &self,
        segments: &[EnhancedSegmentContext]
    ) -> Result<Vec<SegmentAnalysis>> {
        
        // Group segments by similarity for efficient batching
        let batches = self.create_optimized_batches(segments)?;
        
        // Process batches in parallel with context window management
        let batch_futures = batches.into_iter()
            .map(|batch| self.process_batch_with_context(batch))
            .collect::<Vec<_>>();
            
        let batch_results = futures::future::try_join_all(batch_futures).await?;
        
        // Flatten results while preserving order
        Ok(batch_results.into_iter().flatten().collect())
    }
    
    fn create_optimized_batches(&self, segments: &[EnhancedSegmentContext]) -> Result<Vec<Vec<EnhancedSegmentContext>>> {
        // Intelligent batching based on:
        // 1. Token count (stay within context window)
        // 2. Semantic similarity (similar segments batch better)
        // 3. Project structure (related components together)
    }
}
```

## Expected Results After Implementation

### Before (Current Issue)
```yaml
primary_business_domain: "User Management and Profile Services"  # WRONG
project_type: "Web Application"                                   # WRONG  
confidence: 0.65                                                  # LOW
evidence: ["React components", "user interface patterns"]        # MISLEADING
```

### After (Fixed with Context)
```yaml
primary_business_domain: "Codebase Intelligence & Development Workflow Automation"  # CORRECT
project_type: "Software Analysis & Code Intelligence Platform"                      # CORRECT
confidence: 0.92                                                                    # HIGH
evidence: [
  "Cargo.toml: codebase-analyzer binary",                                          # PROJECT CONTEXT
  "README.md: reverse engineering tool description", 
  "src/core/: framework detection and business domain engines",                    # AST STRUCTURE
  "CLI interface for codebase analysis"                                           # FUNCTIONAL CONTEXT
]

hierarchical_analysis:
  tier1_project_context: 0.95    # High confidence in project understanding
  tier2_framework_detection: 0.88 # Good framework identification  
  tier3_ast_segmentation: 0.90   # Strong structural analysis
  tier4_llm_semantic: 0.85       # Context-aware semantic understanding
  
functional_requirements: [
  "Framework Detection and Classification",
  "Business Domain Inference from Code Patterns", 
  "Automated Document Generation (PRDs, User Stories)",
  "Multi-language Codebase Analysis"
]

non_functional_requirements: [
  "CLI Interface for Developer Workflow Integration",
  "Local LLM Processing for Privacy",
  "Configuration Management for Analysis Parameters",  
  "Comprehensive Testing and Validation Framework"
]
```

## Success Metrics

### Accuracy Improvements
- **Business Domain Detection**: 70% → 90%+ accuracy
- **Project Type Classification**: 80% → 95%+ accuracy  
- **False Positive Reduction**: 50% reduction in misclassifications
- **Context Awareness**: Eliminate segment myopia issues

### Performance Targets
- **Analysis Speed**: Maintain <2 minutes for medium projects
- **Resource Efficiency**: 90% cache hit rate on repeated analyses
- **Batch Processing**: 3x improvement in LLM analysis throughput

### Validation Requirements
- **Ground Truth Testing**: 95%+ accuracy on 20+ diverse test projects
- **Cross-Domain Validation**: Consistent performance across business domains
- **Edge Case Handling**: Robust performance on ambiguous/complex projects

## Implementation Priority
1. **URGENT**: Fix segment myopia issue (Phase 1, Tasks 1.1-1.4)
2. **HIGH**: Multi-layered context integration (Phase 2)
3. **MEDIUM**: Result fusion optimization (Phase 3)
4. **LOW**: Performance optimization (Phase 4)

This implementation will transform the analyzer from a pattern-matching tool into a truly intelligent codebase understanding system that rivals human-level code comprehension for business domain classification.