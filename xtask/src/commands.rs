use anyhow::Result;
use clap::Subcommand;
use xshell::Shell;

pub mod build;
pub mod test;
pub mod visualizer;

#[derive(Subcommand)]
pub enum Command {
    /// Build all components
    Build(build::Build),
    /// Run tests
    Test(test::Test),
    /// Visualizer commands
    Visualizer(visualizer::Visualizer),
}

impl Command {
    pub fn run(self, sh: &Shell) -> Result<()> {
        match self {
            Command::Build(cmd) => cmd.run(sh),
            Command::Test(cmd) => cmd.run(sh),
            Command::Visualizer(cmd) => cmd.run(sh),
        }
    }
}
