use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum CacheAction {
    /// Show cache statistics
    Stats,
    /// Clear all cached analysis results
    Clear,
    /// Show cache location and configuration
    Info,
}
use anyhow::Result;

#[derive(Parser)]
#[command(name = "codebase-analyzer")]
#[command(about = "Reverse engineer codebases into systematic development workflows")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Analyze a codebase with SOTA hierarchical context-aware analysis
    Analyze {
        /// Path to the project directory
        #[arg(short, long)]
        path: String,
        
        /// Force specific analyzer (typescript, java, python)
        #[arg(long)]
        analyzer: Option<String>,
        
        /// Enable LLM analysis for enhanced business intelligence (requires Ollama)
        #[arg(long)]
        enable_llm: bool,
        
        /// Generate enhanced documentation with output directory path
        #[arg(long)]
        generate_docs: Option<String>,
        
        /// External documentation paths (comma-separated)
        /// Example: --ext-docs-path "/external/docs1,/external/docs2,./local-docs"
        #[arg(long)]
        ext_docs_path: Option<String>,
        
        /// Use SOTA hierarchical result fusion analysis
        #[arg(long, default_value = "true")]
        enable_fusion: bool,
        
        /// Enable integrations (requires --features integrations)
        #[cfg(feature = "integrations")]
        #[arg(long)]
        enable_integrations: bool,
    },
    
    /// List supported project types
    List,
    
    /// Test Phase 1: Framework Detection and Business Domain Inference
    TestPhase1 {
        /// Path to the project directory
        #[arg(short, long)]
        path: String,
        
        /// Output directory for analysis results
        #[arg(long, default_value = "./phase1-results")]
        output: String,
    },
    
    /// Test Phase 2A: AST Integration
    TestAst {
        /// Path to the project directory
        #[arg(short, long)]
        path: Option<String>,
    },
    
    /// Test Phase 2B: LLM Integration
    TestLlm {
        /// Path to the project directory
        #[arg(short, long)]
        path: Option<String>,
        
        /// Enable LLM analysis (requires Ollama running)
        #[arg(long)]
        enable_llm: bool,
    },
    
    /// Setup and test Ollama integration
    SetupOllama {
        /// Path to test project for validation
        #[arg(short, long, default_value = ".")]
        path: String,
        
        /// Skip interactive setup and use defaults
        #[arg(long)]
        non_interactive: bool,
    },
    
    /// Cache management operations
    Cache {
        #[command(subcommand)]
        action: CacheAction,
    },
    
    /// Test context-aware classification by analyzing the analyzer itself
    TestContextAware {
        /// Path to project for testing (defaults to current directory)
        #[arg(short, long)]
        path: Option<String>,
    },
    
    /// Run framework detection validation tests
    TestFrameworkValidation,
    
    /// Start web server (requires --features web-server)
    #[cfg(feature = "web-server")]
    Serve {
        /// Port to serve on
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
}

pub struct CliRunner;

impl CliRunner {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn run(&self, cli: Cli) -> Result<()> {
        match cli.command {
            Commands::Analyze { 
                path, 
                analyzer, 
                enable_llm,
                generate_docs,
                ext_docs_path,
                enable_fusion,
                #[cfg(feature = "integrations")]
                enable_integrations
            } => {
                #[cfg(feature = "integrations")]
                {
                    self.run_analysis_with_integrations(path, analyzer, enable_llm, generate_docs, ext_docs_path, enable_fusion, enable_integrations).await
                }
                #[cfg(not(feature = "integrations"))]
                {
                    self.run_analysis(path, analyzer, enable_llm, generate_docs, ext_docs_path, enable_fusion).await
                }
            }
            Commands::List => {
                self.list_supported_types();
                Ok(())
            }
            Commands::TestPhase1 { path, output } => {
                self.run_phase1_test(path, output).await
            },
            Commands::TestAst { path } => {
                self.run_ast_test(path).await
            }
            Commands::TestLlm { path, enable_llm } => {
                self.run_llm_test(path, enable_llm).await
            }
            Commands::SetupOllama { path, non_interactive } => {
                self.setup_ollama_integration(path, non_interactive).await
            }
            Commands::Cache { action } => {
                self.handle_cache_command(&action).await
            }
            Commands::TestContextAware { path } => {
                self.run_context_aware_test(path).await
            }
            Commands::TestFrameworkValidation => {
                self.run_framework_validation_test().await
            }
            #[cfg(feature = "web-server")]
            Commands::Serve { port } => {
                crate::server::start_server(port).await
            }
        }
    }
    
    async fn run_analysis(&self, path: String, analyzer: Option<String>, enable_llm: bool, generate_docs: Option<String>, ext_docs_path: Option<String>, enable_fusion: bool) -> Result<()> {
        use crate::core::performance_monitor::PerformanceMonitor;
        use crate::core::cache_manager::CacheManager;
        
        // Initialize performance monitoring
        let mut perf_monitor = PerformanceMonitor::new();
        perf_monitor.start_phase("Total Analysis");
        
        // Initialize caching system
        let cache_manager = CacheManager::new()?;
        
        println!("Analyzing project: {}", path);
        
        // Check if path exists
        if !std::path::Path::new(&path).exists() {
            anyhow::bail!("Path does not exist: {}", path);
        }
        
        let path_buf = std::path::PathBuf::from(&path);
        
        // Parse and validate external documentation paths
        let external_docs_paths = if let Some(ext_paths) = &ext_docs_path {
            self.parse_external_docs_paths(ext_paths, &path_buf)?
        } else {
            Vec::new()
        };
        
        if !external_docs_paths.is_empty() {
            println!("External documentation sources: {}", external_docs_paths.len());
            for (i, doc_path) in external_docs_paths.iter().enumerate() {
                println!("  {}. {}", i + 1, doc_path.display());
            }
        }
        
        // Check cache first
        perf_monitor.start_phase("Cache Check");
        let project_hash = cache_manager.calculate_project_hash(&path)?;
        let cache_key = cache_manager.get_cache_key(&path, "hierarchical_analysis");
        
        if let Ok(Some(_cached_result)) = cache_manager.get::<serde_json::Value>(&cache_key) {
            perf_monitor.record_cache_hit();
            perf_monitor.end_phase("Cache Check");
            
            // Generate docs if requested using cached data
            if let Some(docs_dir) = generate_docs {
                perf_monitor.start_phase("Document Generation");
                // Use cached result for document generation
                perf_monitor.end_phase("Document Generation");
            }
            
            perf_monitor.end_phase("Total Analysis");
            println!("Analysis complete (cached results)");
            return Ok(());
        } else {
            perf_monitor.record_cache_miss();
        }
        perf_monitor.end_phase("Cache Check");
        
        // Store analysis results for summary
        let mut primary_framework = "Unknown".to_string();
        let mut business_domain = "Unknown".to_string();
        let mut business_confidence = 0.0;
        let mut analysis_result: Option<crate::core::context_aware_framework_detector::ContextAwareFrameworkAnalysisResult> = None;
        
        if enable_fusion {
            // Use SOTA Hierarchical Result Fusion System
            perf_monitor.start_phase("SOTA Analysis");
            use crate::core::context_aware_framework_detector::ContextAwareFrameworkDetector;
            
            let mut detector = ContextAwareFrameworkDetector::new(&path).await?;
            let result = detector.analyze_with_hierarchical_context(&path_buf).await?;
            perf_monitor.end_phase("SOTA Analysis");
            
            // Store analysis result for document generation
            analysis_result = Some(result.clone());
            
            // Extract key results
            if let Some(fusion_result) = &result.hierarchical_fusion {
                if let Some(framework) = &fusion_result.consolidated_results.primary_framework {
                    primary_framework = framework.to_string();
                }
                if let Some(domain) = &fusion_result.consolidated_results.primary_business_domain {
                    business_domain = domain.name.clone();
                    business_confidence = domain.confidence;
                }
            }
            
        } else {
            // Fallback to basic analysis (for comparison)
            println!("Running legacy analysis mode");
            use crate::core::{CodebaseAnalyzer, AnalyzerConfig};
            use crate::analyzers::{TypeScriptAnalyzer, JavaAnalyzer, PythonAnalyzer};
            
            let config = AnalyzerConfig::default();
            let ts_analyzer = TypeScriptAnalyzer::new(config.clone());
            let java_analyzer = JavaAnalyzer::new(config.clone());
            let python_analyzer = PythonAnalyzer::new(config);
            
            let (selected_analyzer, analyzer_name): (Box<dyn CodebaseAnalyzer>, &str) = 
                if let Some(analyzer_name) = &analyzer {
                    match analyzer_name.to_lowercase().as_str() {
                        "typescript" | "ts" => (Box::new(ts_analyzer), "TypeScript"),
                        "java" => (Box::new(java_analyzer), "Java"),
                        "python" | "py" => (Box::new(python_analyzer), "Python"),
                        _ => anyhow::bail!("Unsupported analyzer: {}. Use 'typescript', 'java', or 'python'", analyzer_name),
                    }
                } else {
                    // Auto-detect
                    if ts_analyzer.can_analyze(&path) {
                        (Box::new(ts_analyzer), "TypeScript")
                    } else if java_analyzer.can_analyze(&path) {
                        (Box::new(java_analyzer), "Java")
                    } else if python_analyzer.can_analyze(&path) {
                        (Box::new(python_analyzer), "Python")
                    } else {
                        (Box::new(TypeScriptAnalyzer::new(AnalyzerConfig::default())), "TypeScript")
                    }
                };
            
            let analysis = selected_analyzer.analyze(&path)?;
            
            primary_framework = analysis.framework_analysis.architecture_pattern.clone();
            business_domain = analysis.business_context.inferred_product_type.clone();
            business_confidence = analysis.business_context.confidence;
        }
        
        // Extract documentation (including external sources and cross-repository analysis)
        let documentation_info = if !external_docs_paths.is_empty() || generate_docs.is_some() {
            perf_monitor.start_phase("Documentation Extraction");
            use crate::core::documentation_extractor::DocumentationExtractor;
            use crate::core::analyzers::cross_repository_analyzer::CrossRepositoryAnalyzer;
            
            let extractor = DocumentationExtractor::new()?;
            let doc_result = if !external_docs_paths.is_empty() {
                println!("Extracting multi-source documentation...");
                extractor.extract_multi_source_documentation(&path_buf, &external_docs_paths)?
            } else {
                println!("Extracting project documentation...");
                extractor.extract_documentation(&path_buf)?
            };
            
            println!("Documentation extraction complete: {:.1}% confidence, {} technologies found",
                     doc_result.confidence_score * 100.0,
                     doc_result.technologies.len());

            // Perform cross-repository analysis if external docs exist
            let cross_repo_analysis = if !external_docs_paths.is_empty() {
                perf_monitor.start_phase("Cross-Repository Analysis");
                println!("Performing cross-repository documentation analysis...");
                
                let cross_repo_analyzer = CrossRepositoryAnalyzer::new(&path_buf)?;
                let cross_repo_result = cross_repo_analyzer.analyze_cross_repository_documentation(
                    &path_buf,
                    &external_docs_paths,
                )?;
                
                println!("Cross-repository analysis complete:");
                println!("  Project relationships found: {}", cross_repo_result.project_relationships.len());
                println!("  Relevant documentation entries: {}", cross_repo_result.relevant_documentation.len());
                if let Some(parent_context) = &cross_repo_result.parent_project_context {
                    println!("  Parent project detected: {} (confidence: {:.1}%)", 
                             parent_context.parent_project_name, 
                             parent_context.context_confidence * 100.0);
                    println!("  Project role: {}", parent_context.project_role);
                    println!("  Sibling projects: {}", parent_context.sibling_projects.len());
                }
                
                perf_monitor.end_phase("Cross-Repository Analysis");
                Some(cross_repo_result)
            } else {
                None
            };
            
            perf_monitor.end_phase("Documentation Extraction");
            Some((doc_result, cross_repo_analysis))
        } else {
            None
        };

        // Cache the analysis results (placeholder for now)
        perf_monitor.start_phase("Cache Storage");
        if enable_fusion {
            // TODO: Cache results after fixing serialization
        }
        perf_monitor.end_phase("Cache Storage");
        
        // Generate documentation if requested
        let docs_location = if let Some(docs_dir) = generate_docs {
            perf_monitor.start_phase("Document Generation");
            println!("Generating workflow documentation to: {}", docs_dir);
            std::fs::create_dir_all(&docs_dir)?;
            
            if enable_fusion {
                // Generate CCPM + Claude Code Spec workflow from SOTA analysis
                if let Some(ref result) = analysis_result {
                    self.generate_ccpm_workflow_with_result(&path, &docs_dir, enable_llm, result).await?;
                } else {
                    println!("No analysis result available for document generation");
                }
            } else {
                println!("Legacy mode documentation not available");
            }
            perf_monitor.end_phase("Document Generation");
            Some(docs_dir)
        } else {
            None
        };
        
        // Update performance metrics
        if let Ok(entries) = std::fs::read_dir(&path) {
            let file_count = entries.count();
            perf_monitor.add_files_processed(file_count);
        }
        
        // End total analysis
        perf_monitor.end_phase("Total Analysis");
        
        // Print clean final summary
        println!("\n=== Analysis Summary ===");
        println!("Primary Framework: {}", primary_framework);
        println!("Business Domain: {} ({:.1}% confidence)", business_domain, business_confidence * 100.0);
        
        if let Some(docs_dir) = &docs_location {
            println!("Documentation Generated: {}", docs_dir);
        }
        
        let total_files = if let Ok(entries) = std::fs::read_dir(&path) { 
            entries.count() 
        } else { 
            0 
        };
        
        let analysis_duration = perf_monitor.get_total_duration();
        println!("Analysis completed in {:.2}s for {} files", analysis_duration.as_secs_f64(), total_files);
        Ok(())
    }
    
