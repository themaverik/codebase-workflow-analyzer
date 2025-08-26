use clap::{Parser, Subcommand};
use crate::core::{CodebaseAnalyzer, AnalyzerConfig, ProjectType};
use crate::analyzers::{TypeScriptAnalyzer, JavaAnalyzer, PythonAnalyzer};
use crate::intelligence::IntelligenceEngine;
use crate::generators::{DocumentGeneratorFactory, DocumentType};
use anyhow::Result;
use std::path::Path;

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
    /// Analyze a codebase (auto-detects TypeScript/React, Java/Spring Boot, Python/Django/Flask)
    Analyze {
        /// Path to the project directory
        #[arg(short, long)]
        path: String,
        
        /// Output format (json, yaml, summary)
        #[arg(short, long, default_value = "summary")]
        output: String,
        
        /// Output file path (optional, prints to stdout if not specified)
        #[arg(short = 'f', long)]
        output_file: Option<String>,
        
        /// Skip PRD generation
        #[arg(long)]
        skip_prd: bool,
        
        /// Skip user story inference
        #[arg(long)]
        skip_stories: bool,
        
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
        
        /// Force specific analyzer (typescript, java, python)
        #[arg(long)]
        analyzer: Option<String>,
        
        /// Generate enhanced documentation
        #[arg(long)]
        generate_docs: bool,
        
        /// Output directory for generated documents
        #[arg(long)]
        docs_dir: Option<String>,
    },
    
    /// List supported project types
    List,
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
                output, 
                output_file, 
                skip_prd, 
                skip_stories, 
                verbose,
                analyzer,
                generate_docs,
                docs_dir,
            } => {
                self.run_analysis(path, output, output_file, skip_prd, skip_stories, verbose, analyzer, generate_docs, docs_dir).await
            }
            Commands::List => {
                self.list_supported_types();
                Ok(())
            }
        }
    }
    
    async fn run_analysis(
        &self, 
        path: String, 
        output_format: String,
        output_file: Option<String>,
        skip_prd: bool,
        skip_stories: bool,
        verbose: bool,
        forced_analyzer: Option<String>,
        generate_docs: bool,
        docs_dir: Option<String>,
    ) -> Result<()> {
        if verbose {
            println!("🔍 Starting codebase analysis...");
            println!("📁 Project path: {}", path);
        }
        
        // Validate path exists
        if !Path::new(&path).exists() {
            anyhow::bail!("Project path does not exist: {}", path);
        }
        
        // Create analyzer config
        let mut config = AnalyzerConfig::default();
        config.generate_prd = !skip_prd;
        config.infer_user_stories = !skip_stories;
        
        // Detect or create appropriate analyzer
        let (analyzer_name, analysis) = if let Some(forced) = forced_analyzer {
            self.create_forced_analyzer(&forced, config, &path, verbose)?
        } else {
            self.detect_and_create_analyzer(config, &path, verbose)?
        };
        
        if verbose {
            println!("✅ Project type detected: {}", analyzer_name);
            println!("🔧 Analysis complete!");
        }
        
        if verbose {
            println!("   - Files analyzed: {}", analysis.analysis_metadata.files_analyzed);
            println!("   - Components found: {}", analysis.components.len());
            println!("   - User stories: {}", analysis.user_stories.len());
            println!("   - Tasks: {}", analysis.tasks.len());
        }
        
        // Enhanced document generation
        if generate_docs {
            let output_dir = docs_dir.unwrap_or_else(|| format!("{}-docs", analysis.project_name));
            
            if verbose {
                println!("📚 Generating enhanced documentation...");
            }
            
            // Create output directory
            std::fs::create_dir_all(&output_dir)?;
            
            // Generate all documents
            let generated_docs = DocumentGeneratorFactory::generate_all_documents(&analysis, &output_dir)?;
            
            if verbose {
                println!("✅ Generated {} documents:", generated_docs.len());
                for doc in &generated_docs {
                    println!("   - {} ({})", doc.filename, format!("{:?}", doc.document_type));
                }
                println!("📁 Documents saved to: {}", output_dir);
            }
        }
        
        // Generate standard output
        let output_content = match output_format.as_str() {
            "json" => serde_json::to_string_pretty(&analysis)?,
            "summary" => self.generate_summary(&analysis),
            _ => anyhow::bail!("Unsupported output format: {}", output_format),
        };
        
        // Write standard output
        match output_file {
            Some(file_path) => {
                std::fs::write(&file_path, output_content)?;
                if verbose {
                    println!("💾 Standard analysis saved to: {}", file_path);
                }
            }
            None => {
                println!("{}", output_content);
            }
        }
        
        Ok(())
    }
    
    fn detect_and_create_analyzer(&self, config: AnalyzerConfig, path: &str, verbose: bool) -> Result<(String, crate::core::CodebaseAnalysis)> {
        // Try analyzers in order of specificity
        
        // Try TypeScript/React first
        let ts_analyzer = TypeScriptAnalyzer::new(config.clone());
        if ts_analyzer.can_analyze(path) {
            if verbose {
                println!("🔧 Running TypeScript/React analysis...");
            }
            let analysis = ts_analyzer.analyze(path)?;
            return Ok(("TypeScript/React".to_string(), analysis));
        }
        
        // Try Java/Spring Boot
        let java_analyzer = JavaAnalyzer::new(config.clone());
        if java_analyzer.can_analyze(path) {
            if verbose {
                println!("🔧 Running Java/Spring Boot analysis...");
            }
            let analysis = java_analyzer.analyze(path)?;
            return Ok(("Java/Spring Boot".to_string(), analysis));
        }
        
        // Try Python/Django/Flask
        let python_analyzer = PythonAnalyzer::new(config.clone());
        if python_analyzer.can_analyze(path) {
            if verbose {
                println!("🔧 Running Python analysis...");
            }
            let analysis = python_analyzer.analyze(path)?;
            let framework_name = match analysis.project_type {
                ProjectType::Django => "Python/Django",
                ProjectType::Flask => "Python/Flask", 
                _ => "Python",
            };
            return Ok((framework_name.to_string(), analysis));
        }
        
        anyhow::bail!("No suitable analyzer found for project at '{}'. Supported types: TypeScript/React, Java/Spring Boot, Python/Django/Flask", path)
    }
    
    fn create_forced_analyzer(&self, analyzer_type: &str, config: AnalyzerConfig, path: &str, verbose: bool) -> Result<(String, crate::core::CodebaseAnalysis)> {
        match analyzer_type.to_lowercase().as_str() {
            "typescript" | "react" | "ts" => {
                if verbose {
                    println!("🔧 Force-running TypeScript/React analysis...");
                }
                let analyzer = TypeScriptAnalyzer::new(config);
                let analysis = analyzer.analyze(path)?;
                Ok(("TypeScript/React (forced)".to_string(), analysis))
            },
            "java" | "spring" | "springboot" => {
                if verbose {
                    println!("🔧 Force-running Java/Spring Boot analysis...");
                }
                let analyzer = JavaAnalyzer::new(config);
                let analysis = analyzer.analyze(path)?;
                Ok(("Java/Spring Boot (forced)".to_string(), analysis))
            },
            "python" | "django" | "flask" => {
                if verbose {
                    println!("🔧 Force-running Python analysis...");
                }
                let analyzer = PythonAnalyzer::new(config);
                let analysis = analyzer.analyze(path)?;
                let framework_name = match analysis.project_type {
                    ProjectType::Django => "Python/Django (forced)",
                    ProjectType::Flask => "Python/Flask (forced)",
                    _ => "Python (forced)",
                };
                Ok((framework_name.to_string(), analysis))
            },
            _ => anyhow::bail!("Unsupported analyzer type: '{}'. Supported: typescript, java, python", analyzer_type)
        }
    }
    
    fn generate_summary(&self, analysis: &crate::core::CodebaseAnalysis) -> String {
        let mut summary = String::new();
        
        summary.push_str(&format!("# Codebase Analysis: {}\n\n", analysis.project_name));
        
        // Project Overview
        summary.push_str("## 📋 Project Overview\n");
        summary.push_str(&format!("- **Project Type**: {:?}\n", analysis.project_type));
        summary.push_str(&format!("- **Files Analyzed**: {}\n", analysis.analysis_metadata.files_analyzed));
        summary.push_str(&format!("- **Lines of Code**: {}\n", analysis.analysis_metadata.lines_of_code));
        summary.push_str(&format!("- **Components Found**: {}\n", analysis.components.len()));
        summary.push_str(&format!("- **Confidence Score**: {:.1}%\n\n", analysis.analysis_metadata.confidence_score * 100.0));
        
        // Components Summary
        summary.push_str("## 🧩 Components Analysis\n\n");
        
        let mut component_types = std::collections::HashMap::new();
        let mut total_complexity = 0;
        
        for component in &analysis.components {
            *component_types.entry(format!("{:?}", component.component_type)).or_insert(0) += 1;
            total_complexity += component.complexity_score as u32;
        }
        
        summary.push_str("### Component Types:\n");
        for (comp_type, count) in component_types {
            summary.push_str(&format!("- **{}**: {} components\n", comp_type, count));
        }
        
        let avg_complexity = if !analysis.components.is_empty() {
            total_complexity as f32 / analysis.components.len() as f32
        } else {
            0.0
        };
        summary.push_str(&format!("\n**Average Complexity Score**: {:.1}/100\n\n", avg_complexity));
        
        // Implementation Status
        let mut status_counts = std::collections::HashMap::new();
        for component in &analysis.components {
            *status_counts.entry(format!("{:?}", component.implementation_status)).or_insert(0) += 1;
        }
        
        summary.push_str("### Implementation Status:\n");
        for (status, count) in status_counts {
            summary.push_str(&format!("- **{}**: {} components\n", status, count));
        }
        summary.push_str("\n");
        
        // Top Components by Complexity
        let mut sorted_components = analysis.components.clone();
        sorted_components.sort_by(|a, b| b.complexity_score.cmp(&a.complexity_score));
        
        summary.push_str("### Most Complex Components:\n");
        for (i, component) in sorted_components.iter().take(5).enumerate() {
            summary.push_str(&format!("{}. **{}** ({:?}) - Complexity: {}/100\n", 
                i + 1, component.name, component.component_type, component.complexity_score));
            summary.push_str(&format!("   - Purpose: {}\n", component.purpose));
            if !component.api_calls.is_empty() {
                summary.push_str(&format!("   - API Calls: {}\n", component.api_calls.len()));
            }
        }
        summary.push_str("\n");
        
        // User Stories
        if !analysis.user_stories.is_empty() {
            summary.push_str("## 📖 User Stories\n\n");
            
            let mut priority_counts = std::collections::HashMap::new();
            for story in &analysis.user_stories {
                *priority_counts.entry(format!("{:?}", story.priority)).or_insert(0) += 1;
            }
            
            summary.push_str("### By Priority:\n");
            for (priority, count) in priority_counts {
                summary.push_str(&format!("- **{}**: {} stories\n", priority, count));
            }
            summary.push_str("\n");
            
            // Show high-priority stories
            let high_priority_stories: Vec<_> = analysis.user_stories.iter()
                .filter(|s| matches!(s.priority, crate::core::Priority::Critical | crate::core::Priority::High))
                .collect();
            
            if !high_priority_stories.is_empty() {
                summary.push_str("### High-Priority Stories:\n");
                for story in high_priority_stories.iter().take(5) {
                    summary.push_str(&format!("- **{}**: {}\n", story.id, story.title));
                    summary.push_str(&format!("  - {}\n", story.description));
                }
                summary.push_str("\n");
            }
        }
        
        // Tasks
        if !analysis.tasks.is_empty() {
            summary.push_str("## Task Breakdown\n\n");
            
            let incomplete_tasks: Vec<_> = analysis.tasks.iter()
                .filter(|t| !matches!(t.status, crate::core::ImplementationStatus::Complete))
                .collect();
            
            if !incomplete_tasks.is_empty() {
                summary.push_str(&format!("### Outstanding Tasks: {}\n", incomplete_tasks.len()));
                for task in incomplete_tasks.iter().take(10) {
                    summary.push_str(&format!("- **{}** ({:?}): {}\n", 
                        task.id, task.status, task.name));
                    if let Some(effort) = &task.effort_estimate {
                        summary.push_str(&format!("  - Effort: {}\n", effort));
                    }
                }
                if incomplete_tasks.len() > 10 {
                    summary.push_str(&format!("  ... and {} more tasks\n", incomplete_tasks.len() - 10));
                }
            }
        }
        
        // PRD Summary
        summary.push_str("\n## 📄 Product Requirements Summary\n");
        summary.push_str(&format!("**Title**: {}\n\n", analysis.prd.title));
        summary.push_str(&format!("**Overview**: {}\n\n", analysis.prd.overview));
        
        if !analysis.prd.features.is_empty() {
            summary.push_str("**Key Features**:\n");
            for feature in &analysis.prd.features {
                summary.push_str(&format!("- **{}**: {}\n", feature.name, feature.description));
            }
        }
        
        summary.push_str(&format!("\n---\n*Generated by Codebase Analyzer v{} at {}*\n", 
            analysis.analysis_metadata.analyzer_version,
            analysis.analysis_metadata.analyzed_at));
        
        summary
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
    }
}