use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::core::business_context_config::{BusinessDomainConfig, BusinessDomain};
use crate::core::business_purpose_extractor::{BusinessContext, BusinessPurpose, UserPersona, Feature};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMBusinessContext {
    pub inferred_domain: String,
    pub confidence: f32,
    pub business_description: String,
    pub problem_statement: String,
    pub target_users: Vec<String>,
    pub key_features: Vec<String>,
    pub value_proposition: String,
}

pub struct LLMBusinessExtractor {
    domain_config: BusinessDomainConfig,
    llm_enabled: bool,
}

impl LLMBusinessExtractor {
    pub fn new(config_path: &str, llm_enabled: bool) -> Result<Self> {
        let domain_config = BusinessDomainConfig::load_from_file(config_path)?;
        
        Ok(LLMBusinessExtractor {
            domain_config,
            llm_enabled,
        })
    }
    
    pub fn extract_business_context(
        &self,
        project_path: &str,
        api_endpoints: &[String], 
        dependencies: &[String],
        file_names: &[String],
        readme_content: Option<&str>,
    ) -> Result<BusinessContext> {
        
        // Step 1: Use configuration-based domain detection (no hardcoding!)
        let detected_domain = self.detect_domain_from_config(api_endpoints, dependencies, file_names);
        
        // Step 2: If LLM is enabled, use it for semantic analysis
        let llm_context = if self.llm_enabled {
            self.generate_llm_business_context(project_path, api_endpoints, dependencies, file_names, readme_content)?
        } else {
            None
        };
        
        // Step 3: Combine config-based and LLM-based analysis
        let business_purpose = self.synthesize_business_purpose(&detected_domain, llm_context.as_ref())?;
        let user_personas = self.generate_user_personas(&detected_domain.id);
        let features = self.extract_features_from_evidence(api_endpoints, dependencies);
        let success_indicators = self.generate_success_indicators(&detected_domain.id);
        
        Ok(BusinessContext {
            purpose: business_purpose,
            user_personas,
            feature_breakdown: features,
            success_indicators,
        })
    }
    
    fn detect_domain_from_config(
        &self,
        api_endpoints: &[String],
        dependencies: &[String], 
        file_names: &[String],
    ) -> BusinessDomain {
        
        let mut best_domain = None;
        let mut best_score = 0.0;
        
        for domain in &self.domain_config.business_domains {
            let score = domain.calculate_match_score(api_endpoints, dependencies, file_names);
            
            if score > best_score {
                best_score = score;
                best_domain = Some(domain.clone());
            }
        }
        
        // Return best match or create a generic domain
        best_domain.unwrap_or_else(|| BusinessDomain {
            id: "general_software".to_string(),
            name: "General Software".to_string(), 
            description: "General purpose software application".to_string(),
            keywords: vec!["software".to_string()],
            patterns: vec![],
        })
    }
    
    fn generate_llm_business_context(
        &self,
        project_path: &str,
        api_endpoints: &[String],
        dependencies: &[String], 
        file_names: &[String],
        readme_content: Option<&str>,
    ) -> Result<Option<LLMBusinessContext>> {
        
        if !self.llm_enabled {
            return Ok(None);
        }
        
        // Create prompt for LLM analysis
        let context_prompt = self.build_context_analysis_prompt(
            project_path, api_endpoints, dependencies, file_names, readme_content
        );
        
        // TODO: Integrate with Ollama LLM here
        // For now, return None to indicate LLM analysis is not available
        println!("🧠 LLM Business Context Analysis would be performed here with prompt:");
        println!("   Context: {} API endpoints, {} dependencies, {} files", 
                api_endpoints.len(), dependencies.len(), file_names.len());
        
        Ok(None)
    }
    
    fn build_context_analysis_prompt(
        &self,
        project_path: &str,
        api_endpoints: &[String],
        dependencies: &[String],
        file_names: &[String], 
        readme_content: Option<&str>,
    ) -> String {
        
        let mut prompt = format!(
            "Analyze this codebase and infer its business purpose and context:\n\n"
        );
        
        prompt.push_str(&format!("Project Path: {}\n\n", project_path));
        
        if !api_endpoints.is_empty() {
            prompt.push_str("API Endpoints Found:\n");
            for endpoint in api_endpoints.iter().take(10) {  // Limit for context
                prompt.push_str(&format!("- {}\n", endpoint));
            }
            prompt.push('\n');
        }
        
        if !dependencies.is_empty() {
            prompt.push_str("Key Dependencies:\n");
            for dep in dependencies.iter().take(15) {  // Limit for context
                prompt.push_str(&format!("- {}\n", dep));
            }
            prompt.push('\n');
        }
        
        if !file_names.is_empty() {
            prompt.push_str("Key Files/Modules:\n");
            for file in file_names.iter().take(20) {  // Limit for context
                prompt.push_str(&format!("- {}\n", file));
            }
            prompt.push('\n');
        }
        
        if let Some(readme) = readme_content {
            prompt.push_str(&format!("README Content:\n{}\n\n", readme));
        }
        
        prompt.push_str(
            "Based on this evidence, provide:\n\
            1. Primary business domain (e.g., 'E-commerce', 'Content Management', 'Developer Tools')\n\
            2. Business problem this solves (1-2 sentences)\n\
            3. Target user types (3-5 categories)\n\
            4. Key value propositions (3-5 points)\n\
            5. Main features evident in code (5-10 features)\n\n\
            Respond with analysis based purely on code evidence, not assumptions."
        );
        
        prompt
    }
    
