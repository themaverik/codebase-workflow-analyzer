use crate::core::{
    CodebaseAnalysis, CodebaseAnalyzer, ComponentInfo, ComponentType, PropInfo, ApiCall,
    ImplementationStatus, UserStory, Task, TaskType, Priority, Complexity, FeatureDescription,
    ProductRequirementDocument, AnalysisMetadata, ProjectType, AnalyzerConfig,
    FrameworkAnalysis, BusinessContext, ImplementationAnalysis, StatusIntelligence, IntegrationPoints,
    DetectedFramework, UsageExtent, EndpointAnalysis, EntityAnalysis, ComponentRelationship, DataFlowAnalysis,
    FeatureStatus, TechnicalDebt, ExternalService, InternalDependency, ConfigFile
};
use regex::Regex;
use std::fs;
use walkdir::WalkDir;
use anyhow::Result;

pub struct TypeScriptAnalyzer {
    config: AnalyzerConfig,
}

impl TypeScriptAnalyzer {
    pub fn new(config: AnalyzerConfig) -> Self {
        Self { config }
    }

    fn extract_component_info(&self, file_path: &str, content: &str) -> Option<ComponentInfo> {
        // Extract component name from file path
        let component_name = std::path::Path::new(file_path)
            .file_stem()?
            .to_str()?
            .to_string();

        // Determine component type based on patterns
        let component_type = self.determine_component_type(content, &component_name);
        let purpose = self.infer_component_purpose(content, &component_name, &component_type);
        let props = self.extract_props(content);
        let hooks_used = self.extract_hooks(content);
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
            props,
            hooks_used,
            api_calls,
            complexity_score,
            implementation_status,
        })
    }

    fn determine_component_type(&self, content: &str, name: &str) -> ComponentType {
        let name_lower = name.to_lowercase();
        
        if name_lower.contains("page") || content.contains("useRouter") || content.contains("Router") {
            ComponentType::Page
        } else if name_lower.contains("layout") || content.contains("children") {
            ComponentType::Layout
        } else if name_lower.contains("form") || content.contains("onSubmit") {
            ComponentType::Form
        } else if name_lower.contains("nav") || name_lower.contains("menu") {
            ComponentType::Navigation
        } else if name_lower.contains("modal") || name_lower.contains("dialog") {
            ComponentType::Modal
        } else if name_lower.contains("hook") || name_lower.starts_with("use") {
            ComponentType::Hook
        } else if name_lower.contains("context") || content.contains("createContext") {
            ComponentType::Context
        } else if name_lower.contains("service") || name_lower.contains("api") {
            ComponentType::Service
        } else if content.contains("return") && (content.contains("jsx") || content.contains("<")) {
            ComponentType::Display
        } else {
            ComponentType::Utility
        }
    }

    fn infer_component_purpose(&self, content: &str, name: &str, component_type: &ComponentType) -> String {
        match component_type {
            ComponentType::Page => format!("Renders the {} page interface", name),
            ComponentType::Layout => format!("Provides layout structure for {}", name),
            ComponentType::Form => format!("Handles form interactions for {}", name),
            ComponentType::Navigation => format!("Provides navigation functionality for {}", name),
            ComponentType::Modal => format!("Displays modal dialog for {}", name),
            ComponentType::Hook => format!("Custom hook providing {} functionality", name),
            ComponentType::Context => format!("Context provider for {} state management", name),
            ComponentType::Service => format!("Service layer for {} operations", name),
            ComponentType::Display => format!("Displays {} information", name),
            ComponentType::Utility => format!("Utility functions for {}", name),
        }
    }

    fn extract_props(&self, content: &str) -> Vec<PropInfo> {
        let mut props = Vec::new();
        
        // Try to find interface definitions for props
        if let Ok(re) = Regex::new(r"interface\s+(\w+Props)\s*\{([^}]+)\}") {
            for captures in re.captures_iter(content) {
                let props_content = captures.get(2).unwrap().as_str();
                props.extend(self.parse_interface_props(props_content));
            }
        }
        
        // Try to find destructured props in function parameters
        if let Ok(re) = Regex::new(r"function\s+\w+\s*\(\s*\{\s*([^}]+)\s*\}") {
            for captures in re.captures_iter(content) {
                let props_content = captures.get(1).unwrap().as_str();
                props.extend(self.parse_destructured_props(props_content));
            }
        }
        
        props
    }

    fn parse_interface_props(&self, props_content: &str) -> Vec<PropInfo> {
        let mut props = Vec::new();
        
        for line in props_content.lines() {
            let line = line.trim();
            if let Some(colon_pos) = line.find(':') {
                let name_part = &line[..colon_pos].trim();
                let type_part = &line[colon_pos + 1..].trim().trim_end_matches(',').trim_end_matches(';');
                
                let required = !name_part.ends_with('?');
                let name = name_part.trim_end_matches('?').to_string();
                
                props.push(PropInfo {
                    name,
                    prop_type: type_part.to_string(),
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
                    prop_type: "unknown".to_string(),
                    required: true,
                    description: None,
                });
            }
        }
        
        props
    }

    fn extract_hooks(&self, content: &str) -> Vec<String> {
        let mut hooks = Vec::new();
        let hook_patterns = ["useState", "useEffect", "useContext", "useReducer", "useCallback", "useMemo", "useRef"];
        
        for pattern in &hook_patterns {
            if content.contains(pattern) {
                hooks.push(pattern.to_string());
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
                    purpose: "Data fetching".to_string(),
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
                });
            }
        }
        
        api_calls
    }

    fn extract_dependencies(&self, content: &str) -> Vec<String> {
        let mut dependencies = Vec::new();
        
        // Extract import statements
        if let Ok(re) = Regex::new(r#"import\s+.*?\s+from\s+['"`]([^'"`]+)['"`]"#) {
            for captures in re.captures_iter(content) {
                let dep = captures.get(1).unwrap().as_str();
                if !dep.starts_with('.') && !dep.starts_with('/') {
                    dependencies.push(dep.to_string());
                }
            }
        }
        
        dependencies
    }

    fn calculate_complexity(&self, content: &str) -> u8 {
        let mut complexity = 10; // Base complexity
        
        // Add complexity for control structures
        complexity += content.matches("if ").count() as u8 * 2;
        complexity += content.matches("for ").count() as u8 * 3;
        complexity += content.matches("while ").count() as u8 * 3;
        complexity += content.matches("switch ").count() as u8 * 4;
        complexity += content.matches("try ").count() as u8 * 2;
        
        // Add complexity for hooks usage
        complexity += content.matches("useState").count() as u8 * 1;
        complexity += content.matches("useEffect").count() as u8 * 2;
        
        // Add complexity for nested functions
        complexity += content.matches("function").count() as u8 * 1;
        complexity += content.matches("=>").count() as u8 * 1;
        
        complexity.min(100)
    }

    fn determine_implementation_status(&self, content: &str) -> ImplementationStatus {
        if content.contains("TODO") || content.contains("FIXME") {
            ImplementationStatus::Todo
        } else if content.contains("throw new Error") || content.contains("console.error") {
            ImplementationStatus::InProgress
        } else if content.contains("test") || content.contains("spec") {
            ImplementationStatus::Complete
        } else if content.len() < 100 {
            ImplementationStatus::Incomplete
        } else {
            ImplementationStatus::Complete
        }
    }
}

