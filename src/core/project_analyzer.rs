use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use std::fs;

use crate::core::config::get_config;
use crate::core::project_classifier::{ProjectClassifier, ProjectType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    pub project_path: PathBuf,
    pub project_type: Option<ProjectType>,
    pub project_type_confidence: f32,
    pub purpose_description: String,
    pub entry_points: Vec<EntryPoint>,
    pub business_domain_hints: Vec<String>,
    pub metadata: ProjectMetadata,
    pub documentation_analysis: DocumentationAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryPoint {
    pub file_path: PathBuf,
    pub entry_type: EntryPointType,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntryPointType {
    Main,           // main.rs, main.py, index.ts
    Server,         // server.js, app.py
    Binary,         // Cargo.toml [[bin]]
    Module,         // lib.rs, __init__.py
    Application,    // App.tsx, app.component.ts
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub dependencies: HashMap<String, String>,
    pub dev_dependencies: HashMap<String, String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub package_manager: PackageManager,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PackageManager {
    Npm,
    Yarn,
    Pnpm,
    Cargo,
    Poetry,
    Pip,
    Maven,
    Gradle,
    Deno,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationAnalysis {
    pub readme_content: Option<String>,
    pub purpose_keywords: Vec<String>,
    pub domain_keywords: Vec<String>,
    pub setup_instructions: Vec<String>,
    pub api_documentation: Vec<String>,
}

/// Strategy pattern for metadata reading
#[async_trait]
pub trait MetadataReader: Send + Sync {
    async fn can_handle(&self, project_path: &Path) -> bool;
    async fn read_metadata(&self, project_path: &Path) -> Result<ProjectMetadata>;
    fn get_package_manager(&self) -> PackageManager;
}

/// Strategy for Node.js/npm projects
pub struct NodeMetadataReader;

#[async_trait]
impl MetadataReader for NodeMetadataReader {
    async fn can_handle(&self, project_path: &Path) -> bool {
        project_path.join("package.json").exists()
    }

    async fn read_metadata(&self, project_path: &Path) -> Result<ProjectMetadata> {
        let package_json_path = project_path.join("package.json");
        let content = fs::read_to_string(&package_json_path)
            .with_context(|| format!("Failed to read package.json from {:?}", package_json_path))?;

        let package_json: serde_json::Value = serde_json::from_str(&content)
            .with_context(|| "Failed to parse package.json")?;

        let dependencies = self.extract_dependencies(&package_json, "dependencies")?;
        let dev_dependencies = self.extract_dependencies(&package_json, "devDependencies")?;

        Ok(ProjectMetadata {
            name: package_json.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            version: package_json.get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            description: package_json.get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            authors: self.extract_authors(&package_json),
            dependencies,
            dev_dependencies,
            license: package_json.get("license")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            repository: package_json.get("repository")
                .and_then(|v| v.get("url"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            package_manager: self.get_package_manager(),
        })
    }

    fn get_package_manager(&self) -> PackageManager {
        PackageManager::Npm
    }
}

impl NodeMetadataReader {
    fn extract_dependencies(&self, package_json: &serde_json::Value, key: &str) -> Result<HashMap<String, String>> {
        let mut deps = HashMap::new();
        
        if let Some(dependencies) = package_json.get(key).and_then(|v| v.as_object()) {
            for (name, version) in dependencies {
                if let Some(version_str) = version.as_str() {
                    deps.insert(name.clone(), version_str.to_string());
                }
            }
        }
        
        Ok(deps)
    }

    fn extract_authors(&self, package_json: &serde_json::Value) -> Vec<String> {
        if let Some(author) = package_json.get("author").and_then(|v| v.as_str()) {
            vec![author.to_string()]
        } else if let Some(authors) = package_json.get("authors").and_then(|v| v.as_array()) {
            authors.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        } else {
            vec![]
        }
    }
}

/// Strategy for Deno projects
pub struct DenoMetadataReader;

#[async_trait]
impl MetadataReader for DenoMetadataReader {
    async fn can_handle(&self, project_path: &Path) -> bool {
        project_path.join("deno.json").exists() || 
        project_path.join("deno.jsonc").exists()
    }

    async fn read_metadata(&self, project_path: &Path) -> Result<ProjectMetadata> {
        let config_path = if project_path.join("deno.json").exists() {
            project_path.join("deno.json")
        } else {
            project_path.join("deno.jsonc")
        };

        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read Deno config from {:?}", config_path))?;

        // Basic JSON parsing (JSONC support would need additional handling)
        let deno_config: serde_json::Value = serde_json::from_str(&content)
            .with_context(|| "Failed to parse Deno config")?;

        let imports = self.extract_imports(&deno_config)?;

        Ok(ProjectMetadata {
            name: deno_config.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("deno-project")
                .to_string(),
            version: deno_config.get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            description: deno_config.get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            authors: vec![], // Deno doesn't typically have author field
            dependencies: imports,
            dev_dependencies: HashMap::new(),
            license: deno_config.get("license")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            repository: None,
            package_manager: PackageManager::Deno,
        })
    }

    fn get_package_manager(&self) -> PackageManager {
        PackageManager::Deno
    }
}

impl DenoMetadataReader {
    fn extract_imports(&self, deno_config: &serde_json::Value) -> Result<HashMap<String, String>> {
        let mut imports = HashMap::new();
        
        if let Some(import_map) = deno_config.get("imports").and_then(|v| v.as_object()) {
            for (name, url) in import_map {
                if let Some(url_str) = url.as_str() {
                    imports.insert(name.clone(), url_str.to_string());
                }
            }
        }
        
        Ok(imports)
    }
}

/// Strategy for Rust/Cargo projects
pub struct CargoMetadataReader;

#[async_trait]
impl MetadataReader for CargoMetadataReader {
    async fn can_handle(&self, project_path: &Path) -> bool {
        project_path.join("Cargo.toml").exists()
    }

    async fn read_metadata(&self, project_path: &Path) -> Result<ProjectMetadata> {
        let cargo_toml_path = project_path.join("Cargo.toml");
        let content = fs::read_to_string(&cargo_toml_path)
            .with_context(|| format!("Failed to read Cargo.toml from {:?}", cargo_toml_path))?;

        let cargo_toml: toml::Value = toml::from_str(&content)
            .with_context(|| "Failed to parse Cargo.toml")?;

        let dependencies = self.extract_dependencies(&cargo_toml, "dependencies")?;
        let dev_dependencies = self.extract_dependencies(&cargo_toml, "dev-dependencies")?;

        let package = cargo_toml.get("package")
            .ok_or_else(|| anyhow::anyhow!("Missing [package] section in Cargo.toml"))?;

        Ok(ProjectMetadata {
            name: package.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("rust-project")
                .to_string(),
            version: package.get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            description: package.get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            authors: self.extract_authors(&package),
            dependencies,
            dev_dependencies,
            license: package.get("license")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            repository: package.get("repository")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            package_manager: PackageManager::Cargo,
        })
    }

    fn get_package_manager(&self) -> PackageManager {
        PackageManager::Cargo
    }
}

impl CargoMetadataReader {
    fn extract_dependencies(&self, cargo_toml: &toml::Value, section: &str) -> Result<HashMap<String, String>> {
        let mut deps = HashMap::new();
        
        if let Some(dependencies) = cargo_toml.get(section).and_then(|v| v.as_table()) {
            for (name, spec) in dependencies {
                let version = match spec {
                    toml::Value::String(version) => version.clone(),
                    toml::Value::Table(table) => {
                        table.get("version")
                            .and_then(|v| v.as_str())
                            .unwrap_or("*")
                            .to_string()
                    }
                    _ => "*".to_string(),
                };
                deps.insert(name.clone(), version);
            }
        }
        
        Ok(deps)
    }

    fn extract_authors(&self, package: &toml::Value) -> Vec<String> {
        if let Some(authors) = package.get("authors").and_then(|v| v.as_array()) {
            authors.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        } else {
            vec![]
        }
    }
}

/// Strategy for Python projects
pub struct PythonMetadataReader;

#[async_trait]
impl MetadataReader for PythonMetadataReader {
    async fn can_handle(&self, project_path: &Path) -> bool {
        project_path.join("pyproject.toml").exists() ||
        project_path.join("setup.py").exists() ||
        project_path.join("requirements.txt").exists()
    }

    async fn read_metadata(&self, project_path: &Path) -> Result<ProjectMetadata> {
        // Try pyproject.toml first (modern Python projects)
        if let Ok(metadata) = self.read_pyproject_toml(project_path).await {
            return Ok(metadata);
        }

        // Fallback to basic requirements.txt parsing
        self.read_requirements_txt(project_path).await
    }

    fn get_package_manager(&self) -> PackageManager {
        PackageManager::Pip
    }
}

impl PythonMetadataReader {
    async fn read_pyproject_toml(&self, project_path: &Path) -> Result<ProjectMetadata> {
        let pyproject_path = project_path.join("pyproject.toml");
        let content = fs::read_to_string(&pyproject_path)
            .with_context(|| format!("Failed to read pyproject.toml from {:?}", pyproject_path))?;

        let pyproject: toml::Value = toml::from_str(&content)
            .with_context(|| "Failed to parse pyproject.toml")?;

        let project_section = pyproject.get("project")
            .ok_or_else(|| anyhow::anyhow!("Missing [project] section in pyproject.toml"))?;

        let dependencies = self.extract_pyproject_dependencies(&pyproject)?;

        Ok(ProjectMetadata {
            name: project_section.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("python-project")
                .to_string(),
            version: project_section.get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            description: project_section.get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            authors: self.extract_pyproject_authors(project_section),
            dependencies,
            dev_dependencies: HashMap::new(),
            license: project_section.get("license")
                .and_then(|v| v.get("text"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            repository: project_section.get("urls")
                .and_then(|v| v.get("repository"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            package_manager: PackageManager::Poetry,
        })
    }

    async fn read_requirements_txt(&self, project_path: &Path) -> Result<ProjectMetadata> {
        let req_path = project_path.join("requirements.txt");
        let dependencies = if req_path.exists() {
            let content = fs::read_to_string(&req_path)?;
            self.parse_requirements(&content)
        } else {
            HashMap::new()
        };

        Ok(ProjectMetadata {
            name: project_path.file_name()
                .and_then(|os_str| os_str.to_str())
                .unwrap_or("python-project")
                .to_string(),
            version: None,
            description: None,
            authors: vec![],
            dependencies,
            dev_dependencies: HashMap::new(),
            license: None,
            repository: None,
            package_manager: PackageManager::Pip,
        })
    }

    fn extract_pyproject_dependencies(&self, pyproject: &toml::Value) -> Result<HashMap<String, String>> {
        let mut deps = HashMap::new();
        
        if let Some(dependencies) = pyproject.get("project")
            .and_then(|p| p.get("dependencies"))
            .and_then(|v| v.as_array()) {
            for dep in dependencies {
                if let Some(dep_str) = dep.as_str() {
                    if let Some((name, version)) = self.parse_python_dependency(dep_str) {
                        deps.insert(name, version);
                    }
                }
            }
        }
        
        Ok(deps)
    }

    fn extract_pyproject_authors(&self, project_section: &toml::Value) -> Vec<String> {
        if let Some(authors) = project_section.get("authors").and_then(|v| v.as_array()) {
            authors.iter()
                .filter_map(|author| {
                    author.get("name")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
                .collect()
        } else {
            vec![]
        }
    }

    fn parse_requirements(&self, content: &str) -> HashMap<String, String> {
        let mut deps = HashMap::new();
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            if let Some((name, version)) = self.parse_python_dependency(line) {
                deps.insert(name, version);
            }
        }
        
        deps
    }

    fn parse_python_dependency(&self, dep_str: &str) -> Option<(String, String)> {
        // Simple parsing for dependencies like "requests>=2.25.1" or "flask==1.1.4"
        for separator in &[">=", "<=", "==", ">", "<", "~="] {
            if let Some(pos) = dep_str.find(separator) {
                let name = dep_str[..pos].trim().to_string();
                let version = dep_str[pos + separator.len()..].trim().to_string();
                return Some((name, version));
            }
        }
        
        // No version specified
        Some((dep_str.trim().to_string(), "*".to_string()))
    }
}

/// Strategy for entry point detection
#[async_trait]
pub trait EntryPointDetector: Send + Sync {
    async fn detect_entry_points(&self, project_path: &Path, metadata: &ProjectMetadata) -> Result<Vec<EntryPoint>>;
}

pub struct DefaultEntryPointDetector;

#[async_trait]
impl EntryPointDetector for DefaultEntryPointDetector {
    async fn detect_entry_points(&self, project_path: &Path, metadata: &ProjectMetadata) -> Result<Vec<EntryPoint>> {
        let mut entry_points = Vec::new();

        match metadata.package_manager {
            PackageManager::Cargo => {
                self.detect_rust_entry_points(project_path, &mut entry_points).await?;
            }
            PackageManager::Npm | PackageManager::Yarn | PackageManager::Pnpm => {
                self.detect_node_entry_points(project_path, &mut entry_points).await?;
            }
            PackageManager::Deno => {
                self.detect_deno_entry_points(project_path, &mut entry_points).await?;
            }
            PackageManager::Pip | PackageManager::Poetry => {
                self.detect_python_entry_points(project_path, &mut entry_points).await?;
            }
            _ => {
                self.detect_generic_entry_points(project_path, &mut entry_points).await?;
            }
        }

        Ok(entry_points)
    }
}

impl DefaultEntryPointDetector {
    async fn detect_rust_entry_points(&self, project_path: &Path, entry_points: &mut Vec<EntryPoint>) -> Result<()> {
        // Check for src/main.rs (binary)
        let main_rs = project_path.join("src").join("main.rs");
        if main_rs.exists() {
            entry_points.push(EntryPoint {
                file_path: main_rs,
                entry_type: EntryPointType::Main,
                confidence: 0.95,
            });
        }

        // Check for src/lib.rs (library)
        let lib_rs = project_path.join("src").join("lib.rs");
        if lib_rs.exists() {
            entry_points.push(EntryPoint {
                file_path: lib_rs,
                entry_type: EntryPointType::Module,
                confidence: 0.9,
            });
        }

        Ok(())
    }

    async fn detect_node_entry_points(&self, project_path: &Path, entry_points: &mut Vec<EntryPoint>) -> Result<()> {
        // Check common Node.js entry points
        let candidates = [
            ("index.js", EntryPointType::Main, 0.8),
            ("index.ts", EntryPointType::Main, 0.8),
            ("src/index.ts", EntryPointType::Main, 0.85),
            ("src/main.ts", EntryPointType::Main, 0.85),
            ("server.js", EntryPointType::Server, 0.7),
            ("app.js", EntryPointType::Application, 0.7),
        ];

        for (path, entry_type, confidence) in &candidates {
            let full_path = project_path.join(path);
            if full_path.exists() {
                entry_points.push(EntryPoint {
                    file_path: full_path,
                    entry_type: entry_type.clone(),
                    confidence: *confidence,
                });
            }
        }

        Ok(())
    }

    async fn detect_deno_entry_points(&self, project_path: &Path, entry_points: &mut Vec<EntryPoint>) -> Result<()> {
        // Check for mod.ts (Deno convention)
        let mod_ts = project_path.join("mod.ts");
        if mod_ts.exists() {
            entry_points.push(EntryPoint {
                file_path: mod_ts,
                entry_type: EntryPointType::Module,
                confidence: 0.9,
            });
        }

        // Check for main.ts
        let main_ts = project_path.join("main.ts");
        if main_ts.exists() {
            entry_points.push(EntryPoint {
                file_path: main_ts,
                entry_type: EntryPointType::Main,
                confidence: 0.85,
            });
        }

        Ok(())
    }

    async fn detect_python_entry_points(&self, project_path: &Path, entry_points: &mut Vec<EntryPoint>) -> Result<()> {
        let candidates = [
            ("main.py", EntryPointType::Main, 0.8),
            ("app.py", EntryPointType::Application, 0.7),
            ("__init__.py", EntryPointType::Module, 0.6),
        ];

        for (path, entry_type, confidence) in &candidates {
            let full_path = project_path.join(path);
            if full_path.exists() {
                entry_points.push(EntryPoint {
                    file_path: full_path,
                    entry_type: entry_type.clone(),
                    confidence: *confidence,
                });
            }
        }

        Ok(())
    }

    async fn detect_generic_entry_points(&self, project_path: &Path, entry_points: &mut Vec<EntryPoint>) -> Result<()> {
        // Generic detection for unknown project types
        let candidates = [
            "index.js", "index.ts", "main.js", "main.ts", "main.py", 
            "app.js", "app.py", "server.js", "main.rs", "lib.rs"
        ];

        for candidate in &candidates {
            let full_path = project_path.join(candidate);
            if full_path.exists() {
                entry_points.push(EntryPoint {
                    file_path: full_path,
                    entry_type: EntryPointType::Main,
                    confidence: 0.5,
                });
            }
        }

        Ok(())
    }
}

/// Main project analyzer coordinating all strategies
pub struct ProjectAnalyzer {
    metadata_readers: Vec<Box<dyn MetadataReader>>,
    entry_point_detector: Box<dyn EntryPointDetector>,
}

impl ProjectAnalyzer {
    pub fn new() -> Self {
        let metadata_readers: Vec<Box<dyn MetadataReader>> = vec![
            Box::new(DenoMetadataReader),      // Check Deno first (more specific)
            Box::new(NodeMetadataReader),
            Box::new(CargoMetadataReader),
            Box::new(PythonMetadataReader),
        ];

        Self {
            metadata_readers,
            entry_point_detector: Box::new(DefaultEntryPointDetector),
        }
    }

    pub async fn analyze_project_context(&self, project_path: &Path) -> Result<ProjectContext> {
        // Step 1: Read metadata using appropriate strategy
        let metadata = self.read_project_metadata(project_path).await?;

        // Step 2: Detect entry points
        let entry_points = self.entry_point_detector
            .detect_entry_points(project_path, &metadata)
            .await?;

        // Step 3: Analyze documentation
        let documentation_analysis = self.analyze_documentation(project_path).await?;

        // Step 4: Classify project type using modern strategy pattern
        let classifier = ProjectClassifier::new();
        let classification_result = classifier.classify_project(&metadata, &documentation_analysis)?;
        
        // Step 5: Infer project purpose from classification and documentation
        let purpose_description = if let Some(readme) = &documentation_analysis.readme_content {
            if let Some(desc) = &metadata.description {
                format!("{} - {}", desc, readme.lines().next().unwrap_or("").trim_start_matches('#').trim())
            } else {
                readme.lines().next().unwrap_or("Unknown project purpose").trim_start_matches('#').trim().to_string()
            }
        } else {
            metadata.description.clone().unwrap_or_else(|| classification_result.project_type.display_name().to_string())
        };

        // Get business domain hints from configuration
        let business_domain_hints = if classification_result.project_type != ProjectType::Unknown {
            let config = get_config();
            if let Some(project_config) = config.get_project_type(classification_result.project_type.to_config_id()) {
                project_config.domain_hints.clone()
            } else {
                documentation_analysis.domain_keywords.clone()
            }
        } else {
            documentation_analysis.domain_keywords.clone()
        };

        Ok(ProjectContext {
            project_path: project_path.to_path_buf(),
            project_type: Some(classification_result.project_type),
            project_type_confidence: classification_result.confidence,
            purpose_description,
            entry_points,
            business_domain_hints,
            metadata,
            documentation_analysis,
        })
    }

    async fn read_project_metadata(&self, project_path: &Path) -> Result<ProjectMetadata> {
        for reader in &self.metadata_readers {
            if reader.can_handle(project_path).await {
                return reader.read_metadata(project_path).await;
            }
        }

        // Fallback metadata if no reader can handle the project
        Ok(ProjectMetadata {
            name: project_path.file_name()
                .and_then(|os_str| os_str.to_str())
                .unwrap_or("unknown-project")
                .to_string(),
            version: None,
            description: None,
            authors: vec![],
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            license: None,
            repository: None,
            package_manager: PackageManager::Unknown,
        })
    }

    async fn analyze_documentation(&self, project_path: &Path) -> Result<DocumentationAnalysis> {
        let readme_path = project_path.join("README.md");
        let readme_content = if readme_path.exists() {
            fs::read_to_string(&readme_path).ok()
        } else {
            None
        };

        let (purpose_keywords, domain_keywords) = if let Some(content) = &readme_content {
            self.extract_keywords_from_content(content)
        } else {
            (vec![], vec![])
        };

        Ok(DocumentationAnalysis {
            readme_content,
            purpose_keywords,
            domain_keywords,
            setup_instructions: vec![], // TODO: Extract setup instructions
            api_documentation: vec![],  // TODO: Extract API docs
        })
    }

    fn extract_keywords_from_content(&self, content: &str) -> (Vec<String>, Vec<String>) {
        let content_lower = content.to_lowercase();
        let config = get_config();

        let mut purpose_keywords = Vec::new();
        let mut domain_keywords = Vec::new();

        // Extract project type keywords
        for project_type in config.get_all_project_types() {
            for indicator in &project_type.indicators {
                if indicator.r#type == "readme_keywords" {
                    for pattern in &indicator.patterns {
                        if content_lower.contains(&pattern.to_lowercase()) {
                            purpose_keywords.push(pattern.clone());
                        }
                    }
                }
            }
        }

        // Extract business domain keywords
        for domain in config.get_all_business_domains() {
            for keyword in &domain.keywords {
                if content_lower.contains(&keyword.to_lowercase()) {
                    domain_keywords.push(keyword.clone());
                }
            }
        }

        (purpose_keywords, domain_keywords)
    }

}

impl Default for ProjectAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_node_metadata_reader() {
        let temp_dir = TempDir::new().unwrap();
        let package_json = r#"
        {
            "name": "test-project",
            "version": "1.0.0",
            "description": "Test project",
            "dependencies": {
                "react": "^18.0.0",
                "axios": "^0.27.0"
            }
        }
        "#;
        
        fs::write(temp_dir.path().join("package.json"), package_json).unwrap();
        
        let reader = NodeMetadataReader;
        assert!(reader.can_handle(temp_dir.path()).await);
        
        let metadata = reader.read_metadata(temp_dir.path()).await.unwrap();
        assert_eq!(metadata.name, "test-project");
        assert_eq!(metadata.version, Some("1.0.0".to_string()));
        assert!(metadata.dependencies.contains_key("react"));
    }

    #[tokio::test]
    async fn test_deno_metadata_reader() {
        let temp_dir = TempDir::new().unwrap();
        let deno_json = r#"
        {
            "name": "deno-project",
            "version": "1.0.0",
            "imports": {
                "danet": "https://deno.land/x/danet@1.0.0/mod.ts"
            }
        }
        "#;
        
        fs::write(temp_dir.path().join("deno.json"), deno_json).unwrap();
        
        let reader = DenoMetadataReader;
        assert!(reader.can_handle(temp_dir.path()).await);
        
        let metadata = reader.read_metadata(temp_dir.path()).await.unwrap();
        assert_eq!(metadata.name, "deno-project");
        assert_eq!(metadata.package_manager, PackageManager::Deno);
        assert!(metadata.dependencies.contains_key("danet"));
    }

    #[tokio::test]
    async fn test_project_analyzer_integration() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create a simple Rust project
        fs::create_dir(temp_dir.path().join("src")).unwrap();
        fs::write(temp_dir.path().join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(temp_dir.path().join("Cargo.toml"), r#"
[package]
name = "codebase-analyzer"
version = "1.0.0"
description = "A tool for analyzing codebases"

[dependencies]
tree-sitter = "0.20"
"#).unwrap();
        
        fs::write(temp_dir.path().join("README.md"), 
            "# Codebase Analyzer\nThis tool analyzes codebases and detects frameworks."
        ).unwrap();
        
        let analyzer = ProjectAnalyzer::new();
        let context = analyzer.analyze_project_context(temp_dir.path()).await.unwrap();
        
        assert_eq!(context.metadata.name, "codebase-analyzer");
        assert_eq!(context.project_type, Some(ProjectType::AnalysisTool));
        assert!(!context.entry_points.is_empty());
        assert!(context.documentation_analysis.purpose_keywords.contains(&"analyze".to_string()));
    }
}