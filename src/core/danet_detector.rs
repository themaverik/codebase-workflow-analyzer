use std::path::Path;
use std::fs;
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::core::config::get_config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DanetAnalysisResult {
    pub is_danet_project: bool,
    pub confidence: f32,
    pub evidence: Vec<DanetEvidence>,
    pub deno_features: DenoFeatures,
    pub danet_patterns: Vec<DanetPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DanetEvidence {
    pub evidence_type: DanetEvidenceType,
    pub source: String,
    pub description: String,
    pub confidence_contribution: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DanetEvidenceType {
    DenoConfig,          // deno.json/deno.jsonc presence
    DanetImport,         // Direct danet imports
    DenoUrlImport,       // https://deno.land/x/danet imports
    NativeTypeScript,    // .ts files without compilation config
    DenoBuiltinTools,    // deno fmt, deno lint, etc.
    DenoPermissions,     // Deno permission flags
    ModuleStructure,     // mod.ts instead of index.ts
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DenoFeatures {
    pub native_typescript: bool,
    pub builtin_tooling: Vec<String>,
    pub import_system: ImportSystemType,
    pub permissions_model: Option<String>,
    pub std_library_usage: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImportSystemType {
    EsModulesWithUrls,
    ImportMap,
    Mixed,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DanetPattern {
    pub pattern_type: DanetPatternType,
    pub file_path: String,
    pub description: String,
    pub nestjs_equivalent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DanetPatternType {
    Controller,
    Service,
    Module,
    Middleware,
    Guard,
    Injectable,
    Decorator,
}

pub struct DanetDetector;

impl DanetDetector {
    pub fn new() -> Self {
        Self
    }

    /// Intelligently analyze if this is a Danet project vs NestJS
    pub async fn analyze_danet_project(&self, project_path: &Path) -> Result<DanetAnalysisResult> {
        let mut evidence = Vec::new();
        let mut confidence = 0.0;

        // 1. Check for Deno configuration files (strongest indicator)
        let deno_features = self.analyze_deno_configuration(project_path, &mut evidence, &mut confidence).await?;

        // 2. Analyze import patterns for Danet vs NestJS
        self.analyze_import_patterns(project_path, &mut evidence, &mut confidence).await?;

        // 3. Check for Deno-specific tooling usage
        self.analyze_deno_tooling(project_path, &mut evidence, &mut confidence).await?;

        // 4. Analyze TypeScript usage patterns (Deno vs Node.js)
        self.analyze_typescript_patterns(project_path, &mut evidence, &mut confidence).await?;

        // 5. Detect Danet-specific patterns
        let danet_patterns = self.detect_danet_patterns(project_path).await?;

        let is_danet_project = confidence > 0.7;

        Ok(DanetAnalysisResult {
            is_danet_project,
            confidence,
            evidence,
            deno_features,
            danet_patterns,
        })
    }

    async fn analyze_deno_configuration(&self, 
        project_path: &Path, 
        evidence: &mut Vec<DanetEvidence>,
        confidence: &mut f32
    ) -> Result<DenoFeatures> {
        let mut deno_features = DenoFeatures {
            native_typescript: false,
            builtin_tooling: Vec::new(),
            import_system: ImportSystemType::Unknown,
            permissions_model: None,
            std_library_usage: Vec::new(),
        };

        // Check for deno.json or deno.jsonc
        let deno_json_path = project_path.join("deno.json");
        let deno_jsonc_path = project_path.join("deno.jsonc");

        if deno_json_path.exists() || deno_jsonc_path.exists() {
            *confidence += 0.4;
            evidence.push(DanetEvidence {
                evidence_type: DanetEvidenceType::DenoConfig,
                source: if deno_json_path.exists() { "deno.json" } else { "deno.jsonc" }.to_string(),
                description: "Deno configuration file present - strong indicator of Deno project".to_string(),
                confidence_contribution: 0.4,
            });

            // Parse Deno configuration
            let config_path = if deno_json_path.exists() { deno_json_path } else { deno_jsonc_path };
            if let Ok(config_content) = fs::read_to_string(&config_path) {
                self.parse_deno_config(&config_content, &mut deno_features, evidence, confidence)?;
            }
        }

        // Check for absence of Node.js specific files
        if !project_path.join("package.json").exists() {
            *confidence += 0.1;
            evidence.push(DanetEvidence {
                evidence_type: DanetEvidenceType::DenoConfig,
                source: "filesystem".to_string(),
                description: "No package.json found - consistent with Deno project".to_string(),
                confidence_contribution: 0.1,
            });
        }

        // Check for TypeScript without tsconfig.json
        if self.has_typescript_files(project_path) && !project_path.join("tsconfig.json").exists() {
            *confidence += 0.1;
            deno_features.native_typescript = true;
            evidence.push(DanetEvidence {
                evidence_type: DanetEvidenceType::NativeTypeScript,
                source: "filesystem".to_string(),
                description: "TypeScript files without tsconfig.json - Deno native TypeScript".to_string(),
                confidence_contribution: 0.1,
            });
        }

        Ok(deno_features)
    }

    fn parse_deno_config(&self,
        config_content: &str,
        deno_features: &mut DenoFeatures,
        evidence: &mut Vec<DanetEvidence>,
        confidence: &mut f32
    ) -> Result<()> {
        // Basic JSON parsing (could be enhanced to handle JSONC properly)
        if let Ok(config) = serde_json::from_str::<serde_json::Value>(config_content) {
            // Check for import maps
            if config.get("imports").is_some() {
                deno_features.import_system = ImportSystemType::ImportMap;
                *confidence += 0.05;
                evidence.push(DanetEvidence {
                    evidence_type: DanetEvidenceType::DenoConfig,
                    source: "deno.json imports".to_string(),
                    description: "Import map configuration found".to_string(),
                    confidence_contribution: 0.05,
                });
            }

            // Check for Deno tasks
            if let Some(tasks) = config.get("tasks").and_then(|t| t.as_object()) {
                for task_name in tasks.keys() {
                    if task_name.starts_with("deno ") {
                        deno_features.builtin_tooling.push(task_name.clone());
                    }
                }
                
                if !deno_features.builtin_tooling.is_empty() {
                    *confidence += 0.05;
                    evidence.push(DanetEvidence {
                        evidence_type: DanetEvidenceType::DenoBuiltinTools,
                        source: "deno.json tasks".to_string(),
                        description: format!("Deno builtin tools configured: {}", deno_features.builtin_tooling.join(", ")),
                        confidence_contribution: 0.05,
                    });
                }
            }

            // Check for permissions configuration
            if let Some(permissions) = config.get("permissions") {
                deno_features.permissions_model = Some(permissions.to_string());
                *confidence += 0.05;
                evidence.push(DanetEvidence {
                    evidence_type: DanetEvidenceType::DenoPermissions,
                    source: "deno.json permissions".to_string(),
                    description: "Deno permissions model configured".to_string(),
                    confidence_contribution: 0.05,
                });
            }
        }

        Ok(())
    }

    async fn analyze_import_patterns(&self,
        project_path: &Path,
        evidence: &mut Vec<DanetEvidence>,
        confidence: &mut f32
    ) -> Result<()> {
        use walkdir::WalkDir;

        let mut danet_imports = 0;
        let mut nestjs_imports = 0;
        let mut deno_url_imports = 0;

        for entry in WalkDir::new(project_path).max_depth(3) {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }

            let file_path = entry.path();
            if let Some(extension) = file_path.extension() {
                if extension == "ts" || extension == "js" {
                    if let Ok(content) = fs::read_to_string(file_path) {
                        // Look for Danet-specific imports
                        if content.contains("from \"danet\"") || content.contains("from 'danet'") {
                            danet_imports += 1;
                        }

                        // Look for Deno URL imports
                        if content.contains("from \"https://deno.land/x/danet") {
                            deno_url_imports += 1;
                            *confidence += 0.1;
                            evidence.push(DanetEvidence {
                                evidence_type: DanetEvidenceType::DenoUrlImport,
                                source: file_path.to_string_lossy().to_string(),
                                description: "Direct Deno URL import for Danet found".to_string(),
                                confidence_contribution: 0.1,
                            });
                        }

                        // Look for NestJS imports (negative indicator)
                        if content.contains("from \"@nestjs/") {
                            nestjs_imports += 1;
                        }
                    }
                }
            }
        }

        // Score based on import patterns
        if danet_imports > 0 {
            *confidence += 0.2;
            evidence.push(DanetEvidence {
                evidence_type: DanetEvidenceType::DanetImport,
                source: "source files".to_string(),
                description: format!("Found {} Danet import statements", danet_imports),
                confidence_contribution: 0.2,
            });
        }

        if deno_url_imports > 0 {
            *confidence += 0.15;
            evidence.push(DanetEvidence {
                evidence_type: DanetEvidenceType::DenoUrlImport,
                source: "source files".to_string(),
                description: format!("Found {} Deno URL imports", deno_url_imports),
                confidence_contribution: 0.15,
            });
        }

        // Penalize for NestJS imports
        if nestjs_imports > 0 && danet_imports == 0 {
            *confidence -= 0.3;
            evidence.push(DanetEvidence {
                evidence_type: DanetEvidenceType::DanetImport,
                source: "source files".to_string(),
                description: format!("Found {} NestJS imports, likely not Danet", nestjs_imports),
                confidence_contribution: -0.3,
            });
        }

        Ok(())
    }

    async fn analyze_deno_tooling(&self,
        project_path: &Path,
        evidence: &mut Vec<DanetEvidence>,
        confidence: &mut f32
    ) -> Result<()> {
        // Check for deno-specific scripts or commands
        let scripts_to_check = [
            ("scripts/dev.sh", "deno run"),
            ("Makefile", "deno "),
            (".github/workflows", "deno-setup"),
            ("docker", "denoland/deno"),
        ];

        for (file_pattern, search_pattern) in &scripts_to_check {
            let search_path = project_path.join(file_pattern);
            if search_path.exists() {
                if search_path.is_file() {
                    if let Ok(content) = fs::read_to_string(&search_path) {
                        if content.contains(search_pattern) {
                            *confidence += 0.05;
                            evidence.push(DanetEvidence {
                                evidence_type: DanetEvidenceType::DenoBuiltinTools,
                                source: search_path.to_string_lossy().to_string(),
                                description: format!("Deno tooling usage found: {}", search_pattern),
                                confidence_contribution: 0.05,
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn analyze_typescript_patterns(&self,
        project_path: &Path,
        evidence: &mut Vec<DanetEvidence>,
        confidence: &mut f32
    ) -> Result<()> {
        // Check for mod.ts (Deno convention) vs index.ts (Node.js convention)
        if project_path.join("mod.ts").exists() {
            *confidence += 0.1;
            evidence.push(DanetEvidence {
                evidence_type: DanetEvidenceType::ModuleStructure,
                source: "mod.ts".to_string(),
                description: "Deno convention: mod.ts entry point found".to_string(),
                confidence_contribution: 0.1,
            });
        }

        // Check for src/main.ts with Deno-style imports
        let main_ts = project_path.join("src/main.ts");
        if main_ts.exists() {
            if let Ok(content) = fs::read_to_string(&main_ts) {
                if content.contains("Deno.serve") || content.contains("deno.land") {
                    *confidence += 0.1;
                    evidence.push(DanetEvidence {
                        evidence_type: DanetEvidenceType::NativeTypeScript,
                        source: "src/main.ts".to_string(),
                        description: "Deno-specific code patterns found in main.ts".to_string(),
                        confidence_contribution: 0.1,
                    });
                }
            }
        }

        Ok(())
    }

    async fn detect_danet_patterns(&self, project_path: &Path) -> Result<Vec<DanetPattern>> {
        let mut patterns = Vec::new();
        use walkdir::WalkDir;

        for entry in WalkDir::new(project_path).max_depth(3) {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }

            let file_path = entry.path();
            if let Some(extension) = file_path.extension() {
                if extension == "ts" {
                    if let Ok(content) = fs::read_to_string(file_path) {
                        // Detect Danet/NestJS-style decorators
                        if content.contains("@Controller") {
                            patterns.push(DanetPattern {
                                pattern_type: DanetPatternType::Controller,
                                file_path: file_path.to_string_lossy().to_string(),
                                description: "Controller decorator found".to_string(),
                                nestjs_equivalent: Some("@Controller from @nestjs/common".to_string()),
                            });
                        }

                        if content.contains("@Injectable") {
                            patterns.push(DanetPattern {
                                pattern_type: DanetPatternType::Injectable,
                                file_path: file_path.to_string_lossy().to_string(),
                                description: "Injectable decorator found".to_string(),
                                nestjs_equivalent: Some("@Injectable from @nestjs/common".to_string()),
                            });
                        }

                        if content.contains("@Module") {
                            patterns.push(DanetPattern {
                                pattern_type: DanetPatternType::Module,
                                file_path: file_path.to_string_lossy().to_string(),
                                description: "Module decorator found".to_string(),
                                nestjs_equivalent: Some("@Module from @nestjs/common".to_string()),
                            });
                        }
                    }
                }
            }
        }

        Ok(patterns)
    }

    fn has_typescript_files(&self, project_path: &Path) -> bool {
        use walkdir::WalkDir;

        for entry in WalkDir::new(project_path).max_depth(2) {
            if let Ok(entry) = entry {
                if let Some(extension) = entry.path().extension() {
                    if extension == "ts" && !entry.path().to_string_lossy().contains("node_modules") {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Compare with NestJS patterns to provide differential analysis
    pub fn generate_nestjs_comparison(&self, analysis: &DanetAnalysisResult) -> Vec<String> {
        let mut comparisons = Vec::new();

        if analysis.is_danet_project {
            comparisons.push("✓ Uses Deno runtime instead of Node.js".to_string());
            comparisons.push("✓ Native TypeScript support without compilation step".to_string());
            comparisons.push("✓ ES modules with URL imports instead of npm packages".to_string());
            comparisons.push("✓ Built-in tooling (deno fmt, deno lint) instead of separate packages".to_string());
            comparisons.push("✓ Deno permission model for enhanced security".to_string());
            
            if analysis.deno_features.import_system == ImportSystemType::ImportMap {
                comparisons.push("✓ Import maps for dependency management".to_string());
            }
        } else {
            comparisons.push("× Standard Node.js/NestJS project structure".to_string());
            comparisons.push("× Requires TypeScript compilation step".to_string());
            comparisons.push("× Uses npm/yarn package management".to_string());
        }

        comparisons
    }
}

impl Default for DanetDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_danet_detection_with_deno_config() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create deno.json with Danet configuration
        let deno_json = r#"
        {
            "imports": {
                "danet": "https://deno.land/x/danet@1.0.0/mod.ts"
            },
            "tasks": {
                "start": "deno run --allow-net src/main.ts",
                "dev": "deno run --watch --allow-net src/main.ts"
            }
        }
        "#;
        fs::write(temp_dir.path().join("deno.json"), deno_json).unwrap();
        
        // Create a TypeScript file with Danet imports
        fs::create_dir(temp_dir.path().join("src")).unwrap();
        fs::write(temp_dir.path().join("src/main.ts"), r#"
import { DanetApplication } from "danet";
import { Controller, Get } from "danet";

@Controller()
export class AppController {
    @Get("/")
    hello() {
        return "Hello Danet!";
    }
}
        "#).unwrap();
        
        let detector = DanetDetector::new();
        let result = detector.analyze_danet_project(temp_dir.path()).await.unwrap();
        
        assert!(result.is_danet_project);
        assert!(result.confidence > 0.7);
        assert!(result.deno_features.import_system == ImportSystemType::ImportMap);
        assert!(!result.danet_patterns.is_empty());
    }

    #[tokio::test]
    async fn test_nestjs_vs_danet_distinction() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create package.json (Node.js indicator)
        let package_json = r#"
        {
            "name": "nestjs-app",
            "dependencies": {
                "@nestjs/core": "^9.0.0",
                "@nestjs/common": "^9.0.0"
            }
        }
        "#;
        fs::write(temp_dir.path().join("package.json"), package_json).unwrap();
        
        // Create TypeScript file with NestJS imports
        fs::create_dir(temp_dir.path().join("src")).unwrap();
        fs::write(temp_dir.path().join("src/app.controller.ts"), r#"
import { Controller, Get } from '@nestjs/common';

@Controller()
export class AppController {
    @Get()
    getHello(): string {
        return 'Hello NestJS!';
    }
}
        "#).unwrap();
        
        let detector = DanetDetector::new();
        let result = detector.analyze_danet_project(temp_dir.path()).await.unwrap();
        
        assert!(!result.is_danet_project);
        assert!(result.confidence < 0.3);
        
        let comparison = detector.generate_nestjs_comparison(&result);
        assert!(comparison.iter().any(|c| c.contains("Node.js/NestJS")));
    }

    #[tokio::test]
    async fn test_native_typescript_detection() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create mod.ts (Deno convention)
        fs::write(temp_dir.path().join("mod.ts"), r#"
export * from "./src/app.ts";
        "#).unwrap();
        
        // Create TypeScript file without tsconfig.json
        fs::create_dir(temp_dir.path().join("src")).unwrap();
        fs::write(temp_dir.path().join("src/app.ts"), r#"
import { serve } from "https://deno.land/std/http/server.ts";

console.log("Deno TypeScript without compilation!");
        "#).unwrap();
        
        let detector = DanetDetector::new();
        let result = detector.analyze_danet_project(temp_dir.path()).await.unwrap();
        
        assert!(result.deno_features.native_typescript);
        assert!(result.confidence > 0.2);
    }
}