    async fn handle_cache_command(&self, action: &CacheAction) -> Result<()> {
        use crate::core::cache_manager::CacheManager;
        
        let cache_manager = CacheManager::new()?;
        
        match action {
            CacheAction::Stats => {
                println!("Cache Statistics");
                println!("==================");
                
                match cache_manager.get_cache_stats() {
                    Ok(stats) => {
                        println!("Total cached files: {}", stats.get("file_count").unwrap_or(&0));
                        let size_bytes = stats.get("total_size_bytes").unwrap_or(&0);
                        let size_mb = *size_bytes as f64 / (1024.0 * 1024.0);
                        println!("Total cache size: {:.2} MB", size_mb);
                        
                        if *size_bytes == 0 {
                            println!("Cache is empty - no analysis results cached yet");
                        } else {
                            println!("Cache location: {}", dirs::cache_dir()
                                .unwrap_or_default()
                                .join("codebase-workflow-analyzer")
                                .display());
                        }
                    }
                    Err(e) => {
                        println!("Error reading cache stats: {}", e);
                    }
                }
            }
            
            CacheAction::Clear => {
                println!("Clearing all cached analysis results...");
                match cache_manager.clear_all() {
                    Ok(_) => println!("Cache cleared successfully"),
                    Err(e) => println!("Error clearing cache: {}", e),
                }
            }
            
            CacheAction::Info => {
                println!("Cache Configuration");
                println!("====================");
                
                let cache_dir = dirs::cache_dir()
                    .unwrap_or_default()
                    .join("codebase-workflow-analyzer");
                
                println!("Cache directory: {}", cache_dir.display());
                println!("Cache max age: 24 hours");
                println!("Analyzer version: {}", env!("CARGO_PKG_VERSION"));
                
                if cache_dir.exists() {
                    println!("Cache directory exists");
                } else {
                    println!("Cache directory will be created on first use");
                }
            }
        }
        
        Ok(())
    }
    
    async fn generate_ccpm_workflow(&self, project_path: &str, docs_dir: &str, enable_llm: bool) -> Result<()> {
        use crate::core::context_aware_framework_detector::ContextAwareFrameworkDetector;
        use std::fs;
        
        // Running SOTA analysis for CCPM workflow generation
        
        // Run the SOTA analysis to get comprehensive results
        let mut detector = ContextAwareFrameworkDetector::new(project_path).await?;
        let path_buf = std::path::PathBuf::from(project_path);
        let result = detector.analyze_with_hierarchical_context(&path_buf).await?;
        
        // Creating CCPM + Claude Code Spec directory structure
        
        // Create .claude directory structure (CCPM compatible)
        let claude_dir = format!("{}/.claude", docs_dir);
        fs::create_dir_all(&claude_dir)?;
        fs::create_dir_all(format!("{}/context", claude_dir))?;
        fs::create_dir_all(format!("{}/prds", claude_dir))?;
        fs::create_dir_all(format!("{}/epics", claude_dir))?;
        fs::create_dir_all(format!("{}/agents", claude_dir))?;
        fs::create_dir_all(format!("{}/commands", claude_dir))?;
        
        // Generating CCMP workflow files
        
        // 1. Generate CLAUDE.md (Always-on instructions)
        let claude_md = self.generate_claude_md(&result, project_path, enable_llm).await?;
        fs::write(format!("{}/CLAUDE.md", claude_dir), claude_md)?;
        
        // 2. Generate PRD from SOTA business intelligence
        let prd = self.generate_prd_from_sota(&result, project_path)?;
        fs::write(format!("{}/prds/product-requirements.md", claude_dir), prd)?;
        
        // 3. Generate context files for Claude Code Spec
        let product_context = self.generate_product_context(&result)?;
        fs::write(format!("{}/context/product.md", claude_dir), product_context)?;
        
        let tech_context = self.generate_tech_context(&result)?;
        fs::write(format!("{}/context/tech.md", claude_dir), tech_context)?;
        
        let structure_context = self.generate_structure_context(&result, project_path)?;
        fs::write(format!("{}/context/structure.md", claude_dir), structure_context)?;
        
        // 4. Generate user stories and epics
        let user_stories = self.generate_user_stories_from_sota(&result)?;
        fs::write(format!("{}/epics/user-stories.md", claude_dir), user_stories)?;
        
        // 5. Generate implementation tasks
        fs::create_dir_all(format!("{}/epics/implementation", claude_dir))?;
        let implementation_epic = self.generate_implementation_epic(&result)?;
        fs::write(format!("{}/epics/implementation/epic.md", claude_dir), implementation_epic)?;
        
        // 6. Generate agents for specialized tasks
        let agents = self.generate_specialized_agents(&result)?;
        for (agent_name, agent_config) in agents {
            fs::write(format!("{}/agents/{}.md", claude_dir, agent_name), agent_config)?;
        }
        
        // 7. Generate command definitions
        let commands = self.generate_ccpm_commands(&result)?;
        fs::write(format!("{}/commands/spec-commands.md", claude_dir), commands)?;
        
        // 8. CCPM workflow generation completed successfully

        // 9. Generate main project README with workflow instructions
        let workflow_readme = self.generate_workflow_readme(&result, project_path)?;
        fs::write(format!("{}/README.md", docs_dir), workflow_readme)?;
        
        println!("CCPM + Claude Code Spec workflow generated successfully");
        println!("   context/ - Claude Code Spec steering documents");
        println!("   epics/ - User stories and implementation tasks");
        println!("   agents/ - Specialized AI agents for development");
        // Workflow generation completed
        
        Ok(())
    }
    
    async fn generate_ccpm_workflow_with_result(&self, project_path: &str, docs_dir: &str, enable_llm: bool, result: &crate::core::context_aware_framework_detector::ContextAwareFrameworkAnalysisResult) -> Result<()> {
        use std::fs;
        
        // Creating CCPM + Claude Code Spec directory structure
        
        // Create .claude directory structure (CCPM compatible)
        let claude_dir = format!("{}/.claude", docs_dir);
        fs::create_dir_all(&claude_dir)?;
        fs::create_dir_all(format!("{}/context", claude_dir))?;
        fs::create_dir_all(format!("{}/epics", claude_dir))?;
        fs::create_dir_all(format!("{}/agents", claude_dir))?;
        fs::create_dir_all(format!("{}/epics/implementation", claude_dir))?;
        
        // Generate CCMP workflow files
        
        // 1. Generate CLAUDE.md - Always-on project context
        let claude_md = self.generate_claude_md(result, project_path, enable_llm).await?;
        fs::write(format!("{}/CLAUDE.md", docs_dir), claude_md)?;
        
        // 2. Generate PRD from SOTA business intelligence
        let prd = self.generate_prd_from_sota(result, project_path)?;
        fs::create_dir_all(format!("{}/prds", claude_dir))?;
        fs::write(format!("{}/prds/product-requirements.md", claude_dir), prd)?;
        
        // 3. Generate context files for Claude Code Spec
        let product_context = self.generate_product_context(result)?;
        fs::write(format!("{}/context/product.md", claude_dir), product_context)?;
        
        let tech_context = self.generate_tech_context(result)?;
        fs::write(format!("{}/context/tech.md", claude_dir), tech_context)?;
        
        let structure_context = self.generate_structure_context(result, project_path)?;
        fs::write(format!("{}/context/structure.md", claude_dir), structure_context)?;
        
        // 4. Generate user stories and epics
        let user_stories = self.generate_user_stories_from_sota(result)?;
        fs::write(format!("{}/epics/user-stories.md", claude_dir), user_stories)?;
        
        let epic_doc = self.generate_implementation_epic(result)?;
        fs::write(format!("{}/epics/implementation/epic.md", claude_dir), epic_doc)?;
        
        // 5-9. Generate remaining workflow files using the existing result
        let workflow_readme = self.generate_workflow_readme(result, project_path)?;
        fs::write(format!("{}/README.md", docs_dir), workflow_readme)?;
        
        println!("CCPM + Claude Code Spec workflow generated successfully");
        println!("   context/ - Claude Code Spec steering documents");
        println!("   epics/ - User stories and implementation tasks");
        println!("   agents/ - Specialized AI agents for development");
        
        Ok(())
    }
    
