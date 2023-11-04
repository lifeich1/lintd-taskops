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
    /// Holder for per-project distribution receipt
    Dist,
    /// General addon targets
    Rule {
        #[arg(default_value = "default")]
        target: String,
        rule_options: Vec<String>,
    },
}

pub mod ops;

pub trait Addon {
    /// Command dist
    /// # Errors
    /// User throw errors.
    fn dist() -> Result<()> {
        println!("Warning: Empty dist receipt.");
        Ok(())
    }
    /// Command rule
    /// # Errors
    /// User throw errors.
    fn rule(target: String, options: Vec<String>) -> Result<()> {
        println!("Warning: Empty receipt for target {target}, options: {options:?}");
        Ok(())
    }
}

pub trait Make: Addon {
    /// Main entrypoint.
    ///
    /// # Errors
    /// error if tasks failed.
    fn make() -> Result<()> {
        let cli = Cli::parse();
        match &cli.command {
            Commands::Ci => xtaskops::tasks::ci(),
            Commands::Docs => xtaskops::tasks::docs(),
            Commands::Coverage { dev, neo: false } => xtaskops::tasks::coverage(*dev),
            Commands::Coverage { neo: true, .. } => ops::neo_coverage(),
            Commands::Bump { bump } => ops::bump_version(bump),
            Commands::Publish => ops::publish(),
            Commands::Dist => Self::dist(),
            Commands::Rule {
                target,
                rule_options,
            } => Self::rule(target.clone(), rule_options.clone()),
        }
    }
}

impl<T> Make for T where T: Addon {}

struct MakeImpl();
impl Addon for MakeImpl {}

/// Main entrypoint.
///
/// # Errors
/// error if tasks failed.
pub fn make() -> Result<()> {
    MakeImpl::make()
}
