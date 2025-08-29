# Framework Analysis Approach for Codebase Workflow Analyzer

## Overview

This document outlines our comprehensive approach to analyzing different web frameworks and extracting business domain information from codebases. Based on research of modern framework architectures and 2024 best practices, we follow data flow patterns to understand business logic and generate meaningful user stories.

## 🎯 Core Philosophy

### Data Flow-First Analysis
Following the principle of "information flows from end user to persistence layer", we analyze each framework's unique data flow pattern to understand:
1. How requests are processed
2. Where business logic resides  
3. What patterns indicate specific business domains
4. How data transformations occur throughout the stack

### Domain-Driven Detection
Using 2024 best practices in domain-driven design (DDD), we:
- Extract business domains from architectural patterns
- Weight evidence from multiple layers of the application
- Support multi-domain services (e.g., Authentication + User Management)
- Apply confidence thresholds for domain classification

## 🏗️ Framework-Specific Data Flow Patterns

### 1. Flask (Python) - Minimalist Web Framework

**Data Flow:**
```
HTTP Request → WSGI → Flask App → Blueprint → Route Handler → Business Logic → Database
     ↓           ↓        ↓          ↓            ↓              ↓              ↓
  Raw Request  Server   App Inst.  Modular    View Function   Service/Model   SQLAlchemy
```

**Analysis Strategy:**
- **Route Analysis**: Extract business intent from `@app.route()` and `@blueprint.route()` decorators
- **View Functions**: Analyze request handlers for CRUD patterns
- **Service Layer**: Optional but common pattern for business logic separation
- **Model Analysis**: SQLAlchemy models indicate data domains
- **Blueprint Organization**: Modular structure reveals feature domains

**Key Patterns:**
```python
@app.route('/api/users', methods=['POST'])          # User management
@auth_bp.route('/login', methods=['POST'])          # Authentication  
@property_bp.route('/search', methods=['GET'])      # Property management
```

### 2. FastAPI (Python) - Modern Async API Framework

**Data Flow:**
```
HTTP Request → Starlette → FastAPI Router → Dependencies → Path Operation → Pydantic → Service → Repository → Database
     ↓            ↓           ↓               ↓              ↓                ↓          ↓          ↓            ↓
  Raw Request   ASGI      Route Matching   Validation    Handler Function  Serialization Business  Data Access  AsyncDB
```

**Analysis Strategy:**
- **Router Analysis**: Extract business domains from `@router.post()` path patterns
- **Dependency Injection**: Analyze `Depends()` patterns for architectural layers
- **Pydantic Models**: Strong typing indicates data domains and validation logic
- **Async Patterns**: Service layer with `async/await` for business logic
- **Middleware**: Cross-cutting concerns like authentication and logging

**Key Patterns:**
```python
@router.post("/auth/login", response_model=TokenResponse)     # Authentication
@router.get("/users/", dependencies=[Depends(get_current_user)]) # User management
class UserCreate(BaseModel):                                 # Data domain
    email: EmailStr
```

### 3. React (TypeScript) - Frontend Component Framework

**Data Flow:**
```
User Event → Component → Local State → Context/Store → Custom Hook → API Service → Backend
     ↓           ↓           ↓             ↓              ↓             ↓             ↓
  UI Action   React Comp   useState      Global State   Business     HTTP Client   External API
                                        useContext      Logic        (Axios/Fetch)
```

**Analysis Strategy:**
- **Component Analysis**: Extract business domains from component names and structure
- **Hook Patterns**: `useState`, `useEffect`, and custom hooks reveal business logic
- **Context API**: Global state management indicates application domains
- **Service Layer**: API communication patterns show backend integration
- **TypeScript Types**: Interface definitions reveal data structures

**Key Patterns:**
```typescript
const LoginForm: React.FC = () => {...}              # Authentication UI
const useAuth = () => {...}                          # Authentication logic
const UserProfile: React.FC = () => {...}           # User management UI
const AuthContext = createContext<AuthContextType>() # Global auth state
```

