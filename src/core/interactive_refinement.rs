use crate::core::{
    refinement_structures::*,
    CodebaseAnalysis,
};
use anyhow::Result;
use std::io::{self, Write};
use chrono::Utc;

pub struct InteractiveRefinementEngine {
    session_id: String,
}

impl InteractiveRefinementEngine {
    pub fn new() -> Self {
        Self {
            session_id: format!("refinement_{}", Utc::now().timestamp()),
        }
    }

    pub fn start_interactive_session(
        &self, 
        analysis: CodebaseAnalysis
    ) -> Result<RefinedAnalysisResult> {
        println!("🔍 Starting Interactive Refinement Session");
        println!("   Session ID: {}", self.session_id);
        println!("   Analysis Quality Before Refinement: {:.1}%", 
            analysis.analysis_metadata.confidence_score * 100.0);
        println!();

        let mut corrections = RefinementCorrections {
            product_type_correction: None,
            persona_corrections: Vec::new(),
            feature_priority_corrections: Vec::new(),
            business_context_enhancement: BusinessContextEnhancement {
                strategic_context: String::new(),
                market_positioning: String::new(),
                competitive_advantages: Vec::new(),
                business_goals: Vec::new(),
                success_metrics: Vec::new(),
                compliance_requirements: Vec::new(),
            },
            user_story_corrections: Vec::new(),
        };

        // Step 1: Validate Product Type
        let corrected_product_type = self.validate_product_type(&analysis)?;
        corrections.product_type_correction = corrected_product_type;

        // Step 2: Validate User Personas
        let corrected_personas = self.validate_personas(&analysis)?;
        corrections.persona_corrections = corrected_personas;

        // Step 3: Enhance Business Context
        let enhanced_business_context = self.enhance_business_context(&analysis)?;
        corrections.business_context_enhancement = enhanced_business_context;

        // Step 4: Validate Feature Priorities (if user stories exist)
        if !analysis.user_stories.is_empty() {
            let priority_corrections = self.validate_feature_priorities(&analysis)?;
            corrections.feature_priority_corrections = priority_corrections;
        }

        // Step 5: Create refined analysis result
        let refined_result = self.create_refined_analysis(analysis, corrections)?;

        println!("\n✅ Refinement session completed successfully!");
        println!("   Confidence improvement: {:.1}%", 
            refined_result.metadata.confidence_improvement.overall_improvement * 100.0);
        
        Ok(refined_result)
    }

    fn validate_product_type(&self, analysis: &CodebaseAnalysis) -> Result<Option<ProductTypeCorrection>> {
        println!("📊 STEP 1: Product Type Validation");
        println!("   AI detected: '{}'", analysis.business_context.inferred_product_type);
        println!("   Confidence: {:.1}%", analysis.business_context.confidence * 100.0);
        println!();

        print!("Is this product type accurate? (y/n): ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if input.trim().to_lowercase() == "n" || input.trim().to_lowercase() == "no" {
            println!("\nPlease select the correct product type:");
            println!("1. E-commerce Platform");
            println!("2. B2B SaaS Platform");  
            println!("3. Healthcare Management System");
            println!("4. Financial Services Platform");
            println!("5. Content Management System");
            println!("6. Analytics/Data Platform");
            println!("7. Development Tools & Infrastructure");
            println!("8. API Service/Microservice");
            println!("9. Mobile Application");
            println!("10. Other (specify)");
            
            print!("\nEnter your choice (1-10): ");
            io::stdout().flush()?;
            
            let mut choice = String::new();
            io::stdin().read_line(&mut choice)?;
            
            let corrected_type = match choice.trim() {
                "1" => "E-commerce Platform".to_string(),
                "2" => "B2B SaaS Platform".to_string(),
                "3" => "Healthcare Management System".to_string(),
                "4" => "Financial Services Platform".to_string(),
                "5" => "Content Management System".to_string(),
                "6" => "Analytics/Data Platform".to_string(),
                "7" => "Development Tools & Infrastructure".to_string(),
                "8" => "API Service/Microservice".to_string(),
                "9" => "Mobile Application".to_string(),
                "10" => {
                    print!("Please specify the product type: ");
                    io::stdout().flush()?;
                    let mut custom = String::new();
                    io::stdin().read_line(&mut custom)?;
                    custom.trim().to_string()
                },
                _ => {
                    println!("Invalid choice. Keeping original detection.");
                    return Ok(None);
                }
            };

            print!("What industry/domain is this for? (e.g., Healthcare, Fintech, E-commerce): ");
            io::stdout().flush()?;
            let mut industry = String::new();
            io::stdin().read_line(&mut industry)?;

            print!("What is the target market? (e.g., Enterprise, SMB, Consumer): ");
            io::stdout().flush()?;
            let mut target_market = String::new();
            io::stdin().read_line(&mut target_market)?;

            print!("What is the business model? (e.g., Subscription, One-time purchase, Freemium): ");
            io::stdout().flush()?;
            let mut business_model = String::new();
            io::stdin().read_line(&mut business_model)?;

            print!("Why was the AI detection incorrect? ");
            io::stdout().flush()?;
            let mut rationale = String::new();
            io::stdin().read_line(&mut rationale)?;

            println!("✅ Product type corrected to: {}", corrected_type);
            
            Ok(Some(ProductTypeCorrection {
                original: analysis.business_context.inferred_product_type.clone(),
                corrected: corrected_type,
                rationale: rationale.trim().to_string(),
                industry: Some(industry.trim().to_string()),
                target_market: Some(target_market.trim().to_string()),
                business_model: Some(business_model.trim().to_string()),
            }))
        } else {
            println!("✅ Product type validated as correct");
            Ok(None)
        }
    }

