use std::collections::HashMap;
use anyhow::Result;
use serde::{Serialize, Deserialize};

use crate::core::context_types::{ProjectContext, EnhancedSegmentContext};
use crate::core::types::BusinessDomain;
use crate::core::config::Config;
use crate::intelligence::llm_client::LocalLLMManager;

#[derive(Debug, Clone)]
pub struct BusinessContextGroundingEngine {
    domain_templates: DomainPromptTemplateEngine,
    validation_engine: BusinessValidationEngine,
    evidence_collector: ContextEvidenceCollector,
    llm_manager: Option<LocalLLMManager>,
    config: Config,
}

impl BusinessContextGroundingEngine {
    pub async fn new() -> Result<Self> {
        let domain_templates = DomainPromptTemplateEngine::new()?;
        let validation_engine = BusinessValidationEngine::new()?;
        let evidence_collector = ContextEvidenceCollector::new();
        
        let llm_manager = match LocalLLMManager::new(None).await {
            Ok(manager) => Some(manager),
            Err(_) => None,
        };

        Ok(Self {
            domain_templates,
            validation_engine,
            evidence_collector,
            llm_manager,
            config: Config::instance(),
        })
    }

    pub async fn ground_business_context(
        &mut self,
        project_context: &ProjectContext,
        enhanced_segments: &[EnhancedSegmentContext]
    ) -> Result<BusinessContextGroundingResult> {
        let start_time = std::time::Instant::now();

        println!("  - Starting business context grounding");

        // Step 1: Collect evidence from segments and project context
        println!("    - Collecting contextual evidence");
        let evidence = self.evidence_collector
            .collect_business_evidence(project_context, enhanced_segments)?;

        // Step 2: Generate domain-specific prompts
        println!("    - Generating domain-specific analysis prompts");
        let domain_prompts = self.domain_templates
            .generate_domain_prompts(&evidence)?;

        // Step 3: Perform LLM-enhanced business domain analysis
        println!("    - Performing enhanced domain analysis");
        let domain_analysis = if self.llm_manager.is_some() {
            // For now, use fallback since LLM integration needs more work
            self.perform_fallback_domain_analysis(&evidence)?
        } else {
            self.perform_fallback_domain_analysis(&evidence)?
        };

        // Step 4: Validate business context accuracy
        // Validating business context accuracy
        let validation_result = self.validation_engine
            .validate_business_context(&domain_analysis, &evidence)?;

        // Step 5: Ground final business context
        // Grounding final business context
        let grounded_context = self.ground_final_context(
            &domain_analysis,
            &validation_result,
            &evidence
        )?;

        let grounding_time = start_time.elapsed();
        let evidence_summary = evidence.create_summary();
        let evidence_quality = evidence.calculate_quality_score();
        let validation_confidence = validation_result.overall_confidence;
        let domain_coverage = self.calculate_domain_coverage(&grounded_context);

        Ok(BusinessContextGroundingResult {
            grounded_context,
            domain_analysis,
            validation_result,
            evidence_summary,
            performance_metrics: GroundingPerformanceMetrics {
                total_grounding_time_ms: grounding_time.as_millis() as u64,
                evidence_collection_quality: evidence_quality,
                validation_confidence,
                domain_coverage_completeness: domain_coverage,
            },
        })
    }

    async fn perform_llm_domain_analysis(
        &self,
        llm: &mut LocalLLMManager,
        domain_prompts: &DomainSpecificPrompts,
        evidence: &BusinessEvidence
    ) -> Result<EnhancedDomainAnalysis> {
        let mut domain_insights = HashMap::new();
        let mut confidence_scores = HashMap::new();

        // Analyze each identified business domain
        for domain in &evidence.potential_domains {
            if let Some(prompt) = domain_prompts.get_domain_prompt(&domain.name) {
                // For now, use fallback analysis since we need to implement proper LLM integration
                let insight = self.analyze_domain_with_fallback(domain, evidence)?;
                domain_insights.insert(domain.name.clone(), insight);
                confidence_scores.insert(domain.name.clone(), domain.confidence);
            }
        }

        // Perform cross-domain validation
        let cross_domain_analysis = self.perform_cross_domain_analysis(&domain_insights, evidence)?;
        let analysis_quality = self.calculate_analysis_quality(&domain_insights);

        Ok(EnhancedDomainAnalysis {
            domain_insights,
            confidence_scores,
            cross_domain_analysis,
            llm_enhanced: true,
            analysis_quality,
        })
    }

    fn perform_fallback_domain_analysis(
        &self,
        evidence: &BusinessEvidence
    ) -> Result<EnhancedDomainAnalysis> {
        let mut domain_insights = HashMap::new();
        let mut confidence_scores = HashMap::new();

        for domain in &evidence.potential_domains {
            let insight = self.analyze_domain_with_fallback(domain, evidence)?;
            domain_insights.insert(domain.name.clone(), insight);
            confidence_scores.insert(domain.name.clone(), domain.confidence);
        }

        let cross_domain_analysis = self.perform_cross_domain_analysis(&domain_insights, evidence)?;
        let analysis_quality = self.calculate_analysis_quality(&domain_insights);

        Ok(EnhancedDomainAnalysis {
            domain_insights,
            confidence_scores,
            cross_domain_analysis,
            llm_enhanced: false,
            analysis_quality,
        })
    }