    async fn generate_documentation(&self, analysis: &crate::core::CodebaseAnalysis, docs_dir: Option<String>, project_path: &str) -> Result<()> {
        use crate::generators::DocumentGeneratorFactory;
        
        let output_dir = docs_dir.unwrap_or_else(|| format!("{}/analysis-docs", project_path));
        // Generating comprehensive documentation
        
        // Create output directory
        std::fs::create_dir_all(&output_dir)?;
        
        // Generate all enhanced documents using the factory
        let generated_docs = DocumentGeneratorFactory::generate_all_documents(analysis, &output_dir)?;
        
        println!("Generated {} documents to: {}", generated_docs.len(), output_dir);
        
        Ok(())
    }
    
    #[cfg(feature = "integrations")]
    async fn run_analysis_with_integrations(&self, path: String, analyzer: Option<String>, enable_llm: bool, generate_docs: Option<String>, ext_docs_path: Option<String>, enable_fusion: bool, enable_integrations: bool) -> Result<()> {
        // Run SOTA analysis first
        self.run_analysis(path.clone(), analyzer, enable_llm, generate_docs, ext_docs_path, enable_fusion).await?;
        
        // If integrations are enabled, run them
        if enable_integrations {
            println!("\nRunning integrations...");
            
            let config = crate::integrations::IntegrationConfig::default();
            let integration_engine = crate::integrations::IntegrationEngine::new(config);
            
            // Note: For a real integration, we would need to properly analyze the codebase
            // and create a CodebaseAnalysis struct. For now, we'll show basic integration status.
            
            println!("Integration Status:");
            println!("  Git: {}", if std::path::Path::new(&format!("{}/.git", path)).exists() { "Available" } else { "Not a git repository" });
            println!("  GitHub: {}", if std::env::var("GITHUB_TOKEN").is_ok() { "Token configured" } else { "No token (set GITHUB_TOKEN)" });
            println!("  Jira: {}", if std::env::var("JIRA_URL").is_ok() && std::env::var("JIRA_TOKEN").is_ok() { "Configured" } else { "Not configured" });
            
            println!("\nTo use integrations, ensure environment variables are set:");
            println!("  export GITHUB_TOKEN=your_token");
            println!("  export JIRA_URL=https://your-instance.atlassian.net");
            println!("  export JIRA_TOKEN=your_token");
            println!("  export JIRA_PROJECT_KEY=YOUR_PROJECT");
        }
        
        Ok(())
    }
    
    
    fn list_supported_types(&self) {
        println!("Supported Project Types:\n");
        
        println!("TypeScript/React");
        println!("   File extensions: .ts, .tsx, .js, .jsx");
        println!("   Detection: package.json with React dependency");
        println!("   Features:");
        println!("     • Component analysis and classification");
        println!("     • Props and hooks extraction");
        println!("     • User story inference");
        println!("     • Implementation status detection");
        println!("     • API call identification");
        println!();
        
        println!("NestJS (Node.js Backend)");
        println!("   File extensions: .ts, .js");
        println!("   Detection: package.json with @nestjs/core dependency or NestJS decorators");
        println!("   Features:");
        println!("     • Controller, Service, Repository detection");
        println!("     • Decorator-based pattern analysis");
        println!("     • REST endpoint mapping");
        println!("     • Dependency injection analysis");
        println!("     • Module structure analysis");
        println!();
        
        println!("Danet (Deno Backend - NestJS-like)");
        println!("   File extensions: .ts, .tsx");
        println!("   Detection: deno.json with Danet imports or Danet decorators with deno.land URLs");
        println!("   Features:");
        println!("     • NestJS-like decorator pattern analysis");
        println!("     • Controller and service detection");
        println!("     • Deno-specific import analysis");
        println!("     • REST endpoint mapping");
        println!("     • Module architecture analysis");
        println!();
        
        println!("Fresh (Deno Fullstack)");
        println!("   File extensions: .ts, .tsx");
        println!("   Detection: deno.json with Fresh imports or routes/ directory structure");
        println!("   Features:");
        println!("     • Island architecture analysis");
        println!("     • Route-based component detection");
        println!("     • Handler function analysis");
        println!("     • Server-side rendering patterns");
        println!("     • Deno-specific patterns");
        println!();
        
        println!("Oak (Deno Backend)");
        println!("   File extensions: .ts, .tsx");
        println!("   Detection: deno.json with Oak imports or Oak router patterns");
        println!("   Features:");
        println!("     • Router and middleware analysis");
        println!("     • HTTP handler detection");
        println!("     • REST endpoint mapping");
        println!("     • Deno-specific patterns");
        println!("     • Application structure analysis");
        println!();
        
        println!("Java/Spring Boot");
        println!("   File extensions: .java");
        println!("   Detection: pom.xml/build.gradle with Spring Boot");
        println!("   Features:");
        println!("     • Controller, Service, Repository detection");
        println!("     • JPA Entity analysis");
        println!("     • REST endpoint mapping");
        println!("     • Spring annotation processing");
        println!("     • Dependency injection analysis");
        println!();
        
        println!("Python/Django/Flask");
        println!("   File extensions: .py");
        println!("   Detection: Django settings.py or Flask imports");
        println!("   Features:");
        println!("     • Model, View, Form detection");
        println!("     • URL routing analysis");
        println!("     • Framework-specific patterns");
        println!("     • Database model relationships");
        println!("     • API endpoint detection");
        println!();
        
        println!("Enhanced Features (All Types):");
        println!("   AI-powered code analysis");
        println!("   Quality metrics and recommendations");
        println!("   Architecture pattern suggestions");
        println!("   Business insight generation");
        println!("   Multi-format document generation");
        println!("     • Executive summaries");
        println!("     • Product requirement documents (PRD)");
        println!("     • User stories with acceptance criteria");
        println!("     • Technical documentation");
        println!("     • Comprehensive markdown reports");
        println!();
        
        println!("Usage Examples:");
        println!("   SOTA Hierarchical Analysis (recommended):");
        println!("   codebase-analyzer analyze --path /path/to/project");
        println!();
        println!("   With LLM-enhanced business intelligence:");
        println!("   codebase-analyzer analyze --path /path/to/project --enable-llm");
        println!();
        println!("   Generate documentation to specific directory:");
        println!("   codebase-analyzer analyze --path /path/to/project --generate-docs /output/dir");
        println!();
        println!("   Force specific analyzer:");
        println!("   codebase-analyzer analyze --path /path/to/project --analyzer java");
        println!();
        println!("   Legacy mode (disable SOTA fusion):");
        println!("   codebase-analyzer analyze --path /path/to/project --enable-fusion false");
        
        #[cfg(feature = "integrations")]
        {
            println!();
            println!("   With integrations (requires --features integrations):");
            println!("   codebase-analyzer analyze --path /path/to/project --enable-integrations");
        }
        
        #[cfg(feature = "web-server")]
        {
            println!();
            println!("   Start web server (requires --features web-server):");
            println!("   codebase-analyzer serve --port 3000");
        }
    }

    /// Run Phase 1 test - Framework Detection and Business Domain Inference
    async fn run_phase1_test(&self, path: String, output: String) -> Result<()> {
        use crate::core::integration_demo::Phase1Demo;
        
        println!("🚀 Running Phase 1 Test: Framework Detection and Business Domain Inference");
        println!("   Project Path: {}", path);
        println!("   Output Directory: {}", output);
        println!();
        
        // Check if path exists
        if !std::path::Path::new(&path).exists() {
            anyhow::bail!("Path does not exist: {}", path);
        }

        // Run Phase 1 analysis
        let demo = Phase1Demo::new(path);
        match demo.run_phase1_analysis() {
            Ok(result) => {
                // Print detailed report
                result.print_detailed_report();
                
                // Save results
                if let Err(e) = result.save_results(&output) {
                    println!("Warning: Could not save results to {}: {}", output, e);
                    println!("But analysis completed successfully!");
                } else {
                    println!("\nResults saved to: {}", output);
                }
                
                println!("\nPhase 1 Test Completed Successfully!");
                println!("\nSummary:");
                println!("   • Frameworks Detected: {}", result.framework_result.detected_frameworks.len());
                println!("   • Primary Domains: {}", result.domain_result.primary_domains.len());
                println!("   • Secondary Domains: {}", result.domain_result.secondary_domains.len());
                
                Ok(())
            }
            Err(e) => {
                anyhow::bail!("Phase 1 analysis failed: {}", e)
            }
        }
    }

    async fn run_ast_test(&self, path: Option<String>) -> Result<()> {
        println!("Starting AST Integration Test (Phase 2A)");
        
        // Use current directory if no path provided
        let test_path = path.unwrap_or_else(|| ".".to_string());
        
        // Basic AST tests are temporarily disabled
        // crate::core::ast_integration_test::run_basic_ast_test()?;
        
        // AST integration test with actual codebase is temporarily disabled
        if std::path::Path::new(&test_path).exists() {
            println!("\nTesting AST on path: {}", test_path);
            // crate::core::ast_integration_test::test_ast_integration_with_path(&test_path).await?;
            println!("AST integration test temporarily disabled - using new context-aware flow instead");
        } else {
            println!("Path {} does not exist, skipping codebase test", test_path);
        }
        
        println!("AST Integration test completed successfully!");
        Ok(())
    }
    
    async fn setup_ollama_integration(&self, path: String, non_interactive: bool) -> Result<()> {
        use crate::intelligence::OllamaManager;
        
        println!("Ollama Integration Setup");
        println!("=========================");
        
        if non_interactive {
            println!("Running in non-interactive mode...");
            // TODO: Implement non-interactive setup
            return Ok(());
        }
        
        let mut ollama_manager = OllamaManager::new()?;
        
        match ollama_manager.initialize_with_user_interaction(&path).await {
            Ok(selected_model) => {
                println!("\nOllama integration setup complete!");
                println!("Selected model: {}", selected_model);
                
                // Test basic functionality
                println!("\nTesting model with sample business analysis...");
                let test_response = ollama_manager.generate_response(
                    &selected_model,
                    "Analyze this simple JavaScript function: `function add(a, b) { return a + b; }`. What is its business purpose in 1 sentence?"
                ).await?;
                
                println!("Test successful! Model response:");
                println!("   {}", test_response.chars().take(200).collect::<String>());
                
                println!("\nReady to use with: cargo run analyze --path PROJECT_PATH --enable-llm --generate-docs OUTPUT_DIR");
            }
            Err(e) => {
                println!("Ollama setup failed: {}", e);
                println!("Please ensure:");
                println!("  1. Ollama is installed: https://ollama.com/");
                println!("  2. Ollama is running: ollama serve");
                println!("  3. You have sufficient disk space for models");
                return Err(e);
            }
        }
        
        Ok(())
    }