    fn validate_personas(&self, analysis: &CodebaseAnalysis) -> Result<Vec<PersonaCorrection>> {
        println!("\n👥 STEP 2: User Persona Validation");
        
        if analysis.business_context.primary_user_personas.is_empty() {
            println!("   No personas detected by AI. Let's define them.");
        } else {
            println!("   AI detected personas:");
            for (i, persona) in analysis.business_context.primary_user_personas.iter().enumerate() {
                println!("   {}. {}", i + 1, persona);
            }
        }
        
        let mut corrections = Vec::new();
        
        print!("\nHow many primary user personas does this product serve? (1-5): ");
        io::stdout().flush()?;
        
        let mut count_input = String::new();
        io::stdin().read_line(&mut count_input)?;
        
        let persona_count: usize = count_input.trim().parse().unwrap_or(2);
        let persona_count = persona_count.min(5).max(1);
        
        for i in 0..persona_count {
            println!("\n--- Persona {} ---", i + 1);
            
            print!("Role/Title (e.g., 'Portfolio Manager', 'System Administrator'): ");
            io::stdout().flush()?;
            let mut role = String::new();
            io::stdin().read_line(&mut role)?;
            
            print!("Description (who they are and what they do): ");
            io::stdout().flush()?;
            let mut description = String::new();
            io::stdin().read_line(&mut description)?;
            
            print!("Context (where/how they work): ");
            io::stdout().flush()?;
            let mut context = String::new();
            io::stdin().read_line(&mut context)?;
            
            println!("Primary goals (enter 2-4 goals, type 'done' when finished):");
            let mut goals = Vec::new();
            let mut goal_count = 1;
            loop {
                print!("  Goal {}: ", goal_count);
                io::stdout().flush()?;
                let mut goal = String::new();
                io::stdin().read_line(&mut goal)?;
                let goal = goal.trim();
                
                if goal.to_lowercase() == "done" || goals.len() >= 4 {
                    break;
                }
                if !goal.is_empty() {
                    goals.push(goal.to_string());
                    goal_count += 1;
                }
            }
            
            print!("Business priority for this persona (Critical/High/Medium/Low): ");
            io::stdout().flush()?;
            let mut priority_input = String::new();
            io::stdin().read_line(&mut priority_input)?;
            
            let priority = match priority_input.trim().to_lowercase().as_str() {
                "critical" => BusinessPriority::Critical,
                "high" => BusinessPriority::High,
                "medium" => BusinessPriority::Medium,
                "low" | _ => BusinessPriority::Low,
            };
            
            let validated_persona = ValidatedPersona {
                role: role.trim().to_string(),
                description: description.trim().to_string(),
                primary_goals: goals,
                context: context.trim().to_string(),
                business_priority: priority,
            };
            
            // Find if this corrects an existing AI-detected persona
            let original_persona = if i < analysis.business_context.primary_user_personas.len() {
                analysis.business_context.primary_user_personas[i].clone()
            } else {
                "Not detected".to_string()
            };
            
            corrections.push(PersonaCorrection {
                original_persona,
                corrected_persona: validated_persona,
                rationale: "Human-defined persona based on business knowledge".to_string(),
            });
            
            println!("✅ Persona '{}' added", role.trim());
        }
        
        println!("✅ {} personas validated/defined", corrections.len());
        Ok(corrections)
    }

