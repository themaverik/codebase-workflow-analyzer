use std::collections::HashMap;
use std::path::Path;
use std::fs;

use anyhow::{Context, Result};
use regex::Regex;

use crate::core::status_inference_engine::{
    StatusAnalyzer, StatusAnalysisResult, StatusInferenceConfig, CrudAnalysisConfig,
    MissingImplementation, PartialImplementation
};
use crate::core::documentation_extractor::ExtractedDocumentationInfo;
use crate::core::todo_scanner::TodoScanResult;

/// CRUD operation completeness analyzer
/// Detects partial CRUD implementations by analyzing database models, API endpoints, and frontend forms
pub struct CrudAnalyzer {
    config: CrudAnalysisConfig,
    weight: f32,
    crud_patterns: HashMap<String, Vec<Regex>>,
    entity_patterns: Vec<Regex>,
    route_patterns: Vec<Regex>,
}

#[derive(Debug, Clone)]
struct EntityDefinition {
    name: String,
    file_path: String,
    fields: Vec<String>,
    crud_operations: CrudOperations,
    confidence: f32,
}

#[derive(Debug, Clone)]
struct CrudOperations {
    create: CrudOperationStatus,
    read: CrudOperationStatus,
    update: CrudOperationStatus,
    delete: CrudOperationStatus,
}

#[derive(Debug, Clone)]
struct CrudOperationStatus {
    implemented: bool,
    evidence: Vec<String>,
    confidence: f32,
    implementation_type: Vec<String>, // database, api, frontend
}

#[derive(Debug, Clone)]
struct RouteDefinition {
    path: String,
    method: String,
    handler: String,
    entity: Option<String>,
    file_path: String,
}

impl CrudAnalyzer {
    pub fn new(config: CrudAnalysisConfig, weight: f32) -> Self {
        let mut analyzer = Self {
            config,
            weight,
            crud_patterns: HashMap::new(),
            entity_patterns: Vec::new(),
            route_patterns: Vec::new(),
        };

        analyzer.compile_patterns().expect("Failed to compile CRUD patterns");
        analyzer
    }

