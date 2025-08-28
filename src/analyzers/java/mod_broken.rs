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

pub struct JavaAnalyzer {
    config: AnalyzerConfig,
}

impl JavaAnalyzer {
    pub fn new(config: AnalyzerConfig) -> Self {
        Self { config }
    }

    fn is_spring_boot_component(&self, content: &str) -> bool {
        let spring_patterns = [
            r"@RestController",
            r"@Controller",
            r"@Service",
            r"@Repository",
            r"@Component",
            r"@Entity",
            r"@Configuration",
            r"@SpringBootApplication",
            r"import\s+org\.springframework",
        ];

        spring_patterns.iter().any(|pattern| {
            Regex::new(pattern).unwrap().is_match(content)
        })
    }

    fn extract_component_info(&self, file_path: &str, content: &str) -> Option<ComponentInfo> {
        if !self.is_spring_boot_component(content) && !file_path.ends_with(".java") {
            return None;
        }

        let component_name = self.extract_class_name(file_path, content)?;
        let component_type = self.classify_component_type(&component_name, content);
        let purpose = self.infer_component_purpose(&component_name, content);
        let props = self.extract_fields(content);
        let dependencies = self.extract_dependencies(content);
        let api_calls = self.extract_api_endpoints(content);
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
            hooks_used: vec![], // Java doesn't have hooks like React
            api_calls,
            complexity_score,
            implementation_status,
        })
    }

    fn extract_class_name(&self, file_path: &str, content: &str) -> Option<String> {
        // Try to extract from class declaration
        let class_patterns = [
            r"public\s+class\s+(\w+)",
            r"class\s+(\w+)",
            r"public\s+interface\s+(\w+)",
            r"interface\s+(\w+)",
            r"public\s+enum\s+(\w+)",
            r"enum\s+(\w+)",
        ];

        for pattern in &class_patterns {
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
        if content.contains("@RestController") || content.contains("@Controller") {
            return ComponentType::Service; // REST endpoints
        }
        if content.contains("@Service") {
            return ComponentType::Service;
        }
        if content.contains("@Repository") {
            return ComponentType::Service; // Data access
        }
        if content.contains("@Entity") {
            return ComponentType::Context; // Data models
        }
        if content.contains("@Configuration") {
            return ComponentType::Utility; // Configuration
        }
        if name.to_lowercase().contains("controller") {
            return ComponentType::Service;
        }
        if name.to_lowercase().contains("service") {
            return ComponentType::Service;
        }
        if name.to_lowercase().contains("repository") || name.to_lowercase().contains("dao") {
            return ComponentType::Service;
        }
        if name.to_lowercase().contains("model") || name.to_lowercase().contains("entity") {
            return ComponentType::Context;
        }
        if name.to_lowercase().contains("config") {
            return ComponentType::Utility;
        }

        ComponentType::Service
    }

    fn infer_component_purpose(&self, name: &str, content: &str) -> String {
        let name_lower = name.to_lowercase();
        
        if content.contains("@RestController") || content.contains("@Controller") {
            return format!("REST API controller handling HTTP requests for {}", name_lower);
        }
        if content.contains("@Service") {
            return format!("Business logic service for {}", name_lower);
        }
        if content.contains("@Repository") {
            return format!("Data access repository for {}", name_lower);
        }
        if content.contains("@Entity") {
            return format!("JPA entity representing {} data model", name_lower);
        }
        if content.contains("@Configuration") {
            return format!("Spring configuration class for {}", name_lower);
        }

        // Infer from class name patterns
        if name_lower.contains("controller") {
            return format!("Controller handling requests for {}", name_lower.replace("controller", ""));
        }
        if name_lower.contains("service") {
            return format!("Service providing business logic for {}", name_lower.replace("service", ""));
        }
        if name_lower.contains("repository") {
            return format!("Repository managing data persistence for {}", name_lower.replace("repository", ""));
        }
        if name_lower.contains("model") || name_lower.contains("entity") {
            return format!("Data model representing {}", name_lower);
        }

        format!("Java component for {} functionality", name_lower)
    }

    fn extract_fields(&self, content: &str) -> Vec<PropInfo> {
        let mut fields = Vec::new();
        
        // Extract field declarations
        if let Ok(re) = Regex::new(r"(?:private|protected|public)\s+(\w+(?:<[^>]+>)?)\s+(\w+)") {
            for captures in re.captures_iter(content) {
                let field_type = captures.get(1).unwrap().as_str().to_string();
                let field_name = captures.get(2).unwrap().as_str().to_string();
                
                fields.push(PropInfo {
                    name: field_name,
                    prop_type: field_type,
                    required: true, // Assume required unless annotated otherwise
                    description: None,
                });
            }
        }

        // Extract method parameters (for controllers)
        if let Ok(re) = Regex::new(r"@RequestParam(?:\([^)]*\))?\s+(\w+)\s+(\w+)") {
            for captures in re.captures_iter(content) {
                let param_type = captures.get(1).unwrap().as_str().to_string();
                let param_name = captures.get(2).unwrap().as_str().to_string();
                
                fields.push(PropInfo {
                    name: param_name,
                    prop_type: param_type,
                    required: false, // Request params are typically optional
                    description: Some("Request parameter".to_string()),
                        priority: Priority::Medium,
                });
            }
        }
        
        fields
    }

    fn extract_api_endpoints(&self, content: &str) -> Vec<ApiCall> {
        let mut endpoints = Vec::new();
        
        // Extract @RequestMapping, @GetMapping, etc.
        let mapping_patterns = [
            (r#"@GetMapping\s*\(\s*"([^"]+)""#, "GET"),
                        priority: Priority::Medium,
            (r#"@PostMapping\s*\(\s*"([^"]+)""#, "POST"),
                        priority: Priority::Medium,
            (r#"@PutMapping\s*\(\s*"([^"]+)""#, "PUT"),
                        priority: Priority::Medium,
            (r#"@DeleteMapping\s*\(\s*"([^"]+)""#, "DELETE"),
                        priority: Priority::Medium,
            (r#"@RequestMapping\s*\([^)]*value\s*=\s*"([^"]+)""#, "REQUEST"),
                        priority: Priority::Medium,
        ];

        for (pattern, method) in &mapping_patterns {
            if let Ok(re) = Regex::new(pattern) {
                for captures in re.captures_iter(content) {
                    let endpoint = captures.get(1).unwrap().as_str().to_string();
                    endpoints.push(ApiCall {
                        endpoint,
                        method: method.to_string(),
                        priority: Priority::Medium,
                        purpose: "REST API endpoint".to_string(),
                        priority: Priority::Medium,
                    });
                }
            }
        }

        // Extract external API calls
        if let Ok(re) = Regex::new(r#"restTemplate\.(?:get|post|put|delete)\w*\([^,]*"([^"]+)""#) {
            for captures in re.captures_iter(content) {
                let endpoint = captures.get(1).unwrap().as_str().to_string();
                endpoints.push(ApiCall {
                    endpoint,
                    method: "HTTP".to_string(),
                        priority: Priority::Medium,
                    purpose: "External API call".to_string(),
                        priority: Priority::Medium,
                });
            }
        }
        
        endpoints
    }

    fn extract_dependencies(&self, content: &str) -> Vec<String> {
        let mut dependencies = Vec::new();
        
        // Extract imports
        if let Ok(re) = Regex::new(r"import\s+((?:static\s+)?[\w.]+)") {
            for captures in re.captures_iter(content) {
                let import = captures.get(1).unwrap().as_str().to_string();
                if import.starts_with("org.springframework") || 
                   import.starts_with("javax.") ||
                   import.starts_with("java.") {
                    if !dependencies.contains(&import) {
                        dependencies.push(import);
                    }
                }
            }
        }

        // Extract @Autowired dependencies
        if let Ok(re) = Regex::new(r"@Autowired[^;]*?(\w+)\s+(\w+);") {
            for captures in re.captures_iter(content) {
                let dep_type = captures.get(1).unwrap().as_str().to_string();
                if !dependencies.contains(&dep_type) {
                    dependencies.push(dep_type);
                }
            }
        }
        
        dependencies
    }

    fn calculate_complexity(&self, content: &str) -> u8 {
        let mut score = 0u8;
        
        // Count methods
        let methods = Regex::new(r"(?:public|private|protected)\s+(?:\w+\s+)*\w+\s+\w+\s*\(").unwrap();
        score += methods.find_iter(content).count() as u8 * 2;
        
        // Count conditional statements
        let conditionals = Regex::new(r"\b(if|else|switch|case)\b").unwrap();
        score += conditionals.find_iter(content).count() as u8;
        
        // Count loops
        let loops = Regex::new(r"\b(for|while|do)\b").unwrap();
        score += loops.find_iter(content).count() as u8 * 2;
        
        // Count Spring annotations (complexity indicator)
        let annotations = Regex::new(r"@\w+").unwrap();
        score += annotations.find_iter(content).count() as u8;
        
        // Count exception handling
        let exceptions = Regex::new(r"\b(try|catch|throw|throws)\b").unwrap();
        score += exceptions.find_iter(content).count() as u8;
        
        // Lines of code factor
        let lines = content.lines().count();
        score += (lines / 15) as u8;
        
        score.min(100)
    }

    fn determine_implementation_status(&self, content: &str) -> ImplementationStatus {
        let todo_patterns = [
            r"TODO",
            r"FIXME", 
            r"XXX",
            r"HACK",
            r"NotImplemented",
            r"throw new UnsupportedOperationException",
            r"throw new RuntimeException\([^)]*not implemented",
        ];
        
        if todo_patterns.iter().any(|pattern| {
            Regex::new(&format!(r"(?i){}", pattern)).unwrap().is_match(content)
        }) {
            return ImplementationStatus::Todo;
        }
        
        // Check for stub methods
        if content.contains("return null;") || content.contains("return;") {
            let method_count = Regex::new(r"(?:public|private|protected)\s+(?:\w+\s+)*\w+\s+\w+\s*\(").unwrap()
                .find_iter(content).count();
            let return_nulls = Regex::new(r"return null;").unwrap().find_iter(content).count();
            
            if return_nulls > 0 && return_nulls >= method_count / 2 {
                return ImplementationStatus::Incomplete;
            }
        }
        
        // Check substantial implementation
        let lines = content.lines().filter(|line| !line.trim().is_empty()).count();
        if lines < 15 {
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
                    if component.name.to_lowercase().contains("controller") {
                        // API endpoints
                        for api_call in &component.api_calls {
                            stories.push(UserStory {
                                id: format!("US-{:03}", story_id),
                        priority: Priority::Medium,
                                name: format!("Access {} endpoint", api_call.endpoint),
                        priority: Priority::Medium,
                                description: format!("As a client, I want to {} {} so that I can interact with the system", 
                                    api_call.method, api_call.endpoint),
                        priority: Priority::Medium,
                                acceptance_criteria: vec![
                                    format!("Endpoint {} responds successfully", api_call.endpoint),
                        priority: Priority::Medium,
                                    "Proper HTTP status codes are returned".to_string(),
                        priority: Priority::Medium,
                                    "Response format matches API specification".to_string(),
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
                    } else {
                        // Business logic services
                        stories.push(UserStory {
                            id: format!("US-{:03}", story_id),
                        priority: Priority::Medium,
                            name: format!("Use {} service", component.name),
                        priority: Priority::Medium,
                            description: format!("As a system, I need {} service so that I can {}", 
                                component.name, component.purpose.to_lowercase()),
                        priority: Priority::Medium,
                            acceptance_criteria: vec![
                                format!("{} service processes requests correctly", component.name),
                        priority: Priority::Medium,
                                "Business rules are enforced".to_string(),
                        priority: Priority::Medium,
                                "Error handling works as expected".to_string(),
                        priority: Priority::Medium,
                            ],
                            priority: Priority::Medium,
                            complexity: if component.complexity_score > 40 { Complexity::Complex } else { Complexity::Medium },
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
                        description: format!("As a system, I need to persist and retrieve {} data so that I can maintain application state", 
                            component.name),
                        priority: Priority::Medium,
                        acceptance_criteria: vec![
                            format!("{} data can be created, read, updated, and deleted", component.name),
                        priority: Priority::Medium,
                            "Data validation rules are enforced".to_string(),
                        priority: Priority::Medium,
                            "Database constraints are respected".to_string(),
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
                _ => {}
            }
        }

        stories
    }

    fn generate_prd(&self, components: &[ComponentInfo], _user_stories: &[UserStory]) -> ProductRequirementDocument {
        let controllers = components.iter()
            .filter(|c| c.name.to_lowercase().contains("controller"))
            .count();
        let services = components.iter()
            .filter(|c| matches!(c.component_type, ComponentType::Service))
            .count();
        let entities = components.iter()
            .filter(|c| matches!(c.component_type, ComponentType::Context))
            .count();

        ProductRequirementDocument {
            name: "Spring Boot Application Requirements".to_string(),
                        priority: Priority::Medium,
            overview: format!("Java Spring Boot application with {} controllers, {} services, and {} entities, providing comprehensive backend functionality.", controllers, services, entities),
                        priority: Priority::Medium,
            objectives: vec![
                "Provide robust REST API endpoints".to_string(),
                        priority: Priority::Medium,
                "Implement scalable business logic".to_string(),
                        priority: Priority::Medium,
                "Ensure data persistence and integrity".to_string(),
                        priority: Priority::Medium,
                "Maintain high performance and reliability".to_string(),
                        priority: Priority::Medium,
            ],
            target_users: vec![
                "API clients (web, mobile, external systems)".to_string(),
                        priority: Priority::Medium,
                "System administrators".to_string(),
                        priority: Priority::Medium,
                "Other microservices".to_string(),
                        priority: Priority::Medium,
            ],
            features: self.extract_features_from_java_components(components),
                        priority: Priority::Medium,
            technical_requirements: vec![
                "Java Spring Boot framework".to_string(),
                        priority: Priority::Medium,
                "RESTful API architecture".to_string(),
                        priority: Priority::Medium,
                "Database integration (JPA/Hibernate)".to_string(),
                        priority: Priority::Medium,
                "Proper error handling and logging".to_string(),
                        priority: Priority::Medium,
                "Security implementation".to_string(),
                        priority: Priority::Medium,
            ],
            business_context: "Backend service designed to provide reliable API endpoints and business logic processing for enterprise applications.".to_string(),
                        priority: Priority::Medium,
        }
    }

    fn extract_features_from_java_components(&self, components: &[ComponentInfo]) -> Vec<FeatureDescription> {
        let mut features = Vec::new();
        let mut processed_types = std::collections::HashSet::new();

        for component in components {
            let feature_key = format!("{:?}-{}", component.component_type, 
                if component.name.to_lowercase().contains("controller") { "controller" } else { "other" });
            
            if processed_types.contains(&feature_key) {
                continue;
            }
            processed_types.insert(feature_key);

            if component.name.to_lowercase().contains("controller") {
                features.push(FeatureDescription {
                    name: "REST API Endpoints".to_string(),
                        priority: Priority::Medium,
                    description: "RESTful web services providing HTTP-based API access".to_string(),
                        priority: Priority::Medium,
                    user_value: "Clients can interact with the system through well-defined API endpoints".to_string(),
                        priority: Priority::Medium,
                    technical_approach: "Spring Boot controllers with proper HTTP method mappings and response handling".to_string(),
                        priority: Priority::Medium,
                    related_stories: vec![],
                });
            } else if matches!(component.component_type, ComponentType::Service) {
                features.push(FeatureDescription {
                    name: "Business Logic Services".to_string(),
                        priority: Priority::Medium,
                    description: "Core business logic implementation with proper separation of concerns".to_string(),
                        priority: Priority::Medium,
                    user_value: "Ensures consistent business rule enforcement across the application".to_string(),
                        priority: Priority::Medium,
                    technical_approach: "Service layer pattern with Spring dependency injection".to_string(),
                        priority: Priority::Medium,
                    related_stories: vec![],
                });
            } else if matches!(component.component_type, ComponentType::Context) {
                features.push(FeatureDescription {
                    name: "Data Persistence".to_string(),
                        priority: Priority::Medium,
                    description: "Robust data modeling and persistence layer".to_string(),
                        priority: Priority::Medium,
                    user_value: "Reliable data storage and retrieval with referential integrity".to_string(),
                        priority: Priority::Medium,
                    technical_approach: "JPA entities with Hibernate ORM and repository pattern".to_string(),
                        priority: Priority::Medium,
                    related_stories: vec![],
                });
            }
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
                        description: format!("Complete implementation of {} class", component.name),
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
                            "Class compiles without errors".to_string(),
                        priority: Priority::Medium,
                            "All methods are properly implemented".to_string(),
                        priority: Priority::Medium,
                            "Unit tests pass".to_string(),
                        priority: Priority::Medium,
                            "Code follows Java best practices".to_string(),
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
                        description: format!("Finish implementation of {} class", component.name),
                        priority: Priority::Medium,
                        task_type: TaskType::Feature,
                        status: component.implementation_status.clone(),
                        priority: Priority::Medium,
                        effort_estimate: Some( "Small".to_string()),
                        priority: Priority::Medium,
                        related_components: vec![component.name.clone()],
                        dependencies: vec![],
                        acceptance_criteria: vec![
                            "Resolve any TODO or FIXME comments".to_string(),
                        priority: Priority::Medium,
                            "Add proper error handling".to_string(),
                        priority: Priority::Medium,
                            "Ensure class is production-ready".to_string(),
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

impl CodebaseAnalyzer for JavaAnalyzer {
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
                if ext == "java" {
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
            project_type: ProjectType::SpringBoot,
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
                confidence_score: 0.80,
                warnings: Vec::new(),
                        priority: Priority::Medium,
            },
        })
    }

    fn supported_extensions(&self) -> Vec<&str> {
        vec!["java"]
    }

    fn can_analyze(&self, project_path: &str) -> bool {
        // Check for pom.xml or build.gradle with Spring Boot
        let pom_path = Path::new(project_path).join("pom.xml");
        if let Ok(content) = fs::read_to_string(pom_path) {
            if content.contains("spring-boot") {
                return true;
            }
        }
        
        let gradle_path = Path::new(project_path).join("build.gradle");
        if let Ok(content) = fs::read_to_string(gradle_path) {
            if content.contains("spring-boot") {
                return true;
            }
        }
        
        // Check for Spring Boot annotations in Java files
        for entry in WalkDir::new(project_path)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if let Some(ext) = entry.path().extension() {
                if ext == "java" {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if content.contains("@SpringBootApplication") || 
                           content.contains("org.springframework") {
                            return true;
                        }
                    }
                }
            }
        }
        
        false
    }
}