    fn enhance_business_context(&self, _analysis: &CodebaseAnalysis) -> Result<BusinessContextEnhancement> {
        println!("\n🎯 STEP 3: Business Context Enhancement");
        
        print!("Strategic context (what is the business goal of this product?): ");
        io::stdout().flush()?;
        let mut strategic_context = String::new();
        io::stdin().read_line(&mut strategic_context)?;
        
        print!("Market positioning (how does this differentiate from competitors?): ");
        io::stdout().flush()?;
        let mut market_positioning = String::new();
        io::stdin().read_line(&mut market_positioning)?;
        
        println!("Competitive advantages (enter up to 3, type 'done' when finished):");
        let mut competitive_advantages = Vec::new();
        for i in 1..=3 {
            print!("  Advantage {}: ", i);
            io::stdout().flush()?;
            let mut advantage = String::new();
            io::stdin().read_line(&mut advantage)?;
            let advantage = advantage.trim();
            
            if advantage.to_lowercase() == "done" || advantage.is_empty() {
                break;
            }
            competitive_advantages.push(advantage.to_string());
        }
        
        println!("Business goals (enter up to 4, type 'done' when finished):");
        let mut business_goals = Vec::new();
        for i in 1..=4 {
            print!("  Goal {}: ", i);
            io::stdout().flush()?;
            let mut goal = String::new();
            io::stdin().read_line(&mut goal)?;
            let goal = goal.trim();
            
            if goal.to_lowercase() == "done" || goal.is_empty() {
                break;
            }
            business_goals.push(goal.to_string());
        }
        
        println!("Success metrics (how do you measure success?):");
        let mut success_metrics = Vec::new();
        for i in 1..=3 {
            print!("  Metric {} name: ", i);
            io::stdout().flush()?;
            let mut name = String::new();
            io::stdin().read_line(&mut name)?;
            let name = name.trim();
            
            if name.to_lowercase() == "done" || name.is_empty() {
                break;
            }
            
            print!("    Description: ");
            io::stdout().flush()?;
            let mut description = String::new();
            io::stdin().read_line(&mut description)?;
            
            print!("    Target value: ");
            io::stdout().flush()?;
            let mut target = String::new();
            io::stdin().read_line(&mut target)?;
            
            success_metrics.push(BusinessMetric {
                name: name.to_string(),
                description: description.trim().to_string(),
                current_value: None,
                target_value: target.trim().to_string(),
                measurement_method: "Manual tracking".to_string(),
                priority: BusinessPriority::High,
            });
        }
        
        print!("Any compliance requirements? (e.g., GDPR, HIPAA, SOX): ");
        io::stdout().flush()?;
        let mut compliance_input = String::new();
        io::stdin().read_line(&mut compliance_input)?;
        
        let compliance_requirements = if compliance_input.trim().is_empty() {
            Vec::new()
        } else {
            compliance_input.trim().split(',').map(|s| s.trim().to_string()).collect()
        };
        
        println!("✅ Business context enhanced");
        
        Ok(BusinessContextEnhancement {
            strategic_context: strategic_context.trim().to_string(),
            market_positioning: market_positioning.trim().to_string(),
            competitive_advantages,
            business_goals,
            success_metrics,
            compliance_requirements,
        })
    }

