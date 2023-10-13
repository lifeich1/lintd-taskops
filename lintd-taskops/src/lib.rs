use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Coverage with tech instrument-coverage, deps llvm-tools-preview & grcov
    Coverage {
        /// generate an html report
        #[arg(short, long, default_value_t = false)]
        dev: bool,
        /// coveralls to stdout for neovim plugin
        #[arg(long, default_value_t = false)]
        neo: bool,
    },
    /// Regular ci tests: fmt, clippy, test
    Ci,
    /// Run cargo docs in watch mode
    Docs,
    /// Bump version and commit
    Bump {
        /// increment version part
        #[arg(default_value = "patch")]
        bump: String,
    },
    /// Publish and bump version auto commit
    Publish,
}

pub mod ops;

/// Main entrypoint.
///
/// # Errors
/// error if tasks failed.
pub fn make() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Ci => xtaskops::tasks::ci(),
        Commands::Docs => xtaskops::tasks::docs(),
        Commands::Coverage { dev, neo: false } => xtaskops::tasks::coverage(*dev),
        Commands::Coverage { neo: true, .. } => ops::neo_coverage(),
        Commands::Bump { bump } => ops::bump_version(bump),
        Commands::Publish => ops::publish(),
    }
}