    /// Compile regex patterns for CRUD operation detection
    fn compile_patterns(&mut self) -> Result<()> {
        // CRUD operation patterns for different languages/frameworks
        
        // Create operation patterns
        let create_patterns = vec![
            r"(?i)\b(create|insert|add|save|post|new)\b.*\b(user|product|order|item|entity)\b",
            r"(?i)\b(User|Product|Order|Item)\.create\b",
            r"(?i)\.save\(\)",
            r"(?i)INSERT\s+INTO",
            r"(?i)@PostMapping|@Post|app\.post|router\.post",
            r"(?i)def\s+create_|function\s+create|createUser|createProduct",
        ];

        // Read operation patterns  
        let read_patterns = vec![
            r"(?i)\b(get|find|read|fetch|retrieve|select|show)\b.*\b(user|product|order|item|entity)\b",
            r"(?i)\b(User|Product|Order|Item)\.(find|get|where)\b",
            r"(?i)SELECT\s+.*\s+FROM",
            r"(?i)@GetMapping|@Get|app\.get|router\.get",
            r"(?i)def\s+get_|function\s+get|getUser|getProduct",
            r"(?i)\.findById|\.findOne|\.findAll",
        ];

        // Update operation patterns
        let update_patterns = vec![
            r"(?i)\b(update|edit|modify|patch|put)\b.*\b(user|product|order|item|entity)\b",
            r"(?i)\b(User|Product|Order|Item)\.update\b",
            r"(?i)UPDATE\s+.*\s+SET",
            r"(?i)@PutMapping|@PatchMapping|@Put|@Patch|app\.put|app\.patch",
            r"(?i)def\s+update_|function\s+update|updateUser|updateProduct",
            r"(?i)\.findByIdAndUpdate|\.updateOne|\.updateMany",
        ];

        // Delete operation patterns
        let delete_patterns = vec![
            r"(?i)\b(delete|remove|destroy)\b.*\b(user|product|order|item|entity)\b",
            r"(?i)\b(User|Product|Order|Item)\.(delete|destroy|remove)\b",
            r"(?i)DELETE\s+FROM",
            r"(?i)@DeleteMapping|@Delete|app\.delete|router\.delete",
            r"(?i)def\s+delete_|function\s+delete|deleteUser|deleteProduct",
            r"(?i)\.findByIdAndDelete|\.deleteOne|\.deleteMany",
        ];

        // Compile patterns
        for pattern_str in create_patterns {
            let regex = Regex::new(pattern_str)
                .context(format!("Failed to compile CREATE pattern: {}", pattern_str))?;
            self.crud_patterns.entry("create".to_string())
                .or_insert_with(Vec::new)
                .push(regex);
        }

        for pattern_str in read_patterns {
            let regex = Regex::new(pattern_str)
                .context(format!("Failed to compile READ pattern: {}", pattern_str))?;
            self.crud_patterns.entry("read".to_string())
                .or_insert_with(Vec::new)
                .push(regex);
        }

        for pattern_str in update_patterns {
            let regex = Regex::new(pattern_str)
                .context(format!("Failed to compile UPDATE pattern: {}", pattern_str))?;
            self.crud_patterns.entry("update".to_string())
                .or_insert_with(Vec::new)
                .push(regex);
        }

        for pattern_str in delete_patterns {
            let regex = Regex::new(pattern_str)
                .context(format!("Failed to compile DELETE pattern: {}", pattern_str))?;
            self.crud_patterns.entry("delete".to_string())
                .or_insert_with(Vec::new)
                .push(regex);
        }

        // Entity definition patterns
        let entity_pattern_strs = vec![
            r"(?i)class\s+(\w+).*\b(entity|model|schema)\b",
            r"(?i)@Entity.*class\s+(\w+)",
            r"(?i)interface\s+(\w+)\s*\{",
            r"(?i)type\s+(\w+)\s*=",
            r"(?i)model\s+(\w+)",
            r"(?i)CREATE\s+TABLE\s+(\w+)",
        ];

        for pattern_str in entity_pattern_strs {
            let regex = Regex::new(pattern_str)
                .context(format!("Failed to compile entity pattern: {}", pattern_str))?;
            self.entity_patterns.push(regex);
        }

        // Route definition patterns
        let route_pattern_strs = vec![
            "(?i)@(Get|Post|Put|Patch|Delete)Mapping\\s*\\(\\s*[\"']([^\"']*)[\"']\\s*\\)",
            "(?i)(app|router)\\.(get|post|put|patch|delete)\\s*\\(\\s*[\"']([^\"']*)[\"']\\s*,",
            "(?i)Route::(get|post|put|patch|delete)\\s*\\(\\s*[\"']([^\"']*)[\"']\\s*,",
        ];

        for pattern_str in route_pattern_strs {
            let regex = Regex::new(pattern_str)
                .context(format!("Failed to compile route pattern: {}", pattern_str))?;
            self.route_patterns.push(regex);
        }

        println!("Compiled {} CRUD operation patterns, {} entity patterns, {} route patterns",
                 self.crud_patterns.values().map(|v| v.len()).sum::<usize>(),
                 self.entity_patterns.len(),
                 self.route_patterns.len());

        Ok(())
    }

    /// Analyze project for CRUD operation completeness
    fn analyze_crud_completeness(&self, project_path: &Path) -> Result<Vec<EntityDefinition>> {
        let mut entities = Vec::new();

        // Find entity definitions
        let entity_files = self.find_entity_files(project_path)?;
        
        for file_path in entity_files {
            if let Ok(content) = fs::read_to_string(&file_path) {
                let mut file_entities = self.extract_entities_from_file(&file_path, &content)?;
                entities.extend(file_entities.drain(..));
            }
        }

        // Analyze CRUD operations for each entity
        for entity in &mut entities {
            self.analyze_entity_crud_operations(project_path, entity)?;
        }

        Ok(entities)
    }

