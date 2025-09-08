use std::path::PathBuf;
use std::time::SystemTime;
use anyhow::Result;

use crate::core::context_types::{
    SegmentContext, SegmentId, FileContext, FileType, FileRole,
    CrossReferenceMap, ValidationResult, EdgeCaseAnalysisResult,
    ContextQualityMetrics, SegmentType, CrossReferenceType
};
use crate::core::types::{AstSegment, BusinessDomain};
use crate::core::hierarchical_context_manager::HierarchicalContextManager;

/// Integration tests for the enhanced context management system
pub struct ContextIntegrationTests {
    pub test_results: Vec<IntegrationTestResult>,
}

#[derive(Debug, Clone)]
pub struct IntegrationTestResult {
    pub test_name: String,
    pub passed: bool,
    pub execution_time: std::time::Duration,
    pub details: String,
    pub error_message: Option<String>,
}

impl ContextIntegrationTests {
    pub fn new() -> Self {
        Self {
            test_results: Vec::new(),
        }
    }
    
    /// Run all integration tests
    pub async fn run_all_tests(&mut self) -> Result<IntegrationTestSummary> {
        println!("🧪 Starting Context Management Integration Tests...");
        
        // Test 1: Basic context creation and validation
        self.test_basic_context_creation().await?;
        
        // Test 2: Cross-reference validation
        self.test_cross_reference_validation().await?;
        
        // Test 3: Edge case handling
        self.test_edge_case_handling().await?;
        
        // Test 4: Context quality assessment and enhancement
        self.test_context_quality_enhancement().await?;
        
        // Test 5: Performance under load
        self.test_performance_scalability().await?;
        
        // Test 6: End-to-end workflow
        self.test_end_to_end_workflow().await?;
        
        let summary = self.generate_test_summary();
        self.print_test_results(&summary);
        
        Ok(summary)
    }
    
    /// Test 1: Basic context creation and validation
    async fn test_basic_context_creation(&mut self) -> Result<()> {
        let start_time = std::time::Instant::now();
        let mut manager = HierarchicalContextManager::new();
        
        let result = async {
            // Create sample file context
            let file_context = FileContext {
                file_path: PathBuf::from("test/sample.rs"),
                file_type: FileType::SourceCode,
                role_in_project: FileRole::CoreLogic,
                language: Some("Rust".to_string()),
                imports: vec!["std::collections::HashMap".to_string()],
                exports: vec!["SampleStruct".to_string()],
                key_patterns: vec!["struct".to_string(), "impl".to_string()],
                related_files: vec![],
                business_relevance: 0.8,
                last_modified: SystemTime::now(),
            };
            
            // Create sample segment context
            let segment_context = SegmentContext {
                segment_id: "test_segment_1".to_string(),
                segment: AstSegment {
                    file_path: PathBuf::from("test/sample.rs"),
                    start_line: 1,
                    end_line: 10,
                    segment_type: "struct".to_string(),
                    content: "struct SampleStruct { field: String }".to_string(),
                    language: "Rust".to_string(),
                },
                file_context: file_context.clone(),
                segment_type: SegmentType::DataStructure,
                business_purpose: Some("Data modeling for test scenarios".to_string()),
                dependencies: vec!["dependency_1".to_string()],
                dependents: vec!["dependent_1".to_string()],
                confidence: 0.9,
                extracted_at: SystemTime::now(),
            };
            
            // Add contexts to manager
            manager.file_contexts.insert(PathBuf::from("test/sample.rs"), file_context);
            manager.segment_contexts.insert("test_segment_1".to_string(), segment_context);
            
            // Add cross-reference
            manager.cross_references.add_functional_dependency(
                "test_segment_1".to_string(),
                "dependency_1".to_string()
            );
            
            // Validate creation
            if manager.segment_contexts.len() != 1 {
                return Err(anyhow::anyhow!("Expected 1 segment context, found {}", manager.segment_contexts.len()));
            }
            
            if manager.file_contexts.len() != 1 {
                return Err(anyhow::anyhow!("Expected 1 file context, found {}", manager.file_contexts.len()));
            }
            
            // Test cross-reference
            let related = manager.cross_references.get_related_segments(&"test_segment_1".to_string());
            if related.len() != 1 || related[0] != "dependency_1" {
                return Err(anyhow::anyhow!("Cross-reference validation failed"));
            }
            
            Ok(())
        }.await;
        
        let execution_time = start_time.elapsed();
        let (passed, error_message) = match result {
            Ok(()) => (true, None),
            Err(e) => (false, Some(e.to_string())),
        };
        
        self.test_results.push(IntegrationTestResult {
            test_name: "Basic Context Creation".to_string(),
            passed,
            execution_time,
            details: "Tests basic creation of file and segment contexts with cross-references".to_string(),
            error_message,
        });
        
        Ok(())
    }
    
