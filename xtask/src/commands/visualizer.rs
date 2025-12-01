use anyhow::{Context, Result};
use clap::Args;
use xshell::{Shell, cmd};

#[derive(Args)]
pub struct Visualizer {
    #[command(subcommand)]
    command: VisualizerCommand,
}

#[derive(clap::Subcommand)]
enum VisualizerCommand {
    /// Build the visualizer for production
    Build,
}

impl Visualizer {
    pub fn run(&self, sh: &Shell) -> Result<()> {
        match &self.command {
            VisualizerCommand::Build => build_visualizer(sh),
        }
    }
}

fn build_visualizer(sh: &Shell) -> Result<()> {
    // Check if wasm-pack is installed
    let has_wasm_pack = cmd!(sh, "which wasm-pack")
        .ignore_status()
        .quiet()
        .run()
        .is_ok();

    if !has_wasm_pack {
        println!("wasm-pack not found, installing...");
        cmd!(sh, "cargo install wasm-pack").run()?;
    }

    // Build the wasm package
    println!("Building WASM package...");
    let _dir = sh.push_dir("visualizer");
    cmd!(sh, "wasm-pack build --target web --out-dir web/pkg")
        .run()
        .context("Failed to build WASM package")?;

    // Build the web app
    println!("Building web app...");
    let _dir = sh.push_dir("web");
    cmd!(sh, "npm install").run()?;
    cmd!(sh, "npm run build")
        .run()
        .context("Failed to build web app")?;

    println!("✓ Visualizer built successfully at visualizer/web/dist");
    Ok(())
}
