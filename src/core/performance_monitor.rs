use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_analysis_time: Duration,
    pub phase_timings: HashMap<String, Duration>,
    pub file_count: usize,
    pub lines_analyzed: usize,
    pub memory_usage_mb: f64,
    pub cache_hit_rate: f64,
    pub bottlenecks: Vec<PerformanceBottleneck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBottleneck {
    pub phase: String,
    pub duration: Duration,
    pub percentage_of_total: f64,
    pub recommendation: String,
}

pub struct PerformanceMonitor {
    start_time: Instant,
    phase_timers: HashMap<String, Instant>,
    completed_phases: HashMap<String, Duration>,
    file_count: usize,
    lines_analyzed: usize,
    cache_hits: usize,
    cache_misses: usize,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            phase_timers: HashMap::new(),
            completed_phases: HashMap::new(),
            file_count: 0,
            lines_analyzed: 0,
            cache_hits: 0,
            cache_misses: 0,
        }
    }
    
    pub fn start_phase(&mut self, phase_name: &str) {
        // Starting phase: {}
        // (debug output removed)
        self.phase_timers.insert(phase_name.to_string(), Instant::now());
    }
    
    pub fn end_phase(&mut self, phase_name: &str) {
        if let Some(start_time) = self.phase_timers.remove(phase_name) {
            let duration = start_time.elapsed();
            self.completed_phases.insert(phase_name.to_string(), duration);
            // Completed phase: {} in {:.2}s
            // (debug output removed)
        }
    }
    
    pub fn add_files_processed(&mut self, count: usize) {
        self.file_count += count;
    }
    
    pub fn add_lines_analyzed(&mut self, count: usize) {
        self.lines_analyzed += count;
    }
    
    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }
    
    pub fn record_cache_miss(&mut self) {
        self.cache_misses += 1;
    }
    
    pub fn get_memory_usage_mb(&self) -> f64 {
        // Simple memory estimation based on processed data
        let base_memory = 50.0; // Base Rust process memory
        let file_memory = self.file_count as f64 * 0.1; // ~100KB per file
        let line_memory = self.lines_analyzed as f64 * 0.001; // ~1KB per 1000 lines
        
        base_memory + file_memory + line_memory
    }
    
    pub fn get_cache_hit_rate(&self) -> f64 {
        let total_requests = self.cache_hits + self.cache_misses;
        if total_requests == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total_requests as f64
        }
    }
    
    pub fn get_total_duration(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    pub fn identify_bottlenecks(&self, total_time: Duration) -> Vec<PerformanceBottleneck> {
        let mut bottlenecks = Vec::new();
        
        // Find phases taking > 20% of total time
        for (phase, duration) in &self.completed_phases {
            let percentage = duration.as_secs_f64() / total_time.as_secs_f64() * 100.0;
            
            if percentage > 20.0 {
                let recommendation = match phase.as_str() {
                    "Framework Detection" => "Consider caching framework detection results or implementing parallel processing".to_string(),
                    "AST Analysis" => "Enable incremental AST parsing and cache parsed trees".to_string(),
                    "LLM Analysis" => "Optimize prompts, use model caching, or reduce context size".to_string(),
                    "Business Context Extraction" => "Cache business domain classifications and implement fast-path detection".to_string(),
                    _ => "Consider optimization opportunities for this phase".to_string(),
                };
                
                bottlenecks.push(PerformanceBottleneck {
                    phase: phase.clone(),
                    duration: *duration,
                    percentage_of_total: percentage,
                    recommendation,
                });
            }
        }
        
        // Sort by duration (longest first)
        bottlenecks.sort_by(|a, b| b.duration.cmp(&a.duration));
        bottlenecks
    }
    
    pub fn generate_metrics(&self) -> PerformanceMetrics {
        let total_time = self.start_time.elapsed();
        let bottlenecks = self.identify_bottlenecks(total_time);
        
        PerformanceMetrics {
            total_analysis_time: total_time,
            phase_timings: self.completed_phases.clone(),
            file_count: self.file_count,
            lines_analyzed: self.lines_analyzed,
            memory_usage_mb: self.get_memory_usage_mb(),
            cache_hit_rate: self.get_cache_hit_rate(),
            bottlenecks,
        }
    }
    
    pub fn print_summary(&self) {
        let metrics = self.generate_metrics();
        
        println!("\n🚀 Performance Analysis Summary");
        println!("================================");
        println!("📊 Total Analysis Time: {:.2}s", metrics.total_analysis_time.as_secs_f64());
        println!("📁 Files Processed: {}", metrics.file_count);
        println!("📝 Lines Analyzed: {}", metrics.lines_analyzed);
        println!("💾 Memory Usage: {:.1}MB", metrics.memory_usage_mb);
        println!("🎯 Cache Hit Rate: {:.1}%", metrics.cache_hit_rate * 100.0);
        
        if !metrics.phase_timings.is_empty() {
            println!("\n⏱️  Phase Breakdown:");
            let mut phases: Vec<_> = metrics.phase_timings.iter().collect();
            phases.sort_by(|a, b| b.1.cmp(a.1));
            
            for (phase, duration) in phases {
                let percentage = duration.as_secs_f64() / metrics.total_analysis_time.as_secs_f64() * 100.0;
                println!("   {} {:.2}s ({:.1}%)", 
                    format!("{:<25}", phase), 
                    duration.as_secs_f64(), 
                    percentage);
            }
        }
        
        if !metrics.bottlenecks.is_empty() {
            println!("\n🔍 Performance Bottlenecks:");
            for bottleneck in &metrics.bottlenecks {
                println!("   {} ({:.1}%): {}", 
                    bottleneck.phase, 
                    bottleneck.percentage_of_total,
                    bottleneck.recommendation);
            }
        }
        
        // Performance grade
        let grade = self.calculate_performance_grade(&metrics);
        println!("\n🏆 Performance Grade: {} {}", grade.0, grade.1);
    }
    
    fn calculate_performance_grade(&self, metrics: &PerformanceMetrics) -> (&str, &str) {
        let files_per_second = metrics.file_count as f64 / metrics.total_analysis_time.as_secs_f64();
        let cache_hit_rate = metrics.cache_hit_rate;
        
        // Performance thresholds
        match (files_per_second, cache_hit_rate) {
            (fps, chr) if fps > 10.0 && chr > 0.8 => ("A+", "Excellent performance with high cache efficiency"),
            (fps, chr) if fps > 5.0 && chr > 0.6 => ("A", "Good performance with efficient caching"),
            (fps, chr) if fps > 2.0 && chr > 0.4 => ("B", "Acceptable performance, consider optimizations"),
            (fps, chr) if fps > 1.0 && chr > 0.2 => ("C", "Below target performance, optimization needed"),
            _ => ("D", "Poor performance, significant optimization required"),
        }
    }
    
    pub fn meets_performance_targets(&self, file_count: usize) -> bool {
        let total_time = self.start_time.elapsed();
        
        match file_count {
            n if n < 100 => total_time.as_secs() < 30,      // Small projects: <30s
            n if n < 500 => total_time.as_secs() < 120,     // Medium projects: <2min
            _ => total_time.as_secs() < 300,                 // Large projects: <5min
        }
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_phase_timing() {
        let mut monitor = PerformanceMonitor::new();
        
        monitor.start_phase("test_phase");
        thread::sleep(Duration::from_millis(100));
        monitor.end_phase("test_phase");
        
        let metrics = monitor.generate_metrics();
        assert!(metrics.phase_timings.contains_key("test_phase"));
        assert!(metrics.phase_timings["test_phase"].as_millis() >= 100);
    }
    
    #[test]
    fn test_cache_hit_rate() {
        let mut monitor = PerformanceMonitor::new();
        
        monitor.record_cache_hit();
        monitor.record_cache_hit();
        monitor.record_cache_miss();
        
        assert_eq!(monitor.get_cache_hit_rate(), 2.0/3.0);
    }
    
    #[test]
    fn test_bottleneck_identification() {
        let mut monitor = PerformanceMonitor::new();
        
        // Simulate a slow phase
        monitor.completed_phases.insert("Slow Phase".to_string(), Duration::from_secs(30));
        monitor.completed_phases.insert("Fast Phase".to_string(), Duration::from_secs(5));
        
        let bottlenecks = monitor.identify_bottlenecks(Duration::from_secs(40));
        
        assert_eq!(bottlenecks.len(), 1);
        assert_eq!(bottlenecks[0].phase, "Slow Phase");
        assert!(bottlenecks[0].percentage_of_total > 70.0);
    }
}