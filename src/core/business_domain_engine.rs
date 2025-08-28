use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use walkdir;

use crate::core::framework_detector::{Framework, LanguageEcosystem, FrameworkDetectionResult};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BusinessDomain {
    Authentication,
    UserManagement,
    ECommerce,
    ContentManagement,
    Analytics,
    ApiGateway,
    DataProcessing,
    Notification,
    FileManagement,
    PaymentProcessing,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessDomainResult {
    pub domain: BusinessDomain,
    pub confidence: f32,
    pub evidence: Vec<DomainEvidence>,
    pub story_generation_strategy: StoryGenerationStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvidence {
    pub evidence_type: DomainEvidenceType,
    pub source: String,
    pub pattern: String,
    pub confidence_weight: f32,
    pub framework_context: Option<Framework>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainEvidenceType {
    RoutePattern,       // /auth/login, /users, /products
    ServiceName,        // AuthService, UserService, ProductService
    ModelName,          // User, Product, Order, Payment
    MethodName,         // authenticate(), createUser(), processPayment()
    FileStructure,      // /auth, /users, /products directories
    ImportPattern,      // authentication libraries, payment SDKs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StoryGenerationStrategy {
    Comprehensive,  // High confidence - generate detailed user stories
    Core,          // Medium confidence - generate core user stories
    Minimal,       // Low confidence - mention but don't prioritize
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessDomainAnalysisResult {
    pub primary_domains: Vec<BusinessDomainResult>,
    pub secondary_domains: Vec<BusinessDomainResult>,
    pub confidence_thresholds: ConfidenceThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceThresholds {
    pub high_confidence: f32,    // >= 0.8 - Comprehensive stories
    pub medium_confidence: f32,  // >= 0.6 - Core stories  
    pub low_confidence: f32,     // >= 0.4 - Minimal mention
}

impl Default for ConfidenceThresholds {
    fn default() -> Self {
        Self {
            high_confidence: 0.8,
            medium_confidence: 0.6,
            low_confidence: 0.4,
        }
    }
}

pub struct BusinessDomainEngine {
    pub codebase_path: String,
    pub confidence_thresholds: ConfidenceThresholds,
}

impl BusinessDomainEngine {
    pub fn new(codebase_path: String) -> Self {
        Self {
            codebase_path,
            confidence_thresholds: ConfidenceThresholds::default(),
        }
    }

    /// Main entry point for business domain inference
    pub fn infer_business_domains(&self, framework_result: &FrameworkDetectionResult) -> Result<BusinessDomainAnalysisResult, Box<dyn std::error::Error>> {
        let mut domain_scores: HashMap<BusinessDomain, f32> = HashMap::new();
        let mut all_evidence: HashMap<BusinessDomain, Vec<DomainEvidence>> = HashMap::new();

        // Analyze each detected framework
        for framework in &framework_result.detected_frameworks {
            let framework_evidence = self.analyze_framework_for_domains(framework)?;
            
            for (domain, evidence_list) in framework_evidence {
                // Combine evidence
                all_evidence.entry(domain.clone()).or_insert_with(Vec::new).extend(evidence_list.clone());
                
                // Calculate domain score from evidence
                let domain_score: f32 = evidence_list.iter().map(|e| e.confidence_weight).sum();
                *domain_scores.entry(domain).or_insert(0.0) += domain_score;
            }
        }

        // Create business domain results
        let mut business_domains = Vec::new();
        for (domain, total_score) in domain_scores {
            if total_score >= self.confidence_thresholds.low_confidence {
                let evidence = all_evidence.get(&domain).cloned().unwrap_or_default();
                business_domains.push(BusinessDomainResult {
                    domain,
                    confidence: total_score,
                    evidence,
                    story_generation_strategy: self.determine_story_strategy(total_score),
                });
            }
        }

        // Sort by confidence
        business_domains.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        // Separate primary and secondary domains
        let (primary, secondary) = self.separate_domains(business_domains);

        Ok(BusinessDomainAnalysisResult {
            primary_domains: primary,
            secondary_domains: secondary,
            confidence_thresholds: self.confidence_thresholds.clone(),
        })
    }

    /// Analyze a specific framework for business domain evidence
    fn analyze_framework_for_domains(&self, framework: &crate::core::framework_detector::EnhancedDetectedFramework) -> Result<HashMap<BusinessDomain, Vec<DomainEvidence>>, Box<dyn std::error::Error>> {
        match framework.framework {
            Framework::Flask | Framework::FastAPI => self.analyze_python_framework_domains(&framework.framework),
            Framework::React => self.analyze_react_framework_domains(),
            Framework::NestJS => self.analyze_nestjs_framework_domains(),
            Framework::NextJS => self.analyze_nextjs_framework_domains(),
            Framework::SpringBoot => self.analyze_spring_framework_domains(),
            Framework::Danet => self.analyze_danet_framework_domains(),
            Framework::Unknown => Ok(HashMap::new()),
        }
    }

    /// Analyze Python frameworks (Flask/FastAPI) for business domains
    fn analyze_python_framework_domains(&self, framework: &Framework) -> Result<HashMap<BusinessDomain, Vec<DomainEvidence>>, Box<dyn std::error::Error>> {
        let mut domain_evidence: HashMap<BusinessDomain, Vec<DomainEvidence>> = HashMap::new();

        // Analyze route patterns
        self.analyze_python_routes(&mut domain_evidence, framework)?;
        
        // Analyze service/model names
        self.analyze_python_services(&mut domain_evidence, framework)?;
        
        // Analyze file structure
        self.analyze_python_file_structure(&mut domain_evidence, framework)?;

        // Analyze imports
        self.analyze_python_imports(&mut domain_evidence, framework)?;

        Ok(domain_evidence)
    }

    fn analyze_python_routes(&self, domain_evidence: &mut HashMap<BusinessDomain, Vec<DomainEvidence>>, framework: &Framework) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(&self.codebase_path);
        
        for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() && entry.path().extension().map_or(false, |ext| ext == "py") {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    // Flask route patterns
                    if *framework == Framework::Flask {
                        self.extract_flask_route_evidence(&content, domain_evidence, framework)?;
                    }
                    
                    // FastAPI route patterns
                    if *framework == Framework::FastAPI {
                        self.extract_fastapi_route_evidence(&content, domain_evidence, framework)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn extract_flask_route_evidence(&self, content: &str, domain_evidence: &mut HashMap<BusinessDomain, Vec<DomainEvidence>>, framework: &Framework) -> Result<(), Box<dyn std::error::Error>> {
        // Authentication routes
        if content.contains("@app.route('/auth/login')") || content.contains("@bp.route('/login')") || 
           content.contains("@auth_bp.route(") || content.contains("/auth/") {
            domain_evidence.entry(BusinessDomain::Authentication).or_insert_with(Vec::new).push(
                DomainEvidence {
                    evidence_type: DomainEvidenceType::RoutePattern,
                    source: "Flask routes".to_string(),
                    pattern: "Authentication routes".to_string(),
                    confidence_weight: 0.4,
                    framework_context: Some(framework.clone()),
                }
            );
        }

        // User management routes
        if content.contains("/users") || content.contains("/user/") || content.contains("/profile") {
            domain_evidence.entry(BusinessDomain::UserManagement).or_insert_with(Vec::new).push(
                DomainEvidence {
                    evidence_type: DomainEvidenceType::RoutePattern,
                    source: "Flask routes".to_string(),
                    pattern: "User management routes".to_string(),
                    confidence_weight: 0.3,
                    framework_context: Some(framework.clone()),
                }
            );
        }

        // E-commerce routes
        if content.contains("/products") || content.contains("/cart") || content.contains("/orders") || 
           content.contains("/checkout") || content.contains("/payment") {
            domain_evidence.entry(BusinessDomain::ECommerce).or_insert_with(Vec::new).push(
                DomainEvidence {
                    evidence_type: DomainEvidenceType::RoutePattern,
                    source: "Flask routes".to_string(),
                    pattern: "E-commerce routes".to_string(),
                    confidence_weight: 0.35,
                    framework_context: Some(framework.clone()),
                }
            );
        }

        Ok(())
    }

    fn extract_fastapi_route_evidence(&self, content: &str, domain_evidence: &mut HashMap<BusinessDomain, Vec<DomainEvidence>>, framework: &Framework) -> Result<(), Box<dyn std::error::Error>> {
        // Authentication routes
        if content.contains("@router.post(\"/auth/login\")") || content.contains("@router.post(\"/login\")") ||
           content.contains("/auth/") || content.contains("@auth_router") {
            domain_evidence.entry(BusinessDomain::Authentication).or_insert_with(Vec::new).push(
                DomainEvidence {
                    evidence_type: DomainEvidenceType::RoutePattern,
                    source: "FastAPI routes".to_string(),
                    pattern: "Authentication routes".to_string(),
                    confidence_weight: 0.4,
                    framework_context: Some(framework.clone()),
                }
            );
        }

        // User management routes
        if content.contains("/users") || content.contains("/user/") || content.contains("/profile") {
            domain_evidence.entry(BusinessDomain::UserManagement).or_insert_with(Vec::new).push(
                DomainEvidence {
                    evidence_type: DomainEvidenceType::RoutePattern,
                    source: "FastAPI routes".to_string(),
                    pattern: "User management routes".to_string(),
                    confidence_weight: 0.3,
                    framework_context: Some(framework.clone()),
                }
            );
        }

        // E-commerce routes
        if content.contains("/products") || content.contains("/cart") || content.contains("/orders") {
            domain_evidence.entry(BusinessDomain::ECommerce).or_insert_with(Vec::new).push(
                DomainEvidence {
                    evidence_type: DomainEvidenceType::RoutePattern,
                    source: "FastAPI routes".to_string(),
                    pattern: "E-commerce routes".to_string(),
                    confidence_weight: 0.35,
                    framework_context: Some(framework.clone()),
                }
            );
        }

        Ok(())
    }

    fn analyze_python_services(&self, domain_evidence: &mut HashMap<BusinessDomain, Vec<DomainEvidence>>, framework: &Framework) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(&self.codebase_path);
        
        for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() && entry.path().extension().map_or(false, |ext| ext == "py") {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    // Service class names
                    if content.contains("class AuthService") || content.contains("class AuthenticationService") {
                        domain_evidence.entry(BusinessDomain::Authentication).or_insert_with(Vec::new).push(
                            DomainEvidence {
                                evidence_type: DomainEvidenceType::ServiceName,
                                source: "Python classes".to_string(),
                                pattern: "AuthService/AuthenticationService".to_string(),
                                confidence_weight: 0.3,
                                framework_context: Some(framework.clone()),
                            }
                        );
                    }

                    if content.contains("class UserService") || content.contains("class UserManager") {
                        domain_evidence.entry(BusinessDomain::UserManagement).or_insert_with(Vec::new).push(
                            DomainEvidence {
                                evidence_type: DomainEvidenceType::ServiceName,
                                source: "Python classes".to_string(),
                                pattern: "UserService/UserManager".to_string(),
                                confidence_weight: 0.3,
                                framework_context: Some(framework.clone()),
                            }
                        );
                    }

                    if content.contains("class ProductService") || content.contains("class OrderService") ||
                       content.contains("class PaymentService") {
                        domain_evidence.entry(BusinessDomain::ECommerce).or_insert_with(Vec::new).push(
                            DomainEvidence {
                                evidence_type: DomainEvidenceType::ServiceName,
                                source: "Python classes".to_string(),
                                pattern: "E-commerce services".to_string(),
                                confidence_weight: 0.3,
                                framework_context: Some(framework.clone()),
                            }
                        );
                    }
                }
            }
        }

        Ok(())
    }

    fn analyze_python_file_structure(&self, domain_evidence: &mut HashMap<BusinessDomain, Vec<DomainEvidence>>, framework: &Framework) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(&self.codebase_path);

        // Check for auth-related directories
        if path.join("auth").is_dir() || path.join("authentication").is_dir() || path.join("src/auth").is_dir() {
            domain_evidence.entry(BusinessDomain::Authentication).or_insert_with(Vec::new).push(
                DomainEvidence {
                    evidence_type: DomainEvidenceType::FileStructure,
                    source: "Directory structure".to_string(),
                    pattern: "Auth directory".to_string(),
                    confidence_weight: 0.2,
                    framework_context: Some(framework.clone()),
                }
            );
        }

        // Check for user-related directories
        if path.join("users").is_dir() || path.join("user").is_dir() || path.join("src/users").is_dir() {
            domain_evidence.entry(BusinessDomain::UserManagement).or_insert_with(Vec::new).push(
                DomainEvidence {
                    evidence_type: DomainEvidenceType::FileStructure,
                    source: "Directory structure".to_string(),
                    pattern: "Users directory".to_string(),
                    confidence_weight: 0.2,
                    framework_context: Some(framework.clone()),
                }
            );
        }

        // Check for e-commerce directories
        if path.join("products").is_dir() || path.join("orders").is_dir() || 
           path.join("payments").is_dir() || path.join("cart").is_dir() {
            domain_evidence.entry(BusinessDomain::ECommerce).or_insert_with(Vec::new).push(
                DomainEvidence {
                    evidence_type: DomainEvidenceType::FileStructure,
                    source: "Directory structure".to_string(),
                    pattern: "E-commerce directories".to_string(),
                    confidence_weight: 0.2,
                    framework_context: Some(framework.clone()),
                }
            );
        }

        Ok(())
    }

    fn analyze_python_imports(&self, domain_evidence: &mut HashMap<BusinessDomain, Vec<DomainEvidence>>, framework: &Framework) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(&self.codebase_path);
        
        for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() && entry.path().extension().map_or(false, |ext| ext == "py") {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    // Authentication libraries
                    if content.contains("from flask_login import") || content.contains("import jwt") ||
                       content.contains("from passlib") || content.contains("import bcrypt") ||
                       content.contains("from fastapi.security") {
                        domain_evidence.entry(BusinessDomain::Authentication).or_insert_with(Vec::new).push(
                            DomainEvidence {
                                evidence_type: DomainEvidenceType::ImportPattern,
                                source: "Python imports".to_string(),
                                pattern: "Authentication libraries".to_string(),
                                confidence_weight: 0.25,
                                framework_context: Some(framework.clone()),
                            }
                        );
                    }

                    // Payment processing libraries
                    if content.contains("import stripe") || content.contains("import paypal") ||
                       content.contains("from payment") {
                        domain_evidence.entry(BusinessDomain::PaymentProcessing).or_insert_with(Vec::new).push(
                            DomainEvidence {
                                evidence_type: DomainEvidenceType::ImportPattern,
                                source: "Python imports".to_string(),
                                pattern: "Payment libraries".to_string(),
                                confidence_weight: 0.3,
                                framework_context: Some(framework.clone()),
                            }
                        );
                    }
                }
            }
        }

        Ok(())
    }

    // Placeholder implementations for other frameworks
    fn analyze_react_framework_domains(&self) -> Result<HashMap<BusinessDomain, Vec<DomainEvidence>>, Box<dyn std::error::Error>> {
        // TODO: Implement React domain analysis
        Ok(HashMap::new())
    }

    fn analyze_nestjs_framework_domains(&self) -> Result<HashMap<BusinessDomain, Vec<DomainEvidence>>, Box<dyn std::error::Error>> {
        // TODO: Implement NestJS domain analysis
        Ok(HashMap::new())
    }

    fn analyze_nextjs_framework_domains(&self) -> Result<HashMap<BusinessDomain, Vec<DomainEvidence>>, Box<dyn std::error::Error>> {
        // TODO: Implement Next.js domain analysis
        Ok(HashMap::new())
    }

    fn analyze_spring_framework_domains(&self) -> Result<HashMap<BusinessDomain, Vec<DomainEvidence>>, Box<dyn std::error::Error>> {
        // TODO: Implement Spring Boot domain analysis
        Ok(HashMap::new())
    }

    fn analyze_danet_framework_domains(&self) -> Result<HashMap<BusinessDomain, Vec<DomainEvidence>>, Box<dyn std::error::Error>> {
        // TODO: Implement Danet domain analysis
        Ok(HashMap::new())
    }

    fn determine_story_strategy(&self, confidence: f32) -> StoryGenerationStrategy {
        if confidence >= self.confidence_thresholds.high_confidence {
            StoryGenerationStrategy::Comprehensive
        } else if confidence >= self.confidence_thresholds.medium_confidence {
            StoryGenerationStrategy::Core
        } else {
            StoryGenerationStrategy::Minimal
        }
    }

    fn separate_domains(&self, mut domains: Vec<BusinessDomainResult>) -> (Vec<BusinessDomainResult>, Vec<BusinessDomainResult>) {
        domains.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        
        let split_point = domains.iter().position(|d| d.confidence < self.confidence_thresholds.medium_confidence).unwrap_or(domains.len());
        
        let secondary = domains.split_off(split_point);
        (domains, secondary)
    }
}