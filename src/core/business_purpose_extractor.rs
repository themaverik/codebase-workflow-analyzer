use std::collections::HashMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessPurpose {
    pub description: String,
    pub problem_statement: String,
    pub target_users: Vec<String>,
    pub key_features: Vec<String>,
    pub value_proposition: String,
    pub usage_scenarios: Vec<String>,
    pub confidence_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessContext {
    pub purpose: BusinessPurpose,
    pub user_personas: Vec<UserPersona>,
    pub feature_breakdown: Vec<Feature>,
    pub success_indicators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPersona {
    pub name: String,
    pub role: String,
    pub goals: Vec<String>,
    pub pain_points: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub name: String,
    pub description: String,
    pub user_benefit: String,
    pub implementation_evidence: Vec<String>,
}

pub struct BusinessPurposeExtractor {
    domain_patterns: HashMap<String, DomainIndicators>,
}

#[derive(Debug, Clone)]
struct DomainIndicators {
    api_patterns: Vec<String>,
    library_patterns: Vec<String>,
    file_patterns: Vec<String>,
    purpose_description: String,
    target_users: Vec<String>,
    key_benefits: Vec<String>,
}

impl BusinessPurposeExtractor {
    pub fn new() -> Self {
        let mut domain_patterns = HashMap::new();
        
        // Audio Processing Domain
        domain_patterns.insert("audio_processing".to_string(), DomainIndicators {
            api_patterns: vec![
                "/process".to_string(),
                "/separate".to_string(),
                "/extract".to_string(),
                "audio".to_string(),
                "midi".to_string(),
            ],
            library_patterns: vec![
                "demucs".to_string(),
                "basic-pitch".to_string(),
                "librosa".to_string(),
                "tensorflow".to_string(),
                "torch".to_string(),
                "audio".to_string(),
                "music".to_string(),
                "separation".to_string(),
            ],
            file_patterns: vec![
                "separation".to_string(),
                "extraction".to_string(),
                "midi".to_string(),
                "audio".to_string(),
                "music".to_string(),
            ],
            purpose_description: "Specialized microservice for music production and collaboration that intelligently separates audio recordings into individual instrument stems and extracts MIDI notation from the separated tracks.".to_string(),
            target_users: vec![
                "Music producers and sound engineers".to_string(),
                "Musicians and composers".to_string(),
                "Music collaboration platforms".to_string(),
                "Audio content creators".to_string(),
            ],
            key_benefits: vec![
                "Isolate individual instruments from mixed recordings for remixing".to_string(),
                "Extract MIDI notation for musical analysis and editing".to_string(),
                "Enable collaborative music production workflows".to_string(),
                "Provide professional-grade audio processing through simple API".to_string(),
            ],
        });

        // E-commerce Domain
        domain_patterns.insert("ecommerce".to_string(), DomainIndicators {
            api_patterns: vec![
                "/products".to_string(),
                "/cart".to_string(),
                "/orders".to_string(),
                "/payment".to_string(),
                "/checkout".to_string(),
            ],
            library_patterns: vec![
                "stripe".to_string(),
                "payment".to_string(),
                "commerce".to_string(),
                "shop".to_string(),
            ],
            file_patterns: vec![
                "product".to_string(),
                "order".to_string(),
                "payment".to_string(),
                "cart".to_string(),
            ],
            purpose_description: "E-commerce platform that enables online buying and selling of products or services.".to_string(),
            target_users: vec![
                "Online shoppers".to_string(),
                "Merchants and retailers".to_string(),
                "E-commerce administrators".to_string(),
            ],
            key_benefits: vec![
                "Browse and purchase products online".to_string(),
                "Secure payment processing".to_string(),
                "Order management and tracking".to_string(),
                "Inventory management".to_string(),
            ],
        });

        // Developer Tools Domain
        domain_patterns.insert("developer_tools".to_string(), DomainIndicators {
            api_patterns: vec![
                "/analyze".to_string(),
                "/detect".to_string(),
                "/parse".to_string(),
                "/extract".to_string(),
            ],
            library_patterns: vec![
                "tree-sitter".to_string(),
                "ast".to_string(),
                "parser".to_string(),
                "analyzer".to_string(),
                "serde".to_string(),
                "cli".to_string(),
                "rust".to_string(),
                "static".to_string(),
                "code".to_string(),
                "codebase".to_string(),
            ],
            file_patterns: vec![
                "codebase".to_string(),
                "analyzer".to_string(),
                "detector".to_string(),
                "parser".to_string(),
                "ast".to_string(),
                "framework".to_string(),
                "intelligence".to_string(),
                "core".to_string(),
                "code_analysis".to_string(),
                "static_analysis".to_string(),
                "developer_tools".to_string(),
            ],
            purpose_description: "Advanced reverse engineering and codebase analysis tool that transforms existing codebases into systematic development workflows, generating PRDs, user stories, and task breakdowns with AI-powered business domain inference.".to_string(),
            target_users: vec![
                "Software developers and engineers".to_string(),
                "Development teams and tech leads".to_string(),
                "Project managers and product owners".to_string(),
                "Code auditors and consultants".to_string(),
            ],
            key_benefits: vec![
                "Automated reverse engineering of existing codebases".to_string(),
                "AI-powered business domain classification and analysis".to_string(),
                "Generation of systematic development workflows and documentation".to_string(),
                "Framework detection and architecture analysis".to_string(),
            ],
        });

        // Add more domain patterns as needed
        
        BusinessPurposeExtractor { domain_patterns }
    }

    pub fn extract_business_purpose(
        &self,
        project_path: &str,
        api_endpoints: &[String],
        dependencies: &[String],
        file_names: &[String],
        readme_content: Option<&str>,
    ) -> Result<BusinessContext> {
        
        // Determine primary domain
        let primary_domain = self.identify_primary_domain(api_endpoints, dependencies, file_names);
        
        // Extract business purpose based on domain
        let purpose = self.generate_business_purpose(&primary_domain, api_endpoints, readme_content)?;
        
        // Generate user personas
        let user_personas = self.generate_user_personas(&primary_domain);
        
        // Extract features from code evidence
        let feature_breakdown = self.extract_features(&primary_domain, api_endpoints, dependencies);
        
        // Generate success indicators
        let success_indicators = self.generate_success_indicators(&primary_domain);

        Ok(BusinessContext {
            purpose,
            user_personas,
            feature_breakdown,
            success_indicators,
        })
    }

    fn identify_primary_domain(
        &self,
        api_endpoints: &[String],
        dependencies: &[String],
        file_names: &[String],
    ) -> String {
        let mut domain_scores: HashMap<String, f32> = HashMap::new();

        for (domain, indicators) in &self.domain_patterns {
            let mut score = 0.0;

            // Score based on API patterns
            for endpoint in api_endpoints {
                for pattern in &indicators.api_patterns {
                    if endpoint.to_lowercase().contains(&pattern.to_lowercase()) {
                        score += 2.0;
                    }
                }
            }

            // Score based on library patterns
            for dep in dependencies {
                for pattern in &indicators.library_patterns {
                    if dep.to_lowercase().contains(&pattern.to_lowercase()) {
                        score += 3.0;
                    }
                }
            }

            // Score based on file patterns
            for file in file_names {
                for pattern in &indicators.file_patterns {
                    if file.to_lowercase().contains(&pattern.to_lowercase()) {
                        score += 1.0;
                    }
                }
            }

            domain_scores.insert(domain.clone(), score);
        }

        domain_scores
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(domain, _)| domain)
            .unwrap_or_else(|| "general_software".to_string())
    }

    fn generate_business_purpose(
        &self,
        domain: &str,
        api_endpoints: &[String],
        readme_content: Option<&str>,
    ) -> Result<BusinessPurpose> {
        
        let domain_info = self.domain_patterns.get(domain);
        
        let description = if let Some(info) = domain_info {
            info.purpose_description.clone()
        } else {
            "Software application providing specialized functionality through a web-based interface.".to_string()
        };

        let problem_statement = self.infer_problem_statement(domain, api_endpoints, readme_content);
        let target_users = domain_info.map(|i| i.target_users.clone()).unwrap_or_default();
        let key_features = self.extract_key_features(api_endpoints, readme_content);
        let value_proposition = self.generate_value_proposition(domain, &key_features);
        let usage_scenarios = self.generate_usage_scenarios(domain, api_endpoints);

        Ok(BusinessPurpose {
            description,
            problem_statement,
            target_users,
            key_features,
            value_proposition,
            usage_scenarios,
            confidence_score: if domain_info.is_some() { 0.85 } else { 0.60 },
        })
    }

    fn infer_problem_statement(
        &self,
        domain: &str,
        api_endpoints: &[String],
        readme_content: Option<&str>,
    ) -> String {
        match domain {
            "audio_processing" => {
                "Musicians and producers need to isolate individual instruments from mixed recordings for remixing, collaboration, and detailed audio editing, but traditional audio editing tools require manual and time-consuming separation processes.".to_string()
            },
            "ecommerce" => {
                "Businesses need an efficient way to sell products online and customers need a reliable platform to discover and purchase items with secure payment processing.".to_string()
            },
            "developer_tools" => {
                "Development teams need to understand and reverse-engineer existing codebases to continue development, but manual code analysis is time-consuming and often misses critical business context and architectural patterns.".to_string()
            },
            _ => {
                "Users need an efficient solution to accomplish their specific domain tasks through a reliable and user-friendly interface.".to_string()
            }
        }
    }

    fn extract_key_features(&self, api_endpoints: &[String], readme_content: Option<&str>) -> Vec<String> {
        let mut features = Vec::new();

        // Extract features from API endpoints
        for endpoint in api_endpoints {
            if endpoint.contains("/process") {
                features.push("Asynchronous processing of user submissions".to_string());
            }
            if endpoint.contains("/status") {
                features.push("Real-time job status tracking".to_string());
            }
            if endpoint.contains("/download") {
                features.push("Downloadable processed results".to_string());
            }
        }

        // Extract features from README if available
        if let Some(readme) = readme_content {
            if readme.to_lowercase().contains("separation") {
                features.push("Audio source separation into individual stems".to_string());
            }
            if readme.to_lowercase().contains("midi") {
                features.push("MIDI extraction from audio recordings".to_string());
            }
            if readme.to_lowercase().contains("api") {
                features.push("RESTful API for programmatic access".to_string());
            }
        }

        if features.is_empty() {
            features.push("Core functionality as determined by system design".to_string());
        }

        features
    }

    fn generate_value_proposition(&self, domain: &str, features: &[String]) -> String {
        match domain {
            "audio_processing" => {
                "Transforms complex audio engineering tasks into simple API calls, enabling musicians and developers to integrate professional-grade audio processing into their workflows without specialized expertise.".to_string()
            },
            "ecommerce" => {
                "Provides a complete online selling solution with secure payments, inventory management, and customer experience optimization.".to_string()
            },
            _ => {
                format!("Delivers {} through an intuitive interface designed for efficiency and reliability.", 
                       features.join(", ").to_lowercase())
            }
        }
    }

    fn generate_usage_scenarios(&self, domain: &str, api_endpoints: &[String]) -> Vec<String> {
        match domain {
            "audio_processing" => {
                vec![
                    "Music producer uploads a mixed track to extract individual instruments for remixing".to_string(),
                    "Collaborative music platform automatically processes user uploads for multi-user editing".to_string(),
                    "Musician extracts MIDI from audio recordings for notation and arrangement work".to_string(),
                    "Audio engineer isolates vocal tracks for cleaning and enhancement".to_string(),
                ]
            },
            _ => {
                vec![
                    "User accesses the system through the web interface".to_string(),
                    "Developer integrates with the system using API endpoints".to_string(),
                    "Administrator manages system configuration and monitoring".to_string(),
                ]
            }
        }
    }

    fn generate_user_personas(&self, domain: &str) -> Vec<UserPersona> {
        match domain {
            "audio_processing" => {
                vec![
                    UserPersona {
                        name: "Music Producer".to_string(),
                        role: "Professional music producer".to_string(),
                        goals: vec![
                            "Quickly isolate instruments for remixing".to_string(),
                            "Create clean stems for collaboration".to_string(),
                            "Analyze musical arrangements".to_string(),
                        ],
                        pain_points: vec![
                            "Manual audio separation is time-consuming".to_string(),
                            "Existing tools require specialized knowledge".to_string(),
                            "Quality varies between different separation methods".to_string(),
                        ],
                    },
                    UserPersona {
                        name: "Platform Developer".to_string(),
                        role: "Developer building music collaboration tools".to_string(),
                        goals: vec![
                            "Integrate audio processing into their platform".to_string(),
                            "Provide seamless user experience".to_string(),
                            "Scale processing for multiple users".to_string(),
                        ],
                        pain_points: vec![
                            "Complex audio processing requires specialized expertise".to_string(),
                            "Infrastructure for audio processing is expensive".to_string(),
                            "Need reliable API for consistent results".to_string(),
                        ],
                    },
                ]
            },
            _ => {
                vec![
                    UserPersona {
                        name: "End User".to_string(),
                        role: "Primary system user".to_string(),
                        goals: vec!["Accomplish tasks efficiently".to_string()],
                        pain_points: vec!["Current solutions are inadequate".to_string()],
                    },
                ]
            }
        }
    }

    fn extract_features(&self, domain: &str, api_endpoints: &[String], dependencies: &[String]) -> Vec<Feature> {
        let mut features = Vec::new();

        if domain == "audio_processing" {
            features.push(Feature {
                name: "Audio Source Separation".to_string(),
                description: "Separates mixed audio recordings into individual instrument stems".to_string(),
                user_benefit: "Enables remixing and individual track editing".to_string(),
                implementation_evidence: vec![
                    "Demucs library integration".to_string(),
                    "/process API endpoint".to_string(),
                    "source_separation module".to_string(),
                ],
            });

            features.push(Feature {
                name: "MIDI Extraction".to_string(),
                description: "Extracts MIDI notation from separated audio stems".to_string(),
                user_benefit: "Allows musical notation editing and arrangement".to_string(),
                implementation_evidence: vec![
                    "basic-pitch library integration".to_string(),
                    "midi_extraction module".to_string(),
                ],
            });
        }

        // Add more domain-specific features as needed

        features
    }

    fn generate_success_indicators(&self, domain: &str) -> Vec<String> {
        match domain {
            "audio_processing" => {
                vec![
                    "High-quality stem separation with minimal artifacts".to_string(),
                    "Fast processing times for user uploads".to_string(),
                    "Accurate MIDI extraction from audio sources".to_string(),
                    "High user satisfaction with separation quality".to_string(),
                    "Successful API integration by platform developers".to_string(),
                ]
            },
            _ => {
                vec![
                    "User engagement and retention".to_string(),
                    "System performance and reliability".to_string(),
                    "Feature adoption rates".to_string(),
                ]
            }
        }
    }
}