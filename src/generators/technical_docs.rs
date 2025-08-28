use super::{DocumentGenerator, DocumentType, format_component_type};
use crate::core::CodebaseAnalysis;
use crate::intelligence::IntelligentAnalysis;
use anyhow::Result;

pub struct TechnicalDocumentationGenerator;

impl TechnicalDocumentationGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl DocumentGenerator for TechnicalDocumentationGenerator {
    fn generate(&self, analysis: &CodebaseAnalysis, intelligent_analysis: Option<&IntelligentAnalysis>) -> Result<String> {
        let mut content = String::new();
        
        // Document header
        content.push_str("# Technical Documentation\n");
        content.push_str(&format!("## {}\n\n", analysis.project_name));
        
        content.push_str("---\n\n");
        content.push_str("| Property | Value |\n");
        content.push_str("|----------|-------|\n");
        content.push_str(&format!("| **Project** | {} |\n", analysis.project_name));
        content.push_str(&format!("| **Technology** | {:?} |\n", analysis.project_type));
        content.push_str(&format!("| **Components** | {} |\n", analysis.components.len()));
        content.push_str(&format!("| **Generated** | {} |\n", analysis.analysis_metadata.analyzed_at.split('T').next().unwrap_or("")));
        content.push_str("\n---\n\n");
        
        // Table of contents
        content.push_str("##  Table of Contents\n\n");
        content.push_str("1. [System Overview](#1-system-overview)\n");
        content.push_str("2. [Architecture](#2-architecture)\n");
        content.push_str("3. [Component Documentation](#3-component-documentation)\n");
        content.push_str("4. [API Documentation](#4-api-documentation)\n");
        content.push_str("5. [Data Models](#5-data-models)\n");
        content.push_str("6. [Development Guidelines](#6-development-guidelines)\n");
        if intelligent_analysis.is_some() {
            content.push_str("7. [Quality Analysis](#7-quality-analysis)\n");
            content.push_str("8. [Architecture Recommendations](#8-architecture-recommendations)\n");
        }
        content.push_str("\n");
        
        // System Overview
        content.push_str("## 1. System Overview\n\n");
        content.push_str(&format!("### 1.1 Technology Stack\n\n"));
        
        let tech_description = match analysis.project_type {
            crate::core::ProjectType::React => {
                "**Frontend Framework**: React with TypeScript\n\
                - Component-based architecture\n\
                - Modern JavaScript/TypeScript features\n\
                - Reactive user interface patterns\n"
            },
            crate::core::ProjectType::SpringBoot => {
                "**Backend Framework**: Spring Boot with Java\n\
                - Enterprise-grade Java application framework\n\
                - Dependency injection and inversion of control\n\
                - Auto-configuration and starter dependencies\n"
            },
            crate::core::ProjectType::Django => {
                "**Web Framework**: Django with Python\n\
                - Model-View-Template (MVT) architecture\n\
                - Object-Relational Mapping (ORM)\n\
                - Built-in admin interface and authentication\n"
            },
            crate::core::ProjectType::Flask => {
                "**Web Framework**: Flask with Python\n\
                - Lightweight and flexible microframework\n\
                - Modular design with blueprints\n\
                - Extensible through third-party packages\n"
            },
            crate::core::ProjectType::NextJS => {
                "**Full-Stack Framework**: Next.js\n\
                - Server-side rendering and static generation\n\
                - File-based routing system\n\
                - Built-in optimization features\n"
            },
            crate::core::ProjectType::ExpressNodeJS => {
                "**Backend Framework**: Express.js\n\
                - Minimal Node.js web framework\n\
                - Middleware-based architecture\n\
                - RESTful API capabilities\n"
            },
            crate::core::ProjectType::NestJS => {
                "**Backend Framework**: NestJS with TypeScript\n\
                - Modular architecture with dependency injection\n\
                - Decorator-based programming model\n\
                - Built-in support for testing and documentation\n"
            },
            crate::core::ProjectType::FastAPI => {
                "**API Framework**: FastAPI\n\
                - High-performance Python framework\n\
                - Automatic validation and documentation\n\
                - Async/await support\n"
            },
            crate::core::ProjectType::Unknown => {
                "**Technology Stack**: Mixed or Unknown\n\
                - Multiple technologies detected\n\
                - Requires manual verification\n"
            }
        };
        
        content.push_str(tech_description);
        content.push_str("\n");
        
        content.push_str("### 1.2 System Statistics\n\n");
        content.push_str(&format!("- **Total Files**: {} analyzed\n", analysis.analysis_metadata.files_analyzed));
        content.push_str(&format!("- **Lines of Code**: {}\n", analysis.analysis_metadata.lines_of_code));
        content.push_str(&format!("- **Components**: {} identified\n", analysis.components.len()));
        
        let api_endpoints: usize = analysis.components.iter()
            .map(|c| c.api_calls.len())
            .sum();
        content.push_str(&format!("- **API Endpoints**: {}\n", api_endpoints));
        
        let avg_complexity = if !analysis.components.is_empty() {
            analysis.components.iter().map(|c| c.complexity_score as u32).sum::<u32>() as f32 / analysis.components.len() as f32
        } else { 0.0 };
        content.push_str(&format!("- **Average Complexity**: {:.1}/100\n\n", avg_complexity));
        
        // Architecture
        content.push_str("## 2. Architecture\n\n");
        content.push_str("### 2.1 High-Level Architecture\n\n");
        
        let architecture_desc = match analysis.project_type {
            crate::core::ProjectType::React => {
                "```\n\
                ┌─────────────────┐\n\
                │   React App     │\n\
                │                 │\n\
                ├─────────────────┤\n\
                │   Components    │\n\
                │   - Pages       │\n\
                │   - Forms       │\n\
                │   - Navigation  │\n\
                │   - Display     │\n\
                ├─────────────────┤\n\
                │   Services      │\n\
                │   - API Calls   │\n\
                │   - State Mgmt  │\n\
                ├─────────────────┤\n\
                │   Utilities     │\n\
                │   - Helpers     │\n\
                │   - Hooks       │\n\
                └─────────────────┘\n\
                ```\n"
            },
            crate::core::ProjectType::SpringBoot => {
                "```\n\
                ┌─────────────────┐\n\
                │  Controllers    │ ← REST Endpoints\n\
                ├─────────────────┤\n\
                │  Services       │ ← Business Logic\n\
                ├─────────────────┤\n\
                │  Repositories   │ ← Data Access\n\
                ├─────────────────┤\n\
                │  Entities       │ ← Data Models\n\
                ├─────────────────┤\n\
                │  Configuration  │ ← Spring Config\n\
                └─────────────────┘\n\
                         │\n\
                ┌─────────────────┐\n\
                │    Database     │\n\
                └─────────────────┘\n\
                ```\n"
            },
            crate::core::ProjectType::Django => {
                "```\n\
                ┌─────────────────┐\n\
                │     Views       │ ← Request Handling\n\
                ├─────────────────┤\n\
                │    Models       │ ← Data Layer\n\
                ├─────────────────┤\n\
                │   Templates     │ ← Presentation\n\
                ├─────────────────┤\n\
                │     URLs        │ ← Routing\n\
                ├─────────────────┤\n\
                │  Middleware     │ ← Request/Response\n\
                └─────────────────┘\n\
                ```\n"
            },
            crate::core::ProjectType::Flask => {
                "```\n\
                ┌─────────────────┐\n\
                │   Flask App     │\n\
                ├─────────────────┤\n\
                │   Blueprints    │ ← Route Organization\n\
                ├─────────────────┤\n\
                │   Views         │ ← Request Handlers\n\
                ├─────────────────┤\n\
                │   Models        │ ← Data Models\n\
                ├─────────────────┤\n\
                │   Templates     │ ← UI Templates\n\
                └─────────────────┘\n\
                ```\n"
            },
            _ => "Architecture diagram not available for this project type.\n"
        };
        
        content.push_str(architecture_desc);
        content.push_str("\n");
        
        content.push_str("### 2.2 Component Distribution\n\n");
        
        let mut component_types = std::collections::HashMap::new();
        for component in &analysis.components {
            *component_types.entry(&component.component_type).or_insert(0) += 1;
        }
        
        content.push_str("| Component Type | Count | Percentage | Description |\n");
        content.push_str("|----------------|-------|------------|-------------|\n");
        
        for (comp_type, count) in component_types {
            let percentage = count as f32 / analysis.components.len() as f32 * 100.0;
            let description = match comp_type {
                crate::core::ComponentType::Service => "Business logic and API services",
                crate::core::ComponentType::Context => "Data models and state management",
                crate::core::ComponentType::Form => "User input and validation",
                crate::core::ComponentType::Page => "Application screens and routes",
                crate::core::ComponentType::Navigation => "Navigation and routing",
                crate::core::ComponentType::Display => "UI display components",
                crate::core::ComponentType::Modal => "Modal dialogs and overlays",
                crate::core::ComponentType::Utility => "Helper functions and utilities",
                crate::core::ComponentType::Hook => "React hooks (if applicable)",
                crate::core::ComponentType::Layout => "Layout and structural components",
            };
            
            content.push_str(&format!("| {:?} | {} | {:.1}% | {} |\n", comp_type, count, percentage, description));
        }
        content.push_str("\n");
        
        // Component Documentation
        content.push_str("## 3. Component Documentation\n\n");
        
        // Group components by type
        let mut components_by_type: std::collections::HashMap<&crate::core::ComponentType, Vec<&crate::core::ComponentInfo>> = std::collections::HashMap::new();
        for component in &analysis.components {
            components_by_type.entry(&component.component_type)
                .or_insert_with(Vec::new)
                .push(component);
        }
        
        for (comp_type, components) in components_by_type {
            content.push_str(&format!("### 3.{} {} Components\n\n", 
                match comp_type {
                    crate::core::ComponentType::Service => "1",
                    crate::core::ComponentType::Context => "2",
                    crate::core::ComponentType::Form => "3",
                    crate::core::ComponentType::Page => "4",
                    _ => "X",
                },
                format!("{:?}", comp_type)
            ));
            
            for component in components {
                content.push_str(&format!("#### {}\n\n", component.name));
                
                content.push_str("| Attribute | Value |\n");
                content.push_str("|-----------|-------|\n");
                content.push_str(&format!("| **Type** | {} |\n", format_component_type(&component.component_type)));
                content.push_str(&format!("| **File** | `{}` |\n", component.file_path));
                content.push_str(&format!("| **Complexity** | {}/100 |\n", component.complexity_score));
                content.push_str(&format!("| **Status** | {:?} |\n", component.implementation_status));
                content.push_str("\n");
                
                content.push_str(&format!("**Purpose**: {}\n\n", component.purpose));
                
                // Props/Fields
                if !component.props.is_empty() {
                    content.push_str("**Properties/Fields**:\n\n");
                    content.push_str("| Name | Type | Required | Description |\n");
                    content.push_str("|------|------|----------|-------------|\n");
                    
                    for prop in &component.props {
                        let required = if prop.required { "Yes" } else { "No" };
                        let description = prop.description.as_deref().unwrap_or("N/A");
                        content.push_str(&format!("| `{}` | `{}` | {} | {} |\n", 
                            prop.name, prop.prop_type, required, description));
                    }
                    content.push_str("\n");
                }
                
                // Dependencies
                if !component.dependencies.is_empty() {
                    content.push_str("**Dependencies**:\n");
                    for dep in &component.dependencies {
                        content.push_str(&format!("- `{}`\n", dep));
                    }
                    content.push_str("\n");
                }
                
                // Hooks (for React components)
                if !component.hooks_used.is_empty() {
                    content.push_str("**Hooks Used**:\n");
                    for hook in &component.hooks_used {
                        content.push_str(&format!("- `{}`\n", hook));
                    }
                    content.push_str("\n");
                }
                
                content.push_str("---\n\n");
            }
        }
        
        // API Documentation
        content.push_str("## 4. API Documentation\n\n");
        
        let api_components: Vec<_> = analysis.components.iter()
            .filter(|c| !c.api_calls.is_empty())
            .collect();
        
        if api_components.is_empty() {
            content.push_str("No API endpoints were detected in the codebase.\n\n");
        } else {
            content.push_str("### 4.1 API Endpoints\n\n");
            
            let mut all_endpoints = Vec::new();
            for component in &api_components {
                for api_call in &component.api_calls {
                    all_endpoints.push((component, api_call));
                }
            }
            
            content.push_str("| Method | Endpoint | Component | Purpose |\n");
            content.push_str("|--------|----------|-----------|----------|\n");
            
            for (component, api_call) in &all_endpoints {
                content.push_str(&format!("| `{}` | `{}` | {} | {} |\n", 
                    api_call.method, api_call.endpoint, component.name, api_call.purpose));
            }
            content.push_str("\n");
            
            content.push_str("### 4.2 API Guidelines\n\n");
            
            match analysis.project_type {
                crate::core::ProjectType::SpringBoot => {
                    content.push_str("**Spring Boot REST API Conventions**:\n");
                    content.push_str("- Use `@RestController` for REST endpoints\n");
                    content.push_str("- Follow HTTP method conventions (GET, POST, PUT, DELETE)\n");
                    content.push_str("- Return appropriate HTTP status codes\n");
                    content.push_str("- Use `@Valid` for request validation\n");
                    content.push_str("- Implement proper exception handling with `@ControllerAdvice`\n\n");
                },
                crate::core::ProjectType::Django => {
                    content.push_str("**Django REST API Conventions**:\n");
                    content.push_str("- Use Django REST Framework for API endpoints\n");
                    content.push_str("- Implement proper serializers for data validation\n");
                    content.push_str("- Use ViewSets for CRUD operations\n");
                    content.push_str("- Implement authentication and permissions\n");
                    content.push_str("- Follow RESTful URL patterns\n\n");
                },
                crate::core::ProjectType::Flask => {
                    content.push_str("**Flask API Conventions**:\n");
                    content.push_str("- Use Flask-RESTful for REST API structure\n");
                    content.push_str("- Implement proper error handling\n");
                    content.push_str("- Use Flask-Marshmallow for serialization\n");
                    content.push_str("- Organize routes with Blueprints\n");
                    content.push_str("- Implement CORS for cross-origin requests\n\n");
                },
                _ => {
                    content.push_str("**General API Conventions**:\n");
                    content.push_str("- Use consistent naming conventions\n");
                    content.push_str("- Implement proper error handling\n");
                    content.push_str("- Return appropriate HTTP status codes\n");
                    content.push_str("- Use API versioning for backward compatibility\n");
                    content.push_str("- Implement rate limiting and authentication\n\n");
                }
            }
        }
        
        // Data Models
        content.push_str("## 5. Data Models\n\n");
        
        let data_components: Vec<_> = analysis.components.iter()
            .filter(|c| matches!(c.component_type, crate::core::ComponentType::Context))
            .collect();
        
        if data_components.is_empty() {
            content.push_str("No data models were detected in the codebase.\n\n");
        } else {
            content.push_str("### 5.1 Entity Relationship Overview\n\n");
            
            content.push_str("| Model | Fields | File Location |\n");
            content.push_str("|-------|--------|---------------|\n");
            
            for component in &data_components {
                let field_count = component.props.len();
                content.push_str(&format!("| `{}` | {} | `{}` |\n", 
                    component.name, field_count, component.file_path));
            }
            content.push_str("\n");
            
            content.push_str("### 5.2 Model Details\n\n");
            
            for component in &data_components {
                content.push_str(&format!("#### {} Model\n\n", component.name));
                content.push_str(&format!("**Purpose**: {}\n\n", component.purpose));
                
                if !component.props.is_empty() {
                    content.push_str("**Fields**:\n\n");
                    content.push_str("| Field Name | Type | Required | Notes |\n");
                    content.push_str("|------------|------|----------|-------|\n");
                    
                    for prop in &component.props {
                        let required = if prop.required { "Yes" } else { "No" };
                        let notes = prop.description.as_deref().unwrap_or("-");
                        content.push_str(&format!("| `{}` | `{}` | {} | {} |\n", 
                            prop.name, prop.prop_type, required, notes));
                    }
                    content.push_str("\n");
                }
                
                content.push_str("---\n\n");
            }
        }
        
        // Development Guidelines
        content.push_str("## 6. Development Guidelines\n\n");
        
        content.push_str("### 6.1 Code Standards\n\n");
        
        let code_standards = match analysis.project_type {
            crate::core::ProjectType::React => {
                "**React/TypeScript Standards**:\n\
                - Use functional components with hooks\n\
                - Implement proper TypeScript typing\n\
                - Follow React naming conventions (PascalCase for components)\n\
                - Use ESLint and Prettier for code formatting\n\
                - Implement proper error boundaries\n\
                - Use React.memo() for performance optimization where appropriate\n"
            },
            crate::core::ProjectType::SpringBoot => {
                "**Java/Spring Boot Standards**:\n\
                - Follow Java naming conventions (CamelCase)\n\
                - Use proper package structure\n\
                - Implement dependency injection with @Autowired or constructor injection\n\
                - Write comprehensive unit tests with JUnit and Mockito\n\
                - Use Spring Boot starters for common functionality\n\
                - Implement proper exception handling\n"
            },
            crate::core::ProjectType::Django => {
                "**Python/Django Standards**:\n\
                - Follow PEP 8 style guide\n\
                - Use Django's MVT pattern consistently\n\
                - Implement proper model relationships\n\
                - Use Django forms for data validation\n\
                - Write comprehensive tests with Django's test framework\n\
                - Use Django's built-in authentication and authorization\n"
            },
            crate::core::ProjectType::Flask => {
                "**Python/Flask Standards**:\n\
                - Follow PEP 8 style guide\n\
                - Use Blueprints for application modularity\n\
                - Implement proper error handling\n\
                - Use Flask-WTF for form handling\n\
                - Write tests with pytest or unittest\n\
                - Use environment variables for configuration\n"
            },
            _ => {
                "**General Standards**:\n\
                - Follow language-specific conventions\n\
                - Write clean, readable code\n\
                - Implement comprehensive testing\n\
                - Use version control best practices\n\
                - Document complex logic\n"
            }
        };
        
        content.push_str(code_standards);
        content.push_str("\n");
        
        content.push_str("### 6.2 Testing Strategy\n\n");
        content.push_str("**Unit Testing**:\n");
        content.push_str("- Test individual components/functions in isolation\n");
        content.push_str("- Aim for 80%+ code coverage\n");
        content.push_str("- Use mocking for external dependencies\n\n");
        
        content.push_str("**Integration Testing**:\n");
        content.push_str("- Test component interactions\n");
        content.push_str("- Verify API endpoints work correctly\n");
        content.push_str("- Test database operations\n\n");
        
        content.push_str("**End-to-End Testing**:\n");
        content.push_str("- Test complete user workflows\n");
        content.push_str("- Verify system works as a whole\n");
        content.push_str("- Use tools like Cypress, Selenium, or Playwright\n\n");
        
        // Quality Analysis (if intelligent analysis is available)
        if let Some(intel) = intelligent_analysis {
            content.push_str("## 7. Quality Analysis\n\n");
            
            content.push_str("### 7.1 Quality Metrics\n\n");
            content.push_str("| Metric | Score | Status |\n");
            content.push_str("|--------|-------|--------|\n");
            
            let format_score = |score: f32| -> String {
                let percentage = score * 100.0;
                let status = if percentage >= 80.0 { " Excellent" }
                           else if percentage >= 60.0 { "🟡 Good" }
                           else { "🔴 Needs Improvement" };
                format!("{:.1}% | {}", percentage, status)
            };
            
            content.push_str(&format!("| Overall Quality | {} |\n", format_score(intel.quality_metrics.overall_score)));
            content.push_str(&format!("| Maintainability | {} |\n", format_score(intel.quality_metrics.maintainability)));
            content.push_str(&format!("| Complexity Management | {} |\n", format_score(intel.quality_metrics.complexity)));
            content.push_str(&format!("| Technical Debt | {} |\n", format_score(intel.quality_metrics.technical_debt_score)));
            content.push_str(&format!("| Est. Test Coverage | {} |\n", format_score(intel.quality_metrics.test_coverage_estimate)));
            content.push_str(&format!("| Documentation | {} |\n", format_score(intel.quality_metrics.documentation_score)));
            content.push_str("\n");
            
            // Technical insights
            if !intel.technical_insights.is_empty() {
                content.push_str("### 7.2 Technical Issues\n\n");
                
                for insight in &intel.technical_insights {
                    let severity_icon = match insight.severity {
                        crate::intelligence::Severity::Critical => "🔴",
                        crate::intelligence::Severity::High => "🟠",
                        crate::intelligence::Severity::Medium => "🟡",
                        crate::intelligence::Severity::Low => "🟢",
                    };
                    
                    content.push_str(&format!("#### {} {} - {}\n\n", severity_icon, insight.title, format!("{:?}", insight.category)));
                    content.push_str(&format!("**Severity**: {:?}\n\n", insight.severity));
                    content.push_str(&format!("{}\n\n", insight.description));
                    
                    if !insight.affected_components.is_empty() {
                        content.push_str(&format!("**Affected Components**: {}\n\n", insight.affected_components.join(", ")));
                    }
                    
                    content.push_str("**Recommendations**:\n");
                    for rec in &insight.recommendations {
                        content.push_str(&format!("- {}\n", rec));
                    }
                    content.push_str("\n");
                }
            }
            
            // Architecture recommendations
            content.push_str("## 8. Architecture Recommendations\n\n");
            
            if intel.architecture_recommendations.is_empty() {
                content.push_str("No specific architecture recommendations at this time. The current architecture appears to follow good practices.\n\n");
            } else {
                for rec in &intel.architecture_recommendations {
                    content.push_str(&format!("### 8.{} {}\n\n", 
                        intel.architecture_recommendations.iter().position(|r| std::ptr::eq(r, rec)).unwrap() + 1,
                        rec.pattern_name
                    ));
                    
                    content.push_str(&format!("**Description**: {}\n\n", rec.description));
                    content.push_str(&format!("**Implementation Effort**: {}\n\n", rec.implementation_effort));
                    
                    content.push_str("**Benefits**:\n");
                    for benefit in &rec.benefits {
                        content.push_str(&format!("- {}\n", benefit));
                    }
                    content.push_str("\n");
                    
                    if !rec.applicable_components.is_empty() {
                        content.push_str(&format!("**Applicable Components**: {}\n\n", rec.applicable_components.join(", ")));
                    }
                }
            }
        }
        
        // Document footer
        content.push_str("---\n\n");
        content.push_str("**Document Information**\n");
        content.push_str(&format!("- **Generated**: {}\n", analysis.analysis_metadata.analyzed_at));
        content.push_str(&format!("- **Generator**: Codebase Workflow Analyzer v{}\n", analysis.analysis_metadata.analyzer_version));
        content.push_str(&format!("- **Components Documented**: {}\n", analysis.components.len()));
        content.push_str("- **Document Type**: Technical Documentation\n\n");
        
        content.push_str("*This technical documentation was automatically generated from codebase analysis. Please review and update with additional implementation details as needed.*\n");
        
        Ok(content)
    }

    fn get_file_extension(&self) -> &str {
        "md"
    }

    fn get_document_type(&self) -> DocumentType {
        DocumentType::TechnicalDocumentation
    }
}