//! CLI command tests for the crate root (UTA-T7b test-module extraction:
//! this was `mod tests` inline in lib.rs until the row-6 route seams made
//! the ceiling guard require a split; content moved verbatim except the
//! two UTA-T7b row-13 strengthenings and the row-6 route pins below).

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

    // UTA-T7b row 13: was `assert!(result.is_ok())` only. The value the
    // command returns is the measurement itself, so the assertions go to
    // its fields: an empty src/ discovers zero TUs (zero per-TU rows and
    // the affected-TU tally Some(0), the count of the discovery, not a
    // missing datum), and with no load artifacts ever written the
    // collateral scalar is honestly None (UTA-B7c keeps discovery errors
    // loud - they simply cannot happen for a fresh temp repo either).
    let measurement = measure_build_performance(&repo_dir)
        .expect("measure_build_performance on the temp repo must succeed");
    assert!(
        measurement.tu_measurements.is_empty(),
        "an empty src/ must measure zero TUs, got {} rows",
        measurement.tu_measurements.len()
    );
    assert_eq!(
        measurement.affected_tus_count,
        Some(0),
        "the affected count must carry the discovery's zero, not None"
    );
    assert_eq!(
        measurement.collateral_load, None,
        "a repo with no load artifacts must report collateral_load None, not a fabricated zero"
    );
    assert!(
        measurement.repo_root.ends_with("repo"),
        "Repository::locate canonicalizes the path it was given, got {:?}",
        measurement.repo_root
    );

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

    // UTA-T7b row 13: was `assert!(total_tus > 0 || total_tus == 0)` - a
    // tautology that accepted any value. tests/fixtures/basic ships two
    // translation units (src/thing.cpp, tests/consumer.cpp); the stats
    // must say exactly that, warm or cold (reused + reanalyzed partitions
    // the same total).
    let result = run_incremental_analysis(&repo_dir)
        .expect("the basic fixture must analyze without error");
    assert_eq!(
        result.stats.total_tus, 2,
        "tests/fixtures/basic ships two .cpp translation units, got {:?}",
        result.stats
    );
    assert_eq!(
        result.stats.reused_tus + result.stats.reanalyzed_tus, result.stats.total_tus,
        "reused + reanalyzed must partition the total exactly"
    );
    assert_eq!(
        result.index_result.tu_analyses.len(),
        result.stats.total_tus,
        "the index must carry exactly one analysis per discovered TU"
    );
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

// ---------------------------------------------------------------------------
// UTA-T7b row 6: dispatch route tables (seams extracted from
// dispatch_analyze/_includes/_plan; bail strings live in the dispatchers).
// ---------------------------------------------------------------------------

fn load_subcommand(repo: &str) -> AnalyzeSubcommands {
    AnalyzeSubcommands::Load {
        repo: PathBuf::from(repo),
        symbol: Some("Sym".to_string()),
        file: Some(PathBuf::from("f.cpp")),
    }
}

fn explain_subcommand(repo: &str) -> IncludesSubcommands {
    IncludesSubcommands::Explain {
        repo: PathBuf::from(repo),
        file: PathBuf::from("a.h"),
    }
}

fn show_subcommand(repo: &str) -> PlanSubcommands {
    PlanSubcommands::Show {
        repo: PathBuf::from(repo),
        id: "PLAN-001".to_string(),
    }
}

#[test]
fn pin_dispatch_analyze_route_table() {
    assert_eq!(
        route_analyze(None, None, None),
        AnalyzeRoute::MissingRepo,
        "(None, None) is the missing-repo bail lane"
    );
    assert_eq!(
        route_analyze(Some(PathBuf::from("outer")), Some(4), None),
        AnalyzeRoute::Plain {
            repo: PathBuf::from("outer"),
            workers: Some(4),
        },
        "positional without a subcommand runs the plain analysis carrying --workers"
    );
    assert_eq!(
        route_analyze(None, Some(4), Some(load_subcommand("sub"))),
        AnalyzeRoute::Load {
            repo: PathBuf::from("sub"),
            symbol: Some("Sym".to_string()),
            file: Some(PathBuf::from("f.cpp")),
        },
        "the Load subcommand carries its own repo; --workers is not consulted on this lane"
    );
    assert_eq!(
        route_analyze(
            Some(PathBuf::from("same")),
            None,
            Some(load_subcommand("same"))
        ),
        AnalyzeRoute::Load {
            repo: PathBuf::from("same"),
            symbol: Some("Sym".to_string()),
            file: Some(PathBuf::from("f.cpp")),
        },
        "equal repos take either side of the original `if repo_path != load_repo` bytewise"
    );
}

