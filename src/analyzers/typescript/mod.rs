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

pub struct TypeScriptAnalyzer {
    config: AnalyzerConfig,
}

impl TypeScriptAnalyzer {
    pub fn new(config: AnalyzerConfig) -> Self {
        Self { config }
    }

    fn is_react_component(&self, content: &str) -> bool {
        let react_patterns = [
            r#"import\s+.*\bReact\b.*from\s+['"]react['"]"#,
            r#"import\s+\{[^}]*\bComponent\b[^}]*\}\s+from\s+['"]react['"]"#,
            r"export\s+(?:default\s+)?(?:function|const)\s+\w+.*:\s*React\.FC",
            r"export\s+(?:default\s+)?(?:function|const)\s+\w+.*\([^)]*\)\s*\{[^}]*return\s*\(",
            r"\.jsx?$",
            r"\.tsx$",
            r"<[A-Z]\w*[^>]*>.*</[A-Z]\w*>",
        ];

        react_patterns.iter().any(|pattern| {
            Regex::new(pattern).unwrap().is_match(content)
        })
    }

    fn extract_component_info(&self, file_path: &str, content: &str) -> Option<ComponentInfo> {
        if !self.is_react_component(content) {
            return None;
        }

        let component_name = self.extract_component_name(file_path, content)?;
        let component_type = self.classify_component_type(&component_name, content);
        let purpose = self.infer_component_purpose(&component_name, content);
        let props = self.extract_props(content);
        let hooks_used = self.extract_hooks(content);
        let api_calls = self.extract_api_calls(content);
        let dependencies = self.extract_dependencies(content);
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
            hooks_used,
            api_calls,
            complexity_score,
            implementation_status,
        })
    }

    fn extract_component_name(&self, file_path: &str, content: &str) -> Option<String> {
        // Try to extract from export statement first
        let export_patterns = [
            r"export\s+default\s+function\s+(\w+)",
            r"export\s+default\s+const\s+(\w+)",
            r"export\s+function\s+(\w+)",
            r"export\s+const\s+(\w+)",
            r"export\s+default\s+(\w+)",
        ];

        for pattern in &export_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(captures) = re.captures(content) {
                    return Some(captures.get(1)?.as_str().to_string());
                }
            }
        }

        // Fallback to filename
        Path::new(file_path)
            .file_stem()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
    }

    fn classify_component_type(&self, name: &str, content: &str) -> ComponentType {
        let name_lower = name.to_lowercase();
        
        if name_lower.contains("page") || name_lower.contains("screen") || content.contains("useRouter") {
            return ComponentType::Page;
        }
        if name_lower.contains("layout") || name_lower.contains("template") {
            return ComponentType::Layout;
        }
        if name_lower.contains("form") || content.contains("onSubmit") || content.contains("useForm") {
            return ComponentType::Form;
        }
        if name_lower.contains("nav") || name_lower.contains("menu") || name_lower.contains("header") {
            return ComponentType::Navigation;
        }
        if name_lower.contains("modal") || name_lower.contains("dialog") || name_lower.contains("popup") {
            return ComponentType::Modal;
        }
        if content.contains("export const use") || content.contains("export function use") {
            return ComponentType::Hook;
        }
        if content.contains("createContext") || content.contains("useContext") {
            return ComponentType::Context;
        }
        if content.contains("fetch") || content.contains("axios") || content.contains("api") {
            return ComponentType::Service;
        }

        ComponentType::Display
    }

    fn infer_component_purpose(&self, name: &str, content: &str) -> String {
        let name_lower = name.to_lowercase();
        
        if name_lower.contains("login") || content.contains("password") || content.contains("signin") {
            return "Handles user authentication and login functionality".to_string();
        }
        if name_lower.contains("dashboard") {
            return "Displays main dashboard with key metrics and navigation".to_string();
        }
        if name_lower.contains("profile") {
            return "Manages user profile information and settings".to_string();
        }
        if name_lower.contains("list") || name_lower.contains("table") {
            return "Displays a list or table of items with filtering and sorting".to_string();
        }
        if name_lower.contains("detail") || name_lower.contains("view") {
            return "Shows detailed information about a specific item".to_string();
        }
        if name_lower.contains("create") || name_lower.contains("add") || name_lower.contains("new") {
            return "Provides form interface for creating new items".to_string();
        }
        if name_lower.contains("edit") || name_lower.contains("update") {
            return "Enables editing and updating of existing items".to_string();
        }

        format!("React component for {} functionality", name_lower)
    }

    fn extract_props(&self, content: &str) -> Vec<PropInfo> {
        let mut props = Vec::new();
        
        // Extract TypeScript interface props
        if let Ok(re) = Regex::new(r"interface\s+\w*Props\s*\{([^}]+)\}") {
            if let Some(captures) = re.captures(content) {
                let props_content = captures.get(1).unwrap().as_str();
                props.extend(self.parse_interface_props(props_content));
            }
        }

        // Extract inline props
        if let Ok(re) = Regex::new(r"function\s+\w+\s*\(\s*\{([^}]+)\}") {
            if let Some(captures) = re.captures(content) {
                let props_content = captures.get(1).unwrap().as_str();
                props.extend(self.parse_destructured_props(props_content));
            }
        }

        props
    }

    fn parse_interface_props(&self, props_content: &str) -> Vec<PropInfo> {
        let mut props = Vec::new();
        
        if let Ok(re) = Regex::new(r"(\w+)(\?)?:\s*([^;,\n]+)") {
            for captures in re.captures_iter(props_content) {
                let name = captures.get(1).unwrap().as_str().to_string();
                let required = captures.get(2).is_none(); // No ? means required
                let prop_type = captures.get(3).unwrap().as_str().trim().to_string();
                
                props.push(PropInfo {
                    name,
                    prop_type,
                    required,
                    description: None,
                });
            }
        }
        
        props
    }

    fn parse_destructured_props(&self, props_content: &str) -> Vec<PropInfo> {
        let mut props = Vec::new();
        
        for prop in props_content.split(',') {
            let prop = prop.trim();
            if let Some(name) = prop.split(':').next() {
                props.push(PropInfo {
                    name: name.trim().to_string(),
                        priority: Priority::Medium,
                    prop_type: "unknown".to_string(),
                        priority: Priority::Medium,
                    required: true,
                    description: None,
                });
            }
        }
        
        props
    }

    fn extract_hooks(&self, content: &str) -> Vec<String> {
        let mut hooks = Vec::new();
        
        if let Ok(re) = Regex::new(r"use\w+") {
            for mat in re.find_iter(content) {
                let hook = mat.as_str().to_string();
                if !hooks.contains(&hook) {
                    hooks.push(hook);
                }
            }
        }
        
        hooks
    }

    fn extract_api_calls(&self, content: &str) -> Vec<ApiCall> {
        let mut api_calls = Vec::new();
        
        // Extract fetch calls
        if let Ok(re) = Regex::new(r#"fetch\s*\(\s*['"`]([^'"`]+)['"`]"#) {
            for captures in re.captures_iter(content) {
                let endpoint = captures.get(1).unwrap().as_str().to_string();
                api_calls.push(ApiCall {
                    endpoint,
                    method: "GET".to_string(),
                        priority: Priority::Medium,
                    purpose: "Data fetching".to_string(),
                        priority: Priority::Medium,
                });
            }
        }

        // Extract axios calls
        if let Ok(re) = Regex::new(r#"axios\.(\w+)\s*\(\s*['"`]([^'"`]+)['"`]"#) {
            for captures in re.captures_iter(content) {
                let method = captures.get(1).unwrap().as_str().to_uppercase();
                let endpoint = captures.get(2).unwrap().as_str().to_string();
                api_calls.push(ApiCall {
                    endpoint,
                    method,
                    purpose: "API interaction".to_string(),
                        priority: Priority::Medium,
                });
            }
        }
        
        api_calls
    }

    fn extract_dependencies(&self, content: &str) -> Vec<String> {
        let mut dependencies = Vec::new();
        
        if let Ok(re) = Regex::new(r#"import\s+.*from\s+['"`]([^'"`]+)['"`]"#) {
            for captures in re.captures_iter(content) {
                let dep = captures.get(1).unwrap().as_str().to_string();
                if !dep.starts_with('.') && !dependencies.contains(&dep) {
                    dependencies.push(dep);
                }
            }
        }
        
        dependencies
    }

    fn calculate_complexity(&self, content: &str) -> u8 {
        let mut score = 0u8;
        
        // Count conditional statements
        let conditionals = Regex::new(r"\b(if|else|switch|case)\b").unwrap();
        score += conditionals.find_iter(content).count() as u8;
        
        // Count loops
        let loops = Regex::new(r"\b(for|while|map|forEach|filter|reduce)\b").unwrap();
        score += loops.find_iter(content).count() as u8 * 2;
        
        // Count hooks (complexity indicator)
        let hooks = Regex::new(r"\buse\w+\b").unwrap();
        score += hooks.find_iter(content).count() as u8;
        
        // Count functions
        let functions = Regex::new(r"\bfunction\b|=>\s*\{|:\s*\([^)]*\)\s*=>").unwrap();
        score += functions.find_iter(content).count() as u8;
        
        // Lines of code factor
        let lines = content.lines().count();
        score += (lines / 10) as u8;
        
        score.min(100) // Cap at 100
    }

    fn determine_implementation_status(&self, content: &str) -> ImplementationStatus {
        let todo_patterns = [
            r"TODO",
            r"FIXME",
            r"HACK",
            r"XXX",
            r"placeholder",
            r"not implemented",
            r"coming soon",
        ];
        
        let incomplete_patterns = [
            r"throw new Error\([^)]*not implemented",
            r"console\.log\([^)]*TODO",
            r"return null;\s*//\s*TODO",
        ];
        
        // Check for TODO markers
        if todo_patterns.iter().any(|pattern| {
            Regex::new(&format!(r"(?i){}", pattern)).unwrap().is_match(content)
        }) {
            return ImplementationStatus::Todo;
        }
        
        // Check for incomplete implementation
        if incomplete_patterns.iter().any(|pattern| {
            Regex::new(pattern).unwrap().is_match(content)
        }) {
            return ImplementationStatus::Incomplete;
        }
        
        // Check if it's a substantial implementation
        let lines = content.lines().filter(|line| !line.trim().is_empty()).count();
        if lines < 10 {
            return ImplementationStatus::InProgress;
        }
        
        ImplementationStatus::Complete
    }
}

