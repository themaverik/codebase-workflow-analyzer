use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use crate::intelligence::ollama_manager::OllamaManager;
use crate::core::business_purpose_extractor::{BusinessContext, BusinessPurpose, UserPersona, Feature};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMBusinessAnalysis {
    pub inferred_domain: String,
    pub confidence: f32,
    pub business_description: String,
    pub problem_statement: String,
    pub target_users: Vec<String>,
    pub key_features: Vec<String>,
    pub value_proposition: String,
    pub reasoning: String,
}

pub struct LLMBusinessAnalyzer {
    ollama_manager: OllamaManager,
    model_name: String,
}

impl LLMBusinessAnalyzer {
    pub async fn new(project_path: &str) -> Result<Self> {
        let mut ollama_manager = OllamaManager::new()?;
        let model_name = ollama_manager.initialize_with_user_interaction(project_path).await?;

        Ok(Self {
            ollama_manager,
            model_name,
        })
    }

    pub async fn new_non_interactive(project_path: &str) -> Result<Self> {
        // Initializing LLM Business Analyzer (non-interactive)
        // Project path validated
        
        let ollama_manager = OllamaManager::new()?;
        // OllamaManager created successfully
        
        // Check if Ollama is running
        // Checking Ollama connection
        match ollama_manager.check_ollama_connection().await {
            Ok(true) => { /* Ollama is running */ }
            Ok(false) => {
                // Ollama connection returned false
                anyhow::bail!("Ollama is not running. Please start Ollama with: ollama serve");
            }
            Err(e) => {
                // Ollama connection error
                anyhow::bail!("Failed to connect to Ollama: {}. Please start Ollama with: ollama serve", e);
            }
        }

        // Get available models
        // Listing available models
        let available_models = match ollama_manager.list_available_models().await {
            Ok(models) => {
                // Found models
                // Model list loaded
                models
            }
            Err(e) => {
                // Failed to list models
                anyhow::bail!("Failed to list Ollama models: {}", e);
            }
        };
        
        let config = ollama_manager.get_config();
        // Default model from config
        
        // Try to find the default model or first available model
        let model_name = if ollama_manager.model_exists(&available_models, &config.default_model) {
            // Default model found
            config.default_model.clone()
        } else if !available_models.is_empty() {
            // Default model not found, using first available
            available_models[0].name.clone()
        } else {
            // No models available
            anyhow::bail!("No models available. Please install a model with: ollama pull {}", config.default_model);
        };

        println!("Using model: {}", model_name);
        
        // Validate project access (MCP permissions implied by --enable-llm flag)
        // Validating project path
        if !std::path::Path::new(project_path).exists() {
            // Project path does not exist
            anyhow::bail!("Project path does not exist: {}", project_path);
        }
        // Project path exists

        // LLM Business Analyzer initialized successfully
        Ok(Self {
            ollama_manager,
            model_name,
        })
    }

    pub async fn analyze_business_context(
        &self,
        project_path: &str,
        api_endpoints: &[String], 
        dependencies: &[String],
        file_names: &[String],
        readme_content: Option<&str>,
    ) -> Result<BusinessContext> {

        // Generate comprehensive business analysis using LLM
        let llm_analysis = self.perform_llm_analysis(
            project_path, api_endpoints, dependencies, file_names, readme_content
        ).await?;

        // Convert LLM analysis to BusinessContext structure
        let usage_scenarios = self.generate_usage_scenarios_from_llm(&llm_analysis).await?;
        let user_personas = self.generate_user_personas_from_llm(&llm_analysis).await?;
        let features = self.extract_features_from_llm(&llm_analysis, api_endpoints, dependencies).await?;
        let success_indicators = self.generate_success_indicators_from_llm(&llm_analysis).await?;

        let business_purpose = BusinessPurpose {
            description: llm_analysis.business_description,
            problem_statement: llm_analysis.problem_statement,
            target_users: llm_analysis.target_users,
            key_features: llm_analysis.key_features,
            value_proposition: llm_analysis.value_proposition,
            usage_scenarios,
            confidence_score: llm_analysis.confidence,
        };

        Ok(BusinessContext {
            purpose: business_purpose,
            user_personas,
            feature_breakdown: features,
            success_indicators,
        })
    }

    async fn perform_llm_analysis(
        &self,
        project_path: &str,
        api_endpoints: &[String],
        dependencies: &[String], 
        file_names: &[String],
        readme_content: Option<&str>,
    ) -> Result<LLMBusinessAnalysis> {

        let prompt = self.build_business_analysis_prompt(
            project_path, api_endpoints, dependencies, file_names, readme_content
        );

        let response = self.ollama_manager
            .generate_response(&self.model_name, &prompt)
            .await
            .with_context(|| "Failed to generate LLM business analysis")?;

        self.parse_llm_business_response(&response)
    }