    fn analyze_domain_with_fallback(
        &self,
        domain: &BusinessDomain,
        evidence: &BusinessEvidence
    ) -> Result<DomainInsight> {
        // Enhanced pattern-based analysis for business domain insights
        let mut patterns_found = Vec::new();
        let mut business_capabilities = Vec::new();
        let mut user_interactions = Vec::new();

        // Analyze domain-specific patterns
        match domain.name.as_str() {
            "E-commerce" | "E-Commerce" => {
                patterns_found.extend(self.detect_ecommerce_patterns(evidence));
                business_capabilities.extend(vec![
                    "Product Catalog Management".to_string(),
                    "Shopping Cart Operations".to_string(),
                    "Payment Processing".to_string(),
                    "Order Management".to_string(),
                ]);
            },
            "User Management" | "Authentication" => {
                patterns_found.extend(self.detect_auth_patterns(evidence));
                business_capabilities.extend(vec![
                    "User Registration".to_string(),
                    "Authentication & Authorization".to_string(),
                    "Profile Management".to_string(),
                    "Session Management".to_string(),
                ]);
            },
            "Content Management" | "CMS" => {
                patterns_found.extend(self.detect_cms_patterns(evidence));
                business_capabilities.extend(vec![
                    "Content Creation & Editing".to_string(),
                    "Content Publishing".to_string(),
                    "Media Management".to_string(),
                    "Content Workflow".to_string(),
                ]);
            },
            "Analytics" | "Data Analysis" => {
                patterns_found.extend(self.detect_analytics_patterns(evidence));
                business_capabilities.extend(vec![
                    "Data Collection & Processing".to_string(),
                    "Metrics & KPI Tracking".to_string(),
                    "Report Generation".to_string(),
                    "Data Visualization".to_string(),
                ]);
            },
            "API" | "Integration" => {
                patterns_found.extend(self.detect_api_patterns(evidence));
                business_capabilities.extend(vec![
                    "External System Integration".to_string(),
                    "Data Synchronization".to_string(),
                    "Service Orchestration".to_string(),
                    "API Gateway Operations".to_string(),
                ]);
            },
            _ => {
                // Generic domain analysis
                patterns_found.extend(self.detect_generic_patterns(evidence, &domain.name));
                business_capabilities.push(format!("{} Operations", domain.name));
            }
        }

        Ok(DomainInsight {
            domain_name: domain.name.clone(),
            confidence: domain.confidence,
            patterns_found,
            business_capabilities,
            user_interactions,
            implementation_completeness: self.assess_implementation_completeness(evidence, &domain.name),
            business_value_score: self.calculate_business_value_score(evidence, &domain.name),
        })
    }

    fn detect_ecommerce_patterns(&self, evidence: &BusinessEvidence) -> Vec<String> {
        let mut patterns = Vec::new();
        let search_terms = [
            "cart", "checkout", "payment", "order", "product", "inventory",
            "shipping", "billing", "customer", "purchase", "transaction"
        ];

        for term in &search_terms {
            if evidence.code_patterns.iter().any(|p| p.to_lowercase().contains(term)) ||
               evidence.file_names.iter().any(|f| f.to_lowercase().contains(term)) {
                patterns.push(format!("E-commerce: {} management detected", term));
            }
        }
        patterns
    }

    fn detect_auth_patterns(&self, evidence: &BusinessEvidence) -> Vec<String> {
        let mut patterns = Vec::new();
        let search_terms = [
            "auth", "login", "register", "user", "token", "session",
            "password", "jwt", "oauth", "permission", "role", "security"
        ];

        for term in &search_terms {
            if evidence.code_patterns.iter().any(|p| p.to_lowercase().contains(term)) ||
               evidence.file_names.iter().any(|f| f.to_lowercase().contains(term)) {
                patterns.push(format!("Authentication: {} system detected", term));
            }
        }
        patterns
    }

    fn detect_cms_patterns(&self, evidence: &BusinessEvidence) -> Vec<String> {
        let mut patterns = Vec::new();
        let search_terms = [
            "content", "post", "article", "page", "blog", "cms",
            "editor", "publish", "draft", "media", "upload", "gallery"
        ];

        for term in &search_terms {
            if evidence.code_patterns.iter().any(|p| p.to_lowercase().contains(term)) ||
               evidence.file_names.iter().any(|f| f.to_lowercase().contains(term)) {
                patterns.push(format!("CMS: {} functionality detected", term));
            }
        }
        patterns
    }

    fn detect_analytics_patterns(&self, evidence: &BusinessEvidence) -> Vec<String> {
        let mut patterns = Vec::new();
        let search_terms = [
            "analytics", "metrics", "tracking", "report", "dashboard",
            "chart", "graph", "data", "statistics", "kpi", "insight"
        ];

        for term in &search_terms {
            if evidence.code_patterns.iter().any(|p| p.to_lowercase().contains(term)) ||
               evidence.file_names.iter().any(|f| f.to_lowercase().contains(term)) {
                patterns.push(format!("Analytics: {} capability detected", term));
            }
        }
        patterns
    }