### 4. NestJS (TypeScript) - Enterprise Node.js Framework

**Data Flow:**
```
HTTP Request → Middleware → Guard → Interceptor → Controller → Pipe → DTO → Service → Repository → Entity → Database
     ↓            ↓          ↓        ↓            ↓           ↓       ↓      ↓          ↓            ↓         ↓
  Raw Request   Logging    Auth      Transform    Route      Validation Transform Business   Data Access  ORM Model  TypeORM/Prisma
               CORS       Check     Response     Handler    Parse       Logic      Logic
```

**Analysis Strategy:**
- **Decorator Analysis**: Extract business domains from `@Controller()`, `@Get()`, `@Post()` patterns
- **Guard Patterns**: `@UseGuards()` indicates authentication/authorization domains
- **DTO Analysis**: Data Transfer Objects with validation decorators show data domains
- **Service Layer**: `@Injectable()` services contain core business logic
- **Repository Pattern**: Data access layer with ORM integration
- **Module Organization**: `@Module()` structure reveals feature boundaries

**Key Patterns:**
```typescript
@Controller('auth')                    # Authentication domain
@UseGuards(JwtAuthGuard)              # Authentication patterns
@Post('login')                        # Specific business operations
export class AuthService {...}        # Business logic services
export class User {...}               # Domain entities
```

### 5. Danet (Deno) - NestJS-like Framework for Deno

**Data Flow:**
```
HTTP Request → Deno Runtime → Danet Router → Guard → Controller → DTO → Service → Repository → Entity → Database
     ↓            ↓              ↓             ↓        ↓           ↓      ↓          ↓            ↓         ↓
  Raw Request   Deno HTTP    Route Matching   Auth    Route      Validation Business  Data Access  Domain    External DB
               Oak/Hono                       Check   Handler    Transform  Logic                   Model     (MongoDB/PostgreSQL)
```

**Analysis Strategy:**
- **NestJS-like Patterns**: Similar decorator-based analysis but adapted for Deno runtime
- **ESM Imports**: Analyze URL-based imports for framework detection
- **Native TypeScript**: No compilation step analysis
- **Direct Database Clients**: Less ORM abstraction, more direct database patterns

**Key Patterns:**
```typescript
import { Controller } from "https://deno.land/x/danet/mod.ts";  # Framework detection
@Controller('users')                                           # Business domain
export class UsersService {...}                               # Business logic
```

### 6. Spring Boot (Java) - Enterprise Java Framework

**Data Flow:**
```
HTTP Request → Servlet Container → DispatcherServlet → Handler Mapping → Controller → Service → Repository → Entity → Database
     ↓              ↓                    ↓                ↓                ↓           ↓          ↓            ↓         ↓
  Raw Request    Tomcat/Jetty        Spring MVC        Route Matching   @RestController @Service   @Repository   JPA Entity  JPA/Hibernate
                                     Framework                          Method         Bean       Bean         Bean
```

**Analysis Strategy:**
- **Annotation Analysis**: Extract business domains from `@RestController`, `@RequestMapping`
- **Spring Security**: `@PreAuthorize` indicates authentication/authorization patterns
- **Service Layer**: `@Service` and `@Transactional` for business logic
- **Repository Pattern**: `@Repository` with JPA for data access
- **Entity Analysis**: JPA entities with `@Entity` for domain models
- **Configuration**: `@Configuration` classes reveal architectural patterns

**Key Patterns:**
```java
@RestController
@RequestMapping("/api/auth")           # Authentication domain
@PreAuthorize("hasRole('ADMIN')")     # Authorization patterns
@Service                              # Business logic layer
@Repository                           # Data access layer
@Entity                              # Domain models
```

### 7. Next.js (React/TypeScript) - Full-Stack React Framework

