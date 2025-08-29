# Hierarchical Implementation Plan: Research-Enhanced Codebase Analyzer

## Executive Summary

Combining our existing framework detection with the research paper's hierarchical approach creates a powerful 3-tier analysis system:

**Tier 1**: Framework Detection (Current) → **Tier 2**: AST-Based Segmentation (New) → **Tier 3**: LLM Business Domain Classification (New)

## Implementation Phases

### Phase 2A: AST Integration Foundation (3-4 days)

#### Goals
- Integrate tree-sitter for multi-language AST parsing
- Enhance existing analyzers with semantic code segmentation
- Bridge current pattern-based detection with structured code analysis

#### Core Components

```rust
// src/core/ast_analyzer.rs
pub struct ASTAnalyzer {
    parsers: HashMap<Language, Parser>,
    segment_extractors: HashMap<Language, SegmentExtractor>,
    existing_detectors: HashMap<Framework, Box<dyn FrameworkAnalyzer>>, // Bridge to current system
}

pub struct CodeSegment {
    segment_type: SegmentType,  // Function, Class, Interface, Route, etc.
    content: String,
    metadata: SegmentMetadata,
    framework_context: Option<Framework>, // Link to our existing detection
    business_hints: Vec<String>, // Extracted patterns for domain inference
}

pub enum SegmentType {
    Function(FunctionSegment),
    Class(ClassSegment),
    Interface(InterfaceSegment),
    Route(RouteSegment),        // Web framework routes
    Configuration(ConfigSegment), // Framework config files
    Database(DatabaseSegment),   // Models, schemas
}
```

### Phase 2B: Local LLM Integration (4-5 days)

#### Ollama Setup and Model Management

```rust
// src/intelligence/llm_client.rs
pub struct LocalLLMManager {
    ollama_client: OllamaClient,
    model_config: ModelConfig,
    context_manager: ContextManager,
    prompt_templates: PromptTemplateEngine,
}

pub struct ModelConfig {
    model_name: String,           // "llama3.2:3b-instruct-q4_K_M"
    context_window: usize,        // 128K tokens
    temperature: f32,             // 0.1 for consistent analysis
    max_tokens: usize,            // 4K for analysis responses
    timeout: Duration,            // 60 seconds per request
}
```

### Phase 2C: Hierarchical Analysis Pipeline (3-4 days)

#### Two-Step Analysis Implementation

```rust
// src/core/hierarchical_analyzer.rs
pub struct HierarchicalAnalyzer {
    framework_detector: FrameworkDetector,  // Tier 1: Current system
    ast_analyzer: ASTAnalyzer,              // Tier 2: Code segmentation
    llm_manager: LocalLLMManager,           // Tier 3: Semantic analysis
    fusion_engine: AnalysisFusionEngine,    // Combine all tiers
}

impl HierarchicalAnalyzer {
    pub async fn analyze_codebase(&self, codebase_path: &Path) -> Result<HierarchicalAnalysisResult, AnalysisError> {
        println!("Starting hierarchical codebase analysis...");
        
        // Step 1: Framework Detection (Tier 1) - 5-10 seconds
        println!("  Detecting frameworks and languages...");
        let framework_result = self.framework_detector.analyze_with_ast(codebase_path)?;
        
        // Step 2: Semantic Segmentation (Tier 2) - 10-20 seconds  
        println!("  Extracting code segments...");
        let segments = self.ast_analyzer.extract_semantic_segments(
            codebase_path,
            &framework_result.detected_frameworks
        )?;
        
        // Step 3: LLM Analysis (Tier 3) - 60-120 seconds
        println!("  Analyzing business domains with local LLM...");
        let llm_analysis = self.llm_manager.analyze_code_segments(
            &segments,
            AnalysisType::BusinessDomain
        ).await?;
        
        // Step 4: Result Fusion - 1-2 seconds
        println!("  Combining analysis results...");
        let final_result = self.fusion_engine.fuse_analysis_results(
            framework_result,
            segments,
            llm_analysis
        )?;
        
        println!("Analysis complete!");
        Ok(final_result)
    }
}
```

### Phase 2D: Enhanced Reporting (2 days)

#### Rich Analysis Output

```rust
pub struct HierarchicalAnalysisResult {
    pub codebase_metadata: CodebaseMetadata,
    pub framework_analysis: EnhancedFrameworkResult,
    pub business_domains: Vec<FusedBusinessDomain>, 
    pub code_structure: CodeStructureAnalysis,
    pub recommendations: Vec<AnalysisRecommendation>,
    pub processing_stats: ProcessingStatistics,
}
```

## Integration Timeline

### Week 1: Foundation (Phase 2A)
- **Day 1-2**: AST parser integration, basic segment extraction
- **Day 3-4**: Bridge AST analysis with existing framework detection
- **Day 5**: Testing and validation on our 6 test projects

### Week 2: LLM Integration (Phase 2B)  
- **Day 6-7**: Ollama setup, model management, prompt engineering
- **Day 8-9**: LLM client implementation, context management
- **Day 10**: LLM analysis validation and tuning

### Week 3: Hierarchical Pipeline (Phase 2C)
- **Day 11-12**: Complete analysis pipeline implementation
- **Day 13**: Result fusion and confidence scoring
- **Day 14**: Performance optimization and error handling

### Week 4: Polish & Production (Phase 2D)
- **Day 15-16**: Enhanced reporting, CLI improvements
- **Day 17**: Documentation, configuration management
- **Day 18**: Final testing, performance benchmarking

## Success Metrics

### Accuracy Improvements
- **Business Domain Detection**: From 70% → 90%+ accuracy
- **Framework Confidence**: Reduced false positives by 50%
- **New Capability**: Semantic code understanding vs. pattern matching

### Performance Targets  
- **Small Projects** (<100 files): Complete analysis in <30 seconds
- **Medium Projects** (100-500 files): Complete analysis in <2 minutes
- **Large Projects** (500+ files): Complete analysis in <5 minutes

### User Experience
- **Rich Analysis Reports**: Framework + Domain + Structure insights
- **Confidence Transparency**: Clear breakdown of analysis confidence
- **Actionable Recommendations**: Specific next steps for codebase improvement

This hierarchical implementation transforms our analyzer from a pattern-matching tool into an AI-powered codebase intelligence platform, while maintaining the security and privacy benefits of local processing.