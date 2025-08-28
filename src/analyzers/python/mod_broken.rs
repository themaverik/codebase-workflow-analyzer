use crate::core::{
    CodebaseAnalysis, CodebaseAnalyzer, ComponentInfo, ComponentType, PropInfo, ApiCall,
    ImplementationStatus, UserStory, Priority, Complexity, ProductRequirementDocument,
    FeatureDescription, Task, TaskType, AnalysisMetadata, ProjectType, AnalyzerConfig
};
use regex::Regex;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use anyhow::Result;

pub struct PythonAnalyzer {
    config: AnalyzerConfig,
    framework_type: PythonFramework,
}

#[derive(Debug, Clone)]
pub enum PythonFramework {
    Django,
    Flask,
    Unknown,
}

impl PythonAnalyzer {
    pub fn new(config: AnalyzerConfig) -> Self {
        Self { 
            config,
            framework_type: PythonFramework::Unknown,
        }
    }

    fn detect_framework(&mut self, project_path: &str) -> PythonFramework {
        // Check requirements.txt or setup.py
        let req_path = Path::new(project_path).join("requirements.txt");
        if let Ok(content) = fs::read_to_string(req_path) {
            if content.to_lowercase().contains("django") {
                self.framework_type = PythonFramework::Django;
                return PythonFramework::Django;
            }
            if content.to_lowercase().contains("flask") {
                self.framework_type = PythonFramework::Flask;
                return PythonFramework::Flask;
            }
        }

        // Check for Django settings
        for entry in WalkDir::new(project_path).max_depth(3).into_iter().filter_map(|e| e.ok()) {
            if entry.file_name().to_str() == Some("settings.py") {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if content.contains("DJANGO_SETTINGS_MODULE") || content.contains("django.conf") {
                        self.framework_type = PythonFramework::Django;
                        return PythonFramework::Django;
                    }
                }
            }
        }

