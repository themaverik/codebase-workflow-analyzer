use codebase_analyzer::core::self_analysis_test::analyze_self;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Starting Phase 1 SOTA Implementation Test...\n");
    
    let start_time = std::time::Instant::now();
    
    match analyze_self().await {
        Ok(()) => {
            let duration = start_time.elapsed();
            println!("\n🎉 Phase 1 Analysis Complete!");
            println!("⏱️  Duration: {:.2?}", duration);
            println!("\nThis demonstrates the new context-aware project analysis");
            println!("that fixes the segment myopia issue described in the TODO file.");
        }
        Err(e) => {
            eprintln!("❌ Analysis failed: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}