    fn build_business_analysis_prompt(
        &self,
        project_path: &str,
        api_endpoints: &[String],
        dependencies: &[String],
        file_names: &[String], 
        readme_content: Option<&str>,
    ) -> String {
        
        let mut prompt = format!(
            "As an expert business analyst, analyze this software project and infer its business purpose and context.\n\n"
        );
        
        prompt.push_str(&format!("Project: {}\n\n", project_path));
        
        // Include sample of actual file contents for better analysis
        if let Some(sample_content) = self.read_sample_files(project_path, file_names) {
            prompt.push_str("Sample Code Content:\n");
            prompt.push_str(&sample_content);
            prompt.push_str("\n\n");
        }
        
        if !api_endpoints.is_empty() {
            prompt.push_str("API Endpoints:\n");
            for endpoint in api_endpoints.iter().take(15) {
                prompt.push_str(&format!("- {}\n", endpoint));
            }
            prompt.push('\n');
        }
        
        if !dependencies.is_empty() {
            prompt.push_str("Key Dependencies:\n");
            for dep in dependencies.iter().take(20) {
                prompt.push_str(&format!("- {}\n", dep));
            }
            prompt.push('\n');
        }
        
        if !file_names.is_empty() {
            prompt.push_str("Key Files/Components:\n");
            for file in file_names.iter().take(25) {
                prompt.push_str(&format!("- {}\n", file));
            }
            prompt.push('\n');
        }
        
        if let Some(readme) = readme_content {
            let truncated_readme = if readme.len() > 2000 {
                format!("{}...[truncated]", &readme[..2000])
            } else {
                readme.to_string()
            };
            prompt.push_str(&format!("README Content:\n{}\n\n", truncated_readme));
        }
        
        prompt.push_str(
            "Based on this code evidence, analyze and provide:\n\n\
            1. **Primary Business Domain** (e.g., 'E-commerce', 'Developer Tools', 'Content Management')\n\
            2. **Business Description** (2-3 sentences describing what this software does)\n\
            3. **Problem Statement** (1-2 sentences about the problem this solves)\n\
            4. **Target Users** (3-5 specific user types who would use this)\n\
            5. **Key Features** (5-8 main features evident from the code)\n\
            6. **Value Proposition** (1-2 sentences about the unique value)\n\
            7. **Confidence Score** (0.0-1.0 based on evidence quality)\n\
            8. **Reasoning** (brief explanation of your analysis)\n\n\
            Format your response as JSON with these exact fields:\n\
            {\n\
              \"inferred_domain\": \"Domain Name\",\n\
              \"confidence\": 0.85,\n\
              \"business_description\": \"Description...\",\n\
              \"problem_statement\": \"Problem...\",\n\
              \"target_users\": [\"User1\", \"User2\"],\n\
              \"key_features\": [\"Feature1\", \"Feature2\"],\n\
              \"value_proposition\": \"Value...\",\n\
              \"reasoning\": \"Analysis reasoning...\"\n\
            }\n\n\
            Be specific and base your analysis solely on the code evidence provided."
        );
        
        prompt
    }

    fn parse_llm_business_response(&self, response: &str) -> Result<LLMBusinessAnalysis> {
        // Try to extract JSON from the response
        let json_start = response.find('{');
        let json_end = response.rfind('}');

        if let (Some(start), Some(end)) = (json_start, json_end) {
            let json_str = &response[start..=end];
            
            match serde_json::from_str::<LLMBusinessAnalysis>(json_str) {
                Ok(analysis) => return Ok(analysis),
                Err(e) => {
                    println!("⚠️ Failed to parse JSON response: {}", e);
                    println!("Response: {}", json_str);
                }
            }
        }

        // Fallback: Parse response manually if JSON parsing fails
        self.parse_response_manually(response)
    }

    fn parse_response_manually(&self, response: &str) -> Result<LLMBusinessAnalysis> {
        // Simple manual parsing as fallback
        Ok(LLMBusinessAnalysis {
            inferred_domain: "General Software".to_string(),
            confidence: 0.60,
            business_description: "Software application providing functionality based on detected patterns".to_string(),
            problem_statement: "Users need efficient solutions for their domain-specific tasks".to_string(),
            target_users: vec!["Software users".to_string(), "System administrators".to_string()],
            key_features: vec!["Core application functionality".to_string()],
            value_proposition: "Provides reliable software solution for identified use cases".to_string(),
            reasoning: format!("Fallback analysis due to parsing issues. Raw LLM response: {}", 
                             response.chars().take(200).collect::<String>()),
        })
    }

