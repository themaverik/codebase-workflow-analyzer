use crate::core::{
    CodebaseAnalysis, CodebaseAnalyzer, ComponentInfo, ComponentType, PropInfo, ApiCall,
    ImplementationStatus, UserStory, Task, TaskType, Priority, Complexity, FeatureDescription,
    ProductRequirementDocument, AnalysisMetadata, ProjectType, AnalyzerConfig
};
use regex::Regex;
use std::fs;
use walkdir::WalkDir;
use anyhow::Result;

pub struct PythonAnalyzer {
    config: AnalyzerConfig,
}

impl PythonAnalyzer {
    pub fn new(config: AnalyzerConfig) -> Self {
        Self { config }
    }

    fn extract_component_info(&self, file_path: &str, content: &str) -> Option<ComponentInfo> {
        let component_name = std::path::Path::new(file_path)
            .file_stem()?
            .to_str()?
            .to_string();

        let component_type = self.determine_component_type(content, &component_name);
        let purpose = self.infer_component_purpose(content, &component_name, &component_type);
        let api_calls = self.extract_api_calls(content);
        let dependencies = self.extract_dependencies(content);
        let complexity_score = self.calculate_complexity(content);
        let implementation_status = self.determine_implementation_status(content);

        Some(ComponentInfo {
            name: component_name,
            file_path: file_path.to_string(),
            component_type,
            purpose,
            dependencies,
            props: self.extract_props(content),
            hooks_used: vec![], // Python doesn't have hooks like React
            api_calls,
            complexity_score,
            implementation_status,
        })
    }

    fn determine_component_type(&self, content: &str, name: &str) -> ComponentType {
        if content.contains("class") && content.contains("View") {
            ComponentType::Service
        } else if content.contains("class") && content.contains("Model") {
            ComponentType::Display
        } else if content.contains("def") && content.contains("render") {
            ComponentType::Display
        } else if content.contains("@app.route") || content.contains("@bp.route") {
            ComponentType::Service
        } else if content.contains("forms.") || content.contains("Form") {
            ComponentType::Display
        } else if name.contains("utils") || name.contains("helpers") {
            ComponentType::Utility
        } else if name.contains("test") {
            ComponentType::Utility
        } else {
            ComponentType::Utility
        }
    }

    fn infer_component_purpose(&self, content: &str, name: &str, component_type: &ComponentType) -> String {
        if content.contains("@app.route") {
            format!("Flask route handler for {}", name)
        } else if content.contains("class") && content.contains("View") {
            format!("Django view handling requests for {}", name)
        } else if content.contains("class") && content.contains("Model") {
            format!("Django model representing {} data", name)
        } else if content.contains("forms.") {
            format!("Django form for {} data validation", name)
        } else {
            match component_type {
                ComponentType::Service => format!("Service component for {}", name),
                ComponentType::Display => format!("Display component for {}", name),
                ComponentType::Utility => format!("Utility module for {}", name),
                _ => format!("Python module for {}", name),
            }
        }
    }

    fn extract_props(&self, content: &str) -> Vec<PropInfo> {
        let mut props = Vec::new();

        // Extract function parameters that might be props
        if let Ok(re) = Regex::new(r"def\s+\w+\s*\([^)]*(\w+):\s*(\w+)") {
            for captures in re.captures_iter(content) {
                if let (Some(name), Some(prop_type)) = (captures.get(1), captures.get(2)) {
                    props.push(PropInfo {
                        name: name.as_str().to_string(),
                        prop_type: prop_type.as_str().to_string(),
                        required: true,
                        description: Some(format!("Parameter for {}", name.as_str())),
                    });
                }
            }
        }

        props
    }

    fn extract_api_calls(&self, content: &str) -> Vec<ApiCall> {
        let mut api_calls = Vec::new();

        // Extract Flask routes
        if let Ok(re) = Regex::new(r#"@app\.route\s*\(\s*["']([^"']+)["']"#) {
            for captures in re.captures_iter(content) {
                let endpoint = captures.get(1).unwrap().as_str().to_string();
                api_calls.push(ApiCall {
                    endpoint,
                    method: "HTTP".to_string(),
                    purpose: "Flask route endpoint".to_string(),
                });
            }
        }

        // Extract Django URLs
        if let Ok(re) = Regex::new(r#"path\s*\(\s*r?["']([^"']+)["']"#) {
            for captures in re.captures_iter(content) {
                let endpoint = captures.get(1).unwrap().as_str().to_string();
                api_calls.push(ApiCall {
                    endpoint,
                    method: "HTTP".to_string(),
                    purpose: "Django URL pattern".to_string(),
                });
            }
        }

        api_calls
    }

    fn extract_dependencies(&self, content: &str) -> Vec<String> {
        let mut dependencies = Vec::new();

        // Extract import statements
        if let Ok(re) = Regex::new(r"(?:from\s+([a-zA-Z0-9_.]+)\s+import|import\s+([a-zA-Z0-9_.]+))") {
            for captures in re.captures_iter(content) {
                if let Some(import) = captures.get(1).or_else(|| captures.get(2)) {
                    let import_str = import.as_str();
                    if !import_str.starts_with('.') && !import_str.starts_with("__") {
                        dependencies.push(import_str.to_string());
                    }
                }
            }
        }

        dependencies.sort();
        dependencies.dedup();
        dependencies
    }

