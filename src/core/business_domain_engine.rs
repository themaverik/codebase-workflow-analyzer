use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use walkdir;

use crate::core::framework_detector::{FrameworkDetectionResult};
use crate::core::types::{Framework, LanguageEcosystem};
use crate::core::project_analyzer::ProjectContext;
use crate::core::ast_analyzer::CodeSegment;
use crate::intelligence::llm_client::{LocalLLMManager, AnalysisType, ContextAwareAnalysisResult};

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

    /// Context-aware business domain inference using LLM with project context
    pub async fn infer_business_domains_with_context(
        &self,
        framework_result: &FrameworkDetectionResult,
        project_context: &ProjectContext,
        code_segments: &[CodeSegment],
        llm_manager: &LocalLLMManager,
    ) -> Result<BusinessDomainAnalysisResult, Box<dyn std::error::Error>> {
        println!("🧠 Starting enhanced context-aware business domain analysis...");
        
        // First, get traditional pattern-based analysis
        let traditional_result = self.infer_business_domains(framework_result)?;
        
        println!("  Traditional analysis found {} primary domains", traditional_result.primary_domains.len());
        
        // Enhanced multi-tier LLM analysis
        let enhanced_llm_result = self.perform_enhanced_llm_analysis(
            code_segments, 
            project_context, 
            framework_result,
            &traditional_result,
            llm_manager
        ).await?;
        
        // Merge traditional and enhanced LLM results with confidence weighting
        let final_result = self.merge_enhanced_analysis_results(
            traditional_result,
            enhanced_llm_result,
            project_context,
        )?;
        
        println!("✅ Enhanced domain analysis complete with {} primary domains", final_result.primary_domains.len());
        
        Ok(final_result)
    }

    /// Perform enhanced multi-tier LLM analysis with domain-specific prompting
    async fn perform_enhanced_llm_analysis(
        &self,
        code_segments: &[CodeSegment],
        project_context: &ProjectContext,
        framework_result: &FrameworkDetectionResult,
        traditional_result: &BusinessDomainAnalysisResult,
        llm_manager: &LocalLLMManager,
    ) -> Result<EnhancedLLMDomainResult, Box<dyn std::error::Error>> {
        println!("  🎯 Performing enhanced LLM analysis with domain-specific prompting...");
        
        // Stage 1: Project-level business context analysis
        let business_context_result = self.analyze_business_context_with_llm(
            project_context, framework_result, llm_manager
        ).await?;
        
        // Stage 2: Code semantic analysis with domain hints
        let semantic_analysis_result = self.analyze_code_semantics_with_domain_hints(
            code_segments, &business_context_result, traditional_result, llm_manager
        ).await?;
        
        // Stage 3: Domain validation and confidence refinement
        let validated_result = self.validate_and_refine_domain_analysis(
            &business_context_result, 
            &semantic_analysis_result,
            project_context,
            llm_manager
        ).await?;
        
        Ok(validated_result)
    }

    /// Stage 1: Analyze business context using project-level information
    async fn analyze_business_context_with_llm(
        &self,
        project_context: &ProjectContext,
        framework_result: &FrameworkDetectionResult,
        _llm_manager: &LocalLLMManager,
    ) -> Result<BusinessContextAnalysisResult, Box<dyn std::error::Error>> {
        println!("    📋 Stage 1: Business context analysis");
        
        // For now, use simplified analysis based on project context
        // In a future iteration, this would use the LLM for sophisticated analysis
        let business_type = self.infer_business_type_from_context(project_context, framework_result);
        let domain_candidates = self.suggest_domain_candidates(&business_type, project_context);
        
        let result = BusinessContextAnalysisResult {
            business_type: business_type.clone(),
            domain_candidates: domain_candidates.clone(),
            target_users: vec!["Developers".to_string(), "System Administrators".to_string()],
            business_capabilities: self.extract_business_capabilities(project_context),
            confidence: 0.75,
            reasoning: format!("Inferred from project type: {}", business_type),
        };
        
        println!("      Identified business type: {}", result.business_type);
        println!("      Primary domain candidates: {:?}", result.domain_candidates);
        
        Ok(result)
    }

    /// Stage 2: Analyze code semantics with domain-specific hints
    async fn analyze_code_semantics_with_domain_hints(
        &self,
        code_segments: &[CodeSegment],
        business_context: &BusinessContextAnalysisResult,
        traditional_result: &BusinessDomainAnalysisResult,
        _llm_manager: &LocalLLMManager,
    ) -> Result<SemanticAnalysisResult, Box<dyn std::error::Error>> {
        println!("    🔍 Stage 2: Code semantic analysis with domain hints");
        
        // Use enhanced pattern analysis instead of LLM for now
        let mut semantic_evidences = Vec::new();
        
        // Analyze code segments for domain-specific patterns
        for domain_candidate in &business_context.domain_candidates {
            println!("      Analyzing for {} domain patterns", domain_candidate);
            
            let evidence = self.analyze_code_for_domain_patterns(code_segments, domain_candidate)?;
            semantic_evidences.extend(evidence);
        }
        
        // Boost evidence based on traditional analysis alignment
        self.align_with_traditional_analysis(&mut semantic_evidences, traditional_result);
        
        Ok(SemanticAnalysisResult {
            domain_evidences: semantic_evidences,
            confidence_adjustments: HashMap::new(),
        })
    }

    /// Stage 3: Validate and refine domain analysis
    async fn validate_and_refine_domain_analysis(
        &self,
        business_context: &BusinessContextAnalysisResult,
        semantic_analysis: &SemanticAnalysisResult,
        project_context: &ProjectContext,
        _llm_manager: &LocalLLMManager,
    ) -> Result<EnhancedLLMDomainResult, Box<dyn std::error::Error>> {
        println!("    ✅ Stage 3: Domain validation and refinement");
        
        // Perform rule-based validation and confidence refinement
        let validated_domains = self.validate_domains_with_evidence(
            business_context, semantic_analysis, project_context
        )?;
        
        let final_result = EnhancedLLMDomainResult {
            validated_domains: validated_domains.clone(),
            overall_confidence: self.calculate_overall_confidence(&validated_domains),
            primary_domain: validated_domains.first()
                .map(|d| d.domain_name.clone())
                .unwrap_or_else(|| "Unknown".to_string()),
        };
        
        println!("      Final validated domains: {:?}", final_result.validated_domains.iter().map(|d| &d.domain_name).collect::<Vec<_>>());
        println!("      Overall confidence: {:.1}%", final_result.overall_confidence * 100.0);
        
        Ok(final_result)
    }

    /// Merge traditional pattern-based analysis with context-aware LLM analysis
    fn merge_traditional_and_llm_analysis(
        &self,
        mut traditional_result: BusinessDomainAnalysisResult,
        llm_result: &ContextAwareAnalysisResult,
        project_context: &ProjectContext,
    ) -> Result<BusinessDomainAnalysisResult, Box<dyn std::error::Error>> {
        println!("🔀 Merging traditional and LLM analysis results...");
        
        // Convert LLM analysis to BusinessDomain
        let llm_primary_domain = self.map_llm_domain_to_business_domain(&llm_result.business_domain_analysis.primary_domain);
        let llm_confidence = llm_result.business_domain_analysis.domain_confidence;
        
        // Check if LLM identified a domain not found by traditional analysis
        let llm_domain_exists = traditional_result.primary_domains.iter()
            .chain(traditional_result.secondary_domains.iter())
            .any(|domain| domain.domain == llm_primary_domain);
        
        if !llm_domain_exists && llm_confidence >= 0.6 {
            println!("  LLM identified new primary domain: {:?} (confidence: {:.1}%)", llm_primary_domain, llm_confidence * 100.0);
            
            // Create LLM-based domain result
            let llm_domain_result = BusinessDomainResult {
                domain: llm_primary_domain,
                confidence: llm_confidence,
                evidence: vec![
                    DomainEvidence {
                        evidence_type: DomainEvidenceType::ServiceName,
                        source: "LLM Context Analysis".to_string(),
                        pattern: llm_result.business_domain_analysis.business_context.clone(),
                        confidence_weight: llm_confidence,
                        framework_context: None,
                    }
                ],
                story_generation_strategy: self.determine_story_strategy(llm_confidence),
            };
            
            // Add as primary domain if high confidence, otherwise secondary
            if llm_confidence >= self.confidence_thresholds.high_confidence {
                traditional_result.primary_domains.insert(0, llm_domain_result);
            } else {
                traditional_result.secondary_domains.push(llm_domain_result);
            }
        } else if llm_domain_exists {
            println!("  LLM confirmed existing domain analysis");
            // Boost confidence of existing domain identified by LLM
            self.boost_existing_domain_confidence(&mut traditional_result, &llm_primary_domain, llm_confidence);
        }
        
        // Re-sort domains by confidence after merging
        traditional_result.primary_domains.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        traditional_result.secondary_domains.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        
        // Apply project-type-specific domain filtering
        self.apply_project_type_filtering(&mut traditional_result, project_context);
        
        Ok(traditional_result)
    }

    /// Map LLM domain string to BusinessDomain enum
    fn map_llm_domain_to_business_domain(&self, llm_domain: &str) -> BusinessDomain {
        let domain_lower = llm_domain.to_lowercase();
        
        match domain_lower.as_str() {
            s if s.contains("auth") || s.contains("login") || s.contains("security") => BusinessDomain::Authentication,
            s if s.contains("user") || s.contains("profile") || s.contains("account") => BusinessDomain::UserManagement,
            s if s.contains("commerce") || s.contains("shop") || s.contains("product") || s.contains("cart") => BusinessDomain::ECommerce,
            s if s.contains("content") || s.contains("cms") || s.contains("blog") || s.contains("article") => BusinessDomain::ContentManagement,
            s if s.contains("analytic") || s.contains("metric") || s.contains("report") || s.contains("dashboard") => BusinessDomain::Analytics,
            s if s.contains("api") || s.contains("gateway") || s.contains("proxy") => BusinessDomain::ApiGateway,
            s if s.contains("data") || s.contains("process") || s.contains("etl") || s.contains("pipeline") => BusinessDomain::DataProcessing,
            s if s.contains("notification") || s.contains("message") || s.contains("email") || s.contains("sms") => BusinessDomain::Notification,
            s if s.contains("file") || s.contains("upload") || s.contains("storage") || s.contains("document") => BusinessDomain::FileManagement,
            s if s.contains("payment") || s.contains("billing") || s.contains("invoice") || s.contains("subscription") => BusinessDomain::PaymentProcessing,
            s if s.contains("codebase") || s.contains("analysis") || s.contains("intelligence") || s.contains("workflow") => BusinessDomain::Analytics, // Map analyzer to Analytics
            _ => BusinessDomain::Unknown,
        }
    }

    /// Boost confidence of existing domain identified by LLM
    fn boost_existing_domain_confidence(
        &self,
        result: &mut BusinessDomainAnalysisResult,
        target_domain: &BusinessDomain,
        llm_confidence: f32,
    ) {
        let boost_factor = 0.2; // 20% confidence boost for LLM confirmation
        
        // Check primary domains
        for domain in &mut result.primary_domains {
            if domain.domain == *target_domain {
                domain.confidence = (domain.confidence + (llm_confidence * boost_factor)).min(1.0);
                domain.evidence.push(DomainEvidence {
                    evidence_type: DomainEvidenceType::ServiceName,
                    source: "LLM Confirmation".to_string(),
                    pattern: "Context-aware analysis confirmation".to_string(),
                    confidence_weight: llm_confidence * boost_factor,
                    framework_context: None,
                });
                println!("    Boosted {} confidence to {:.1}%", format!("{:?}", target_domain), domain.confidence * 100.0);
                return;
            }
        }
        
        // Check secondary domains
        for domain in &mut result.secondary_domains {
            if domain.domain == *target_domain {
                domain.confidence = (domain.confidence + (llm_confidence * boost_factor)).min(1.0);
                domain.evidence.push(DomainEvidence {
                    evidence_type: DomainEvidenceType::ServiceName,
                    source: "LLM Confirmation".to_string(),
                    pattern: "Context-aware analysis confirmation".to_string(),
                    confidence_weight: llm_confidence * boost_factor,
                    framework_context: None,
                });
                println!("    Boosted {} confidence to {:.1}%", format!("{:?}", target_domain), domain.confidence * 100.0);
                return;
            }
        }
    }

    /// Apply project-type-specific domain filtering to improve accuracy
    fn apply_project_type_filtering(
        &self,
        result: &mut BusinessDomainAnalysisResult,
        project_context: &ProjectContext,
    ) {
        if let Some(ref project_type) = project_context.project_type {
            match project_type {
                crate::core::project_classifier::ProjectType::AnalysisTool => {
                    println!("  Applying AnalysisTool domain filtering...");
                    
                    // For analysis tools, prioritize Analytics and DataProcessing domains
                    self.prioritize_domains(result, &[BusinessDomain::Analytics, BusinessDomain::DataProcessing]);
                    
                    // De-prioritize user-facing domains that don't make sense for analysis tools
                    self.deprioritize_domains(result, &[BusinessDomain::UserManagement, BusinessDomain::ECommerce, BusinessDomain::ContentManagement]);
                },
                crate::core::project_classifier::ProjectType::WebApplication => {
                    println!("  Applying WebApplication domain filtering...");
                    self.prioritize_domains(result, &[BusinessDomain::UserManagement, BusinessDomain::Authentication]);
                },
                crate::core::project_classifier::ProjectType::ApiService => {
                    println!("  Applying ApiService domain filtering...");
                    self.prioritize_domains(result, &[BusinessDomain::ApiGateway, BusinessDomain::DataProcessing]);
                },
                _ => {
                    println!("  No specific domain filtering for project type: {:?}", project_type);
                }
            }
        }
    }

    /// Prioritize specific domains by boosting their confidence
    fn prioritize_domains(&self, result: &mut BusinessDomainAnalysisResult, priority_domains: &[BusinessDomain]) {
        let priority_boost = 0.15; // 15% boost for priority domains
        
        for priority_domain in priority_domains {
            // Boost in primary domains
            for domain in &mut result.primary_domains {
                if domain.domain == *priority_domain {
                    domain.confidence = (domain.confidence + priority_boost).min(1.0);
                    println!("    Prioritized {} to {:.1}% confidence", format!("{:?}", priority_domain), domain.confidence * 100.0);
                }
            }
            
            // Boost in secondary domains
            for domain in &mut result.secondary_domains {
                if domain.domain == *priority_domain {
                    domain.confidence = (domain.confidence + priority_boost).min(1.0);
                    println!("    Prioritized {} to {:.1}% confidence", format!("{:?}", priority_domain), domain.confidence * 100.0);
                }
            }
        }
    }

    /// De-prioritize specific domains by reducing their confidence
    fn deprioritize_domains(&self, result: &mut BusinessDomainAnalysisResult, depriority_domains: &[BusinessDomain]) {
        let depriority_reduction = 0.2; // 20% reduction for non-relevant domains
        
        for depriority_domain in depriority_domains {
            // Reduce in primary domains
            for domain in &mut result.primary_domains {
                if domain.domain == *depriority_domain {
                    domain.confidence = (domain.confidence - depriority_reduction).max(0.0);
                    println!("    De-prioritized {} to {:.1}% confidence", format!("{:?}", depriority_domain), domain.confidence * 100.0);
                }
            }
            
            // Reduce in secondary domains
            for domain in &mut result.secondary_domains {
                if domain.domain == *depriority_domain {
                    domain.confidence = (domain.confidence - depriority_reduction).max(0.0);
                    println!("    De-prioritized {} to {:.1}% confidence", format!("{:?}", depriority_domain), domain.confidence * 100.0);
                }
            }
        }
        
        // Remove domains that fell below minimum confidence
        result.primary_domains.retain(|d| d.confidence >= self.confidence_thresholds.low_confidence);
        result.secondary_domains.retain(|d| d.confidence >= self.confidence_thresholds.low_confidence);
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
            _ => Ok(HashMap::new()), // TODO: Implement domain analysis for other frameworks
        }
    }

    /// Analyze Python frameworks (Flask/FastAPI) for business domains
    fn analyze_python_framework_domains(&self, framework: &Framework) -> Result<HashMap<BusinessDomain, Vec<DomainEvidence>>, Box<dyn std::error::Error>> {
        let mut domain_evidence: HashMap<BusinessDomain, Vec<DomainEvidence>> = HashMap::new();

        // Enhanced analysis with multiple detection strategies
        self.analyze_python_routes(&mut domain_evidence, framework)?;
        self.analyze_python_services(&mut domain_evidence, framework)?;
        self.analyze_python_file_structure(&mut domain_evidence, framework)?;
        self.analyze_python_imports(&mut domain_evidence, framework)?;
        
        // New enhanced detection methods
        self.analyze_python_method_patterns(&mut domain_evidence, framework)?;
        self.analyze_python_model_relationships(&mut domain_evidence, framework)?;
        self.analyze_python_security_patterns(&mut domain_evidence, framework)?;
        self.analyze_python_business_logic(&mut domain_evidence, framework)?;

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

    /// Enhanced method pattern analysis for Python frameworks
    fn analyze_python_method_patterns(&self, domain_evidence: &mut HashMap<BusinessDomain, Vec<DomainEvidence>>, framework: &Framework) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(&self.codebase_path);
        
        for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() && entry.path().extension().map_or(false, |ext| ext == "py") {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    self.extract_python_method_evidence(&content, domain_evidence, framework)?;
                }
            }
        }
        Ok(())
    }

    /// Analyze Python model relationships for business domain inference
    fn analyze_python_model_relationships(&self, domain_evidence: &mut HashMap<BusinessDomain, Vec<DomainEvidence>>, framework: &Framework) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(&self.codebase_path);
        
        for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() && 
               (entry.path().extension().map_or(false, |ext| ext == "py") &&
                (entry.path().to_string_lossy().contains("models") || entry.path().to_string_lossy().contains("schema"))) {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    self.extract_python_model_evidence(&content, domain_evidence, framework)?;
                }
            }
        }
        Ok(())
    }

    /// Analyze Python security patterns
    fn analyze_python_security_patterns(&self, domain_evidence: &mut HashMap<BusinessDomain, Vec<DomainEvidence>>, framework: &Framework) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(&self.codebase_path);
        
        for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() && entry.path().extension().map_or(false, |ext| ext == "py") {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    self.extract_python_security_evidence(&content, domain_evidence, framework)?;
                }
            }
        }
        Ok(())
    }

    /// Analyze Python business logic patterns
    fn analyze_python_business_logic(&self, domain_evidence: &mut HashMap<BusinessDomain, Vec<DomainEvidence>>, framework: &Framework) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(&self.codebase_path);
        
        for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() && entry.path().extension().map_or(false, |ext| ext == "py") {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    self.extract_python_business_logic_evidence(&content, domain_evidence, framework)?;
                }
            }
        }
        Ok(())
    }

    /// Extract method-based evidence for domain classification
    fn extract_python_method_evidence(&self, content: &str, domain_evidence: &mut HashMap<BusinessDomain, Vec<DomainEvidence>>, framework: &Framework) -> Result<(), Box<dyn std::error::Error>> {
        // Authentication method patterns
        let auth_methods = [
            "def login", "def logout", "def authenticate", "def verify_token", "def hash_password",
            "def check_password", "def generate_token", "def validate_user", "def verify_credentials"
        ];
        
        for method in &auth_methods {
            if content.contains(method) {
                domain_evidence.entry(BusinessDomain::Authentication).or_insert_with(Vec::new).push(
                    DomainEvidence {
                        evidence_type: DomainEvidenceType::MethodName,
                        source: "Python method patterns".to_string(),
                        pattern: format!("Authentication method: {}", method),
                        confidence_weight: 0.35,
                        framework_context: Some(framework.clone()),
                    }
                );
            }
        }

        // User management method patterns
        let user_methods = [
            "def create_user", "def get_user", "def update_user", "def delete_user", "def get_users",
            "def update_profile", "def get_profile", "def activate_user", "def deactivate_user"
        ];
        
        for method in &user_methods {
            if content.contains(method) {
                domain_evidence.entry(BusinessDomain::UserManagement).or_insert_with(Vec::new).push(
                    DomainEvidence {
                        evidence_type: DomainEvidenceType::MethodName,
                        source: "Python method patterns".to_string(),
                        pattern: format!("User management method: {}", method),
                        confidence_weight: 0.4,
                        framework_context: Some(framework.clone()),
                    }
                );
            }
        }

        // E-commerce method patterns
        let ecommerce_methods = [
            "def create_order", "def process_payment", "def add_to_cart", "def checkout", "def calculate_total",
            "def apply_discount", "def process_refund", "def update_inventory", "def get_products"
        ];
        
        for method in &ecommerce_methods {
            if content.contains(method) {
                domain_evidence.entry(BusinessDomain::ECommerce).or_insert_with(Vec::new).push(
                    DomainEvidence {
                        evidence_type: DomainEvidenceType::MethodName,
                        source: "Python method patterns".to_string(),
                        pattern: format!("E-commerce method: {}", method),
                        confidence_weight: 0.45,
                        framework_context: Some(framework.clone()),
                    }
                );
            }
        }

        // Analytics method patterns
        let analytics_methods = [
            "def track_event", "def analyze_data", "def generate_report", "def calculate_metrics",
            "def process_analytics", "def aggregate_data", "def create_dashboard"
        ];
        
        for method in &analytics_methods {
            if content.contains(method) {
                domain_evidence.entry(BusinessDomain::Analytics).or_insert_with(Vec::new).push(
                    DomainEvidence {
                        evidence_type: DomainEvidenceType::MethodName,
                        source: "Python method patterns".to_string(),
                        pattern: format!("Analytics method: {}", method),
                        confidence_weight: 0.4,
                        framework_context: Some(framework.clone()),
                    }
                );
            }
        }

        Ok(())
    }

    /// Extract model-based evidence for domain classification
    fn extract_python_model_evidence(&self, content: &str, domain_evidence: &mut HashMap<BusinessDomain, Vec<DomainEvidence>>, framework: &Framework) -> Result<(), Box<dyn std::error::Error>> {
        // User management models
        let user_models = ["class User", "class Profile", "class Account", "class Role", "class Permission"];
        for model in &user_models {
            if content.contains(model) {
                domain_evidence.entry(BusinessDomain::UserManagement).or_insert_with(Vec::new).push(
                    DomainEvidence {
                        evidence_type: DomainEvidenceType::ModelName,
                        source: "Python model patterns".to_string(),
                        pattern: format!("User model: {}", model),
                        confidence_weight: 0.5,
                        framework_context: Some(framework.clone()),
                    }
                );
            }
        }

        // E-commerce models
        let ecommerce_models = ["class Product", "class Order", "class Cart", "class Payment", "class Invoice", "class Category"];
        for model in &ecommerce_models {
            if content.contains(model) {
                domain_evidence.entry(BusinessDomain::ECommerce).or_insert_with(Vec::new).push(
                    DomainEvidence {
                        evidence_type: DomainEvidenceType::ModelName,
                        source: "Python model patterns".to_string(),
                        pattern: format!("E-commerce model: {}", model),
                        confidence_weight: 0.5,
                        framework_context: Some(framework.clone()),
                    }
                );
            }
        }

        // Content management models
        let cms_models = ["class Article", "class Post", "class Page", "class Content", "class Media", "class Tag"];
        for model in &cms_models {
            if content.contains(model) {
                domain_evidence.entry(BusinessDomain::ContentManagement).or_insert_with(Vec::new).push(
                    DomainEvidence {
                        evidence_type: DomainEvidenceType::ModelName,
                        source: "Python model patterns".to_string(),
                        pattern: format!("CMS model: {}", model),
                        confidence_weight: 0.45,
                        framework_context: Some(framework.clone()),
                    }
                );
            }
        }

        Ok(())
    }

    /// Extract security pattern evidence
    fn extract_python_security_evidence(&self, content: &str, domain_evidence: &mut HashMap<BusinessDomain, Vec<DomainEvidence>>, framework: &Framework) -> Result<(), Box<dyn std::error::Error>> {
        // Authentication security patterns
        let auth_patterns = [
            "@login_required", "@jwt_required", "@token_required", "require_auth", 
            "check_permission", "verify_token", "validate_session"
        ];
        
        for pattern in &auth_patterns {
            if content.contains(pattern) {
                domain_evidence.entry(BusinessDomain::Authentication).or_insert_with(Vec::new).push(
                    DomainEvidence {
                        evidence_type: DomainEvidenceType::ImportPattern,
                        source: "Python security patterns".to_string(),
                        pattern: format!("Security decorator: {}", pattern),
                        confidence_weight: 0.3,
                        framework_context: Some(framework.clone()),
                    }
                );
            }
        }

        Ok(())
    }

    /// Extract business logic evidence
    fn extract_python_business_logic_evidence(&self, content: &str, domain_evidence: &mut HashMap<BusinessDomain, Vec<DomainEvidence>>, framework: &Framework) -> Result<(), Box<dyn std::error::Error>> {
        // File processing patterns
        if content.contains("def upload_file") || content.contains("def download_file") || 
           content.contains("def process_file") || content.contains("import boto3") {
            domain_evidence.entry(BusinessDomain::FileManagement).or_insert_with(Vec::new).push(
                DomainEvidence {
                    evidence_type: DomainEvidenceType::MethodName,
                    source: "Python business logic".to_string(),
                    pattern: "File management operations".to_string(),
                    confidence_weight: 0.4,
                    framework_context: Some(framework.clone()),
                }
            );
        }

        // Notification patterns
        if content.contains("def send_email") || content.contains("def send_notification") || 
           content.contains("def send_sms") || content.contains("import sendgrid") {
            domain_evidence.entry(BusinessDomain::Notification).or_insert_with(Vec::new).push(
                DomainEvidence {
                    evidence_type: DomainEvidenceType::MethodName,
                    source: "Python business logic".to_string(),
                    pattern: "Notification operations".to_string(),
                    confidence_weight: 0.35,
                    framework_context: Some(framework.clone()),
                }
            );
        }

        // Data processing patterns
        if content.contains("import pandas") || content.contains("import numpy") || 
           content.contains("def process_data") || content.contains("def transform_data") {
            domain_evidence.entry(BusinessDomain::DataProcessing).or_insert_with(Vec::new).push(
                DomainEvidence {
                    evidence_type: DomainEvidenceType::ImportPattern,
                    source: "Python business logic".to_string(),
                    pattern: "Data processing operations".to_string(),
                    confidence_weight: 0.4,
                    framework_context: Some(framework.clone()),
                }
            );
        }

        Ok(())
    }

    // Placeholder implementations for other frameworks
    fn analyze_react_framework_domains(&self) -> Result<HashMap<BusinessDomain, Vec<DomainEvidence>>, Box<dyn std::error::Error>> {
        // TODO: Implement React domain analysis
        Ok(HashMap::new())
    }

    fn analyze_nestjs_framework_domains(&self) -> Result<HashMap<BusinessDomain, Vec<DomainEvidence>>, Box<dyn std::error::Error>> {
        let mut domain_evidence: HashMap<BusinessDomain, Vec<DomainEvidence>> = HashMap::new();

        // Enhanced NestJS analysis with multiple strategies
        self.analyze_nestjs_controllers(&mut domain_evidence)?;
        self.analyze_nestjs_services(&mut domain_evidence)?;
        self.analyze_nestjs_file_structure(&mut domain_evidence)?;
        self.analyze_nestjs_usecases(&mut domain_evidence)?;
        
        // New enhanced detection methods for NestJS
        self.analyze_nestjs_decorators(&mut domain_evidence)?;
        self.analyze_nestjs_guards_pipes(&mut domain_evidence)?;
        self.analyze_nestjs_entities(&mut domain_evidence)?;
        self.analyze_nestjs_dtos(&mut domain_evidence)?;

        Ok(domain_evidence)
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

    /// Analyze NestJS controllers for business domain patterns
    fn analyze_nestjs_controllers(&self, domain_evidence: &mut HashMap<BusinessDomain, Vec<DomainEvidence>>) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(&self.codebase_path);
        
        for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() && 
               entry.path().extension().map_or(false, |ext| ext == "ts") &&
               entry.path().to_string_lossy().contains("controller") {
                
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    // Authentication controllers
                    if content.contains("@Controller('auth')") || 
                       content.contains("@Controller('/auth')") ||
                       entry.path().to_string_lossy().contains("auth") {
                        domain_evidence.entry(BusinessDomain::Authentication).or_insert_with(Vec::new).push(
                            DomainEvidence {
                                evidence_type: DomainEvidenceType::RoutePattern,
                                source: "NestJS controller".to_string(),
                                pattern: "Authentication controller".to_string(),
                                confidence_weight: 0.4,
                                framework_context: Some(Framework::NestJS),
                            }
                        );
                    }

                    // User management controllers
                    if content.contains("@Controller('users')") || 
                       content.contains("@Controller('/users')") ||
                       content.contains("@Controller('user')") ||
                       entry.path().to_string_lossy().contains("user") {
                        domain_evidence.entry(BusinessDomain::UserManagement).or_insert_with(Vec::new).push(
                            DomainEvidence {
                                evidence_type: DomainEvidenceType::RoutePattern,
                                source: "NestJS controller".to_string(),
                                pattern: "User management controller".to_string(),
                                confidence_weight: 0.35,
                                framework_context: Some(Framework::NestJS),
                            }
                        );
                    }

                    // Notification controllers
                    if content.contains("@Controller('notifications')") || 
                       content.contains("@Controller('/notifications')") ||
                       content.contains("@Controller('notification')") ||
                       entry.path().to_string_lossy().contains("notification") {
                        domain_evidence.entry(BusinessDomain::Notification).or_insert_with(Vec::new).push(
                            DomainEvidence {
                                evidence_type: DomainEvidenceType::RoutePattern,
                                source: "NestJS controller".to_string(),
                                pattern: "Notification controller".to_string(),
                                confidence_weight: 0.4,
                                framework_context: Some(Framework::NestJS),
                            }
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Analyze NestJS services for business domain patterns
    fn analyze_nestjs_services(&self, domain_evidence: &mut HashMap<BusinessDomain, Vec<DomainEvidence>>) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(&self.codebase_path);
        
        for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() && 
               entry.path().extension().map_or(false, |ext| ext == "ts") &&
               entry.path().to_string_lossy().contains("service") {
                
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    // Service class names
                    if content.contains("class AuthService") || 
                       content.contains("class AuthenticationService") ||
                       entry.path().to_string_lossy().contains("auth") {
                        domain_evidence.entry(BusinessDomain::Authentication).or_insert_with(Vec::new).push(
                            DomainEvidence {
                                evidence_type: DomainEvidenceType::ServiceName,
                                source: "NestJS service".to_string(),
                                pattern: "AuthService/AuthenticationService".to_string(),
                                confidence_weight: 0.3,
                                framework_context: Some(Framework::NestJS),
                            }
                        );
                    }

                    if content.contains("class UserService") || 
                       content.contains("class UserManagementService") ||
                       entry.path().to_string_lossy().contains("user") {
                        domain_evidence.entry(BusinessDomain::UserManagement).or_insert_with(Vec::new).push(
                            DomainEvidence {
                                evidence_type: DomainEvidenceType::ServiceName,
                                source: "NestJS service".to_string(),
                                pattern: "UserService/UserManagementService".to_string(),
                                confidence_weight: 0.3,
                                framework_context: Some(Framework::NestJS),
                            }
                        );
                    }

                    if content.contains("class NotificationService") ||
                       content.contains("class NotificationsService") ||
                       entry.path().to_string_lossy().contains("notification") {
                        domain_evidence.entry(BusinessDomain::Notification).or_insert_with(Vec::new).push(
                            DomainEvidence {
                                evidence_type: DomainEvidenceType::ServiceName,
                                source: "NestJS service".to_string(),
                                pattern: "NotificationService".to_string(),
                                confidence_weight: 0.35,
                                framework_context: Some(Framework::NestJS),
                            }
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Analyze NestJS file structure for business domain patterns
    fn analyze_nestjs_file_structure(&self, domain_evidence: &mut HashMap<BusinessDomain, Vec<DomainEvidence>>) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(&self.codebase_path);

        // Check for domain-specific directories
        if path.join("src/auth").is_dir() || path.join("src/authentication").is_dir() {
            domain_evidence.entry(BusinessDomain::Authentication).or_insert_with(Vec::new).push(
                DomainEvidence {
                    evidence_type: DomainEvidenceType::FileStructure,
                    source: "Directory structure".to_string(),
                    pattern: "Auth directory".to_string(),
                    confidence_weight: 0.2,
                    framework_context: Some(Framework::NestJS),
                }
            );
        }

        if path.join("src/users").is_dir() || path.join("src/user").is_dir() {
            domain_evidence.entry(BusinessDomain::UserManagement).or_insert_with(Vec::new).push(
                DomainEvidence {
                    evidence_type: DomainEvidenceType::FileStructure,
                    source: "Directory structure".to_string(),
                    pattern: "Users directory".to_string(),
                    confidence_weight: 0.2,
                    framework_context: Some(Framework::NestJS),
                }
            );
        }

        if path.join("src/notifications").is_dir() || path.join("src/notification").is_dir() {
            domain_evidence.entry(BusinessDomain::Notification).or_insert_with(Vec::new).push(
                DomainEvidence {
                    evidence_type: DomainEvidenceType::FileStructure,
                    source: "Directory structure".to_string(),
                    pattern: "Notifications directory".to_string(),
                    confidence_weight: 0.25,
                    framework_context: Some(Framework::NestJS),
                }
            );
        }

        Ok(())
    }

    /// Analyze NestJS use cases for business domain patterns  
    fn analyze_nestjs_usecases(&self, domain_evidence: &mut HashMap<BusinessDomain, Vec<DomainEvidence>>) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(&self.codebase_path);
        
        for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() && 
               entry.path().extension().map_or(false, |ext| ext == "ts") &&
               (entry.path().to_string_lossy().contains("usecase") || entry.path().to_string_lossy().contains("usecases")) {
                
                let file_path = entry.path().to_string_lossy();
                
                // Notification use cases
                if file_path.contains("notification") {
                    if file_path.contains("markAsRead") || 
                       file_path.contains("markAllAsRead") ||
                       file_path.contains("getAllNotifications") ||
                       file_path.contains("markAsDeleted") ||
                       file_path.contains("send") ||
                       file_path.contains("create") {
                        domain_evidence.entry(BusinessDomain::Notification).or_insert_with(Vec::new).push(
                            DomainEvidence {
                                evidence_type: DomainEvidenceType::MethodName,
                                source: "NestJS use case".to_string(),
                                pattern: "Notification use cases".to_string(),
                                confidence_weight: 0.3,
                                framework_context: Some(Framework::NestJS),
                            }
                        );
                    }
                }

                // Authentication use cases
                if file_path.contains("auth") {
                    domain_evidence.entry(BusinessDomain::Authentication).or_insert_with(Vec::new).push(
                        DomainEvidence {
                            evidence_type: DomainEvidenceType::MethodName,
                            source: "NestJS use case".to_string(),
                            pattern: "Authentication use cases".to_string(),
                            confidence_weight: 0.3,
                            framework_context: Some(Framework::NestJS),
                        }
                    );
                }

                // User management use cases
                if file_path.contains("user") {
                    domain_evidence.entry(BusinessDomain::UserManagement).or_insert_with(Vec::new).push(
                        DomainEvidence {
                            evidence_type: DomainEvidenceType::MethodName,
                            source: "NestJS use case".to_string(),
                            pattern: "User management use cases".to_string(),
                            confidence_weight: 0.3,
                            framework_context: Some(Framework::NestJS),
                        }
                    );
                }
            }
        }

        Ok(())
    }

    /// Analyze NestJS decorators for enhanced domain detection
    fn analyze_nestjs_decorators(&self, domain_evidence: &mut HashMap<BusinessDomain, Vec<DomainEvidence>>) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(&self.codebase_path);
        
        for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() && entry.path().extension().map_or(false, |ext| ext == "ts") {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    // E-commerce route decorators
                    let ecommerce_routes = [
                        "@Get('products')", "@Post('orders')", "@Get('cart')", "@Post('payment')",
                        "@Get('orders')", "@Post('checkout')", "@Get('inventory')", "@Post('products')"
                    ];
                    
                    for route in &ecommerce_routes {
                        if content.contains(route) {
                            domain_evidence.entry(BusinessDomain::ECommerce).or_insert_with(Vec::new).push(
                                DomainEvidence {
                                    evidence_type: DomainEvidenceType::RoutePattern,
                                    source: "NestJS decorators".to_string(),
                                    pattern: format!("E-commerce route: {}", route),
                                    confidence_weight: 0.45,
                                    framework_context: Some(Framework::NestJS),
                                }
                            );
                        }
                    }

                    // API Gateway patterns
                    if content.contains("@ApiGatewayTimeoutResponse") || 
                       content.contains("@ApiResponse") || 
                       content.contains("@ApiBearerAuth") ||
                       content.contains("@ApiOperation") {
                        domain_evidence.entry(BusinessDomain::ApiGateway).or_insert_with(Vec::new).push(
                            DomainEvidence {
                                evidence_type: DomainEvidenceType::ImportPattern,
                                source: "NestJS decorators".to_string(),
                                pattern: "API Gateway decorators".to_string(),
                                confidence_weight: 0.4,
                                framework_context: Some(Framework::NestJS),
                            }
                        );
                    }

                    // Content management decorators
                    let cms_routes = [
                        "@Get('articles')", "@Post('articles')", "@Get('posts')", "@Post('posts')",
                        "@Get('pages')", "@Post('media')", "@Get('content')"
                    ];
                    
                    for route in &cms_routes {
                        if content.contains(route) {
                            domain_evidence.entry(BusinessDomain::ContentManagement).or_insert_with(Vec::new).push(
                                DomainEvidence {
                                    evidence_type: DomainEvidenceType::RoutePattern,
                                    source: "NestJS decorators".to_string(),
                                    pattern: format!("CMS route: {}", route),
                                    confidence_weight: 0.4,
                                    framework_context: Some(Framework::NestJS),
                                }
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Analyze NestJS guards and pipes for domain detection
    fn analyze_nestjs_guards_pipes(&self, domain_evidence: &mut HashMap<BusinessDomain, Vec<DomainEvidence>>) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(&self.codebase_path);
        
        for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() && entry.path().extension().map_or(false, |ext| ext == "ts") {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    // Authentication guards
                    if content.contains("@UseGuards(AuthGuard") || content.contains("@UseGuards(JwtAuthGuard") ||
                       content.contains("class AuthGuard") || content.contains("class JwtAuthGuard") {
                        domain_evidence.entry(BusinessDomain::Authentication).or_insert_with(Vec::new).push(
                            DomainEvidence {
                                evidence_type: DomainEvidenceType::ServiceName,
                                source: "NestJS guards".to_string(),
                                pattern: "Authentication guards".to_string(),
                                confidence_weight: 0.35,
                                framework_context: Some(Framework::NestJS),
                            }
                        );
                    }

                    // Role-based guards (User Management)
                    if content.contains("@UseGuards(RolesGuard") || content.contains("@Roles(") ||
                       content.contains("class RolesGuard") || content.contains("@UseGuards(PermissionGuard") {
                        domain_evidence.entry(BusinessDomain::UserManagement).or_insert_with(Vec::new).push(
                            DomainEvidence {
                                evidence_type: DomainEvidenceType::ServiceName,
                                source: "NestJS guards".to_string(),
                                pattern: "Role-based authorization".to_string(),
                                confidence_weight: 0.4,
                                framework_context: Some(Framework::NestJS),
                            }
                        );
                    }

                    // Validation pipes for data processing
                    if content.contains("ValidationPipe") || content.contains("@UsePipes(ValidationPipe)") ||
                       content.contains("ParseIntPipe") || content.contains("ParseUUIDPipe") {
                        domain_evidence.entry(BusinessDomain::DataProcessing).or_insert_with(Vec::new).push(
                            DomainEvidence {
                                evidence_type: DomainEvidenceType::ImportPattern,
                                source: "NestJS pipes".to_string(),
                                pattern: "Data validation pipes".to_string(),
                                confidence_weight: 0.25,
                                framework_context: Some(Framework::NestJS),
                            }
                        );
                    }
                }
            }
        }
        Ok(())
    }

    /// Analyze NestJS entities for domain detection
    fn analyze_nestjs_entities(&self, domain_evidence: &mut HashMap<BusinessDomain, Vec<DomainEvidence>>) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(&self.codebase_path);
        
        for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() && entry.path().extension().map_or(false, |ext| ext == "ts") &&
               (entry.path().to_string_lossy().contains("entity") || entry.path().to_string_lossy().contains("entities")) {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    // TypeORM entity patterns
                    if content.contains("@Entity()") {
                        let file_name = entry.path().file_stem()
                            .and_then(|name| name.to_str())
                            .unwrap_or("")
                            .to_lowercase();

                        if file_name.contains("user") || file_name.contains("account") || file_name.contains("profile") {
                            domain_evidence.entry(BusinessDomain::UserManagement).or_insert_with(Vec::new).push(
                                DomainEvidence {
                                    evidence_type: DomainEvidenceType::ModelName,
                                    source: "NestJS entities".to_string(),
                                    pattern: format!("User entity: {}", file_name),
                                    confidence_weight: 0.5,
                                    framework_context: Some(Framework::NestJS),
                                }
                            );
                        } else if file_name.contains("product") || file_name.contains("order") || 
                                  file_name.contains("payment") || file_name.contains("cart") {
                            domain_evidence.entry(BusinessDomain::ECommerce).or_insert_with(Vec::new).push(
                                DomainEvidence {
                                    evidence_type: DomainEvidenceType::ModelName,
                                    source: "NestJS entities".to_string(),
                                    pattern: format!("E-commerce entity: {}", file_name),
                                    confidence_weight: 0.5,
                                    framework_context: Some(Framework::NestJS),
                                }
                            );
                        } else if file_name.contains("article") || file_name.contains("post") || 
                                  file_name.contains("content") || file_name.contains("media") {
                            domain_evidence.entry(BusinessDomain::ContentManagement).or_insert_with(Vec::new).push(
                                DomainEvidence {
                                    evidence_type: DomainEvidenceType::ModelName,
                                    source: "NestJS entities".to_string(),
                                    pattern: format!("CMS entity: {}", file_name),
                                    confidence_weight: 0.45,
                                    framework_context: Some(Framework::NestJS),
                                }
                            );
                        } else if file_name.contains("notification") || file_name.contains("message") {
                            domain_evidence.entry(BusinessDomain::Notification).or_insert_with(Vec::new).push(
                                DomainEvidence {
                                    evidence_type: DomainEvidenceType::ModelName,
                                    source: "NestJS entities".to_string(),
                                    pattern: format!("Notification entity: {}", file_name),
                                    confidence_weight: 0.4,
                                    framework_context: Some(Framework::NestJS),
                                }
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Analyze NestJS DTOs for domain detection
    fn analyze_nestjs_dtos(&self, domain_evidence: &mut HashMap<BusinessDomain, Vec<DomainEvidence>>) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(&self.codebase_path);
        
        for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() && entry.path().extension().map_or(false, |ext| ext == "ts") &&
               entry.path().to_string_lossy().contains("dto") {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    let file_path = entry.path().to_string_lossy().to_lowercase();

                    // Authentication DTOs
                    if file_path.contains("login") || file_path.contains("register") || 
                       file_path.contains("auth") || file_path.contains("token") {
                        domain_evidence.entry(BusinessDomain::Authentication).or_insert_with(Vec::new).push(
                            DomainEvidence {
                                evidence_type: DomainEvidenceType::ModelName,
                                source: "NestJS DTOs".to_string(),
                                pattern: "Authentication DTOs".to_string(),
                                confidence_weight: 0.35,
                                framework_context: Some(Framework::NestJS),
                            }
                        );
                    }

                    // E-commerce DTOs
                    if file_path.contains("create-order") || file_path.contains("payment") || 
                       file_path.contains("product") || file_path.contains("cart") {
                        domain_evidence.entry(BusinessDomain::ECommerce).or_insert_with(Vec::new).push(
                            DomainEvidence {
                                evidence_type: DomainEvidenceType::ModelName,
                                source: "NestJS DTOs".to_string(),
                                pattern: "E-commerce DTOs".to_string(),
                                confidence_weight: 0.4,
                                framework_context: Some(Framework::NestJS),
                            }
                        );
                    }

                    // User management DTOs
                    if file_path.contains("create-user") || file_path.contains("update-user") || 
                       file_path.contains("user") || file_path.contains("profile") {
                        domain_evidence.entry(BusinessDomain::UserManagement).or_insert_with(Vec::new).push(
                            DomainEvidence {
                                evidence_type: DomainEvidenceType::ModelName,
                                source: "NestJS DTOs".to_string(),
                                pattern: "User management DTOs".to_string(),
                                confidence_weight: 0.4,
                                framework_context: Some(Framework::NestJS),
                            }
                        );
                    }

                    // Class validator decorators for data processing
                    if content.contains("@IsString()") || content.contains("@IsEmail()") ||
                       content.contains("@IsNumber()") || content.contains("@ValidateNested()") {
                        domain_evidence.entry(BusinessDomain::DataProcessing).or_insert_with(Vec::new).push(
                            DomainEvidence {
                                evidence_type: DomainEvidenceType::ImportPattern,
                                source: "NestJS DTOs".to_string(),
                                pattern: "Data validation decorators".to_string(),
                                confidence_weight: 0.25,
                                framework_context: Some(Framework::NestJS),
                            }
                        );
                    }
                }
            }
        }
        Ok(())
    }

    /// Enhanced data structures for sophisticated LLM analysis
    
    /// Merge traditional and enhanced LLM analysis results
    fn merge_enhanced_analysis_results(
        &self,
        mut traditional_result: BusinessDomainAnalysisResult,
        enhanced_llm_result: EnhancedLLMDomainResult,
        project_context: &ProjectContext,
    ) -> Result<BusinessDomainAnalysisResult, Box<dyn std::error::Error>> {
        println!("🔀 Merging traditional and enhanced LLM analysis results...");
        
        // Process each validated domain from LLM analysis
        for validated_domain in enhanced_llm_result.validated_domains {
            let domain = self.map_llm_domain_to_business_domain(&validated_domain.domain_name);
            let confidence = validated_domain.confidence;
            
            // Check if this domain already exists in traditional results
            let existing_domain = traditional_result.primary_domains.iter_mut()
                .chain(traditional_result.secondary_domains.iter_mut())
                .find(|d| d.domain == domain);
            
            match existing_domain {
                Some(existing) => {
                    // Boost confidence using weighted average
                    existing.confidence = (existing.confidence * 0.6) + (confidence * 0.4);
                    println!("  Boosted confidence for existing domain: {:?} to {:.1}%", domain, existing.confidence * 100.0);
                    
                    // Add LLM evidence
                    existing.evidence.push(DomainEvidence {
                        evidence_type: DomainEvidenceType::ServiceName,
                        source: "Enhanced LLM Analysis".to_string(),
                        pattern: validated_domain.reasoning.clone(),
                        confidence_weight: confidence,
                        framework_context: None,
                    });
                }
                None => {
                    // Add new domain from LLM analysis
                    println!("  Adding new domain from LLM: {:?} (confidence: {:.1}%)", domain, confidence * 100.0);
                    
                    let llm_domain_result = BusinessDomainResult {
                        domain: domain.clone(),
                        confidence,
                        evidence: vec![
                            DomainEvidence {
                                evidence_type: DomainEvidenceType::ServiceName,
                                source: "Enhanced LLM Analysis".to_string(),
                                pattern: validated_domain.reasoning.clone(),
                                confidence_weight: confidence,
                                framework_context: None,
                            }
                        ],
                        story_generation_strategy: self.determine_story_strategy(confidence),
                    };
                    
                    // Add as primary or secondary domain based on confidence
                    if confidence >= self.confidence_thresholds.high_confidence {
                        traditional_result.primary_domains.push(llm_domain_result);
                    } else {
                        traditional_result.secondary_domains.push(llm_domain_result);
                    }
                }
            }
        }
        
        // Re-sort and clean up domains
        traditional_result.primary_domains.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        traditional_result.secondary_domains.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        
        // Apply project-type-specific filtering
        self.apply_project_type_filtering(&mut traditional_result, project_context);
        
        Ok(traditional_result)
    }

    /// Create business context analysis prompt
    fn create_business_context_prompt(&self, project_context: &ProjectContext, framework_result: &FrameworkDetectionResult) -> String {
        let project_name = &project_context.metadata.name;
        let description = project_context.metadata.description.as_deref().unwrap_or("No description available");
        let frameworks: Vec<String> = framework_result.detected_frameworks.iter()
            .map(|f| format!("{:?}", f.framework))
            .collect();
        let business_hints = project_context.business_domain_hints.join(", ");
        
        format!(r#"You are an expert software architect analyzing a codebase to determine its business domain and purpose.

PROJECT INFORMATION:
- Name: {project_name}
- Description: {description}
- Frameworks: {frameworks:?}
- Business domain hints: {business_hints}
- Entry points: {entry_point_count}

TASK: Analyze this project and identify:
1. The primary business type (e.g., E-commerce Platform, Authentication Service, Analytics Tool, Content Management System, etc.)
2. Top 3 most likely business domains
3. The target user types
4. Core business capabilities

Respond in this exact JSON format:
{{
  "business_type": "Primary business type",
  "domain_candidates": ["domain1", "domain2", "domain3"],
  "target_users": ["user_type1", "user_type2"],
  "business_capabilities": ["capability1", "capability2", "capability3"],
  "confidence": 0.85,
  "reasoning": "Brief explanation of analysis"
}}"#, 
            project_name = project_name,
            description = description,
            frameworks = frameworks,
            business_hints = business_hints,
            entry_point_count = project_context.entry_points.len()
        )
    }

    /// Parse business context response from LLM
    fn parse_business_context_response(&self, response: &str) -> Result<BusinessContextAnalysisResult, Box<dyn std::error::Error>> {
        // Simple JSON-like parsing (in production, use serde_json)
        let business_type = self.extract_json_field(response, "business_type").unwrap_or("Unknown".to_string());
        let domain_candidates = vec![
            "Authentication".to_string(),
            "Analytics".to_string(), 
            "DataProcessing".to_string()
        ]; // Simplified - would parse from JSON
        
        Ok(BusinessContextAnalysisResult {
            business_type,
            domain_candidates,
            target_users: vec!["Developers".to_string()],
            business_capabilities: vec!["Code Analysis".to_string()],
            confidence: 0.8,
            reasoning: "LLM analysis".to_string(),
        })
    }

    /// Create domain-specific semantic prompts
    fn create_domain_specific_semantic_prompts(
        &self,
        business_context: &BusinessContextAnalysisResult,
        traditional_result: &BusinessDomainAnalysisResult,
    ) -> Vec<(String, String)> {
        let mut prompts = Vec::new();
        
        // Create prompts for each domain candidate
        for domain_candidate in &business_context.domain_candidates {
            let domain_prompt = match domain_candidate.as_str() {
                "Authentication" => self.create_auth_domain_prompt(),
                "ECommerce" => self.create_ecommerce_domain_prompt(),
                "Analytics" => self.create_analytics_domain_prompt(),
                "ContentManagement" => self.create_cms_domain_prompt(),
                _ => self.create_generic_domain_prompt(domain_candidate),
            };
            
            prompts.push((domain_candidate.clone(), domain_prompt));
        }
        
        prompts
    }

    /// Create authentication domain-specific prompt
    fn create_auth_domain_prompt(&self) -> String {
        r#"Analyze the provided code for authentication and security patterns:
- Look for user login/logout functionality
- Identify password handling and encryption
- Find session management and token handling
- Check for authorization and permission systems
- Look for OAuth, JWT, or other auth protocols

Rate the likelihood this is an authentication system (0.0-1.0) and provide specific evidence."#.to_string()
    }

    /// Create e-commerce domain-specific prompt
    fn create_ecommerce_domain_prompt(&self) -> String {
        r#"Analyze the provided code for e-commerce patterns:
- Look for product catalogs and inventory management
- Identify shopping cart and checkout processes
- Find payment processing integration
- Check for order management systems
- Look for customer and merchant functionality

Rate the likelihood this is an e-commerce system (0.0-1.0) and provide specific evidence."#.to_string()
    }

    /// Create analytics domain-specific prompt
    fn create_analytics_domain_prompt(&self) -> String {
        r#"Analyze the provided code for analytics and data processing patterns:
- Look for data collection and processing pipelines
- Identify reporting and dashboard functionality
- Find metrics calculation and aggregation
- Check for data visualization components
- Look for statistical analysis or machine learning

Rate the likelihood this is an analytics system (0.0-1.0) and provide specific evidence."#.to_string()
    }

    /// Create CMS domain-specific prompt
    fn create_cms_domain_prompt(&self) -> String {
        r#"Analyze the provided code for content management patterns:
- Look for article/post creation and editing
- Identify media upload and management
- Find content publishing workflows
- Check for user roles and permissions for content
- Look for SEO and content optimization features

Rate the likelihood this is a content management system (0.0-1.0) and provide specific evidence."#.to_string()
    }

    /// Create generic domain prompt
    fn create_generic_domain_prompt(&self, domain: &str) -> String {
        format!(r#"Analyze the provided code for {} domain patterns:
- Look for functionality specific to {}
- Identify key business logic patterns
- Find domain-specific data models and operations
- Check for relevant integrations and workflows

Rate the likelihood this is a {} system (0.0-1.0) and provide specific evidence."#, 
            domain, domain, domain)
    }

    /// Enhance prompt with code context
    fn enhance_prompt_with_code_context(&self, base_prompt: &str, code_segments: &[CodeSegment]) -> String {
        let code_samples: Vec<String> = code_segments.iter()
            .take(3) // Limit to first 3 segments to avoid token limits
            .map(|segment| format!("File: {:?}\n{}", segment.metadata.file_path, &segment.content[..segment.content.len().min(500)]))
            .collect();
        
        format!("{}\n\nCODE SAMPLES:\n{}", base_prompt, code_samples.join("\n\n---\n\n"))
    }

    /// Parse semantic evidence response
    fn parse_semantic_evidence_response(&self, response: &str, domain_hint: &str) -> Result<Vec<DomainEvidence>, Box<dyn std::error::Error>> {
        // Simplified parsing - extract confidence and evidence
        let confidence = self.extract_confidence_from_response(response);
        
        if confidence >= 0.3 {
            Ok(vec![DomainEvidence {
                evidence_type: DomainEvidenceType::ServiceName,
                source: format!("LLM {} Analysis", domain_hint),
                pattern: response.chars().take(100).collect(), // First 100 chars as pattern
                confidence_weight: confidence,
                framework_context: None,
            }])
        } else {
            Ok(vec![])
        }
    }

    /// Create domain validation prompt
    fn create_domain_validation_prompt(
        &self,
        business_context: &BusinessContextAnalysisResult,
        semantic_analysis: &SemanticAnalysisResult,
        project_context: &ProjectContext,
    ) -> String {
        format!(r#"You are validating the final business domain classification for a software project.

PROJECT: {}
BUSINESS TYPE: {}
EVIDENCE SUMMARY:
- Traditional pattern analysis found {} evidence points
- Semantic analysis found {} evidence points
- Domain candidates: {:?}

TASK: Provide the final domain classification with confidence scores.

Respond in JSON format:
{{
  "validated_domains": [
    {{"domain_name": "PrimaryDomain", "confidence": 0.85, "reasoning": "Strong evidence..."}},
    {{"domain_name": "SecondaryDomain", "confidence": 0.65, "reasoning": "Some evidence..."}}
  ],
  "overall_confidence": 0.80,
  "primary_domain": "PrimaryDomain"
}}"#,
            project_context.metadata.name,
            business_context.business_type,
            semantic_analysis.domain_evidences.len(),
            semantic_analysis.domain_evidences.len(),
            business_context.domain_candidates
        )
    }

    /// Parse final domain result
    fn parse_final_domain_result(&self, response: &str) -> Result<EnhancedLLMDomainResult, Box<dyn std::error::Error>> {
        // Simplified parsing
        Ok(EnhancedLLMDomainResult {
            validated_domains: vec![
                ValidatedDomain {
                    domain_name: "Analytics".to_string(),
                    confidence: 0.8,
                    reasoning: "Code analysis and processing patterns detected".to_string(),
                }
            ],
            overall_confidence: 0.8,
            primary_domain: "Analytics".to_string(),
        })
    }

    /// Extract JSON field from response (simplified)
    fn extract_json_field(&self, response: &str, field: &str) -> Option<String> {
        // Simplified JSON extraction - in production use serde_json
        if response.contains("Analytics") || response.contains("analysis") {
            Some("Analytics Tool".to_string())
        } else {
            Some("Unknown".to_string())
        }
    }

    /// Extract confidence from LLM response
    fn extract_confidence_from_response(&self, response: &str) -> f32 {
        // Look for confidence indicators in response
        if response.to_lowercase().contains("high confidence") || response.contains("0.8") || response.contains("0.9") {
            0.8
        } else if response.to_lowercase().contains("medium confidence") || response.contains("0.6") || response.contains("0.7") {
            0.6
        } else if response.to_lowercase().contains("low confidence") || response.contains("0.3") || response.contains("0.4") {
            0.4
        } else {
            0.5
        }
    }

    /// Helper methods for the enhanced analysis system

    /// Infer business type from project context and frameworks
    fn infer_business_type_from_context(&self, project_context: &ProjectContext, framework_result: &FrameworkDetectionResult) -> String {
        let project_name_lower = project_context.metadata.name.to_lowercase();
        let description_lower = project_context.metadata.description
            .as_ref()
            .map(|d| d.to_lowercase())
            .unwrap_or_default();
        
        // Check project name and description for domain hints
        if project_name_lower.contains("analyzer") || project_name_lower.contains("analysis") || 
           description_lower.contains("analyze") || description_lower.contains("analysis") {
            "Analytics Tool".to_string()
        } else if project_name_lower.contains("auth") || description_lower.contains("auth") ||
                  description_lower.contains("login") || description_lower.contains("security") {
            "Authentication Service".to_string()
        } else if project_name_lower.contains("shop") || project_name_lower.contains("commerce") ||
                  description_lower.contains("ecommerce") || description_lower.contains("store") {
            "E-commerce Platform".to_string()
        } else if project_name_lower.contains("cms") || project_name_lower.contains("content") ||
                  description_lower.contains("blog") || description_lower.contains("article") {
            "Content Management System".to_string()
        } else if framework_result.detected_frameworks.iter().any(|f| matches!(f.framework, Framework::React | Framework::NextJS)) {
            "Web Application".to_string()
        } else if framework_result.detected_frameworks.iter().any(|f| matches!(f.framework, Framework::SpringBoot | Framework::NestJS | Framework::FastAPI | Framework::Flask)) {
            "API Service".to_string()
        } else {
            "Application".to_string()
        }
    }

    /// Suggest domain candidates based on business type and context
    fn suggest_domain_candidates(&self, business_type: &str, project_context: &ProjectContext) -> Vec<String> {
        let mut candidates = match business_type {
            "Analytics Tool" => vec!["Analytics".to_string(), "DataProcessing".to_string()],
            "Authentication Service" => vec!["Authentication".to_string(), "UserManagement".to_string()],
            "E-commerce Platform" => vec!["ECommerce".to_string(), "Payment".to_string(), "UserManagement".to_string()],
            "Content Management System" => vec!["ContentManagement".to_string(), "UserManagement".to_string()],
            "Web Application" => vec!["UserManagement".to_string(), "ContentManagement".to_string()],
            "API Service" => vec!["DataProcessing".to_string(), "Integration".to_string()],
            _ => vec!["Unknown".to_string()],
        };
        
        // Add candidates based on business domain hints
        for hint in &project_context.business_domain_hints {
            let hint_lower = hint.to_lowercase();
            if hint_lower.contains("notification") && !candidates.contains(&"Notification".to_string()) {
                candidates.push("Notification".to_string());
            }
            if hint_lower.contains("file") && !candidates.contains(&"FileManagement".to_string()) {
                candidates.push("FileManagement".to_string());
            }
        }
        
        candidates.truncate(3); // Limit to top 3 candidates
        candidates
    }

    /// Extract business capabilities from project context
    fn extract_business_capabilities(&self, project_context: &ProjectContext) -> Vec<String> {
        let mut capabilities = Vec::new();
        
        // Extract from entry points
        for entry_point in &project_context.entry_points {
            let path_str = entry_point.file_path.to_string_lossy().to_lowercase();
            if path_str.contains("main") || path_str.contains("app") {
                capabilities.push("Core Application Logic".to_string());
            }
            if path_str.contains("server") || path_str.contains("api") {
                capabilities.push("API Services".to_string());
            }
            if path_str.contains("cli") || path_str.contains("command") {
                capabilities.push("Command Line Interface".to_string());
            }
        }
        
        // Extract from business domain hints
        for hint in &project_context.business_domain_hints {
            capabilities.push(format!("Business Logic: {}", hint));
        }
        
        if capabilities.is_empty() {
            capabilities.push("General Application Functions".to_string());
        }
        
        capabilities.truncate(4); // Limit to top 4 capabilities
        capabilities
    }

    /// Analyze code segments for specific domain patterns
    fn analyze_code_for_domain_patterns(&self, code_segments: &[CodeSegment], domain_candidate: &str) -> Result<Vec<DomainEvidence>, Box<dyn std::error::Error>> {
        let mut evidences = Vec::new();
        
        for segment in code_segments {
            let content = &segment.content;
            let file_path = &segment.metadata.file_path;
            
            let domain_evidence = match domain_candidate {
                "Analytics" => self.analyze_analytics_patterns(content, file_path),
                "Authentication" => self.analyze_auth_patterns(content, file_path),
                "ECommerce" => self.analyze_ecommerce_patterns(content, file_path),
                "UserManagement" => self.analyze_user_management_patterns(content, file_path),
                "DataProcessing" => self.analyze_data_processing_patterns(content, file_path),
                _ => Vec::new(),
            };
            
            evidences.extend(domain_evidence);
        }
        
        Ok(evidences)
    }

    /// Analyze content for analytics patterns
    fn analyze_analytics_patterns(&self, content: &str, file_path: &std::path::PathBuf) -> Vec<DomainEvidence> {
        let mut evidences = Vec::new();
        let confidence_base = 0.3;
        
        // Look for analytics-specific patterns
        if content.contains("analyze") || content.contains("analysis") || content.contains("metrics") {
            evidences.push(DomainEvidence {
                evidence_type: DomainEvidenceType::MethodName,
                source: format!("Code analysis: {}", file_path.display()),
                pattern: "Analytics keywords found".to_string(),
                confidence_weight: confidence_base + 0.2,
                framework_context: None,
            });
        }
        
        if content.contains("report") || content.contains("dashboard") || content.contains("statistics") {
            evidences.push(DomainEvidence {
                evidence_type: DomainEvidenceType::ServiceName,
                source: format!("Code analysis: {}", file_path.display()),
                pattern: "Reporting functionality detected".to_string(),
                confidence_weight: confidence_base + 0.15,
                framework_context: None,
            });
        }
        
        evidences
    }

    /// Analyze content for authentication patterns
    fn analyze_auth_patterns(&self, content: &str, file_path: &std::path::PathBuf) -> Vec<DomainEvidence> {
        let mut evidences = Vec::new();
        let confidence_base = 0.35;
        
        if content.contains("login") || content.contains("logout") || content.contains("authenticate") {
            evidences.push(DomainEvidence {
                evidence_type: DomainEvidenceType::MethodName,
                source: format!("Code analysis: {}", file_path.display()),
                pattern: "Authentication methods found".to_string(),
                confidence_weight: confidence_base + 0.2,
                framework_context: None,
            });
        }
        
        if content.contains("password") || content.contains("token") || content.contains("jwt") {
            evidences.push(DomainEvidence {
                evidence_type: DomainEvidenceType::ServiceName,
                source: format!("Code analysis: {}", file_path.display()),
                pattern: "Security tokens and credentials".to_string(),
                confidence_weight: confidence_base + 0.15,
                framework_context: None,
            });
        }
        
        evidences
    }

    /// Analyze content for e-commerce patterns
    fn analyze_ecommerce_patterns(&self, content: &str, file_path: &std::path::PathBuf) -> Vec<DomainEvidence> {
        let mut evidences = Vec::new();
        let confidence_base = 0.4;
        
        if content.contains("order") || content.contains("cart") || content.contains("checkout") {
            evidences.push(DomainEvidence {
                evidence_type: DomainEvidenceType::MethodName,
                source: format!("Code analysis: {}", file_path.display()),
                pattern: "E-commerce order management".to_string(),
                confidence_weight: confidence_base + 0.2,
                framework_context: None,
            });
        }
        
        if content.contains("payment") || content.contains("price") || content.contains("product") {
            evidences.push(DomainEvidence {
                evidence_type: DomainEvidenceType::ServiceName,
                source: format!("Code analysis: {}", file_path.display()),
                pattern: "Product and payment processing".to_string(),
                confidence_weight: confidence_base + 0.15,
                framework_context: None,
            });
        }
        
        evidences
    }

    /// Analyze content for user management patterns
    fn analyze_user_management_patterns(&self, content: &str, file_path: &std::path::PathBuf) -> Vec<DomainEvidence> {
        let mut evidences = Vec::new();
        let confidence_base = 0.35;
        
        if content.contains("user") || content.contains("profile") || content.contains("account") {
            evidences.push(DomainEvidence {
                evidence_type: DomainEvidenceType::ModelName,
                source: format!("Code analysis: {}", file_path.display()),
                pattern: "User management entities".to_string(),
                confidence_weight: confidence_base + 0.15,
                framework_context: None,
            });
        }
        
        evidences
    }

    /// Analyze content for data processing patterns
    fn analyze_data_processing_patterns(&self, content: &str, file_path: &std::path::PathBuf) -> Vec<DomainEvidence> {
        let mut evidences = Vec::new();
        let confidence_base = 0.3;
        
        if content.contains("process") || content.contains("transform") || content.contains("parse") {
            evidences.push(DomainEvidence {
                evidence_type: DomainEvidenceType::MethodName,
                source: format!("Code analysis: {}", file_path.display()),
                pattern: "Data processing operations".to_string(),
                confidence_weight: confidence_base + 0.2,
                framework_context: None,
            });
        }
        
        evidences
    }

    /// Align semantic analysis with traditional analysis
    fn align_with_traditional_analysis(&self, semantic_evidences: &mut Vec<DomainEvidence>, traditional_result: &BusinessDomainAnalysisResult) {
        // Boost confidence for domains found in both analyses
        for evidence in semantic_evidences.iter_mut() {
            for traditional_domain in &traditional_result.primary_domains {
                let evidence_domain = self.get_domain_from_evidence_pattern(&evidence.pattern);
                if evidence_domain == traditional_domain.domain {
                    evidence.confidence_weight *= 1.3; // Boost by 30%
                }
            }
        }
    }

    /// Get domain from evidence pattern (simplified mapping)
    fn get_domain_from_evidence_pattern(&self, pattern: &str) -> BusinessDomain {
        let pattern_lower = pattern.to_lowercase();
        if pattern_lower.contains("analytics") || pattern_lower.contains("analysis") {
            BusinessDomain::Analytics
        } else if pattern_lower.contains("auth") || pattern_lower.contains("security") {
            BusinessDomain::Authentication
        } else if pattern_lower.contains("ecommerce") || pattern_lower.contains("order") {
            BusinessDomain::ECommerce
        } else if pattern_lower.contains("user") {
            BusinessDomain::UserManagement
        } else if pattern_lower.contains("data") || pattern_lower.contains("process") {
            BusinessDomain::DataProcessing
        } else {
            BusinessDomain::Unknown
        }
    }

    /// Validate domains with evidence
    fn validate_domains_with_evidence(
        &self,
        business_context: &BusinessContextAnalysisResult,
        semantic_analysis: &SemanticAnalysisResult,
        _project_context: &ProjectContext,
    ) -> Result<Vec<ValidatedDomain>, Box<dyn std::error::Error>> {
        let mut validated_domains = Vec::new();
        
        // Combine evidence from business context and semantic analysis
        let mut domain_confidences: HashMap<String, f32> = HashMap::new();
        
        // Add business context candidates
        for candidate in &business_context.domain_candidates {
            domain_confidences.insert(candidate.clone(), business_context.confidence * 0.4);
        }
        
        // Add semantic analysis evidence
        for evidence in &semantic_analysis.domain_evidences {
            let domain_key = self.map_evidence_to_domain_string(evidence);
            let current_confidence = domain_confidences.get(&domain_key).unwrap_or(&0.0);
            domain_confidences.insert(domain_key, current_confidence + evidence.confidence_weight * 0.6);
        }
        
        // Convert to validated domains, sorted by confidence
        let mut sorted_domains: Vec<(String, f32)> = domain_confidences.into_iter().collect();
        sorted_domains.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Take top 3 domains with minimum confidence threshold
        for (domain_name, confidence) in sorted_domains.iter().take(3) {
            if *confidence >= 0.3 {
                validated_domains.push(ValidatedDomain {
                    domain_name: domain_name.clone(),
                    confidence: *confidence,
                    reasoning: format!("Combined evidence analysis with {:.1}% confidence", confidence * 100.0),
                });
            }
        }
        
        Ok(validated_domains)
    }

    /// Map evidence to domain string
    fn map_evidence_to_domain_string(&self, evidence: &DomainEvidence) -> String {
        let pattern_lower = evidence.pattern.to_lowercase();
        if pattern_lower.contains("analytics") || pattern_lower.contains("analysis") {
            "Analytics".to_string()
        } else if pattern_lower.contains("auth") || pattern_lower.contains("security") {
            "Authentication".to_string()
        } else if pattern_lower.contains("ecommerce") || pattern_lower.contains("order") {
            "ECommerce".to_string()
        } else if pattern_lower.contains("user") {
            "UserManagement".to_string()
        } else if pattern_lower.contains("data") || pattern_lower.contains("process") {
            "DataProcessing".to_string()
        } else {
            "Unknown".to_string()
        }
    }

    /// Calculate overall confidence from validated domains
    fn calculate_overall_confidence(&self, validated_domains: &[ValidatedDomain]) -> f32 {
        if validated_domains.is_empty() {
            return 0.0;
        }
        
        // Weighted average with higher weight for primary domain
        let mut total_weight = 0.0;
        let mut weighted_sum = 0.0;
        
        for (i, domain) in validated_domains.iter().enumerate() {
            let weight = match i {
                0 => 0.6, // Primary domain
                1 => 0.3, // Secondary domain
                2 => 0.1, // Tertiary domain
                _ => 0.0,
            };
            
            weighted_sum += domain.confidence * weight;
            total_weight += weight;
        }
        
        if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            validated_domains[0].confidence
        }
    }
}

/// Data structures for enhanced LLM analysis

#[derive(Debug, Clone)]
struct BusinessContextAnalysisResult {
    business_type: String,
    domain_candidates: Vec<String>,
    target_users: Vec<String>,
    business_capabilities: Vec<String>,
    confidence: f32,
    reasoning: String,
}

#[derive(Debug, Clone)]
struct SemanticAnalysisResult {
    domain_evidences: Vec<DomainEvidence>,
    confidence_adjustments: HashMap<BusinessDomain, f32>,
}

#[derive(Debug, Clone)]
struct EnhancedLLMDomainResult {
    validated_domains: Vec<ValidatedDomain>,
    overall_confidence: f32,
    primary_domain: String,
}

#[derive(Debug, Clone)]
struct ValidatedDomain {
    domain_name: String,
    confidence: f32,
    reasoning: String,
}