    /// Test 2: Cross-reference validation
    async fn test_cross_reference_validation(&mut self) -> Result<()> {
        let start_time = std::time::Instant::now();
        let mut manager = self.create_test_manager().await?;
        
        let result = async {
            // Add some invalid cross-references
            manager.cross_references.add_functional_dependency(
                "existing_segment".to_string(),
                "non_existent_segment".to_string()
            );
            
            // Run validation
            let validation_result = manager.validate_cross_references().await?;
            
            // Check validation results
            if validation_result.is_valid {
                return Err(anyhow::anyhow!("Expected validation to fail due to missing references"));
            }
            
            if validation_result.missing_references.is_empty() {
                return Err(anyhow::anyhow!("Expected missing references to be detected"));
            }
            
            if validation_result.warnings.is_empty() {
                return Err(anyhow::anyhow!("Expected validation warnings"));
            }
            
            Ok(())
        }.await;
        
        let execution_time = start_time.elapsed();
        let (passed, error_message) = match result {
            Ok(()) => (true, None),
            Err(e) => (false, Some(e.to_string())),
        };
        
        self.test_results.push(IntegrationTestResult {
            test_name: "Cross-reference Validation".to_string(),
            passed,
            execution_time,
            details: "Tests validation of cross-references and detection of missing references".to_string(),
            error_message,
        });
        
        Ok(())
    }
    
    /// Test 3: Edge case handling
    async fn test_edge_case_handling(&mut self) -> Result<()> {
        let start_time = std::time::Instant::now();
        let mut manager = HierarchicalContextManager::new();
        
        let result = async {
            // Test empty project handling
            let temp_dir = std::env::temp_dir().join("empty_test_project");
            std::fs::create_dir_all(&temp_dir)?;
            
            let edge_case_result = manager.handle_edge_cases(&temp_dir).await?;
            
            // Verify empty project detection
            let has_empty_case = edge_case_result.edge_cases_detected
                .iter()
                .any(|case| matches!(case.case_type, crate::core::context_types::EdgeCaseType::EmptyProject));
            
            if !has_empty_case {
                return Err(anyhow::anyhow!("Empty project edge case not detected"));
            }
            
            if edge_case_result.handling_recommendations.is_empty() {
                return Err(anyhow::anyhow!("No handling recommendations provided"));
            }
            
            // Clean up
            std::fs::remove_dir_all(&temp_dir)?;
            
            Ok(())
        }.await;
        
        let execution_time = start_time.elapsed();
        let (passed, error_message) = match result {
            Ok(()) => (true, None),
            Err(e) => (false, Some(e.to_string())),
        };
        
        self.test_results.push(IntegrationTestResult {
            test_name: "Edge Case Handling".to_string(),
            passed,
            execution_time,
            details: "Tests detection and handling of edge cases like empty projects".to_string(),
            error_message,
        });
        
        Ok(())
    }
    
    /// Test 4: Context quality assessment and enhancement
    async fn test_context_quality_enhancement(&mut self) -> Result<()> {
        let start_time = std::time::Instant::now();
        let mut manager = self.create_test_manager().await?;
        
        let result = async {
            // Assess initial quality
            let initial_quality = manager.assess_context_quality().await?;
            
            // Enhance context quality
            let enhancement_result = manager.enhance_context_quality().await?;
            
            // Verify improvements
            if enhancement_result.enhanced_quality.completeness_score < initial_quality.completeness_score {
                return Err(anyhow::anyhow!("Context quality should not decrease after enhancement"));
            }
            
            if enhancement_result.improvements_applied.is_empty() {
                // This might be OK if the context was already high quality
                println!("⚠️  No improvements applied - context may already be optimal");
            }
            
            // Verify quality metrics are reasonable
            let quality = &enhancement_result.enhanced_quality;
            if quality.completeness_score < 0.0 || quality.completeness_score > 1.0 {
                return Err(anyhow::anyhow!("Completeness score out of range: {}", quality.completeness_score));
            }
            
            if quality.consistency_score < 0.0 || quality.consistency_score > 1.0 {
                return Err(anyhow::anyhow!("Consistency score out of range: {}", quality.consistency_score));
            }
            
            Ok(())
        }.await;
        
        let execution_time = start_time.elapsed();
        let (passed, error_message) = match result {
            Ok(()) => (true, None),
            Err(e) => (false, Some(e.to_string())),
        };
        
        self.test_results.push(IntegrationTestResult {
            test_name: "Context Quality Enhancement".to_string(),
            passed,
            execution_time,
            details: "Tests quality assessment and enhancement of context data".to_string(),
            error_message,
        });
        
        Ok(())
    }
    
