use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::Result;

use crate::core::framework_detector::{FrameworkDetector, EnhancedDetectedFramework};
use crate::core::types::{Framework, LanguageEcosystem};

/// Framework validation and benchmarking system
pub struct FrameworkValidationSystem {
    test_cases: Vec<ValidationTestCase>,
}

/// Test case for framework validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationTestCase {
    pub name: String,
    pub project_path: String,
    pub expected_frameworks: Vec<ExpectedFramework>,
    pub expected_ecosystem: LanguageEcosystem,
    pub test_type: TestType,
}

/// Expected framework detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedFramework {
    pub framework: Framework,
    pub min_confidence: f32,
    pub should_detect: bool,
}

/// Type of validation test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestType {
    PositiveDetection,  // Should detect specific frameworks
    NegativeDetection,  // Should NOT detect specific frameworks
    Disambiguation,     // Should distinguish between similar frameworks
    VersionAccuracy,    // Should extract correct version information
}

/// Results of validation testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResults {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub overall_accuracy: f32,
    pub per_framework_accuracy: HashMap<Framework, FrameworkAccuracyStats>,
    pub test_details: Vec<TestResult>,
}

/// Per-framework accuracy statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkAccuracyStats {
    pub true_positives: usize,
    pub false_positives: usize,
    pub true_negatives: usize,
    pub false_negatives: usize,
    pub precision: f32,
    pub recall: f32,
    pub f1_score: f32,
}

/// Individual test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_name: String,
    pub passed: bool,
    pub detected_frameworks: Vec<EnhancedDetectedFramework>,
    pub expected_frameworks: Vec<ExpectedFramework>,
    pub issues: Vec<String>,
    pub confidence_scores: HashMap<Framework, f32>,
}

impl FrameworkValidationSystem {
    pub fn new() -> Self {
        Self {
            test_cases: Self::create_default_test_cases(),
        }
    }
    
    /// Create default test cases for common framework scenarios
    fn create_default_test_cases() -> Vec<ValidationTestCase> {
        vec![
            // React project test
            ValidationTestCase {
                name: "React TypeScript Project".to_string(),
                project_path: "test_data/react_typescript".to_string(),
                expected_frameworks: vec![
                    ExpectedFramework {
                        framework: Framework::React,
                        min_confidence: 0.8,
                        should_detect: true,
                    }
                ],
                expected_ecosystem: LanguageEcosystem::TypeScript,
                test_type: TestType::PositiveDetection,
            },
            
            // Flask project test
            ValidationTestCase {
                name: "Flask Python Project".to_string(),
                project_path: "test_data/flask_app".to_string(),
                expected_frameworks: vec![
                    ExpectedFramework {
                        framework: Framework::Flask,
                        min_confidence: 0.7,
                        should_detect: true,
                    }
                ],
                expected_ecosystem: LanguageEcosystem::Python,
                test_type: TestType::PositiveDetection,
            },
            
            // Spring Boot project test
            ValidationTestCase {
                name: "Spring Boot Java Project".to_string(),
                project_path: "test_data/spring_boot_app".to_string(),
                expected_frameworks: vec![
                    ExpectedFramework {
                        framework: Framework::SpringBoot,
                        min_confidence: 0.8,
                        should_detect: true,
                    }
                ],
                expected_ecosystem: LanguageEcosystem::Java,
                test_type: TestType::PositiveDetection,
            },
            
            // Danet vs NestJS disambiguation
            ValidationTestCase {
                name: "Danet Deno Project (not NestJS)".to_string(),
                project_path: "test_data/danet_app".to_string(),
                expected_frameworks: vec![
                    ExpectedFramework {
                        framework: Framework::Danet,
                        min_confidence: 0.7,
                        should_detect: true,
                    },
                    ExpectedFramework {
                        framework: Framework::NestJS,
                        min_confidence: 0.0,
                        should_detect: false,
                    }
                ],
                expected_ecosystem: LanguageEcosystem::Deno,
                test_type: TestType::Disambiguation,
            },
            
            // Next.js vs React disambiguation
            ValidationTestCase {
                name: "Next.js Project (not plain React)".to_string(),
                project_path: "test_data/nextjs_app".to_string(),
                expected_frameworks: vec![
                    ExpectedFramework {
                        framework: Framework::NextJS,
                        min_confidence: 0.8,
                        should_detect: true,
                    },
                    ExpectedFramework {
                        framework: Framework::React,
                        min_confidence: 0.0,
                        should_detect: false, // Should detect Next.js, not plain React
                    }
                ],
                expected_ecosystem: LanguageEcosystem::TypeScript,
                test_type: TestType::Disambiguation,
            },
        ]
    }
    