    async fn generate_usage_scenarios_from_llm(&self, analysis: &LLMBusinessAnalysis) -> Result<Vec<String>> {
        let prompt = format!(
            "Based on this business analysis:\n\
            Domain: {}\n\
            Description: {}\n\
            Target Users: {:?}\n\n\
            Generate 3-4 realistic usage scenarios showing how users would interact with this system.\n\
            Format as a simple list, one scenario per line starting with '-'.",
            analysis.inferred_domain,
            analysis.business_description,
            analysis.target_users
        );

        let response = self.ollama_manager.generate_response(&self.model_name, &prompt).await?;
        
        Ok(response
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if trimmed.starts_with('-') {
                    Some(trimmed[1..].trim().to_string())
                } else {
                    None
                }
            })
            .collect())
    }

    async fn generate_user_personas_from_llm(&self, analysis: &LLMBusinessAnalysis) -> Result<Vec<UserPersona>> {
        let prompt = format!(
            "Based on these target users for a {} system:\n{:?}\n\n\
            Create detailed user personas with:\n\
            - Name (role-based)\n\
            - Role description\n\
            - 3 main goals\n\
            - 3 pain points\n\n\
            Format as JSON array with this structure:\n\
            [\n\
              {{\n\
                \"name\": \"Role Name\",\n\
                \"role\": \"Role description\",\n\
                \"goals\": [\"Goal1\", \"Goal2\", \"Goal3\"],\n\
                \"pain_points\": [\"Pain1\", \"Pain2\", \"Pain3\"]\n\
              }}\n\
            ]",
            analysis.inferred_domain,
            analysis.target_users
        );

        let response = self.ollama_manager.generate_response(&self.model_name, &prompt).await?;
        
        // Try to parse JSON response
        if let Some(start) = response.find('[') {
            if let Some(end) = response.rfind(']') {
                let json_str = &response[start..=end];
                if let Ok(personas) = serde_json::from_str::<Vec<UserPersona>>(json_str) {
                    return Ok(personas);
                }
            }
        }

        // Fallback: Generate basic personas
        Ok(analysis.target_users.iter().take(2).map(|user| {
            UserPersona {
                name: user.clone(),
                role: format!("Primary {}", user.to_lowercase()),
                goals: vec!["Accomplish tasks efficiently".to_string()],
                pain_points: vec!["Current solutions are inadequate".to_string()],
            }
        }).collect())
    }

    async fn extract_features_from_llm(
        &self,
        analysis: &LLMBusinessAnalysis,
        api_endpoints: &[String],
        dependencies: &[String],
    ) -> Result<Vec<Feature>> {

        let features: Vec<Feature> = analysis.key_features.iter().map(|feature_name| {
            Feature {
                name: feature_name.clone(),
                description: format!("Provides {} capability", feature_name.to_lowercase()),
                user_benefit: "Enables users to accomplish their objectives effectively".to_string(),
                implementation_evidence: vec![
                    format!("Evidence from {} analysis", analysis.inferred_domain),
                    format!("Based on {} endpoints and {} dependencies", 
                           api_endpoints.len(), dependencies.len())
                ],
            }
        }).collect();

        Ok(features)
    }

    async fn generate_success_indicators_from_llm(&self, analysis: &LLMBusinessAnalysis) -> Result<Vec<String>> {
        let prompt = format!(
            "For a {} system with this value proposition:\n{}\n\n\
            List 4-5 key success indicators that would measure if this system is achieving its goals.\n\
            Format as simple list, one per line starting with '-'.",
            analysis.inferred_domain,
            analysis.value_proposition
        );

        let response = self.ollama_manager.generate_response(&self.model_name, &prompt).await?;
        
        Ok(response
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if trimmed.starts_with('-') {
                    Some(trimmed[1..].trim().to_string())
                } else {
                    None
                }
            })
            .take(5)
            .collect())
    }

    fn read_sample_files(&self, project_path: &str, _file_names: &[String]) -> Option<String> {
        let mut content = String::new();
        let mut files_read = 0;
        const MAX_FILES: usize = 5;
        const MAX_FILE_SIZE: usize = 2000; // characters per file
        
        // Find actual source files instead of using component names
        let source_extensions = vec!["rs", "ts", "js", "py", "java", "md"];
        
        if let Ok(entries) = std::fs::read_dir(project_path) {
            for entry in entries {
                if files_read >= MAX_FILES {
                    break;
                }
                
                if let Ok(entry) = entry {
                    let path = entry.path();
                    
                    // Skip directories and look for source files
                    if path.is_file() {
                        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
                            if source_extensions.contains(&extension) {
                                match std::fs::read_to_string(&path) {
                                    Ok(file_content) => {
                                        let truncated_content = if file_content.len() > MAX_FILE_SIZE {
                                            format!("{}...[truncated]", &file_content[..MAX_FILE_SIZE])
                                        } else {
                                            file_content
                                        };
                                        
                                        let file_name = path.file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or("unknown");
                                        
                                        content.push_str(&format!("\n=== {} ===\n", file_name));
                                        content.push_str(&truncated_content);
                                        content.push_str("\n");
                                        files_read += 1;
                                    }
                                    Err(_) => continue, // Skip files we can't read
                                }
                            }
                        }
                    }
                }
            }
        }
        
        if content.is_empty() {
            None
        } else {
            Some(content)
        }
    }
}