    /// Test 5: Performance scalability
    async fn test_performance_scalability(&mut self) -> Result<()> {
        let start_time = std::time::Instant::now();
        let mut manager = HierarchicalContextManager::new();
        
        let result = async {
            // Create a larger number of contexts to test performance
            for i in 0..100 {
                let file_context = FileContext {
                    file_path: PathBuf::from(format!("test/file_{}.rs", i)),
                    file_type: FileType::SourceCode,
                    role_in_project: FileRole::CoreLogic,
                    language: Some("Rust".to_string()),
                    imports: vec![],
                    exports: vec![],
                    key_patterns: vec![],
                    related_files: vec![],
                    business_relevance: 0.5,
                    last_modified: SystemTime::now(),
                };
                
                let segment_context = SegmentContext {
                    segment_id: format!("segment_{}", i),
                    segment: AstSegment {
                        file_path: PathBuf::from(format!("test/file_{}.rs", i)),
                        start_line: 1,
                        end_line: 10,
                        segment_type: "function".to_string(),
                        content: format!("fn test_function_{}() {{}}", i),
                        language: "Rust".to_string(),
                    },
                    file_context: file_context.clone(),
                    segment_type: SegmentType::FunctionDefinition,
                    business_purpose: None,
                    dependencies: vec![],
                    dependents: vec![],
                    confidence: 0.7,
                    extracted_at: SystemTime::now(),
                };
                
                manager.file_contexts.insert(PathBuf::from(format!("test/file_{}.rs", i)), file_context);
                manager.segment_contexts.insert(format!("segment_{}", i), segment_context);
            }
            
            // Test performance of quality assessment
            let quality_start = std::time::Instant::now();
            let _quality = manager.assess_context_quality().await?;
            let quality_time = quality_start.elapsed();
            
            // Performance should be reasonable (< 1 second for 100 items)
            if quality_time.as_secs() > 1 {
                return Err(anyhow::anyhow!("Quality assessment too slow: {:?}", quality_time));
            }
            
            // Test performance of validation
            let validation_start = std::time::Instant::now();
            let _validation = manager.validate_cross_references().await?;
            let validation_time = validation_start.elapsed();
            
            if validation_time.as_secs() > 1 {
                return Err(anyhow::anyhow!("Validation too slow: {:?}", validation_time));
            }
            
            Ok(())
        }.await;
        
        let execution_time = start_time.elapsed();
        let (passed, error_message) = match result {
            Ok(()) => (true, None),
            Err(e) => (false, Some(e.to_string())),
        };
        
        self.test_results.push(IntegrationTestResult {
            test_name: "Performance Scalability".to_string(),
            passed,
            execution_time,
            details: "Tests performance with larger numbers of contexts (100 segments)".to_string(),
            error_message,
        });
        
        Ok(())
    }
    
    /// Test 6: End-to-end workflow
    async fn test_end_to_end_workflow(&mut self) -> Result<()> {
        let start_time = std::time::Instant::now();
        let mut manager = self.create_test_manager().await?;
        
        let result = async {
            // Simulate complete workflow
            
            // 1. Validate contexts
            let validation = manager.validate_cross_references().await?;
            println!("✓ Validation completed: {} warnings", validation.warnings.len());
            
            // 2. Assess quality
            let initial_quality = manager.assess_context_quality().await?;
            println!("✓ Initial quality assessed: completeness={:.2}", initial_quality.completeness_score);
            
            // 3. Enhance quality
            let enhancement = manager.enhance_context_quality().await?;
            println!("✓ Enhancement completed: {} improvements", enhancement.improvements_applied.len());
            
            // 4. Handle edge cases (with temp directory)
            let temp_dir = std::env::temp_dir().join("workflow_test_project");
            std::fs::create_dir_all(&temp_dir)?;
            let edge_cases = manager.handle_edge_cases(&temp_dir).await?;
            std::fs::remove_dir_all(&temp_dir)?;
            println!("✓ Edge cases handled: {} cases detected", edge_cases.edge_cases_detected.len());
            
            // 5. Final quality check
            let final_quality = manager.assess_context_quality().await?;
            println!("✓ Final quality: completeness={:.2}", final_quality.completeness_score);
            
            // Verify workflow completed successfully
            if final_quality.completeness_score < 0.0 {
                return Err(anyhow::anyhow!("Invalid final quality score"));
            }
            
            Ok(())
        }.await;
        
        let execution_time = start_time.elapsed();
        let (passed, error_message) = match result {
            Ok(()) => (true, None),
            Err(e) => (false, Some(e.to_string())),
        };
        
        self.test_results.push(IntegrationTestResult {
            test_name: "End-to-End Workflow".to_string(),
            passed,
            execution_time,
            details: "Tests complete workflow: validation -> quality assessment -> enhancement -> edge cases".to_string(),
            error_message,
        });
        
        Ok(())
    }
    