    fn calculate_complexity(&self, content: &str) -> u8 {
        let mut complexity = 10;

        complexity += content.matches("if ").count() as u8 * 2;
        complexity += content.matches("for ").count() as u8 * 3;
        complexity += content.matches("while ").count() as u8 * 3;
        complexity += content.matches("try:").count() as u8 * 2;
        complexity += content.matches("except ").count() as u8 * 2;
        complexity += content.matches("def ").count() as u8 * 1;
        complexity += content.matches("class ").count() as u8 * 2;

        complexity.min(100)
    }

    fn determine_implementation_status(&self, content: &str) -> ImplementationStatus {
        if content.contains("TODO") || content.contains("FIXME") {
            ImplementationStatus::Todo
        } else if content.contains("pass") && content.matches("pass").count() > 1 {
            ImplementationStatus::Todo
        } else if content.contains("print(") || content.contains("logging.debug") {
            ImplementationStatus::InProgress
        } else if content.contains("def test_") || content.contains("class Test") {
            ImplementationStatus::Complete
        } else if content.len() < 200 {
            ImplementationStatus::Incomplete
        } else {
            ImplementationStatus::Complete
        }
    }

    fn generate_user_stories(&self, components: &[ComponentInfo]) -> Vec<UserStory> {
        let mut stories = Vec::new();
        let mut story_id = 1;

        for component in components {
            match component.component_type {
                ComponentType::Service => {
                    stories.push(UserStory {
                        id: format!("US-{:03}", story_id),
                        title: format!("Access {} functionality", component.name),
                        description: format!("As a user, I want to access {} service functionality", component.name),
                        acceptance_criteria: vec![
                            format!("{} service responds correctly", component.name),
                            "Service handles errors gracefully".to_string(),
                            "Service provides expected functionality".to_string(),
                        ],
                        priority: Priority::Medium,
                        complexity: if component.complexity_score > 50 { Complexity::Complex } else { Complexity::Medium },
                        related_components: vec![component.name.clone()],
                        status: component.implementation_status.clone(),
                        inferred_from: vec![component.file_path.clone()],
                    });
                    story_id += 1;
                },
                ComponentType::Display => {
                    stories.push(UserStory {
                        id: format!("US-{:03}", story_id),
                        title: format!("View {} interface", component.name),
                        description: format!("As a user, I want to interact with {} interface", component.name),
                        acceptance_criteria: vec![
                            format!("{} displays correctly", component.name),
                            "Interface is responsive and user-friendly".to_string(),
                            "All interactive elements work as expected".to_string(),
                        ],
                        priority: Priority::Medium,
                        complexity: if component.complexity_score > 40 { Complexity::Complex } else { Complexity::Simple },
                        related_components: vec![component.name.clone()],
                        status: component.implementation_status.clone(),
                        inferred_from: vec![component.file_path.clone()],
                    });
                    story_id += 1;
                },
                _ => {}
            }
        }

        stories
    }

    fn generate_prd(&self, components: &[ComponentInfo]) -> ProductRequirementDocument {
        let services = components.iter()
            .filter(|c| matches!(c.component_type, ComponentType::Service))
            .count();
        let displays = components.iter()
            .filter(|c| matches!(c.component_type, ComponentType::Display))
            .count();

        ProductRequirementDocument {
            title: "Python Application Requirements".to_string(),
            overview: format!("Python-based application with {} service components and {} display components.", services, displays),
            objectives: vec![
                "Provide robust backend services".to_string(),
                "Deliver intuitive user interfaces".to_string(),
                "Ensure high performance and scalability".to_string(),
                "Maintain code quality and testability".to_string(),
            ],
            target_users: vec![
                "End users interacting with the application".to_string(),
                "API clients consuming backend services".to_string(),
                "System administrators managing the application".to_string(),
            ],
            features: self.extract_features_from_components(components),
            technical_requirements: vec![
                "Python-based web framework (Django/Flask)".to_string(),
                "RESTful API design".to_string(),
                "Database integration".to_string(),
                "Template rendering system".to_string(),
            ],
            business_context: "Python web application designed to provide comprehensive functionality with both backend services and user-facing interfaces.".to_string(),
        }
    }

