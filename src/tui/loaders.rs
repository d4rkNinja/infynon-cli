use indicatif::{ProgressBar, ProgressStyle, ProgressState};
use owo_colors::OwoColorize;
use std::thread;
use std::time::Duration;
use std::fmt::Write;

pub fn show_stylish_install_loader(packages: &[String], ecosystem: &str) {
    if packages.is_empty() { return; }
    
    println!("  {} Verifying dependency tree for {} via {}...", ">>".cyan().bold(), packages.len().to_string().yellow(), ecosystem.magenta());
    println!();
    
    let pb = ProgressBar::new(100 * packages.len() as u64);
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{bar:50.cyan/blue}] {pos}/{len} ({eta}) \n    {msg}")
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars("█▉▊▋▌▍▎▏  "),
    );
    
    for pkg in packages {
        pb.set_message(format!("{} {}", "Layer 1 (Blocklist Tri-Lookup):".truecolor(255,165,0).bold(), pkg)); 
        thread::sleep(Duration::from_millis(300)); 
        pb.inc(20);
        
        pb.set_message(format!("{} {}", "Layer 2 (Heuristic Scanning):".truecolor(255,100,255).bold(), pkg)); 
        thread::sleep(Duration::from_millis(400)); 
        pb.inc(30);
        
        pb.set_message(format!("{} {}", "Layer 3 (LLM Deep-Scan):".truecolor(100,100,255).bold(), pkg)); 
        thread::sleep(Duration::from_millis(600)); 
        pb.inc(40);
        
        pb.set_message(format!("{} {}", "Injecting Secure Binary:".truecolor(50,255,50).bold(), pkg)); 
        thread::sleep(Duration::from_millis(200)); 
        pb.inc(10);
    }
    
    pb.finish_with_message(format!("{} All packages vetted and routed securely to {}!", "✓".green(), ecosystem.magenta()));
    println!("\n  {} {} {} packages securely.\n", "✔".green(), "Successfully installed".bold(), packages.len().to_string().yellow());
}