    /// Helper: Create a test manager with sample data
    async fn create_test_manager(&self) -> Result<HierarchicalContextManager> {
        let mut manager = HierarchicalContextManager::new();
        
        // Add sample contexts for testing
        let file_context = FileContext {
            file_path: PathBuf::from("test/existing.rs"),
            file_type: FileType::SourceCode,
            role_in_project: FileRole::CoreLogic,
            language: Some("Rust".to_string()),
            imports: vec![],
            exports: vec![],
            key_patterns: vec![],
            related_files: vec![],
            business_relevance: 0.8,
            last_modified: SystemTime::now(),
        };
        
        let segment_context = SegmentContext {
            segment_id: "existing_segment".to_string(),
            segment: AstSegment {
                file_path: PathBuf::from("test/existing.rs"),
                start_line: 1,
                end_line: 10,
                segment_type: "function".to_string(),
                content: "fn existing_function() {}".to_string(),
                language: "Rust".to_string(),
            },
            file_context: file_context.clone(),
            segment_type: SegmentType::FunctionDefinition,
            business_purpose: Some("Test function".to_string()),
            dependencies: vec![],
            dependents: vec![],
            confidence: 0.8,
            extracted_at: SystemTime::now(),
        };
        
        manager.file_contexts.insert(PathBuf::from("test/existing.rs"), file_context);
        manager.segment_contexts.insert("existing_segment".to_string(), segment_context);
        
        Ok(manager)
    }
    
    /// Generate test summary
    fn generate_test_summary(&self) -> IntegrationTestSummary {
        let total_tests = self.test_results.len();
        let passed_tests = self.test_results.iter().filter(|r| r.passed).count();
        let failed_tests = total_tests - passed_tests;
        
        let total_execution_time = self.test_results
            .iter()
            .map(|r| r.execution_time)
            .sum();
        
        let success_rate = if total_tests > 0 {
            (passed_tests as f32 / total_tests as f32) * 100.0
        } else {
            0.0
        };
        
        IntegrationTestSummary {
            total_tests,
            passed_tests,
            failed_tests,
            success_rate,
            total_execution_time,
        }
    }
    
    /// Print test results
    fn print_test_results(&self, summary: &IntegrationTestSummary) {
        println!("\n🎯 Context Management Integration Test Results");
        println!("==============================================");
        println!("Total Tests: {}", summary.total_tests);
        println!("Passed: {}", summary.passed_tests);
        println!("Failed: {}", summary.failed_tests);
        println!("Success Rate: {:.1}%", summary.success_rate);
        println!("Total Execution Time: {:?}", summary.total_execution_time);
        
        println!("\nDetailed Results:");
        for result in &self.test_results {
            let status = if result.passed { "✅ PASS" } else { "❌ FAIL" };
            println!("  {} - {} ({:?})", status, result.test_name, result.execution_time);
            if let Some(error) = &result.error_message {
                println!("    Error: {}", error);
            }
        }
        
        if summary.success_rate >= 80.0 {
            println!("\n🎉 Integration tests completed successfully!");
        } else {
            println!("\n⚠️  Some integration tests failed. Review errors above.");
        }
    }
}

#[derive(Debug, Clone)]
pub struct IntegrationTestSummary {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub success_rate: f32,
    pub total_execution_time: std::time::Duration,
}

/// Run context management integration tests
pub async fn run_context_integration_tests() -> Result<IntegrationTestSummary> {
    let mut test_suite = ContextIntegrationTests::new();
    test_suite.run_all_tests().await
}