    fn detect_api_patterns(&self, evidence: &BusinessEvidence) -> Vec<String> {
        let mut patterns = Vec::new();
        let search_terms = [
            "api", "endpoint", "route", "controller", "service", "integration",
            "webhook", "rest", "graphql", "microservice", "gateway"
        ];

        for term in &search_terms {
            if evidence.code_patterns.iter().any(|p| p.to_lowercase().contains(term)) ||
               evidence.file_names.iter().any(|f| f.to_lowercase().contains(term)) {
                patterns.push(format!("API: {} pattern detected", term));
            }
        }
        patterns
    }

    fn detect_generic_patterns(&self, evidence: &BusinessEvidence, domain_name: &str) -> Vec<String> {
        let mut patterns = Vec::new();
        let domain_lower = domain_name.to_lowercase();
        
        if evidence.code_patterns.iter().any(|p| p.to_lowercase().contains(&domain_lower)) ||
           evidence.file_names.iter().any(|f| f.to_lowercase().contains(&domain_lower)) {
            patterns.push(format!("{}: domain-specific patterns detected", domain_name));
        }

        patterns
    }

    fn perform_cross_domain_analysis(
        &self,
        domain_insights: &HashMap<String, DomainInsight>,
        evidence: &BusinessEvidence
    ) -> Result<CrossDomainAnalysis> {
        let mut domain_relationships = Vec::new();
        let mut integration_points = Vec::new();
        let mut conflicting_domains = Vec::new();

        // Analyze relationships between domains
        let domains: Vec<_> = domain_insights.keys().collect();
        for (i, domain_a) in domains.iter().enumerate() {
            for domain_b in domains.iter().skip(i + 1) {
                if let Some(relationship) = self.analyze_domain_relationship(
                    domain_a, domain_b, domain_insights, evidence
                ) {
                    domain_relationships.push(relationship);
                }
            }
        }

        // Identify integration points
        integration_points = self.identify_integration_points(domain_insights, evidence);

        // Detect conflicting domains
        conflicting_domains = self.detect_conflicting_domains(domain_insights);

        Ok(CrossDomainAnalysis {
            domain_relationships: domain_relationships.clone(),
            integration_points,
            conflicting_domains,
            overall_coherence_score: self.calculate_coherence_score(&domain_relationships),
        })
    }

    fn analyze_domain_relationship(
        &self,
        domain_a: &str,
        domain_b: &str,
        domain_insights: &HashMap<String, DomainInsight>,
        _evidence: &BusinessEvidence
    ) -> Option<DomainRelationship> {
        // Check for common business capability patterns that indicate relationships
        let insight_a = domain_insights.get(domain_a)?;
        let insight_b = domain_insights.get(domain_b)?;

        let common_patterns = insight_a.patterns_found.iter()
            .filter(|p| insight_b.patterns_found.iter().any(|q| self.patterns_related(p, q)))
            .count();

        if common_patterns > 0 {
            let relationship_strength = (common_patterns as f32 / 
                (insight_a.patterns_found.len() + insight_b.patterns_found.len()) as f32).min(1.0);
            
            Some(DomainRelationship {
                domain_a: domain_a.to_string(),
                domain_b: domain_b.to_string(),
                relationship_type: self.determine_relationship_type(domain_a, domain_b),
                strength: relationship_strength,
                integration_points: vec![format!("Shared patterns between {} and {}", domain_a, domain_b)],
            })
        } else {
            None
        }
    }

    fn patterns_related(&self, pattern_a: &str, pattern_b: &str) -> bool {
        // Simple heuristic for pattern relatedness
        let words_a: Vec<&str> = pattern_a.split_whitespace().collect();
        let words_b: Vec<&str> = pattern_b.split_whitespace().collect();
        
        words_a.iter().any(|w| words_b.contains(w))
    }

    fn determine_relationship_type(&self, domain_a: &str, domain_b: &str) -> String {
        match (domain_a, domain_b) {
            ("User Management", "E-commerce") | ("E-commerce", "User Management") => "Authentication Integration".to_string(),
            ("Analytics", _) | (_, "Analytics") => "Data Flow".to_string(),
            ("API", _) | (_, "API") => "Service Integration".to_string(),
            _ => "Functional Dependency".to_string(),
        }
    }

    fn identify_integration_points(
        &self,
        domain_insights: &HashMap<String, DomainInsight>,
        _evidence: &BusinessEvidence
    ) -> Vec<String> {
        let mut integration_points = Vec::new();
        
        for (domain, insight) in domain_insights {
            if insight.patterns_found.iter().any(|p| p.contains("api") || p.contains("service")) {
                integration_points.push(format!("{}: Service integration capabilities", domain));
            }
            if insight.patterns_found.iter().any(|p| p.contains("auth") || p.contains("user")) {
                integration_points.push(format!("{}: User context integration", domain));
            }
        }

        integration_points
    }