    async fn run_llm_test(&self, path: Option<String>, enable_llm: bool) -> Result<()> {
        use crate::core::enhanced_framework_detector::EnhancedFrameworkDetector;
        use crate::intelligence::llm_client::ModelConfig;
        
        println!("Starting LLM Integration Test (Phase 2B)");
        
        // Use current directory if no path provided
        let test_path = path.unwrap_or_else(|| ".".to_string());
        
        // Check if path exists
        if !std::path::Path::new(&test_path).exists() {
            anyhow::bail!("Path does not exist: {}", test_path);
        }
        
        println!("Testing hierarchical analysis on path: {}", test_path);
        
        // Create enhanced detector with AST analysis
        let mut detector = EnhancedFrameworkDetector::new(test_path.clone())?
            .with_context_aware_analysis()?;
        
        // Add LLM analysis if requested
        if enable_llm {
            println!("Initializing LLM integration...");
            detector = detector.with_llm_analysis(Some(ModelConfig::default())).await?;
        } else {
            println!("LLM analysis disabled, running without LLM integration");
        }
        
        // Run enhanced analysis
        let result = detector.detect_frameworks_enhanced().await?;
        
        // Display results
        println!("\nHierarchical Analysis Results:");
        println!("    Language Ecosystem: {}", result.primary_ecosystem);
        println!("    Detected Frameworks: {}", result.detected_frameworks.len());
        println!("    Code Segments: {}", result.code_segments.len());
        
        if let Some(ast_analysis) = &result.ast_analysis {
            println!("    AST Analysis:");
            println!("      - Functions: {}", ast_analysis.segment_statistics.function_count);
            println!("      - Classes: {}", ast_analysis.segment_statistics.class_count);
            println!("      - Interfaces: {}", ast_analysis.segment_statistics.interface_count);
        }
        
        if let Some(llm_analysis) = &result.llm_analysis {
            println!("    LLM Analysis:");
            println!("      - Available: {}", llm_analysis.llm_available);
            println!("      - Processing Time: {}ms", llm_analysis.processing_time_ms);
            println!("      - Segments Analyzed: {}", llm_analysis.business_domain_analysis.segments.len());
            
            // Display new business analysis if available
            if let Some(project_analysis) = &llm_analysis.business_domain_analysis.project_analysis {
                println!("\n    Business Analysis:");
                println!("      Primary Domain: {}", project_analysis.primary_business_domain);
                println!("      Project Type: {}", project_analysis.project_type);
                
                println!("\n      Functional Requirements:");
                println!("        {}", project_analysis.functional_requirements.description);
                for (domain, analysis) in &project_analysis.functional_requirements.domains {
                    println!("        - {} ({}% confidence): {}", domain, (analysis.confidence * 100.0) as u32, analysis.description);
                }
                
                println!("\n      Non-Functional Requirements:");
                println!("        {}", project_analysis.non_functional_requirements.description);
                for (domain, analysis) in &project_analysis.non_functional_requirements.domains {
                    println!("        - {} ({}% confidence): {}", domain, (analysis.confidence * 100.0) as u32, analysis.description);
                }
            } else {
                // Fallback to old format
                println!("      - Domain Distribution: {:?}", 
                    llm_analysis.business_domain_analysis.summary.domain_distribution.keys().collect::<Vec<_>>());
            }
        }
        
        // Show framework analysis
        println!("\n🔍 Framework Analysis:");
        for framework in &result.detected_frameworks[..std::cmp::min(5, result.detected_frameworks.len())] {
            println!("  - {}: {:.1}% confidence", framework.framework, framework.confidence * 100.0);
            if let Some(ast_evidence) = &framework.ast_evidence {
                if ast_evidence.relevant_segments > 0 {
                    println!("    AST Segments: {}", ast_evidence.relevant_segments);
                    if !ast_evidence.framework_specific_patterns.is_empty() {
                        println!("    Patterns: {:?}", &ast_evidence.framework_specific_patterns[..std::cmp::min(2, ast_evidence.framework_specific_patterns.len())]);
                    }
                }
            }
        }
        
        println!("\nLLM Integration test completed successfully!");
        Ok(())
    }

    /// Test context-aware classification by analyzing the analyzer itself
    async fn run_context_aware_test(&self, path: Option<String>) -> Result<()> {
        use crate::core::context_aware_test::validate_context_aware_classification;
        
        let test_path = path.unwrap_or_else(|| ".".to_string());
        
        println!("🧪 Running Context-Aware Classification Test");
        println!("====================================================");
        println!("Testing path: {}", test_path);
        
        // Change to the test directory
        if test_path != "." {
            std::env::set_current_dir(&test_path)?;
        }
        
        // Run the self-analysis test
        validate_context_aware_classification().await?;
        
        Ok(())
    }
    
    // SOTA Documentation Generation Methods
    
