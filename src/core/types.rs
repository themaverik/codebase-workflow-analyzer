use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Language {
    TypeScript,
    JavaScript,
    Python,
    Java,
    Rust,
    Go,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Framework {
    // JavaScript/TypeScript
    React,
    NextJS,
    NestJS,
    Express,
    Vue,
    Angular,
    
    // Python
    Flask,
    FastAPI,
    Django,
    
    // Java
    SpringBoot,
    Quarkus,
    
    // Deno
    Danet,
    
    // Rust
    Axum,
    Warp,
    Actix,
    
    // Go
    Gin,
    Fiber,
    
    // Cross-platform
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LanguageEcosystem {
    JavaScript,
    TypeScript,
    Python,
    Java,
    Rust,
    Go,
    Deno,
    Mixed,
}

impl Framework {
    pub fn language(&self) -> Language {
        match self {
            Framework::React | Framework::NextJS | Framework::NestJS | 
            Framework::Express | Framework::Vue | Framework::Angular => Language::TypeScript,
            
            Framework::Flask | Framework::FastAPI | Framework::Django => Language::Python,
            
            Framework::SpringBoot | Framework::Quarkus => Language::Java,
            
            Framework::Danet => Language::TypeScript, // Deno framework
            
            Framework::Axum | Framework::Warp | Framework::Actix => Language::Rust,
            
            Framework::Gin | Framework::Fiber => Language::Go,
            
            Framework::Unknown => Language::TypeScript, // Default fallback
        }
    }

    pub fn is_web_framework(&self) -> bool {
        matches!(self, 
            Framework::React | Framework::NextJS | Framework::NestJS | Framework::Express |
            Framework::Flask | Framework::FastAPI | Framework::Django |
            Framework::SpringBoot | Framework::Quarkus |
            Framework::Vue | Framework::Angular | Framework::Danet |
            Framework::Axum | Framework::Warp | Framework::Actix |
            Framework::Gin | Framework::Fiber
        )
    }

    pub fn is_frontend_framework(&self) -> bool {
        matches!(self, Framework::React | Framework::NextJS | Framework::Vue | Framework::Angular)
    }

    pub fn is_backend_framework(&self) -> bool {
        matches!(self, 
            Framework::NestJS | Framework::Express |
            Framework::Flask | Framework::FastAPI | Framework::Django |
            Framework::SpringBoot | Framework::Quarkus |
            Framework::Axum | Framework::Warp | Framework::Actix |
            Framework::Gin | Framework::Fiber
        )
    }
}

impl std::fmt::Display for Framework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Framework::React => write!(f, "React"),
            Framework::NextJS => write!(f, "Next.js"),
            Framework::NestJS => write!(f, "NestJS"),
            Framework::Express => write!(f, "Express"),
            Framework::Vue => write!(f, "Vue.js"),
            Framework::Angular => write!(f, "Angular"),
            Framework::Flask => write!(f, "Flask"),
            Framework::FastAPI => write!(f, "FastAPI"),
            Framework::Django => write!(f, "Django"),
            Framework::SpringBoot => write!(f, "Spring Boot"),
            Framework::Quarkus => write!(f, "Quarkus"),
            Framework::Danet => write!(f, "Danet"),
            Framework::Axum => write!(f, "Axum"),
            Framework::Warp => write!(f, "Warp"),
            Framework::Actix => write!(f, "Actix Web"),
            Framework::Gin => write!(f, "Gin"),
            Framework::Fiber => write!(f, "Fiber"),
            Framework::Unknown => write!(f, "Unknown"),
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::TypeScript => write!(f, "TypeScript"),
            Language::JavaScript => write!(f, "JavaScript"),
            Language::Python => write!(f, "Python"),
            Language::Java => write!(f, "Java"),
            Language::Rust => write!(f, "Rust"),
            Language::Go => write!(f, "Go"),
        }
    }
}

impl std::fmt::Display for LanguageEcosystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LanguageEcosystem::JavaScript => write!(f, "JavaScript"),
            LanguageEcosystem::TypeScript => write!(f, "TypeScript"),
            LanguageEcosystem::Python => write!(f, "Python"),
            LanguageEcosystem::Java => write!(f, "Java"),
            LanguageEcosystem::Rust => write!(f, "Rust"),
            LanguageEcosystem::Go => write!(f, "Go"),
            LanguageEcosystem::Deno => write!(f, "Deno"),
            LanguageEcosystem::Mixed => write!(f, "Mixed"),
        }
    }
}