    fn detect_conflicting_domains(&self, domain_insights: &HashMap<String, DomainInsight>) -> Vec<String> {
        let mut conflicts = Vec::new();
        
        // Simple heuristic: domains with very similar capabilities might conflict
        let domains: Vec<_> = domain_insights.keys().collect();
        for (i, domain_a) in domains.iter().enumerate() {
            for domain_b in domains.iter().skip(i + 1) {
                if let (Some(insight_a), Some(insight_b)) = 
                    (domain_insights.get(*domain_a), domain_insights.get(*domain_b)) {
                    
                    let overlap = insight_a.business_capabilities.iter()
                        .filter(|cap| insight_b.business_capabilities.contains(cap))
                        .count();
                    
                    if overlap > insight_a.business_capabilities.len() / 2 {
                        conflicts.push(format!("High overlap between {} and {}", domain_a, domain_b));
                    }
                }
            }
        }
        
        conflicts
    }

    fn assess_implementation_completeness(&self, evidence: &BusinessEvidence, domain_name: &str) -> f32 {
        let domain_lower = domain_name.to_lowercase();
        let relevant_patterns = evidence.code_patterns.iter()
            .filter(|p| p.to_lowercase().contains(&domain_lower))
            .count();
        
        // Simple heuristic: more patterns suggest more complete implementation
        (relevant_patterns as f32 / 10.0).min(1.0)
    }

    fn calculate_business_value_score(&self, evidence: &BusinessEvidence, domain_name: &str) -> f32 {
        let domain_lower = domain_name.to_lowercase();
        let mut score = 0.5; // Base score
        
        // Boost score for high-value domains
        match domain_lower.as_str() {
            s if s.contains("ecommerce") || s.contains("payment") => score += 0.3,
            s if s.contains("user") || s.contains("auth") => score += 0.2,
            s if s.contains("analytics") || s.contains("data") => score += 0.2,
            _ => {}
        }
        
        // Adjust based on evidence strength
        let evidence_strength = evidence.code_patterns.len() as f32 / 20.0;
        score += evidence_strength * 0.2;
        
        score.min(1.0)
    }

    fn calculate_analysis_quality(&self, domain_insights: &HashMap<String, DomainInsight>) -> f32 {
        if domain_insights.is_empty() {
            return 0.0;
        }

        let total_quality = domain_insights.values()
            .map(|insight| {
                let pattern_quality = (insight.patterns_found.len() as f32 / 5.0).min(1.0);
                let capability_quality = (insight.business_capabilities.len() as f32 / 3.0).min(1.0);
                (pattern_quality + capability_quality + insight.confidence) / 3.0
            })
            .sum::<f32>();

        total_quality / domain_insights.len() as f32
    }

    fn calculate_coherence_score(&self, relationships: &[DomainRelationship]) -> f32 {
        if relationships.is_empty() {
            return 0.5; // Neutral score for no relationships
        }

        let average_strength = relationships.iter()
            .map(|r| r.strength)
            .sum::<f32>() / relationships.len() as f32;

        average_strength
    }

    fn ground_final_context(
        &self,
        domain_analysis: &EnhancedDomainAnalysis,
        validation_result: &BusinessValidationResult,
        evidence: &BusinessEvidence
    ) -> Result<GroundedBusinessContext> {
        let mut final_domains = Vec::new();
        let mut business_capabilities = Vec::new();
        // Implementation roadmap will be generated below

        // Filter domains based on validation confidence
        for (domain_name, insight) in &domain_analysis.domain_insights {
            if validation_result.domain_validations.get(domain_name)
                .map(|v| v.confidence > 0.6)
                .unwrap_or(false) {
                
                final_domains.push(GroundedDomain {
                    name: domain_name.clone(),
                    confidence: insight.confidence,
                    evidence_strength: self.calculate_evidence_strength(evidence, domain_name),
                    business_capabilities: insight.business_capabilities.clone(),
                    implementation_status: self.determine_implementation_status(insight),
                });

                business_capabilities.extend(insight.business_capabilities.clone());
            }
        }

        // Generate implementation roadmap based on grounded context
        let implementation_roadmap = self.generate_implementation_roadmap(&final_domains, &domain_analysis.cross_domain_analysis);

        Ok(GroundedBusinessContext {
            final_domains,
            business_capabilities: business_capabilities.into_iter().collect::<std::collections::HashSet<_>>().into_iter().collect(),
            cross_domain_relationships: domain_analysis.cross_domain_analysis.domain_relationships.clone(),
            implementation_roadmap,
            overall_confidence: validation_result.overall_confidence,
            grounding_quality_score: self.calculate_grounding_quality(validation_result, evidence),
        })
    }

    fn calculate_evidence_strength(&self, evidence: &BusinessEvidence, domain_name: &str) -> f32 {
        let domain_lower = domain_name.to_lowercase();
        let pattern_matches = evidence.code_patterns.iter()
            .filter(|p| p.to_lowercase().contains(&domain_lower))
            .count();
        let file_matches = evidence.file_names.iter()
            .filter(|f| f.to_lowercase().contains(&domain_lower))
            .count();

        ((pattern_matches + file_matches) as f32 / 10.0).min(1.0)
    }

    fn determine_implementation_status(&self, insight: &DomainInsight) -> String {
        if insight.implementation_completeness > 0.8 {
            "Complete".to_string()
        } else if insight.implementation_completeness > 0.5 {
            "In Progress".to_string()
        } else if insight.implementation_completeness > 0.2 {
            "Partial".to_string()
        } else {
            "Planned".to_string()
        }
    }