    /// Find files that likely contain entity definitions
    fn find_entity_files(&self, project_path: &Path) -> Result<Vec<std::path::PathBuf>> {
        let mut entity_files = Vec::new();

        // Common patterns for entity/model files
        let patterns = vec![
            "**/models/**/*.{ts,js,java,py,rs}",
            "**/entities/**/*.{ts,js,java,py,rs}",
            "**/schema/**/*.{ts,js,java,py,rs}",
            "**/domain/**/*.{ts,js,java,py,rs}",
            "**/*Model.{ts,js,java,py}",
            "**/*Entity.{ts,js,java,py}",
            "**/*Schema.{ts,js,java,py}",
        ];

        // Simple recursive directory traversal (would use glob in production)
        self.traverse_directory(project_path, &mut entity_files, &patterns)?;

        Ok(entity_files)
    }

    /// Recursively traverse directory looking for entity files
    fn traverse_directory(
        &self, 
        dir: &Path, 
        files: &mut Vec<std::path::PathBuf>,
        _patterns: &[&str]
    ) -> Result<()> {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Skip common ignore patterns
                    if let Some(name) = path.file_name() {
                        if name == "node_modules" || name == "target" || name == ".git" {
                            continue;
                        }
                    }
                    self.traverse_directory(&path, files, _patterns)?;
                } else if path.is_file() {
                    // Check file extensions
                    if let Some(extension) = path.extension() {
                        if matches!(extension.to_str(), Some("ts") | Some("js") | Some("java") | Some("py") | Some("rs")) {
                            // Check if filename suggests entity/model
                            if let Some(filename) = path.file_name() {
                                let filename_str = filename.to_string_lossy().to_lowercase();
                                if filename_str.contains("model") || 
                                   filename_str.contains("entity") || 
                                   filename_str.contains("schema") ||
                                   path.to_string_lossy().contains("/models/") ||
                                   path.to_string_lossy().contains("/entities/") {
                                    files.push(path);
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Extract entity definitions from a file
    fn extract_entities_from_file(
        &self, 
        file_path: &Path, 
        content: &str
    ) -> Result<Vec<EntityDefinition>> {
        let mut entities = Vec::new();

        for pattern in &self.entity_patterns {
            for captures in pattern.captures_iter(content) {
                if let Some(entity_name) = captures.get(1) {
                    let entity = EntityDefinition {
                        name: entity_name.as_str().to_string(),
                        file_path: file_path.to_string_lossy().to_string(),
                        fields: self.extract_entity_fields(content, entity_name.as_str()),
                        crud_operations: CrudOperations {
                            create: CrudOperationStatus::default(),
                            read: CrudOperationStatus::default(),
                            update: CrudOperationStatus::default(),
                            delete: CrudOperationStatus::default(),
                        },
                        confidence: 0.8, // Base confidence for entity detection
                    };
                    entities.push(entity);
                }
            }
        }

        Ok(entities)
    }

    /// Extract field names from entity definition
    fn extract_entity_fields(&self, content: &str, entity_name: &str) -> Vec<String> {
        let mut fields = Vec::new();
        
        // Simple field extraction patterns
        let field_patterns = vec![
            format!(r"(?i)class\s+{}\s*\{{([^}}]+)}}", regex::escape(entity_name)),
            format!(r"(?i)interface\s+{}\s*\{{([^}}]+)}}", regex::escape(entity_name)),
        ];

        for pattern_str in field_patterns {
            if let Ok(pattern) = Regex::new(&pattern_str) {
                if let Some(captures) = pattern.captures(content) {
                    if let Some(class_body) = captures.get(1) {
                        // Extract field names (simplified)
                        let field_pattern = Regex::new(r"(\w+)\s*:\s*\w+").unwrap();
                        for field_match in field_pattern.captures_iter(class_body.as_str()) {
                            if let Some(field_name) = field_match.get(1) {
                                fields.push(field_name.as_str().to_string());
                            }
                        }
                    }
                }
            }
        }

        fields
    }

    /// Analyze CRUD operations for a specific entity
    fn analyze_entity_crud_operations(
        &self,
        project_path: &Path,
        entity: &mut EntityDefinition,
    ) -> Result<()> {
        // Scan project files for CRUD operations related to this entity
        let mut source_files = Vec::new();
        self.find_source_files(project_path, &mut source_files)?;

        for operation in &self.config.required_operations {
            let mut operation_status = CrudOperationStatus::default();
            
            if let Some(patterns) = self.crud_patterns.get(operation) {
                for file_path in &source_files {
                    if let Ok(content) = fs::read_to_string(file_path) {
                        for pattern in patterns {
                            if let Some(matches) = pattern.find(&content) {
                                // Check if this match is related to our entity
                                if self.is_operation_related_to_entity(&content, matches.as_str(), &entity.name) {
                                    operation_status.implemented = true;
                                    operation_status.evidence.push(format!(
                                        "Found {} operation in {}: {}",
                                        operation,
                                        file_path.display(),
                                        matches.as_str().trim()
                                    ));
                                    operation_status.confidence += 0.2;
                                    
                                    // Determine implementation type
                                    let file_path_str = file_path.to_string_lossy();
                                    if file_path_str.contains("controller") || file_path_str.contains("route") {
                                        operation_status.implementation_type.push("api".to_string());
                                    } else if file_path_str.contains("service") || file_path_str.contains("repository") {
                                        operation_status.implementation_type.push("database".to_string());
                                    } else if file_path_str.contains("component") || file_path_str.contains("page") {
                                        operation_status.implementation_type.push("frontend".to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }

            operation_status.confidence = operation_status.confidence.min(1.0);

            // Assign to appropriate CRUD operation
            match operation.as_str() {
                "create" => entity.crud_operations.create = operation_status,
                "read" => entity.crud_operations.read = operation_status,
                "update" => entity.crud_operations.update = operation_status,
                "delete" => entity.crud_operations.delete = operation_status,
                _ => {}
            }
        }

        Ok(())
    }

    /// Check if a CRUD operation match is related to a specific entity
    fn is_operation_related_to_entity(&self, content: &str, operation_text: &str, entity_name: &str) -> bool {
        let entity_variations = vec![
            entity_name.to_lowercase(),
            format!("{}s", entity_name.to_lowercase()),
            entity_name.to_string(),
            format!("{}s", entity_name),
        ];

        // Check if entity name appears near the operation
        let context_window = 200; // Characters before and after
        if let Some(match_pos) = content.find(operation_text) {
            let start = match_pos.saturating_sub(context_window);
            let end = (match_pos + operation_text.len() + context_window).min(content.len());
            let context = &content[start..end];

            for variation in entity_variations {
                if context.to_lowercase().contains(&variation) {
                    return true;
                }
            }
        }

        false
    }

    /// Find source code files to analyze
    fn find_source_files(&self, project_path: &Path, files: &mut Vec<std::path::PathBuf>) -> Result<()> {
        if let Ok(entries) = fs::read_dir(project_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Skip common ignore patterns
                    if let Some(name) = path.file_name() {
                        if name == "node_modules" || name == "target" || name == ".git" {
                            continue;
                        }
                    }
                    self.find_source_files(&path, files)?;
                } else if path.is_file() {
                    if let Some(extension) = path.extension() {
                        if matches!(extension.to_str(), Some("ts") | Some("js") | Some("java") | Some("py") | Some("rs")) {
                            files.push(path);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Generate analysis results from entity definitions
    fn generate_analysis_results(&self, entities: Vec<EntityDefinition>) -> StatusAnalysisResult {
        let incomplete_features = Vec::new();
        let mut missing_implementations = Vec::new();
        let mut partial_implementations = Vec::new();
        let mut evidence = Vec::new();
        
        for entity in &entities {
            let operations = vec![
                ("create", &entity.crud_operations.create),
                ("read", &entity.crud_operations.read),
                ("update", &entity.crud_operations.update),
                ("delete", &entity.crud_operations.delete),
            ];

            let implemented_ops: Vec<_> = operations.iter()
                .filter(|(_, op)| op.implemented)
                .map(|(name, _)| name.to_string())
                .collect();

            let missing_ops: Vec<_> = operations.iter()
                .filter(|(_, op)| !op.implemented)
                .map(|(name, _)| name.to_string())
                .collect();

            if !missing_ops.is_empty() {
                let completeness = implemented_ops.len() as f32 / 4.0; // 4 CRUD operations

                if completeness < self.config.partial_implementation_threshold {
                    // Mostly incomplete - treat as missing implementation
                    missing_implementations.push(MissingImplementation {
                        implementation_type: "CRUD Operations".to_string(),
                        expected_location: format!("Entity: {}", entity.name),
                        description: format!("CRUD operations for {} entity are mostly missing. Only {} implemented: {:?}",
                                           entity.name, implemented_ops.len(), implemented_ops),
                        confidence: entity.confidence,
                        evidence: operations.iter()
                            .flat_map(|(_, op)| op.evidence.iter())
                            .cloned()
                            .collect(),
                        suggested_implementation: Some(format!(
                            "Implement missing {} operations for {} entity",
                            missing_ops.len(), entity.name
                        )),
                    });
                } else {
                    // Partially implemented
                    partial_implementations.push(PartialImplementation {
                        implementation_name: format!("{} CRUD Operations", entity.name),
                        implemented_parts: implemented_ops.clone(),
                        missing_parts: missing_ops,
                        completeness_percentage: completeness * 100.0,
                        category: "CRUD".to_string(),
                        evidence: operations.iter()
                            .flat_map(|(_, op)| op.evidence.iter())
                            .cloned()
                            .collect(),
                        next_steps: vec![
                            format!("Implement missing CRUD operations for {}", entity.name),
                            "Add proper error handling for CRUD operations".to_string(),
                            "Add validation for CRUD operations".to_string(),
                        ],
                    });
                }

                evidence.push(format!("Entity {} has incomplete CRUD operations: {} of 4 implemented", 
                                    entity.name, implemented_ops.len()));
            } else {
                // Complete CRUD implementation
                evidence.push(format!("Entity {} has complete CRUD operations", entity.name));
            }
        }

        // Calculate overall confidence
        let confidence = if entities.is_empty() {
            0.0
        } else {
            entities.iter().map(|e| e.confidence).sum::<f32>() / entities.len() as f32
        };

        StatusAnalysisResult {
            analyzer_name: self.name().to_string(),
            incomplete_features,
            missing_implementations,
            partial_implementations,
            confidence,
            evidence,
            analysis_notes: vec![
                format!("Analyzed {} entities for CRUD completeness", entities.len()),
                format!("Used {} CRUD operation patterns", self.crud_patterns.values().map(|v| v.len()).sum::<usize>()),
                "CRUD analysis focuses on Create, Read, Update, Delete operation completeness".to_string(),
            ],
        }
    }
}

impl Default for CrudOperationStatus {
    fn default() -> Self {
        Self {
            implemented: false,
            evidence: Vec::new(),
            confidence: 0.0,
            implementation_type: Vec::new(),
        }
    }
}

impl StatusAnalyzer for CrudAnalyzer {
    fn name(&self) -> &str { 
        "CRUD Analyzer" 
    }
    
    fn weight(&self) -> f32 { 
        self.weight 
    }
    
    fn can_analyze(&self, project_path: &Path) -> bool {
        // Check if project has entity/model files
        let model_patterns = vec![
            project_path.join("models"),
            project_path.join("entities"),
            project_path.join("src/models"),
            project_path.join("src/entities"),
        ];

        model_patterns.iter().any(|path| path.exists()) ||
        // Or has files that suggest entities
        self.find_entity_files(project_path).unwrap_or_default().len() > 0
    }
    
    fn analyze(
        &self,
        project_path: &Path,
        _documentation: &ExtractedDocumentationInfo,
        _todo_results: &TodoScanResult,
        _config: &StatusInferenceConfig,
    ) -> Result<StatusAnalysisResult> {
        println!("Analyzing CRUD operations for project: {}", project_path.display());
        
        let entities = self.analyze_crud_completeness(project_path)
            .context("Failed to analyze CRUD completeness")?;
        
        let result = self.generate_analysis_results(entities);
        
        println!("CRUD analysis completed: {} items found, {:.1}% confidence",
                 result.missing_implementations.len() + result.partial_implementations.len(),
                 result.confidence * 100.0);
        
        Ok(result)
    }
}