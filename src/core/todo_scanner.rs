use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
// Using standard println for logging - could be upgraded to proper logging later

// use crate::core::config::get_config;

/// Types of TODO-style comments found in code
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TodoType {
    Todo,
    Fixme,
    Hack,
    Note,
    Bug,
    Deprecated,
    Custom(String),
}

impl TodoType {
    /// Convert from string representation
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "TODO" => Self::Todo,
            "FIXME" => Self::Fixme,
            "HACK" => Self::Hack,
            "NOTE" => Self::Note,
            "BUG" => Self::Bug,
            "DEPRECATED" => Self::Deprecated,
            _ => Self::Custom(s.to_string()),
        }
    }

    /// Get display string for the TODO type
    pub fn as_str(&self) -> &str {
        match self {
            Self::Todo => "TODO",
            Self::Fixme => "FIXME",
            Self::Hack => "HACK", 
            Self::Note => "NOTE",
            Self::Bug => "BUG",
            Self::Deprecated => "DEPRECATED",
            Self::Custom(s) => s,
        }
    }
}

/// Priority level for TODO items
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TodoPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl TodoPriority {
    /// Determine priority based on TODO type and context
    pub fn from_todo_type_and_context(todo_type: &TodoType, context: &str) -> Self {
        let context_lower = context.to_lowercase();
        
        // Critical indicators
        if context_lower.contains("critical") || context_lower.contains("urgent") || 
           context_lower.contains("security") || context_lower.contains("vulnerability") {
            return Self::Critical;
        }

        // High priority indicators  
        if context_lower.contains("important") || context_lower.contains("blocker") ||
           context_lower.contains("broken") || context_lower.contains("crash") {
            return Self::High;
        }

        // Base priority by type
        match todo_type {
            TodoType::Bug | TodoType::Fixme => Self::High,
            TodoType::Hack => Self::Medium,
            TodoType::Deprecated => Self::Medium,
            TodoType::Todo => Self::Medium,
            TodoType::Note => Self::Low,
            TodoType::Custom(_) => Self::Medium,
        }
    }
}

/// A single TODO item found in the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActualTodoItem {
    pub file_path: PathBuf,
    pub line_number: u32,
    pub column_number: Option<u32>,
    pub todo_type: TodoType,
    pub description: String,
    pub priority: TodoPriority,
    pub author: Option<String>,
    pub date_added: Option<String>,
    pub context_lines: Vec<String>,
    pub tags: Vec<String>,
}

/// Results from scanning for TODO comments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoScanResult {
    pub todo_items: Vec<ActualTodoItem>,
    pub summary: TodoSummary,
    pub scan_metadata: TodoScanMetadata,
}

/// Summary statistics for TODO items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoSummary {
    pub total_items: usize,
    pub by_type: HashMap<TodoType, usize>,
    pub by_priority: HashMap<TodoPriority, usize>,
    pub by_file_type: HashMap<String, usize>,
    pub most_problematic_files: Vec<ProblematicFile>,
}

/// File with high concentration of TODO items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblematicFile {
    pub file_path: PathBuf,
    pub todo_count: usize,
    pub high_priority_count: usize,
    pub file_size_lines: Option<u32>,
    pub todo_density: f32, // TODOs per 100 lines
}

/// Metadata about the TODO scanning process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoScanMetadata {
    pub files_scanned: usize,
    pub total_lines_scanned: u64,
    pub scan_duration_ms: u64,
    pub file_types_scanned: Vec<String>,
    pub excluded_patterns: Vec<String>,
    pub scan_errors: Vec<String>,
}

/// Configuration for TODO scanning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoScanConfig {
    pub comment_patterns: Vec<String>,
    pub include_file_patterns: Vec<String>,
    pub exclude_file_patterns: Vec<String>,
    pub context_lines_before: u32,
    pub context_lines_after: u32,
    pub max_scan_depth: u32,
    pub max_file_size_mb: u64,
}

impl Default for TodoScanConfig {
    fn default() -> Self {
        Self {
            comment_patterns: vec![
                "TODO".to_string(),
                "FIXME".to_string(), 
                "HACK".to_string(),
                "NOTE".to_string(),
                "BUG".to_string(),
                "DEPRECATED".to_string(),
            ],
            include_file_patterns: vec![
                "*.rs".to_string(),
                "*.py".to_string(),
                "*.js".to_string(),
                "*.ts".to_string(),
                "*.java".to_string(),
                "*.cpp".to_string(),
                "*.c".to_string(),
                "*.h".to_string(),
                "*.go".to_string(),
            ],
            exclude_file_patterns: vec![
                "target/**".to_string(),
                "node_modules/**".to_string(),
                "*.min.js".to_string(),
                "*.bundle.js".to_string(),
                ".git/**".to_string(),
            ],
            context_lines_before: 2,
            context_lines_after: 1,
            max_scan_depth: 10,
            max_file_size_mb: 10,
        }
    }
}

