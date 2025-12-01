use anyhow::Result;
use clap::Subcommand;
use xshell::Shell;

pub mod arduino;
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
    /// Arduino commands
    Arduino(arduino::Arduino),
}

impl Command {
    pub fn run(self, sh: &Shell) -> Result<()> {
        match self {
            Command::Build(cmd) => cmd.run(sh),
            Command::Test(cmd) => cmd.run(sh),
            Command::Visualizer(cmd) => cmd.run(sh),
            Command::Arduino(cmd) => cmd.run(sh),
        }
    }
}