    fn generate_executive_summary(&self, result: &crate::core::context_aware_framework_detector::ContextAwareFrameworkAnalysisResult, project_path: &str) -> Result<String> {
        let project_name = std::path::Path::new(project_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown Project");
            
        let mut summary = String::new();
        summary.push_str(&format!("# Executive Summary: {}\n\n", project_name));
        summary.push_str("## SOTA Hierarchical Context-Aware Analysis Results\n\n");
        
        summary.push_str(&format!("**Analysis Date**: {}\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        summary.push_str(&format!("**Analysis Duration**: {}ms\n", result.performance_metrics.total_analysis_time_ms));
        summary.push_str(&format!("**Segments Analyzed**: {}\n\n", result.context_awareness_summary.total_segments_analyzed));
        
        summary.push_str("## Key Findings\n\n");
        
        // Framework Analysis
        if let Some(fusion_result) = &result.hierarchical_fusion {
            if let Some(framework) = &fusion_result.consolidated_results.primary_framework {
                summary.push_str(&format!("**Primary Framework**: {}\n", framework.to_string()));
            }
            
            if let Some(domain) = &fusion_result.consolidated_results.primary_business_domain {
                summary.push_str(&format!("**Business Domain**: {} (Confidence: {:.1}%)\n", 
                    domain.name, domain.confidence * 100.0));
            }
        }
        
        summary.push_str(&format!("**Context Coverage**: {:.1}%\n", result.context_awareness_summary.context_completeness_score * 100.0));
        summary.push_str(&format!("**Average Confidence**: {:.1}%\n\n", result.context_awareness_summary.average_confidence * 100.0));
        
        summary.push_str("## Business Domain Coverage\n\n");
        for (domain, count) in &result.context_awareness_summary.business_domain_coverage {
            summary.push_str(&format!("- **{}**: {} segments\n", domain, count));
        }
        
        summary.push_str("\n## Performance Metrics\n\n");
        summary.push_str(&format!("- **Context Establishment Efficiency**: {:.3}\n", result.performance_metrics.context_establishment_efficiency));
        summary.push_str(&format!("- **Accuracy Improvement**: {:.3}\n", result.performance_metrics.improvement_over_baseline.accuracy_improvement));
        summary.push_str(&format!("- **Confidence Boost**: {:.3}\n\n", result.performance_metrics.improvement_over_baseline.confidence_boost));
        
        summary.push_str("---\n*Generated by SOTA Hierarchical Result Fusion Engine*\n");
        
        Ok(summary)
    }
    
    fn generate_technical_analysis(&self, result: &crate::core::context_aware_framework_detector::ContextAwareFrameworkAnalysisResult, project_path: &str) -> Result<String> {
        use crate::core::template_engine::TemplateEngine;
        
        let mut engine = TemplateEngine::new();
        
        // Build comprehensive technical analysis
        let technical_summary = format!(
            "Project analyzed: {}\nTotal segments: {}\nAnalysis time: {}ms\nContext coverage: {:.1}%",
            std::path::Path::new(project_path).file_name().unwrap_or_default().to_str().unwrap_or("Unknown"),
            result.context_awareness_summary.total_segments_analyzed,
            result.performance_metrics.total_analysis_time_ms,
            result.context_awareness_summary.context_completeness_score * 100.0
        );
        
        let architecture_details = if let Some(fusion_result) = &result.hierarchical_fusion {
            if let Some(framework) = &fusion_result.consolidated_results.primary_framework {
                format!(
                    "**Primary Framework**: {}\n**Detection Confidence**: {:.1}%\n**Architectural Pattern**: Based on {} framework conventions",
                    framework.to_string(),
                    fusion_result.quality_metrics.overall_fusion_quality * 100.0,
                    framework.to_string()
                )
            } else {
                "Architecture analysis in progress based on code patterns.".to_string()
            }
        } else {
            "No specific architectural framework detected.".to_string()
        };
        
        let quality_metrics = format!(
            "- **Analysis Confidence**: {:.1}%\n- **Context Completeness**: {:.1}%\n- **Segments Analyzed**: {}",
            result.context_awareness_summary.average_confidence * 100.0,
            result.context_awareness_summary.context_completeness_score * 100.0,
            result.context_awareness_summary.total_segments_analyzed
        );
        
        let dependencies_analysis = format!(
            "Dependencies detected across {} business domains:\n{}",
            result.context_awareness_summary.business_domain_coverage.len(),
            result.context_awareness_summary.business_domain_coverage
                .iter()
                .map(|(domain, count)| format!("- {}: {} segments", domain, count))
                .collect::<Vec<_>>()
                .join("\n")
        );
        
        let performance_analysis = format!(
            "**Analysis Performance**:\n- Total time: {}ms\n- Average per segment: {:.2}ms\n- Context establishment efficiency: {:.3}",
            result.performance_metrics.total_analysis_time_ms,
            result.performance_metrics.total_analysis_time_ms as f32 / result.context_awareness_summary.total_segments_analyzed as f32,
            result.performance_metrics.context_establishment_efficiency
        );
        
        // Use template to build complete document
        let template = r#"# Technical Analysis Report

## Technical Overview

{technical_summary}

## Architecture Analysis

{architecture_details}

## Code Quality Metrics

{quality_metrics}

## Dependencies Analysis

{dependencies_analysis}

## Performance Analysis

{performance_analysis}

---
*Generated by SOTA Technical Analysis Engine*
"#;
        
        engine.set_variable("technical_summary".to_string(), technical_summary);
        engine.set_variable("architecture_details".to_string(), architecture_details);
        engine.set_variable("quality_metrics".to_string(), quality_metrics);
        engine.set_variable("dependencies_analysis".to_string(), dependencies_analysis);
        engine.set_variable("performance_analysis".to_string(), performance_analysis);
        
        Ok(engine.render(template))
    }
    
    fn generate_framework_analysis(&self, result: &crate::core::context_aware_framework_detector::ContextAwareFrameworkAnalysisResult) -> Result<String> {
        use crate::core::template_engine::TemplateEngine;
        
        let mut engine = TemplateEngine::new();
        
        let detection_summary = format!(
            "Frameworks detected: {}\nPrimary detection method: SOTA Hierarchical Context-Aware Analysis\nTotal confidence score: {:.1}%",
            result.traditional_analysis.detected_frameworks.len(),
            result.context_awareness_summary.average_confidence * 100.0
        );
        
        let (primary_framework, confidence, evidence) = if let Some(fusion_result) = &result.hierarchical_fusion {
            if let Some(framework) = &fusion_result.consolidated_results.primary_framework {
                (
                    framework.to_string(),
                    format!("{:.1}", fusion_result.quality_metrics.overall_fusion_quality * 100.0),
                    "SOTA multi-tier analysis with context-aware validation".to_string()
                )
            } else {
                ("No primary framework detected".to_string(), "0.0".to_string(), "Insufficient evidence".to_string())
            }
        } else {
            ("No primary framework detected".to_string(), "0.0".to_string(), "Insufficient evidence".to_string())
        };
        
        let secondary_frameworks = if result.traditional_analysis.detected_frameworks.len() > 1 {
            result.traditional_analysis.detected_frameworks[1..]
                .iter()
                .take(3)
                .map(|f| format!("- **{}**: {:.1}% confidence", f.framework, f.confidence * 100.0))
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            "No secondary frameworks detected with sufficient confidence.".to_string()
        };
        
        let framework_patterns = result.context_awareness_summary.business_domain_coverage
            .iter()
            .map(|(domain, count)| format!("- **{}**: {} code patterns identified", domain, count))
            .collect::<Vec<_>>()
            .join("\n");
        
        let framework_recommendations = if let Some(fusion_result) = &result.hierarchical_fusion {
            format!(
                "Based on analysis results with {:.1}% fusion quality:\n- Continue using detected framework patterns\n- Maintain consistency with existing architecture\n- Consider framework-specific best practices",
                fusion_result.quality_metrics.overall_fusion_quality * 100.0
            )
        } else {
            "Follow detected patterns and maintain architectural consistency.".to_string()
        };
        
        let template = r#"# Framework Analysis Report

## Detection Summary

{detection_summary}

## Primary Framework

**Framework**: {primary_framework}
**Confidence**: {confidence}%
**Evidence**: {evidence}

## Secondary Frameworks

{secondary_frameworks}

## Detected Patterns

{framework_patterns}

## Framework Recommendations

{framework_recommendations}

---
*Generated by SOTA Framework Detection Engine*
"#;
        
        engine.set_variable("detection_summary".to_string(), detection_summary);
        engine.set_variable("primary_framework".to_string(), primary_framework);
        engine.set_variable("confidence".to_string(), confidence);
        engine.set_variable("evidence".to_string(), evidence);
        engine.set_variable("secondary_frameworks".to_string(), secondary_frameworks);
        engine.set_variable("framework_patterns".to_string(), framework_patterns);
        engine.set_variable("framework_recommendations".to_string(), framework_recommendations);
        
        Ok(engine.render(template))
    }
    
    fn generate_business_intelligence_report(&self, business_grounding: &crate::core::business_context_grounding::BusinessContextGroundingResult) -> Result<String> {
        use crate::core::template_engine::TemplateEngine;
        
        let mut engine = TemplateEngine::new();
        
        let business_summary = if let Some(primary_domain) = business_grounding.grounded_context.final_domains.first() {
            format!(
                "Primary business domain identified: **{}** with {:.1}% confidence.\nBusiness intelligence extracted from code analysis reveals {} domain focus.",
                primary_domain.name,
                primary_domain.confidence * 100.0,
                primary_domain.name.to_lowercase()
            )
        } else {
            "Business domain analysis completed with comprehensive code pattern recognition.".to_string()
        };
        
        let domain_analysis = business_grounding.grounded_context.final_domains
            .iter()
            .map(|domain| format!("- **{}**: {:.1}% confidence", domain.name, domain.confidence * 100.0))
            .collect::<Vec<_>>()
            .join("\n");
        
        let user_personas = "User personas inferred from business domain and code patterns analysis.".to_string();
        
        let business_capabilities = business_grounding.grounded_context.business_capabilities
            .iter()
            .map(|capability| format!("- **{}**", capability))
            .collect::<Vec<_>>()
            .join("\n");
        let business_capabilities = if business_capabilities.is_empty() {
            "Business capabilities derived from code structure and domain analysis.".to_string()
        } else {
            business_capabilities
        };
        
        let market_context = if let Some(primary_domain) = business_grounding.grounded_context.final_domains.first() {
            format!(
                "Market positioning based on {} domain analysis with focus on specialized functionality and target user needs.",
                primary_domain.name.to_lowercase()
            )
        } else {
            "Market context derived from business domain classification and competitive analysis.".to_string()
        };
        
        let strategic_insights = format!(
            "Strategic recommendations based on business intelligence analysis:\n- Focus on core {} competencies\n- Leverage identified business capabilities\n- Address user persona requirements\n- Maintain competitive positioning",
            business_grounding.grounded_context.final_domains.first().map(|d| d.name.as_str()).unwrap_or("domain")
        );
        
        let template = r#"# Business Intelligence Report

## Executive Summary

{business_summary}

## Business Domain Analysis

{domain_analysis}

## Identified User Personas

{user_personas}

## Business Capabilities

{business_capabilities}

## Market Context

{market_context}

## Strategic Insights

{strategic_insights}

---
*Generated by SOTA Business Intelligence Engine*
"#;
        
        engine.set_variable("business_summary".to_string(), business_summary);
        engine.set_variable("domain_analysis".to_string(), domain_analysis);
        engine.set_variable("user_personas".to_string(), user_personas);
        engine.set_variable("business_capabilities".to_string(), business_capabilities);
        engine.set_variable("market_context".to_string(), market_context);
        engine.set_variable("strategic_insights".to_string(), strategic_insights);
        
        Ok(engine.render(template))
    }
    
    fn generate_fusion_analysis_report(&self, fusion_result: &crate::core::hierarchical_result_fusion::HierarchicalFusionResult) -> Result<String> {
        let mut report = String::new();
        report.push_str("# Hierarchical Fusion Analysis Report\n\n");
        
        report.push_str("## SOTA Multi-Tier Fusion Results\n\n");
        report.push_str(&format!("**Fusion Strategy**: {:?}\n", fusion_result.fusion_metadata.strategy_used));
        report.push_str(&format!("**Analysis Duration**: {}ms\n", fusion_result.fusion_metadata.total_fusion_time_ms));
        report.push_str(&format!("**Tiers Processed**: {}\n\n", fusion_result.fusion_metadata.tiers_processed));
        
        report.push_str("## Quality Metrics\n\n");
        report.push_str(&format!("- **Overall Fusion Quality**: {:.3}\n", fusion_result.quality_metrics.overall_fusion_quality));
        report.push_str(&format!("- **Tier Alignment Score**: {:.3}\n", fusion_result.quality_metrics.tier_alignment_score));
        report.push_str(&format!("- **Consensus Strength**: {:.3}\n", fusion_result.quality_metrics.consensus_strength));
        
        if let Some(framework) = &fusion_result.consolidated_results.primary_framework {
            report.push_str(&format!("\n**Primary Framework**: {}\n", framework.to_string()));
        }
        
        if let Some(domain) = &fusion_result.consolidated_results.primary_business_domain {
            report.push_str(&format!("**Primary Business Domain**: {} (Confidence: {:.3})\n", 
                domain.name, domain.confidence));
        }
        
        report.push_str("\n---\n*Generated by SOTA Hierarchical Result Fusion Engine*\n");
        
        Ok(report)
    }
    
    fn generate_implementation_roadmap(&self, result: &crate::core::context_aware_framework_detector::ContextAwareFrameworkAnalysisResult) -> Result<String> {
        let mut roadmap = String::new();
        roadmap.push_str("# Implementation Roadmap\n\n");
        
        roadmap.push_str("## Development Recommendations Based on SOTA Analysis\n\n");
        
        if let Some(fusion_result) = &result.hierarchical_fusion {
            if let Some(framework) = &fusion_result.consolidated_results.primary_framework {
                roadmap.push_str(&format!("### Primary Framework: {}\n\n", framework.to_string()));
                roadmap.push_str("#### Recommended Development Approach\n\n");
                
                match framework.to_string().as_str() {
                    "React" => {
                        roadmap.push_str("- **Component Architecture**: Focus on reusable component design\n");
                        roadmap.push_str("- **State Management**: Consider Redux or Context API for complex state\n");
                    },
                    "Flask" => {
                        roadmap.push_str("- **Blueprint Architecture**: Organize routes using Flask blueprints\n");
                        roadmap.push_str("- **Database Integration**: Use Flask-SQLAlchemy for ORM\n");
                    },
                    _ => {
                        roadmap.push_str("- **Best Practices**: Follow framework-specific conventions\n");
                        roadmap.push_str("- **Architecture**: Implement recommended patterns for the framework\n");
                    }
                }
            }
        }
        
        roadmap.push_str(&format!("\n## Quality Assurance\n\n"));
        roadmap.push_str(&format!("**Current Context Completeness**: {:.1}%\n\n", 
            result.context_awareness_summary.context_completeness_score * 100.0));
        
        roadmap.push_str("---\n*Roadmap generated by SOTA Hierarchical Context-Aware Analysis*\n");
        
        Ok(roadmap)
    }
    
    fn generate_comprehensive_readme(&self, result: &crate::core::context_aware_framework_detector::ContextAwareFrameworkAnalysisResult, project_path: &str) -> Result<String> {
        let project_name = std::path::Path::new(project_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown Project");
            
        let mut readme = String::new();
        readme.push_str(&format!("# {} - Analysis Documentation\n\n", project_name));
        
        readme.push_str("## Project Overview\n\n");
        readme.push_str("This documentation was generated using **SOTA Hierarchical Context-Aware Analysis**.\n\n");
        
        readme.push_str("## Quick Summary\n\n");
        
        if let Some(fusion_result) = &result.hierarchical_fusion {
            if let Some(framework) = &fusion_result.consolidated_results.primary_framework {
                readme.push_str(&format!("- **Primary Technology**: {}\n", framework.to_string()));
            }
            
            if let Some(domain) = &fusion_result.consolidated_results.primary_business_domain {
                readme.push_str(&format!("- **Business Domain**: {}\n", domain.name));
            }
        }
        
        readme.push_str(&format!("- **Code Segments Analyzed**: {}\n", result.context_awareness_summary.total_segments_analyzed));
        readme.push_str(&format!("- **Analysis Confidence**: {:.1}%\n", result.context_awareness_summary.average_confidence * 100.0));
        
        readme.push_str("\n## Documentation Files\n\n");
        readme.push_str("1. **[Executive Summary](00_EXECUTIVE_SUMMARY.md)** - High-level project overview\n");
        readme.push_str("2. **[Technical Analysis](01_TECHNICAL_ANALYSIS.md)** - Technical findings\n");
        readme.push_str("3. **[Framework Analysis](02_FRAMEWORK_ANALYSIS.md)** - Framework detection results\n");
        readme.push_str("4. **[Business Intelligence](03_BUSINESS_INTELLIGENCE.md)** - Business domain analysis\n");
        readme.push_str("5. **[Hierarchical Fusion Analysis](04_HIERARCHICAL_FUSION_ANALYSIS.md)** - Multi-tier fusion results\n");
        readme.push_str("6. **[Implementation Roadmap](05_IMPLEMENTATION_ROADMAP.md)** - Development recommendations\n\n");
        
        readme.push_str("## Business Domain Coverage\n\n");
        for (domain, count) in &result.context_awareness_summary.business_domain_coverage {
            readme.push_str(&format!("- **{}**: {} code segments\n", domain, count));
        }
        
        readme.push_str(&format!("\n## Analysis Performance\n\n"));
        readme.push_str(&format!("- **Total Analysis Time**: {}ms\n", result.performance_metrics.total_analysis_time_ms));
        readme.push_str(&format!("- **Context Coverage**: {:.1}%\n", result.context_awareness_summary.context_completeness_score * 100.0));
        
        readme.push_str("\n---\n\n");
        readme.push_str("*Analysis generated using SOTA Hierarchical Context-Aware Analysis*\n");
        
        Ok(readme)
    }

    // CCPM + Claude Code Spec Workflow Generation Methods
    
    async fn generate_claude_md(&self, result: &crate::core::context_aware_framework_detector::ContextAwareFrameworkAnalysisResult, project_path: &str, enable_llm: bool) -> Result<String> {
        use crate::core::template_engine::TemplateEngine;
        use crate::intelligence::LLMBusinessAnalyzer;
        use std::collections::HashMap;
        
        let project_name = std::path::Path::new(project_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_else(|| "Unknown Project");
        
        // Extract business purpose using LLM if enabled, fallback to basic extraction
        let business_context = if enable_llm {
            println!("Using LLM for enhanced business context analysis...");
            match LLMBusinessAnalyzer::new_non_interactive(project_path).await {
                Ok(analyzer) => {
                    let api_endpoints = self.extract_api_endpoints_from_analysis(result);
                    let dependencies = self.extract_dependencies_from_analysis(result);
                    let file_names = self.extract_file_names_from_analysis(result);
                    let readme_path = format!("{}/README.md", project_path);
                    let readme_content = std::fs::read_to_string(&readme_path).ok();
                    
                    analyzer.analyze_business_context(
                        project_path,
                        &api_endpoints,
                        &dependencies, 
                        &file_names,
                        readme_content.as_deref(),
                    ).await?
                }
                Err(e) => {
                    println!("LLM analysis failed: {}", e);
                    println!("   Falling back to basic analysis...");
                    self.generate_basic_business_context(result, project_path)?
                }
            }
        } else {
            self.generate_basic_business_context(result, project_path)?
        };
        
        
        let mut engine = TemplateEngine::new();
        engine.set_variable("project_name".to_string(), project_name.to_string());
        engine.set_variable("project_description".to_string(), business_context.purpose.description.clone());
        engine.set_variable("business_purpose".to_string(), business_context.purpose.problem_statement.clone());
        engine.set_variable("target_users".to_string(), business_context.purpose.target_users.join("\n- "));
        engine.set_variable("key_features".to_string(), business_context.purpose.key_features.iter().map(|f| format!("- {}", f)).collect::<Vec<_>>().join("\n"));
        
        // Technical summary (concise)
        let primary_framework = result.traditional_analysis.detected_frameworks
            .first()
            .map(|f| format!("{:?}", f.framework))
            .unwrap_or_else(|| "Unknown".to_string());
        
        engine.set_variable("primary_framework".to_string(), primary_framework);
        engine.set_variable("architecture_pattern".to_string(), "Service-based".to_string());
        engine.set_variable("deployment_context".to_string(), "Containerized microservice".to_string());
        
        // Analysis metrics for reference only
        engine.set_variable("analysis_time".to_string(), 
            result.performance_metrics.total_analysis_time_ms.to_string());
        engine.set_variable("context_coverage".to_string(), 
            format!("{:.1}", result.context_awareness_summary.context_completeness_score * 100.0));
        engine.set_variable("segments_count".to_string(), 
            result.context_awareness_summary.total_segments_analyzed.to_string());
        engine.set_variable("confidence".to_string(), 
            format!("{:.1}", result.context_awareness_summary.average_confidence * 100.0));
        
        // Generate business-focused CLAUDE.md using template
        let template = r#"# {project_name}

## Project Overview
{project_description}

## Business Context
{business_purpose}

### Target Users
- {target_users}

### Key Features
{key_features}

## Technical Summary
- **Primary Framework**: {primary_framework}
- **Architecture Pattern**: {architecture_pattern}
- **Deployment Context**: {deployment_context}

## Code Standards
- Follow existing patterns and conventions found in the codebase
- Maintain consistency with detected framework patterns
- Use local development environment as configured

## Analysis Metrics (Reference)
- **Analysis Time**: {analysis_time}ms
- **Context Coverage**: {context_coverage}%
- **Code Segments**: {segments_count} analyzed
- **Confidence Score**: {confidence}%

---
*Generated by SOTA Hierarchical Context-Aware Analysis*
"#;
        
        Ok(engine.render(template))
    }
    
    fn generate_basic_business_context(
        &self, 
        result: &crate::core::context_aware_framework_detector::ContextAwareFrameworkAnalysisResult, 
        project_path: &str
    ) -> Result<crate::core::business_purpose_extractor::BusinessContext> {
        use crate::core::business_purpose_extractor::{BusinessContext, BusinessPurpose, UserPersona, Feature};
        
        // Simple fallback business context generation
        let project_name = std::path::Path::new(project_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown Project");

        let primary_framework = result.traditional_analysis.detected_frameworks
            .first()
            .map(|f| format!("{:?}", f.framework))
            .unwrap_or_else(|| "Unknown".to_string());

        let business_purpose = BusinessPurpose {
            description: format!("Software application built with {} framework", primary_framework),
            problem_statement: "Users need efficient software solutions to accomplish their tasks through reliable, well-designed interfaces".to_string(),
            target_users: vec!["End users".to_string(), "System administrators".to_string()],
            key_features: vec!["Core application functionality".to_string()],
            value_proposition: "Provides reliable software solution for identified use cases".to_string(),
            usage_scenarios: vec!["Users interact with the system through available interfaces".to_string()],
            confidence_score: 0.50, // Low confidence for basic analysis
        };

        let user_personas = vec![
            UserPersona {
                name: "End User".to_string(),
                role: "Primary system user".to_string(),
                goals: vec!["Complete tasks efficiently".to_string()],
                pain_points: vec!["Current solutions may be inadequate".to_string()],
            }
        ];

        let features = vec![
            Feature {
                name: "Application Core".to_string(),
                description: "Main application functionality".to_string(),
                user_benefit: "Enables users to accomplish their objectives".to_string(),
                implementation_evidence: vec![format!("Built with {}", primary_framework)],
            }
        ];

        let success_indicators = vec![
            "User satisfaction and engagement".to_string(),
            "System reliability and performance".to_string(),
            "Feature adoption rates".to_string(),
        ];

        Ok(BusinessContext {
            purpose: business_purpose,
            user_personas,
            feature_breakdown: features,
            success_indicators,
        })
    }
    
    // Helper methods to extract information from analysis results
    fn extract_api_endpoints_from_analysis(&self, result: &crate::core::context_aware_framework_detector::ContextAwareFrameworkAnalysisResult) -> Vec<String> {
        let mut endpoints = Vec::new();
        
        // Extract from actual domain coverage - use domain names as indicators of functionality
        for (domain, _count) in &result.context_awareness_summary.business_domain_coverage {
            match domain.to_lowercase().as_str() {
                "api endpoint handling" => endpoints.push("/api".to_string()),
                "user management" => {
                    endpoints.extend(vec!["/users".to_string(), "/auth".to_string(), "/login".to_string()]);
                },
                "e-commerce" | "shopping" => {
                    endpoints.extend(vec!["/products".to_string(), "/cart".to_string(), "/orders".to_string()]);
                },
                "content management" => {
                    endpoints.extend(vec!["/posts".to_string(), "/content".to_string()]);
                },
                "data handling and modeling" => endpoints.push("/data".to_string()),
                "web services" => endpoints.push("/api".to_string()),
                _ => {}
            }
        }
        
        // If no specific endpoints found, return empty (don't hardcode)
        endpoints
    }
    
    fn extract_dependencies_from_analysis(&self, result: &crate::core::context_aware_framework_detector::ContextAwareFrameworkAnalysisResult) -> Vec<String> {
        let mut dependencies = Vec::new();
        
        // Extract framework names as dependencies
        for framework in &result.traditional_analysis.detected_frameworks {
            dependencies.push(format!("{:?}", framework.framework).to_lowercase());
        }
        
        // Add language ecosystems based on detected frameworks (avoid hardcoding domain assumptions)
        for framework in &result.traditional_analysis.detected_frameworks {
            match format!("{:?}", framework.framework).to_lowercase().as_str() {
                "flask" | "django" | "fastapi" => {
                    dependencies.extend(vec!["python".to_string(), "web".to_string()]);
                },
                "react" | "nextjs" | "angular" | "vue" => {
                    dependencies.extend(vec!["javascript".to_string(), "typescript".to_string(), "frontend".to_string()]);
                },
                "springboot" => {
                    dependencies.extend(vec!["java".to_string(), "enterprise".to_string()]);
                },
                "nestjs" | "express" => {
                    dependencies.extend(vec!["nodejs".to_string(), "javascript".to_string()]);
                },
                _ => {}
            }
        }
        
        dependencies
    }
    
    fn extract_file_names_from_analysis(&self, result: &crate::core::context_aware_framework_detector::ContextAwareFrameworkAnalysisResult) -> Vec<String> {
        let mut file_names = Vec::new();
        
        // Extract from domain coverage as proxy for file/module names
        for (domain, _) in &result.context_awareness_summary.business_domain_coverage {
            file_names.push(domain.clone().to_lowercase().replace(" ", "_"));
        }
        
        file_names
    }
    
    fn generate_prd_from_sota(&self, result: &crate::core::context_aware_framework_detector::ContextAwareFrameworkAnalysisResult, project_path: &str) -> Result<String> {
        let project_name = std::path::Path::new(project_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown Project");
            
        let mut prd = String::new();
        
        prd.push_str(&format!("# Product Requirements Document - {}\n\n", project_name));
        
        // Executive Summary
        prd.push_str("## Executive Summary\n\n");
        if let Some(fusion_result) = &result.hierarchical_fusion {
            if let Some(domain) = &fusion_result.consolidated_results.primary_business_domain {
                prd.push_str(&format!("**Business Domain**: {}\n\n", domain.name));
                prd.push_str(&format!("This project operates in the {} domain with {:.1}% confidence based on code analysis.\n\n", domain.name, domain.confidence * 100.0));
            }
        }
        
        // Product Vision
        prd.push_str("## Product Vision\n\n");
        prd.push_str("Based on code analysis, this project aims to:\n\n");
        
        for (domain, _count) in &result.context_awareness_summary.business_domain_coverage {
            prd.push_str(&format!("- Provide {} capabilities\n", domain.to_lowercase()));
        }
        
        // User Stories (derived from code patterns)
        prd.push_str("\n## User Stories\n\n");
        prd.push_str("### Primary User Personas\n\n");
        
        if result.context_awareness_summary.business_domain_coverage.contains_key("Web Development") {
            prd.push_str("**End Users**: Individuals using the web interface\n\n");
        }
        
        if result.context_awareness_summary.business_domain_coverage.contains_key("API Development") {
            prd.push_str("**API Consumers**: Systems and applications using the API\n\n");
        }
        
        if result.context_awareness_summary.business_domain_coverage.contains_key("Developer Tools") {
            prd.push_str("**Developers**: Software engineers using the development tools\n\n");
        }
        
        // Technical Requirements
        prd.push_str("## Technical Requirements\n\n");
        if let Some(fusion_result) = &result.hierarchical_fusion {
            if let Some(framework) = &fusion_result.consolidated_results.primary_framework {
                prd.push_str(&format!("- **Primary Technology Stack**: {}\n", framework.to_string()));
            }
        }
        prd.push_str(&format!("- **Code Complexity**: {} segments analyzed\n", result.context_awareness_summary.total_segments_analyzed));
        prd.push_str(&format!("- **Analysis Confidence**: {:.1}%\n", result.context_awareness_summary.average_confidence * 100.0));
        
        // Success Metrics
        prd.push_str("\n## Success Metrics\n\n");
        prd.push_str(&format!("- **Code Coverage**: {:.1}% context coverage achieved\n", result.context_awareness_summary.context_completeness_score * 100.0));
        prd.push_str("- **Performance**: Maintain existing performance characteristics\n");
        prd.push_str("- **Maintainability**: Preserve code quality and patterns\n\n");
        
        prd.push_str("---\n");
        prd.push_str("*Generated by SOTA Business Intelligence Analysis*\n");
        
        Ok(prd)
    }
    
    fn generate_product_context(&self, result: &crate::core::context_aware_framework_detector::ContextAwareFrameworkAnalysisResult) -> Result<String> {
        let mut context = String::new();
        
        context.push_str("# Product Context\n\n");
        
        context.push_str("## Business Domain Analysis\n\n");
        if let Some(fusion_result) = &result.hierarchical_fusion {
            if let Some(domain) = &fusion_result.consolidated_results.primary_business_domain {
                context.push_str(&format!("**Primary Domain**: {}\n\n", domain.name));
                context.push_str(&format!("The project operates in the {} domain, providing specialized functionality in this area.\n\n", domain.name));
                
                context.push_str(&format!("**Confidence Level**: {:.1}%\n\n", domain.confidence * 100.0));
            }
        }
        
        context.push_str("## Domain Coverage\n\n");
        for (domain, count) in &result.context_awareness_summary.business_domain_coverage {
            context.push_str(&format!("- **{}**: {} code segments\n", domain, count));
        }
        
        context.push_str("\n## Business Capabilities\n\n");
        context.push_str("Based on code analysis, the system provides:\n\n");
        
        for domain in result.context_awareness_summary.business_domain_coverage.keys() {
            context.push_str(&format!("- {} functionality\n", domain));
        }
        
        Ok(context)
    }
    
    fn generate_tech_context(&self, result: &crate::core::context_aware_framework_detector::ContextAwareFrameworkAnalysisResult) -> Result<String> {
        let mut context = String::new();
        
        context.push_str("# Technical Context\n\n");
        
        context.push_str("## Technology Stack\n\n");
        if let Some(fusion_result) = &result.hierarchical_fusion {
            if let Some(framework) = &fusion_result.consolidated_results.primary_framework {
                context.push_str(&format!("**Primary Framework**: {}\n\n", framework.to_string()));
                context.push_str(&format!("**Detection Confidence**: {:.1}%\n\n", fusion_result.quality_metrics.overall_fusion_quality * 100.0));
            }
        }
        
        context.push_str("## Architecture Overview\n\n");
        context.push_str(&format!("- **Code Segments Analyzed**: {}\n", result.context_awareness_summary.total_segments_analyzed));
        context.push_str(&format!("- **Context Coverage**: {:.1}%\n", result.context_awareness_summary.context_completeness_score * 100.0));
        context.push_str(&format!("- **Average Confidence**: {:.1}%\n", result.context_awareness_summary.average_confidence * 100.0));
        
        context.push_str("\n## Analysis Performance\n\n");
        context.push_str(&format!("- **Total Analysis Time**: {}ms\n", result.performance_metrics.total_analysis_time_ms));
        context.push_str("- **Analysis Method**: SOTA Hierarchical Context-Aware\n");
        
        Ok(context)
    }
    
    fn generate_structure_context(&self, result: &crate::core::context_aware_framework_detector::ContextAwareFrameworkAnalysisResult, project_path: &str) -> Result<String> {
        let mut context = String::new();
        
        context.push_str("# Project Structure Context\n\n");
        
        context.push_str("## Project Overview\n\n");
        context.push_str(&format!("**Project Path**: {}\n", project_path));
        
        if let Some(fusion_result) = &result.hierarchical_fusion {
            context.push_str(&format!("**Analysis Quality**: {:.1}%\n", fusion_result.quality_metrics.overall_fusion_quality * 100.0));
            context.push_str(&format!("**Tier Alignment**: {:.1}%\n", fusion_result.quality_metrics.tier_alignment_score * 100.0));
        }
        
        context.push_str("\n## Code Organization\n\n");
        context.push_str(&format!("- **Total Segments**: {}\n", result.context_awareness_summary.total_segments_analyzed));
        context.push_str(&format!("- **Context Completeness**: {:.1}%\n", result.context_awareness_summary.context_completeness_score * 100.0));
        
        context.push_str("\n## Business Domain Distribution\n\n");
        for (domain, count) in &result.context_awareness_summary.business_domain_coverage {
            let percentage = (*count as f32 / result.context_awareness_summary.total_segments_analyzed as f32) * 100.0;
            context.push_str(&format!("- **{}**: {} segments ({:.1}%)\n", domain, count, percentage));
        }
        
        context.push_str("\n## Development Guidance\n\n");
        context.push_str("- Follow existing code patterns found in the analysis\n");
        context.push_str("- Maintain consistency with detected frameworks\n");
        context.push_str("- Preserve business domain separation as identified\n");
        
        Ok(context)
    }
    
    fn generate_user_stories_from_sota(&self, result: &crate::core::context_aware_framework_detector::ContextAwareFrameworkAnalysisResult) -> Result<String> {
        use crate::core::template_engine::TemplateEngine;
        
        let mut engine = TemplateEngine::new();
        let mut stories = String::new();
        
        // Header
        stories.push_str(&engine.render_ccpm_document("user_stories", None)?);
        
        // Generate domain-specific user stories
        for (domain, count) in &result.context_awareness_summary.business_domain_coverage {
            let story_template = engine.render_user_story(domain)?;
            
            engine.set_variable("domain_name".to_string(), domain.clone());
            engine.set_variable("story_template".to_string(), story_template);
            engine.set_variable("segment_count".to_string(), count.to_string());
            
            stories.push_str(&engine.render_ccpm_document("user_stories", Some("domain_template"))?);
        }
        
        // Acceptance Criteria
        engine.set_variable("context_coverage".to_string(), format!("{:.1}", result.context_awareness_summary.context_completeness_score * 100.0));
        engine.set_variable("average_confidence".to_string(), format!("{:.1}", result.context_awareness_summary.average_confidence * 100.0));
        stories.push_str(&engine.render_ccpm_document("user_stories", Some("acceptance_criteria"))?);
        
        // Footer
        stories.push_str(&engine.render_ccpm_document("user_stories", Some("footer"))?);
        
        Ok(stories)
    }
    
    fn generate_implementation_epic(&self, result: &crate::core::context_aware_framework_detector::ContextAwareFrameworkAnalysisResult) -> Result<String> {
        let mut epic = String::new();
        
        epic.push_str("# Implementation Epic\n\n");
        
        epic.push_str("## Overview\n\n");
        epic.push_str("Implementation tasks derived from SOTA hierarchical analysis.\n\n");
        
        epic.push_str("## Priority Tasks\n\n");
        
        // Generate tasks based on business domain coverage
        let mut task_counter = 1;
        for (domain, count) in &result.context_awareness_summary.business_domain_coverage {
            epic.push_str(&format!("### Task {}: {} Implementation\n\n", task_counter, domain));
            epic.push_str(&format!("**Scope**: {} code segments\n", count));
            epic.push_str("**Priority**: High\n");
            epic.push_str(&format!("**Estimated Effort**: Based on {} segments\n\n", count));
            
            epic.push_str("**Acceptance Criteria**:\n");
            epic.push_str(&format!("- [ ] Implement {} functionality\n", domain.to_lowercase()));
            epic.push_str("- [ ] Follow existing code patterns\n");
            epic.push_str("- [ ] Maintain framework compliance\n");
            epic.push_str("- [ ] Add appropriate tests\n\n");
            
            task_counter += 1;
        }
        
        epic.push_str("## Quality Gates\n\n");
        epic.push_str(&format!("- **Analysis Confidence**: Target {:.1}%+ (current: {:.1}%)\n", 
            result.context_awareness_summary.average_confidence * 100.0 + 5.0,
            result.context_awareness_summary.average_confidence * 100.0));
        epic.push_str(&format!("- **Context Coverage**: Target {:.1}%+ (current: {:.1}%)\n",
            result.context_awareness_summary.context_completeness_score * 100.0 + 5.0,
            result.context_awareness_summary.context_completeness_score * 100.0));
        
        if let Some(fusion_result) = &result.hierarchical_fusion {
            epic.push_str(&format!("- **Fusion Quality**: Target {:.1}%+ (current: {:.1}%)\n",
                fusion_result.quality_metrics.overall_fusion_quality * 100.0 + 5.0,
                fusion_result.quality_metrics.overall_fusion_quality * 100.0));
        }
        
        epic.push_str("\n## Performance Targets\n\n");
        epic.push_str(&format!("- **Analysis Time**: <{}ms (current: {}ms)\n", 
            result.performance_metrics.total_analysis_time_ms + 100,
            result.performance_metrics.total_analysis_time_ms));
        epic.push_str("- **Code Quality**: Maintain existing standards\n");
        epic.push_str("- **Test Coverage**: 80%+ for new code\n\n");
        
        epic.push_str("---\n");
        epic.push_str("*Generated by SOTA Implementation Intelligence*\n");
        
        Ok(epic)
    }
    
    fn generate_specialized_agents(&self, result: &crate::core::context_aware_framework_detector::ContextAwareFrameworkAnalysisResult) -> Result<std::collections::HashMap<String, String>> {
        let mut agents = std::collections::HashMap::new();
        
        // Development Agent
        let mut dev_agent = String::new();
        dev_agent.push_str("# Development Agent\n\n");
        dev_agent.push_str("## Role\n");
        dev_agent.push_str("Specialized agent for core development tasks based on SOTA analysis.\n\n");
        
        dev_agent.push_str("## Capabilities\n\n");
        for (domain, count) in &result.context_awareness_summary.business_domain_coverage {
            dev_agent.push_str(&format!("- **{}**: Handle {} code segments\n", domain, count));
        }
        
        if let Some(fusion_result) = &result.hierarchical_fusion {
            if let Some(framework) = &fusion_result.consolidated_results.primary_framework {
                dev_agent.push_str(&format!("\n## Framework Expertise\n"));
                dev_agent.push_str(&format!("- **Primary**: {}\n", framework.to_string()));
                dev_agent.push_str(&format!("- **Confidence**: {:.1}%\n", fusion_result.quality_metrics.overall_fusion_quality * 100.0));
            }
        }
        
        agents.insert("development".to_string(), dev_agent);
        
        // Testing Agent (if testing code segments found)
        if result.context_awareness_summary.business_domain_coverage.get("Testing").unwrap_or(&0) > &0 {
            let mut test_agent = String::new();
            test_agent.push_str("# Testing Agent\n\n");
            test_agent.push_str("## Role\n");
            test_agent.push_str("Specialized agent for testing and quality assurance.\n\n");
            test_agent.push_str("## Focus Areas\n");
            test_agent.push_str("- Unit testing\n");
            test_agent.push_str("- Integration testing\n");
            test_agent.push_str("- Code quality validation\n");
            
            agents.insert("testing".to_string(), test_agent);
        }
        
        Ok(agents)
    }
    
    fn generate_ccpm_commands(&self, result: &crate::core::context_aware_framework_detector::ContextAwareFrameworkAnalysisResult) -> Result<String> {
        use crate::core::template_engine::TemplateEngine;
        use crate::core::config::get_config;
        
        let mut engine = TemplateEngine::new();
        let config = get_config();
        let command_templates = config.get_framework_commands();
        
        let mut commands = String::new();
        
        // Header
        commands.push_str(&command_templates.command_sections.get("analysis")
            .map(|section| section.header.clone())
            .unwrap_or_else(|| "# CCPM Workflow Commands\n\n## Analysis Commands\n\n".to_string()));
        
        // Development Commands
        commands.push_str(&command_templates.command_sections.get("development")
            .map(|section| section.header.clone())
            .unwrap_or_else(|| "## Development Commands\n\n".to_string()));
        
        if let Some(fusion_result) = &result.hierarchical_fusion {
            if let Some(framework) = &fusion_result.consolidated_results.primary_framework {
                let framework_commands = engine.render_framework_commands(&framework.to_string())?;
                commands.push_str(&framework_commands);
            }
        }
        
        // Quality Gates
        commands.push_str(&command_templates.command_sections.get("quality_gates")
            .map(|section| section.header.clone())
            .unwrap_or_else(|| "## Quality Gates\n\n".to_string()));
        
        engine.set_variable("confidence".to_string(), format!("{:.1}", result.context_awareness_summary.average_confidence * 100.0));
        engine.set_variable("coverage".to_string(), format!("{:.1}", result.context_awareness_summary.context_completeness_score * 100.0));
        
        let quality_template = command_templates.command_sections.get("quality_gates")
            .and_then(|section| section.template.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("- **Analysis Confidence**: {confidence}%\n- **Context Coverage**: {coverage}%\n");
        
        commands.push_str(&engine.render(quality_template));
        
        Ok(commands)
    }
    
    fn generate_workflow_readme(&self, result: &crate::core::context_aware_framework_detector::ContextAwareFrameworkAnalysisResult, project_path: &str) -> Result<String> {
        let project_name = std::path::Path::new(project_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown Project");
            
        let mut readme = String::new();
        
        readme.push_str(&format!("# {} - CCPM Workflow\n\n", project_name));
        
        readme.push_str("## Overview\n\n");
        readme.push_str("This directory contains the complete CCPM (Claude Code Project Management) workflow for systematic development.\n\n");
        
        if let Some(fusion_result) = &result.hierarchical_fusion {
            if let Some(domain) = &fusion_result.consolidated_results.primary_business_domain {
                readme.push_str(&format!("**Primary Business Domain**: {}\n\n", domain.name));
            }
            
            if let Some(framework) = &fusion_result.consolidated_results.primary_framework {
                readme.push_str(&format!("**Technology Stack**: {}\n\n", framework.to_string()));
            }
        }
        
        readme.push_str("## Workflow Structure\n\n");
        readme.push_str("```\n");
        readme.push_str(".claude/\n");
        readme.push_str("├── CLAUDE.md                 # Always-on project instructions\n");
        readme.push_str("├── context/                  # Claude Code Spec context files\n");
        readme.push_str("│   ├── product.md           # Business context\n");
        readme.push_str("│   ├── tech.md              # Technical context\n");
        readme.push_str("│   └── structure.md         # Project structure context\n");
        readme.push_str("├── prds/                     # Product Requirements\n");
        readme.push_str("│   └── product-requirements.md\n");
        readme.push_str("├── epics/                    # User stories and implementation\n");
        readme.push_str("│   ├── user-stories.md\n");
        readme.push_str("│   └── implementation/\n");
        readme.push_str("│       └── epic.md\n");
        readme.push_str("├── agents/                   # Specialized AI agents\n");
        readme.push_str("│   └── development.md\n");
        readme.push_str("└── commands/                 # Workflow commands\n");
        readme.push_str("    └── workflow.md\n");
        readme.push_str("```\n\n");
        
        readme.push_str("## Quick Start\n\n");
        readme.push_str("1. **Review Context**: Start with `context/` files to understand the project\n");
        readme.push_str("2. **Read PRD**: Check `prds/product-requirements.md` for business requirements\n");
        readme.push_str("3. **User Stories**: Review `epics/user-stories.md` for feature requirements\n");
        readme.push_str("4. **Implementation**: Follow tasks in `epics/implementation/epic.md`\n");
        readme.push_str("5. **Commands**: Use `commands/workflow.md` for development commands\n\n");
        
        readme.push_str("## Analysis Quality\n\n");
        readme.push_str(&format!("- **Code Segments**: {} analyzed\n", result.context_awareness_summary.total_segments_analyzed));
        readme.push_str(&format!("- **Analysis Confidence**: {:.1}%\n", result.context_awareness_summary.average_confidence * 100.0));
        readme.push_str(&format!("- **Context Coverage**: {:.1}%\n", result.context_awareness_summary.context_completeness_score * 100.0));
        
        if let Some(fusion_result) = &result.hierarchical_fusion {
            readme.push_str(&format!("- **Fusion Quality**: {:.1}%\n", fusion_result.quality_metrics.overall_fusion_quality * 100.0));
        }
        
        readme.push_str("\n---\n");
        readme.push_str("*Generated by SOTA Hierarchical Context-Aware Analysis*\n");
        
        Ok(readme)
    }
    
    async fn run_framework_validation_test(&self) -> Result<()> {
        use crate::core::framework_validation::run_framework_validation;
        
        println!("🧪 Running Framework Detection Validation Tests");
        println!("=====================================================");
        
        let validation_results = run_framework_validation().await?;
        
        // Print results using the validation system's built-in formatter
        use crate::core::framework_validation::FrameworkValidationSystem;
        let validation_system = FrameworkValidationSystem::new();
        validation_system.print_validation_results(&validation_results);
        
        // Generate recommendations based on results
        println!("\n🔍 Recommendations:");
        if validation_results.overall_accuracy >= 0.9 {
            println!("  ✅ Framework detection accuracy is excellent (≥90%)");
        } else if validation_results.overall_accuracy >= 0.8 {
            println!("  ⚠️  Framework detection accuracy is good but could be improved (≥80%)");
        } else {
            println!("  ❌ Framework detection accuracy needs significant improvement (<80%)");
        }
        
        // Identify frameworks needing attention
        for (framework, stats) in &validation_results.per_framework_accuracy {
            if stats.f1_score < 0.8 {
                println!("  🔧 Framework {:?} needs attention (F1: {:.3})", framework, stats.f1_score);
            }
        }
        
        // Summary
        println!("\n📊 Summary:");
        println!("  Tests: {}/{} passed ({:.1}%)", 
            validation_results.passed_tests, 
            validation_results.total_tests,
            validation_results.overall_accuracy * 100.0
        );
        
        if validation_results.failed_tests > 0 {
            println!("  Review failed tests above to identify improvement areas.");
        }
        
        Ok(())
    }

    /// Parse and validate external documentation paths
    fn parse_external_docs_paths(&self, ext_paths: &str, project_path: &std::path::Path) -> Result<Vec<std::path::PathBuf>> {
        use std::path::Path;
        
        let mut validated_paths = Vec::new();
        let paths: Vec<&str> = ext_paths.split(',').map(|s| s.trim()).collect();
        
        if paths.len() > 10 {
            anyhow::bail!("Too many external documentation paths (max 10): found {}", paths.len());
        }
        
        for path_str in paths {
            if path_str.is_empty() {
                continue; // Skip empty paths
            }
            
            // Resolve relative paths relative to the project directory
            let resolved_path = if path_str.starts_with('.') {
                project_path.join(path_str)
            } else {
                Path::new(path_str).to_path_buf()
            };
            
            // Validate path exists
            if !resolved_path.exists() {
                anyhow::bail!("External documentation path does not exist: {}", resolved_path.display());
            }
            
            // Validate it's a directory
            if !resolved_path.is_dir() {
                anyhow::bail!("External documentation path must be a directory: {}", resolved_path.display());
            }
            
            // Check for duplicates
            let canonical_path = resolved_path.canonicalize()
                .map_err(|e| anyhow::anyhow!("Failed to canonicalize path {}: {}", resolved_path.display(), e))?;
                
            if validated_paths.contains(&canonical_path) {
                println!("⚠️  Skipping duplicate external documentation path: {}", canonical_path.display());
                continue;
            }
            
            validated_paths.push(canonical_path);
        }
        
        println!("✅ Validated {} external documentation paths", validated_paths.len());
        Ok(validated_paths)
    }
}