**Data Flow (SSR/SSG):**
```
HTTP Request → Next.js Server → Page/API Route → getServerSideProps/getStaticProps → Component → Client Hydration
     ↓              ↓                ↓                        ↓                          ↓              ↓
  Raw Request    Node.js Server   Route Handler           Data Fetching              React Comp    Client State
```

**Data Flow (Client-Side):**
```
User Event → Next.js Client → React Component → Custom Hook → API Route/External API → Server Action
     ↓            ↓                ↓               ↓              ↓                        ↓
  UI Action    Client Router    Component State   Business      Internal/External       Database
                                                 Logic         API Call
```

**Analysis Strategy:**
- **File-based Routing**: Extract business domains from route structure
- **API Routes**: Analyze `/api/` directory for backend functionality patterns
- **Server Components**: SSR patterns indicate data fetching domains
- **Client Components**: Interactive UI patterns
- **Server Actions**: Form handling and server-side business logic

**Key Patterns:**
```typescript
// app/api/auth/login/route.ts        # Authentication API
// app/users/[id]/page.tsx            # User management page
'use server';                        # Server action
export default async function UserPage() {...}  # Server component
```

## 🎯 Unified Analysis Strategy

### 1. Framework Detection
```rust
pub struct FrameworkDetector {
    pub fn detect_framework(&self, codebase: &Codebase) -> Vec<DetectedFramework> {
        // Multi-signal detection
        let config_signals = self.analyze_config_files(&codebase);
        let import_signals = self.analyze_imports(&codebase);
        let file_structure_signals = self.analyze_file_patterns(&codebase);
        let content_signals = self.analyze_code_patterns(&codebase);
        
        self.combine_detection_signals(config_signals, import_signals, file_structure_signals, content_signals)
    }
}
```

### 2. Layer-by-Layer Analysis
```rust
pub struct DataFlowAnalyzer {
    pub fn analyze_service_architecture(&self, components: &[Component], framework: &DetectedFramework) -> ServiceArchitecture {
        match framework {
            Framework::NestJS => self.analyze_nestjs_layers(components),
            Framework::FastAPI => self.analyze_fastapi_layers(components),
            Framework::Flask => self.analyze_flask_layers(components),
            Framework::React => self.analyze_react_layers(components),
            Framework::Danet => self.analyze_danet_layers(components),
            Framework::SpringBoot => self.analyze_spring_layers(components),
            Framework::NextJS => self.analyze_nextjs_layers(components),
        }
    }
}
```

### 3. Business Domain Inference with Confidence Scoring
```rust
pub struct DomainInferenceEngine {
    pub fn infer_business_domains(&self, architecture: &ServiceArchitecture, framework: &DetectedFramework) -> Vec<BusinessDomainResult> {
        let mut domain_scores = HashMap::new();
        
        // Framework-specific scoring weights
        match framework {
            Framework::NestJS => {
                self.score_from_controllers(&architecture.controllers, &mut domain_scores, 5);
                self.score_from_services(&architecture.services, &mut domain_scores, 4);
                self.score_from_entities(&architecture.entities, &mut domain_scores, 3);
                self.score_from_dtos(&architecture.dtos, &mut domain_scores, 2);
            },
            Framework::FastAPI => {
                self.score_from_routers(&architecture.routers, &mut domain_scores, 5);
                self.score_from_pydantic_models(&architecture.models, &mut domain_scores, 4);
                self.score_from_dependencies(&architecture.dependencies, &mut domain_scores, 3);
            },
            // ... other frameworks
        }
        
        self.apply_confidence_thresholds(domain_scores)
    }
}
```

### 4. Confidence-Based Domain Classification
```rust
pub struct ConfidenceThresholds {
    pub high_confidence: f32 = 0.8,      // Generate comprehensive user stories
    pub medium_confidence: f32 = 0.6,    // Generate core user stories with caveats  
    pub low_confidence: f32 = 0.4,       // Mention in analysis but don't prioritize
}

pub struct BusinessDomainResult {
    pub domain: BusinessDomain,
    pub confidence: f32,
    pub evidence: Vec<Evidence>,
    pub story_generation_strategy: StoryStrategy,
}
```

