use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub data: T,
    pub timestamp: u64,
    pub file_hash: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    pub project_path: String,
    pub last_analysis: u64,
    pub file_count: usize,
    pub total_size: u64,
}

pub struct CacheManager {
    cache_dir: String,
    max_age: Duration,
    version: String,
}

impl CacheManager {
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| std::path::PathBuf::from(".cache"))
            .join("codebase-workflow-analyzer");
        
        std::fs::create_dir_all(&cache_dir)
            .context("Failed to create cache directory")?;
        
        Ok(Self {
            cache_dir: cache_dir.to_string_lossy().to_string(),
            max_age: Duration::from_secs(3600 * 24), // 24 hours default
            version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }
    
    pub fn with_max_age(mut self, max_age: Duration) -> Self {
        self.max_age = max_age;
        self
    }
    
    pub fn get_cache_key(&self, project_path: &str, analysis_type: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(project_path.as_bytes());
        hasher.update(analysis_type.as_bytes());
        hasher.update(self.version.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    pub fn calculate_project_hash(&self, project_path: &str) -> Result<String> {
        let mut hasher = Sha256::new();
        let mut file_count = 0;
        let mut total_size = 0u64;
        
        // Hash key files that indicate project structure changes
        let important_files = vec![
            "Cargo.toml", "package.json", "pom.xml", "requirements.txt",
            "README.md", ".gitignore", "tsconfig.json", "setup.py"
        ];
        
        for file_name in important_files {
            let file_path = Path::new(project_path).join(file_name);
            if file_path.exists() {
                if let Ok(metadata) = std::fs::metadata(&file_path) {
                    hasher.update(file_name.as_bytes());
                    hasher.update(&metadata.len().to_le_bytes());
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                            hasher.update(&duration.as_secs().to_le_bytes());
                        }
                    }
                    file_count += 1;
                    total_size += metadata.len();
                }
            }
        }
        
        // Hash directory structure
        if let Ok(entries) = std::fs::read_dir(project_path) {
            let mut dirs: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .filter(|name| !name.starts_with('.') && name != "target" && name != "node_modules")
                .collect();
            
            dirs.sort();
            for dir in dirs {
                hasher.update(dir.as_bytes());
            }
        }
        
        // Store metadata for cache validation
        let metadata = CacheMetadata {
            project_path: project_path.to_string(),
            last_analysis: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            file_count,
            total_size,
        };
        
        let metadata_path = Path::new(&self.cache_dir).join("metadata.json");
        if let Ok(metadata_json) = serde_json::to_string_pretty(&metadata) {
            let _ = std::fs::write(metadata_path, metadata_json);
        }
        
        Ok(format!("{:x}", hasher.finalize()))
    }
    
    pub fn get<T>(&self, cache_key: &str) -> Result<Option<T>> 
    where 
        T: for<'de> Deserialize<'de>
    {
        let cache_file = Path::new(&self.cache_dir).join(format!("{}.json", cache_key));
        
        if !cache_file.exists() {
            return Ok(None);
        }
        
        let content = std::fs::read_to_string(&cache_file)
            .context("Failed to read cache file")?;
        
        let entry: CacheEntry<T> = serde_json::from_str(&content)
            .context("Failed to deserialize cache entry")?;
        
        // Check if cache is still valid
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let age = Duration::from_secs(now - entry.timestamp);
        
        if age > self.max_age {
            // Cache expired, remove it
            let _ = std::fs::remove_file(cache_file);
            return Ok(None);
        }
        
        // Check version compatibility
        if entry.version != self.version {
            let _ = std::fs::remove_file(cache_file);
            return Ok(None);
        }
        
        Ok(Some(entry.data))
    }
    
    pub fn set<T>(&self, cache_key: &str, data: T, file_hash: &str) -> Result<()>
    where 
        T: Serialize
    {
        let entry = CacheEntry {
            data,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            file_hash: file_hash.to_string(),
            version: self.version.clone(),
        };
        
        let cache_file = Path::new(&self.cache_dir).join(format!("{}.json", cache_key));
        let content = serde_json::to_string_pretty(&entry)
            .context("Failed to serialize cache entry")?;
        
        std::fs::write(cache_file, content)
            .context("Failed to write cache file")?;
        
        Ok(())
    }
    
    pub fn invalidate(&self, cache_key: &str) -> Result<()> {
        let cache_file = Path::new(&self.cache_dir).join(format!("{}.json", cache_key));
        if cache_file.exists() {
            std::fs::remove_file(cache_file)
                .context("Failed to remove cache file")?;
        }
        Ok(())
    }
    
    pub fn clear_all(&self) -> Result<()> {
        if Path::new(&self.cache_dir).exists() {
            std::fs::remove_dir_all(&self.cache_dir)
                .context("Failed to clear cache directory")?;
            std::fs::create_dir_all(&self.cache_dir)
                .context("Failed to recreate cache directory")?;
        }
        Ok(())
    }
    
    pub fn get_cache_stats(&self) -> Result<HashMap<String, u64>> {
        let mut stats = HashMap::new();
        let mut total_size = 0u64;
        let mut file_count = 0u64;
        
        if let Ok(entries) = std::fs::read_dir(&self.cache_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                    file_count += 1;
                }
            }
        }
        
        stats.insert("total_size_bytes".to_string(), total_size);
        stats.insert("file_count".to_string(), file_count);
        
        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_cache_basic_operations() {
        let cache = CacheManager::new().unwrap();
        let test_data = vec!["test", "data"];
        let cache_key = "test_key";
        let file_hash = "test_hash";
        
        // Test set and get
        cache.set(cache_key, &test_data, file_hash).unwrap();
        let retrieved: Option<Vec<&str>> = cache.get(cache_key).unwrap();
        
        assert_eq!(retrieved, Some(test_data));
        
        // Test invalidation
        cache.invalidate(cache_key).unwrap();
        let after_invalidation: Option<Vec<&str>> = cache.get(cache_key).unwrap();
        assert_eq!(after_invalidation, None);
    }
    
    #[test]
    fn test_project_hash_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_str().unwrap();
        
        // Create test files
        std::fs::write(temp_dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
        std::fs::write(temp_dir.path().join("README.md"), "# Test Project").unwrap();
        
        let cache = CacheManager::new().unwrap();
        let hash1 = cache.calculate_project_hash(project_path).unwrap();
        let hash2 = cache.calculate_project_hash(project_path).unwrap();
        
        assert_eq!(hash1, hash2); // Same project should have same hash
        
        // Modify a file and hash should change
        std::fs::write(temp_dir.path().join("Cargo.toml"), "[package]\nname = \"modified\"").unwrap();
        std::thread::sleep(Duration::from_millis(100)); // Ensure different timestamp
        
        let hash3 = cache.calculate_project_hash(project_path).unwrap();
        assert_ne!(hash1, hash3); // Hash should change after modification
    }
}