# Codebase Workflow Analyzer - Architecture Overview

## System Architecture

The Codebase Workflow Analyzer employs a hierarchical 4-tier analysis system designed to eliminate the "segment myopia" problem while maintaining high performance and accuracy.

## Core Architecture Principles

### 1. Hierarchical Context-Aware Analysis
- **Project-Level Understanding**: Global context before segment analysis
- **Multi-Tier Integration**: Traditional patterns + AST + LLM semantic analysis
- **Context Propagation**: Each analysis layer informed by previous layers
- **Confidence Fusion**: Weighted scoring across analysis tiers

### 2. Local-First Privacy Design
- **No External Dependencies**: All analysis performed locally
- **Secure Content Handling**: Content sanitization and access controls
- **Model Context Protocol**: Secure interface between analyzer and LLM
- **Docker Containerization**: Isolated LLM execution environment

### 3. Plugin-Ready Extension System
- **Framework Detectors**: Modular language and framework support
- **Business Domain Engines**: Extensible domain classification
- **Document Generators**: Pluggable output format support
- **LLM Backends**: Swappable local LLM implementations

## Architecture Layers

### Layer 1: Project Context Analysis
**Purpose**: Establish global project understanding to prevent misclassification

**Components**:
- `ProjectAnalyzer`: Metadata extraction and project purpose inference
- `ProjectClassifier`: Type-based project categorization
- `MetadataReader`: Package.json, Cargo.toml, README analysis
- `EntryPointDetector`: Main file and application structure detection

**Key Features**:
- Project type classification (AnalysisTool, WebApplication, CLI, etc.)
- Purpose inference from documentation and project structure
- Domain hints extraction from README and docs folder
- Entry point analysis for application architecture understanding

```
Project Context Analysis
├── Metadata Extraction
│   ├── package.json / Cargo.toml analysis
│   ├── README.md purpose detection  
│   └── docs/ folder documentation parsing
├── Entry Point Detection
│   ├── main.rs / index.ts / app.py identification
│   ├── CLI vs Web vs Library structure analysis
│   └── Application architecture pattern detection
└── Project Classification
    ├── 20+ project type categories
    ├── Context-aware business domain hints
    └── Cross-reference validation
```

### Layer 2: Framework Detection Engine
**Purpose**: Identify technical frameworks and libraries with high confidence

**Components**:
- `FrameworkDetector`: Multi-language framework identification
- `DependencyAnalyzer`: Package and import analysis
- `PatternMatcher`: Structural code pattern detection
- `ConfidenceScorer`: Statistical confidence calculation

**Supported Technologies**:
- **TypeScript/JavaScript**: React, Next.js, NestJS, Express, Vue, Angular
- **Java**: Spring Boot, Spring Framework, Maven, Gradle
- **Python**: Django, Flask, FastAPI, Poetry, pip requirements
- **Rust**: Cargo ecosystem, web frameworks (Axum, Rocket, Actix)

```
Framework Detection Pipeline
├── Dependency Analysis
│   ├── Package manifest parsing
│   ├── Import/require statement analysis
│   └── Version compatibility checking
├── Structural Pattern Detection
│   ├── File organization patterns
│   ├── Naming convention analysis
│   └── Architecture pattern identification
└── Evidence Collection
    ├── Multiple evidence source validation
    ├── Statistical confidence scoring
    └── Cross-framework dependency resolution
```

### Layer 3: AST-Based Semantic Analysis
**Purpose**: Extract structural code patterns and semantic understanding

**Components**:
- `AstAnalyzer`: Tree-sitter based parsing for multiple languages
- `SegmentExtractor`: Intelligent code segmentation
- `SemanticAnalyzer`: Code structure and relationship analysis
- `BusinessPatternDetector`: Business logic pattern recognition

**Analysis Capabilities**:
- Function and class structure extraction
- API endpoint pattern detection
- Database entity relationship mapping
- Component hierarchy analysis
- Cross-file dependency tracking