## 🔍 Evidence Patterns by Domain

### Authentication Domain
- **Route Patterns**: `/auth/login`, `/auth/register`, `/auth/logout`
- **Service Names**: `AuthService`, `AuthenticationService`, `LoginService`
- **Model/DTO Names**: `LoginDto`, `AuthUser`, `Token`, `Credentials`
- **Method Names**: `authenticate()`, `login()`, `validateToken()`
- **Security Patterns**: Guards, middleware, JWT tokens

### User Management Domain
- **Route Patterns**: `/users`, `/profile`, `/user/{id}`
- **Service Names**: `UserService`, `ProfileService`, `UserManagementService`
- **Model/DTO Names**: `User`, `UserProfile`, `CreateUserDto`
- **Method Names**: `createUser()`, `updateProfile()`, `findUserById()`
- **CRUD Patterns**: Complete user lifecycle operations

### E-commerce Domain
- **Route Patterns**: `/products`, `/cart`, `/orders`, `/checkout`
- **Service Names**: `ProductService`, `CartService`, `OrderService`
- **Model Names**: `Product`, `Cart`, `Order`, `Payment`
- **Method Names**: `addToCart()`, `processPayment()`, `fulfillOrder()`

## 📊 Multi-Domain Support Strategy

### Scoring Combination
```rust
impl DomainInferenceEngine {
    fn combine_domain_evidence(&self, evidence: Vec<DomainEvidence>) -> Vec<BusinessDomainResult> {
        let mut combined_scores = HashMap::new();
        
        // Aggregate evidence across all layers
        for evidence_item in evidence {
            let entry = combined_scores.entry(evidence_item.domain).or_insert(DomainScore::new());
            entry.add_evidence(evidence_item);
        }
        
        // Handle overlapping domains (e.g., Auth + User Management)
        self.resolve_domain_relationships(combined_scores)
    }
}
```

### Relationship Handling
- **Complementary Domains**: Authentication + User Management (common pattern)
- **Overlapping Domains**: User Management + Profile Management (merge strategy)
- **Conflicting Domains**: E-commerce + Blog (separate services or monolith detection)

## 🧪 Validation Approach

### Framework Detection Validation

#### Language-Aware Detection Strategy
```rust
pub struct FrameworkDetector {
    pub fn detect_framework(&self, codebase: &Codebase) -> Vec<DetectedFramework> {
        // Step 1: Detect primary language ecosystem
        let language_ecosystem = self.detect_language_ecosystem(&codebase);
        
        // Step 2: Apply language-specific framework detection
        match language_ecosystem {
            LanguageEcosystem::Python => self.detect_python_frameworks(&codebase),
            LanguageEcosystem::JavaScript => self.detect_js_frameworks(&codebase),
            LanguageEcosystem::TypeScript => self.detect_ts_frameworks(&codebase),
            LanguageEcosystem::Java => self.detect_java_frameworks(&codebase),
            LanguageEcosystem::Deno => self.detect_deno_frameworks(&codebase),
            LanguageEcosystem::Mixed => self.detect_mixed_frameworks(&codebase),
        }
    }
}
```

#### 1. Python Framework Detection

**Config Files:**
- `requirements.txt` - pip dependencies
- `pyproject.toml` - modern Python project config
- `Pipfile` - pipenv dependencies  
- `poetry.lock` - poetry dependencies
- `setup.py` - package setup
- `environment.yml` - conda environment

**Flask Detection:**
```python
# requirements.txt patterns
Flask==2.3.0
flask-login>=0.6.0
Flask-SQLAlchemy

# Import patterns
from flask import Flask, request, jsonify
from flask_sqlalchemy import SQLAlchemy
import flask

# File structure
app.py or main.py (entry point)
/templates/ (Jinja2 templates)
/static/ (CSS/JS assets)
/migrations/ (Flask-Migrate)

# Content patterns
app = Flask(__name__)
@app.route('/')
@bp.route('/users')
```