    fn generate_implementation_roadmap(
        &self,
        domains: &[GroundedDomain],
        cross_domain_analysis: &CrossDomainAnalysis
    ) -> Vec<ImplementationStep> {
        let mut roadmap = Vec::new();

        // Sort domains by business value and implementation status
        let mut sorted_domains = domains.to_vec();
        sorted_domains.sort_by(|a, b| {
            let a_priority = self.calculate_implementation_priority(a);
            let b_priority = self.calculate_implementation_priority(b);
            b_priority.partial_cmp(&a_priority).unwrap_or(std::cmp::Ordering::Equal)
        });

        for (index, domain) in sorted_domains.iter().enumerate() {
            roadmap.push(ImplementationStep {
                step_number: index + 1,
                domain_focus: domain.name.clone(),
                description: format!("Implement {} capabilities", domain.name),
                estimated_effort: self.estimate_implementation_effort(domain),
                dependencies: self.identify_domain_dependencies(domain, cross_domain_analysis),
                business_value: self.calculate_domain_business_value(domain),
            });
        }

        roadmap
    }

    fn calculate_implementation_priority(&self, domain: &GroundedDomain) -> f32 {
        let completeness_weight = match domain.implementation_status.as_str() {
            "Complete" => 0.2,
            "In Progress" => 0.8,
            "Partial" => 0.6,
            "Planned" => 0.4,
            _ => 0.5,
        };

        (domain.confidence * 0.4) + (domain.evidence_strength * 0.3) + (completeness_weight * 0.3)
    }

    fn estimate_implementation_effort(&self, domain: &GroundedDomain) -> String {
        match domain.implementation_status.as_str() {
            "Complete" => "Maintenance".to_string(),
            "In Progress" => "Medium".to_string(),
            "Partial" => "High".to_string(),
            "Planned" => "Very High".to_string(),
            _ => "Medium".to_string(),
        }
    }

