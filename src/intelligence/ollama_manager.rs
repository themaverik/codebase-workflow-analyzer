use std::collections::HashMap;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context, bail};
use std::io::{self, Write};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub default_model: String,
    pub ollama_url: String,
    pub model_settings: ModelSettings,
    pub recommended_models: Vec<RecommendedModel>,
    pub mcp_permissions: McpPermissions,
    pub disk_space: DiskSpaceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSettings {
    pub temperature: f32,
    pub context_window: usize,
    pub max_tokens: usize,
    pub timeout_seconds: u64,
    pub top_p: f32,
    pub repeat_penalty: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedModel {
    pub name: String,
    pub description: String,
    pub size_gb: f32,
    pub use_case: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPermissions {
    pub file_system_access: PermissionSpec,
    pub directory_listing: PermissionSpec,
    pub read_permissions: PermissionSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionSpec {
    pub required: bool,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskSpaceConfig {
    pub minimum_free_gb: f32,
    pub recommended_free_gb: f32,
    pub check_before_download: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaModelList {
    pub models: Vec<OllamaModel>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub modified_at: String,
    pub size: i64,
    pub digest: String,
    pub details: Option<ModelDetails>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelDetails {
    pub format: String,
    pub family: String,
    pub families: Option<Vec<String>>,
    pub parameter_size: String,
    pub quantization_level: String,
}

pub struct OllamaManager {
    client: reqwest::Client,
    config: OllamaConfig,
}

impl OllamaManager {
    pub fn new() -> Result<Self> {
        // Creating OllamaManager
        let config = Self::load_config()?;
        // Config loaded successfully
        
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.model_settings.timeout_seconds))
            .build()?;
        // HTTP client created

        Ok(Self { client, config })
    }

    fn load_config() -> Result<OllamaConfig> {
        let config_path = "configs/data/ollama_config.json";
        // Loading config from file
        
        // Check if file exists
        if !std::path::Path::new(config_path).exists() {
            // Config file does not exist
            return Err(anyhow::anyhow!("Config file not found: {}. Please ensure the file exists.", config_path));
        }
        
        let content = std::fs::read_to_string(config_path)
            .with_context(|| format!("Failed to read Ollama config from {}", config_path))?;
        
        // Config file read successfully
        
        let config: OllamaConfig = serde_json::from_str(&content)
            .with_context(|| "Failed to parse Ollama configuration")?;
        
        // Config parsed successfully
        // Default model loaded
        // Ollama URL configured
        
        Ok(config)
    }

    pub async fn initialize_with_user_interaction(&mut self, project_path: &str) -> Result<String> {
        println!("\n🤖 Setting up Ollama LLM Integration");
        println!("=====================================");

        // Step 1: Check if Ollama is running
        if !self.check_ollama_connection().await? {
            println!("❌ Ollama is not running or not accessible at {}", self.config.ollama_url);
            println!("Please ensure Ollama is installed and running:");
            println!("  1. Install: https://ollama.com/");
            println!("  2. Start: ollama serve");
            bail!("Ollama not available");
        }

        println!("✅ Ollama is running at {}", self.config.ollama_url);

        // Step 2: Check for available models
        let available_models = self.list_available_models().await?;
        
        // Step 3: Check if default model exists
        let selected_model = if self.model_exists(&available_models, &self.config.default_model) {
            println!("✅ Default model '{}' is available", self.config.default_model);
            self.config.default_model.clone()
        } else {
            self.handle_model_selection(&available_models).await?
        };

        // Step 4: Request MCP permissions
        self.request_mcp_permissions(project_path)?;

        // Step 5: Validate model works with a test prompt
        self.validate_model(&selected_model).await?;

        println!("✅ Ollama LLM integration successfully configured!");
        println!("   Model: {}", selected_model);
        println!("   Ready for business context analysis\n");

        Ok(selected_model)
    }

    pub async fn check_ollama_connection(&self) -> Result<bool> {
        let response = self.client
            .get(&format!("{}/api/tags", self.config.ollama_url))
            .send()
            .await;

        match response {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    pub async fn list_available_models(&self) -> Result<Vec<OllamaModel>> {
        let response = self.client
            .get(&format!("{}/api/tags", self.config.ollama_url))
            .send()
            .await?;

        if !response.status().is_success() {
            bail!("Failed to fetch models from Ollama: {}", response.status());
        }

        let model_list: OllamaModelList = response.json().await?;
        Ok(model_list.models)
    }

    pub fn model_exists(&self, models: &[OllamaModel], model_name: &str) -> bool {
        models.iter().any(|m| m.name.starts_with(model_name))
    }

    async fn handle_model_selection(&self, available_models: &[OllamaModel]) -> Result<String> {
        println!("\n🔍 Default model '{}' not found", self.config.default_model);
        
        if available_models.is_empty() {
            println!("No models installed. Would you like to install the recommended model?");
        } else {
            println!("Available models:");
            for (i, model) in available_models.iter().enumerate() {
                let size_mb = model.size as f64 / 1_048_576.0;
                println!("  {}. {} ({:.1} MB)", i + 1, model.name, size_mb);
            }
        }

        println!("\nRecommended models:");
        for (i, rec_model) in self.config.recommended_models.iter().enumerate() {
            println!("  {}. {} - {} ({:.1} GB)", 
                    i + 1, rec_model.name, rec_model.description, rec_model.size_gb);
            println!("     Use case: {}", rec_model.use_case);
        }

        print!("\nOptions:\n");
        print!("1. Use default model '{}' (will download if needed)\n", self.config.default_model);
        print!("2. Select from available models\n");
        print!("3. Specify a different model\n");
        print!("Choice (1-3): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        match input.trim() {
            "1" => {
                self.download_model_if_needed(&self.config.default_model).await?;
                Ok(self.config.default_model.clone())
            }
            "2" => {
                if available_models.is_empty() {
                    println!("No models available to select from.");
                    self.download_model_if_needed(&self.config.default_model).await?;
                    Ok(self.config.default_model.clone())
                } else {
                    self.select_from_available_models(available_models).await
                }
            }
            "3" => self.specify_custom_model().await,
            _ => {
                println!("Invalid choice, using default model.");
                self.download_model_if_needed(&self.config.default_model).await?;
                Ok(self.config.default_model.clone())
            }
        }
    }

    async fn download_model_if_needed(&self, model_name: &str) -> Result<()> {
        // Check disk space first
        if self.config.disk_space.check_before_download {
            self.check_disk_space(model_name)?;
        }

        print!("Model '{}' needs to be downloaded. Continue? (y/N): ", model_name);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().to_lowercase().starts_with('y') {
            bail!("Model download cancelled by user");
        }

        println!("📥 Downloading model '{}' - this may take several minutes...", model_name);
        
        let response = self.client
            .post(&format!("{}/api/pull", self.config.ollama_url))
            .json(&serde_json::json!({
                "name": model_name,
                "stream": false
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            bail!("Failed to download model '{}': {}", model_name, error_text);
        }

        println!("✅ Model '{}' downloaded successfully", model_name);
        Ok(())
    }

    fn check_disk_space(&self, model_name: &str) -> Result<()> {
        // Find model info to estimate size
        let estimated_size = self.config.recommended_models
            .iter()
            .find(|m| m.name == model_name)
            .map(|m| m.size_gb)
            .unwrap_or(4.0); // Default estimate

        // Simple disk space check (platform-specific implementation would be better)
        println!("⚠️  Model '{}' requires approximately {:.1} GB of disk space", model_name, estimated_size);
        println!("Please ensure you have at least {:.1} GB free space before proceeding.", 
                self.config.disk_space.minimum_free_gb);

        Ok(())
    }

    async fn select_from_available_models(&self, models: &[OllamaModel]) -> Result<String> {
        print!("Select model number (1-{}): ", models.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if let Ok(choice) = input.trim().parse::<usize>() {
            if choice > 0 && choice <= models.len() {
                return Ok(models[choice - 1].name.clone());
            }
        }

        println!("Invalid selection, using first available model.");
        Ok(models[0].name.clone())
    }

    async fn specify_custom_model(&self) -> Result<String> {
        print!("Enter model name (e.g., 'llama3.2:3b'): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let model_name = input.trim().to_string();

        if model_name.is_empty() {
            println!("Empty model name, using default.");
            return Ok(self.config.default_model.clone());
        }

        // Check if model exists, if not offer to download
        let available_models = self.list_available_models().await?;
        if !self.model_exists(&available_models, &model_name) {
            println!("Model '{}' not found locally.", model_name);
            self.download_model_if_needed(&model_name).await?;
        }

        Ok(model_name)
    }

    fn request_mcp_permissions(&self, project_path: &str) -> Result<()> {
        println!("\n🔒 MCP Permission Request");
        println!("=========================");
        println!("The LLM analysis requires access to your project files for:");

        for (name, permission) in [
            ("File System Access", &self.config.mcp_permissions.file_system_access),
            ("Directory Listing", &self.config.mcp_permissions.directory_listing),
            ("Read Permissions", &self.config.mcp_permissions.read_permissions),
        ] {
            if permission.required {
                println!("  ✓ {}: {}", name, permission.description);
            }
        }

        println!("\nProject to analyze: {}", project_path);
        println!("This tool will only read files and will not modify your project.");

        print!("\nGrant permissions for MCP file system access? (y/N): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().to_lowercase().starts_with('y') {
            bail!("MCP permissions denied by user. LLM analysis cannot proceed without file access.");
        }

        println!("✅ MCP permissions granted");

        // Validate we can actually access the project
        self.validate_project_access(project_path)?;

        Ok(())
    }

    fn validate_project_access(&self, project_path: &str) -> Result<()> {
        let path = std::path::Path::new(project_path);
        
        if !path.exists() {
            bail!("Project path '{}' does not exist", project_path);
        }

        if !path.is_dir() {
            bail!("Project path '{}' is not a directory", project_path);
        }

        // Test reading a file in the directory
        match std::fs::read_dir(path) {
            Ok(entries) => {
                let count = entries.count();
                println!("✅ Successfully validated access to project directory ({} items)", count);
            }
            Err(e) => {
                bail!("Cannot read project directory '{}': {}", project_path, e);
            }
        }

        Ok(())
    }

    async fn validate_model(&self, model_name: &str) -> Result<()> {
        println!("🧪 Testing model '{}' with sample prompt...", model_name);

        let test_prompt = "Analyze this code: `function hello() { return 'world'; }` - what is its purpose?";
        
        let response = self.client
            .post(&format!("{}/api/generate", self.config.ollama_url))
            .json(&serde_json::json!({
                "model": model_name,
                "prompt": test_prompt,
                "stream": false,
                "options": {
                    "temperature": self.config.model_settings.temperature,
                    "num_predict": 50  // Short response for test
                }
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            bail!("Model validation failed: {}", error_text);
        }

        let response_json: serde_json::Value = response.json().await?;
        if let Some(response_text) = response_json["response"].as_str() {
            if response_text.trim().is_empty() {
                bail!("Model returned empty response - may not be functioning correctly");
            }
            println!("✅ Model validation successful - received response ({} chars)", 
                    response_text.len());
        } else {
            bail!("Invalid response format from model");
        }

        Ok(())
    }

    pub async fn generate_response(&self, model: &str, prompt: &str) -> Result<String> {
        // Generating LLM response
        // Using model for generation
        // Processing prompt
        // Connecting to Ollama API
        
        let request_payload = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": self.config.model_settings.temperature,
                "num_predict": self.config.model_settings.max_tokens,
                "top_p": self.config.model_settings.top_p,
                "repeat_penalty": self.config.model_settings.repeat_penalty
            }
        });
        
        // Preparing request payload
        
        // Sending request to Ollama
        let response = match self.client
            .post(&format!("{}/api/generate", self.config.ollama_url))
            .json(&request_payload)
            .send()
            .await {
                Ok(resp) => {
                    // Received response from Ollama
                    // Response received successfully
                    resp
                }
                Err(e) => {
                    // Failed to send request to Ollama
                    return Err(e.into());
                }
            };

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            // Ollama returned error
            bail!("LLM generation failed (status {}): {}", status, error_text);
        }

        // Parsing response JSON
        let response_text = response.text().await?;
        // Processing response
        
        let response_json: serde_json::Value = match serde_json::from_str(&response_text) {
            Ok(json) => json,
            Err(e) => {
                // Failed to parse JSON
                bail!("Failed to parse JSON response from Ollama: {}", e);
            }
        };
        
        // Skip logging full JSON to avoid token arrays in output
        // Successfully parsed response JSON
        
        match response_json["response"].as_str() {
            Some(content) => {
                // Extracted response content
                // Content extracted successfully
                Ok(content.to_string())
            }
            None => {
                // No 'response' field found in JSON
                bail!("Invalid response format from LLM - missing 'response' field")
            }
        }
    }

    pub fn get_config(&self) -> &OllamaConfig {
        &self.config
    }
}