/// Scanner for TODO-style comments in source code
#[derive(Debug)]
pub struct TodoScanner {
    config: TodoScanConfig,
    comment_regexes: HashMap<String, Regex>,
    file_type_regexes: Vec<(String, Regex)>,
}

impl TodoScanner {
    /// Create a new TODO scanner with configuration
    pub fn new() -> Result<Self> {
        let config = Self::load_config()
            .context("Failed to load TODO scanner configuration")?;

        let mut scanner = Self {
            config: config.clone(),
            comment_regexes: HashMap::new(),
            file_type_regexes: Vec::new(),
        };

        scanner.compile_patterns()
            .context("Failed to compile TODO scanning patterns")?;

        // Debug: Initialized TodoScanner with comment patterns

        Ok(scanner)
    }

    /// Load configuration from the analyzer config file
    fn load_config() -> Result<TodoScanConfig> {
        // This is a placeholder - you'll need to integrate with the config system
        // For now, return default config
        Ok(TodoScanConfig::default())
    }

    /// Compile regex patterns for efficient matching
    fn compile_patterns(&mut self) -> Result<()> {
        // Compile comment patterns for different languages
        for pattern in &self.config.comment_patterns {
            // Pattern for various comment styles
            let regex_patterns = vec![
                format!(r"(?i)//\s*{}\s*:?\s*(.+)", regex::escape(pattern)),           // C++ style: // TODO: description
                format!(r"(?i)#\s*{}\s*:?\s*(.+)", regex::escape(pattern)),            // Python style: # TODO: description  
                format!(r"(?i)/\*\s*{}\s*:?\s*(.+?)\s*\*/", regex::escape(pattern)),   // C style: /* TODO: description */
                format!(r"(?i)<!--\s*{}\s*:?\s*(.+?)\s*-->", regex::escape(pattern)),  // HTML style: <!-- TODO: description -->
            ];

            let combined_pattern = format!("({})", regex_patterns.join("|"));
            let regex = Regex::new(&combined_pattern)
                .context(format!("Failed to compile regex for pattern: {}", pattern))?;
            
            self.comment_regexes.insert(pattern.clone(), regex);
        }

        // Compile file type patterns
        let file_type_patterns = vec![
            ("rust", r"\.rs$"),
            ("python", r"\.py$"),
            ("javascript", r"\.js$"),
            ("typescript", r"\.ts$"),
            ("java", r"\.java$"),
            ("cpp", r"\.(cpp|cxx|cc)$"),
            ("c", r"\.c$"),
            ("header", r"\.(h|hpp|hxx)$"),
            ("go", r"\.go$"),
            ("markdown", r"\.(md|markdown)$"),
            ("yaml", r"\.(yaml|yml)$"),
        ];

        for (lang, pattern) in file_type_patterns {
            let regex = Regex::new(pattern)
                .context(format!("Failed to compile file type regex for: {}", lang))?;
            self.file_type_regexes.push((lang.to_string(), regex));
        }

        println!("Compiled {} comment patterns and {} file type patterns",
              self.comment_regexes.len(), self.file_type_regexes.len());

        Ok(())
    }

    /// Scan a project directory for TODO comments
    pub fn scan_project(&self, project_path: &Path) -> Result<TodoScanResult> {
        let start_time = std::time::Instant::now();
        println!("Starting TODO scan for project: {}", project_path.display());

        let mut todo_items = Vec::new();
        let mut files_scanned = 0;
        let mut total_lines_scanned = 0u64;
        let mut file_types_scanned = std::collections::HashSet::new();
        let mut scan_errors = Vec::new();

        // Find all source files to scan
        let source_files = self.find_source_files(project_path)
            .context("Failed to find source files")?;

        for file_path in source_files {
            match self.scan_file(&file_path) {
                Ok(file_result) => {
                    files_scanned += 1;
                    total_lines_scanned += file_result.lines_scanned as u64;
                    todo_items.extend(file_result.todo_items);
                    
                    if let Some(file_type) = self.determine_file_type(&file_path) {
                        file_types_scanned.insert(file_type);
                    }
                }
                Err(e) => {
                    let error_msg = format!("Failed to scan {}: {}", file_path.display(), e);
                    println!("Warning: {}", error_msg);
                    scan_errors.push(error_msg);
                }
            }
        }

        let scan_duration_ms = start_time.elapsed().as_millis() as u64;

        // Generate summary
        let summary = self.generate_summary(&todo_items);

        let scan_metadata = TodoScanMetadata {
            files_scanned,
            total_lines_scanned,
            scan_duration_ms,
            file_types_scanned: file_types_scanned.into_iter().collect(),
            excluded_patterns: self.config.exclude_file_patterns.clone(),
            scan_errors,
        };

        println!("TODO scan completed: {} items found in {} files ({} lines) in {}ms",
              todo_items.len(), files_scanned, total_lines_scanned, scan_duration_ms);

        Ok(TodoScanResult {
            todo_items,
            summary,
            scan_metadata,
        })
    }

