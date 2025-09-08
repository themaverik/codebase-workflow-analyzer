use std::path::Path;
use anyhow::{Result, Context as AnyhowContext};
use walkdir;

use crate::core::types::{AstSegment, Framework, Language};
use crate::core::context_types::{ProjectContext, SegmentContext, EnhancedSegmentContext};
use crate::core::hierarchical_context_manager::HierarchicalContextManager;
use crate::core::ast_analyzer::{ASTAnalyzer, CodeSegment};
use crate::core::config::Config;

pub struct ContextAwareASTAnalyzer {
    ast_analyzer: ASTAnalyzer,
    context_manager: HierarchicalContextManager,
    config: Config,
}

impl ContextAwareASTAnalyzer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            ast_analyzer: ASTAnalyzer::new()?,
            context_manager: HierarchicalContextManager::new(), // new() doesn't return Result
            config: Config::instance(),
        })
    }

    pub async fn initialize(&mut self, project_path: &Path) -> Result<()> {
        self.context_manager.initialize(project_path).await
            .with_context(|| "Failed to initialize context manager")?;
        Ok(())
    }

    pub async fn extract_segments_with_context(
        &mut self, 
        project_path: &Path
    ) -> Result<ContextAwareSegmentExtractionResult> {
        let start_time = std::time::Instant::now();

        let traditional_segments = self.extract_traditional_segments(project_path).await?;
        
        let enhanced_segments = self.enhance_segments_with_context(traditional_segments).await?;
        
        let extraction_time = start_time.elapsed();
        let segments_count = enhanced_segments.len();
        let context_awareness = self.calculate_context_awareness_score(&enhanced_segments);
        
        Ok(ContextAwareSegmentExtractionResult {
            enhanced_segments,
            extraction_metadata: ExtractionMetadata {
                total_files_processed: self.count_processed_files(project_path).await?,
                total_segments_extracted: segments_count,
                extraction_time_ms: extraction_time.as_millis() as u64,
                context_awareness_score: context_awareness,
            },
        })
    }

    async fn extract_traditional_segments(&mut self, project_path: &Path) -> Result<Vec<AstSegment>> {
        let mut segments = Vec::new();
        
        // Use empty frameworks list since we'll enhance detection later
        let frameworks = Vec::new();
        let code_segments = self.ast_analyzer.extract_segments(project_path, &frameworks)?;
        
        for code_segment in &code_segments {
            let ast_segment = self.convert_code_segment_to_ast_segment(code_segment, project_path)?;
            segments.push(ast_segment);
        }
        
        Ok(segments)
    }

    async fn enhance_segments_with_context(
        &mut self, 
        segments: Vec<AstSegment>
    ) -> Result<Vec<EnhancedSegmentContext>> {
        let mut enhanced_segments = Vec::new();
        
        for segment in segments {
            match self.context_manager.build_enhanced_segment_context(&segment).await {
                Ok(enhanced_context) => {
                    enhanced_segments.push(enhanced_context);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to enhance segment context for {}: {}", 
                        segment.file_path.display(), e);
                    continue;
                }
            }
        }
        
        Ok(enhanced_segments)
    }

    fn convert_code_segment_to_ast_segment(
        &self, 
        code_segment: &CodeSegment, 
        project_path: &Path
    ) -> Result<AstSegment> {
        let file_path = if code_segment.metadata.file_path.is_absolute() {
            code_segment.metadata.file_path.clone()
        } else {
            project_path.join(&code_segment.metadata.file_path)
        };

        let segment_type = match &code_segment.segment_type {
            crate::core::ast_analyzer::SegmentType::Function(_) => "function",
            crate::core::ast_analyzer::SegmentType::Class(_) => "class",
            crate::core::ast_analyzer::SegmentType::Interface(_) => "interface",
            crate::core::ast_analyzer::SegmentType::Route(_) => "route",
            crate::core::ast_analyzer::SegmentType::Configuration(_) => "configuration",
            crate::core::ast_analyzer::SegmentType::Database(_) => "database",
        }.to_string();

        let language = self.derive_language_from_path(&file_path);
        
        Ok(AstSegment {
            file_path,
            start_line: code_segment.metadata.line_start,
            end_line: code_segment.metadata.line_end,
            segment_type,
            content: code_segment.content.clone(),
            language,
        })
    }

    fn derive_language_from_path(&self, file_path: &Path) -> String {
        if let Some(extension) = file_path.extension().and_then(|e| e.to_str()) {
            match extension {
                "ts" | "tsx" => "TypeScript".to_string(),
                "js" | "jsx" => "JavaScript".to_string(),
                "py" => "Python".to_string(),
                "java" => "Java".to_string(),
                "rs" => "Rust".to_string(),
                "go" => "Go".to_string(),
                _ => "TypeScript".to_string(), // Default fallback
            }
        } else {
            "TypeScript".to_string() // Default fallback
        }
    }

    async fn count_processed_files(&self, project_path: &Path) -> Result<usize> {
        let mut count = 0;
        let walker = walkdir::WalkDir::new(project_path);
        
        for entry in walker {
            let entry = entry?;
            if entry.file_type().is_file() {
                if let Some(extension) = entry.path().extension() {
                    if matches!(extension.to_str(), Some("rs") | Some("ts") | Some("js") | Some("py") | Some("java")) {
                        count += 1;
                    }
                }
            }
        }
        
        Ok(count)
    }

    fn calculate_context_awareness_score(&self, enhanced_segments: &[EnhancedSegmentContext]) -> f32 {
        if enhanced_segments.is_empty() {
            return 0.0;
        }

        let mut total_score = 0.0;
        
        for segment in enhanced_segments {
            let mut segment_score = 0.0;
            
            segment_score += if !segment.project_context.business_domains.is_empty() { 0.3 } else { 0.0 };
            
            segment_score += if !segment.business_hints.is_empty() { 0.3 } else { 0.0 };
            
            segment_score += if !segment.related_segments.is_empty() { 0.2 } else { 0.0 };
            
            segment_score += if !segment.cross_references.is_empty() { 0.2 } else { 0.0 };
            
            total_score += segment_score;
        }
        
        total_score / enhanced_segments.len() as f32
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContextAwareSegmentExtractionResult {
    pub enhanced_segments: Vec<EnhancedSegmentContext>,
    pub extraction_metadata: ExtractionMetadata,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtractionMetadata {
    pub total_files_processed: usize,
    pub total_segments_extracted: usize,
    pub extraction_time_ms: u64,
    pub context_awareness_score: f32,
}

pub struct ContextAwareFusionEngine {
    weights: FusionWeights,
    config: Config,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FusionWeights {
    pub project_context: f32,
    pub traditional_patterns: f32,
    pub ast_structure: f32,
    pub llm_semantic: f32,
}

impl Default for FusionWeights {
    fn default() -> Self {
        Self {
            project_context: 0.4,
            traditional_patterns: 0.3,
            ast_structure: 0.2,
            llm_semantic: 0.1,
        }
    }
}

impl ContextAwareFusionEngine {
    pub fn new() -> Self {
        Self {
            weights: FusionWeights::default(),
            config: Config::instance(),
        }
    }

    pub fn with_weights(weights: FusionWeights) -> Self {
        Self {
            weights,
            config: Config::instance(),
        }
    }

    pub async fn fuse_context_aware_analysis(
        &self,
        enhanced_segments: &[EnhancedSegmentContext]
    ) -> Result<FusedAnalysisResult> {
        let start_time = std::time::Instant::now();
        
        let mut fused_segments = Vec::new();
        let mut confidence_breakdown = ConfidenceBreakdown::new();
        
        for segment in enhanced_segments {
            let fused_segment = self.fuse_single_segment(segment)?;
            confidence_breakdown.add_segment_confidence(&fused_segment);
            fused_segments.push(fused_segment);
        }
        
        let fusion_time = start_time.elapsed();
        let average_confidence = confidence_breakdown.calculate_average_confidence();
        
        Ok(FusedAnalysisResult {
            fused_segments,
            confidence_breakdown,
            fusion_metadata: FusionMetadata {
                fusion_time_ms: fusion_time.as_millis() as u64,
                segments_processed: enhanced_segments.len(),
                average_confidence,
            },
        })
    }

    fn fuse_single_segment(&self, segment: &EnhancedSegmentContext) -> Result<FusedSegmentAnalysis> {
        let project_confidence = self.calculate_project_context_confidence(&segment.project_context);
        
        let structure_confidence = self.calculate_structural_confidence(&segment.segment_context);
        
        let business_confidence = self.calculate_business_context_confidence(&segment.business_hints);
        
        let architectural_confidence = self.calculate_architectural_confidence(&segment.architectural_context);
        
        let fused_confidence = 
            project_confidence * self.weights.project_context +
            structure_confidence * self.weights.ast_structure +
            business_confidence * self.weights.traditional_patterns +
            architectural_confidence * self.weights.llm_semantic;

        Ok(FusedSegmentAnalysis {
            segment_id: segment.segment_context.segment_id.clone(),
            fused_confidence,
            confidence_components: ConfidenceComponents {
                project_context: project_confidence,
                structural: structure_confidence,
                business: business_confidence,
                architectural: architectural_confidence,
            },
            business_domains: self.extract_business_domains(segment),
            architectural_patterns: segment.architectural_context.patterns.clone(),
            quality_score: self.calculate_quality_score(fused_confidence, segment),
        })
    }

    fn calculate_project_context_confidence(&self, project_context: &ProjectContext) -> f32 {
        let mut confidence: f32 = 0.0;
        
        if project_context.confidence > 0.7 {
            confidence += 0.4;
        } else if project_context.confidence > 0.5 {
            confidence += 0.3;
        } else if project_context.confidence > 0.3 {
            confidence += 0.2;
        }
        
        if !project_context.business_domains.is_empty() {
            confidence += 0.3;
        }
        
        if !project_context.entry_points.is_empty() {
            confidence += 0.2;
        }
        
        if !project_context.dependency_overview.direct_dependencies.is_empty() {
            confidence += 0.1;
        }
        
        confidence.min(1.0f32)
    }

    fn calculate_structural_confidence(&self, segment_context: &SegmentContext) -> f32 {
        let mut confidence: f32 = 0.0;
        
        confidence += segment_context.confidence * 0.6;
        
        if segment_context.business_purpose.is_some() {
            confidence += 0.2;
        }
        
        if !segment_context.dependencies.is_empty() {
            confidence += 0.1;
        }
        
        if !segment_context.dependents.is_empty() {
            confidence += 0.1;
        }
        
        confidence.min(1.0f32)
    }

    fn calculate_business_context_confidence(&self, business_hints: &[String]) -> f32 {
        if business_hints.is_empty() {
            0.0
        } else {
            (business_hints.len() as f32 * 0.2).min(1.0)
        }
    }

    fn calculate_architectural_confidence(&self, architectural_context: &crate::core::context_types::ArchitecturalContext) -> f32 {
        let mut confidence: f32 = 0.0;
        
        confidence += match architectural_context.layer {
            crate::core::context_types::ArchitecturalLayer::Presentation => 0.3,
            crate::core::context_types::ArchitecturalLayer::Business => 0.4,
            crate::core::context_types::ArchitecturalLayer::Data => 0.3,
            crate::core::context_types::ArchitecturalLayer::Infrastructure => 0.2,
            crate::core::context_types::ArchitecturalLayer::Cross => 0.1,
        };
        
        if !architectural_context.patterns.is_empty() {
            confidence += 0.3;
        }
        
        if !architectural_context.responsibilities.is_empty() {
            confidence += 0.2;
        }
        
        confidence.min(1.0f32)
    }

    fn extract_business_domains(&self, segment: &EnhancedSegmentContext) -> Vec<String> {
        let mut domains = Vec::new();
        
        for domain in &segment.project_context.business_domains {
            domains.push(domain.name.clone());
        }
        
        for hint in &segment.business_hints {
            if !domains.contains(hint) {
                domains.push(hint.clone());
            }
        }
        
        domains
    }

    fn calculate_quality_score(&self, fused_confidence: f32, segment: &EnhancedSegmentContext) -> f32 {
        let mut quality = fused_confidence * 0.7;
        
        if !segment.cross_references.is_empty() {
            quality += 0.1;
        }
        
        if !segment.related_segments.is_empty() {
            quality += 0.1;
        }
        
        if segment.segment_context.file_context.business_relevance > 0.7 {
            quality += 0.1;
        }
        
        quality.min(1.0)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FusedAnalysisResult {
    pub fused_segments: Vec<FusedSegmentAnalysis>,
    pub confidence_breakdown: ConfidenceBreakdown,
    pub fusion_metadata: FusionMetadata,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FusedSegmentAnalysis {
    pub segment_id: String,
    pub fused_confidence: f32,
    pub confidence_components: ConfidenceComponents,
    pub business_domains: Vec<String>,
    pub architectural_patterns: Vec<String>,
    pub quality_score: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfidenceComponents {
    pub project_context: f32,
    pub structural: f32,
    pub business: f32,
    pub architectural: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfidenceBreakdown {
    pub high_confidence_segments: usize,
    pub medium_confidence_segments: usize,
    pub low_confidence_segments: usize,
    pub total_segments: usize,
}

impl ConfidenceBreakdown {
    fn new() -> Self {
        Self {
            high_confidence_segments: 0,
            medium_confidence_segments: 0,
            low_confidence_segments: 0,
            total_segments: 0,
        }
    }

    fn add_segment_confidence(&mut self, segment: &FusedSegmentAnalysis) {
        self.total_segments += 1;
        
        if segment.fused_confidence > 0.7 {
            self.high_confidence_segments += 1;
        } else if segment.fused_confidence > 0.4 {
            self.medium_confidence_segments += 1;
        } else {
            self.low_confidence_segments += 1;
        }
    }

    pub fn calculate_average_confidence(&self) -> f32 {
        if self.total_segments == 0 {
            return 0.0;
        }
        
        let weighted_sum = 
            self.high_confidence_segments as f32 * 0.85 +
            self.medium_confidence_segments as f32 * 0.55 +
            self.low_confidence_segments as f32 * 0.25;
            
        weighted_sum / self.total_segments as f32
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FusionMetadata {
    pub fusion_time_ms: u64,
    pub segments_processed: usize,
    pub average_confidence: f32,
}