**FastAPI Detection:**
```python
# requirements.txt patterns  
fastapi>=0.104.0
uvicorn[standard]>=0.24.0
pydantic>=2.0.0

# Import patterns
from fastapi import FastAPI, Depends, HTTPException
from pydantic import BaseModel
import fastapi

# File structure
main.py (entry point)
/app/ (application package)
/models/ (Pydantic models)
/routers/ (API routes)
/schemas/ (request/response schemas)

# Content patterns
app = FastAPI()
@app.post("/")
@router.get("/users/")
class UserCreate(BaseModel):
```

#### 2. JavaScript/TypeScript Framework Detection

**Config Files:**
- `package.json` - npm/yarn dependencies and scripts
- `package-lock.json` - npm lock file
- `yarn.lock` - yarn lock file
- `tsconfig.json` - TypeScript configuration
- `next.config.js` - Next.js configuration
- `nest-cli.json` - NestJS CLI configuration

**React Detection:**
```json
// package.json dependencies
{
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "@types/react": "^18.2.0"
  }
}

// Import patterns
import React from 'react';
import { useState, useEffect } from 'react';
import * as React from 'react';

// File structure
/src/components/
/src/hooks/
/src/context/
/public/
index.html

// Content patterns
const Component: React.FC = () => {...}
function Component() {...}
export default function Component() {...}
```

**NestJS Detection:**
```json
// package.json dependencies
{
  "dependencies": {
    "@nestjs/core": "^10.0.0",
    "@nestjs/common": "^10.0.0",
    "@nestjs/platform-express": "^10.0.0"
  }
}

// Import patterns
import { Controller, Get, Post } from '@nestjs/common';
import { Injectable } from '@nestjs/common';

// File structure
/src/
nest-cli.json
main.ts (entry point)
/src/modules/
/src/controllers/
/src/services/

// Content patterns
@Controller()
@Injectable()
@Module({})
```

**Next.js Detection:**
```json
// package.json dependencies
{
  "dependencies": {
    "next": "^14.0.0",
    "react": "^18.2.0",
    "react-dom": "^18.2.0"
  }
}

// File structure
/pages/ or /app/ (routing)
/public/
next.config.js
/components/
/api/ (API routes)

// Content patterns
export default function Page() {...}
export async function getServerSideProps() {...}
'use client'
'use server'
```

#### 3. Java Framework Detection

**Config Files:**
- `pom.xml` - Maven project configuration
- `build.gradle` - Gradle build configuration
- `application.properties` - Spring Boot properties
- `application.yml` - Spring Boot YAML config

**Spring Boot Detection:**
```xml
<!-- pom.xml dependencies -->
<dependency>
    <groupId>org.springframework.boot</groupId>
    <artifactId>spring-boot-starter-web</artifactId>
</dependency>
<dependency>
    <groupId>org.springframework.boot</groupId>
    <artifactId>spring-boot-starter-data-jpa</artifactId>
</dependency>

<!-- Import patterns -->
import org.springframework.boot.SpringApplication;
import org.springframework.web.bind.annotation.RestController;
import org.springframework.stereotype.Service;

<!-- File structure -->
src/main/java/
src/main/resources/
Application.java (main class)
/controller/
/service/
/repository/
/entity/

<!-- Content patterns -->
@SpringBootApplication
@RestController
@Service
@Repository
@Entity
```

#### 4. Deno Framework Detection

**Config Files:**
- `deno.json` - Deno configuration
- `deno.lock` - Deno lock file
- `import_map.json` - Import maps

**Danet Detection:**
```json
// deno.json dependencies
{
  "imports": {
    "danet": "https://deno.land/x/danet@2.0.0/mod.ts"
  }
}

// Import patterns (URL-based)
import { Controller, Get } from "https://deno.land/x/danet/mod.ts";
import { Injectable } from "https://deno.land/x/danet/common.ts";

// File structure
main.ts (entry point)
deno.json
/src/
/controllers/
/services/

// Content patterns
@Controller()
@Injectable()  
@Module({})
```