    /// Add a custom test case
    pub fn add_test_case(&mut self, test_case: ValidationTestCase) {
        self.test_cases.push(test_case);
    }
    
    /// Run all validation tests
    pub async fn run_validation_tests(&self) -> Result<ValidationResults> {
        let mut test_results = Vec::new();
        let mut per_framework_stats: HashMap<Framework, FrameworkAccuracyStats> = HashMap::new();
        
        println!("🧪 Running {} framework validation tests...", self.test_cases.len());
        
        for test_case in &self.test_cases {
            println!("  Running test: {}", test_case.name);
            let test_result = self.run_single_test(test_case).await?;
            
            // Update per-framework statistics
            self.update_framework_stats(&mut per_framework_stats, test_case, &test_result);
            
            test_results.push(test_result);
        }
        
        // Calculate overall statistics
        let passed_tests = test_results.iter().filter(|r| r.passed).count();
        let total_tests = test_results.len();
        let overall_accuracy = if total_tests > 0 {
            passed_tests as f32 / total_tests as f32
        } else {
            0.0
        };
        
        // Finalize per-framework statistics
        for stats in per_framework_stats.values_mut() {
            self.calculate_framework_metrics(stats);
        }
        
        Ok(ValidationResults {
            total_tests,
            passed_tests,
            failed_tests: total_tests - passed_tests,
            overall_accuracy,
            per_framework_accuracy: per_framework_stats,
            test_details: test_results,
        })
    }
    
    /// Run a single validation test
    async fn run_single_test(&self, test_case: &ValidationTestCase) -> Result<TestResult> {
        // Create framework detector for the test project
        let detector = FrameworkDetector::new(test_case.project_path.clone());
        
        // Detect frameworks
        let detection_result = detector.detect_frameworks()
            .map_err(|e| anyhow::anyhow!("Framework detection failed: {}", e))?;
        
        // Validate results
        let mut issues = Vec::new();
        let mut passed = true;
        let mut confidence_scores = HashMap::new();
        
        // Check ecosystem detection
        if detection_result.primary_ecosystem != test_case.expected_ecosystem {
            issues.push(format!(
                "Expected ecosystem: {:?}, detected: {:?}",
                test_case.expected_ecosystem,
                detection_result.primary_ecosystem
            ));
            passed = false;
        }
        
        // Check framework detection
        for expected in &test_case.expected_frameworks {
            let detected_framework = detection_result.detected_frameworks
                .iter()
                .find(|f| f.framework == expected.framework);
            
            confidence_scores.insert(
                expected.framework, 
                detected_framework.map(|f| f.confidence).unwrap_or(0.0)
            );
            
            match (expected.should_detect, detected_framework) {
                (true, Some(detected)) => {
                    // Should detect and did detect
                    if detected.confidence < expected.min_confidence {
                        issues.push(format!(
                            "Framework {:?} detected with confidence {:.2} < required {:.2}",
                            expected.framework, detected.confidence, expected.min_confidence
                        ));
                        passed = false;
                    }
                },
                (true, None) => {
                    // Should detect but didn't
                    issues.push(format!(
                        "Framework {:?} should be detected but wasn't found",
                        expected.framework
                    ));
                    passed = false;
                },
                (false, Some(detected)) => {
                    // Shouldn't detect but did
                    issues.push(format!(
                        "Framework {:?} shouldn't be detected but was found with confidence {:.2}",
                        expected.framework, detected.confidence
                    ));
                    passed = false;
                },
                (false, None) => {
                    // Shouldn't detect and didn't - correct
                }
            }
        }
        
        Ok(TestResult {
            test_name: test_case.name.clone(),
            passed,
            detected_frameworks: detection_result.detected_frameworks,
            expected_frameworks: test_case.expected_frameworks.clone(),
            issues,
            confidence_scores,
        })
    }
    