        // Check for Flask app
        for entry in WalkDir::new(project_path).max_depth(3).into_iter().filter_map(|e| e.ok()) {
            if let Some(ext) = entry.path().extension() {
                if ext == "py" {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if content.contains("from flask import") || content.contains("Flask(__name__)") {
                            self.framework_type = PythonFramework::Flask;
                            return PythonFramework::Flask;
                        }
                    }
                }
            }
        }

        PythonFramework::Unknown
    }

    fn is_framework_component(&self, content: &str) -> bool {
        match self.framework_type {
            PythonFramework::Django => {
                let django_patterns = [
                    r"from django",
                    r"import django",
                    r"class.*\(.*Model\)",
                    r"class.*\(.*View\)",
                    r"def.*\(request",
                    r"@api_view",
                    r"Django",
                ];
                django_patterns.iter().any(|pattern| {
                    Regex::new(pattern).unwrap().is_match(content)
                })
            },
            PythonFramework::Flask => {
                let flask_patterns = [
                    r"from flask import",
                    r"import flask",
                    r"@app\.route",
                    r"Flask\(__name__\)",
                    r"request\.",
                    r"jsonify",
                ];
                flask_patterns.iter().any(|pattern| {
                    Regex::new(pattern).unwrap().is_match(content)
                })
            },
            PythonFramework::Unknown => {
                content.contains("def ") || content.contains("class ")
            }
        }
    }

    fn extract_component_info(&self, file_path: &str, content: &str) -> Option<ComponentInfo> {
        if !file_path.ends_with(".py") {
            return None;
        }

        let component_name = self.extract_main_class_or_module(file_path, content)?;
        let component_type = self.classify_component_type(&component_name, content);
        let purpose = self.infer_component_purpose(&component_name, content);
        let props = self.extract_fields_and_params(content);
        let dependencies = self.extract_imports(content);
        let api_calls = self.extract_routes_and_calls(content);
        let complexity_score = self.calculate_complexity(content);
        let implementation_status = self.determine_implementation_status(content);

        Some(ComponentInfo {
            name: component_name,
            file_path: file_path.to_string(),
                        priority: Priority::Medium,
            component_type,
            purpose,
            dependencies,
            props,
            hooks_used: vec![], // Python doesn't have hooks
            api_calls,
            complexity_score,
            implementation_status,
        })
    }

    fn extract_main_class_or_module(&self, file_path: &str, content: &str) -> Option<String> {
        // Try to find main class
        if let Ok(re) = Regex::new(r"class\s+(\w+)") {
            let classes: Vec<_> = re.captures_iter(content)
                .map(|cap| cap.get(1).unwrap().as_str().to_string())
                .collect();
            
            // Prefer View, Model, or Form classes
            for class_name in &classes {
                let lower = class_name.to_lowercase();
                if lower.contains("view") || lower.contains("model") || 
                   lower.contains("form") || lower.contains("serializer") {
                    return Some(class_name.clone());
                }
            }
            
            // Return first class if any
            if !classes.is_empty() {
                return Some(classes[0].clone());
            }
        }

        // Fallback to module name
        Path::new(file_path)
            .file_stem()
            .and_then(|name| name.to_str())
            .map(|name| name.replace("_", " ").to_string())
    }

    fn classify_component_type(&self, name: &str, content: &str) -> ComponentType {
        let name_lower = name.to_lowercase();
        
        match self.framework_type {
            PythonFramework::Django => {
                if content.contains("(models.Model)") || name_lower.contains("model") {
                    return ComponentType::Context; // Data models
                }
                if content.contains("(View)") || content.contains("@api_view") || 
                   name_lower.contains("view") || content.contains("def get(") {
                    return ComponentType::Service; // Views handle requests
                }
                if name_lower.contains("form") || content.contains("forms.") {
                    return ComponentType::Form;
                }
                if name_lower.contains("admin") {
                    return ComponentType::Utility;
                }
                if name_lower.contains("serializer") {
                    return ComponentType::Service;
                }
            },
            PythonFramework::Flask => {
                if content.contains("@app.route") || content.contains("@bp.route") {
                    return ComponentType::Service; // Route handlers
                }
                if name_lower.contains("model") || content.contains("db.Model") {
                    return ComponentType::Context;
                }
                if name_lower.contains("form") {
                    return ComponentType::Form;
                }
            },
            _ => {}
        }

        // Generic classification
        if name_lower.contains("view") || name_lower.contains("controller") {
            return ComponentType::Service;
        }
        if name_lower.contains("model") || name_lower.contains("entity") {
            return ComponentType::Context;
        }
        if name_lower.contains("form") {
            return ComponentType::Form;
        }
        if name_lower.contains("util") || name_lower.contains("helper") {
            return ComponentType::Utility;
        }

        ComponentType::Service
    }

    fn infer_component_purpose(&self, name: &str, content: &str) -> String {
        let name_lower = name.to_lowercase();
        
        match self.framework_type {
            PythonFramework::Django => {
                if content.contains("(models.Model)") {
                    return format!("Django model representing {} data structure", name_lower);
                }
                if content.contains("@api_view") || content.contains("(View)") {
                    return format!("Django view handling HTTP requests for {}", name_lower);
                }
                if name_lower.contains("serializer") {
                    return format!("Django REST serializer for {} data", name_lower);
                }
                if name_lower.contains("admin") {
                    return format!("Django admin configuration for {}", name_lower);
                }
            },
            PythonFramework::Flask => {
                if content.contains("@app.route") {
                    return format!("Flask route handler for {} endpoints", name_lower);
                }
                if content.contains("db.Model") {
                    return format!("SQLAlchemy model for {} data", name_lower);
                }
            },
            _ => {}
        }

        // Generic purposes
        if name_lower.contains("view") {
            return format!("View component handling {} functionality", name_lower);
        }
        if name_lower.contains("model") {
            return format!("Data model for {} entity", name_lower);
        }
        if name_lower.contains("form") {
            return format!("Form handling {} input", name_lower);
        }
        if name_lower.contains("service") {
            return format!("Service providing {} functionality", name_lower);
        }

        format!("Python component for {} functionality", name_lower)
    }

    fn extract_fields_and_params(&self, content: &str) -> Vec<PropInfo> {
        let mut fields = Vec::new();
        
        // Django model fields
        if let Ok(re) = Regex::new(r"(\w+)\s*=\s*models\.(\w+Field)") {
            for captures in re.captures_iter(content) {
                let field_name = captures.get(1).unwrap().as_str().to_string();
                let field_type = captures.get(2).unwrap().as_str().to_string();
                
                fields.push(PropInfo {
                    name: field_name,
                    prop_type: field_type,
                    required: !content.contains("null=True"), // Simplified
                    description: None,
                });
            }
        }

        // Function parameters
        if let Ok(re) = Regex::new(r"def\s+\w+\s*\([^)]*?(\w+):\s*(\w+)") {
            for captures in re.captures_iter(content) {
                let param_name = captures.get(1).unwrap().as_str().to_string();
                let param_type = captures.get(2).unwrap().as_str().to_string();
                
                if param_name != "self" && param_name != "request" {
                    fields.push(PropInfo {
                        name: param_name,
                        prop_type: param_type,
                        required: true,
                        description: Some("Function parameter".to_string()),
                        priority: Priority::Medium,
                    });
                }
            }
        }

        // Class attributes
        if let Ok(re) = Regex::new(r"self\.(\w+)\s*=") {
            for captures in re.captures_iter(content) {
                let attr_name = captures.get(1).unwrap().as_str().to_string();
                if !fields.iter().any(|f| f.name == attr_name) {
                    fields.push(PropInfo {
                        name: attr_name,
                        prop_type: "Any".to_string(),
                        priority: Priority::Medium,
                        required: true,
                        description: Some("Instance attribute".to_string()),
                        priority: Priority::Medium,
                    });
                }
            }
        }
        
        fields
    }

    fn extract_routes_and_calls(&self, content: &str) -> Vec<ApiCall> {
        let mut api_calls = Vec::new();
        
        match self.framework_type {
            PythonFramework::Django => {
                // URL patterns and API views
                if let Ok(re) = Regex::new(r#"@api_view\s*\(\s*\[['"](\w+)['"]\]"#) {
                    for captures in re.captures_iter(content) {
                        let method = captures.get(1).unwrap().as_str().to_string();
                        api_calls.push(ApiCall {
                            endpoint: "API endpoint".to_string(),
                        priority: Priority::Medium,
                            method,
                            purpose: "Django REST API view".to_string(),
                        priority: Priority::Medium,
                        });
                    }
                }

                // External API calls
                if let Ok(re) = Regex::new(r#"requests\.(\w+)\s*\([^,]*['"]([^'"]+)['"]"#) {
                    for captures in re.captures_iter(content) {
                        let method = captures.get(1).unwrap().as_str().to_uppercase();
                        let url = captures.get(2).unwrap().as_str().to_string();
                        api_calls.push(ApiCall {
                            endpoint: url,
                            method,
                            purpose: "External API call".to_string(),
                        priority: Priority::Medium,
                        });
                    }
                }
            },
            PythonFramework::Flask => {
                // Flask routes
                if let Ok(re) = Regex::new(r#"@(?:app|bp)\.route\s*\(\s*['"]([^'"]+)['"][^)]*methods\s*=\s*\[['"](\w+)['"]\]"#) {
                    for captures in re.captures_iter(content) {
                        let route = captures.get(1).unwrap().as_str().to_string();
                        let method = captures.get(2).unwrap().as_str().to_string();
                        api_calls.push(ApiCall {
                            endpoint: route,
                            method,
                            purpose: "Flask route handler".to_string(),
                        priority: Priority::Medium,
                        });
                    }
                }

                // Simple routes without explicit methods
                if let Ok(re) = Regex::new(r#"@(?:app|bp)\.route\s*\(\s*['"]([^'"]+)['"]"#) {
                    for captures in re.captures_iter(content) {
                        let route = captures.get(1).unwrap().as_str().to_string();
                        if !api_calls.iter().any(|call| call.endpoint == route) {
                            api_calls.push(ApiCall {
                                endpoint: route,
                                method: "GET".to_string(),
                        priority: Priority::Medium,
                                purpose: "Flask route handler".to_string(),
                        priority: Priority::Medium,
                            });
                        }
                    }
                }
            },
            _ => {}
        }

        api_calls
    }

    fn extract_imports(&self, content: &str) -> Vec<String> {
        let mut imports = Vec::new();
        
        // Standard imports
        if let Ok(re) = Regex::new(r"(?:from\s+([\w.]+)\s+import|import\s+([\w.]+))") {
            for captures in re.captures_iter(content) {
                let import = captures.get(1)
                    .or_else(|| captures.get(2))
                    .unwrap()
                    .as_str()
                    .to_string();
                
                if import.starts_with("django") || import.starts_with("flask") || 
                   import.starts_with("requests") || import.starts_with("sqlalchemy") {
                    if !imports.contains(&import) {
                        imports.push(import);
                    }
                }
            }
        }
        
        imports
    }

    fn calculate_complexity(&self, content: &str) -> u8 {
        let mut score = 0u8;
        
        // Count functions and methods
        let functions = Regex::new(r"def\s+\w+").unwrap();
        score += functions.find_iter(content).count() as u8 * 2;
        
        // Count classes
        let classes = Regex::new(r"class\s+\w+").unwrap();
        score += classes.find_iter(content).count() as u8 * 3;
        
        // Count conditionals
        let conditionals = Regex::new(r"\b(if|elif|else|match|case)\b").unwrap();
        score += conditionals.find_iter(content).count() as u8;
        
        // Count loops
        let loops = Regex::new(r"\b(for|while)\b").unwrap();
        score += loops.find_iter(content).count() as u8 * 2;
        
        // Count decorators (complexity indicators)
        let decorators = Regex::new(r"@\w+").unwrap();
        score += decorators.find_iter(content).count() as u8;
        
        // Count exception handling
        let exceptions = Regex::new(r"\b(try|except|finally|raise)\b").unwrap();
        score += exceptions.find_iter(content).count() as u8;
        
        // Lines of code factor
        let lines = content.lines().count();
        score += (lines / 12) as u8;
        
        score.min(100)
    }

    fn determine_implementation_status(&self, content: &str) -> ImplementationStatus {
        let todo_patterns = [
            r"TODO",
            r"FIXME",
            r"XXX", 
            r"HACK",
            r"NotImplemented",
            r"raise NotImplementedError",
            r"pass\s*#.*todo",
        ];
        
        if todo_patterns.iter().any(|pattern| {
            Regex::new(&format!(r"(?i){}", pattern)).unwrap().is_match(content)
        }) {
            return ImplementationStatus::Todo;
        }
        
        // Check for stub implementations
        let pass_count = Regex::new(r"\bpass\b").unwrap().find_iter(content).count();
        let function_count = Regex::new(r"def\s+").unwrap().find_iter(content).count();
        
        if pass_count > 0 && function_count > 0 && pass_count >= function_count / 2 {
            return ImplementationStatus::Incomplete;
        }
        
        // Check substantial implementation
        let meaningful_lines = content.lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && !trimmed.starts_with('#')
            })
            .count();
            
        if meaningful_lines < 10 {
            return ImplementationStatus::InProgress;
        }
        
        ImplementationStatus::Complete
    }

    fn generate_user_stories(&self, components: &[ComponentInfo]) -> Vec<UserStory> {
        let mut stories = Vec::new();
        let mut story_id = 1;

        for component in components {
            match component.component_type {
                ComponentType::Service => {
                    // API endpoints
                    for api_call in &component.api_calls {
                        stories.push(UserStory {
                            id: format!("US-{:03}", story_id),
                        priority: Priority::Medium,
                            name: format!("Access {} endpoint", api_call.endpoint),
                        priority: Priority::Medium,
                            description: match self.framework_type {
                                PythonFramework::Django => format!("As a client, I want to {} {} so that I can interact with the Django API", 
                                    api_call.method, api_call.endpoint),
                        priority: Priority::Medium,
                                PythonFramework::Flask => format!("As a user, I want to {} {} so that I can access Flask functionality", 
                                    api_call.method, api_call.endpoint),
                        priority: Priority::Medium,
                                _ => format!("As a user, I want to access {} endpoint", api_call.endpoint),
                        priority: Priority::Medium,
                            },
                            acceptance_criteria: vec![
                                format!("Endpoint {} responds correctly", api_call.endpoint),
                        priority: Priority::Medium,
                                "Proper status codes are returned".to_string(),
                        priority: Priority::Medium,
                                "Response format is consistent".to_string(),
                        priority: Priority::Medium,
                            ],
                            priority: Priority::High,
                            complexity: if component.complexity_score > 50 { Complexity::Complex } else { Complexity::Medium },
                            related_components: vec![component.name.clone()],
                            status: component.implementation_status.clone(),
                        priority: Priority::Medium,
                            inferred_from: vec![component.file_path.clone()],
                        });
                        story_id += 1;
                    }

                    // Generic service story if no API calls
                    if component.api_calls.is_empty() && component.complexity_score > 20 {
                        stories.push(UserStory {
                            id: format!("US-{:03}", story_id),
                        priority: Priority::Medium,
                            name: format!("Use {} functionality", component.name),
                        priority: Priority::Medium,
                            description: format!("As a system, I need {} so that I can {}", 
                                component.name, component.purpose.to_lowercase()),
                        priority: Priority::Medium,
                            acceptance_criteria: vec![
                                format!("{} processes requests correctly", component.name),
                        priority: Priority::Medium,
                                "Business logic is implemented".to_string(),
                        priority: Priority::Medium,
                                "Error handling works properly".to_string(),
                        priority: Priority::Medium,
                            ],
                            priority: Priority::Medium,
                            complexity: if component.complexity_score > 40 { Complexity::Complex } else { Complexity::Simple },
                            related_components: vec![component.name.clone()],
                            status: component.implementation_status.clone(),
                        priority: Priority::Medium,
                            inferred_from: vec![component.file_path.clone()],
                        });
                        story_id += 1;
                    }
                },
                ComponentType::Context => {
                    // Data models
                    stories.push(UserStory {
                        id: format!("US-{:03}", story_id),
                        priority: Priority::Medium,
                        name: format!("Manage {} data", component.name),
                        priority: Priority::Medium,
                        description: match self.framework_type {
                            PythonFramework::Django => format!("As a system, I need to manage {} model data so that I can persist application state", component.name),
                        priority: Priority::Medium,
                            _ => format!("As a system, I need to manage {} data so that I can maintain state", component.name),
                        priority: Priority::Medium,
                        },
                        acceptance_criteria: vec![
                            format!("{} data can be created and retrieved", component.name),
                        priority: Priority::Medium,
                            "Data validation is enforced".to_string(),
                        priority: Priority::Medium,
                            "Database operations work correctly".to_string(),
                        priority: Priority::Medium,
                        ],
                        priority: Priority::High,
                        complexity: Complexity::Simple,
                        related_components: vec![component.name.clone()],
                        status: component.implementation_status.clone(),
                        priority: Priority::Medium,
                        inferred_from: vec![component.file_path.clone()],
                    });
                    story_id += 1;
                },
                ComponentType::Form => {
                    stories.push(UserStory {
                        id: format!("US-{:03}", story_id),
                        priority: Priority::Medium,
                        name: format!("Submit {} form", component.name),
                        priority: Priority::Medium,
                        description: format!("As a user, I want to submit {} form so that I can provide input data", component.name),
                        priority: Priority::Medium,
                        acceptance_criteria: vec![
                            "Form validates input correctly".to_string(),
                        priority: Priority::Medium,
                            "Form submission works as expected".to_string(),
                        priority: Priority::Medium,
                            "User receives appropriate feedback".to_string(),
                        priority: Priority::Medium,
                        ],
                        priority: Priority::Medium,
                        complexity: Complexity::Medium,
                        related_components: vec![component.name.clone()],
                        status: component.implementation_status.clone(),
                        priority: Priority::Medium,
                        inferred_from: vec![component.file_path.clone()],
                    });
                    story_id += 1;
                },
                _ => {}
            }
        }

        stories
    }

    fn generate_prd(&self, components: &[ComponentInfo], _user_stories: &[UserStory]) -> ProductRequirementDocument {
        let views_or_routes = components.iter()
            .filter(|c| matches!(c.component_type, ComponentType::Service))
            .count();
        let models = components.iter()
            .filter(|c| matches!(c.component_type, ComponentType::Context))
            .count();
        let forms = components.iter()
            .filter(|c| matches!(c.component_type, ComponentType::Form))
            .count();

        let framework_name = match self.framework_type {
            PythonFramework::Django => "Django",
            PythonFramework::Flask => "Flask", 
            PythonFramework::Unknown => "Python",
        };

        ProductRequirementDocument {
            name: format!("{} Application Requirements", framework_name),
                        priority: Priority::Medium,
            overview: format!("{} web application with {} views/routes, {} models, and {} forms, providing comprehensive web functionality.", 
                framework_name, views_or_routes, models, forms),
                        priority: Priority::Medium,
            objectives: vec![
                "Provide robust web application functionality".to_string(),
                        priority: Priority::Medium,
                "Implement scalable data models".to_string(),
                        priority: Priority::Medium,
                "Ensure proper request handling".to_string(),
                        priority: Priority::Medium,
                "Maintain clean separation of concerns".to_string(),
                        priority: Priority::Medium,
            ],
            target_users: vec![
                "Web application users".to_string(),
                        priority: Priority::Medium,
                "API consumers".to_string(),
                        priority: Priority::Medium,
                "System administrators".to_string(),
                        priority: Priority::Medium,
            ],
            features: self.extract_features_from_python_components(components),
                        priority: Priority::Medium,
            technical_requirements: match self.framework_type {
                PythonFramework::Django => vec![
                    "Django web framework".to_string(),
                        priority: Priority::Medium,
                    "Django REST Framework for APIs".to_string(),
                        priority: Priority::Medium,
                    "Database integration (Django ORM)".to_string(),
                        priority: Priority::Medium,
                    "Template system for rendering".to_string(),
                        priority: Priority::Medium,
                    "Middleware for request processing".to_string(),
                        priority: Priority::Medium,
                ],
                PythonFramework::Flask => vec![
                    "Flask web framework".to_string(),
                        priority: Priority::Medium,
                    "SQLAlchemy for database operations".to_string(),
                        priority: Priority::Medium,
                    "Jinja2 templating".to_string(),
                        priority: Priority::Medium,
                    "RESTful API design".to_string(),
                        priority: Priority::Medium,
                    "Blueprint organization".to_string(),
                        priority: Priority::Medium,
                ],
                PythonFramework::Unknown => vec![
                    "Python web framework".to_string(),
                        priority: Priority::Medium,
                    "Database integration".to_string(),
                        priority: Priority::Medium,
                    "HTTP request handling".to_string(),
                        priority: Priority::Medium,
                ],
            },
            business_context: format!("{} web application designed to provide reliable web services and data management for modern applications.", framework_name),
                        priority: Priority::Medium,
        }
    }

    fn extract_features_from_python_components(&self, components: &[ComponentInfo]) -> Vec<FeatureDescription> {
        let mut features = Vec::new();
        let mut processed_types = std::collections::HashSet::new();

        for component in components {
            let feature_key = format!("{:?}", component.component_type);
            if processed_types.contains(&feature_key) {
                continue;
            }
            processed_types.insert(feature_key);

            let feature = match component.component_type {
                ComponentType::Service => FeatureDescription {
                    name: "Web Request Handling".to_string(),
                        priority: Priority::Medium,
                    description: match self.framework_type {
                        PythonFramework::Django => "Django views and API endpoints for handling HTTP requests".to_string(),
                        priority: Priority::Medium,
                        PythonFramework::Flask => "Flask routes and handlers for web requests".to_string(),
                        priority: Priority::Medium,
                        _ => "Web request processing and response handling".to_string(),
                        priority: Priority::Medium,
                    },
                    user_value: "Users can interact with the web application through various endpoints".to_string(),
                        priority: Priority::Medium,
                    technical_approach: match self.framework_type {
                        PythonFramework::Django => "Django class-based and function-based views with URL routing".to_string(),
                        priority: Priority::Medium,
                        PythonFramework::Flask => "Flask route decorators with function-based handlers".to_string(),
                        priority: Priority::Medium,
                        _ => "HTTP request routing and processing".to_string(),
                        priority: Priority::Medium,
                    },
                    related_stories: vec![],
                },
                ComponentType::Context => FeatureDescription {
                    name: "Data Management".to_string(),
                        priority: Priority::Medium,
                    description: match self.framework_type {
                        PythonFramework::Django => "Django ORM models for data persistence and relationships".to_string(),
                        priority: Priority::Medium,
                        _ => "Data models and database integration for persistent storage".to_string(),
                        priority: Priority::Medium,
                    },
                    user_value: "Reliable data storage and retrieval with proper data relationships".to_string(),
                        priority: Priority::Medium,
                    technical_approach: match self.framework_type {
                        PythonFramework::Django => "Django models with database migrations and ORM queries".to_string(),
                        priority: Priority::Medium,
                        PythonFramework::Flask => "SQLAlchemy models with database schema management".to_string(),
                        priority: Priority::Medium,
                        _ => "Object-relational mapping for data persistence".to_string(),
                        priority: Priority::Medium,
                    },
                    related_stories: vec![],
                },
                ComponentType::Form => FeatureDescription {
                    name: "User Input Processing".to_string(),
                        priority: Priority::Medium,
                    description: "Form handling and validation for user data input".to_string(),
                        priority: Priority::Medium,
                    user_value: "Users can submit data through validated forms with proper error handling".to_string(),
                        priority: Priority::Medium,
                    technical_approach: match self.framework_type {
                        PythonFramework::Django => "Django forms with field validation and CSRF protection".to_string(),
                        priority: Priority::Medium,
                        PythonFramework::Flask => "WTForms integration with validation and rendering".to_string(),
                        priority: Priority::Medium,
                        _ => "Form processing with validation and error handling".to_string(),
                        priority: Priority::Medium,
                    },
                    related_stories: vec![],
                },
                _ => continue,
            };
            
            features.push(feature);
        }

        features
    }

    fn generate_tasks(&self, components: &[ComponentInfo], _user_stories: &[UserStory]) -> Vec<Task> {
        let mut tasks = Vec::new();
        let mut task_id = 1;

        for component in components {
            match component.implementation_status {
                ImplementationStatus::Todo | ImplementationStatus::Incomplete => {
                    tasks.push(Task {
                        id: format!("T-{:03}", task_id),
                        priority: Priority::Medium,
                        name: format!("Implement {}", component.name),
                        priority: Priority::Medium,
                        description: format!("Complete implementation of {} component", component.name),
                        priority: Priority::Medium,
                        task_type: TaskType::Feature,
                        status: component.implementation_status.clone(),
                        priority: Priority::Medium,
                        effort_estimate: Some( if component.complexity_score > 60 { "Large".to_string()) } 
                                        else if component.complexity_score > 30 { "Medium".to_string()) } 
                                        else { "Small".to_string()) },
                        related_components: vec![component.name.clone()],
                        dependencies: component.dependencies.clone(),
                        priority: Priority::Medium,
                        acceptance_criteria: vec![
                            "Code runs without errors".to_string(),
                        priority: Priority::Medium,
                            "All functions are properly implemented".to_string(),
                        priority: Priority::Medium,
                            "Tests pass successfully".to_string(),
                        priority: Priority::Medium,
                            "Code follows Python best practices".to_string(),
                        priority: Priority::Medium,
                        ],
                    });
                    task_id += 1;
                },
                ImplementationStatus::InProgress => {
                    tasks.push(Task {
                        id: format!("T-{:03}", task_id),
                        priority: Priority::Medium,
                        name: format!("Complete {}", component.name),
                        priority: Priority::Medium,
                        description: format!("Finish implementation of {} component", component.name),
                        priority: Priority::Medium,
                        task_type: TaskType::Feature,
                        status: component.implementation_status.clone(),
                        priority: Priority::Medium,
                        effort_estimate: Some( "Small".to_string()),
                        priority: Priority::Medium,
                        related_components: vec![component.name.clone()],
                        dependencies: vec![],
                        acceptance_criteria: vec![
                            "Remove any TODO or FIXME comments".to_string(),
                        priority: Priority::Medium,
                            "Add proper error handling".to_string(),
                        priority: Priority::Medium,
                            "Ensure component is production-ready".to_string(),
                        priority: Priority::Medium,
                        ],
                    });
                    task_id += 1;
                },
                _ => {}
            }
        }

        tasks
    }
}

