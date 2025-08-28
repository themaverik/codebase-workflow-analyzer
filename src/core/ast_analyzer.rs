use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use tree_sitter::{Language, Parser, Tree, Node};
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use crate::core::types::{Framework, Language as AnalysisLanguage};
use crate::core::extractors::{TypeScriptExtractor, PythonExtractor, JavaExtractor};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSegment {
    pub segment_type: SegmentType,
    pub content: String,
    pub metadata: SegmentMetadata,
    pub framework_context: Option<Framework>,
    pub business_hints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SegmentType {
    Function(FunctionSegment),
    Class(ClassSegment),
    Interface(InterfaceSegment),
    Route(RouteSegment),
    Configuration(ConfigSegment),
    Database(DatabaseSegment),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSegment {
    pub name: String,
    pub parameters: Vec<String>,
    pub return_type: Option<String>,
    pub is_async: bool,
    pub decorators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassSegment {
    pub name: String,
    pub extends: Option<String>,
    pub implements: Vec<String>,
    pub is_react_component: bool,
    pub props: Vec<String>,
    pub hooks: Vec<String>,
    pub methods: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceSegment {
    pub name: String,
    pub extends: Vec<String>,
    pub properties: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteSegment {
    pub path: String,
    pub method: String,
    pub handler: String,
    pub middleware: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSegment {
    pub config_type: String,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSegment {
    pub model_name: String,
    pub table_name: Option<String>,
    pub fields: Vec<String>,
    pub relationships: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentMetadata {
    pub line_start: usize,
    pub line_end: usize,
    pub file_path: PathBuf,
    pub byte_start: usize,
    pub byte_end: usize,
}

pub struct ASTAnalyzer {
    parsers: HashMap<AnalysisLanguage, Parser>,
    extractors: HashMap<AnalysisLanguage, Box<dyn SegmentExtractor>>,
}

pub trait SegmentExtractor: Send + Sync {
    fn extract_segments(&self, source: &str, file_path: &Path) -> Result<Vec<CodeSegment>>;
    fn extract_business_hints(&self, node: &Node, source: &str) -> Vec<String>;
}

impl ASTAnalyzer {
    pub fn new() -> Result<Self> {
        let mut parsers = HashMap::new();
        let mut extractors: HashMap<AnalysisLanguage, Box<dyn SegmentExtractor>> = HashMap::new();

        // Initialize TypeScript parser
        let mut ts_parser = Parser::new();
        ts_parser.set_language(tree_sitter_typescript::language_typescript())
            .context("Failed to set TypeScript language")?;
        parsers.insert(AnalysisLanguage::TypeScript, ts_parser);
        extractors.insert(AnalysisLanguage::TypeScript, Box::new(TypeScriptExtractor::new()));

        // Initialize Python parser
        let mut py_parser = Parser::new();
        py_parser.set_language(tree_sitter_python::language())
            .context("Failed to set Python language")?;
        parsers.insert(AnalysisLanguage::Python, py_parser);
        extractors.insert(AnalysisLanguage::Python, Box::new(PythonExtractor::new()));

        // Initialize Java parser
        let mut java_parser = Parser::new();
        java_parser.set_language(tree_sitter_java::language())
            .context("Failed to set Java language")?;
        parsers.insert(AnalysisLanguage::Java, java_parser);
        extractors.insert(AnalysisLanguage::Java, Box::new(JavaExtractor::new()));

        // Initialize JavaScript parser (use TypeScript extractor for JS files)
        let mut js_parser = Parser::new();
        js_parser.set_language(tree_sitter_javascript::language())
            .context("Failed to set JavaScript language")?;
        parsers.insert(AnalysisLanguage::JavaScript, js_parser);
        extractors.insert(AnalysisLanguage::JavaScript, Box::new(TypeScriptExtractor::new()));

        Ok(ASTAnalyzer {
            parsers,
            extractors,
        })
    }

    pub fn extract_segments(&mut self, codebase_path: &Path, detected_frameworks: &[Framework]) -> Result<Vec<CodeSegment>> {
        let mut all_segments = Vec::new();
        
        for entry in walkdir::WalkDir::new(codebase_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path();
            let language = self.detect_file_language(file_path)?;
            
            if let Some(lang) = language {
                let content = fs::read_to_string(file_path)
                    .context(format!("Failed to read file: {}", file_path.display()))?;
                
                if let Some(parser) = self.parsers.get_mut(&lang) {
                    if let Some(tree) = parser.parse(&content, None) {
                        if let Some(extractor) = self.extractors.get(&lang) {
                            let mut segments = extractor.extract_segments(&content, file_path)?;
                            
                            // Add framework context to segments
                            for segment in &mut segments {
                                segment.framework_context = self.infer_framework_context(
                                    &segment,
                                    detected_frameworks,
                                    file_path
                                );
                            }
                            
                            all_segments.extend(segments);
                        }
                    }
                }
            }
        }
        
        Ok(all_segments)
    }

    fn detect_file_language(&self, file_path: &Path) -> Result<Option<AnalysisLanguage>> {
        if let Some(extension) = file_path.extension().and_then(|e| e.to_str()) {
            match extension {
                "ts" | "tsx" => Ok(Some(AnalysisLanguage::TypeScript)),
                "js" | "jsx" => Ok(Some(AnalysisLanguage::JavaScript)),
                "py" => Ok(Some(AnalysisLanguage::Python)),
                "java" => Ok(Some(AnalysisLanguage::Java)),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    fn infer_framework_context(&self, segment: &CodeSegment, detected_frameworks: &[Framework], file_path: &Path) -> Option<Framework> {
        let file_path_str = file_path.to_string_lossy().to_lowercase();
        let content_lower = segment.content.to_lowercase();
        
        // Infer framework based on file path and content patterns
        for framework in detected_frameworks {
            match framework {
                Framework::React => {
                    if file_path_str.contains("component") || 
                       content_lower.contains("usestate") ||
                       content_lower.contains("useeffect") ||
                       content_lower.contains("jsx") {
                        return Some(Framework::React);
                    }
                }
                Framework::NestJS => {
                    if content_lower.contains("@controller") ||
                       content_lower.contains("@injectable") ||
                       content_lower.contains("@module") {
                        return Some(Framework::NestJS);
                    }
                }
                Framework::Flask => {
                    if content_lower.contains("@app.route") ||
                       content_lower.contains("flask") {
                        return Some(Framework::Flask);
                    }
                }
                Framework::FastAPI => {
                    if content_lower.contains("@app.get") ||
                       content_lower.contains("fastapi") {
                        return Some(Framework::FastAPI);
                    }
                }
                Framework::SpringBoot => {
                    if content_lower.contains("@restcontroller") ||
                       content_lower.contains("@service") ||
                       content_lower.contains("@component") {
                        return Some(Framework::SpringBoot);
                    }
                }
                _ => {}
            }
        }
        
        None
    }

    pub fn get_segment_statistics(&self, segments: &[CodeSegment]) -> SegmentStatistics {
        let mut stats = SegmentStatistics::default();
        
        for segment in segments {
            match &segment.segment_type {
                SegmentType::Function(_) => stats.function_count += 1,
                SegmentType::Class(_) => stats.class_count += 1,
                SegmentType::Interface(_) => stats.interface_count += 1,
                SegmentType::Route(_) => stats.route_count += 1,
                SegmentType::Configuration(_) => stats.config_count += 1,
                SegmentType::Database(_) => stats.database_count += 1,
            }
            
            if segment.framework_context.is_some() {
                stats.framework_segments += 1;
            }
            
            stats.total_business_hints += segment.business_hints.len();
        }
        
        stats.total_segments = segments.len();
        stats
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SegmentStatistics {
    pub total_segments: usize,
    pub function_count: usize,
    pub class_count: usize,
    pub interface_count: usize,
    pub route_count: usize,
    pub config_count: usize,
    pub database_count: usize,
    pub framework_segments: usize,
    pub total_business_hints: usize,
}

