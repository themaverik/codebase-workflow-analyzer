use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTypeConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub indicators: Vec<ProjectIndicator>,
    pub analysis_focus: String,
    pub domain_hints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectIndicator {
    pub r#type: String,
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkConfig {
    pub id: String,
    pub name: String,
    pub language: String,
    pub category: String,
    pub ecosystem: String,
    pub indicators: Vec<FrameworkIndicator>,
    #[serde(default)]
    pub deno_specific: Option<DenoSpecificConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DenoSpecificConfig {
    pub native_typescript: bool,
    pub builtin_tooling: Vec<String>,
    pub import_system: String,
    pub package_manager: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkIndicator {
    pub r#type: String,
    pub patterns: Vec<String>,
    pub confidence_weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessDomainConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub patterns: Vec<DomainPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainPattern {
    pub r#type: String,
    pub patterns: Vec<String>,
    pub confidence_weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectTypesData {
    project_types: Vec<ProjectTypeConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FrameworksData {
    frameworks: Vec<FrameworkConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BusinessDomainsData {
    business_domains: Vec<BusinessDomainConfig>,
}

pub struct ConfigManager {
    project_types: HashMap<String, ProjectTypeConfig>,
    frameworks: HashMap<String, FrameworkConfig>,
    business_domains: HashMap<String, BusinessDomainConfig>,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let config_path = Path::new("configs/data");
        
        let project_types = Self::load_project_types(config_path)?;
        let frameworks = Self::load_frameworks(config_path)?;
        let business_domains = Self::load_business_domains(config_path)?;

        Ok(ConfigManager {
            project_types,
            frameworks,
            business_domains,
        })
    }

    fn load_project_types(config_path: &Path) -> Result<HashMap<String, ProjectTypeConfig>> {
        let file_path = config_path.join("project_types.json");
        let content = fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read project types config from {:?}", file_path))?;
        
        let data: ProjectTypesData = serde_json::from_str(&content)
            .with_context(|| "Failed to parse project types JSON")?;

        let mut map = HashMap::new();
        for project_type in data.project_types {
            map.insert(project_type.id.clone(), project_type);
        }

        Ok(map)
    }

    fn load_frameworks(config_path: &Path) -> Result<HashMap<String, FrameworkConfig>> {
        let file_path = config_path.join("frameworks.json");
        let content = fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read frameworks config from {:?}", file_path))?;
        
        let data: FrameworksData = serde_json::from_str(&content)
            .with_context(|| "Failed to parse frameworks JSON")?;

        let mut map = HashMap::new();
        for framework in data.frameworks {
            map.insert(framework.id.clone(), framework);
        }

        Ok(map)
    }

    fn load_business_domains(config_path: &Path) -> Result<HashMap<String, BusinessDomainConfig>> {
        let file_path = config_path.join("business_domains.json");
        let content = fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read business domains config from {:?}", file_path))?;
        
        let data: BusinessDomainsData = serde_json::from_str(&content)
            .with_context(|| "Failed to parse business domains JSON")?;

        let mut map = HashMap::new();
        for domain in data.business_domains {
            map.insert(domain.id.clone(), domain);
        }

        Ok(map)
    }

    pub fn get_project_type(&self, id: &str) -> Option<&ProjectTypeConfig> {
        self.project_types.get(id)
    }

    pub fn get_all_project_types(&self) -> impl Iterator<Item = &ProjectTypeConfig> {
        self.project_types.values()
    }

    pub fn get_framework(&self, id: &str) -> Option<&FrameworkConfig> {
        self.frameworks.get(id)
    }

    pub fn get_all_frameworks(&self) -> impl Iterator<Item = &FrameworkConfig> {
        self.frameworks.values()
    }

    pub fn get_frameworks_by_ecosystem(&self, ecosystem: &str) -> Vec<&FrameworkConfig> {
        self.frameworks.values()
            .filter(|f| f.ecosystem == ecosystem)
            .collect()
    }

    pub fn get_business_domain(&self, id: &str) -> Option<&BusinessDomainConfig> {
        self.business_domains.get(id)
    }

    pub fn get_all_business_domains(&self) -> impl Iterator<Item = &BusinessDomainConfig> {
        self.business_domains.values()
    }

    pub fn find_frameworks_by_language(&self, language: &str) -> Vec<&FrameworkConfig> {
        self.frameworks.values()
            .filter(|f| f.language.eq_ignore_ascii_case(language))
            .collect()
    }

    pub fn find_project_types_by_indicator(&self, indicator_type: &str, pattern: &str) -> Vec<&ProjectTypeConfig> {
        self.project_types.values()
            .filter(|pt| {
                pt.indicators.iter().any(|indicator| {
                    indicator.r#type == indicator_type && 
                    indicator.patterns.iter().any(|p| pattern.contains(p))
                })
            })
            .collect()
    }
}

// Global singleton instance
static CONFIG_MANAGER: Lazy<ConfigManager> = Lazy::new(|| {
    ConfigManager::new().expect("Failed to initialize ConfigManager")
});

pub fn get_config() -> &'static ConfigManager {
    &CONFIG_MANAGER
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_manager_initialization() {
        let config = ConfigManager::new();
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert!(!config.project_types.is_empty());
        assert!(!config.frameworks.is_empty());
        assert!(!config.business_domains.is_empty());
    }

    #[test]
    fn test_framework_lookup() {
        let config = get_config();
        let react = config.get_framework("react");
        assert!(react.is_some());
        assert_eq!(react.unwrap().name, "React");
        
        let danet = config.get_framework("danet");
        assert!(danet.is_some());
        assert_eq!(danet.unwrap().ecosystem, "Deno");
        assert!(danet.unwrap().deno_specific.is_some());
    }

    #[test]
    fn test_project_type_lookup() {
        let config = get_config();
        let analysis_tool = config.get_project_type("analysis_tool");
        assert!(analysis_tool.is_some());
        assert!(analysis_tool.unwrap().name.contains("Analysis"));
    }

    #[test]
    fn test_business_domain_lookup() {
        let config = get_config();
        let auth = config.get_business_domain("authentication");
        assert!(auth.is_some());
        assert!(auth.unwrap().keywords.contains(&"auth".to_string()));
    }
}