# Framework Analysis - Detection Methodology

## Overview

The Codebase Workflow Analyzer employs a sophisticated **data flow-first analysis** approach to detect frameworks and extract business domain information. This methodology focuses on understanding how information flows from end users through the application stack to identify business patterns and domain-specific implementations.

## Core Philosophy: Data Flow-First Analysis

Following the principle that "information flows from end user to persistence layer", we analyze each framework's unique data flow pattern to understand:

1. **Request Processing Patterns** - How requests enter and are routed through the system
2. **Business Logic Location** - Where domain-specific processing occurs
3. **Domain Indicators** - Patterns that reveal specific business domains  
4. **Data Transformation** - How information is processed throughout the stack

## Supported Frameworks

### Web Frameworks

#### **Spring Boot (Java)**
**Detection Patterns:**
- `@SpringBootApplication`, `@RestController`, `@Service` annotations
- Maven `pom.xml` with spring-boot dependencies
- Application properties configuration files

**Data Flow Analysis:**
- **Entry Point**: `@RestController` endpoints
- **Business Logic**: `@Service` and `@Component` classes
- **Data Layer**: `@Repository` classes and JPA entities
- **Configuration**: `application.yml/properties` files

**Business Domain Indicators:**
- Controller endpoint patterns (`/api/users`, `/orders`, `/products`)
- Entity relationship patterns in JPA models
- Service method naming conventions
- Integration patterns (payment, notification, etc.)

#### **React (TypeScript/JavaScript)**
**Detection Patterns:**
- `package.json` with React dependencies
- JSX/TSX file presence
- React component patterns and hooks

**Data Flow Analysis:**
- **Entry Point**: Route components and navigation
- **Business Logic**: Custom hooks and service layers
- **State Management**: Redux, Context API, or local state patterns
- **API Integration**: HTTP client configurations and API calls

**Business Domain Indicators:**
- Component naming patterns (`UserProfile`, `ProductCatalog`)
- State shape and data models
- API endpoint interactions
- Form validation and business rules

#### **NestJS (TypeScript)**
**Detection Patterns:**
- `@Controller`, `@Injectable`, `@Module` decorators
- `nest-cli.json` configuration file
- TypeScript with NestJS-specific imports

**Data Flow Analysis:**
- **Entry Point**: Controller route handlers
- **Business Logic**: Service providers and business logic modules
- **Data Layer**: Repository patterns and database integrations
- **Middleware**: Guards, interceptors, and pipes

#### **Flask (Python)**
**Detection Patterns:**
- Flask application instances and route decorators
- `requirements.txt` or `pyproject.toml` with Flask
- Blueprint patterns for modular applications

**Data Flow Analysis:**
- **Entry Point**: Route handlers and view functions
- **Business Logic**: Service modules and business functions
- **Data Layer**: SQLAlchemy models or database integrations
- **Configuration**: Flask configuration patterns

#### **Django (Python)**
**Detection Patterns:**
- Django project structure with `manage.py`
- Model-View-Template patterns
- Django-specific imports and configurations

**Data Flow Analysis:**
- **Entry Point**: URL patterns and view functions/classes
- **Business Logic**: Model methods and service layers
- **Data Layer**: Django ORM models and database schemas
- **Templates**: Business logic embedded in template contexts

#### **Rust Web Frameworks**
**Actix-Web Detection:**
- `Cargo.toml` with actix-web dependencies
- Handler functions with Actix-specific attributes
- Application builder patterns

**Axum Detection:**
- `Cargo.toml` with axum and tokio dependencies
- Router and handler function patterns
- Tower service integrations

## Analysis Methodology

### 1. Pattern-Based Detection

```rust
pub struct FrameworkDetector {
    confidence_threshold: f32,
    pattern_matchers: HashMap<FrameworkType, Vec<Pattern>>,
}

pub struct Pattern {
    file_pattern: Regex,
    content_pattern: Regex,
    weight: f32,
    required: bool,
}
```

**Detection Process:**
1. **File System Scanning**: Identify key files and directory structures
2. **Dependency Analysis**: Parse package managers for framework dependencies
3. **Content Pattern Matching**: Search for framework-specific code patterns
4. **Confidence Scoring**: Weighted scoring based on pattern matches

### 2. AST-Enhanced Detection

**Enhanced Analysis:**
- **Semantic Understanding**: AST parsing for deeper code comprehension
- **Import Analysis**: Dependency graph construction from import statements
- **Function Signature Analysis**: API endpoint and business method identification
- **Type System Integration**: Leverage type annotations for business domain hints

### 3. Business Domain Classification

**Domain Detection Patterns:**

#### **E-commerce Indicators**
- Entity names: `Product`, `Order`, `Cart`, `Payment`, `Inventory`
- Endpoint patterns: `/shop`, `/checkout`, `/products`, `/orders`
- Business logic: Price calculations, inventory management, payment processing
- Integration patterns: Payment gateways, shipping APIs, tax calculations