```
AST Analysis Pipeline
├── Multi-Language Parsing
│   ├── Tree-sitter integration
│   ├── Language-specific extractors
│   └── Syntax tree normalization
├── Semantic Segmentation
│   ├── Function/class boundary detection
│   ├── Module and package organization
│   └── API endpoint extraction
└── Pattern Recognition
    ├── Business logic identification
    ├── Data flow analysis
    └── Architecture pattern detection
```

### Layer 4: LLM-Enhanced Business Intelligence
**Purpose**: Context-aware business domain classification and semantic understanding

**Components**:
- `LlmClient`: Local Ollama integration with Llama-3.2-3B
- `ContextManager`: Hierarchical context management
- `DomainGroundingEngine`: Business domain classification
- `PromptTemplateManager`: Project-type-specific prompting

**Intelligence Features**:
- Context-aware business domain classification
- Intent inference from code patterns
- User story extraction from implementation
- Technical debt and status detection

```
LLM Business Intelligence
├── Context-Aware Prompting
│   ├── Project-type-specific templates
│   ├── Hierarchical context injection
│   └── Domain-grounded classification
├── Semantic Analysis
│   ├── Business purpose inference
│   ├── User journey extraction
│   └── Feature status detection
└── Result Validation
    ├── Cross-tier confidence scoring
    ├── Evidence validation
    └── Accuracy calibration
```

## Data Flow Architecture

### Analysis Pipeline
```
Input: Project Directory
    ↓
[Layer 1] Project Context Analysis
    ↓ (Project Context + Metadata)
[Layer 2] Framework Detection
    ↓ (Framework Info + Project Context)
[Layer 3] AST Semantic Analysis  
    ↓ (Code Segments + Framework + Project Context)
[Layer 4] LLM Business Intelligence
    ↓ (Enhanced Analysis + All Previous Context)
Fusion Engine: Multi-Tier Result Integration
    ↓
Output: Comprehensive Analysis Report
```

### Context Propagation Model
```
Project Context (Global)
├── Framework Context (Per-Language)
│   ├── AST Context (Per-File)
│   │   ├── Segment Context (Per-Function/Class)
│   │   │   └── LLM Analysis (Context-Aware)
│   │   └── Business Pattern Recognition
│   └── Cross-Framework Integration
└── Hierarchical Result Fusion
```

## Performance Architecture

### Caching Strategy
```
Analysis Cache Hierarchy
├── Project-Level Cache
│   ├── Framework detection results
│   ├── Project metadata analysis
│   └── Global context information
├── File-Level Cache  
│   ├── AST parsing results
│   ├── Segment extraction
│   └── Individual file analysis
└── LLM Response Cache
    ├── Prompt-response pairs
    ├── Context-specific results
    └── Batch processing optimization
```

### Batch Processing Optimization
- **Intelligent Batching**: Group similar segments for efficient LLM processing
- **Context Window Management**: Optimal token usage within model limits
- **Parallel Processing**: Concurrent analysis across different tiers
- **Incremental Analysis**: Delta-based updates for changed files

## Security Architecture

### Content Sanitization Pipeline
```
Code Content Processing
├── File Access Control
│   ├── Path traversal prevention
│   ├── File type validation
│   └── Size limit enforcement
├── Content Sanitization
│   ├── Sensitive data detection
│   ├── Secret pattern removal
│   └── PII filtering
└── Secure LLM Communication
    ├── Model Context Protocol (MCP)
    ├── Sandboxed execution
    └── Result validation
```

### Privacy Guarantees
- **Local Processing**: No data leaves the local environment
- **No Network Calls**: All analysis performed offline
- **Secure Containers**: Docker isolation for LLM execution
- **Audit Logging**: Complete analysis trail for enterprise use

## Integration Architecture