impl CodebaseAnalyzer for TypeScriptAnalyzer {
    fn analyze(&self, project_path: &str) -> Result<CodebaseAnalysis> {
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
                if ["ts", "tsx", "js", "jsx"].contains(&ext.to_str().unwrap_or("")) {
                    if let Ok(content) = fs::read_to_string(path) {
                        files_analyzed += 1;
                        lines_of_code += content.lines().count() as u32;
                        
                        if let Some(component) = self.extract_component_info(
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

        let user_stories = self.generate_user_stories(&components);
        let prd = self.generate_prd(&components, &user_stories);
        let tasks = self.generate_tasks(&components, &user_stories);
        
        let project_name = Path::new(project_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown Project")
            .to_string();

        Ok(CodebaseAnalysis {
            project_name,
            project_type: ProjectType::React,
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
                confidence_score: 0.85,
                warnings: Vec::new(),
                        priority: Priority::Medium,
            },
        })
    }

    fn supported_extensions(&self) -> Vec<&str> {
        vec!["ts", "tsx", "js", "jsx"]
    }

    fn can_analyze(&self, project_path: &str) -> bool {
        // Check for package.json with React dependency
        let package_json_path = Path::new(project_path).join("package.json");
        if let Ok(content) = fs::read_to_string(package_json_path) {
            return content.contains("\"react\":");
        }
        
        // Check for TypeScript React files
        for entry in WalkDir::new(project_path)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if let Some(ext) = entry.path().extension() {
                if ["tsx", "jsx"].contains(&ext.to_str().unwrap_or("")) {
                    return true;
                }
            }
        }
        
        false
    }
}


impl TypeScriptAnalyzer {
    fn generate_user_stories(&self, components: &[ComponentInfo]) -> Vec<UserStory> {
        let mut stories = Vec::new();
        let mut story_id = 1;

        for component in components {
            match component.component_type {
                ComponentType::Page => {
                    stories.push(UserStory {
                        id: format!("US-{:03}", story_id),
                        priority: Priority::Medium,
                        name: format!("Access {} page", component.name),
                        priority: Priority::Medium,
                        description: format!("As a user, I want to access the {} page so that I can {}", 
                            component.name, component.purpose.to_lowercase()),
                        priority: Priority::Medium,
                        acceptance_criteria: vec![
                            format!("User can navigate to {}", component.name),
                        priority: Priority::Medium,
                            "Page loads without errors".to_string(),
                        priority: Priority::Medium,
                            "All required data is displayed".to_string(),
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
                },
                ComponentType::Form => {
                    stories.push(UserStory {
                        id: format!("US-{:03}", story_id),
                        priority: Priority::Medium,
                        name: format!("Submit {} form", component.name),
                        priority: Priority::Medium,
                        description: format!("As a user, I want to submit the {} form so that I can provide required information", component.name),
                        priority: Priority::Medium,
                        acceptance_criteria: vec![
                            "Form validates input data".to_string(),
                        priority: Priority::Medium,
                            "Form submits successfully".to_string(),
                        priority: Priority::Medium,
                            "User receives feedback on submission".to_string(),
                        priority: Priority::Medium,
                        ],
                        priority: Priority::High,
                        complexity: Complexity::Medium,
                        related_components: vec![component.name.clone()],
                        status: component.implementation_status.clone(),
                        priority: Priority::Medium,
                        inferred_from: vec![component.file_path.clone()],
                    });
                    story_id += 1;
                },
                _ => {
                    // Generate generic story for other component types
                    if component.complexity_score > 20 {
                        stories.push(UserStory {
                            id: format!("US-{:03}", story_id),
                        priority: Priority::Medium,
                            name: format!("Use {} functionality", component.name),
                        priority: Priority::Medium,
                            description: format!("As a user, I want to use {} so that I can {}", 
                                component.name, component.purpose.to_lowercase()),
                        priority: Priority::Medium,
                            acceptance_criteria: vec![
                                format!("{} component renders correctly", component.name),
                        priority: Priority::Medium,
                                "User can interact with the component".to_string(),
                        priority: Priority::Medium,
                                "Component behaves as expected".to_string(),
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
                }
            }
        }

        stories
    }

    fn generate_prd(&self, components: &[ComponentInfo], _user_stories: &[UserStory]) -> ProductRequirementDocument {
        let pages = components.iter()
            .filter(|c| matches!(c.component_type, ComponentType::Page))
            .count();
        let forms = components.iter()
            .filter(|c| matches!(c.component_type, ComponentType::Form))
            .count();

        ProductRequirementDocument {
            name: "Product Requirements Document".to_string(),
                        priority: Priority::Medium,
            overview: format!("React-based web application with {} pages and {} forms, providing comprehensive user interface functionality.", pages, forms),
                        priority: Priority::Medium,
            objectives: vec![
                "Provide intuitive user interface".to_string(),
                        priority: Priority::Medium,
                "Ensure responsive design across devices".to_string(),
                        priority: Priority::Medium,
                "Maintain high performance and accessibility".to_string(),
                        priority: Priority::Medium,
            ],
            target_users: vec![
                "End users seeking web-based functionality".to_string(),
                        priority: Priority::Medium,
                "Administrators managing system data".to_string(),
                        priority: Priority::Medium,
            ],
            features: self.extract_features_from_components(components),
                        priority: Priority::Medium,
            technical_requirements: vec![
                "React-based frontend framework".to_string(),
                        priority: Priority::Medium,
                "TypeScript for type safety".to_string(),
                        priority: Priority::Medium,
                "Modern browser compatibility".to_string(),
                        priority: Priority::Medium,
                "Responsive design implementation".to_string(),
                        priority: Priority::Medium,
            ],
            business_context: "Web application designed to provide efficient user experience through modern React components and interactions.".to_string(),
                        priority: Priority::Medium,
        }
    }

    fn extract_features_from_components(&self, components: &[ComponentInfo]) -> Vec<FeatureDescription> {
        let mut features = Vec::new();
        let mut processed_types = std::collections::HashSet::new();

        for component in components {
            let feature_key = format!("{:?}", component.component_type);
            if processed_types.contains(&feature_key) {
                continue;
            }
            processed_types.insert(feature_key);

            let feature = match component.component_type {
                ComponentType::Page => FeatureDescription {
                    name: "Page Navigation".to_string(),
                        priority: Priority::Medium,
                    description: "Multi-page application with navigation between different views".to_string(),
                        priority: Priority::Medium,
                    user_value: "Users can access different sections of the application efficiently".to_string(),
                        priority: Priority::Medium,
                    technical_approach: "React Router for client-side routing and navigation".to_string(),
                        priority: Priority::Medium,
                    related_stories: vec![], // Would be populated with actual story IDs
                },
                ComponentType::Form => FeatureDescription {
                    name: "Data Input Forms".to_string(),
                        priority: Priority::Medium,
                    description: "Interactive forms for data collection and submission".to_string(),
                        priority: Priority::Medium,
                    user_value: "Users can input and submit data through validated forms".to_string(),
                        priority: Priority::Medium,
                    technical_approach: "Controlled components with validation and error handling".to_string(),
                        priority: Priority::Medium,
                    related_stories: vec![],
                },
                ComponentType::Layout => FeatureDescription {
                    name: "Application Layout".to_string(),
                        priority: Priority::Medium,
                    description: "Consistent layout structure across the application".to_string(),
                        priority: Priority::Medium,
                    user_value: "Users experience consistent navigation and visual structure".to_string(),
                        priority: Priority::Medium,
                    technical_approach: "Reusable layout components with flexible content areas".to_string(),
                        priority: Priority::Medium,
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
                        effort_estimate: Some(if component.complexity_score > 50 { "Large".to_string() } else { "Medium".to_string() }),
                        priority: Priority::Medium,
                        priority: Priority::Medium,
                        related_components: vec![component.name.clone()],
                        dependencies: component.dependencies.clone(),
                        priority: Priority::Medium,
                        acceptance_criteria: vec![
                            "Component renders without errors".to_string(),
                        priority: Priority::Medium,
                            "All props are properly handled".to_string(),
                        priority: Priority::Medium,
                            "Component meets design requirements".to_string(),
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
                            "Resolve any TODOs or incomplete sections".to_string(),
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