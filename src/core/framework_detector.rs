use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::core::{DetectedFramework, UsageExtent};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LanguageEcosystem {
    Python,      // .py files, requirements.txt, pyproject.toml
    JavaScript,  // .js files, package.json (no TypeScript)
    TypeScript,  // .ts/.tsx files, tsconfig.json, package.json
    Java,        // .java files, pom.xml, build.gradle
    Deno,        // .ts files with URL imports, deno.json
    Mixed,       // Multiple ecosystems detected
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Framework {
    Flask,
    FastAPI,
    React,
    NestJS,
    Danet,
    SpringBoot,
    NextJS,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkDetectionResult {
    pub primary_ecosystem: LanguageEcosystem,
    pub detected_frameworks: Vec<EnhancedDetectedFramework>,
    pub confidence_summary: HashMap<Framework, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedDetectedFramework {
    pub framework: Framework,
    pub version: Option<String>,
    pub confidence: f32,
    pub evidence: Vec<DetectionEvidence>,
    pub usage_extent: UsageExtent,
    pub ecosystem: LanguageEcosystem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionEvidence {
    pub evidence_type: EvidenceType,
    pub source: String,
    pub pattern: String,
    pub confidence_weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceType {
    ConfigFile,     // requirements.txt, package.json, pom.xml
    ImportPattern,  // from flask import, import React
    FileStructure,  // /templates, /src/components
    ContentPattern, // @app.route, @Controller
}

pub struct FrameworkDetector {
    pub codebase_path: String,
}

impl FrameworkDetector {
    pub fn new(codebase_path: String) -> Self {
        Self { codebase_path }
    }

    /// Main entry point for framework detection
    pub fn detect_frameworks(&self) -> Result<FrameworkDetectionResult, Box<dyn std::error::Error>> {
        // Step 1: Detect primary language ecosystem
        let language_ecosystem = self.detect_language_ecosystem()?;
        
        // Step 2: Apply language-specific framework detection
        let detected_frameworks = match language_ecosystem {
            LanguageEcosystem::Python => self.detect_python_frameworks()?,
            LanguageEcosystem::JavaScript => self.detect_js_frameworks()?,
            LanguageEcosystem::TypeScript => self.detect_ts_frameworks()?,
            LanguageEcosystem::Java => self.detect_java_frameworks()?,
            LanguageEcosystem::Deno => self.detect_deno_frameworks()?,
            LanguageEcosystem::Mixed => self.detect_mixed_frameworks()?,
        };

        // Step 3: Create confidence summary
        let confidence_summary = self.create_confidence_summary(&detected_frameworks);

        Ok(FrameworkDetectionResult {
            primary_ecosystem: language_ecosystem,
            detected_frameworks,
            confidence_summary,
        })
    }

    /// Detect the primary language ecosystem
    pub fn detect_language_ecosystem(&self) -> Result<LanguageEcosystem, Box<dyn std::error::Error>> {
        let mut scores = HashMap::new();
        
        // File extension analysis
        let file_counts = self.count_files_by_extension()?;
        
        // Python ecosystem detection
        if *file_counts.get(".py").unwrap_or(&0) > 0 {
            let python_score = file_counts.get(".py").unwrap_or(&0) * 10;
            scores.insert(LanguageEcosystem::Python, python_score);
        }
        
        // TypeScript/Deno ecosystem detection  
        let ts_files = *file_counts.get(".ts").unwrap_or(&0) + *file_counts.get(".tsx").unwrap_or(&0);
        if ts_files > 0 {
            if self.has_file("deno.json")? || self.has_url_imports()? {
                scores.insert(LanguageEcosystem::Deno, &ts_files * 10);
            } else {
                scores.insert(LanguageEcosystem::TypeScript, &ts_files * 10);
            }
        }
        
        // JavaScript ecosystem detection
        if *file_counts.get(".js").unwrap_or(&0) > 0 && !self.has_file("tsconfig.json")? {
            let js_score = file_counts.get(".js").unwrap_or(&0) * 10;
            scores.insert(LanguageEcosystem::JavaScript, js_score);
        }
        
        // Java ecosystem detection
        if *file_counts.get(".java").unwrap_or(&0) > 0 {
            let java_score = file_counts.get(".java").unwrap_or(&0) * 10;
            scores.insert(LanguageEcosystem::Java, java_score);
        }
        
        // Determine primary ecosystem
        if scores.len() > 1 {
            Ok(LanguageEcosystem::Mixed)
        } else {
            Ok(scores.into_keys().next().unwrap_or(LanguageEcosystem::Mixed))
        }
    }

    /// Python framework detection
    fn detect_python_frameworks(&self) -> Result<Vec<EnhancedDetectedFramework>, Box<dyn std::error::Error>> {
        let mut frameworks = Vec::new();
        
        // Check for Flask
        if let Some(flask_detection) = self.analyze_flask_confidence()? {
            frameworks.push(flask_detection);
        }
        
        // Check for FastAPI
        if let Some(fastapi_detection) = self.analyze_fastapi_confidence()? {
            frameworks.push(fastapi_detection);
        }
        
        Ok(frameworks)
    }

    /// Analyze Flask framework confidence
    fn analyze_flask_confidence(&self) -> Result<Option<EnhancedDetectedFramework>, Box<dyn std::error::Error>> {
        let mut confidence = 0.0;
        let mut evidence = Vec::new();
        
        // Config evidence - requirements.txt
        if let Ok(requirements_content) = self.read_file("requirements.txt") {
            if requirements_content.contains("Flask") {
                confidence += 0.3;
                evidence.push(DetectionEvidence {
                    evidence_type: EvidenceType::ConfigFile,
                    source: "requirements.txt".to_string(),
                    pattern: "Flask dependency".to_string(),
                    confidence_weight: 0.3,
                });
            }
        }
        
        // Config evidence - pyproject.toml
        if let Ok(pyproject_content) = self.read_file("pyproject.toml") {
            if pyproject_content.contains("flask") {
                confidence += 0.25;
                evidence.push(DetectionEvidence {
                    evidence_type: EvidenceType::ConfigFile,
                    source: "pyproject.toml".to_string(),
                    pattern: "flask dependency".to_string(),
                    confidence_weight: 0.25,
                });
            }
        }
        
        // Import evidence
        if self.has_import_pattern("from flask import")? || self.has_import_pattern("import flask")? {
            confidence += 0.2;
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::ImportPattern,
                source: "Python files".to_string(),
                pattern: "Flask imports".to_string(),
                confidence_weight: 0.2,
            });
        }
        
        // File structure evidence
        let mut structure_score = 0.0;
        if self.has_file("app.py")? {
            structure_score += 0.1;
        }
        if self.has_directory("templates")? {
            structure_score += 0.1;
        }
        if self.has_directory("static")? {
            structure_score += 0.05;
        }
        
        if structure_score > 0.0 {
            confidence += structure_score;
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::FileStructure,
                source: "Project structure".to_string(),
                pattern: "Flask file structure".to_string(),
                confidence_weight: structure_score,
            });
        }
        
        // Content evidence
        if self.has_content_pattern("@app.route")? || self.has_content_pattern("Flask(__name__)")? {
            confidence += 0.3;
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::ContentPattern,
                source: "Python files".to_string(),
                pattern: "Flask decorators/patterns".to_string(),
                confidence_weight: 0.3,
            });
        }
        
        if confidence >= 0.3 {
            Ok(Some(EnhancedDetectedFramework {
                framework: Framework::Flask,
                version: self.extract_version_from_requirements("Flask")?,
                confidence,
                evidence,
                usage_extent: self.determine_usage_extent(confidence),
                ecosystem: LanguageEcosystem::Python,
            }))
        } else {
            Ok(None)
        }
    }

    /// Analyze FastAPI framework confidence
    fn analyze_fastapi_confidence(&self) -> Result<Option<EnhancedDetectedFramework>, Box<dyn std::error::Error>> {
        let mut confidence = 0.0;
        let mut evidence = Vec::new();
        
        // Config evidence - requirements.txt
        if let Ok(requirements_content) = self.read_file("requirements.txt") {
            if requirements_content.contains("fastapi") {
                confidence += 0.3;
                evidence.push(DetectionEvidence {
                    evidence_type: EvidenceType::ConfigFile,
                    source: "requirements.txt".to_string(),
                    pattern: "fastapi dependency".to_string(),
                    confidence_weight: 0.3,
                });
            }
            if requirements_content.contains("uvicorn") {
                confidence += 0.1;
            }
        }
        
        // Import evidence
        if self.has_import_pattern("from fastapi import")? || self.has_import_pattern("import fastapi")? {
            confidence += 0.25;
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::ImportPattern,
                source: "Python files".to_string(),
                pattern: "FastAPI imports".to_string(),
                confidence_weight: 0.25,
            });
        }
        
        // File structure evidence
        let mut structure_score = 0.0;
        if self.has_file("main.py")? {
            structure_score += 0.1;
        }
        if self.has_directory("app")? {
            structure_score += 0.1;
        }
        if self.has_directory("routers")? {
            structure_score += 0.1;
        }
        
        if structure_score > 0.0 {
            confidence += structure_score;
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::FileStructure,
                source: "Project structure".to_string(),
                pattern: "FastAPI file structure".to_string(),
                confidence_weight: structure_score,
            });
        }
        
        // Content evidence
        if self.has_content_pattern("FastAPI()")? || self.has_content_pattern("@router")? || self.has_content_pattern("BaseModel")? {
            confidence += 0.3;
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::ContentPattern,
                source: "Python files".to_string(),
                pattern: "FastAPI patterns".to_string(),
                confidence_weight: 0.3,
            });
        }
        
        if confidence >= 0.3 {
            Ok(Some(EnhancedDetectedFramework {
                framework: Framework::FastAPI,
                version: self.extract_version_from_requirements("fastapi")?,
                confidence,
                evidence,
                usage_extent: self.determine_usage_extent(confidence),
                ecosystem: LanguageEcosystem::Python,
            }))
        } else {
            Ok(None)
        }
    }

    /// JavaScript framework detection
    fn detect_js_frameworks(&self) -> Result<Vec<EnhancedDetectedFramework>, Box<dyn std::error::Error>> {
        let mut frameworks = Vec::new();
        
        // Check for React (JavaScript)
        if let Some(react_detection) = self.analyze_react_confidence()? {
            frameworks.push(react_detection);
        }
        
        Ok(frameworks)
    }

    /// TypeScript framework detection
    fn detect_ts_frameworks(&self) -> Result<Vec<EnhancedDetectedFramework>, Box<dyn std::error::Error>> {
        let mut frameworks = Vec::new();
        
        // Check for React (TypeScript)
        if let Some(react_detection) = self.analyze_react_confidence()? {
            frameworks.push(react_detection);
        }
        
        // Check for NestJS
        if let Some(nestjs_detection) = self.analyze_nestjs_confidence()? {
            frameworks.push(nestjs_detection);
        }
        
        // Check for Next.js
        if let Some(nextjs_detection) = self.analyze_nextjs_confidence()? {
            frameworks.push(nextjs_detection);
        }
        
        Ok(frameworks)
    }

    fn detect_java_frameworks(&self) -> Result<Vec<EnhancedDetectedFramework>, Box<dyn std::error::Error>> {
        // TODO: Implement Java framework detection
        Ok(Vec::new())
    }

    fn detect_deno_frameworks(&self) -> Result<Vec<EnhancedDetectedFramework>, Box<dyn std::error::Error>> {
        // TODO: Implement Deno framework detection
        Ok(Vec::new())
    }

    fn detect_mixed_frameworks(&self) -> Result<Vec<EnhancedDetectedFramework>, Box<dyn std::error::Error>> {
        // TODO: Implement mixed ecosystem framework detection
        Ok(Vec::new())
    }

    /// Analyze React framework confidence
    fn analyze_react_confidence(&self) -> Result<Option<EnhancedDetectedFramework>, Box<dyn std::error::Error>> {
        let mut confidence = 0.0;
        let mut evidence = Vec::new();
        
        // Config evidence - package.json
        if let Ok(package_content) = self.read_file("package.json") {
            if package_content.contains("\"react\"") {
                confidence += 0.3;
                evidence.push(DetectionEvidence {
                    evidence_type: EvidenceType::ConfigFile,
                    source: "package.json".to_string(),
                    pattern: "React dependency".to_string(),
                    confidence_weight: 0.3,
                });
            }
            if package_content.contains("\"react-dom\"") {
                confidence += 0.1;
            }
            if package_content.contains("\"@types/react\"") {
                confidence += 0.1;
            }
        }
        
        // Import evidence
        if self.has_js_import_pattern("import React")? || self.has_js_import_pattern("from 'react'")? {
            confidence += 0.2;
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::ImportPattern,
                source: "JS/TS files".to_string(),
                pattern: "React imports".to_string(),
                confidence_weight: 0.2,
            });
        }
        
        // File structure evidence
        let mut structure_score = 0.0;
        if self.has_directory("src/components")? {
            structure_score += 0.1;
        }
        if self.has_directory("public")? {
            structure_score += 0.05;
        }
        if self.has_file("src/App.tsx")? || self.has_file("src/App.jsx")? {
            structure_score += 0.1;
        }
        
        if structure_score > 0.0 {
            confidence += structure_score;
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::FileStructure,
                source: "Project structure".to_string(),
                pattern: "React file structure".to_string(),
                confidence_weight: structure_score,
            });
        }
        
        // Content evidence
        if self.has_jsx_content()? {
            confidence += 0.25;
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::ContentPattern,
                source: "JS/TS files".to_string(),
                pattern: "JSX/TSX content".to_string(),
                confidence_weight: 0.25,
            });
        }
        
        if confidence >= 0.3 {
            let ecosystem = if self.has_file("tsconfig.json")? { 
                LanguageEcosystem::TypeScript 
            } else { 
                LanguageEcosystem::JavaScript 
            };
            
            Ok(Some(EnhancedDetectedFramework {
                framework: Framework::React,
                version: self.extract_version_from_package("react")?,
                confidence,
                evidence,
                usage_extent: self.determine_usage_extent(confidence),
                ecosystem,
            }))
        } else {
            Ok(None)
        }
    }

    /// Analyze NestJS framework confidence
    fn analyze_nestjs_confidence(&self) -> Result<Option<EnhancedDetectedFramework>, Box<dyn std::error::Error>> {
        let mut confidence = 0.0;
        let mut evidence = Vec::new();
        
        // Config evidence - package.json
        if let Ok(package_content) = self.read_file("package.json") {
            if package_content.contains("\"@nestjs/core\"") {
                confidence += 0.3;
                evidence.push(DetectionEvidence {
                    evidence_type: EvidenceType::ConfigFile,
                    source: "package.json".to_string(),
                    pattern: "@nestjs/core dependency".to_string(),
                    confidence_weight: 0.3,
                });
            }
            if package_content.contains("\"@nestjs/common\"") {
                confidence += 0.1;
            }
        }
        
        // Config evidence - nest-cli.json
        if self.has_file("nest-cli.json")? {
            confidence += 0.2;
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::ConfigFile,
                source: "nest-cli.json".to_string(),
                pattern: "NestJS CLI config".to_string(),
                confidence_weight: 0.2,
            });
        }
        
        // Import evidence
        if self.has_js_import_pattern("from '@nestjs/")? {
            confidence += 0.2;
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::ImportPattern,
                source: "TS files".to_string(),
                pattern: "NestJS imports".to_string(),
                confidence_weight: 0.2,
            });
        }
        
        // File structure evidence
        let mut structure_score = 0.0;
        if self.has_file("main.ts")? {
            structure_score += 0.1;
        }
        if self.has_directory("src/controllers")? {
            structure_score += 0.1;
        }
        if self.has_directory("src/services")? {
            structure_score += 0.1;
        }
        
        if structure_score > 0.0 {
            confidence += structure_score;
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::FileStructure,
                source: "Project structure".to_string(),
                pattern: "NestJS file structure".to_string(),
                confidence_weight: structure_score,
            });
        }
        
        // Content evidence - decorators
        if self.has_decorator_pattern("@Controller")? || 
           self.has_decorator_pattern("@Injectable")? || 
           self.has_decorator_pattern("@Module")? {
            confidence += 0.3;
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::ContentPattern,
                source: "TS files".to_string(),
                pattern: "NestJS decorators".to_string(),
                confidence_weight: 0.3,
            });
        }
        
        if confidence >= 0.3 {
            Ok(Some(EnhancedDetectedFramework {
                framework: Framework::NestJS,
                version: self.extract_version_from_package("@nestjs/core")?,
                confidence,
                evidence,
                usage_extent: self.determine_usage_extent(confidence),
                ecosystem: LanguageEcosystem::TypeScript,
            }))
        } else {
            Ok(None)
        }
    }

    /// Analyze Next.js framework confidence
    fn analyze_nextjs_confidence(&self) -> Result<Option<EnhancedDetectedFramework>, Box<dyn std::error::Error>> {
        let mut confidence = 0.0;
        let mut evidence = Vec::new();
        
        // Config evidence - package.json
        if let Ok(package_content) = self.read_file("package.json") {
            if package_content.contains("\"next\"") {
                confidence += 0.4;
                evidence.push(DetectionEvidence {
                    evidence_type: EvidenceType::ConfigFile,
                    source: "package.json".to_string(),
                    pattern: "Next.js dependency".to_string(),
                    confidence_weight: 0.4,
                });
            }
        }
        
        // Config evidence - next.config.js
        if self.has_file("next.config.js")? || self.has_file("next.config.ts")? {
            confidence += 0.2;
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::ConfigFile,
                source: "next.config.js".to_string(),
                pattern: "Next.js config file".to_string(),
                confidence_weight: 0.2,
            });
        }
        
        // File structure evidence - App Router or Pages Router
        let mut structure_score = 0.0;
        if self.has_directory("app")? && self.has_file("app/layout.tsx")? {
            structure_score += 0.2; // App Router
        } else if self.has_directory("pages")? {
            structure_score += 0.15; // Pages Router
        }
        
        if self.has_directory("public")? {
            structure_score += 0.05;
        }
        
        if structure_score > 0.0 {
            confidence += structure_score;
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::FileStructure,
                source: "Project structure".to_string(),
                pattern: "Next.js file structure".to_string(),
                confidence_weight: structure_score,
            });
        }
        
        // Content evidence
        if self.has_nextjs_patterns()? {
            confidence += 0.2;
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::ContentPattern,
                source: "JS/TS files".to_string(),
                pattern: "Next.js patterns".to_string(),
                confidence_weight: 0.2,
            });
        }
        
        if confidence >= 0.4 {
            let ecosystem = if self.has_file("tsconfig.json")? { 
                LanguageEcosystem::TypeScript 
            } else { 
                LanguageEcosystem::JavaScript 
            };
            
            Ok(Some(EnhancedDetectedFramework {
                framework: Framework::NextJS,
                version: self.extract_version_from_package("next")?,
                confidence,
                evidence,
                usage_extent: self.determine_usage_extent(confidence),
                ecosystem,
            }))
        } else {
            Ok(None)
        }
    }

    // Utility methods
    fn count_files_by_extension(&self) -> Result<HashMap<String, usize>, Box<dyn std::error::Error>> {
        let mut counts = HashMap::new();
        let path = Path::new(&self.codebase_path);
        
        if path.is_dir() {
            for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    if let Some(extension) = entry.path().extension() {
                        let ext = format!(".{}", extension.to_string_lossy());
                        *counts.entry(ext).or_insert(0) += 1;
                    }
                }
            }
        }
        
        Ok(counts)
    }

    fn has_file(&self, filename: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let path = Path::new(&self.codebase_path).join(filename);
        Ok(path.exists())
    }

    fn has_directory(&self, dirname: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let path = Path::new(&self.codebase_path).join(dirname);
        Ok(path.is_dir())
    }

    fn read_file(&self, filename: &str) -> Result<String, Box<dyn std::error::Error>> {
        let path = Path::new(&self.codebase_path).join(filename);
        let content = fs::read_to_string(path)?;
        Ok(content)
    }

    fn has_import_pattern(&self, pattern: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let path = Path::new(&self.codebase_path);
        
        for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Some(extension) = entry.path().extension() {
                    if extension == "py" {
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            if content.contains(pattern) {
                                return Ok(true);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(false)
    }

    fn has_content_pattern(&self, pattern: &str) -> Result<bool, Box<dyn std::error::Error>> {
        self.has_import_pattern(pattern) // Same implementation for now
    }

    fn has_url_imports(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let path = Path::new(&self.codebase_path);
        
        for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Some(extension) = entry.path().extension() {
                    if extension == "ts" || extension == "js" {
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            if content.contains("https://deno.land/") {
                                return Ok(true);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(false)
    }

    fn extract_version_from_requirements(&self, package_name: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        if let Ok(content) = self.read_file("requirements.txt") {
            for line in content.lines() {
                if line.contains(package_name) {
                    // Simple version extraction - could be enhanced
                    if let Some(version_part) = line.split("==").nth(1) {
                        return Ok(Some(version_part.trim().to_string()));
                    }
                    if let Some(version_part) = line.split(">=").nth(1) {
                        return Ok(Some(format!(">={}", version_part.trim())));
                    }
                }
            }
        }
        Ok(None)
    }

    fn extract_version_from_package(&self, package_name: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        if let Ok(content) = self.read_file("package.json") {
            // Simple JSON parsing - in production should use a proper JSON parser
            let search_pattern = format!("\"{}\":", package_name);
            if let Some(start) = content.find(&search_pattern) {
                if let Some(version_start) = content[start..].find("\"") {
                    if let Some(version_end) = content[start + version_start + 1..].find("\"") {
                        let version = &content[start + version_start + 1..start + version_start + 1 + version_end];
                        return Ok(Some(version.trim().to_string()));
                    }
                }
            }
        }
        Ok(None)
    }

    fn has_js_import_pattern(&self, pattern: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let path = Path::new(&self.codebase_path);
        
        for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Some(extension) = entry.path().extension() {
                    if extension == "js" || extension == "ts" || extension == "jsx" || extension == "tsx" {
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            if content.contains(pattern) {
                                return Ok(true);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(false)
    }

    fn has_jsx_content(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let path = Path::new(&self.codebase_path);
        
        for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Some(extension) = entry.path().extension() {
                    if extension == "jsx" || extension == "tsx" {
                        return Ok(true);
                    }
                    if extension == "js" || extension == "ts" {
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            // Look for JSX patterns
                            if content.contains("React.createElement") ||
                               content.contains("<div") ||
                               content.contains("<span") ||
                               content.contains("useState") ||
                               content.contains("useEffect") {
                                return Ok(true);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(false)
    }

    fn has_decorator_pattern(&self, pattern: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let path = Path::new(&self.codebase_path);
        
        for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Some(extension) = entry.path().extension() {
                    if extension == "ts" || extension == "js" {
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            if content.contains(pattern) {
                                return Ok(true);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(false)
    }

    fn has_nextjs_patterns(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let path = Path::new(&self.codebase_path);
        
        for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Some(extension) = entry.path().extension() {
                    if extension == "ts" || extension == "tsx" || extension == "js" || extension == "jsx" {
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            if content.contains("getServerSideProps") ||
                               content.contains("getStaticProps") ||
                               content.contains("'use client'") ||
                               content.contains("'use server'") ||
                               content.contains("next/router") ||
                               content.contains("next/navigation") {
                                return Ok(true);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(false)
    }

    fn determine_usage_extent(&self, confidence: f32) -> UsageExtent {
        match confidence {
            c if c >= 0.8 => UsageExtent::Core,
            c if c >= 0.6 => UsageExtent::Extensive,
            c if c >= 0.4 => UsageExtent::Moderate,
            _ => UsageExtent::Limited,
        }
    }

    fn create_confidence_summary(&self, frameworks: &[EnhancedDetectedFramework]) -> HashMap<Framework, f32> {
        let mut summary = HashMap::new();
        for framework in frameworks {
            summary.insert(framework.framework.clone(), framework.confidence);
        }
        summary
    }
}

// Convert to legacy DetectedFramework for compatibility
impl From<EnhancedDetectedFramework> for DetectedFramework {
    fn from(enhanced: EnhancedDetectedFramework) -> Self {
        DetectedFramework {
            name: format!("{:?}", enhanced.framework),
            version: enhanced.version,
            confidence: enhanced.confidence,
            evidence: enhanced.evidence.into_iter().map(|e| e.pattern).collect(),
            usage_extent: enhanced.usage_extent,
        }
    }
}