    /// Update per-framework accuracy statistics
    fn update_framework_stats(
        &self,
        stats: &mut HashMap<Framework, FrameworkAccuracyStats>,
        test_case: &ValidationTestCase,
        test_result: &TestResult,
    ) {
        for expected in &test_case.expected_frameworks {
            let framework_stats = stats.entry(expected.framework)
                .or_insert_with(|| FrameworkAccuracyStats {
                    true_positives: 0,
                    false_positives: 0,
                    true_negatives: 0,
                    false_negatives: 0,
                    precision: 0.0,
                    recall: 0.0,
                    f1_score: 0.0,
                });
            
            let was_detected = test_result.detected_frameworks
                .iter()
                .any(|f| f.framework == expected.framework);
            
            match (expected.should_detect, was_detected) {
                (true, true) => framework_stats.true_positives += 1,
                (true, false) => framework_stats.false_negatives += 1,
                (false, true) => framework_stats.false_positives += 1,
                (false, false) => framework_stats.true_negatives += 1,
            }
        }
    }
    
    /// Calculate precision, recall, and F1 score for framework statistics
    fn calculate_framework_metrics(&self, stats: &mut FrameworkAccuracyStats) {
        let tp = stats.true_positives as f32;
        let fp = stats.false_positives as f32;
        let fn_ = stats.false_negatives as f32;
        
        // Precision = TP / (TP + FP)
        stats.precision = if tp + fp > 0.0 {
            tp / (tp + fp)
        } else {
            1.0 // Perfect precision if no false positives and no detections
        };
        
        // Recall = TP / (TP + FN)
        stats.recall = if tp + fn_ > 0.0 {
            tp / (tp + fn_)
        } else {
            1.0 // Perfect recall if no false negatives and no expected detections
        };
        
        // F1 Score = 2 * (Precision * Recall) / (Precision + Recall)
        stats.f1_score = if stats.precision + stats.recall > 0.0 {
            2.0 * (stats.precision * stats.recall) / (stats.precision + stats.recall)
        } else {
            0.0
        };
    }
    
    /// Print detailed validation results
    pub fn print_validation_results(&self, results: &ValidationResults) {
        println!("\n🎯 Framework Detection Validation Results");
        println!("==========================================");
        println!("Overall Accuracy: {:.1}% ({}/{})", 
            results.overall_accuracy * 100.0, 
            results.passed_tests, 
            results.total_tests
        );
        
        println!("\nPer-Framework Accuracy:");
        for (framework, stats) in &results.per_framework_accuracy {
            println!("  {:?}:", framework);
            println!("    Precision: {:.3}", stats.precision);
            println!("    Recall: {:.3}", stats.recall);
            println!("    F1 Score: {:.3}", stats.f1_score);
            println!("    TP: {}, FP: {}, TN: {}, FN: {}", 
                stats.true_positives, stats.false_positives, 
                stats.true_negatives, stats.false_negatives);
        }
        
        println!("\nTest Details:");
        for test_result in &results.test_details {
            let status = if test_result.passed { "✅ PASS" } else { "❌ FAIL" };
            println!("  {} - {}", status, test_result.test_name);
            
            if !test_result.passed {
                for issue in &test_result.issues {
                    println!("    Issue: {}", issue);
                }
            }
            
            if !test_result.detected_frameworks.is_empty() {
                println!("    Detected: {:?}", 
                    test_result.detected_frameworks.iter()
                        .map(|f| format!("{:?} ({:.2})", f.framework, f.confidence))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }
    }
}

/// Run framework validation tests
pub async fn run_framework_validation() -> Result<ValidationResults> {
    let validation_system = FrameworkValidationSystem::new();
    validation_system.run_validation_tests().await
}