#### 5. Multi-Signal Detection Algorithm

```rust
impl FrameworkDetector {
    fn detect_python_frameworks(&self, codebase: &Codebase) -> Vec<DetectedFramework> {
        let mut frameworks = Vec::new();
        
        // Config file analysis
        if let Some(requirements) = codebase.find_file("requirements.txt") {
            if requirements.contains("Flask") {
                frameworks.push(self.analyze_flask_confidence(&codebase));
            }
            if requirements.contains("fastapi") {
                frameworks.push(self.analyze_fastapi_confidence(&codebase));
            }
        }
        
        if let Some(pyproject) = codebase.find_file("pyproject.toml") {
            frameworks.extend(self.analyze_pyproject_dependencies(&pyproject));
        }
        
        // Import analysis
        let python_files = codebase.find_files_by_extension(".py");
        for file in python_files {
            frameworks.extend(self.analyze_python_imports(&file));
        }
        
        // File structure analysis
        frameworks.extend(self.analyze_python_file_structure(&codebase));
        
        self.consolidate_framework_detection(frameworks)
    }
    
    fn analyze_flask_confidence(&self, codebase: &Codebase) -> DetectedFramework {
        let mut confidence = 0.0;
        let mut evidence = Vec::new();
        
        // Config evidence
        if codebase.has_dependency("Flask") {
            confidence += 0.3;
            evidence.push("Flask dependency found");
        }
        
        // Import evidence
        if codebase.has_import_pattern("from flask import") {
            confidence += 0.2;
            evidence.push("Flask imports found");
        }
        
        // Structure evidence
        if codebase.has_file("app.py") || codebase.has_directory("templates") {
            confidence += 0.2;
            evidence.push("Flask file structure");
        }
        
        // Content evidence
        if codebase.has_pattern("@app.route") || codebase.has_pattern("Flask(__name__)") {
            confidence += 0.3;
            evidence.push("Flask decorators/patterns found");
        }
        
        DetectedFramework {
            name: Framework::Flask,
            confidence,
            evidence,
        }
    }
}
```

#### 6. Language Ecosystem Priority

```rust
pub enum LanguageEcosystem {
    Python,      // .py files, requirements.txt, pyproject.toml
    JavaScript,  // .js files, package.json (no TypeScript)
    TypeScript,  // .ts/.tsx files, tsconfig.json, package.json
    Java,        // .java files, pom.xml, build.gradle
    Deno,        // .ts files with URL imports, deno.json
    Mixed,       // Multiple ecosystems detected
}

impl FrameworkDetector {
    fn detect_language_ecosystem(&self, codebase: &Codebase) -> LanguageEcosystem {
        let mut scores = HashMap::new();
        
        // File extension analysis
        let file_counts = codebase.count_files_by_extension();
        
        if file_counts.get(".py").unwrap_or(&0) > &0 {
            scores.insert(LanguageEcosystem::Python, file_counts.get(".py").unwrap_or(&0));
        }
        
        if file_counts.get(".ts").unwrap_or(&0) > &0 || file_counts.get(".tsx").unwrap_or(&0) > &0 {
            if codebase.has_file("deno.json") || codebase.has_url_imports() {
                scores.insert(LanguageEcosystem::Deno, file_counts.get(".ts").unwrap_or(&0));
            } else {
                scores.insert(LanguageEcosystem::TypeScript, 
                    file_counts.get(".ts").unwrap_or(&0) + file_counts.get(".tsx").unwrap_or(&0));
            }
        }
        
        if file_counts.get(".js").unwrap_or(&0) > &0 && !codebase.has_file("tsconfig.json") {
            scores.insert(LanguageEcosystem::JavaScript, file_counts.get(".js").unwrap_or(&0));
        }
        
        if file_counts.get(".java").unwrap_or(&0) > &0 {
            scores.insert(LanguageEcosystem::Java, file_counts.get(".java").unwrap_or(&0));
        }
        
        // Return primary ecosystem or Mixed
        if scores.len() > 1 {
            LanguageEcosystem::Mixed
        } else {
            scores.into_keys().next().unwrap_or(LanguageEcosystem::Mixed)
        }
    }
}
```