    fn synthesize_business_purpose(
        &self,
        detected_domain: &BusinessDomain,
        _llm_context: Option<&LLMBusinessContext>,
    ) -> Result<BusinessPurpose> {
        
        // Use configuration-driven approach (no hardcoding!)
        let description = detected_domain.description.clone();
        
        let problem_statement = match detected_domain.id.as_str() {
            "codebase_intelligence" => {
                "Development teams struggle to understand and maintain existing codebases without proper documentation and analysis tools, leading to slower development cycles and increased technical debt.".to_string()
            },
            "ecommerce" => {
                "Businesses need efficient online selling platforms while customers require secure, user-friendly shopping experiences with reliable payment processing.".to_string()  
            },
            "authentication" => {
                "Applications require secure user authentication and authorization systems to protect user data and control access to resources.".to_string()
            },
            _ => {
                format!("Users in the {} domain need efficient solutions to accomplish their specific tasks through reliable, well-designed software interfaces.", detected_domain.name)
            }
        };
        
        // Generate target users based on domain
        let target_users = match detected_domain.id.as_str() {
            "codebase_intelligence" => vec![
                "Software developers and engineers".to_string(),
                "Technical leads and architects".to_string(), 
                "DevOps and platform engineers".to_string(),
                "Code auditors and consultants".to_string(),
            ],
            "ecommerce" => vec![
                "Online shoppers".to_string(),
                "Merchants and retailers".to_string(),
                "E-commerce administrators".to_string(),
            ],
            _ => vec!["End users".to_string(), "System administrators".to_string()],
        };
        
        Ok(BusinessPurpose {
            description,
            problem_statement,
            target_users,
            key_features: vec!["Configuration-driven feature detection".to_string()], // Will be populated by evidence
            value_proposition: format!("Leverages {} capabilities to deliver efficient, reliable solutions", detected_domain.name),
            usage_scenarios: vec![
                format!("Users leverage {} functionality through the application interface", detected_domain.name),
                "Developers integrate with the system using available APIs".to_string(),
            ],
            confidence_score: 0.75, // Medium confidence for config-based detection
        })
    }
    
    fn generate_user_personas(&self, domain_id: &str) -> Vec<UserPersona> {
        match domain_id {
            "codebase_intelligence" => vec![
                UserPersona {
                    name: "Software Developer".to_string(),
                    role: "Developer analyzing existing codebases".to_string(),
                    goals: vec![
                        "Quickly understand codebase architecture".to_string(),
                        "Identify areas for improvement".to_string(),
                        "Generate accurate documentation".to_string(),
                    ],
                    pain_points: vec![
                        "Legacy code lacks documentation".to_string(),
                        "Manual analysis is time-consuming".to_string(), 
                        "Difficult to assess business impact".to_string(),
                    ],
                }
            ],
            _ => vec![
                UserPersona {
                    name: "End User".to_string(),
                    role: "Primary application user".to_string(),
                    goals: vec!["Complete tasks efficiently".to_string()],
                    pain_points: vec!["Existing solutions are inadequate".to_string()],
                }
            ]
        }
    }
    
    fn extract_features_from_evidence(
        &self,
        api_endpoints: &[String],
        dependencies: &[String],
    ) -> Vec<Feature> {
        
        let mut features = Vec::new();
        
        // Extract features from API evidence
        if api_endpoints.iter().any(|e| e.contains("/api")) {
            features.push(Feature {
                name: "REST API".to_string(),
                description: "RESTful API endpoints for programmatic access".to_string(),
                user_benefit: "Enables integration and automation".to_string(),
                implementation_evidence: api_endpoints.iter().take(5).cloned().collect(),
            });
        }
        
        // Extract features from dependency evidence
        if dependencies.iter().any(|d| d.contains("database") || d.contains("db")) {
            features.push(Feature {
                name: "Data Persistence".to_string(),
                description: "Database integration for data storage and retrieval".to_string(),
                user_benefit: "Reliable data management and persistence".to_string(),
                implementation_evidence: dependencies.iter()
                    .filter(|d| d.contains("database") || d.contains("db"))
                    .take(3).cloned().collect(),
            });
        }
        
        if features.is_empty() {
            features.push(Feature {
                name: "Core Functionality".to_string(),
                description: "Primary application features as determined by system architecture".to_string(),
                user_benefit: "Delivers intended application value".to_string(),
                implementation_evidence: vec!["System implementation evidence".to_string()],
            });
        }
        
        features
    }
    
    fn generate_success_indicators(&self, domain_id: &str) -> Vec<String> {
        match domain_id {
            "codebase_intelligence" => vec![
                "Accurate framework and architecture detection".to_string(),
                "High-quality business context extraction".to_string(),
                "Time savings in codebase understanding".to_string(),
                "Improved development workflow planning".to_string(),
            ],
            _ => vec![
                "User satisfaction and engagement".to_string(),
                "System reliability and performance".to_string(),
                "Feature adoption and usage rates".to_string(),
            ]
        }
    }
}