    fn identify_domain_dependencies(
        &self,
        domain: &GroundedDomain,
        cross_domain_analysis: &CrossDomainAnalysis
    ) -> Vec<String> {
        cross_domain_analysis.domain_relationships
            .iter()
            .filter_map(|rel| {
                if rel.domain_a == domain.name {
                    Some(rel.domain_b.clone())
                } else if rel.domain_b == domain.name {
                    Some(rel.domain_a.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    fn calculate_domain_business_value(&self, domain: &GroundedDomain) -> f32 {
        let base_value = match domain.name.to_lowercase().as_str() {
            s if s.contains("ecommerce") || s.contains("payment") => 0.9,
            s if s.contains("user") || s.contains("auth") => 0.8,
            s if s.contains("analytics") || s.contains("data") => 0.7,
            s if s.contains("content") || s.contains("cms") => 0.6,
            _ => 0.5,
        };

        (base_value * domain.confidence * domain.evidence_strength).min(1.0)
    }

    fn calculate_grounding_quality(&self, validation_result: &BusinessValidationResult, evidence: &BusinessEvidence) -> f32 {
        let validation_quality = validation_result.overall_confidence;
        let evidence_quality = evidence.calculate_quality_score();
        
        (validation_quality + evidence_quality) / 2.0
    }

    fn calculate_domain_coverage(&self, grounded_context: &GroundedBusinessContext) -> f32 {
        if grounded_context.final_domains.is_empty() {
            return 0.0;
        }

        let high_confidence_domains = grounded_context.final_domains
            .iter()
            .filter(|d| d.confidence > 0.7)
            .count();

        high_confidence_domains as f32 / grounded_context.final_domains.len() as f32
    }
}

// Domain-specific prompt templates
#[derive(Debug, Clone)]
pub struct DomainPromptTemplateEngine {
    templates: HashMap<String, PromptTemplate>,
    config: Config,
}

impl DomainPromptTemplateEngine {
    pub fn new() -> Result<Self> {
        let mut templates = HashMap::new();
        
        // Load domain-specific prompt templates
        templates.insert("E-commerce".to_string(), PromptTemplate::ecommerce());
        templates.insert("User Management".to_string(), PromptTemplate::user_management());
        templates.insert("Content Management".to_string(), PromptTemplate::cms());
        templates.insert("Analytics".to_string(), PromptTemplate::analytics());
        templates.insert("API".to_string(), PromptTemplate::api());
        
        Ok(Self {
            templates,
            config: Config::instance(),
        })
    }

    pub fn generate_domain_prompts(&self, evidence: &BusinessEvidence) -> Result<DomainSpecificPrompts> {
        let mut prompts = HashMap::new();

        for domain in &evidence.potential_domains {
            if let Some(template) = self.templates.get(&domain.name) {
                let prompt = template.generate_prompt(evidence, domain)?;
                prompts.insert(domain.name.clone(), prompt);
            }
        }

        Ok(DomainSpecificPrompts { prompts })
    }
}

#[derive(Debug, Clone)]
pub struct PromptTemplate {
    pub domain: String,
    pub system_prompt: String,
    pub analysis_points: Vec<String>,
    pub validation_criteria: Vec<String>,
}

impl PromptTemplate {
    pub fn ecommerce() -> Self {
        Self {
            domain: "E-commerce".to_string(),
            system_prompt: "You are analyzing an e-commerce application. Focus on commerce-related functionality.".to_string(),
            analysis_points: vec![
                "Product catalog management".to_string(),
                "Shopping cart functionality".to_string(),
                "Payment processing integration".to_string(),
                "Order management workflow".to_string(),
                "Inventory tracking systems".to_string(),
            ],
            validation_criteria: vec![
                "Presence of product-related entities".to_string(),
                "Cart or order management logic".to_string(),
                "Payment integration indicators".to_string(),
            ],
        }
    }

    pub fn user_management() -> Self {
        Self {
            domain: "User Management".to_string(),
            system_prompt: "You are analyzing user management and authentication systems.".to_string(),
            analysis_points: vec![
                "User registration processes".to_string(),
                "Authentication mechanisms".to_string(),
                "Authorization and permissions".to_string(),
                "Profile management features".to_string(),
                "Session handling".to_string(),
            ],
            validation_criteria: vec![
                "User-related data models".to_string(),
                "Authentication logic".to_string(),
                "Permission or role systems".to_string(),
            ],
        }
    }

    pub fn cms() -> Self {
        Self {
            domain: "Content Management".to_string(),
            system_prompt: "You are analyzing a content management system.".to_string(),
            analysis_points: vec![
                "Content creation workflows".to_string(),
                "Publishing mechanisms".to_string(),
                "Media management".to_string(),
                "Editorial workflows".to_string(),
            ],
            validation_criteria: vec![
                "Content-related entities".to_string(),
                "Publishing or editing logic".to_string(),
                "Media handling systems".to_string(),
            ],
        }
    }

    pub fn analytics() -> Self {
        Self {
            domain: "Analytics".to_string(),
            system_prompt: "You are analyzing analytics and data processing systems.".to_string(),
            analysis_points: vec![
                "Data collection mechanisms".to_string(),
                "Metrics calculation".to_string(),
                "Report generation".to_string(),
                "Dashboard functionality".to_string(),
            ],
            validation_criteria: vec![
                "Data aggregation logic".to_string(),
                "Reporting systems".to_string(),
                "Visualization components".to_string(),
            ],
        }
    }

    pub fn api() -> Self {
        Self {
            domain: "API".to_string(),
            system_prompt: "You are analyzing API and integration systems.".to_string(),
            analysis_points: vec![
                "External service integration".to_string(),
                "API endpoint definitions".to_string(),
                "Data synchronization".to_string(),
                "Service orchestration".to_string(),
            ],
            validation_criteria: vec![
                "API route definitions".to_string(),
                "External service calls".to_string(),
                "Integration middleware".to_string(),
            ],
        }
    }

    pub fn generate_prompt(&self, evidence: &BusinessEvidence, domain: &BusinessDomain) -> Result<String> {
        let mut prompt = format!("DOMAIN: {}\n\n", self.domain);
        prompt.push_str(&format!("SYSTEM: {}\n\n", self.system_prompt));
        
        prompt.push_str("EVIDENCE PROVIDED:\n");
        prompt.push_str(&format!("- Code patterns: {:?}\n", evidence.code_patterns.iter().take(5).collect::<Vec<_>>()));
        prompt.push_str(&format!("- File patterns: {:?}\n", evidence.file_names.iter().take(5).collect::<Vec<_>>()));
        prompt.push_str(&format!("- Domain confidence: {:.2}\n\n", domain.confidence));
        
        prompt.push_str("ANALYSIS FOCUS POINTS:\n");
        for point in &self.analysis_points {
            prompt.push_str(&format!("- {}\n", point));
        }
        
        prompt.push_str("\nVALIDATION CRITERIA:\n");
        for criterion in &self.validation_criteria {
            prompt.push_str(&format!("- {}\n", criterion));
        }
        
        prompt.push_str("\nPlease provide detailed analysis of this domain's implementation in the codebase.");
        
        Ok(prompt)
    }
}

// Supporting data structures
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DomainSpecificPrompts {
    pub prompts: HashMap<String, String>,
}

impl DomainSpecificPrompts {
    pub fn get_domain_prompt(&self, domain: &str) -> Option<&String> {
        self.prompts.get(domain)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BusinessEvidence {
    pub potential_domains: Vec<BusinessDomain>,
    pub code_patterns: Vec<String>,
    pub file_names: Vec<String>,
    pub architectural_indicators: Vec<String>,
    pub business_logic_indicators: Vec<String>,
}

impl BusinessEvidence {
    pub fn calculate_quality_score(&self) -> f32 {
        let pattern_quality = (self.code_patterns.len() as f32 / 20.0).min(1.0);
        let file_quality = (self.file_names.len() as f32 / 10.0).min(1.0);
        let domain_quality = (self.potential_domains.len() as f32 / 5.0).min(1.0);
        
        (pattern_quality + file_quality + domain_quality) / 3.0
    }

    pub fn create_summary(&self) -> EvidenceSummary {
        EvidenceSummary {
            total_patterns_found: self.code_patterns.len(),
            total_files_analyzed: self.file_names.len(),
            domains_identified: self.potential_domains.len(),
            strongest_domain: self.potential_domains
                .iter()
                .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap_or(std::cmp::Ordering::Equal))
                .map(|d| d.name.clone())
                .unwrap_or_else(|| "None".to_string()),
            evidence_quality_score: self.calculate_quality_score(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceSummary {
    pub total_patterns_found: usize,
    pub total_files_analyzed: usize,
    pub domains_identified: usize,
    pub strongest_domain: String,
    pub evidence_quality_score: f32,
}

// Context evidence collector
#[derive(Debug, Clone)]
pub struct ContextEvidenceCollector {
    // Simplified for now - we'll implement pattern extraction directly
}

impl ContextEvidenceCollector {
    pub fn new() -> Self {
        Self {}
    }

    pub fn collect_business_evidence(
        &self,
        project_context: &ProjectContext,
        enhanced_segments: &[EnhancedSegmentContext]
    ) -> Result<BusinessEvidence> {
        let mut code_patterns = Vec::new();
        let mut file_names = Vec::new();
        let mut architectural_indicators = Vec::new();
        let mut business_logic_indicators = Vec::new();

        // Extract patterns from segments
        for segment in enhanced_segments {
            // Extract code patterns
            code_patterns.extend(self.extract_code_patterns(&segment.segment_context.segment.content));
            
            // Extract file name patterns
            if let Some(file_name) = segment.segment_context.file_context.file_path.file_name() {
                if let Some(name_str) = file_name.to_str() {
                    file_names.push(name_str.to_string());
                }
            }

            // Extract business logic indicators
            business_logic_indicators.extend(segment.business_hints.clone());

            // Extract architectural indicators
            architectural_indicators.extend(segment.architectural_context.patterns.clone());
        }

        Ok(BusinessEvidence {
            potential_domains: project_context.business_domains.clone(),
            code_patterns,
            file_names,
            architectural_indicators,
            business_logic_indicators,
        })
    }

    fn extract_code_patterns(&self, content: &str) -> Vec<String> {
        let mut patterns = Vec::new();
        
        // Simple pattern extraction - look for key business terms
        let business_keywords = [
            "user", "auth", "login", "register", "product", "order", "cart",
            "payment", "billing", "inventory", "customer", "account", "profile",
            "content", "post", "article", "media", "upload", "publish",
            "analytics", "metrics", "report", "dashboard", "chart", "data",
            "api", "endpoint", "service", "integration", "webhook"
        ];

        let content_lower = content.to_lowercase();
        for keyword in &business_keywords {
            if content_lower.contains(keyword) {
                patterns.push(format!("business_keyword:{}", keyword));
            }
        }

        patterns
    }
}

// Pattern extraction traits and implementations
pub trait PatternExtractor: Send + Sync {
    fn extract_patterns(&self, content: &str) -> Vec<String>;
}

#[derive(Debug, Clone)]
pub struct CodePatternExtractor;

impl CodePatternExtractor {
    pub fn new() -> Self {
        Self
    }
}

impl PatternExtractor for CodePatternExtractor {
    fn extract_patterns(&self, content: &str) -> Vec<String> {
        // Implementation for extracting code patterns
        vec![]
    }
}

#[derive(Debug, Clone)]
pub struct FileNameExtractor;

impl FileNameExtractor {
    pub fn new() -> Self {
        Self
    }
}

impl PatternExtractor for FileNameExtractor {
    fn extract_patterns(&self, content: &str) -> Vec<String> {
        // Implementation for extracting file name patterns
        vec![]
    }
}

#[derive(Debug, Clone)]
pub struct ArchitecturalPatternExtractor;

impl ArchitecturalPatternExtractor {
    pub fn new() -> Self {
        Self
    }
}

impl PatternExtractor for ArchitecturalPatternExtractor {
    fn extract_patterns(&self, content: &str) -> Vec<String> {
        // Implementation for extracting architectural patterns
        vec![]
    }
}

// Business validation engine
#[derive(Debug)]
#[derive(Clone)]
pub struct BusinessValidationEngine {
    // Simplified validation using built-in rules
    config: Config,
}

impl BusinessValidationEngine {
    pub fn new() -> Result<Self> {
        Ok(Self {
            config: Config::instance(),
        })
    }

    pub fn validate_business_context(
        &self,
        domain_analysis: &EnhancedDomainAnalysis,
        evidence: &BusinessEvidence
    ) -> Result<BusinessValidationResult> {
        let mut domain_validations = HashMap::new();
        let mut overall_issues = Vec::new();

        for (domain_name, insight) in &domain_analysis.domain_insights {
            let mut domain_confidence = 0.8; // Base confidence
            let mut domain_issues = Vec::new();

            // Built-in validation rules
            let coherence_validation = self.validate_domain_coherence(domain_name, insight, evidence);
            let evidence_validation = self.validate_evidence_strength(domain_name, insight, evidence);
            let consistency_validation = self.validate_implementation_consistency(domain_name, insight, evidence);

            domain_confidence = (coherence_validation.confidence_adjustment + 
                               evidence_validation.confidence_adjustment + 
                               consistency_validation.confidence_adjustment) / 3.0;
            
            domain_issues.extend(coherence_validation.issues);
            domain_issues.extend(evidence_validation.issues);
            domain_issues.extend(consistency_validation.issues);

            domain_confidence = domain_confidence.max(0.0).min(1.0);

            domain_validations.insert(domain_name.clone(), DomainValidation {
                confidence: domain_confidence,
                issues: domain_issues.clone(),
                validated_capabilities: insight.business_capabilities.clone(),
            });

            overall_issues.extend(domain_issues);
        }

        let overall_confidence = if domain_validations.is_empty() {
            0.0
        } else {
            domain_validations.values().map(|v| v.confidence).sum::<f32>() / domain_validations.len() as f32
        };

        Ok(BusinessValidationResult {
            domain_validations,
            overall_confidence,
            validation_issues: overall_issues,
            evidence_quality_assessment: evidence.calculate_quality_score(),
        })
    }

    fn validate_domain_coherence(&self, domain_name: &str, insight: &DomainInsight, _evidence: &BusinessEvidence) -> ValidationResult {
        let mut confidence = 0.8;
        let mut issues = Vec::new();

        if insight.patterns_found.len() < 2 {
            confidence -= 0.3;
            issues.push(format!("Insufficient patterns found for {}", domain_name));
        }

        if insight.business_capabilities.is_empty() {
            confidence -= 0.2;
            issues.push(format!("No business capabilities identified for {}", domain_name));
        }

        ValidationResult {
            confidence_adjustment: confidence,
            issues,
        }
    }

    fn validate_evidence_strength(&self, domain_name: &str, insight: &DomainInsight, evidence: &BusinessEvidence) -> ValidationResult {
        let mut confidence = 0.7;
        let mut issues = Vec::new();

        let domain_lower = domain_name.to_lowercase();
        let evidence_count = evidence.code_patterns.iter()
            .filter(|p| p.to_lowercase().contains(&domain_lower))
            .count();

        if evidence_count == 0 {
            confidence -= 0.4;
            issues.push(format!("No supporting evidence found for {}", domain_name));
        } else if evidence_count < 3 {
            confidence -= 0.2;
            issues.push(format!("Limited evidence for {}", domain_name));
        }

        if insight.implementation_completeness < 0.3 {
            confidence -= 0.2;
            issues.push(format!("Low implementation completeness for {}", domain_name));
        }

        ValidationResult {
            confidence_adjustment: confidence,
            issues,
        }
    }

    fn validate_implementation_consistency(&self, _domain_name: &str, insight: &DomainInsight, _evidence: &BusinessEvidence) -> ValidationResult {
        let mut confidence = 0.8;
        let mut issues = Vec::new();

        let pattern_capability_ratio = if insight.business_capabilities.is_empty() {
            0.0
        } else {
            insight.patterns_found.len() as f32 / insight.business_capabilities.len() as f32
        };

        if pattern_capability_ratio < 0.5 {
            confidence -= 0.3;
            issues.push("Inconsistency between detected patterns and claimed capabilities".to_string());
        }

        ValidationResult {
            confidence_adjustment: confidence,
            issues,
        }
    }
}

// Supporting validation types

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationResult {
    pub confidence_adjustment: f32,
    pub issues: Vec<String>,
}

// Validation rule structs removed as they're now built into the validation engine

// Result structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessContextGroundingResult {
    pub grounded_context: GroundedBusinessContext,
    pub domain_analysis: EnhancedDomainAnalysis,
    pub validation_result: BusinessValidationResult,
    pub evidence_summary: EvidenceSummary,
    pub performance_metrics: GroundingPerformanceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundedBusinessContext {
    pub final_domains: Vec<GroundedDomain>,
    pub business_capabilities: Vec<String>,
    pub cross_domain_relationships: Vec<DomainRelationship>,
    pub implementation_roadmap: Vec<ImplementationStep>,
    pub overall_confidence: f32,
    pub grounding_quality_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundedDomain {
    pub name: String,
    pub confidence: f32,
    pub evidence_strength: f32,
    pub business_capabilities: Vec<String>,
    pub implementation_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationStep {
    pub step_number: usize,
    pub domain_focus: String,
    pub description: String,
    pub estimated_effort: String,
    pub dependencies: Vec<String>,
    pub business_value: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedDomainAnalysis {
    pub domain_insights: HashMap<String, DomainInsight>,
    pub confidence_scores: HashMap<String, f32>,
    pub cross_domain_analysis: CrossDomainAnalysis,
    pub llm_enhanced: bool,
    pub analysis_quality: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainInsight {
    pub domain_name: String,
    pub confidence: f32,
    pub patterns_found: Vec<String>,
    pub business_capabilities: Vec<String>,
    pub user_interactions: Vec<String>,
    pub implementation_completeness: f32,
    pub business_value_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossDomainAnalysis {
    pub domain_relationships: Vec<DomainRelationship>,
    pub integration_points: Vec<String>,
    pub conflicting_domains: Vec<String>,
    pub overall_coherence_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRelationship {
    pub domain_a: String,
    pub domain_b: String,
    pub relationship_type: String,
    pub strength: f32,
    pub integration_points: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessValidationResult {
    pub domain_validations: HashMap<String, DomainValidation>,
    pub overall_confidence: f32,
    pub validation_issues: Vec<String>,
    pub evidence_quality_assessment: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainValidation {
    pub confidence: f32,
    pub issues: Vec<String>,
    pub validated_capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundingPerformanceMetrics {
    pub total_grounding_time_ms: u64,
    pub evidence_collection_quality: f32,
    pub validation_confidence: f32,
    pub domain_coverage_completeness: f32,
}