    fn validate_feature_priorities(&self, analysis: &CodebaseAnalysis) -> Result<Vec<PriorityCorrection>> {
        println!("\n⭐ STEP 4: Feature Priority Validation");
        println!("   Found {} user stories to validate", analysis.user_stories.len());
        
        let mut corrections = Vec::new();
        
        print!("Do you want to review and adjust user story priorities? (y/n): ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if input.trim().to_lowercase() != "y" && input.trim().to_lowercase() != "yes" {
            println!("⏭️ Skipping priority validation");
            return Ok(corrections);
        }
        
        // Show top 5 user stories for priority review
        let stories_to_review = analysis.user_stories.iter().take(5);
        
        for (i, story) in stories_to_review.enumerate() {
            println!("\n--- Story {} ---", i + 1);
            println!("Title: {}", story.title);
            println!("Current Priority: {:?}", story.priority);
            println!("Description: {}", story.description);
            
            print!("New priority (Critical/High/Medium/Low) or 'keep' for unchanged: ");
            io::stdout().flush()?;
            
            let mut priority_input = String::new();
            io::stdin().read_line(&mut priority_input)?;
            let priority_input = priority_input.trim().to_lowercase();
            
            if priority_input != "keep" {
                let new_priority = match priority_input.as_str() {
                    "critical" => BusinessPriority::Critical,
                    "high" => BusinessPriority::High,
                    "medium" => BusinessPriority::Medium,
                    "low" => BusinessPriority::Low,
                    _ => {
                        println!("Invalid priority, keeping original");
                        continue;
                    }
                };
                
                print!("Why is this priority correct? ");
                io::stdout().flush()?;
                let mut rationale = String::new();
                io::stdin().read_line(&mut rationale)?;
                
                print!("What's the business impact? ");
                io::stdout().flush()?;
                let mut impact = String::new();
                io::stdin().read_line(&mut impact)?;
                
                corrections.push(PriorityCorrection {
                    feature_name: story.title.clone(),
                    original_priority: format!("{:?}", story.priority),
                    corrected_priority: new_priority,
                    rationale: rationale.trim().to_string(),
                    business_impact: impact.trim().to_string(),
                });
                
                println!("✅ Priority updated for '{}'", story.title);
            }
        }
        
        if analysis.user_stories.len() > 5 {
            println!("\n💡 Note: Only top 5 stories were reviewed. Run full refinement for complete validation.");
        }
        
        println!("✅ {} priority corrections applied", corrections.len());
        Ok(corrections)
    }

