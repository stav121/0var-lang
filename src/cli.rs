//! Command-line interface for the zvar compiler

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// The zvar programming language compiler
#[derive(Parser)]
#[command(name = "zvar")]
#[command(about = "A bytecode programming language that eliminates naming")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Disable colored output
    #[arg(long)]
    pub no_color: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Compile and run a zvar program
    Run {
        /// Input file to run (.zvar or .0var)
        file: PathBuf,

        /// Show bytecode disassembly
        #[arg(long)]
        disasm: bool,

        /// Show debug information
        #[arg(long)]
        debug: bool,
    },

    /// Compile a zvar program to bytecode
    Compile {
        /// Input file to compile (.zvar or .0var)
        file: PathBuf,

        /// Output file (optional)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Show bytecode disassembly
        #[arg(long)]
        disasm: bool,
    },

    /// Check syntax without compiling
    Check {
        /// Input file to check (.zvar or .0var)
        file: PathBuf,
    },

    /// Show information about entities in a program
    Info {
        /// Input file to analyze (.zvar or .0var)
        file: PathBuf,

        /// Show only documentation
        #[arg(long)]
        docs_only: bool,
    },

    /// Interactive REPL mode
    Repl {
        /// Show bytecode for each expression
        #[arg(long)]
        show_bytecode: bool,
    },
}

impl Cli {
    /// Parse command line arguments
    pub fn parse_args() -> Self {
        Cli::parse()
    }

    /// Get the input file path if available
    pub fn input_file(&self) -> Option<&PathBuf> {
        match &self.command {
            Commands::Run { file, .. } => Some(file),
            Commands::Compile { file, .. } => Some(file),
            Commands::Check { file } => Some(file),
            Commands::Info { file, .. } => Some(file),
            Commands::Repl { .. } => None,
        }
    }

    /// Check if debug output is requested
    pub fn debug_mode(&self) -> bool {
        self.verbose || matches!(&self.command, Commands::Run { debug: true, .. })
    }

    /// Check if disassembly is requested
    pub fn show_disasm(&self) -> bool {
        matches!(
            &self.command,
            Commands::Run { disasm: true, .. } | Commands::Compile { disasm: true, .. }
        )
    }

    /// Validate that the input file has a supported extension
    pub fn validate_file_extension(&self) -> Result<(), String> {
        if let Some(file) = self.input_file() {
            let extension = file.extension().and_then(|ext| ext.to_str()).unwrap_or("");

            match extension {
                "zvar" | "0var" => Ok(()),
                "" => Err("No file extension provided. Expected .zvar or .0var".to_string()),
                _ => Err(format!(
                    "Unsupported file extension '.{}'. Expected .zvar or .0var",
                    extension
                )),
            }
        } else {
            Ok(()) // No file needed (e.g., REPL mode)
        }
    }

    /// Get a human-readable description of supported file types
    pub fn supported_extensions() -> &'static str {
        ".zvar or .0var"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_verification() {
        // This ensures our CLI structure is valid
        Cli::command().debug_assert();
    }

    #[test]
    fn test_input_file_extraction() {
        let cli = Cli {
            command: Commands::Run {
                file: PathBuf::from("test.zvar"),
                disasm: false,
                debug: false,
            },
            verbose: false,
            no_color: false,
        };

        assert_eq!(cli.input_file(), Some(&PathBuf::from("test.zvar")));
        assert!(!cli.debug_mode());
        assert!(!cli.show_disasm());
        assert!(cli.validate_file_extension().is_ok());
    }

    #[test]
    fn test_file_extension_validation() {
        let cli_zvar = Cli {
            command: Commands::Run {
                file: PathBuf::from("test.zvar"),
                disasm: false,
                debug: false,
            },
            verbose: false,
            no_color: false,
        };
        assert!(cli_zvar.validate_file_extension().is_ok());

        let cli_0var = Cli {
            command: Commands::Run {
                file: PathBuf::from("test.0var"),
                disasm: false,
                debug: false,
            },
            verbose: false,
            no_color: false,
        };
        assert!(cli_0var.validate_file_extension().is_ok());

        let cli_invalid = Cli {
            command: Commands::Run {
                file: PathBuf::from("test.txt"),
                disasm: false,
                debug: false,
            },
            verbose: false,
            no_color: false,
        };
        assert!(cli_invalid.validate_file_extension().is_err());
    }
}
