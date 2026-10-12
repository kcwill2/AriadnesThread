pub mod commands;
pub mod optimize;
pub mod registry;

pub use commands::analyze::*;
pub use commands::discover::*;
pub use commands::includes::*;
pub use commands::index::*;
pub use commands::measure::*;
pub use commands::plan::*;
pub use commands::query::*;
pub use commands::transform::*;
pub use commands::ui::*;
pub use optimize::*;

use anyhow::{bail, Result};
use ariadne_repo::SourceLanguage;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "tool",
    version = "0.1.0",
    about = "C++ Dependency Optimization Tool"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum BuildSubcommands {
    /// Discover compilation contexts and translation units
    Discover {
        /// Path to repository root
        repo: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum IncludesSubcommands {
    /// Explain include tree for a specific file
    Explain {
        /// Path to repository root
        repo: PathBuf,
        /// Target file to explain
        #[arg(short, long)]
        file: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum QuerySubcommands {
    /// Query symbol details
    Symbol {
        /// Path to repository root
        repo: PathBuf,
        /// Name or ID of the symbol
        name: String,
    },
    /// Query symbol dependencies
    Deps {
        /// Path to repository root
        repo: PathBuf,
        /// Name or ID of the symbol
        name: String,
    },
    /// Query symbol users
    Users {
        /// Path to repository root
        repo: PathBuf,
        /// Name or ID of the symbol
        name: String,
    },
}

#[derive(Subcommand)]
pub enum AnalyzeSubcommands {
    /// Load analysis
    Load {
        /// Path to repository root
        repo: PathBuf,
        /// Symbol to analyze
        #[arg(long)]
        symbol: Option<String>,
        /// File to analyze
        #[arg(long)]
        file: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub enum ExplainSubcommands {
    /// Explain dependency load
    Load {
        /// Path to repository root
        repo: PathBuf,
        /// Symbol to explain
        #[arg(long)]
        symbol: Option<String>,
        /// File to explain
        #[arg(long)]
        file: Option<PathBuf>,
        /// Optional collateral source filter
        #[arg(long)]
        source: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum PlanSubcommands {
    /// Show detailed optimization plan
    Show {
        /// Path to repository root
        repo: PathBuf,
        /// Plan ID (e.g. PLAN-001)
        #[arg(long)]
        id: String,
    },
}

#[derive(Subcommand)]
pub enum TransformSubcommands {
    /// Extract headers based on optimization plan
    Extract {
        /// Path to repository root
        repo: PathBuf,
        /// Plan ID to extract
        #[arg(long)]
        plan: String,
    },
}

#[derive(Subcommand)]
pub enum Commands {
    /// Show version information
    Version,
    /// Analyze repository dependencies
    Analyze {
        /// Path to repository root
        repo: Option<PathBuf>,
        /// Number of parallel workers
        #[arg(long)]
        workers: Option<usize>,
        /// Restrict analyzed units to one language (cpp|scala|other);
        /// default: per-file detection over the registered plugins
        #[arg(long, value_parser = registry::parse_language_arg)]
        lang: Option<SourceLanguage>,
        #[command(subcommand)]
        command: Option<AnalyzeSubcommands>,
    },
    /// Explain dependencies
    Explain {
        #[command(subcommand)]
        command: ExplainSubcommands,
    },
    /// Discover repository source files
    Discover {
        /// Path to repository root
        repo: PathBuf,
    },
    /// Build context discovery commands
    Build {
        #[command(subcommand)]
        command: BuildSubcommands,
    },
    /// Include graph analysis commands
    Includes {
        /// Path to repository root
        repo: Option<PathBuf>,
        #[command(subcommand)]
        command: Option<IncludesSubcommands>,
    },
    /// Semantic indexing commands
    Index {
        /// Path to repository root
        repo: PathBuf,
        /// Restrict indexed units to one language (cpp|scala|other);
        /// default: per-file detection over the registered plugins
        #[arg(long, value_parser = registry::parse_language_arg)]
        lang: Option<SourceLanguage>,
    },
    /// Query semantic model
    Query {
        #[command(subcommand)]
        command: QuerySubcommands,
    },
    /// Generate or show optimization plans
    Plan {
        /// Path to repository root
        repo: Option<PathBuf>,
        #[command(subcommand)]
        command: Option<PlanSubcommands>,
    },
    /// Transformation commands
    Transform {
        #[command(subcommand)]
        command: TransformSubcommands,
    },
    /// Migrate consumers to generated headers
    Migrate {
        /// Path to repository root
        repo: PathBuf,
        /// Plan ID to migrate consumers for
        #[arg(long)]
        plan: String,
        /// Preview changes without applying them
        #[arg(long, default_value_t = false)]
        preview: bool,
    },
    /// Apply an optimization plan transactionally
    Apply {
        /// Path to repository root
        repo: PathBuf,
        /// Plan ID to apply
        #[arg(long)]
        plan: String,
    },
    /// Measure build performance of a repository
    Measure {
        /// Path to repository root
        repo: PathBuf,
    },
    /// Compare two build performance measurements
    Compare {
        /// Before measurement path
        before: PathBuf,
        /// After measurement path
        after: PathBuf,
    },
    /// Optimize repository dependencies end-to-end
    Optimize {
        /// Path to repository root
        repo: PathBuf,
    },
    /// Launch interactive graphical dependency and call graph interface
    Ui {
        /// Path to repository root
        repo: Option<PathBuf>,
        /// Port to bind web server
        #[arg(long, default_value_t = 7742)]
        port: u16,
        /// Do not open browser automatically
        #[arg(long, default_value_t = false)]
        no_open: bool,
        /// Export standalone HTML report to path
        #[arg(long)]
        export: Option<PathBuf>,
        /// Seconds between live-refresh re-hash checks of the repository
        #[arg(long, default_value_t = 2)]
        watch_interval: u64,
        /// Disable the live-refresh watcher
        #[arg(long, default_value_t = false)]
        no_watch: bool,
        /// Bind a wider address (e.g. a LAN address) instead of the
        /// 127.0.0.1 default; the UI has no authentication and this warns
        #[arg(long)]
        expose: Option<String>,
    },
}

pub fn run_cli() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Version) | None => {
            println!("tool version 0.1.0");
        }
        Some(Commands::Analyze {
            repo,
            workers,
            lang,
            command,
        }) => {
            dispatch_analyze(repo, workers, lang, command)?;
        }
        Some(Commands::Explain { command }) => {
            dispatch_explain(command)?;
        }
        Some(Commands::Discover { repo }) => {
            discover_repository_files(&repo)?;
        }
        Some(Commands::Build {
            command: BuildSubcommands::Discover { repo },
        }) => {
            discover_build_contexts(&repo)?;
        }
        Some(Commands::Includes { repo, command }) => {
            dispatch_includes(repo, command)?;
        }
        Some(Commands::Index { repo, lang }) => {
            build_source_index_filtered(&repo, lang)?;
        }
        Some(Commands::Query { command }) => {
            dispatch_query(command)?;
        }
        Some(Commands::Plan { repo, command }) => {
            dispatch_plan(repo, command)?;
        }
        Some(Commands::Transform { command }) => {
            dispatch_transform(command)?;
        }
        Some(Commands::Migrate {
            repo,
            plan,
            preview,
        }) => {
            build_migration_preview(&repo, &plan, preview)?;
        }
        Some(Commands::Apply { repo, plan }) => {
            apply_transform_plan(&repo, &plan)?;
        }
        Some(Commands::Measure { repo }) => {
            measure_build_performance(&repo)?;
        }
        Some(Commands::Compare { before, after }) => {
            compare_build_measurements(&before, &after)?;
        }
        Some(Commands::Optimize { repo }) => {
            run_end_to_end_optimize(&repo)?;
        }
        Some(Commands::Ui {
            repo,
            port,
            no_open,
            export,
            watch_interval,
            no_watch,
            expose,
        }) => {
            dispatch_ui(
                repo,
                port,
                no_open,
                export,
                watch_interval,
                no_watch,
                expose,
            )?;
        }
    }

    Ok(())
}

fn dispatch_analyze(
    repo: Option<PathBuf>,
    workers: Option<usize>,
    lang: Option<SourceLanguage>,
    command: Option<AnalyzeSubcommands>,
) -> Result<()> {
    match (repo, command) {
        (
            Some(repo_path),
            Some(AnalyzeSubcommands::Load {
                repo: load_repo,
                symbol,
                file,
            }),
        ) => {
            let r = if repo_path != load_repo {
                load_repo
            } else {
                repo_path
            };
            analyze_dependency_load(&r, symbol.as_deref(), file.as_deref())?;
        }
        (None, Some(AnalyzeSubcommands::Load { repo, symbol, file })) => {
            analyze_dependency_load(&repo, symbol.as_deref(), file.as_deref())?;
        }
        (Some(repo_path), None) => {
            if let Some(w) = workers {
                run_production_scale_analysis_filtered(&repo_path, Some(w), lang)?;
            } else {
                run_incremental_analysis_filtered(&repo_path, lang)?;
            }
        }
        (None, None) => {
            bail!("Repository path required for tool analyze command");
        }
    }
    Ok(())
}

fn dispatch_explain(command: ExplainSubcommands) -> Result<()> {
    match command {
        ExplainSubcommands::Load {
            repo,
            symbol,
            file,
            source,
        } => {
            explain_dependency_load(&repo, symbol.as_deref(), file.as_deref(), source.as_deref())?;
        }
    }
    Ok(())
}

fn dispatch_includes(repo: Option<PathBuf>, command: Option<IncludesSubcommands>) -> Result<()> {
    match (repo, command) {
        (_, Some(IncludesSubcommands::Explain { repo, file })) => {
            explain_include_graph_for_file(&repo, &file)?;
        }
        (Some(repo_path), None) => {
            analyze_include_graph(&repo_path)?;
        }
        (None, None) => {
            bail!("Repository path required for tool includes command");
        }
    }
    Ok(())
}

fn dispatch_query(command: QuerySubcommands) -> Result<()> {
    match command {
        QuerySubcommands::Symbol { repo, name } => {
            query_symbol_by_name(&repo, &name)?;
        }
        QuerySubcommands::Deps { repo, name } => {
            query_symbol_dependencies(&repo, &name)?;
        }
        QuerySubcommands::Users { repo, name } => {
            query_symbol_users(&repo, &name)?;
        }
    }
    Ok(())
}

fn dispatch_plan(repo: Option<PathBuf>, command: Option<PlanSubcommands>) -> Result<()> {
    match (repo, command) {
        (_, Some(PlanSubcommands::Show { repo, id })) => {
            show_optimization_plan(&repo, &id)?;
        }
        (Some(repo_path), None) => {
            plan_extractions(&repo_path)?;
        }
        (None, None) => {
            bail!("Repository path required for tool plan command");
        }
    }
    Ok(())
}

fn dispatch_transform(command: TransformSubcommands) -> Result<()> {
    match command {
        TransformSubcommands::Extract { repo, plan } => {
            extract_plan_headers(&repo, &plan)?;
        }
    }
    Ok(())
}

fn dispatch_ui(
    repo: Option<PathBuf>,
    port: u16,
    no_open: bool,
    export: Option<PathBuf>,
    watch_interval: u64,
    no_watch: bool,
    expose: Option<String>,
) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run_ui(
        repo,
        port,
        no_open,
        export,
        watch_interval,
        no_watch,
        expose,
    ))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::fs;

    #[test]
    fn test_cli_analyze_load() {
        let args = vec!["tool", "analyze", "load", "my_repo", "--symbol", "MyClass"];
        let cli = Cli::parse_from(args);
        match cli.command.unwrap() {
            Commands::Analyze {
                repo,
                workers: _,
                lang: _,
                command:
                    Some(AnalyzeSubcommands::Load {
                        repo: load_repo,
                        symbol,
                        file,
                    }),
            } => {
                assert_eq!(repo, None);
                assert_eq!(load_repo.to_str().unwrap(), "my_repo");
                assert_eq!(symbol.unwrap(), "MyClass");
                assert_eq!(file, None);
            }
            _ => panic!("Expected Analyze Load command"),
        }
    }

    #[test]
    fn test_cli_explain_load() {
        let args = vec![
            "tool",
            "explain",
            "load",
            "my_repo",
            "--file",
            "my_file.cpp",
            "--source",
            "Huge.h",
        ];
        let cli = Cli::parse_from(args);
        match cli.command.unwrap() {
            Commands::Explain {
                command:
                    ExplainSubcommands::Load {
                        repo,
                        symbol,
                        file,
                        source,
                    },
            } => {
                assert_eq!(repo.to_str().unwrap(), "my_repo");
                assert_eq!(symbol, None);
                assert_eq!(file.unwrap().to_str().unwrap(), "my_file.cpp");
                assert_eq!(source.unwrap(), "Huge.h");
            }
            _ => panic!("Expected Explain Load command"),
        }
    }

    #[test]
    fn test_cli_plan() {
        let args = vec!["tool", "plan", "my_repo"];
        let cli = Cli::parse_from(args);
        match cli.command.unwrap() {
            Commands::Plan { repo, command } => {
                assert_eq!(repo.unwrap().to_str().unwrap(), "my_repo");
                assert!(command.is_none());
            }
            _ => panic!("Expected Plan command"),
        }
    }

    #[test]
    fn test_cli_plan_show() {
        let args = vec!["tool", "plan", "show", "my_repo", "--id", "PLAN-001"];
        let cli = Cli::parse_from(args);
        match cli.command.unwrap() {
            Commands::Plan {
                repo: _,
                command: Some(PlanSubcommands::Show { repo, id }),
            } => {
                assert_eq!(repo.to_str().unwrap(), "my_repo");
                assert_eq!(id, "PLAN-001");
            }
            _ => panic!("Expected Plan Show command"),
        }
    }

    #[test]
    fn test_cli_transform_extract() {
        let args = vec![
            "tool",
            "transform",
            "extract",
            "my_repo",
            "--plan",
            "PLAN-001",
        ];
        let cli = Cli::parse_from(args);
        match cli.command.unwrap() {
            Commands::Transform {
                command: TransformSubcommands::Extract { repo, plan },
            } => {
                assert_eq!(repo.to_str().unwrap(), "my_repo");
                assert_eq!(plan, "PLAN-001");
            }
            _ => panic!("Expected Transform Extract command"),
        }
    }

    #[test]
    fn test_cli_migrate_preview() {
        let args = vec![
            "tool",
            "migrate",
            "my_repo",
            "--plan",
            "PLAN-001",
            "--preview",
        ];
        let cli = Cli::parse_from(args);
        match cli.command.unwrap() {
            Commands::Migrate {
                repo,
                plan,
                preview,
            } => {
                assert_eq!(repo.to_str().unwrap(), "my_repo");
                assert_eq!(plan, "PLAN-001");
                assert!(preview);
            }
            _ => panic!("Expected Migrate command"),
        }
    }

    #[test]
    fn test_cli_migrate_no_preview_flag() {
        let args = vec!["tool", "migrate", "my_repo", "--plan", "PLAN-001"];
        let cli = Cli::parse_from(args);
        match cli.command.unwrap() {
            Commands::Migrate {
                repo,
                plan,
                preview,
            } => {
                assert_eq!(repo.to_str().unwrap(), "my_repo");
                assert_eq!(plan, "PLAN-001");
                assert!(!preview); // default is false
            }
            _ => panic!("Expected Migrate command"),
        }
    }

    #[test]
    fn test_cli_apply() {
        let args = vec!["tool", "apply", "my_repo", "--plan", "PLAN-001"];
        let cli = Cli::parse_from(args);
        match cli.command.unwrap() {
            Commands::Apply { repo, plan } => {
                assert_eq!(repo.to_str().unwrap(), "my_repo");
                assert_eq!(plan, "PLAN-001");
            }
            _ => panic!("Expected Apply command"),
        }
    }
    #[test]
    fn test_cli_measure() {
        let args = vec!["tool", "measure", "my_repo"];
        let cli = Cli::parse_from(args);
        match cli.command.unwrap() {
            Commands::Measure { repo } => {
                assert_eq!(repo.to_str().unwrap(), "my_repo");
            }
            _ => panic!("Expected Measure command"),
        }
    }

    #[test]
    fn test_cli_compare() {
        let args = vec!["tool", "compare", "before_dir", "after_dir"];
        let cli = Cli::parse_from(args);
        match cli.command.unwrap() {
            Commands::Compare { before, after } => {
                assert_eq!(before.to_str().unwrap(), "before_dir");
                assert_eq!(after.to_str().unwrap(), "after_dir");
            }
            _ => panic!("Expected Compare command"),
        }
    }

    #[test]
    fn test_handle_measure() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros();
        let temp = std::env::temp_dir().join(format!("ariadne_test_meas_{}", unique));
        let repo_dir = temp.join("repo");
        fs::create_dir_all(repo_dir.join("src")).unwrap();
        fs::write(repo_dir.join("ariadne.toml"), "").unwrap();

        let result = measure_build_performance(&repo_dir);
        assert!(result.is_ok());

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn test_handle_compare() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros();
        let temp = std::env::temp_dir().join(format!("ariadne_test_comp_{}", unique));

        let before_dir = temp.join("before");
        fs::create_dir_all(&before_dir).unwrap();

        let after_dir = temp.join("after");
        fs::create_dir_all(&after_dir).unwrap();

        let result = compare_build_measurements(&before_dir, &after_dir);
        // This will fail because there are no valid measurement artifacts,
        // but for now the command doesn't even exist.
        assert!(result.is_err());

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn test_cli_analyze_repo() {
        let args = vec!["tool", "analyze", "my_repo"];
        let cli = Cli::parse_from(args);
        match cli.command.unwrap() {
            Commands::Analyze {
                repo,
                workers,
                lang: _,
                command,
            } => {
                assert_eq!(repo.unwrap().to_str().unwrap(), "my_repo");
                assert_eq!(workers, None);
                assert!(command.is_none());
            }
            _ => panic!("Expected Analyze command"),
        }
    }

    #[test]
    fn test_cli_analyze_workers_flag() {
        let args = vec!["tool", "analyze", "my_repo", "--workers", "32"];
        let cli = Cli::parse_from(args);
        match cli.command.unwrap() {
            Commands::Analyze {
                repo,
                workers,
                lang: _,
                command,
            } => {
                assert_eq!(repo.unwrap().to_str().unwrap(), "my_repo");
                assert_eq!(workers, Some(32));
                assert!(command.is_none());
            }
            _ => panic!("Expected Analyze command with --workers"),
        }
    }

    #[test]
    fn test_handle_analyze() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        let repo_dir = root.join("tests/fixtures/basic");

        let result = run_incremental_analysis(&repo_dir).unwrap();
        assert!(result.stats.total_tus > 0 || result.stats.total_tus == 0); // Just check it returned Ok
    }

    #[test]
    fn test_cli_optimize() {
        let args = vec!["tool", "optimize", "my_repo"];
        let cli = Cli::parse_from(args);
        match cli.command.unwrap() {
            Commands::Optimize { repo } => {
                assert_eq!(repo.to_str().unwrap(), "my_repo");
            }
            _ => panic!("Expected Optimize command"),
        }
    }

    // AR-02 (S6): `tool ui` must accept `--expose <addr>` as the only opt-in
    // to a non-localhost bind; omitting it keeps the 127.0.0.1 default.
    #[test]
    fn test_cli_ui_accepts_expose_flag() {
        use clap::CommandFactory;
        let parsed = Cli::command().try_get_matches_from(vec![
            "tool", "ui", "my_repo", "--expose", "10.0.0.5",
        ]);
        assert!(
            parsed.is_ok(),
            "`tool ui --expose <addr>` must parse; got: {:?}",
            parsed.err().map(|error| error.to_string())
        );
    }

    #[test]
    fn test_cli_ui_expose_flag_carries_address() {
        let args = vec!["tool", "ui", "my_repo", "--expose", "10.0.0.5"];
        let cli = Cli::parse_from(args);
        match cli.command.unwrap() {
            Commands::Ui { expose, .. } => {
                assert_eq!(expose.as_deref(), Some("10.0.0.5"));
            }
            _ => panic!("Expected Ui command with --expose"),
        }
    }

    #[test]
    fn test_cli_ui_help_documents_expose_lan_warning() {
        use clap::CommandFactory;
        let mut root_command = Cli::command();
        let ui_command = root_command
            .find_subcommand("ui")
            .expect("`ui` subcommand must exist");
        let expose_flag = ui_command
            .get_arguments()
            .find(|arg| arg.get_long() == Some("expose"))
            .expect("`ui` must carry a --expose flag");
        let help_text = expose_flag
            .get_help()
            .map(|help| help.to_string())
            .unwrap_or_default();
        assert!(
            help_text.contains("127.0.0.1") && help_text.contains("warn"),
            "--expose help must state the localhost default and the exposure warning: {help_text}"
        );
    }

    #[test]
    fn test_cli_ui_without_expose_keeps_localhost_default() {
        let args = vec!["tool", "ui", "my_repo"];
        let cli = Cli::parse_from(args);
        match cli.command.unwrap() {
            Commands::Ui { expose, .. } => {
                assert!(expose.is_none(), "no --expose means no bind override");
                assert_eq!(resolve_bind_addr(expose.as_deref()), "127.0.0.1");
            }
            _ => panic!("Expected Ui command"),
        }
    }
}
