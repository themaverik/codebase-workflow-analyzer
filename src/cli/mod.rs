use clap::{Parser, Subcommand};
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
    /// Analyze a codebase (basic functionality)
    Analyze {
        /// Path to the project directory
        #[arg(short, long)]
        path: String,
        
        /// Force specific analyzer (typescript, java, python)
        #[arg(long)]
        analyzer: Option<String>,
        
        /// Generate enhanced documentation
        #[arg(long)]
        generate_docs: bool,
        
        /// Output directory for generated documents
        #[arg(long)]
        docs_dir: Option<String>,
        
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
                generate_docs, 
                docs_dir,
                #[cfg(feature = "integrations")]
                enable_integrations
            } => {
                #[cfg(feature = "integrations")]
                {
                    self.run_analysis_with_integrations(path, analyzer, generate_docs, docs_dir, enable_integrations).await
                }
                #[cfg(not(feature = "integrations"))]
                {
                    self.run_analysis(path, analyzer, generate_docs, docs_dir).await
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
            #[cfg(feature = "web-server")]
            Commands::Serve { port } => {
                crate::server::start_server(port).await
            }
        }
    }
    
    async fn run_analysis(&self, path: String, analyzer: Option<String>, generate_docs: bool, docs_dir: Option<String>) -> Result<()> {
        println!("Analyzing project at: {}", path);
        
        // Check if path exists
        if !std::path::Path::new(&path).exists() {
            anyhow::bail!("Path does not exist: {}", path);
        }
        
        // Use the actual analyzers instead of basic file scanning
        use crate::core::{CodebaseAnalyzer, AnalyzerConfig};
        use crate::analyzers::{TypeScriptAnalyzer, JavaAnalyzer, PythonAnalyzer};
        
        let config = AnalyzerConfig::default();
        
        // Select analyzer based on parameter or auto-detection
        let (selected_analyzer, analyzer_name): (Box<dyn CodebaseAnalyzer>, &str) = 
            if let Some(analyzer_name) = &analyzer {
                match analyzer_name.to_lowercase().as_str() {
                    "typescript" | "ts" => {
                        (Box::new(TypeScriptAnalyzer::new(config)), "TypeScript")
                    },
                    "java" => {
                        (Box::new(JavaAnalyzer::new(config)), "Java")
                    },
                    "python" | "py" => {
                        (Box::new(PythonAnalyzer::new(config)), "Python")
                    },
                    _ => {
                        anyhow::bail!("Unsupported analyzer: {}. Use 'typescript', 'java', or 'python'", analyzer_name);
                    }
                }
            } else {
                // Auto-detect analyzer
                let ts_analyzer = TypeScriptAnalyzer::new(config.clone());
                let java_analyzer = JavaAnalyzer::new(config.clone());
                let python_analyzer = PythonAnalyzer::new(config.clone());
                
                if ts_analyzer.can_analyze(&path) {
                    (Box::new(ts_analyzer), "TypeScript")
                } else if java_analyzer.can_analyze(&path) {
                    (Box::new(java_analyzer), "Java")
                } else if python_analyzer.can_analyze(&path) {
                    (Box::new(python_analyzer), "Python")
                } else {
                    // Default to TypeScript analyzer
                    (Box::new(TypeScriptAnalyzer::new(config)), "TypeScript")
                }
            };
        
        println!("Using {} analyzer", analyzer_name);
        println!("Running comprehensive analysis...");
        
        // Run the actual analysis
        match selected_analyzer.analyze(&path) {
            Ok(analysis) => {
                println!("Analysis completed successfully!");
                println!("Project: {} ({:?})", analysis.project_name, analysis.project_type);
                println!("Components found: {}", analysis.components.len());
                println!("User stories generated: {}", analysis.user_stories.len());
                println!("Framework: {}", analysis.framework_analysis.architecture_pattern);
                println!("Business context: {} ({:.1}% confidence)", 
                    analysis.business_context.inferred_product_type,
                    analysis.business_context.confidence * 100.0);
                
                if generate_docs {
                    self.generate_documentation(&analysis, docs_dir, &path).await?;
                }
            },
            Err(e) => {
                anyhow::bail!("Analysis failed: {}", e);
            }
        }
        
        Ok(())
    }
    
    async fn generate_documentation(&self, analysis: &crate::core::CodebaseAnalysis, docs_dir: Option<String>, project_path: &str) -> Result<()> {
        use crate::generators::DocumentGeneratorFactory;
        
        let output_dir = docs_dir.unwrap_or_else(|| format!("{}/analysis-docs", project_path));
        println!("\nGenerating comprehensive documentation to: {}", output_dir);
        
        // Create output directory
        std::fs::create_dir_all(&output_dir)?;
        
        // Generate all enhanced documents using the factory
        let generated_docs = DocumentGeneratorFactory::generate_all_documents(analysis, &output_dir)?;
        
        println!("Generated {} enhanced documents:", generated_docs.len());
        for doc in &generated_docs {
            println!("  - {} ({:?})", doc.filename, doc.document_type);
        }
        println!("Documents saved to: {}", output_dir);
        
        Ok(())
    }
    
    #[cfg(feature = "integrations")]
    async fn run_analysis_with_integrations(&self, path: String, analyzer: Option<String>, generate_docs: bool, docs_dir: Option<String>, enable_integrations: bool) -> Result<()> {
        // Run basic analysis first
        self.run_analysis(path.clone(), analyzer, generate_docs, docs_dir).await?;
        
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
        println!("   Auto-detect project type");
        println!("   codebase-analyzer analyze --path /path/to/project");
        println!();
        println!("   Force specific analyzer");
        println!("   codebase-analyzer analyze --path /path/to/project --analyzer java");
        println!();
        println!("   Generate enhanced documentation");
        println!("   codebase-analyzer analyze --path /path/to/project --generate-docs");
        println!();
        println!("   Custom output directory for docs");
        println!("   codebase-analyzer analyze --path /path/to/project --generate-docs --docs-dir custom-docs");
        
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
                    println!("⚠️  Warning: Could not save results to {}: {}", output, e);
                    println!("   But analysis completed successfully!");
                } else {
                    println!("\n💾 Results saved to: {}", output);
                }
                
                println!("\n✅ Phase 1 Test Completed Successfully!");
                println!("\n📋 Summary:");
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
        println!("🚀 Starting AST Integration Test (Phase 2A)");
        
        // Use current directory if no path provided
        let test_path = path.unwrap_or_else(|| ".".to_string());
        
        // First test basic AST analyzer creation
        crate::core::ast_integration_test::run_basic_ast_test()?;
        
        // Then test with actual codebase
        if std::path::Path::new(&test_path).exists() {
            println!("\n🔍 Testing AST on path: {}", test_path);
            crate::core::ast_integration_test::test_ast_integration_with_path(&test_path).await?;
        } else {
            println!("⚠️  Path {} does not exist, skipping codebase test", test_path);
        }
        
        println!("✅ AST Integration test completed successfully!");
        Ok(())
    }
    
    async fn run_llm_test(&self, path: Option<String>, enable_llm: bool) -> Result<()> {
        use crate::core::enhanced_framework_detector::EnhancedFrameworkDetector;
        use crate::intelligence::llm_client::ModelConfig;
        
        println!("🚀 Starting LLM Integration Test (Phase 2B)");
        
        // Use current directory if no path provided
        let test_path = path.unwrap_or_else(|| ".".to_string());
        
        // Check if path exists
        if !std::path::Path::new(&test_path).exists() {
            anyhow::bail!("Path does not exist: {}", test_path);
        }
        
        println!("🔍 Testing hierarchical analysis on path: {}", test_path);
        
        // Create enhanced detector with AST analysis
        let mut detector = EnhancedFrameworkDetector::new(test_path.clone())?
            .with_ast_analysis()?;
        
        // Add LLM analysis if requested
        if enable_llm {
            println!("🧠 Initializing LLM integration...");
            detector = detector.with_llm_analysis(Some(ModelConfig::default())).await?;
        } else {
            println!("⚠️  LLM analysis disabled, running without LLM integration");
        }
        
        // Run enhanced analysis
        let result = detector.detect_frameworks_enhanced().await?;
        
        // Display results
        println!("\n📊 Hierarchical Analysis Results:");
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
                println!("\n    📋 Business Analysis:");
                println!("      Primary Domain: {}", project_analysis.primary_business_domain);
                println!("      Project Type: {}", project_analysis.project_type);
                
                println!("\n      🎯 Functional Requirements:");
                println!("        {}", project_analysis.functional_requirements.description);
                for (domain, analysis) in &project_analysis.functional_requirements.domains {
                    println!("        - {} ({}% confidence): {}", domain, (analysis.confidence * 100.0) as u32, analysis.description);
                }
                
                println!("\n      🔧 Non-Functional Requirements:");
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
        
        println!("\n✅ LLM Integration test completed successfully!");
        Ok(())
    }
}