    fn extract_features_from_components(&self, components: &[ComponentInfo]) -> Vec<FeatureDescription> {
        let mut features = Vec::new();

        if components.iter().any(|c| matches!(c.component_type, ComponentType::Service)) {
            features.push(FeatureDescription {
                name: "Backend Services".to_string(),
                description: "Core business logic and data processing services".to_string(),
                user_value: "Reliable backend functionality for application operations".to_string(),
                technical_approach: "Python-based services with proper separation of concerns".to_string(),
                related_stories: vec![],
            });
        }

        if components.iter().any(|c| matches!(c.component_type, ComponentType::Display)) {
            features.push(FeatureDescription {
                name: "User Interface".to_string(),
                description: "Web-based user interface components".to_string(),
                user_value: "Intuitive and responsive user experience".to_string(),
                technical_approach: "Template-based rendering with modern web standards".to_string(),
                related_stories: vec![],
            });
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
                        name: format!("Implement {}", component.name),
                        description: format!("Complete implementation of {} component", component.name),
                        task_type: TaskType::Feature,
                        status: component.implementation_status.clone(),
                        effort_estimate: Some(if component.complexity_score > 60 { 
                            "Large".to_string() 
                        } else if component.complexity_score > 30 { 
                            "Medium".to_string() 
                        } else { 
                            "Small".to_string() 
                        }),
                        priority: Priority::Medium,
                        related_components: vec![component.name.clone()],
                        dependencies: component.dependencies.clone(),
                        acceptance_criteria: vec![
                            "Code runs without errors".to_string(),
                            "All functions are properly implemented".to_string(),
                            "Tests pass successfully".to_string(),
                            "Code follows Python best practices".to_string(),
                        ],
                    });
                    task_id += 1;
                },
                ImplementationStatus::InProgress => {
                    tasks.push(Task {
                        id: format!("T-{:03}", task_id),
                        name: format!("Complete {}", component.name),
                        description: format!("Finish implementation of {} component", component.name),
                        task_type: TaskType::Feature,
                        status: component.implementation_status.clone(),
                        effort_estimate: Some("Small".to_string()),
                        priority: Priority::Medium,
                        related_components: vec![component.name.clone()],
                        dependencies: vec![],
                        acceptance_criteria: vec![
                            "Remove debugging code".to_string(),
                            "Add proper error handling".to_string(),
                            "Ensure production readiness".to_string(),
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
        let mut components = Vec::new();
        let mut files_analyzed = 0;
        let mut lines_of_code = 0;

        for entry in WalkDir::new(project_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            
            if let Some(ext) = path.extension() {
                if ext == "py" {
                    if let Ok(content) = fs::read_to_string(path) {
                        files_analyzed += 1;
                        lines_of_code += content.lines().count() as u32;
                        
                        if let Some(component) = self.extract_component_info(
                            path.to_str().unwrap(),
                            &content
                        ) {
                            components.push(component);
                        }
                    }
                }
            }
        }

        let project_name = std::path::Path::new(project_path)
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or("unknown")
            .to_string();

        let user_stories = self.generate_user_stories(&components);
        let prd = self.generate_prd(&components);
        let tasks = self.generate_tasks(&components, &user_stories);

        Ok(CodebaseAnalysis {
            project_name,
            project_type: ProjectType::Django,
            components,
            user_stories,
            prd,
            tasks,
            analysis_metadata: AnalysisMetadata {
                analyzed_at: chrono::Utc::now().to_rfc3339(),
                analyzer_version: "0.1.0".to_string(),
                files_analyzed,
                lines_of_code,
                confidence_score: 0.75,
                warnings: Vec::new(),
            },
            // Default implementations for enhanced fields
            framework_analysis: crate::core::FrameworkAnalysis {
                detected_frameworks: Vec::new(),
                confidence_scores: std::collections::HashMap::new(),
                architecture_pattern: "Django Web Application".to_string(),
            },
            business_context: crate::core::BusinessContext {
                inferred_product_type: "Python Web Application".to_string(),
                confidence: 0.7,
                evidence: Vec::new(),
                primary_user_personas: Vec::new(),
                user_journeys_discovered: Vec::new(),
                business_domain: "Web Application".to_string(),
            },
            implementation_analysis: crate::core::ImplementationAnalysis {
                api_endpoints: Vec::new(),
                database_entities: Vec::new(),
                component_relationships: Vec::new(),
                data_flow: Vec::new(),
            },
            status_intelligence: crate::core::StatusIntelligence {
                completed_features: Vec::new(),
                in_progress_features: Vec::new(),
                todo_features: Vec::new(),
                technical_debt: Vec::new(),
                overall_completion_percentage: 0.0,
            },
            integration_points: crate::core::IntegrationPoints {
                external_services: Vec::new(),
                internal_dependencies: Vec::new(),
                configuration_files: Vec::new(),
                environment_variables: Vec::new(),
            },
        })
    }

    fn supported_extensions(&self) -> Vec<&str> {
        vec!["py"]
    }

    fn can_analyze(&self, project_path: &str) -> bool {
        let requirements_txt = std::path::Path::new(project_path).join("requirements.txt");
        let manage_py = std::path::Path::new(project_path).join("manage.py");
        let app_py = std::path::Path::new(project_path).join("app.py");
        
        if manage_py.exists() {
            return true; // Django project
        }
        
        if app_py.exists() {
            return true; // Flask project
        }
        
        if requirements_txt.exists() {
            if let Ok(content) = fs::read_to_string(requirements_txt) {
                return content.contains("django") || content.contains("flask");
            }
        }
        
        false
    }
}