### Input Sources
- **File System**: Direct codebase analysis
- **Git Integration**: Repository metadata and history
- **Package Managers**: Dependency and configuration analysis
- **Documentation**: README, docs folder, inline comments

### Output Formats
```
Document Generation Pipeline
├── Structured Data (YAML)
│   ├── Framework analysis results
│   ├── Business domain classifications
│   └── Hierarchical confidence scores
├── Executive Reports (Markdown)
│   ├── Business context summaries
│   ├── Technical architecture analysis
│   └── Implementation status reports  
└── Integration Formats
    ├── CCPM-compatible task imports
    ├── Claude Code Spec context
    └── Custom JSON/XML exports
```

### External System Integration
- **CCPM Workflow**: Automated requirement extraction
- **Claude Code Spec**: Technical context for development continuation
- **CI/CD Integration**: Automated analysis on code changes
- **Project Management**: Task and story extraction

## Scalability Architecture

### Performance Targets
- **Small Projects** (<100 files): <30 seconds complete analysis
- **Medium Projects** (100-500 files): <2 minutes complete analysis  
- **Large Projects** (500+ files): <5 minutes complete analysis
- **Enterprise Repositories**: <10 minutes with incremental updates

### Resource Management
```
Resource Utilization Strategy
├── Memory Management
│   ├── Streaming file processing
│   ├── Lazy AST loading
│   └── LLM context optimization
├── CPU Optimization
│   ├── Multi-threaded analysis
│   ├── Parallel tier processing
│   └── Efficient caching
└── Storage Efficiency
    ├── Compressed cache storage
    ├── Incremental result updates
    └── Configurable retention policies
```

## Development Architecture

### Module Organization
```
src/
├── core/                           # Core analysis engines
│   ├── project_analyzer.rs         # NEW: Project context analysis
│   ├── project_classifier.rs       # NEW: Project type classification
│   ├── framework_detector.rs       # Enhanced framework detection
│   ├── enhanced_framework_detector.rs  # Tier integration
│   ├── business_domain_engine.rs   # Business intelligence
│   ├── ast_analyzer.rs            # AST parsing and analysis
│   ├── hierarchical_context_manager.rs  # NEW: Context management
│   └── confidence_fusion_engine.rs # NEW: Multi-tier fusion
├── analyzers/                      # Language-specific analyzers
│   ├── typescript/                 # TypeScript/JavaScript analysis
│   ├── java/                      # Java framework analysis
│   ├── python/                    # Python framework analysis
│   └── rust/                      # NEW: Rust ecosystem analysis
├── intelligence/                   # LLM integration layer
│   ├── llm_client.rs              # Enhanced Ollama integration
│   ├── domain_grounding_engine.rs  # NEW: Business context grounding
│   └── prompt_template_manager.rs  # NEW: Context-aware prompting
├── generators/                     # Document generation
│   ├── yaml_analysis.rs           # Structured analysis reports
│   ├── executive_summary.rs       # Business context summaries
│   ├── technical_docs.rs          # Technical documentation
│   ├── prd.rs                     # Product requirements
│   ├── stories.rs                 # User story extraction
│   └── ccpm_import.rs             # CCPM integration
├── testing/                        # NEW: Comprehensive testing
│   ├── hierarchical_validation.rs  # Multi-tier test validation
│   ├── accuracy_benchmarks.rs     # Performance and accuracy testing
│   └── sample_projects/           # Test project repository
└── cli/                           # Command-line interface
    └── mod.rs                     # Enhanced CLI with new features
```

### Extension Points
- **Framework Detectors**: Add new language/framework support
- **Business Domain Engines**: Custom domain classification
- **LLM Backends**: Alternative local LLM integration
- **Output Generators**: Custom document formats
- **Integration Adapters**: External system connections

This architecture enables the analyzer to evolve from a pattern-matching tool into an intelligent codebase understanding system that rivals human-level comprehension for business domain classification and technical analysis.