impl CodebaseAnalyzer for PythonAnalyzer {
    fn analyze(&self, project_path: &str) -> Result<CodebaseAnalysis> {
        let mut analyzer = PythonAnalyzer::new(self.config.clone());
        let framework = analyzer.detect_framework(project_path);
        
        let mut components = Vec::new();
        let mut files_analyzed = 0u32;
        let mut lines_of_code = 0u32;
        
        for entry in WalkDir::new(project_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "py" {
                    if let Ok(content) = fs::read_to_string(path) {
                        files_analyzed += 1;
                        lines_of_code += content.lines().count() as u32;
                        
                        if let Some(component) = analyzer.extract_component_info(
                            path.to_str().unwrap(),
                        priority: Priority::Medium,
                            &content
                        ) {
                            components.push(component);
                        }
                    }
                }
            }
        }

        let user_stories = analyzer.generate_user_stories(&components);
        let prd = analyzer.generate_prd(&components, &user_stories);
        let tasks = analyzer.generate_tasks(&components, &user_stories);
        
        let project_name = Path::new(project_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown Project")
            .to_string();

        let project_type = match framework {
            PythonFramework::Django => ProjectType::Django,
            PythonFramework::Flask => ProjectType::Flask,
            PythonFramework::Unknown => ProjectType::Unknown,
        };

        Ok(CodebaseAnalysis {
            project_name,
            project_type,
            components,
            user_stories,
            prd,
            tasks,
            analysis_metadata: AnalysisMetadata {
                analyzed_at: chrono::Utc::now().to_rfc3339(),
                        priority: Priority::Medium,
                analyzer_version: "0.1.0".to_string(),
                        priority: Priority::Medium,
                files_analyzed,
                lines_of_code,
                confidence_score: 0.75,
                warnings: Vec::new(),
                        priority: Priority::Medium,
            },
        })
    }

    fn supported_extensions(&self) -> Vec<&str> {
        vec!["py"]
    }

    fn can_analyze(&self, project_path: &str) -> bool {
        // Check for Django/Flask specific files
        let django_indicators = [
            "manage.py",
            "settings.py", 
            "wsgi.py",
        ];
        
        let flask_indicators = [
            "app.py",
            "requirements.txt",
        ];

        for indicator in &django_indicators {
            if Path::new(project_path).join(indicator).exists() {
                return true;
            }
        }

        for indicator in &flask_indicators {
            let path = Path::new(project_path).join(indicator);
            if path.exists() && indicator == &"requirements.txt" {
                if let Ok(content) = fs::read_to_string(path) {
                    if content.to_lowercase().contains("flask") {
                        return true;
                    }
                }
            } else if path.exists() {
                return true;
            }
        }
        
        // Check for Python files with Django/Flask imports
        for entry in WalkDir::new(project_path)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if let Some(ext) = entry.path().extension() {
                if ext == "py" {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if content.contains("from django") || content.contains("from flask") {
                            return true;
                        }
                    }
                }
            }
        }
        
        false
    }
}