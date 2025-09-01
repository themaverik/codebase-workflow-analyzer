use std::collections::HashMap;
use anyhow::Result;
use crate::core::config::get_config;

pub struct TemplateEngine {
    variables: HashMap<String, String>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn set_variable(&mut self, key: String, value: String) {
        self.variables.insert(key, value);
    }

    pub fn set_variables(&mut self, vars: HashMap<String, String>) {
        self.variables.extend(vars);
    }

    pub fn render(&self, template: &str) -> String {
        let mut result = template.to_string();
        
        // Replace variables in the format {variable_name}
        for (key, value) in &self.variables {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }
        
        result
    }

    pub fn render_ccpm_document(&self, document_type: &str, section: Option<&str>) -> Result<String> {
        let config = get_config();
        let templates = config.get_ccpm_templates();
        
        let template_str = match document_type {
            "claude_md" => {
                if let Some(section_name) = section {
                    templates.claude_md.sections.get(section_name)
                        .map(|s| s.as_str())
                        .unwrap_or("")
                } else {
                    &templates.claude_md.header
                }
            },
            "prd" => {
                if let Some(section_name) = section {
                    templates.prd.sections.get(section_name)
                        .map(|s| s.as_str())
                        .unwrap_or("")
                } else {
                    &templates.prd.header
                }
            },
            "user_stories" => {
                if let Some(section_name) = section {
                    templates.user_stories.sections.get(section_name)
                        .map(|s| s.as_str())
                        .unwrap_or("")
                } else {
                    &templates.user_stories.header
                }
            },
            _ => return Ok(String::new()),
        };
        
        Ok(self.render(template_str))
    }

    pub fn render_user_story(&self, domain_name: &str) -> Result<String> {
        let config = get_config();
        let templates = config.get_user_story_templates();
        
        let story_config = templates.domain_stories.get(domain_name)
            .unwrap_or(templates.domain_stories.get("default").unwrap());
            
        let mut engine = TemplateEngine::new();
        engine.set_variable("actor".to_string(), story_config.actor.clone());
        engine.set_variable("want".to_string(), story_config.want.clone());
        engine.set_variable("so_that".to_string(), story_config.so_that.clone());
        
        // If it's the default template, also replace domain name
        if domain_name != "default" && story_config.want.contains("{domain_name}") {
            engine.set_variable("domain_name".to_string(), domain_name.to_lowercase());
        }
        
        Ok(engine.render(&templates.story_template))
    }

    pub fn render_framework_commands(&self, framework_name: &str) -> Result<String> {
        let config = get_config();
        let commands = config.get_framework_commands();
        
        let framework_commands = commands.framework_commands.get(framework_name)
            .unwrap_or(commands.framework_commands.get("default").unwrap());
            
        let mut result = String::new();
        result.push_str("```bash\n");
        
        for (command_type, command) in framework_commands {
            result.push_str(&format!("# {}\n{}\n", command_type, command));
        }
        
        result.push_str("```\n\n");
        Ok(result)
    }

    pub fn get_user_persona(&self, domain_name: &str) -> String {
        let config = get_config();
        let templates = config.get_user_story_templates();
        
        templates.user_personas.get(domain_name)
            .unwrap_or(templates.user_personas.get("default").unwrap())
            .clone()
    }

    pub fn get_unknown_project_name(&self) -> String {
        let config = get_config();
        let templates = config.get_ccpm_templates();
        
        templates.common.get("unknown_project")
            .unwrap_or(&"Unknown Project".to_string())
            .clone()
    }

    pub fn get_llm_prompt(&self, domain_name: &str, prompt_type: &str) -> Result<String> {
        let config = get_config();
        let templates = config.get_llm_prompt_templates();
        
        match prompt_type {
            "system" => {
                let domain_prompt = templates.domain_analysis_prompts.get(domain_name)
                    .unwrap_or(templates.domain_analysis_prompts.get("default").unwrap());
                Ok(domain_prompt.system_prompt.clone())
            },
            "analysis_focus" => {
                let domain_prompt = templates.domain_analysis_prompts.get(domain_name)
                    .unwrap_or(templates.domain_analysis_prompts.get("default").unwrap());
                Ok(domain_prompt.analysis_focus.join(", "))
            },
            _ => {
                templates.context_aware_prompts.get(prompt_type)
                    .or_else(|| templates.validation_prompts.get(prompt_type))
                    .map(|s| self.render(s))
                    .ok_or_else(|| anyhow::anyhow!("Unknown prompt type: {}", prompt_type))
            }
        }
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_replacement() {
        let mut engine = TemplateEngine::new();
        engine.set_variable("name".to_string(), "Test Project".to_string());
        engine.set_variable("version".to_string(), "1.0.0".to_string());
        
        let template = "Project: {name}, Version: {version}";
        let result = engine.render(template);
        
        assert_eq!(result, "Project: Test Project, Version: 1.0.0");
    }

    #[test]
    fn test_user_story_rendering() {
        let engine = TemplateEngine::new();
        let result = engine.render_user_story("Web Development").unwrap();
        
        assert!(result.contains("**As a** user"));
        assert!(result.contains("**I want** to interact with a web interface"));
        assert!(result.contains("**So that** I can access the application's functionality"));
    }
}