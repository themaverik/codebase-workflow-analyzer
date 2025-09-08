use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use anyhow::Result;

use crate::core::UsageExtent;
use crate::core::types::{Framework, LanguageEcosystem};

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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
            LanguageEcosystem::Rust => Vec::new(), // TODO: Implement Rust framework detection
            LanguageEcosystem::Go => Vec::new(), // TODO: Implement Go framework detection
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
        
        // File extension analysis (excluding common ignore directories)
        let file_counts = self.count_files_by_extension()?;
        
        // Python ecosystem detection
        if *file_counts.get(".py").unwrap_or(&0) > 0 {
            let python_score = *file_counts.get(".py").unwrap_or(&0) * 10;
            scores.insert(LanguageEcosystem::Python, python_score);
        }
        
        // TypeScript/Deno ecosystem detection  
        let ts_files = *file_counts.get(".ts").unwrap_or(&0) + *file_counts.get(".tsx").unwrap_or(&0);
        if ts_files > 0 {
            if self.has_file("deno.json")? || self.has_url_imports()? {
                scores.insert(LanguageEcosystem::Deno, ts_files * 10);
            } else {
                scores.insert(LanguageEcosystem::TypeScript, ts_files * 10);
            }
        }
        
        // JavaScript ecosystem detection
        if *file_counts.get(".js").unwrap_or(&0) > 0 && !self.has_file("tsconfig.json")? {
            let js_score = *file_counts.get(".js").unwrap_or(&0) * 10;
            scores.insert(LanguageEcosystem::JavaScript, js_score);
        }
        
        // Java ecosystem detection
        if *file_counts.get(".java").unwrap_or(&0) > 0 {
            let java_score = *file_counts.get(".java").unwrap_or(&0) * 10;
            scores.insert(LanguageEcosystem::Java, java_score);
        }
        
        // Determine primary ecosystem - prioritize the one with highest score
        if scores.is_empty() {
            Ok(LanguageEcosystem::Mixed)
        } else {
            // Return the ecosystem with the highest score
            let primary_ecosystem = scores.into_iter().max_by_key(|(_, score)| *score).unwrap().0;
            Ok(primary_ecosystem)
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
    /// Analyze Flask framework confidence with enhanced patterns
    fn analyze_flask_confidence(&self) -> Result<Option<EnhancedDetectedFramework>, Box<dyn std::error::Error>> {
        let mut confidence: f32 = 0.0;
        let mut evidence = Vec::new();
        
        // 1. Dependency analysis - requirements.txt
        if let Ok(requirements_content) = self.read_file("requirements.txt") {
            if requirements_content.contains("Flask") || requirements_content.contains("flask") {
                confidence += 0.3;
                evidence.push(DetectionEvidence {
                    evidence_type: EvidenceType::ConfigFile,
                    source: "requirements.txt".to_string(),
                    pattern: "Flask dependency".to_string(),
                    confidence_weight: 0.3,
                });
            }
            // Look for Flask extensions
            let flask_extensions = vec!["Flask-SQLAlchemy", "Flask-Login", "Flask-WTF", "Flask-Migrate", "Flask-Admin"];
            for extension in &flask_extensions {
                if requirements_content.contains(extension) {
                    confidence += 0.05;
                }
            }
        }
        
        // 2. Config evidence - pyproject.toml
        if let Ok(pyproject_content) = self.read_file("pyproject.toml") {
            if pyproject_content.contains("flask") || pyproject_content.contains("Flask") {
                confidence += 0.25;
                evidence.push(DetectionEvidence {
                    evidence_type: EvidenceType::ConfigFile,
                    source: "pyproject.toml".to_string(),
                    pattern: "flask dependency".to_string(),
                    confidence_weight: 0.25,
                });
            }
        }
        
        // 3. Import pattern analysis
        let flask_import_patterns = vec![
            "from flask import",
            "import flask",
            "from flask import Flask",
            "from flask import request",
            "from flask import jsonify",
            "from flask import render_template",
            "Flask(__name__)"
        ];
        
        if self.has_patterns_in_files(&["*.py"], &flask_import_patterns)? {
            confidence += 0.2;
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::ImportPattern,
                source: "Python files".to_string(),
                pattern: "Flask imports".to_string(),
                confidence_weight: 0.2,
            });
        }
        
        // 4. Flask-specific code patterns
        let flask_code_patterns = vec![
            "@app.route",
            "app = Flask(",
            ".run(debug=",
            "request.json",
            "request.form",
            "session[",
            "flash(",
            "url_for(",
            "redirect("
        ];
        
        if self.has_patterns_in_files(&["*.py"], &flask_code_patterns)? {
            confidence += 0.2;
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::ContentPattern,
                source: "Python files".to_string(),
                pattern: "Flask decorators and functions".to_string(),
                confidence_weight: 0.2,
            });
        }
        
        // 5. File structure evidence
        let mut structure_score = 0.0;
        if self.has_file("app.py")? {
            structure_score += 0.1;
        }
        if self.has_file("wsgi.py")? {
            structure_score += 0.05;
        }
        if self.has_directory("templates")? {
            structure_score += 0.1;
        }
        if self.has_directory("static")? {
            structure_score += 0.05;
        }
        if self.has_file("config.py")? {
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
            // Cap confidence at 1.0 (100%)
            let normalized_confidence = confidence.min(1.0);
            
            Ok(Some(EnhancedDetectedFramework {
                framework: Framework::Flask,
                version: self.extract_version_from_requirements("Flask")?,
                confidence: normalized_confidence,
                evidence,
                usage_extent: self.determine_usage_extent(normalized_confidence),
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
            // Cap confidence at 1.0 (100%)
            let normalized_confidence = confidence.min(1.0);
            
            Ok(Some(EnhancedDetectedFramework {
                framework: Framework::FastAPI,
                version: self.extract_version_from_requirements("fastapi")?,
                confidence: normalized_confidence,
                evidence,
                usage_extent: self.determine_usage_extent(normalized_confidence),
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
        let mut frameworks = Vec::new();
        
        // Spring Boot detection
        if let Some(spring_boot) = self.analyze_spring_boot_confidence()? {
            frameworks.push(spring_boot);
        }
        
        // Future: Add other Java frameworks (Quarkus, Micronaut, etc.)
        
        Ok(frameworks)
    }

    fn detect_deno_frameworks(&self) -> Result<Vec<EnhancedDetectedFramework>, Box<dyn std::error::Error>> {
        let mut frameworks = Vec::new();
        
        // Danet framework detection
        if let Some(danet_framework) = self.analyze_danet_confidence()? {
            frameworks.push(danet_framework);
        }
        
        // Future: Add other Deno frameworks (Fresh, Ultra, etc.)
        
        Ok(frameworks)
    }

    fn detect_mixed_frameworks(&self) -> Result<Vec<EnhancedDetectedFramework>, Box<dyn std::error::Error>> {
        let mut frameworks = Vec::new();
        
        // Detect all framework types across different ecosystems
        let js_frameworks = self.detect_js_frameworks()?;
        let ts_frameworks = self.detect_ts_frameworks()?;
        let python_frameworks = self.detect_python_frameworks()?;
        let java_frameworks = self.detect_java_frameworks()?;
        let deno_frameworks = self.detect_deno_frameworks()?;
        
        // Combine all detected frameworks
        frameworks.extend(js_frameworks);
        frameworks.extend(ts_frameworks);
        frameworks.extend(python_frameworks);
        frameworks.extend(java_frameworks);
        frameworks.extend(deno_frameworks);
        
        // Analyze cross-framework relationships and dependencies
        self.analyze_cross_framework_dependencies(&mut frameworks)?;
        
        // Detect common multi-framework patterns
        self.detect_fullstack_patterns(&mut frameworks)?;
        
        Ok(frameworks)
    }

    /// Analyze React framework confidence with enhanced patterns
    fn analyze_react_confidence(&self) -> Result<Option<EnhancedDetectedFramework>, Box<dyn std::error::Error>> {
        let mut confidence: f32 = 0.0;
        let mut evidence = Vec::new();
        
        // 1. Package.json analysis (strongest indicator)
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
                confidence += 0.15;
                evidence.push(DetectionEvidence {
                    evidence_type: EvidenceType::ConfigFile,
                    source: "package.json".to_string(),
                    pattern: "ReactDOM dependency".to_string(),
                    confidence_weight: 0.15,
                });
            }
            if package_content.contains("\"@types/react\"") {
                confidence += 0.1;
                evidence.push(DetectionEvidence {
                    evidence_type: EvidenceType::ConfigFile,
                    source: "package.json".to_string(),
                    pattern: "React TypeScript types".to_string(),
                    confidence_weight: 0.1,
                });
            }
        }
        
        // 2. Import pattern analysis
        let react_import_patterns = vec![
            "import React",
            "from 'react'",
            "from \"react\"",
            "import { useState",
            "import { useEffect",
            "import { Component",
            "import { FC",
            "import { FunctionComponent"
        ];
        
        if self.has_patterns_in_files(&["*.js", "*.jsx", "*.ts", "*.tsx"], &react_import_patterns)? {
            confidence += 0.2;
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::ImportPattern,
                source: "JS/TS files".to_string(),
                pattern: "React imports and hooks".to_string(),
                confidence_weight: 0.2,
            });
        }
        
        // 3. File structure analysis
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
        if self.has_file("src/index.tsx")? || self.has_file("src/index.jsx")? {
            structure_score += 0.05;
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
        
        // 4. JSX/TSX content analysis
        if self.has_jsx_content()? {
            confidence += 0.25;
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::ContentPattern,
                source: "JSX/TSX files".to_string(),
                pattern: "JSX/TSX content".to_string(),
                confidence_weight: 0.25,
            });
        }
        
        // 5. React-specific patterns
        let react_patterns = vec![
            "ReactDOM.render",
            "createRoot",
            "render(",
            "useState(",
            "useEffect(",
            "useContext(",
            "props.",
            "this.state",
            "this.setState"
        ];
        
        if self.has_patterns_in_files(&["*.js", "*.jsx", "*.ts", "*.tsx"], &react_patterns)? {
            confidence += 0.15;
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::ContentPattern,
                source: "Component files".to_string(),
                pattern: "React patterns and lifecycle methods".to_string(),
                confidence_weight: 0.15,
            });
        }
        
        if confidence >= 0.3 {
            let ecosystem = if self.has_file("tsconfig.json")? { 
                LanguageEcosystem::TypeScript 
            } else { 
                LanguageEcosystem::JavaScript 
            };
            
            // Apply enhanced confidence validation
            let weighted_confidence = self.calculate_weighted_confidence(&evidence);
            let cross_validation_factor = self.cross_validate_framework_detection(&Framework::React, &evidence);
            let final_confidence = (weighted_confidence * cross_validation_factor).min(1.0);
            
            Ok(Some(EnhancedDetectedFramework {
                framework: Framework::React,
                version: self.extract_version_from_package("react")?,
                confidence: final_confidence,
                evidence,
                usage_extent: self.determine_usage_extent(final_confidence),
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
            // Cap confidence at 1.0 (100%)
            let normalized_confidence = confidence.min(1.0);
            
            Ok(Some(EnhancedDetectedFramework {
                framework: Framework::NestJS,
                version: self.extract_version_from_package("@nestjs/core")?,
                confidence: normalized_confidence,
                evidence,
                usage_extent: self.determine_usage_extent(normalized_confidence),
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
            
            // Cap confidence at 1.0 (100%)
            let normalized_confidence = confidence.min(1.0);
            
            Ok(Some(EnhancedDetectedFramework {
                framework: Framework::NextJS,
                version: self.extract_version_from_package("next")?,
                confidence: normalized_confidence,
                evidence,
                usage_extent: self.determine_usage_extent(normalized_confidence),
                ecosystem,
            }))
        } else {
            Ok(None)
        }
    }

    /// Analyze Spring Boot framework confidence with enhanced patterns
    fn analyze_spring_boot_confidence(&self) -> Result<Option<EnhancedDetectedFramework>, Box<dyn std::error::Error>> {
        let mut confidence: f32 = 0.0;
        let mut evidence = Vec::new();
        
        // 1. Maven dependency analysis - pom.xml
        if let Ok(pom_content) = self.read_file("pom.xml") {
            if pom_content.contains("spring-boot-starter") {
                confidence += 0.4;
                evidence.push(DetectionEvidence {
                    evidence_type: EvidenceType::ConfigFile,
                    source: "pom.xml".to_string(),
                    pattern: "spring-boot-starter dependencies".to_string(),
                    confidence_weight: 0.4,
                });
            }
            if pom_content.contains("org.springframework.boot") {
                confidence += 0.2;
                evidence.push(DetectionEvidence {
                    evidence_type: EvidenceType::ConfigFile,
                    source: "pom.xml".to_string(),
                    pattern: "Spring Boot parent/dependencies".to_string(),
                    confidence_weight: 0.2,
                });
            }
        }
        
        // 2. Gradle dependency analysis - build.gradle
        if let Ok(gradle_content) = self.read_file("build.gradle") {
            if gradle_content.contains("org.springframework.boot") || gradle_content.contains("spring-boot-starter") {
                confidence += 0.35;
                evidence.push(DetectionEvidence {
                    evidence_type: EvidenceType::ConfigFile,
                    source: "build.gradle".to_string(),
                    pattern: "Spring Boot Gradle dependencies".to_string(),
                    confidence_weight: 0.35,
                });
            }
        }
        
        // 3. Spring Boot specific files
        if self.has_file("src/main/java")? || self.has_file("application.yml")? || self.has_file("application.properties")? {
            confidence += 0.15;
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::FileStructure,
                source: "Project structure".to_string(),
                pattern: "Spring Boot project structure".to_string(),
                confidence_weight: 0.15,
            });
        }
        
        // 4. Spring Boot annotations and imports
        let spring_boot_patterns = vec![
            "@SpringBootApplication",
            "@RestController",
            "@Controller",
            "@Service",
            "@Repository",
            "@Component",
            "@Autowired",
            "import org.springframework",
            "SpringApplication.run",
            "@GetMapping",
            "@PostMapping",
            "@RequestMapping"
        ];
        
        if self.has_patterns_in_files(&["*.java"], &spring_boot_patterns)? {
            confidence += 0.25;
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::ContentPattern,
                source: "Java files".to_string(),
                pattern: "Spring Boot annotations and patterns".to_string(),
                confidence_weight: 0.25,
            });
        }
        
        // 5. Spring Boot configuration files
        let mut config_score = 0.0;
        if self.has_file("application.properties")? {
            config_score += 0.1;
        }
        if self.has_file("application.yml")? {
            config_score += 0.1;
        }
        if self.has_file("bootstrap.yml")? || self.has_file("bootstrap.properties")? {
            config_score += 0.05;
        }
        
        if config_score > 0.0 {
            confidence += config_score;
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::ConfigFile,
                source: "Configuration files".to_string(),
                pattern: "Spring Boot configuration files".to_string(),
                confidence_weight: config_score,
            });
        }
        
        // Cap confidence at 1.0 (100%)
        let normalized_confidence = confidence.min(1.0);
        
        if normalized_confidence > 0.3 {
            // Apply enhanced confidence validation
            let weighted_confidence = self.calculate_weighted_confidence(&evidence);
            let cross_validation_factor = self.cross_validate_framework_detection(&Framework::SpringBoot, &evidence);
            let final_confidence = (weighted_confidence * cross_validation_factor).min(1.0);
            
            Ok(Some(EnhancedDetectedFramework {
                framework: Framework::SpringBoot,
                version: self.extract_spring_boot_version()?,
                confidence: final_confidence,
                evidence,
                usage_extent: self.determine_usage_extent(final_confidence),
                ecosystem: LanguageEcosystem::Java,
            }))
        } else {
            Ok(None)
        }
    }
    
    /// Extract Spring Boot version from Maven or Gradle files
    fn extract_spring_boot_version(&self) -> Result<Option<String>, Box<dyn std::error::Error>> {
        // Try Maven first
        if let Ok(pom_content) = self.read_file("pom.xml") {
            if let Some(version) = self.extract_maven_version(&pom_content, "org.springframework.boot") {
                return Ok(Some(version));
            }
        }
        
        // Try Gradle
        if let Ok(gradle_content) = self.read_file("build.gradle") {
            if let Some(version) = self.extract_gradle_version(&gradle_content, "org.springframework.boot") {
                return Ok(Some(version));
            }
        }
        
        Ok(None)
    }
    
    /// Extract version from Maven pom.xml
    fn extract_maven_version(&self, content: &str, group_id: &str) -> Option<String> {
        use regex::Regex;
        
        // Look for version in dependency or parent section
        let patterns = vec![
            format!(r"<groupId>{}</groupId>\s*<artifactId>[^<]*</artifactId>\s*<version>([^<]+)</version>", regex::escape(group_id)),
            format!(r"<groupId>{}</groupId>\s*<artifactId>[^<]*</artifactId>\s*<version>([^<]+)</version>", regex::escape(group_id))
        ];
        
        for pattern in patterns {
            if let Ok(re) = Regex::new(&pattern) {
                if let Some(captures) = re.captures(content) {
                    return captures.get(1).map(|m| m.as_str().to_string());
                }
            }
        }
        
        None
    }
    
    /// Extract version from Gradle build.gradle
    fn extract_gradle_version(&self, content: &str, group_id: &str) -> Option<String> {
        use regex::Regex;
        
        // Look for version in implementation or dependency
        let pattern = format!(r#"['"]{}[^'"]+'([^'"]+)['"]"#, regex::escape(group_id));
        if let Ok(re) = Regex::new(&pattern) {
            if let Some(captures) = re.captures(content) {
                return captures.get(1).map(|m| m.as_str().to_string());
            }
        }
        
        None
    }

    /// Analyze Danet framework confidence
    fn analyze_danet_confidence(&self) -> Result<Option<EnhancedDetectedFramework>, Box<dyn std::error::Error>> {
        let mut confidence: f32 = 0.0;
        let mut evidence = Vec::new();
        
        // Prerequisite: Must be a Deno project
        if !self.is_deno_project()? {
            return Ok(None);
        }
        
        // 1. Danet import detection (strongest signal)
        if self.has_danet_imports()? {
            confidence += 0.4;
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::ImportPattern,
                source: "TypeScript files".to_string(),
                pattern: "@danet/core imports".to_string(),
                confidence_weight: 0.4,
            });
        }
        
        // 2. Danet decorators in TypeScript files
        if self.has_danet_decorators()? {
            confidence += 0.3;
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::ContentPattern,
                source: "Controller/Service files".to_string(), 
                pattern: "Danet decorators (@Controller, @Injectable)".to_string(),
                confidence_weight: 0.3,
            });
        }
        
        // 3. Deno.json task patterns
        if self.has_danet_tasks()? {
            confidence += 0.2;
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::ConfigFile,
                source: "deno.json".to_string(),
                pattern: "launch-server task".to_string(),
                confidence_weight: 0.2,
            });
        }
        
        // 4. Project structure patterns
        if self.has_danet_structure()? {
            confidence += 0.1;
            evidence.push(DetectionEvidence {
                evidence_type: EvidenceType::FileStructure,
                source: "Project structure".to_string(),
                pattern: "Danet module organization".to_string(),
                confidence_weight: 0.1,
            });
        }
        
        // Cap confidence at 1.0 (100%)
        let normalized_confidence = confidence.min(1.0);
        
        if normalized_confidence > 0.3 {
            // Apply enhanced confidence validation
            let weighted_confidence = self.calculate_weighted_confidence(&evidence);
            let cross_validation_factor = self.cross_validate_framework_detection(&Framework::Danet, &evidence);
            let final_confidence = (weighted_confidence * cross_validation_factor).min(1.0);
            
            Ok(Some(EnhancedDetectedFramework {
                framework: Framework::Danet,
                version: self.extract_danet_version()?,
                confidence: final_confidence,
                evidence,
                usage_extent: self.determine_usage_extent(final_confidence),
                ecosystem: LanguageEcosystem::Deno,
            }))
        } else {
            Ok(None)
        }
    }

    /// Check if this is a Deno project
    fn is_deno_project(&self) -> Result<bool, Box<dyn std::error::Error>> {
        // Primary indicators of Deno project
        if self.has_file("deno.json")? || self.has_file("deno.jsonc")? {
            return Ok(true);
        }
        
        // Secondary indicator: URL-based imports in TypeScript files
        if self.has_url_imports()? {
            return Ok(true);
        }
        
        // Tertiary indicator: Deno-specific files
        if self.has_file("deno.lock")? {
            return Ok(true);
        }
        
        Ok(false)
    }
    
    /// Check for Danet-specific import patterns
    fn has_danet_imports(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let danet_import_patterns = vec![
            "jsr:@danet/core",
            "jsr:@danet/cli", 
            "https://deno.land/x/danet",
            "@danet/core",
            "from \"@danet/",
            "from 'jsr:@danet/"
        ];
        
        self.has_patterns_in_files(&["*.ts", "*.tsx"], &danet_import_patterns)
    }
    
    /// Check for Danet-specific decorators
    fn has_danet_decorators(&self) -> Result<bool, Box<dyn std::error::Error>> {
        // Look for Danet decorators in context of Deno imports
        let danet_decorator_patterns = vec![
            "@Controller",
            "@Injectable", 
            "@Module",
            "@Get",
            "@Post",
            "@Put",
            "@Delete",
            "@Patch"
        ];
        
        // Must have both decorators AND Danet imports to distinguish from NestJS
        if self.has_patterns_in_files(&["*.ts", "*.tsx"], &danet_decorator_patterns)? {
            // Validate it's Danet, not NestJS by checking for Deno-specific patterns
            return self.has_danet_imports();
        }
        
        Ok(false)
    }
    
    /// Check for Danet-specific tasks in deno.json
    fn has_danet_tasks(&self) -> Result<bool, Box<dyn std::error::Error>> {
        if let Ok(content) = self.read_file("deno.json") {
            if content.contains("launch-server") || content.contains("danet") {
                return Ok(true);
            }
        }
        
        if let Ok(content) = self.read_file("deno.jsonc") {
            if content.contains("launch-server") || content.contains("danet") {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// Check for Danet-specific project structure
    fn has_danet_structure(&self) -> Result<bool, Box<dyn std::error::Error>> {
        // Look for typical Danet project structure patterns
        let structure_indicators = vec![
            "src/controllers",
            "src/modules", 
            "src/services",
            "controllers",
            "modules",
            "services"
        ];
        
        for indicator in structure_indicators {
            if self.has_directory(indicator)? {
                // Verify it's Danet by checking for Deno project markers
                return self.is_deno_project();
            }
        }
        
        Ok(false)
    }
    
    /// Extract Danet version from imports or config
    fn extract_danet_version(&self) -> Result<Option<String>, Box<dyn std::error::Error>> {
        // Try to extract version from jsr: imports
        if let Ok(files) = self.find_files_with_extension("ts") {
            for file_path in files {
                if let Ok(content) = fs::read_to_string(&file_path) {
                    // Look for jsr:@danet/core@1.2.3 patterns
                    if let Some(version_match) = self.extract_jsr_version(&content, "@danet/core") {
                        return Ok(Some(version_match));
                    }
                }
            }
        }
        
        // Try to extract from deno.json imports
        if let Ok(content) = self.read_file("deno.json") {
            if let Some(version) = self.extract_jsr_version(&content, "@danet/core") {
                return Ok(Some(version));
            }
        }
        
        Ok(None)
    }
    
    /// Extract version from JSR import pattern
    fn extract_jsr_version(&self, content: &str, package: &str) -> Option<String> {
        use regex::Regex;
        
        // Pattern: jsr:@danet/core@1.2.3 or similar
        let pattern = format!(r"jsr:{}@(\d+\.\d+\.\d+)", regex::escape(package));
        if let Ok(re) = Regex::new(&pattern) {
            if let Some(captures) = re.captures(content) {
                return captures.get(1).map(|m| m.as_str().to_string());
            }
        }
        
        None
    }

    // Utility methods
    fn count_files_by_extension(&self) -> Result<HashMap<String, usize>, Box<dyn std::error::Error>> {
        let mut counts = HashMap::new();
        let path = Path::new(&self.codebase_path);
        
        // Common directories to ignore during language detection
        let ignore_dirs = vec![
            "venv", "env", ".env", "virtualenv", ".venv",  // Python virtual environments
            "node_modules", ".pnpm-store",                  // Node.js dependencies
            ".git", ".svn", ".hg",                          // Version control
            "target", "build", "dist", "out",              // Build outputs
            "__pycache__", ".pytest_cache",                // Python cache
            ".idea", ".vscode", ".vs",                      // IDE files
            "vendor",                                       // Vendor dependencies
            ".specstory",                                   // Our tool's output
        ];
        
        if path.is_dir() {
            for entry in walkdir::WalkDir::new(path)
                .into_iter()
                .filter_entry(|e| {
                    // Skip ignored directories
                    if e.file_type().is_dir() {
                        if let Some(name) = e.file_name().to_str() {
                            return !ignore_dirs.contains(&name);
                        }
                    }
                    true
                })
                .filter_map(|e| e.ok()) 
            {
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
        
        // Common directories to ignore
        let ignore_dirs = vec![
            "venv", "env", ".env", "virtualenv", ".venv",  // Python virtual environments
            "node_modules", ".pnpm-store",                  // Node.js dependencies
            ".git", ".svn", ".hg",                          // Version control
            "target", "build", "dist", "out",              // Build outputs
            "__pycache__", ".pytest_cache",                // Python cache
            ".idea", ".vscode", ".vs",                      // IDE files
            "vendor",                                       // Vendor dependencies
        ];
        
        for entry in walkdir::WalkDir::new(path)
            .into_iter()
            .filter_entry(|e| {
                // Skip ignored directories
                if e.file_type().is_dir() {
                    if let Some(name) = e.file_name().to_str() {
                        return !ignore_dirs.contains(&name);
                    }
                }
                true
            })
            .filter_map(|e| e.ok()) 
        {
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

    /// Check for patterns in files with specific extensions
    fn has_patterns_in_files(&self, extensions: &[&str], patterns: &[&str]) -> Result<bool, Box<dyn std::error::Error>> {
        let path = Path::new(&self.codebase_path);
        
        for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Some(extension) = entry.path().extension() {
                    let ext_str = extension.to_string_lossy().to_lowercase();
                    
                    // Check if file extension matches any of the target extensions
                    let matches_extension = extensions.iter().any(|target_ext| {
                        if target_ext.starts_with("*.") {
                            target_ext[2..] == ext_str
                        } else {
                            target_ext == &ext_str
                        }
                    });
                    
                    if matches_extension {
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            for pattern in patterns {
                                if content.contains(pattern) {
                                    return Ok(true);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(false)
    }
    
    /// Find all files with a specific extension
    fn find_files_with_extension(&self, extension: &str) -> Result<Vec<std::path::PathBuf>, Box<dyn std::error::Error>> {
        let path = Path::new(&self.codebase_path);
        let mut files = Vec::new();
        
        for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Some(file_extension) = entry.path().extension() {
                    if file_extension.to_string_lossy().to_lowercase() == extension.to_lowercase() {
                        files.push(entry.path().to_path_buf());
                    }
                }
            }
        }
        
        Ok(files)
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

    /// Determine usage extent with enhanced confidence mapping
    fn determine_usage_extent(&self, confidence: f32) -> UsageExtent {
        match confidence {
            c if c >= 0.85 => UsageExtent::Core,        // Very high confidence
            c if c >= 0.65 => UsageExtent::Extensive,   // High confidence  
            c if c >= 0.45 => UsageExtent::Moderate,    // Medium confidence
            _ => UsageExtent::Limited,                   // Low confidence
        }
    }
    
    /// Enhanced confidence calculation with evidence quality weighting
    fn calculate_weighted_confidence(&self, evidence: &[DetectionEvidence]) -> f32 {
        if evidence.is_empty() {
            return 0.0;
        }
        
        let mut total_weight = 0.0;
        let mut weighted_sum = 0.0;
        
        for ev in evidence {
            let quality_multiplier = self.get_evidence_quality_multiplier(&ev.evidence_type);
            let weighted_confidence = ev.confidence_weight * quality_multiplier;
            
            weighted_sum += weighted_confidence;
            total_weight += quality_multiplier;
        }
        
        if total_weight == 0.0 {
            return 0.0;
        }
        
        // Normalize and cap at 1.0
        (weighted_sum / evidence.len() as f32).min(1.0)
    }
    
    /// Get quality multiplier for different evidence types
    fn get_evidence_quality_multiplier(&self, evidence_type: &EvidenceType) -> f32 {
        match evidence_type {
            EvidenceType::ConfigFile => 1.2,      // Config files are very reliable
            EvidenceType::ImportPattern => 1.1,   // Import patterns are quite reliable
            EvidenceType::ContentPattern => 1.0,  // Content patterns are standard reliability
            EvidenceType::FileStructure => 0.8,   // File structure is less reliable alone
        }
    }
    
    /// Cross-validate framework detection results
    fn cross_validate_framework_detection(&self, framework: &Framework, evidence: &[DetectionEvidence]) -> f32 {
        let mut confidence_adjustment = 1.0;
        
        // Check for conflicting evidence
        match framework {
            Framework::React => {
                // If we detect React, ensure we're not in a Next.js project
                if self.has_file("next.config.js").unwrap_or(false) || 
                   self.has_patterns_in_files(&["*.js", "*.ts"], &["next/", "import { NextPage"]).unwrap_or(false) {
                    confidence_adjustment *= 0.7; // Reduce React confidence if Next.js indicators present
                }
            },
            Framework::Danet => {
                // Ensure we're actually in a Deno project, not Node.js
                if self.has_file("package.json").unwrap_or(false) && 
                   !self.is_deno_project().unwrap_or(false) {
                    confidence_adjustment *= 0.3; // Heavily reduce if looks like Node.js
                }
            },
            Framework::NestJS => {
                // Ensure we're in Node.js, not Deno
                if self.is_deno_project().unwrap_or(false) {
                    confidence_adjustment *= 0.2; // Very low if in Deno project
                }
            },
            _ => {}
        }
        
        // Check evidence diversity - having multiple evidence types increases confidence
        let evidence_types: std::collections::HashSet<_> = evidence.iter()
            .map(|e| &e.evidence_type)
            .collect();
        
        let diversity_bonus = match evidence_types.len() {
            4 => 1.1,  // All evidence types present
            3 => 1.05, // Three evidence types
            2 => 1.02, // Two evidence types
            1 => 0.95, // Only one evidence type (less confident)
            _ => 0.9,  // No evidence (shouldn't happen)
        };
        
        confidence_adjustment * diversity_bonus
    }

    fn create_confidence_summary(&self, frameworks: &[EnhancedDetectedFramework]) -> HashMap<Framework, f32> {
        let mut summary = HashMap::new();
        for framework in frameworks {
            summary.insert(framework.framework.clone(), framework.confidence);
        }
        summary
    }
    
    /// Analyze cross-framework dependencies and relationships
    fn analyze_cross_framework_dependencies(&self, frameworks: &mut Vec<EnhancedDetectedFramework>) -> Result<(), Box<dyn std::error::Error>> {
        // Group frameworks by ecosystem for dependency analysis
        let mut frontend_frameworks = Vec::new();
        let mut backend_frameworks = Vec::new();
        let mut api_frameworks = Vec::new();
        
        for framework in frameworks.iter_mut() {
            match framework.framework {
                // Frontend frameworks
                Framework::React | Framework::NextJS | Framework::Vue | Framework::Angular => {
                    frontend_frameworks.push(&mut *framework);
                },
                // Backend frameworks
                Framework::Flask | Framework::FastAPI | Framework::Django | Framework::SpringBoot => {
                    backend_frameworks.push(&mut *framework);
                },
                // API/Middleware frameworks
                Framework::NestJS | Framework::Express | Framework::Danet => {
                    api_frameworks.push(&mut *framework);
                },
                _ => {} // Other frameworks
            }
        }
        
        // Detect frontend-backend communication patterns
        self.detect_api_communication_patterns(frameworks)?;
        
        // Adjust confidence scores based on ecosystem compatibility
        self.adjust_cross_framework_confidence(frameworks)?;
        
        Ok(())
    }
    
    /// Detect common fullstack architecture patterns
    fn detect_fullstack_patterns(&self, frameworks: &mut Vec<EnhancedDetectedFramework>) -> Result<(), Box<dyn std::error::Error>> {
        // Common patterns to detect:
        // 1. MERN (MongoDB, Express, React, Node.js)
        // 2. MEAN (MongoDB, Express, Angular, Node.js)
        // 3. Django + React
        // 4. Spring Boot + React
        // 5. FastAPI + React
        // 6. Next.js (full-stack React)
        
        let has_react = frameworks.iter().any(|f| f.framework == Framework::React);
        let has_nextjs = frameworks.iter().any(|f| f.framework == Framework::NextJS);
        let has_express = frameworks.iter().any(|f| f.framework == Framework::Express);
        let has_django = frameworks.iter().any(|f| f.framework == Framework::Django);
        let has_flask = frameworks.iter().any(|f| f.framework == Framework::Flask);
        let has_fastapi = frameworks.iter().any(|f| f.framework == Framework::FastAPI);
        let has_spring_boot = frameworks.iter().any(|f| f.framework == Framework::SpringBoot);
        
        // Detect MERN/MEAN stack patterns
        if has_react && has_express {
            self.boost_fullstack_confidence(frameworks, &[Framework::React, Framework::Express])?;
        }
        
        // Detect Django + React pattern
        if has_django && has_react {
            self.boost_fullstack_confidence(frameworks, &[Framework::Django, Framework::React])?;
        }
        
        // Detect FastAPI + React pattern
        if has_fastapi && has_react {
            self.boost_fullstack_confidence(frameworks, &[Framework::FastAPI, Framework::React])?;
        }
        
        // Detect Spring Boot + React pattern
        if has_spring_boot && has_react {
            self.boost_fullstack_confidence(frameworks, &[Framework::SpringBoot, Framework::React])?;
        }
        
        // Next.js is full-stack by nature
        if has_nextjs {
            for framework in frameworks.iter_mut() {
                if framework.framework == Framework::NextJS {
                    framework.confidence = (framework.confidence * 1.1).min(1.0);
                }
            }
        }
        
        Ok(())
    }
    
    /// Detect API communication patterns between frontend and backend
    fn detect_api_communication_patterns(&self, frameworks: &mut Vec<EnhancedDetectedFramework>) -> Result<(), Box<dyn std::error::Error>> {
        // Look for common API patterns
        let api_patterns = vec![
            "/api/",
            "/rest/",
            "fetch(",
            "axios.",
            "api.get",
            "api.post",
            "@RequestMapping",
            "@GetMapping",
            "@PostMapping",
            "app.get(",
            "app.post(",
            "router.get(",
            "router.post("
        ];
        
        if self.has_patterns_in_files(&["*.js", "*.ts", "*.jsx", "*.tsx", "*.py", "*.java"], &api_patterns)? {
            // Boost confidence for API-capable frameworks
            for framework in frameworks.iter_mut() {
                match framework.framework {
                    Framework::Express | Framework::NestJS | Framework::FastAPI | 
                    Framework::Flask | Framework::Django | Framework::SpringBoot | Framework::Danet => {
                        framework.confidence = (framework.confidence * 1.05).min(1.0);
                        framework.evidence.push(DetectionEvidence {
                            evidence_type: EvidenceType::ContentPattern,
                            source: "API communication patterns".to_string(),
                            pattern: "Cross-framework API integration".to_string(),
                            confidence_weight: 0.05,
                        });
                    },
                    _ => {}
                }
            }
        }
        
        Ok(())
    }
    
    /// Adjust confidence scores based on ecosystem compatibility
    fn adjust_cross_framework_confidence(&self, frameworks: &mut Vec<EnhancedDetectedFramework>) -> Result<(), Box<dyn std::error::Error>> {
        // Detect conflicting frameworks that shouldn't coexist
        let conflicting_pairs = vec![
            (Framework::Danet, Framework::NestJS), // Both are similar but different ecosystems
            (Framework::React, Framework::NextJS), // Next.js includes React, so shouldn't detect both
        ];
        
        for (framework1, framework2) in conflicting_pairs {
            let has_framework1 = frameworks.iter().any(|f| f.framework == framework1);
            let has_framework2 = frameworks.iter().any(|f| f.framework == framework2);
            
            if has_framework1 && has_framework2 {
                // Reduce confidence for the less confident framework
                let f1_confidence = frameworks.iter()
                    .find(|f| f.framework == framework1)
                    .map(|f| f.confidence)
                    .unwrap_or(0.0);
                let f2_confidence = frameworks.iter()
                    .find(|f| f.framework == framework2)
                    .map(|f| f.confidence)
                    .unwrap_or(0.0);
                
                let weaker_framework = if f1_confidence < f2_confidence { framework1 } else { framework2 };
                
                for framework in frameworks.iter_mut() {
                    if framework.framework == weaker_framework {
                        framework.confidence *= 0.7; // Reduce confidence by 30%
                        framework.evidence.push(DetectionEvidence {
                            evidence_type: EvidenceType::ContentPattern,
                            source: "Cross-framework analysis".to_string(),
                            pattern: "Potential framework conflict detected".to_string(),
                            confidence_weight: -0.3,
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Boost confidence for well-known fullstack patterns
    fn boost_fullstack_confidence(&self, frameworks: &mut Vec<EnhancedDetectedFramework>, pattern_frameworks: &[Framework]) -> Result<(), Box<dyn std::error::Error>> {
        for target_framework in pattern_frameworks {
            for framework in frameworks.iter_mut() {
                if framework.framework == *target_framework {
                    framework.confidence = (framework.confidence * 1.1).min(1.0);
                    framework.evidence.push(DetectionEvidence {
                        evidence_type: EvidenceType::ContentPattern,
                        source: "Fullstack pattern detection".to_string(),
                        pattern: format!("Part of common fullstack pattern: {:?}", pattern_frameworks),
                        confidence_weight: 0.1,
                    });
                }
            }
        }
        Ok(())
    }
}

// Convert to legacy DetectedFramework for compatibility
// TODO: Re-enable when DetectedFramework is available
// impl From<EnhancedDetectedFramework> for DetectedFramework {
//     fn from(enhanced: EnhancedDetectedFramework) -> Self {
//         DetectedFramework {
//             name: format!("{:?}", enhanced.framework),
//             version: enhanced.version,
//             confidence: enhanced.confidence,
//             evidence: enhanced.evidence.into_iter().map(|e| e.pattern).collect(),
//             usage_extent: enhanced.usage_extent,
//         }
//     }
// }