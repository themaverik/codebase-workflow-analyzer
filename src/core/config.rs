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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CCPMTemplates {
    pub claude_md: CCPMTemplate,
    pub prd: CCPMTemplate,
    pub user_stories: CCPMTemplate,
    pub context: CCPMContextTemplates,
    pub common: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CCPMTemplate {
    pub header: String,
    pub sections: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CCPMContextTemplates {
    pub product: HashMap<String, String>,
    pub tech: HashMap<String, String>,
    pub structure: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStoryTemplates {
    pub domain_stories: HashMap<String, UserStoryTemplate>,
    pub story_template: String,
    pub user_personas: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStoryTemplate {
    pub actor: String,
    pub want: String,
    pub so_that: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkCommands {
    pub framework_commands: HashMap<String, HashMap<String, String>>,
    pub command_sections: HashMap<String, CommandSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSection {
    pub header: String,
    #[serde(default)]
    pub footer: Option<String>,
    #[serde(default)]
    pub template: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMPromptTemplates {
    pub domain_analysis_prompts: HashMap<String, DomainPrompt>,
    pub context_aware_prompts: HashMap<String, String>,
    pub validation_prompts: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainPrompt {
    pub system_prompt: String,
    pub analysis_focus: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplates {
    pub agents: HashMap<String, HashMap<String, String>>,
    pub commands: HashMap<String, String>,
    pub workflow_readme: HashMap<String, String>,
    pub implementation_epic: HashMap<String, String>,
    pub task_template: HashMap<String, String>,
}

pub struct ConfigManager {
    project_types: HashMap<String, ProjectTypeConfig>,
    frameworks: HashMap<String, FrameworkConfig>,
    business_domains: HashMap<String, BusinessDomainConfig>,
    ccpm_templates: CCPMTemplates,
    user_story_templates: UserStoryTemplates,
    framework_commands: FrameworkCommands,
    llm_prompt_templates: LLMPromptTemplates,
    workflow_templates: WorkflowTemplates,
    document_templates: CCPMTemplates,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let config_path = Path::new("configs/data");
        
        let project_types = Self::load_project_types(config_path)?;
        let frameworks = Self::load_frameworks(config_path)?;
        let business_domains = Self::load_business_domains(config_path)?;
        let ccpm_templates = Self::load_ccpm_templates(config_path)?;
        let user_story_templates = Self::load_user_story_templates(config_path)?;
        let framework_commands = Self::load_framework_commands(config_path)?;
        let llm_prompt_templates = Self::load_llm_prompt_templates(config_path)?;
        let workflow_templates = Self::load_workflow_templates(config_path)?;
        let document_templates = Self::load_document_templates(config_path)?;

        Ok(ConfigManager {
            project_types,
            frameworks,
            business_domains,
            ccpm_templates,
            user_story_templates,
            framework_commands,
            llm_prompt_templates,
            workflow_templates,
            document_templates,
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

    fn load_ccpm_templates(config_path: &Path) -> Result<CCPMTemplates> {
        let file_path = config_path.join("ccpm_templates.json");
        let content = fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read CCPM templates file: {:?}", file_path))?;
        
        serde_json::from_str(&content)
            .with_context(|| "Failed to parse CCPM templates JSON")
    }

    fn load_user_story_templates(config_path: &Path) -> Result<UserStoryTemplates> {
        let file_path = config_path.join("user_story_templates.json");
        let content = fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read user story templates file: {:?}", file_path))?;
        
        serde_json::from_str(&content)
            .with_context(|| "Failed to parse user story templates JSON")
    }

    fn load_framework_commands(config_path: &Path) -> Result<FrameworkCommands> {
        let file_path = config_path.join("framework_commands.json");
        let content = fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read framework commands file: {:?}", file_path))?;
        
        serde_json::from_str(&content)
            .with_context(|| "Failed to parse framework commands JSON")
    }

    fn load_llm_prompt_templates(config_path: &Path) -> Result<LLMPromptTemplates> {
        let file_path = config_path.join("llm_prompt_templates.json");
        let content = fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read LLM prompt templates file: {:?}", file_path))?;
        
        serde_json::from_str(&content)
            .with_context(|| "Failed to parse LLM prompt templates JSON")
    }

    fn load_workflow_templates(config_path: &Path) -> Result<WorkflowTemplates> {
        let file_path = config_path.join("workflow_templates.json");
        let content = fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read workflow templates file: {:?}", file_path))?;
        
        serde_json::from_str(&content)
            .with_context(|| "Failed to parse workflow templates JSON")
    }

    fn load_document_templates(config_path: &Path) -> Result<CCPMTemplates> {
        let file_path = config_path.join("document_templates.json");
        let content = fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read document templates file: {:?}", file_path))?;
        
        serde_json::from_str(&content)
            .with_context(|| "Failed to parse document templates JSON")
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

    pub fn get_ccpm_templates(&self) -> &CCPMTemplates {
        &self.ccpm_templates
    }

    pub fn get_user_story_templates(&self) -> &UserStoryTemplates {
        &self.user_story_templates
    }

    pub fn get_framework_commands(&self) -> &FrameworkCommands {
        &self.framework_commands
    }

    pub fn get_llm_prompt_templates(&self) -> &LLMPromptTemplates {
        &self.llm_prompt_templates
    }

    pub fn get_workflow_templates(&self) -> &WorkflowTemplates {
        &self.workflow_templates
    }

    pub fn get_document_templates(&self) -> &CCPMTemplates {
        &self.document_templates
    }
}

// Global singleton instance
static CONFIG_MANAGER: Lazy<ConfigManager> = Lazy::new(|| {
    ConfigManager::new().expect("Failed to initialize ConfigManager")
});

pub fn get_config() -> &'static ConfigManager {
    &CONFIG_MANAGER
}

// Simple Config struct for compatibility
#[derive(Debug, Clone)]
pub struct Config;

impl Config {
    pub fn instance() -> Self {
        Config
    }
    
    pub fn get_context_management_config(&self) -> serde_json::Value {
        let config_path = Path::new("configs/data/context_management.json");
        if let Ok(content) = fs::read_to_string(config_path) {
            serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
        } else {
            serde_json::json!({})
        }
    }
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