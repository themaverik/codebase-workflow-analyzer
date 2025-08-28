use std::path::Path;
use crate::core::enhanced_framework_detector::EnhancedFrameworkDetector;

pub async fn test_ast_integration_with_path(test_path: &str) -> anyhow::Result<()> {
    println!("🧪 Testing AST Integration on: {}", test_path);
    
    let mut detector = EnhancedFrameworkDetector::new(test_path.to_string())?
        .with_ast_analysis()?;
    
    println!("  ✓ AST Analyzer initialized successfully");
    
    // Run enhanced framework detection with AST analysis
    let result = detector.detect_frameworks_enhanced().await?;
    
    println!("  ✓ Enhanced framework detector created");
    
    // Display basic analysis results
    println!("  📊 Analysis Results:");
    println!("    - Language Ecosystem: {}", result.primary_ecosystem);
    println!("    - Detected Frameworks: {}", result.detected_frameworks.len());
    println!("    - Code Segments Analyzed: {}", result.code_segments.len());
    
    if let Some(ast_analysis) = &result.ast_analysis {
        println!("    - Functions Found: {}", ast_analysis.segment_statistics.function_count);
        println!("    - Classes Found: {}", ast_analysis.segment_statistics.class_count);
        println!("    - Interfaces Found: {}", ast_analysis.segment_statistics.interface_count);
        
        if !ast_analysis.business_hints.is_empty() {
            println!("    - Business Hints: {:?}", ast_analysis.business_hints.keys().collect::<Vec<_>>());
        }
    }
    
    // Show framework-specific segments
    if !result.detected_frameworks.is_empty() {
        println!("  🔍 Framework Analysis:");
        for framework in &result.detected_frameworks[..std::cmp::min(3, result.detected_frameworks.len())] {
            println!("    - {}: {:.1}% confidence", framework.framework, framework.confidence * 100.0);
            if let Some(ast_evidence) = &framework.ast_evidence {
                if ast_evidence.relevant_segments > 0 {
                    println!("      - AST Segments: {}", ast_evidence.relevant_segments);
                    if !ast_evidence.framework_specific_patterns.is_empty() {
                        println!("      - Patterns: {:?}", &ast_evidence.framework_specific_patterns[..std::cmp::min(2, ast_evidence.framework_specific_patterns.len())]);
                    }
                }
            }
        }
    }
    
    println!("🎉 AST Integration test passed!");
    
    Ok(())
}

pub async fn test_ast_integration() -> anyhow::Result<()> {
    // Test on current project for backwards compatibility
    let current_dir = std::env::current_dir()?;
    test_ast_integration_with_path(&current_dir.to_string_lossy()).await
}

pub fn run_basic_ast_test() -> anyhow::Result<()> {
    use crate::core::ast_analyzer::ASTAnalyzer;
    
    println!("🔍 Testing basic AST analyzer creation...");
    
    match ASTAnalyzer::new() {
        Ok(_analyzer) => {
            println!("  ✅ AST Analyzer created successfully");
            println!("  ✅ Tree-sitter parsers initialized");
            Ok(())
        }
        Err(e) => {
            println!("  ❌ Failed to create AST Analyzer: {}", e);
            Err(e)
        }
    }
}