impl CodebaseAnalyzer for TypeScriptAnalyzer {
    fn analyze(&self, project_path: &str) -> Result<CodebaseAnalysis> {
        let mut components = Vec::new();
        let mut files_analyzed = 0;
        let mut lines_of_code = 0;

        // Walk through the project directory
        for entry in WalkDir::new(project_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            
            if let Some(ext) = path.extension() {
                if ["ts", "tsx", "js", "jsx"].contains(&ext.to_str().unwrap_or("")) {
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

        // Enhanced analysis
        let framework_analysis = self.analyze_frameworks(project_path, &components);
        let business_context = self.infer_business_context(&components, &user_stories);
        let implementation_analysis = self.analyze_implementation(&components);
        let status_intelligence = self.analyze_status(&components, &tasks);
        let integration_points = self.analyze_integration_points(project_path);

        // Determine project type based on framework analysis
        let project_type = if framework_analysis.detected_frameworks.iter().any(|f| f.name == "NestJS") {
            ProjectType::NestJS
        } else if framework_analysis.detected_frameworks.iter().any(|f| f.name == "Next.js") {
            ProjectType::NextJS
        } else if framework_analysis.detected_frameworks.iter().any(|f| f.name == "React") {
            ProjectType::React
        } else {
            ProjectType::Unknown
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
                analyzer_version: "0.1.0".to_string(),
                files_analyzed,
                lines_of_code,
                confidence_score: 0.85,
                warnings: Vec::new(),
            },
            framework_analysis,
            business_context,
            implementation_analysis,
            status_intelligence,
            integration_points,
        })
    }

    fn supported_extensions(&self) -> Vec<&str> {
        vec!["ts", "tsx", "js", "jsx"]
    }

    fn can_analyze(&self, project_path: &str) -> bool {
        // Check for package.json with React dependency
        let package_json_path = std::path::Path::new(project_path).join("package.json");
        if let Ok(content) = fs::read_to_string(package_json_path) {
            content.contains("\"react\"")
        } else {
            false
        }
    }
}

impl TypeScriptAnalyzer {
    fn generate_user_stories(&self, components: &[ComponentInfo]) -> Vec<UserStory> {
        let mut stories = Vec::new();
        let mut story_id = 1;

        // Use business logic analysis to categorize components
        let business_analysis = self.analyze_business_logic_patterns(components);
        let mut domain_components = std::collections::HashMap::new();
        
        // Categorize components based on file patterns and business logic
        for component in components {
            let name_lower = component.name.to_lowercase();
            let file_path_lower = component.file_path.to_lowercase();
            
            // User/Auth domain components
            if name_lower.contains("user") || name_lower.contains("auth") || 
               file_path_lower.contains("/user/") || file_path_lower.contains("/auth/") ||
               name_lower.contains("session") || name_lower.contains("jwt") {
                domain_components.entry("UserAPI").or_insert(Vec::new()).push(component);
            }
            // Property domain components
            else if name_lower.contains("property") || file_path_lower.contains("/property/") {
                domain_components.entry("PropertyAPI").or_insert(Vec::new()).push(component);
            }
            // Location domain components
            else if name_lower.contains("location") || file_path_lower.contains("/location/") {
                domain_components.entry("LocationAPI").or_insert(Vec::new()).push(component);
            }
            // Image domain components
            else if name_lower.contains("image") || file_path_lower.contains("/image/") {
                domain_components.entry("ImageAPI").or_insert(Vec::new()).push(component);
            }
            // Generic API components
            else if name_lower.contains("controller") || name_lower.contains("service") || 
                     name_lower.contains("repository") || name_lower.contains("middleware") {
                domain_components.entry("API").or_insert(Vec::new()).push(component);
            }
        }

        // Generate business-focused user stories based on domain analysis
        if let Some(_user_components) = domain_components.get("User").or(domain_components.get("UserAPI")) {
            // User authentication and registration
            stories.push(UserStory {
                id: format!("US-{:03}", story_id),
                title: "User authentication and registration".to_string(),
                description: "As an end user, I want to register for an account and authenticate securely so that I can access protected features and maintain my personal data".to_string(),
                acceptance_criteria: vec![
                    "User can register with email and secure password".to_string(),
                    "User receives email verification for account activation".to_string(),
                    "User can log in with valid credentials and receive authentication token".to_string(),
                    "Failed login attempts are handled securely with appropriate error messages".to_string(),
                    "User can reset password through secure email process".to_string(),
                ],
                priority: Priority::Critical,
                complexity: Complexity::Complex,
                related_components: domain_components.get("UserAPI").unwrap_or(&vec![]).iter().map(|c| c.name.clone()).collect(),
                status: self.determine_feature_status(&domain_components.get("UserAPI").unwrap_or(&vec![])),
                inferred_from: domain_components.get("UserAPI").unwrap_or(&vec![]).iter().map(|c| c.file_path.clone()).collect(),
            });
            story_id += 1;

            // User profile management
            stories.push(UserStory {
                id: format!("US-{:03}", story_id),
                title: "Manage user profile and account settings".to_string(),
                description: "As an authenticated user, I want to manage my profile information and account settings so that I can keep my data current and customize my experience".to_string(),
                acceptance_criteria: vec![
                    "User can view and update personal information (name, email, etc.)".to_string(),
                    "User can change password with current password verification".to_string(),
                    "User can manage account preferences and settings".to_string(),
                    "Profile updates are validated and saved securely".to_string(),
                    "User can delete account with proper confirmation process".to_string(),
                ],
                priority: Priority::High,
                complexity: Complexity::Medium,
                related_components: domain_components.get("UserAPI").unwrap_or(&vec![]).iter().map(|c| c.name.clone()).collect(),
                status: self.determine_feature_status(&domain_components.get("UserAPI").unwrap_or(&vec![])),
                inferred_from: domain_components.get("UserAPI").unwrap_or(&vec![]).iter().map(|c| c.file_path.clone()).collect(),
            });
            story_id += 1;

            // Session management
            stories.push(UserStory {
                id: format!("US-{:03}", story_id),
                title: "Secure session and token management".to_string(),
                description: "As a client application, I want secure session and token management so that I can maintain user authentication state and handle token refresh seamlessly".to_string(),
                acceptance_criteria: vec![
                    "JWT tokens are issued upon successful authentication".to_string(),
                    "Token refresh mechanism handles expired tokens automatically".to_string(),
                    "Session state is maintained securely across requests".to_string(),
                    "User can log out and invalidate sessions properly".to_string(),
                    "Concurrent sessions are handled according to security policies".to_string(),
                ],
                priority: Priority::High,
                complexity: Complexity::Complex,
                related_components: domain_components.get("UserAPI").unwrap_or(&vec![]).iter().map(|c| c.name.clone()).collect(),
                status: self.determine_feature_status(&domain_components.get("UserAPI").unwrap_or(&vec![])),
                inferred_from: domain_components.get("UserAPI").unwrap_or(&vec![]).iter().map(|c| c.file_path.clone()).collect(),
            });
            story_id += 1;
        } else if let Some(_property_components) = domain_components.get("Property").or(domain_components.get("PropertyAPI")) {
            // Core property management story
            stories.push(UserStory {
                id: format!("US-{:03}", story_id),
                title: "Manage real estate property listings".to_string(),
                description: "As a property manager, I want to create, update, and manage property listings so that I can efficiently handle real estate inventory and provide accurate information to potential buyers or renters".to_string(),
                acceptance_criteria: vec![
                    "Property manager can create new property listings with all required details".to_string(),
                    "Property information can be updated (price, status, description, features)".to_string(),
                    "Property status can be managed (available, sold, rented, under contract)".to_string(),
                    "Property listings include comprehensive details (area, location, amenities)".to_string(),
                    "Data validation ensures property information integrity".to_string(),
                ],
                priority: Priority::Critical,
                complexity: Complexity::Complex,
                related_components: domain_components.get("PropertyAPI").unwrap_or(&vec![]).iter().map(|c| c.name.clone()).collect(),
                status: self.determine_feature_status(&domain_components.get("PropertyAPI").unwrap_or(&vec![])),
                inferred_from: domain_components.get("PropertyAPI").unwrap_or(&vec![]).iter().map(|c| c.file_path.clone()).collect(),
            });
            story_id += 1;

            // Property search and discovery
            stories.push(UserStory {
                id: format!("US-{:03}", story_id),
                title: "Search and filter properties".to_string(),
                description: "As a property seeker (buyer/renter), I want to search for properties using various criteria so that I can find properties that match my specific needs and preferences".to_string(),
                acceptance_criteria: vec![
                    "Users can search properties by location, price range, and property type".to_string(),
                    "Advanced filters available (area size, number of rooms, amenities)".to_string(),
                    "Search results display key property information and images".to_string(),
                    "Search performance is optimized for large property inventories".to_string(),
                    "Users can save search criteria and receive notifications for new matches".to_string(),
                ],
                priority: Priority::High,
                complexity: Complexity::Medium,
                related_components: domain_components.get("PropertyAPI").unwrap_or(&vec![]).iter().map(|c| c.name.clone()).collect(),
                status: self.determine_feature_status(&domain_components.get("PropertyAPI").unwrap_or(&vec![])),
                inferred_from: domain_components.get("PropertyAPI").unwrap_or(&vec![]).iter().map(|c| c.file_path.clone()).collect(),
            });
            story_id += 1;
        }

        if let Some(_location_components) = domain_components.get("Location").or(domain_components.get("LocationAPI")) {
            stories.push(UserStory {
                id: format!("US-{:03}", story_id),
                title: "Manage property location data".to_string(),
                description: "As a system user, I want accurate geographic and address information for properties so that I can provide location-based services and help users find properties in desired areas".to_string(),
                acceptance_criteria: vec![
                    "Property locations include detailed address information".to_string(),
                    "Geographic coordinates support map-based property display".to_string(),
                    "Location data supports proximity-based searches".to_string(),
                    "Address validation ensures data accuracy".to_string(),
                    "Integration with mapping services for enhanced user experience".to_string(),
                ],
                priority: Priority::High,
                complexity: Complexity::Medium,
                related_components: domain_components.get("LocationAPI").unwrap_or(&vec![]).iter().map(|c| c.name.clone()).collect(),
                status: self.determine_feature_status(&domain_components.get("LocationAPI").unwrap_or(&vec![])),
                inferred_from: domain_components.get("LocationAPI").unwrap_or(&vec![]).iter().map(|c| c.file_path.clone()).collect(),
            });
            story_id += 1;
        }

        if let Some(_image_components) = domain_components.get("Image").or(domain_components.get("ImageAPI")) {
            stories.push(UserStory {
                id: format!("US-{:03}", story_id),
                title: "Manage property images and media".to_string(),
                description: "As a property manager, I want to upload, organize, and display high-quality images of properties so that potential buyers/renters can get a comprehensive visual understanding of the property".to_string(),
                acceptance_criteria: vec![
                    "Multiple images can be uploaded for each property".to_string(),
                    "Images are optimized for web display and mobile viewing".to_string(),
                    "Image metadata includes descriptions and categorization".to_string(),
                    "Primary/featured image can be designated for listings".to_string(),
                    "Image management supports reordering and deletion".to_string(),
                ],
                priority: Priority::High,
                complexity: Complexity::Medium,
                related_components: domain_components.get("ImageAPI").unwrap_or(&vec![]).iter().map(|c| c.name.clone()).collect(),
                status: self.determine_feature_status(&domain_components.get("ImageAPI").unwrap_or(&vec![])),
                inferred_from: domain_components.get("ImageAPI").unwrap_or(&vec![]).iter().map(|c| c.file_path.clone()).collect(),
            });
            story_id += 1;
        }

        // If we have API components but no specific domain, create a general microservice story
        if domain_components.contains_key("API") && stories.is_empty() {
            stories.push(UserStory {
                id: format!("US-{:03}", story_id),
                title: "Provide comprehensive REST API service".to_string(),
                description: "As an API consumer (frontend application, mobile app, or third-party service), I want access to a reliable REST API so that I can integrate with the system's data and functionality".to_string(),
                acceptance_criteria: vec![
                    "RESTful endpoints follow standard HTTP methods and status codes".to_string(),
                    "API provides comprehensive CRUD operations for core entities".to_string(),
                    "Request/response validation ensures data integrity".to_string(),
                    "API documentation is available and up-to-date".to_string(),
                    "Error handling provides meaningful feedback to consumers".to_string(),
                ],
                priority: Priority::High,
                complexity: Complexity::Complex,
                related_components: domain_components.get("API").unwrap_or(&vec![]).iter().map(|c| c.name.clone()).collect(),
                status: self.determine_feature_status(&domain_components.get("API").unwrap_or(&vec![])),
                inferred_from: domain_components.get("API").unwrap_or(&vec![]).iter().map(|c| c.file_path.clone()).collect(),
            });
            story_id += 1;
        }

        // Fallback: if no domain-specific patterns found, generate generic stories for major components
        if stories.is_empty() {
            for component in components.iter().filter(|c| c.complexity_score > 30) {
                stories.push(UserStory {
                    id: format!("US-{:03}", story_id),
                    title: format!("Utilize {} functionality", component.name),
                    description: format!("As a system user, I want to access {} capabilities so that I can {}", 
                        component.name, component.purpose.to_lowercase()),
                    acceptance_criteria: vec![
                        format!("{} operates reliably and efficiently", component.name),
                        "User interface is intuitive and responsive".to_string(),
                        "Data integrity is maintained throughout operations".to_string(),
                    ],
                    priority: Priority::Medium,
                    complexity: if component.complexity_score > 60 { Complexity::Complex } else { Complexity::Medium },
                    related_components: vec![component.name.clone()],
                    status: component.implementation_status.clone(),
                    inferred_from: vec![component.file_path.clone()],
                });
                story_id += 1;
            }
        }

        stories
    }