#[test]
fn pin_dispatch_includes_route_table() {
    assert_eq!(
        route_includes(None, None),
        IncludesRoute::MissingRepo,
        "(None, None) is the missing-repo bail lane"
    );
    assert_eq!(
        route_includes(Some(PathBuf::from("outer")), None),
        IncludesRoute::Graph {
            repo: PathBuf::from("outer"),
        },
        "positional without a subcommand runs the aggregate graph"
    );
    assert_eq!(
        route_includes(None, Some(explain_subcommand("sub"))),
        IncludesRoute::Explain {
            repo: PathBuf::from("sub"),
            file: PathBuf::from("a.h"),
        },
        "(None, Some(Explain)) uses the subcommand's repo"
    );
    assert_eq!(
        route_includes(Some(PathBuf::from("outer")), Some(explain_subcommand("sub"))),
        IncludesRoute::Explain {
            repo: PathBuf::from("sub"),
            file: PathBuf::from("a.h"),
        },
        "(_, Some(Explain)) is a WILDCARD arm: the subcommand's repo wins, the positional is silently dropped"
    );
}

#[test]
fn pin_dispatch_plan_route_table() {
    assert_eq!(
        route_plan(None, None),
        PlanRoute::MissingRepo,
        "(None, None) is the missing-repo bail lane"
    );
    assert_eq!(
        route_plan(Some(PathBuf::from("outer")), None),
        PlanRoute::Plans {
            repo: PathBuf::from("outer"),
        },
        "positional without a subcommand plans extractions"
    );
    assert_eq!(
        route_plan(None, Some(show_subcommand("sub"))),
        PlanRoute::Show {
            repo: PathBuf::from("sub"),
            id: "PLAN-001".to_string(),
        },
        "(None, Some(Show)) uses the subcommand's repo"
    );
    assert_eq!(
        route_plan(Some(PathBuf::from("outer")), Some(show_subcommand("sub"))),
        PlanRoute::Show {
            repo: PathBuf::from("sub"),
            id: "PLAN-001".to_string(),
        },
        "(_, Some(Show)) is a WILDCARD arm: the subcommand's repo wins, the positional is silently dropped"
    );
}

/// UTA-T7b row 6 (REPORTED): the precedence is asymmetric in SHAPE -
/// dispatch_analyze compares the two paths (`if repo_path != load_repo`)
/// while _includes and _plan use a wildcard arm that drops the positional
/// without ever comparing. The RESULT is the same for different paths:
/// the subcommand repo silently wins in all three dispatchers, and no
/// warning or error is ever printed for the disagreement.
#[test]
fn pin_subrepo_positional_precedence_silent() {
    let routed_analyze = route_analyze(
        Some(PathBuf::from("outer")),
        None,
        Some(load_subcommand("inner")),
    );
    assert_eq!(
        routed_analyze,
        AnalyzeRoute::Load {
            repo: PathBuf::from("inner"),
            symbol: Some("Sym".to_string()),
            file: Some(PathBuf::from("f.cpp")),
        },
        "analyze Load: a different positional repo is silently dropped"
    );
    assert_eq!(
        route_includes(Some(PathBuf::from("outer")), Some(explain_subcommand("inner"))),
        IncludesRoute::Explain {
            repo: PathBuf::from("inner"),
            file: PathBuf::from("a.h"),
        },
        "includes Explain: the positional is dropped by the wildcard arm, never compared"
    );
    assert_eq!(
        route_plan(Some(PathBuf::from("outer")), Some(show_subcommand("inner"))),
        PlanRoute::Show {
            repo: PathBuf::from("inner"),
            id: "PLAN-001".to_string(),
        },
        "plan Show: the positional is dropped by the wildcard arm, never compared"
    );
}

/// UTA-T7b row 6: the observable bail side of the dispatchers stays the
/// execution contract (the routes above only decide; these strings are
/// what the user sees, byte-for-byte, and nothing runs before them).
#[test]
fn pin_missing_repo_dispatch_bails_are_verbatim_and_run_nothing() {
    let analyze_error = dispatch_analyze(None, None, None, None)
        .expect_err("(None, None) must bail")
        .to_string();
    assert_eq!(
        analyze_error, "Repository path required for tool analyze command",
        "the analyze bail text is the frozen user contract"
    );
    let includes_error = dispatch_includes(None, None)
        .expect_err("(None, None) must bail")
        .to_string();
    assert_eq!(
        includes_error, "Repository path required for tool includes command",
        "the includes bail text is the frozen user contract"
    );
    let plan_error = dispatch_plan(None, None)
        .expect_err("(None, None) must bail")
        .to_string();
    assert_eq!(
        plan_error, "Repository path required for tool plan command",
        "the plan bail text is the frozen user contract"
    );
}