#### 7. Framework-Specific File Structure Patterns

| Framework | Config Files | Key Directories | Entry Points | Unique Patterns |
|-----------|-------------|-----------------|--------------|-----------------|
| **Flask** | `requirements.txt`, `pyproject.toml` | `/templates`, `/static`, `/migrations` | `app.py`, `main.py` | `@app.route`, `Flask(__name__)` |
| **FastAPI** | `requirements.txt`, `pyproject.toml` | `/app`, `/routers`, `/schemas`, `/models` | `main.py` | `FastAPI()`, `@router.get`, `BaseModel` |
| **React** | `package.json`, `tsconfig.json` | `/src`, `/components`, `/hooks`, `/public` | `index.html`, `App.jsx/tsx` | `React.FC`, `useState`, `useEffect` |
| **NestJS** | `package.json`, `nest-cli.json`, `tsconfig.json` | `/src`, `/modules`, `/controllers`, `/services` | `main.ts` | `@Controller`, `@Injectable`, `@Module` |
| **Danet** | `deno.json`, `import_map.json` | `/src`, `/controllers`, `/services` | `main.ts` | URL imports, `@Controller` |
| **Spring Boot** | `pom.xml`, `application.properties` | `/src/main/java`, `/controller`, `/service`, `/repository` | `Application.java` | `@SpringBootApplication`, `@RestController` |
| **Next.js** | `package.json`, `next.config.js` | `/pages` or `/app`, `/public`, `/components` | `_app.js/tsx`, `index.js/tsx` | File-based routing, `getServerSideProps` |

### Business Domain Validation
1. **Cross-Layer Verification**: Evidence from multiple architectural layers
2. **Pattern Consistency**: Consistent naming and organization patterns
3. **Confidence Thresholds**: Statistical validation of domain classification
4. **Multi-Domain Logic**: Handling services with multiple business domains

### Performance Considerations
1. **Lazy Loading**: Analyze only necessary files for initial detection
2. **Caching**: Cache analysis results for repeated operations
3. **Parallel Processing**: Analyze multiple files concurrently
4. **Progressive Analysis**: Start with high-confidence patterns, expand as needed

## 🚀 Implementation Priorities

### Phase 1: Core Framework Detection
1. Implement multi-signal framework detection
2. Create framework-specific analyzers for NestJS and FastAPI
3. Build basic business domain inference engine
4. Establish confidence scoring system

### Phase 2: Full Framework Support
1. Add React and Next.js frontend analysis
2. Implement Spring Boot and Flask support
3. Add Danet framework support
4. Create multi-domain handling logic

### Phase 3: Advanced Features
1. Cross-framework integration detection
2. Microservice relationship mapping  
3. Advanced business domain relationships
4. Performance optimizations

## 📝 Output Generation Strategy

### User Story Generation
- **High Confidence Domains**: Generate detailed user stories with acceptance criteria
- **Medium Confidence Domains**: Generate core user stories with uncertainty notes
- **Low Confidence Domains**: Mention in analysis summary only

### PRD Generation
- **Primary Domain**: Focus PRD on highest confidence business domain
- **Secondary Domains**: Include as additional features or modules
- **Technical Architecture**: Document detected framework patterns and data flows

### Technical Documentation
- **Framework Analysis**: Document detected frameworks and confidence levels
- **Data Flow Diagrams**: Visual representation of framework-specific flows
- **Business Logic Mapping**: Connect code patterns to business requirements

This comprehensive approach ensures accurate framework detection, meaningful business domain extraction, and generation of valuable development workflows from existing codebases.