    fn determine_feature_status(&self, components: &[&ComponentInfo]) -> ImplementationStatus {
        if components.is_empty() {
            return ImplementationStatus::Todo;
        }
        
        let complete_count = components.iter().filter(|c| matches!(c.implementation_status, ImplementationStatus::Complete)).count();
        let in_progress_count = components.iter().filter(|c| matches!(c.implementation_status, ImplementationStatus::InProgress)).count();
        let total_count = components.len();
        
        if complete_count == total_count {
            ImplementationStatus::Complete
        } else if complete_count > 0 || in_progress_count > 0 {
            ImplementationStatus::InProgress
        } else {
            ImplementationStatus::Todo
        }
    }

    fn generate_prd(&self, components: &[ComponentInfo]) -> ProductRequirementDocument {
        let pages = components.iter()
            .filter(|c| matches!(c.component_type, ComponentType::Page))
            .count();
        let forms = components.iter()
            .filter(|c| matches!(c.component_type, ComponentType::Form))
            .count();

        ProductRequirementDocument {
            title: "Product Requirements Document".to_string(),
            overview: format!("React-based web application with {} pages and {} forms, providing comprehensive user interface functionality.", pages, forms),
            objectives: vec![
                "Provide intuitive user interface".to_string(),
                "Ensure responsive design across devices".to_string(),
                "Maintain high performance and accessibility".to_string(),
            ],
            target_users: vec![
                "End users seeking web-based functionality".to_string(),
                "Administrators managing system data".to_string(),
            ],
            features: self.extract_features_from_components(components),
            technical_requirements: vec![
                "React-based frontend framework".to_string(),
                "TypeScript for type safety".to_string(),
                "Modern browser compatibility".to_string(),
                "Responsive design implementation".to_string(),
            ],
            business_context: "Web application designed to provide efficient user experience through modern React components and interactions.".to_string(),
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
                    description: "Multi-page application with navigation between different views".to_string(),
                    user_value: "Users can access different sections of the application efficiently".to_string(),
                    technical_approach: "React Router for client-side routing and navigation".to_string(),
                    related_stories: vec![], // Would be populated with actual story IDs
                },
                ComponentType::Form => FeatureDescription {
                    name: "Data Input Forms".to_string(),
                    description: "Interactive forms for data collection and submission".to_string(),
                    user_value: "Users can input and submit data through validated forms".to_string(),
                    technical_approach: "Controlled components with validation and error handling".to_string(),
                    related_stories: vec![],
                },
                ComponentType::Layout => FeatureDescription {
                    name: "Application Layout".to_string(),
                    description: "Consistent layout structure across the application".to_string(),
                    user_value: "Users experience consistent navigation and visual structure".to_string(),
                    technical_approach: "Reusable layout components with flexible content areas".to_string(),
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
                        name: format!("Implement {}", component.name),
                        description: format!("Complete implementation of {} component", component.name),
                        task_type: TaskType::Feature,
                        status: component.implementation_status.clone(),
                        effort_estimate: Some(if component.complexity_score > 50 { "Large".to_string() } else { "Medium".to_string() }),
                        priority: Priority::Medium,
                        related_components: vec![component.name.clone()],
                        dependencies: component.dependencies.clone(),
                        acceptance_criteria: vec![
                            "Component renders without errors".to_string(),
                            "All props are properly handled".to_string(),
                            "Component meets design requirements".to_string(),
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
                            "Resolve any TODOs or incomplete sections".to_string(),
                            "Add proper error handling".to_string(),
                            "Ensure component is production-ready".to_string(),
                        ],
                    });
                    task_id += 1;
                },
                _ => {}
            }
        }

        tasks
    }

    // Enhanced analysis methods
    fn analyze_frameworks(&self, project_path: &str, components: &[ComponentInfo]) -> FrameworkAnalysis {
        let mut detected_frameworks = Vec::new();
        let mut confidence_scores = std::collections::HashMap::new();

        // Analyze package.json for framework detection
        if let Ok(package_json) = fs::read_to_string(format!("{}/package.json", project_path)) {
            // React detection
            if package_json.contains("\"react\"") {
                let version = self.extract_version_from_package_json(&package_json, "react");
                let confidence = if package_json.contains("\"@types/react\"") { 0.95 } else { 0.85 };
                
                detected_frameworks.push(DetectedFramework {
                    name: "React".to_string(),
                    version,
                    confidence,
                    evidence: vec!["package.json".to_string(), "*.tsx files".to_string()],
                    usage_extent: UsageExtent::Core,
                });
                confidence_scores.insert("React".to_string(), confidence);
            }

            // Next.js detection
            if package_json.contains("\"next\"") {
                let version = self.extract_version_from_package_json(&package_json, "next");
                detected_frameworks.push(DetectedFramework {
                    name: "Next.js".to_string(),
                    version,
                    confidence: 0.9,
                    evidence: vec!["package.json".to_string(), "next.config.js".to_string()],
                    usage_extent: UsageExtent::Core,
                });
                confidence_scores.insert("Next.js".to_string(), 0.9);
            }

            // NestJS detection
            if package_json.contains("\"@nestjs/core\"") || package_json.contains("\"@nestjs/common\"") {
                let version = self.extract_version_from_package_json(&package_json, "@nestjs/core");
                let mut confidence = 0.95;
                let mut evidence = vec!["package.json".to_string()];

                // Check for additional NestJS evidence
                if package_json.contains("\"@nestjs/platform-express\"") {
                    evidence.push("Express platform detected".to_string());
                }
                if package_json.contains("\"@nestjs/platform-fastify\"") {
                    evidence.push("Fastify platform detected".to_string());
                }
                if package_json.contains("\"@nestjs/swagger\"") {
                    evidence.push("Swagger integration".to_string());
                }
                if package_json.contains("\"@nestjs/typeorm\"") || package_json.contains("\"@nestjs/mongoose\"") {
                    evidence.push("Database integration".to_string());
                }

                detected_frameworks.push(DetectedFramework {
                    name: "NestJS".to_string(),
                    version,
                    confidence,
                    evidence,
                    usage_extent: UsageExtent::Core,
                });
                confidence_scores.insert("NestJS".to_string(), confidence);
            }

            // TypeScript detection
            if package_json.contains("\"typescript\"") || package_json.contains("\"@types/") {
                let version = self.extract_version_from_package_json(&package_json, "typescript");
                detected_frameworks.push(DetectedFramework {
                    name: "TypeScript".to_string(),
                    version,
                    confidence: 0.95,
                    evidence: vec!["package.json".to_string(), "*.ts files".to_string()],
                    usage_extent: UsageExtent::Core,
                });
                confidence_scores.insert("TypeScript".to_string(), 0.95);
            }
        }

        // Deno framework detection
        self.detect_deno_frameworks(project_path, &mut detected_frameworks, &mut confidence_scores);

        // Check for NestJS patterns in source code files
        let nestjs_patterns = self.detect_nestjs_patterns(project_path);
        if !nestjs_patterns.is_empty() && !detected_frameworks.iter().any(|f| f.name == "NestJS") {
            // Found NestJS patterns in code but not in package.json
            detected_frameworks.push(DetectedFramework {
                name: "NestJS".to_string(),
                version: None,
                confidence: 0.85, // Lower confidence without package.json evidence
                evidence: nestjs_patterns,
                usage_extent: UsageExtent::Core,
            });
            confidence_scores.insert("NestJS".to_string(), 0.85);
        }

        let architecture_pattern = if detected_frameworks.iter().any(|f| f.name == "Next.js") {
            "SSR/SSG with React".to_string()
        } else if detected_frameworks.iter().any(|f| f.name == "NestJS") {
            "Modular Backend API with NestJS".to_string()
        } else if detected_frameworks.iter().any(|f| f.name == "Danet") {
            "Deno Backend API with Danet (NestJS-like)".to_string()
        } else if detected_frameworks.iter().any(|f| f.name == "Fresh") {
            "Deno Fullstack with Fresh".to_string()
        } else if detected_frameworks.iter().any(|f| f.name == "Oak") {
            "Deno Backend API with Oak".to_string()
        } else if detected_frameworks.iter().any(|f| f.name.starts_with("Deno")) {
            "Deno Application".to_string()
        } else if detected_frameworks.iter().any(|f| f.name == "React") {
            "SPA with React".to_string()
        } else {
            "Web Application".to_string()
        };

        FrameworkAnalysis {
            detected_frameworks,
            confidence_scores,
            architecture_pattern,
        }
    }

    fn infer_business_context(&self, components: &[ComponentInfo], _user_stories: &[UserStory]) -> BusinessContext {
        let mut evidence = Vec::new();
        let mut user_personas = Vec::new();
        let mut user_journeys = Vec::new();

        // Analyze actual file structure and business logic patterns
        let mut domain_indicators = std::collections::HashMap::new();
        let mut framework_type = "Generic".to_string();
        
        // Analyze controllers, services, and middleware to understand business domain
        let business_analysis = self.analyze_business_logic_patterns(components);
        
        // Update framework type based on detected patterns
        framework_type = business_analysis.framework_type.clone();
        
        // Add evidence from business logic analysis
        evidence.extend(business_analysis.evidence.clone());
        
        // Determine domain based on business logic analysis
        for (domain, score) in business_analysis.domain_scores {
            *domain_indicators.entry(domain).or_insert(0) += score;
        }
        
        // Fallback to basic component analysis if no clear patterns found
        if domain_indicators.is_empty() {
            for component in components {
                let name_lower = component.name.to_lowercase();
                let file_path_lower = component.file_path.to_lowercase();
                
                // Generic architectural patterns
                if name_lower.contains("controller") || name_lower.contains("service") || name_lower.contains("repository") {
                    *domain_indicators.entry("API".to_string()).or_insert(0) += 1;
                }
                
                // Determine framework type from patterns
                if name_lower.contains("controller") && (file_path_lower.contains("deno") || 
                   component.dependencies.iter().any(|d| d.contains("danet"))) {
                    framework_type = "Danet".to_string();
                } else if name_lower.contains("controller") {
                    framework_type = "NestJS".to_string();
                } else if name_lower.contains("component") && name_lower.ends_with("tsx") {
                    framework_type = "React".to_string();
                }
            }
        }

        // Determine the primary business domain
        let primary_domain = domain_indicators.iter()
            .max_by_key(|(_, count)| *count)
            .map(|(domain, _)| domain.as_str())
            .unwrap_or("Generic");

        // Generate business context based on detected domain
        let (product_type, confidence, business_domain) = match primary_domain {
            "UserManagement" => {
                user_personas.extend(vec![
                    "End User".to_string(), 
                    "System Administrator".to_string(), 
                    "Client Application".to_string(),
                    "Frontend Developer".to_string()
                ]);
                user_journeys.extend(vec![
                    "User registers for new account and verifies email".to_string(),
                    "User logs in using credentials and receives authentication token".to_string(),
                    "User manages profile information and account settings".to_string(),
                    "Administrator manages user accounts and permissions".to_string(),
                    "Client application authenticates users and manages sessions".to_string(),
                ]);
                (format!("{} User Authentication & Management Service", framework_type), 0.95, "User Management".to_string())
            },
            "Authentication" => {
                user_personas.extend(vec![
                    "End User".to_string(), 
                    "Client Application".to_string(), 
                    "System Administrator".to_string()
                ]);
                user_journeys.extend(vec![
                    "User authenticates using credentials to access protected resources".to_string(),
                    "Client application obtains and refreshes authentication tokens".to_string(),
                    "Administrator manages authentication policies and security settings".to_string(),
                ]);
                (format!("{} Authentication Service", framework_type), 0.9, "Authentication".to_string())
            },
            "RealEstate" => {
                user_personas.extend(vec![
                    "Property Buyer/Renter".to_string(), 
                    "Property Owner/Seller".to_string(), 
                    "Real Estate Agent".to_string(),
                    "Property Manager".to_string()
                ]);
                user_journeys.extend(vec![
                    "Property Buyer searches for properties by location, price, and features".to_string(),
                    "Property Owner lists their property with photos and details".to_string(),
                    "Real Estate Agent manages property listings and client inquiries".to_string(),
                    "Property Manager tracks property status and maintenance".to_string(),
                ]);
                (format!("{} Real Estate Property Management System", framework_type), 0.9, "Real Estate".to_string())
            },
            "Ecommerce" => {
                user_personas.extend(vec!["Customer".to_string(), "Store Owner".to_string(), "Admin".to_string()]);
                user_journeys.extend(vec![
                    "Customer browses products and makes purchases".to_string(),
                    "Store owner manages inventory and orders".to_string(),
                ]);
                ("E-commerce Platform".to_string(), 0.85, "E-commerce".to_string())
            },
            "Healthcare" => {
                user_personas.extend(vec!["Patient".to_string(), "Doctor".to_string(), "Admin".to_string()]);
                user_journeys.extend(vec![
                    "Patient books appointments and views medical records".to_string(),
                    "Doctor manages patient care and medical records".to_string(),
                ]);
                ("Healthcare Management System".to_string(), 0.85, "Healthcare".to_string())
            },
            "Financial" => {
                user_personas.extend(vec!["Account Holder".to_string(), "Financial Advisor".to_string(), "Admin".to_string()]);
                user_journeys.extend(vec![
                    "Account holder manages transactions and balances".to_string(),
                    "Financial advisor provides account analysis and recommendations".to_string(),
                ]);
                ("Financial Management System".to_string(), 0.85, "Financial Services".to_string())
            },
            "Education" => {
                user_personas.extend(vec!["Student".to_string(), "Teacher".to_string(), "Admin".to_string()]);
                user_journeys.extend(vec![
                    "Student enrolls in courses and completes assignments".to_string(),
                    "Teacher manages courses and grades student work".to_string(),
                ]);
                ("Educational Management System".to_string(), 0.85, "Education".to_string())
            },
            "ContentManagement" => {
                user_personas.extend(vec!["Content Creator".to_string(), "Reader".to_string(), "Editor".to_string()]);
                user_journeys.extend(vec![
                    "Content creator publishes articles and manages content".to_string(),
                    "Reader discovers and consumes content".to_string(),
                ]);
                ("Content Management System".to_string(), 0.8, "Content Management".to_string())
            },
            _ => {
                // Fallback based on architectural patterns
                if domain_indicators.get("API").unwrap_or(&0) > &2 {
                    user_personas.extend(vec![
                        "API Client".to_string(), 
                        "Frontend Application".to_string(), 
                        "Mobile App".to_string(),
                        "System Administrator".to_string()
                    ]);
                    user_journeys.extend(vec![
                        "Client applications consume REST API endpoints for data operations".to_string(),
                        "Third-party integrations access business data via API".to_string(),
                        "Administrators monitor API performance and manage system health".to_string(),
                    ]);
                    (format!("{} REST API Service", framework_type), 0.7, "API Service".to_string())
                } else {
                    user_personas.push("End User".to_string());
                    user_journeys.push("User interacts with the application interface".to_string());
                    ("Web Application".to_string(), 0.6, "Generic".to_string())
                }
            }
        };

        BusinessContext {
            inferred_product_type: product_type,
            confidence,
            evidence,
            primary_user_personas: user_personas,
            user_journeys_discovered: user_journeys,
            business_domain,
        }
    }

    fn analyze_implementation(&self, components: &[ComponentInfo]) -> ImplementationAnalysis {
        let api_endpoints = self.extract_api_endpoints_from_components(components);
        let entities = self.extract_entities_from_components(components);
        let relationships = self.analyze_component_relationships(components);
        let data_flows = self.analyze_data_flows(components);

        ImplementationAnalysis {
            api_endpoints,
            database_entities: entities,
            component_relationships: relationships,
            data_flow: data_flows,
        }
    }

    fn analyze_status(&self, components: &[ComponentInfo], tasks: &[Task]) -> StatusIntelligence {
        let mut completed_features = Vec::new();
        let mut in_progress_features = Vec::new();
        let mut todo_features = Vec::new();
        let mut technical_debt = Vec::new();

        let mut completed_count = 0;
        let total_count = components.len();

        for component in components {
            match component.implementation_status {
                ImplementationStatus::Complete => {
                    completed_count += 1;
                    completed_features.push(FeatureStatus {
                        name: component.name.clone(),
                        description: component.purpose.clone(),
                        evidence: vec![format!("Complete implementation at {}", component.file_path)],
                        confidence: 0.9,
                        related_files: vec![component.file_path.clone()],
                    });
                }
                ImplementationStatus::InProgress => {
                    in_progress_features.push(FeatureStatus {
                        name: component.name.clone(),
                        description: component.purpose.clone(),
                        evidence: vec![format!("In progress at {}", component.file_path)],
                        confidence: 0.8,
                        related_files: vec![component.file_path.clone()],
                    });
                }
                ImplementationStatus::Todo | ImplementationStatus::Incomplete => {
                    todo_features.push(FeatureStatus {
                        name: component.name.clone(),
                        description: component.purpose.clone(),
                        evidence: vec![format!("TODO/Incomplete at {}", component.file_path)],
                        confidence: 0.7,
                        related_files: vec![component.file_path.clone()],
                    });
                }
            }

            // Analyze for technical debt indicators
            if component.complexity_score > 80 {
                technical_debt.push(TechnicalDebt {
                    description: format!("High complexity in {}", component.name),
                    severity: "Medium".to_string(),
                    location: component.file_path.clone(),
                    recommendation: "Consider refactoring to reduce complexity".to_string(),
                });
            }
        }

        let overall_completion_percentage = if total_count > 0 {
            (completed_count as f32 / total_count as f32) * 100.0
        } else {
            0.0
        };

        StatusIntelligence {
            completed_features,
            in_progress_features,
            todo_features,
            technical_debt,
            overall_completion_percentage,
        }
    }

    fn analyze_integration_points(&self, project_path: &str) -> IntegrationPoints {
        let mut external_services = Vec::new();
        let mut internal_dependencies = Vec::new();
        let mut config_files = Vec::new();
        let mut environment_variables = Vec::new();

        // Analyze package.json for external dependencies
        if let Ok(package_json) = fs::read_to_string(format!("{}/package.json", project_path)) {
            config_files.push(ConfigFile {
                file_path: "package.json".to_string(),
                file_type: "NPM Configuration".to_string(),
                purpose: "Dependencies and scripts management".to_string(),
                key_configurations: vec!["dependencies".to_string(), "devDependencies".to_string()],
            });

            // Extract external service indicators
            if package_json.contains("\"axios\"") || package_json.contains("\"fetch\"") {
                external_services.push(ExternalService {
                    name: "HTTP API".to_string(),
                    service_type: "REST API".to_string(),
                    usage_context: "HTTP requests".to_string(),
                    integration_points: vec!["API calls in components".to_string()],
                });
            }
        }

        // Check for common config files
        let config_file_patterns = vec![
            ("next.config.js", "Next.js Configuration"),
            (".env", "Environment Variables"),
            (".env.local", "Local Environment Variables"),
            ("tsconfig.json", "TypeScript Configuration"),
            ("tailwind.config.js", "Tailwind CSS Configuration"),
        ];

        for (filename, description) in config_file_patterns {
            let file_path = format!("{}/{}", project_path, filename);
            if std::path::Path::new(&file_path).exists() {
                config_files.push(ConfigFile {
                    file_path: filename.to_string(),
                    file_type: description.to_string(),
                    purpose: format!("Configuration for {}", description),
                    key_configurations: vec!["Various settings".to_string()],
                });
            }
        }

        IntegrationPoints {
            external_services,
            internal_dependencies,
            configuration_files: config_files,
            environment_variables,
        }
    }

    // Helper methods for enhanced analysis
    fn extract_version_from_package_json(&self, package_json: &str, dependency: &str) -> Option<String> {
        let pattern = format!("\"{}\":\\s*\"([^\"]+)\"", dependency);
        if let Ok(re) = regex::Regex::new(&pattern) {
            if let Some(captures) = re.captures(package_json) {
                return captures.get(1).map(|m| m.as_str().to_string());
            }
        }
        None
    }

    fn extract_api_endpoints_from_components(&self, components: &[ComponentInfo]) -> Vec<EndpointAnalysis> {
        let mut endpoints = Vec::new();
        
        for component in components {
            for api_call in &component.api_calls {
                endpoints.push(EndpointAnalysis {
                    path: api_call.endpoint.clone(),
                    method: api_call.method.clone(),
                    controller: format!("Component: {}", component.name),
                    purpose: api_call.purpose.clone(),
                    request_schema: None,
                    response_schema: None,
                    authentication_required: false,
                    status: component.implementation_status.clone(),
                    related_frontend_components: vec![component.name.clone()],
                });
            }
        }
        
        endpoints
    }

    fn extract_entities_from_components(&self, _components: &[ComponentInfo]) -> Vec<EntityAnalysis> {
        // For TypeScript/React, entities would typically be in separate files or API responses
        Vec::new()
    }

    fn analyze_component_relationships(&self, components: &[ComponentInfo]) -> Vec<ComponentRelationship> {
        let mut relationships = Vec::new();
        
        for component in components {
            for dependency in &component.dependencies {
                if components.iter().any(|c| c.name == *dependency) {
                    relationships.push(ComponentRelationship {
                        source: component.name.clone(),
                        target: dependency.clone(),
                        relationship_type: "imports".to_string(),
                        description: format!("{} imports {}", component.name, dependency),
                    });
                }
            }
        }
        
        relationships
    }

    fn analyze_data_flows(&self, components: &[ComponentInfo]) -> Vec<DataFlowAnalysis> {
        let mut flows = Vec::new();
        
        for component in components {
            for api_call in &component.api_calls {
                flows.push(DataFlowAnalysis {
                    source: component.name.clone(),
                    target: api_call.endpoint.clone(),
                    data_type: "HTTP Request".to_string(),
                    flow_type: api_call.method.clone(),
                });
            }
        }
        
        flows
    }

    // NestJS-specific detection method
    fn detect_nestjs_patterns(&self, project_path: &str) -> Vec<String> {
        let mut evidence = Vec::new();
        
        // NestJS decorators to look for
        let nestjs_decorators = vec![
            "@Controller",
            "@Injectable",
            "@Module", 
            "@Get",
            "@Post",
            "@Put",
            "@Delete",
            "@Patch",
            "@Body",
            "@Param",
            "@Query",
            "@Guard",
            "@UseGuards",
            "@UseInterceptors",
            "@UsePipes",
        ];

        // NestJS imports to look for
        let nestjs_imports = vec![
            "from '@nestjs/common'",
            "from '@nestjs/core'",
            "import { NestFactory }",
            "import { Module }",
            "import { Controller }",
            "import { Injectable }",
        ];

        // Scan TypeScript files for NestJS patterns
        if let Ok(entries) = std::fs::read_dir(project_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "ts" && !path.to_string_lossy().contains("node_modules") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            // Check for NestJS decorators
                            for decorator in &nestjs_decorators {
                                if content.contains(decorator) {
                                    evidence.push(format!("{} found in {}", decorator, path.file_name().unwrap().to_string_lossy()));
                                    break; // Only add one evidence per file
                                }
                            }

                            // Check for NestJS imports
                            for import in &nestjs_imports {
                                if content.contains(import) {
                                    evidence.push(format!("NestJS import found in {}", path.file_name().unwrap().to_string_lossy()));
                                    break; // Only add one evidence per file
                                }
                            }

                            // Check for NestJS main.ts pattern
                            if path.file_name().unwrap() == "main.ts" && content.contains("NestFactory.create") {
                                evidence.push("NestJS application bootstrap in main.ts".to_string());
                            }

                            // Check for app.module.ts pattern
                            if path.file_name().unwrap() == "app.module.ts" && content.contains("@Module") {
                                evidence.push("NestJS root module in app.module.ts".to_string());
                            }
                        }
                    }
                }
            }
        }

        // Also check subdirectories (src/, etc.)
        for subdir in &["src", "apps", "libs"] {
            let subdir_path = format!("{}/{}", project_path, subdir);
            if let Ok(entries) = walkdir::WalkDir::new(&subdir_path).max_depth(3).into_iter().collect::<Result<Vec<_>, _>>() {
                for entry in entries {
                    let path = entry.path();
                    if let Some(extension) = path.extension() {
                        if extension == "ts" && !path.to_string_lossy().contains("node_modules") {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                // Check for NestJS decorators
                                for decorator in &nestjs_decorators {
                                    if content.contains(decorator) {
                                        evidence.push(format!("{} found in {}", decorator, path.file_name().unwrap().to_string_lossy()));
                                        break;
                                    }
                                }

                                // Check for NestJS-specific class patterns
                                if content.contains("@Controller") && content.contains("@Get") {
                                    evidence.push(format!("NestJS controller pattern in {}", path.file_name().unwrap().to_string_lossy()));
                                }

                                if content.contains("@Injectable") && (content.contains("@InjectRepository") || content.contains("Repository")) {
                                    evidence.push(format!("NestJS service pattern in {}", path.file_name().unwrap().to_string_lossy()));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Remove duplicates and limit evidence count
        evidence.sort();
        evidence.dedup();
        evidence.into_iter().take(10).collect() // Limit to 10 pieces of evidence
    }

    fn detect_deno_frameworks(&self, project_path: &str, detected_frameworks: &mut Vec<DetectedFramework>, confidence_scores: &mut std::collections::HashMap<String, f32>) {
        let mut deno_evidence = Vec::new();
        let mut framework_detected = false;

        // Check for deno.json or deno.jsonc
        if std::path::Path::new(&format!("{}/deno.json", project_path)).exists() ||
           std::path::Path::new(&format!("{}/deno.jsonc", project_path)).exists() {
            deno_evidence.push("deno.json configuration file".to_string());
            
            // Try to read deno.json for more specific framework detection
            let deno_config_path = if std::path::Path::new(&format!("{}/deno.json", project_path)).exists() {
                format!("{}/deno.json", project_path)
            } else {
                format!("{}/deno.jsonc", project_path)
            };
            
            if let Ok(deno_config) = std::fs::read_to_string(deno_config_path) {
                // Danet detection (NestJS-like framework for Deno)
                if deno_config.contains("danet") || deno_config.contains("https://deno.land/x/danet") {
                    detected_frameworks.push(DetectedFramework {
                        name: "Danet".to_string(),
                        version: None,
                        confidence: 0.95,
                        evidence: vec!["deno.json with Danet dependency".to_string()],
                        usage_extent: UsageExtent::Core,
                    });
                    confidence_scores.insert("Danet".to_string(), 0.95);
                    framework_detected = true;
                }
                
                // Fresh detection (Deno fullstack framework)
                if deno_config.contains("fresh") || deno_config.contains("https://deno.land/x/fresh") {
                    detected_frameworks.push(DetectedFramework {
                        name: "Fresh".to_string(),
                        version: None,
                        confidence: 0.95,
                        evidence: vec!["deno.json with Fresh dependency".to_string()],
                        usage_extent: UsageExtent::Core,
                    });
                    confidence_scores.insert("Fresh".to_string(), 0.95);
                    framework_detected = true;
                }
                
                // Oak detection (Deno HTTP server framework)
                if deno_config.contains("oak") || deno_config.contains("https://deno.land/x/oak") {
                    detected_frameworks.push(DetectedFramework {
                        name: "Oak".to_string(),
                        version: None,
                        confidence: 0.90,
                        evidence: vec!["deno.json with Oak dependency".to_string()],
                        usage_extent: UsageExtent::Core,
                    });
                    confidence_scores.insert("Oak".to_string(), 0.90);
                    framework_detected = true;
                }
            }
        }

        // Check for import_map.json (Deno import maps)
        if std::path::Path::new(&format!("{}/import_map.json", project_path)).exists() {
            deno_evidence.push("import_map.json file".to_string());
            
            if let Ok(import_map) = std::fs::read_to_string(format!("{}/import_map.json", project_path)) {
                // Check for framework imports in import map
                if import_map.contains("danet") {
                    if !framework_detected {
                        detected_frameworks.push(DetectedFramework {
                            name: "Danet".to_string(),
                            version: None,
                            confidence: 0.90,
                            evidence: vec!["Danet import in import_map.json".to_string()],
                            usage_extent: UsageExtent::Core,
                        });
                        confidence_scores.insert("Danet".to_string(), 0.90);
                        framework_detected = true;
                    }
                }
                
                if import_map.contains("oak") {
                    if !detected_frameworks.iter().any(|f| f.name == "Oak") {
                        detected_frameworks.push(DetectedFramework {
                            name: "Oak".to_string(),
                            version: None,
                            confidence: 0.85,
                            evidence: vec!["Oak import in import_map.json".to_string()],
                            usage_extent: UsageExtent::Core,
                        });
                        confidence_scores.insert("Oak".to_string(), 0.85);
                        framework_detected = true;
                    }
                }
            }
        }

        // Scan source files for Deno-specific import patterns
        let deno_patterns = self.detect_deno_source_patterns(project_path);
        for (framework_name, evidence) in deno_patterns {
            if !detected_frameworks.iter().any(|f| f.name == framework_name) {
                let confidence = match framework_name.as_str() {
                    "Danet" => 0.85,
                    "Fresh" => 0.85,
                    "Oak" => 0.80,
                    _ => 0.75,
                };
                
                detected_frameworks.push(DetectedFramework {
                    name: framework_name.clone(),
                    version: None,
                    confidence,
                    evidence: evidence.clone(),
                    usage_extent: UsageExtent::Core,
                });
                confidence_scores.insert(framework_name, confidence);
                framework_detected = true;
            }
        }

        // If we found Deno config files but no specific framework, mark as generic Deno
        if !deno_evidence.is_empty() && !framework_detected {
            detected_frameworks.push(DetectedFramework {
                name: "Deno Runtime".to_string(),
                version: None,
                confidence: 0.90,
                evidence: deno_evidence,
                usage_extent: UsageExtent::Core,
            });
            confidence_scores.insert("Deno Runtime".to_string(), 0.90);
        }
    }

    fn detect_deno_source_patterns(&self, project_path: &str) -> Vec<(String, Vec<String>)> {
        let mut framework_patterns = Vec::new();
        let mut danet_evidence = Vec::new();
        let mut fresh_evidence = Vec::new();
        let mut oak_evidence = Vec::new();

        // Danet patterns (NestJS-like decorators and imports for Deno)
        let danet_decorators = vec![
            "@Controller", "@Injectable", "@Module", "@Get", "@Post", "@Put", "@Delete"
        ];
        let danet_imports = vec![
            "from \"https://deno.land/x/danet",
            "import { Controller, Get, Post",
            "import { Injectable, Module",
        ];

        // Fresh patterns
        let fresh_patterns = vec![
            "export { Handler }",
            "import { HandlerContext }",
            "islands/",
            "routes/",
        ];

        // Oak patterns
        let oak_patterns = vec![
            "import { Application, Router } from \"https://deno.land/x/oak",
            "import { Application } from \"https://deno.land/x/oak",
            "import { Router } from \"https://deno.land/x/oak",
            "new Application()",
            "new Router()",
        ];

        // Scan TypeScript files for Deno framework patterns
        for subdir in &["", "src", "routes", "controllers", "services"] {
            let scan_path = if subdir.is_empty() {
                project_path.to_string()
            } else {
                format!("{}/{}", project_path, subdir)
            };

            if let Ok(entries) = walkdir::WalkDir::new(&scan_path).max_depth(3).into_iter().collect::<Result<Vec<_>, _>>() {
                for entry in entries {
                    let path = entry.path();
                    if let Some(extension) = path.extension() {
                        if (extension == "ts" || extension == "tsx") && !path.to_string_lossy().contains("node_modules") {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                // Check for Danet patterns
                                for decorator in &danet_decorators {
                                    if content.contains(decorator) && content.contains("deno.land/x/danet") {
                                        danet_evidence.push(format!("{} with Danet import in {}", decorator, path.file_name().unwrap().to_string_lossy()));
                                        break;
                                    }
                                }
                                
                                for import in &danet_imports {
                                    if content.contains(import) {
                                        danet_evidence.push(format!("Danet import in {}", path.file_name().unwrap().to_string_lossy()));
                                        break;
                                    }
                                }

                                // Check for Fresh patterns
                                for pattern in &fresh_patterns {
                                    if content.contains(pattern) {
                                        fresh_evidence.push(format!("Fresh pattern '{}' in {}", pattern, path.file_name().unwrap().to_string_lossy()));
                                        break;
                                    }
                                }

                                // Check for Oak patterns
                                for pattern in &oak_patterns {
                                    if content.contains(pattern) {
                                        oak_evidence.push(format!("Oak pattern in {}", path.file_name().unwrap().to_string_lossy()));
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Collect framework patterns with evidence
        if !danet_evidence.is_empty() {
            framework_patterns.push(("Danet".to_string(), danet_evidence));
        }
        if !fresh_evidence.is_empty() {
            framework_patterns.push(("Fresh".to_string(), fresh_evidence));
        }
        if !oak_evidence.is_empty() {
            framework_patterns.push(("Oak".to_string(), oak_evidence));
        }

        framework_patterns
    }

    fn analyze_business_logic_patterns(&self, components: &[ComponentInfo]) -> BusinessLogicAnalysis {
        let mut evidence = Vec::new();
        let mut domain_scores = std::collections::HashMap::new();
        let mut framework_type = "Generic".to_string();
        
        // Separate components by type based on file patterns
        let controllers: Vec<_> = components.iter()
            .filter(|c| c.file_path.contains(".controller.") || 
                       c.name.to_lowercase().contains("controller"))
            .collect();
        
        let services: Vec<_> = components.iter()
            .filter(|c| c.file_path.contains(".service.") || 
                       c.name.to_lowercase().contains("service"))
            .collect();
        
        let middleware: Vec<_> = components.iter()
            .filter(|c| c.file_path.contains(".middleware.") || 
                       c.name.to_lowercase().contains("middleware"))
            .collect();

        // Determine framework type
        for component in &controllers {
            if component.file_path.contains("deno") || 
               component.dependencies.iter().any(|d| d.contains("danet")) {
                framework_type = "Danet".to_string();
                break;
            } else if component.dependencies.iter().any(|d| d.contains("@nestjs")) {
                framework_type = "NestJS".to_string();
                break;
            }
        }

        // Analyze controller endpoints and business logic
        let mut auth_endpoints = 0;
        let mut user_endpoints = 0;
        let mut property_endpoints = 0;
        let mut product_endpoints = 0;
        
        for controller in &controllers {
            let controller_name = controller.name.to_lowercase();
            let file_path = controller.file_path.to_lowercase();
            
            // Analyze controller names and paths for business context
            if controller_name.contains("auth") || file_path.contains("/auth/") {
                auth_endpoints += 3;
                evidence.push(format!("Authentication controller: {}", controller.name));
            }
            
            if controller_name.contains("user") || file_path.contains("/user/") {
                user_endpoints += 3;
                evidence.push(format!("User management controller: {}", controller.name));
            }
            
            if controller_name.contains("property") || file_path.contains("/property/") {
                property_endpoints += 3;
                evidence.push(format!("Property management controller: {}", controller.name));
            }
            
            if controller_name.contains("product") || file_path.contains("/product/") {
                product_endpoints += 3;
                evidence.push(format!("Product management controller: {}", controller.name));
            }
            
            // Analyze API calls to understand endpoints
            for api_call in &controller.api_calls {
                let endpoint = api_call.endpoint.to_lowercase();
                let method = api_call.method.to_lowercase();
                
                // Authentication patterns
                if endpoint.contains("auth") || endpoint.contains("login") || endpoint.contains("register") {
                    auth_endpoints += 2;
                    evidence.push(format!("Auth endpoint: {} {}", method.to_uppercase(), api_call.endpoint));
                }
                
                // User management patterns
                if endpoint.contains("user") || endpoint.contains("profile") {
                    user_endpoints += 2;
                    evidence.push(format!("User endpoint: {} {}", method.to_uppercase(), api_call.endpoint));
                }
                
                // Property management patterns
                if endpoint.contains("property") || endpoint.contains("listing") {
                    property_endpoints += 2;
                    evidence.push(format!("Property endpoint: {} {}", method.to_uppercase(), api_call.endpoint));
                }
                
                // Product management patterns
                if endpoint.contains("product") || endpoint.contains("inventory") {
                    product_endpoints += 2;
                    evidence.push(format!("Product endpoint: {} {}", method.to_uppercase(), api_call.endpoint));
                }
            }
        }

        // Analyze services for business logic
        let mut auth_services = 0;
        let mut user_services = 0;
        
        for service in &services {
            let service_name = service.name.to_lowercase();
            let file_path = service.file_path.to_lowercase();
            
            if service_name.contains("auth") || service_name.contains("jwt") || 
               service_name.contains("session") || file_path.contains("/auth/") {
                auth_services += 2;
                evidence.push(format!("Authentication service: {}", service.name));
            }
            
            if service_name.contains("user") || file_path.contains("/user/") {
                user_services += 2;
                evidence.push(format!("User service: {}", service.name));
            }
        }

        // Analyze middleware for cross-cutting concerns
        let mut auth_middleware = 0;
        
        for mw in &middleware {
            let mw_name = mw.name.to_lowercase();
            
            if mw_name.contains("auth") || mw_name.contains("jwt") || mw_name.contains("session") {
                auth_middleware += 2;
                evidence.push(format!("Authentication middleware: {}", mw.name));
            }
        }

        // Determine business domain based on analysis
        if (auth_endpoints + auth_services + auth_middleware) > 3 && (user_endpoints + user_services) > 2 {
            domain_scores.insert("UserManagement".to_string(), auth_endpoints + auth_services + auth_middleware + user_endpoints + user_services);
        } else if property_endpoints > 3 {
            domain_scores.insert("RealEstate".to_string(), property_endpoints);
        } else if product_endpoints > 3 {
            domain_scores.insert("Ecommerce".to_string(), product_endpoints);
        } else if (auth_endpoints + auth_services + auth_middleware) > 2 {
            domain_scores.insert("Authentication".to_string(), auth_endpoints + auth_services + auth_middleware);
        } else if controllers.len() > 0 {
            domain_scores.insert("API".to_string(), controllers.len());
        }

        BusinessLogicAnalysis {
            framework_type,
            domain_scores,
            evidence,
        }
    }
}

#[derive(Debug, Clone)]
struct BusinessLogicAnalysis {
    framework_type: String,
    domain_scores: std::collections::HashMap<String, usize>,
    evidence: Vec<String>,
}