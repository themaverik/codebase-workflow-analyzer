# Codebase Workflow Analyzer

## Project Overview
Advanced reverse engineering tool that transforms existing codebases into systematic development workflows, generating PRDs, user stories, and task breakdowns with AI-powered business domain inference and intelligent status detection.

## Architecture
- **Language:** Rust (core) with Local LLM Integration (Ollama)
- **Pattern:** Hierarchical 3-tier analysis system
- **Target Languages:** Java, TypeScript, Python
- **Framework Detection:** Spring Boot, React, NestJS, Django/Flask, FastAPI
- **AI Enhancement:** Local Llama-3.2-3B for semantic code understanding

## Current Capabilities

### Framework Detection Engine
- Multi-language AST parsing with tree-sitter
- Confidence-scored pattern matching
- Evidence-based framework validation
- Cross-framework dependency analysis

### Business Domain Intelligence
- Hierarchical code segmentation and analysis
- LLM-enhanced semantic understanding
- Context-aware business domain classification
- Intent inference from code patterns and architecture

### Document Generation Pipeline
- Structured YAML analysis reports
- Executive summaries with business context
- User story extraction from code patterns
- Technical documentation with implementation status
- CCPM-compatible import formats

## Development Priorities

### Phase 3: SOTA Hybrid Hierarchical Context-Aware Implementation
1. **Project-Level Context Analysis**: Fix current LLM "segment myopia" issue
2. **Multi-layered Context Integration**: Combine traditional + AST + LLM analysis
3. **Business Context Grounding**: Domain-specific prompting for accurate classification  
4. **Hierarchical Result Fusion**: Weighted confidence scoring across analysis tiers
5. **Performance Optimization**: Caching, batch processing, incremental analysis

### Current Implementation Status
- **Framework Detection**: Production-ready with 95%+ accuracy
- **AST Integration**: Partially implemented, needs context enhancement
- **LLM Integration**: Working but suffers from context loss issues
- **Business Domain Engine**: Needs hierarchical context-aware redesign

### Critical Issue Identified
Current LLM analysis misclassifies the analyzer tool itself as a "User Management Web Application" instead of recognizing it as a "Codebase Intelligence & Development Workflow Automation Platform". Root cause: segment-level analysis without project-level context understanding.

## Code Standards
- Security-first design with content sanitization
- Local-only processing for enterprise privacy
- Clear domain boundaries and comprehensive error handling
- Plugin-ready architecture for framework extensions
- Comprehensive testing with phase-specific validation
- Context-aware analysis to prevent classification errors

## Testing Strategy
- **Phase-specific test commands**: `test-basic`, `test-llm --enable-llm`
- **Sample project validation**: 6 diverse test codebases
- **Accuracy benchmarks**: Framework detection, domain classification
- **Performance monitoring**: Analysis speed and resource usage

## Enhanced Output Requirements

### Analysis Report Structure
The analyzer generates comprehensive business intelligence reports:

```yaml
metadata:
  analyzer_version: "2.0.0"
  analysis_date: "[ISO-8601 timestamp]"
  project_path: "[analyzed project path]"
  frameworks_detected: []
  confidence_scores: {}
  project_context: {}  # NEW: Project-level understanding

business_context:
  inferred_project_type: "[Analysis Tool, E-commerce Platform, etc.]"  # CORRECTED
  primary_business_domain: "[Codebase Intelligence, User Management, etc.]"
  user_personas: []
  business_capabilities: []

implementation_analysis:
  user_stories: []
  components: []
  api_endpoints: []
  database_entities: []
  
status_intelligence:
  completed_features: []
  in_progress_features: []
  todo_features: []
  technical_debt: []

hierarchical_analysis:  # NEW: Multi-tier results
  tier1_framework_detection: {}
  tier2_ast_segmentation: {}
  tier3_llm_semantic_analysis: {}
  fusion_confidence_scores: {}
```

## Integration Points
- **CCPM Workflow**: Automated requirement extraction for project management
- **Claude Code Spec**: Technical context for development continuation
- **Local LLM Stack**: Ollama + Llama-3.2-3B for privacy-preserving analysis
- **Docker Integration**: One-command setup with model management

## Performance Targets
- **Small Projects** (<100 files): Complete analysis in <30 seconds
- **Medium Projects** (100-500 files): Complete analysis in <2 minutes
- **Large Projects** (500+ files): Complete analysis in <5 minutes
- **Business Domain Accuracy**: 90%+ with context-aware LLM enhancement
- **Framework Detection Accuracy**: 95%+ with AST validation

This project represents a fundamental shift from pattern-matching code analysis to AI-powered business intelligence extraction, enabling automated reverse engineering of codebases into systematic development workflows.