#### **Content Management Indicators**
- Entity names: `Article`, `Page`, `Media`, `Category`, `Author`
- Endpoint patterns: `/admin`, `/content`, `/media`, `/cms`
- Business logic: Content publishing, media management, user permissions
- Integration patterns: CDN, search engines, analytics

#### **User Management Indicators**
- Entity names: `User`, `Profile`, `Role`, `Permission`, `Session`
- Endpoint patterns: `/auth`, `/users`, `/profile`, `/admin`
- Business logic: Authentication, authorization, user lifecycle
- Integration patterns: OAuth, LDAP, email services

#### **Financial Services Indicators**
- Entity names: `Account`, `Transaction`, `Balance`, `Audit`, `Report`
- Endpoint patterns: `/transactions`, `/accounts`, `/reports`
- Business logic: Transaction processing, compliance, reporting
- Integration patterns: Banking APIs, regulatory reporting, fraud detection

### 4. Confidence Scoring Algorithm

```rust
pub fn calculate_framework_confidence(
    patterns: &[PatternMatch],
    weights: &FrameworkWeights,
) -> f32 {
    let pattern_score = patterns.iter()
        .map(|p| p.confidence * weights.pattern_weight)
        .sum::<f32>();
    
    let dependency_score = calculate_dependency_confidence(patterns)
        * weights.dependency_weight;
    
    let structure_score = calculate_structure_confidence(patterns)
        * weights.structure_weight;
    
    (pattern_score + dependency_score + structure_score) / 3.0
}
```

**Scoring Factors:**
- **Pattern Matches** (40%): Direct code pattern recognition
- **Dependencies** (35%): Framework-specific dependencies and versions
- **Project Structure** (25%): File organization and architectural patterns

## Integration with AST Analysis

### Multi-Language AST Support

The framework uses **tree-sitter** for multi-language AST parsing:

- **TypeScript/JavaScript**: React component analysis, hook patterns
- **Java**: Spring annotation processing, dependency injection patterns  
- **Python**: Decorator analysis, Django/Flask pattern recognition
- **Rust**: Macro expansion, trait implementations, async patterns

### AST-Driven Business Logic Detection

```rust
pub struct BusinessLogicExtractor {
    ast_analyzer: ASTAnalyzer,
    pattern_database: PatternDatabase,
}

impl BusinessLogicExtractor {
    pub fn extract_business_patterns(&self, code: &str, language: Language) -> Vec<BusinessPattern> {
        let ast = self.ast_analyzer.parse(code, language)?;
        let patterns = self.find_business_patterns(&ast);
        self.classify_business_domain(patterns)
    }
}
```

**Business Pattern Extraction:**
- **API Endpoints**: REST/GraphQL endpoint identification
- **Data Models**: Entity relationships and business object patterns
- **Business Rules**: Validation logic and business constraint patterns
- **Integration Points**: External service calls and third-party integrations

## Framework-Specific Analysis Approaches

### Spring Boot Deep Analysis

**Annotation-Driven Analysis:**
- `@Entity` classes → Data model understanding
- `@Service` classes → Business logic identification  
- `@RestController` → API surface area mapping
- `@Configuration` → Integration and setup patterns

**Business Domain Inference:**
- JPA relationship patterns indicate domain complexity
- Service method signatures reveal business operations
- Controller endpoint groupings show feature boundaries

### React Component Analysis

**Component Hierarchy Mapping:**
- Route components → Application structure understanding
- Business components → Feature identification
- Utility components → Cross-cutting concern identification

**State Management Patterns:**
- Redux patterns → Complex business state management
- Context usage → Domain boundary identification
- Props flow → Data dependency analysis

### NestJS Module Analysis

**Dependency Injection Analysis:**
- Module organization → Domain boundary identification
- Provider relationships → Business logic flow understanding
- Guard implementations → Security and access patterns

## Performance Optimization

### Caching Strategy

**Framework Detection Cache:**
- **Key**: Project path + framework type + file modification times
- **Value**: Detection results with confidence scores
- **TTL**: 24 hours or until file system changes detected

**Pattern Matching Optimization:**
- **Regex Compilation**: Pre-compile frequently used patterns
- **File Filtering**: Skip binary files and common non-source directories
- **Parallel Processing**: Concurrent analysis of multiple files

### Memory Management

**Streaming Analysis:**
- Process files individually to avoid loading entire codebase
- AST tree disposal after pattern extraction
- Incremental confidence scoring updates

## Future Enhancements

### Planned Framework Support
- **Vue.js** with composition API analysis
- **Svelte/SvelteKit** component and store patterns
- **Express.js** middleware and routing analysis
- **FastAPI** automatic API documentation patterns
- **Ruby on Rails** convention-over-configuration analysis

### Advanced Business Domain Detection
- **Machine Learning Integration**: Pattern learning from analyzed codebases  
- **Domain-Specific Language Support**: DSL pattern recognition
- **Microservice Architecture**: Cross-service business flow analysis
- **Event-Driven Patterns**: Event sourcing and CQRS pattern detection

---

*This framework analysis methodology provides the foundation for accurate business domain classification and intelligent workflow generation.*