    /// Find all source files matching include patterns
    fn find_source_files(&self, project_path: &Path) -> Result<Vec<PathBuf>> {
        let mut source_files = Vec::new();
        
        self.walk_directory(project_path, &mut source_files, 0)?;

        // Sort files for consistent ordering
        source_files.sort();
        
        // Debug: Found source files to scan
        Ok(source_files)
    }

    /// Recursively walk directory to find source files
    fn walk_directory(&self, dir: &Path, files: &mut Vec<PathBuf>, depth: u32) -> Result<()> {
        if depth > self.config.max_scan_depth {
            return Ok(());
        }

        if !dir.is_dir() {
            return Ok(());
        }

        let entries = fs::read_dir(dir)
            .context(format!("Failed to read directory: {}", dir.display()))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            // Skip excluded paths
            if self.should_exclude_path(&path) {
                continue;
            }

            if path.is_dir() {
                self.walk_directory(&path, files, depth + 1)?;
            } else if self.should_include_file(&path) {
                // Check file size
                if let Ok(metadata) = fs::metadata(&path) {
                    let size_mb = metadata.len() / (1024 * 1024);
                    if size_mb <= self.config.max_file_size_mb {
                        files.push(path);
                    } else {
                        // Debug: Skipping large file
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if a path should be excluded from scanning
    fn should_exclude_path(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        
        for pattern in &self.config.exclude_file_patterns {
            if self.matches_glob_pattern(&path_str, pattern) {
                return true;
            }
        }
        
        false
    }

    /// Check if a file should be included in scanning
    fn should_include_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        
        for pattern in &self.config.include_file_patterns {
            if self.matches_glob_pattern(&path_str, pattern) {
                return true;
            }
        }
        
        false
    }

    /// Simple glob pattern matching
    fn matches_glob_pattern(&self, text: &str, pattern: &str) -> bool {
        // Simplified glob matching - in production use the `glob` crate
        if pattern.contains('*') {
            let pattern_parts: Vec<&str> = pattern.split('*').collect();
            if pattern_parts.len() == 2 {
                text.starts_with(pattern_parts[0]) && text.ends_with(pattern_parts[1])
            } else {
                false
            }
        } else {
            text.ends_with(pattern)
        }
    }

    /// Determine file type from file extension
    fn determine_file_type(&self, file_path: &Path) -> Option<String> {
        let path_str = file_path.to_string_lossy();
        
        for (file_type, regex) in &self.file_type_regexes {
            if regex.is_match(&path_str) {
                return Some(file_type.clone());
            }
        }
        
        None
    }

    /// Scan a single file for TODO comments
    fn scan_file(&self, file_path: &Path) -> Result<FileScanResult> {
        // Debug: Scanning file

        let content = fs::read_to_string(file_path)
            .context(format!("Failed to read file: {}", file_path.display()))?;

        let lines: Vec<&str> = content.lines().collect();
        let mut todo_items = Vec::new();

        for (line_num, line) in lines.iter().enumerate() {
            for (pattern, regex) in &self.comment_regexes {
                if let Some(captures) = regex.captures(line) {
                    let description = captures.get(1)
                        .map(|m| m.as_str().trim().to_string())
                        .unwrap_or_else(|| "No description".to_string());

                    let todo_type = TodoType::from_str(pattern);
                    let priority = TodoPriority::from_todo_type_and_context(&todo_type, &description);
                    
                    let context_lines = self.extract_context_lines(&lines, line_num);
                    let (author, date_added) = self.extract_author_and_date(&description);
                    let tags = self.extract_tags(&description);

                    todo_items.push(ActualTodoItem {
                        file_path: file_path.to_path_buf(),
                        line_number: (line_num + 1) as u32,
                        column_number: Some(line.find(pattern).unwrap_or(0) as u32),
                        todo_type,
                        description,
                        priority,
                        author,
                        date_added,
                        context_lines,
                        tags,
                    });

                    break; // Only match first pattern per line
                }
            }
        }

        Ok(FileScanResult {
            todo_items,
            lines_scanned: lines.len(),
        })
    }

    /// Extract context lines around a TODO comment
    fn extract_context_lines(&self, lines: &[&str], todo_line: usize) -> Vec<String> {
        let mut context = Vec::new();
        
        let start = todo_line.saturating_sub(self.config.context_lines_before as usize);
        let end = std::cmp::min(
            todo_line + self.config.context_lines_after as usize + 1,
            lines.len()
        );

        for i in start..end {
            if i != todo_line {
                context.push(lines[i].to_string());
            }
        }

        context
    }

    /// Extract author and date from TODO description
    fn extract_author_and_date(&self, description: &str) -> (Option<String>, Option<String>) {
        // Look for patterns like "@author" or "(author, 2023-01-01)"
        let author_regex = Regex::new(r"@([a-zA-Z0-9_-]+)").ok();
        let date_regex = Regex::new(r"(\d{4}-\d{2}-\d{2})").ok();
        
        let author = author_regex
            .and_then(|r| r.captures(description))
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string());
        
        let date = date_regex
            .and_then(|r| r.captures(description))
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string());

        (author, date)
    }

    /// Extract tags from TODO description  
    fn extract_tags(&self, description: &str) -> Vec<String> {
        let tag_regex = Regex::new(r"#([a-zA-Z0-9_-]+)").ok();
        
        if let Some(regex) = tag_regex {
            regex.find_iter(description)
                .map(|m| m.as_str()[1..].to_string()) // Remove the '#'
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Generate summary statistics for TODO items
    fn generate_summary(&self, todo_items: &[ActualTodoItem]) -> TodoSummary {
        let mut by_type = HashMap::new();
        let mut by_priority = HashMap::new();
        let mut by_file_type = HashMap::new();
        let mut file_todo_counts = HashMap::new();

        for item in todo_items {
            *by_type.entry(item.todo_type.clone()).or_insert(0) += 1;
            *by_priority.entry(item.priority.clone()).or_insert(0) += 1;
            
            if let Some(file_type) = self.determine_file_type(&item.file_path) {
                *by_file_type.entry(file_type).or_insert(0) += 1;
            }

            *file_todo_counts.entry(item.file_path.clone()).or_insert(0) += 1;
        }

        // Find most problematic files
        let mut file_scores: Vec<_> = file_todo_counts.into_iter().collect();
        file_scores.sort_by(|a, b| b.1.cmp(&a.1));
        
        let most_problematic_files = file_scores.into_iter()
            .take(10)
            .map(|(path, count)| {
                let high_priority_count = todo_items.iter()
                    .filter(|item| item.file_path == path && 
                           matches!(item.priority, TodoPriority::High | TodoPriority::Critical))
                    .count();

                // Calculate TODO density (would need file line count)
                let todo_density = count as f32; // Simplified

                ProblematicFile {
                    file_path: path,
                    todo_count: count,
                    high_priority_count,
                    file_size_lines: None, // Could be calculated
                    todo_density,
                }
            })
            .collect();

        TodoSummary {
            total_items: todo_items.len(),
            by_type,
            by_priority,
            by_file_type,
            most_problematic_files,
        }
    }
}

/// Result from scanning a single file
#[derive(Debug)]
struct FileScanResult {
    todo_items: Vec<ActualTodoItem>,
    lines_scanned: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_todo_type_from_str() {
        assert_eq!(TodoType::from_str("TODO"), TodoType::Todo);
        assert_eq!(TodoType::from_str("fixme"), TodoType::Fixme);
        assert_eq!(TodoType::from_str("CUSTOM"), TodoType::Custom("CUSTOM".to_string()));
    }

    #[test]  
    fn test_priority_determination() {
        let todo_type = TodoType::Todo;
        let context = "This is a critical security issue";
        let priority = TodoPriority::from_todo_type_and_context(&todo_type, context);
        assert_eq!(priority, TodoPriority::Critical);

        let context2 = "Minor improvement needed";
        let priority2 = TodoPriority::from_todo_type_and_context(&todo_type, context2);
        assert_eq!(priority2, TodoPriority::Medium);
    }

    #[test]
    fn test_pattern_compilation() {
        let scanner = TodoScanner::new().unwrap();
        assert!(!scanner.comment_regexes.is_empty());
        assert!(!scanner.file_type_regexes.is_empty());
    }

    #[test]
    fn test_file_type_detection() {
        let scanner = TodoScanner::new().unwrap();
        let rust_file = PathBuf::from("src/main.rs");
        let file_type = scanner.determine_file_type(&rust_file);
        assert_eq!(file_type, Some("rust".to_string()));
    }
}