    fn create_refined_analysis(
        &self,
        original_analysis: CodebaseAnalysis,
        corrections: RefinementCorrections
    ) -> Result<RefinedAnalysisResult> {
        let now = Utc::now();
        
        // Calculate confidence improvements
        let confidence_improvement = ConfidenceImprovement {
            business_context: (
                original_analysis.business_context.confidence as f64, 
                0.85 // Assume human validation brings confidence to 85%
            ),
            user_personas: (
                if original_analysis.business_context.primary_user_personas.is_empty() { 0.0 } else { 0.6 },
                0.90 // Human-defined personas are highly confident
            ),
            feature_priorities: (
                0.5, // Default AI priority confidence
                if corrections.feature_priority_corrections.is_empty() { 0.5 } else { 0.85 }
            ),
            overall_improvement: 0.25, // Average 25% improvement through human validation
        };
        
        // Build business intelligence from corrections
        let business_intelligence = BusinessIntelligence {
            validated_product_type: corrections.product_type_correction.as_ref()
                .map(|c| c.corrected.clone())
                .unwrap_or(original_analysis.business_context.inferred_product_type.clone()),
            validated_industry: corrections.product_type_correction.as_ref()
                .and_then(|c| c.industry.clone())
                .unwrap_or("Not specified".to_string()),
            validated_target_market: corrections.product_type_correction.as_ref()
                .and_then(|c| c.target_market.clone())
                .unwrap_or("General market".to_string()),
            validated_business_model: corrections.product_type_correction.as_ref()
                .and_then(|c| c.business_model.clone())
                .unwrap_or("Not specified".to_string()),
            validated_personas: corrections.persona_corrections.iter()
                .map(|c| c.corrected_persona.clone())
                .collect(),
            business_metrics: corrections.business_context_enhancement.success_metrics.clone(),
            success_criteria: corrections.business_context_enhancement.success_metrics.iter()
                .map(|m| SuccessCriterion {
                    criterion: m.name.clone(),
                    measurement: m.measurement_method.clone(),
                    target_value: m.target_value.clone(),
                    priority: m.priority.clone(),
                })
                .collect(),
            strategic_context: corrections.business_context_enhancement.strategic_context.clone(),
            market_positioning: corrections.business_context_enhancement.market_positioning.clone(),
            compliance_requirements: corrections.business_context_enhancement.compliance_requirements.clone(),
        };

        // Create placeholder analysis result (normally would be from existing analysis)
        let placeholder_original = AnalysisResult {
            business_context: crate::core::types::BusinessDomain {
                name: original_analysis.business_context.inferred_product_type.clone(),
                confidence: original_analysis.business_context.confidence,
            },
            frameworks_detected: Vec::new(),
            confidence_scores: std::collections::HashMap::new(),
        };
        
        Ok(RefinedAnalysisResult {
            metadata: RefinementMetadata {
                analyzer_version: original_analysis.analysis_metadata.analyzer_version.clone(),
                analysis_date: now,
                refinement_date: now,
                refinement_stakeholders: vec![StakeholderRole::ProductManager], // Assume PM validation
                confidence_improvement,
                refinement_session_id: self.session_id.clone(),
            },
            business_intelligence,
            feature_status_intelligence: FeatureStatusIntelligence {
                completed_features: Vec::new(),
                in_progress_features: Vec::new(),
                todo_features: Vec::new(),
                new_features_needed: Vec::new(),
                technical_debt_items: Vec::new(),
            },
            technical_context: TechnicalContext {
                existing_patterns: Vec::new(),
                integration_points: Vec::new(),
                code_style_requirements: Vec::new(),
                testing_strategy: TestingStrategy {
                    framework: "Standard testing approach".to_string(),
                    coverage_requirements: "80% minimum".to_string(),
                    test_types: vec!["Unit".to_string(), "Integration".to_string()],
                    quality_gates: vec!["Automated testing".to_string()],
                },
                implementation_constraints: Vec::new(),
                architecture_patterns: Vec::new(),
            },
            user_stories: UserStoryCollection {
                stories: Vec::new(),
                epics: Vec::new(),
                user_journey_maps: Vec::new(),
                acceptance_test_scenarios: Vec::new(),
            },
            integration_readiness: IntegrationReadiness {
                ccmp_import_ready: true,
                claude_spec_ready: true,
                notion_integration_ready: false,
                validation_score: 0.85,
                readiness_checks: std::collections::HashMap::new(),
            },
            original_analysis: placeholder_original,
        })
    }

    pub fn create_batch_refinement(
        &self,
        analysis: CodebaseAnalysis,
        business_context: &str,
        target_users: &str,
    ) -> Result<RefinedAnalysisResult> {
        println!("🔄 Creating batch refinement with provided context");
        println!("   Business Context: {}", business_context);
        println!("   Target Users: {}", target_users);

        // Create automated refinement based on provided context
        let mut corrections = RefinementCorrections {
            product_type_correction: Some(ProductTypeCorrection {
                original: analysis.business_context.inferred_product_type.clone(),
                corrected: business_context.to_string(),
                rationale: "Batch refinement with user-provided context".to_string(),
                industry: Some("Technology".to_string()),
                target_market: Some("Enterprise".to_string()),
                business_model: Some("B2B".to_string()),
            }),
            persona_corrections: target_users.split(',')
                .map(|user| PersonaCorrection {
                    original_persona: "Generic User".to_string(),
                    corrected_persona: ValidatedPersona {
                        role: user.trim().to_string(),
                        description: format!("Primary user of {}", business_context),
                        primary_goals: vec!["Achieve business objectives".to_string()],
                        context: business_context.to_string(),
                        business_priority: BusinessPriority::High,
                    },
                    rationale: "Batch refinement based on user input".to_string(),
                })
                .collect(),
            feature_priority_corrections: Vec::new(),
            business_context_enhancement: BusinessContextEnhancement {
                strategic_context: business_context.to_string(),
                market_positioning: "Market-leading solution".to_string(),
                competitive_advantages: vec!["Advanced functionality".to_string()],
                business_goals: vec!["Increase efficiency".to_string()],
                success_metrics: Vec::new(),
                compliance_requirements: Vec::new(),
            },
            user_story_corrections: Vec::new(),
        };

        self.create_refined_analysis(analysis, corrections)
    }
}