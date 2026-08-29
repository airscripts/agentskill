use clap::{Args, Parser, Subcommand};

use agentskill_core::output::{ANALYZER_NAMES, write_value};
use serde_json::Value;

#[derive(Parser)]
#[command(
    name = "agentskill",
    version,
    about = "Collect repository evidence for LLM-authored AGENTS.md files"
)]
pub struct Cli {
    #[arg(long, global = true, help = "Pretty-print JSON output")]
    pretty: bool,
    #[arg(
        long,
        global = true,
        value_name = "FILE",
        help = "Write output to a file instead of stdout"
    )]
    out: Option<String>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Run all analyzers and merge output")]
    Analyze(AnalyzeArgs),
    #[command(about = "Build normalized evidence for the LLM skill")]
    Evidence(RepoLangArgs),
    #[command(about = "Directory tree and file inventory")]
    Scan(RepoLangArgs),
    #[command(about = "Exact formatting metrics")]
    Measure(RepoLangArgs),
    #[command(about = "Formatter, linter, and type-checker detection")]
    Config(RepoArgs),
    #[command(about = "Commit log and branch analysis")]
    Git(RepoArgs),
    #[command(about = "Internal import graph")]
    Graph(RepoLangArgs),
    #[command(about = "Symbol name extraction and pattern clustering")]
    Symbols(RepoLangArgs),
    #[command(about = "Test-to-source mapping and framework detection")]
    Tests(RepoArgs),
    #[command(about = "Validate LLM-authored AGENTS.md files without writing")]
    Validate(RepoArgs),
    #[command(about = "Report stale AGENTS.md references without writing")]
    Drift(RepoArgs),
}

#[derive(Args)]
struct RepoArgs {
    repo: String,
}

#[derive(Args)]
struct RepoLangArgs {
    repo: String,
    #[arg(long)]
    lang: Option<String>,
}

#[derive(Args)]
struct AnalyzeArgs {
    #[arg(required = true)]
    repos: Vec<String>,
    #[arg(long)]
    lang: Option<String>,
}

pub fn run() -> i32 {
    let cli = Cli::parse();

    match dispatch(cli) {
        Ok(failed) => i32::from(failed),
        Err(error) => {
            eprintln!("{error}");
            1
        }
    }
}

fn dispatch(cli: Cli) -> agentskill_core::Result<bool> {
    let pretty = cli.pretty;
    let out = cli.out.as_deref();

    match cli.command {
        Commands::Analyze(args) => {
            let value = agentskill_analyzers::run_many(&args.repos, args.lang.as_deref());
            let failed = aggregate_failed(&value, args.repos.len() > 1);
            write_value(&value, pretty, out).map(|()| failed)
        }
        Commands::Evidence(args) => write_result(
            agentskill_analyzers::run_evidence(&args.repo, args.lang.as_deref()),
            pretty,
            out,
        ),
        Commands::Scan(args) => {
            write_analyzer("scan", &args.repo, args.lang.as_deref(), pretty, out)
        }
        Commands::Measure(args) => {
            write_analyzer("measure", &args.repo, args.lang.as_deref(), pretty, out)
        }
        Commands::Config(args) => write_analyzer("config", &args.repo, None, pretty, out),
        Commands::Git(args) => write_analyzer("git", &args.repo, None, pretty, out),
        Commands::Graph(args) => {
            write_analyzer("graph", &args.repo, args.lang.as_deref(), pretty, out)
        }
        Commands::Symbols(args) => {
            write_analyzer("symbols", &args.repo, args.lang.as_deref(), pretty, out)
        }
        Commands::Tests(args) => write_analyzer("tests", &args.repo, None, pretty, out),
        Commands::Validate(args) => write_validation(
            agentskill_validation::validate(&args.repo),
            pretty,
            out,
            |value| value["valid"] != true,
        ),
        Commands::Drift(args) => write_validation(
            agentskill_validation::drift(&args.repo),
            pretty,
            out,
            |value| value["stale"] == true,
        ),
    }
}

fn write_analyzer(
    name: &str,
    repo: &str,
    lang: Option<&str>,
    pretty: bool,
    out: Option<&str>,
) -> agentskill_core::Result<bool> {
    let value = agentskill_analyzers::run_one(name, repo, lang);
    let failed = value.get("error").is_some();
    write_value(&value, pretty, out)?;

    Ok(failed)
}

fn aggregate_failed(value: &Value, multiple_repositories: bool) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };

    if multiple_repositories {
        return object
            .values()
            .any(|repository| aggregate_failed(repository, false));
    }

    ANALYZER_NAMES.iter().any(|name| {
        object
            .get(*name)
            .and_then(Value::as_object)
            .is_some_and(|analyzer| analyzer.contains_key("error"))
    })
}

fn write_result(
    result: agentskill_core::Result<serde_json::Value>,
    pretty: bool,
    out: Option<&str>,
) -> agentskill_core::Result<bool> {
    let value = result?;
    write_value(&value, pretty, out)?;
    Ok(false)
}

fn write_validation(
    result: agentskill_core::Result<serde_json::Value>,
    pretty: bool,
    out: Option<&str>,
    failed: impl FnOnce(&serde_json::Value) -> bool,
) -> agentskill_core::Result<bool> {
    let value = result?;
    let failed = failed(&value);
    write_value(&value, pretty, out)?;
    Ok(failed)
}

#[cfg(test)]
mod unit_tests {
    use std::fs;

    use super::{Cli, dispatch};
    use clap::Parser;
    use tempfile::tempdir;

    #[test]
    fn dispatches_analyzers_evidence_and_validation() {
        let directory = tempdir().unwrap();
        fs::create_dir_all(directory.path().join("src")).unwrap();
        fs::write(directory.path().join("src/main.rs"), "fn main() {}\n").unwrap();
        fs::write(directory.path().join("AGENTS.md"), "# AGENTS.md\n").unwrap();

        let repo = directory.path().to_string_lossy().into_owned();
        let output_directory = format!("target/agentskill-test-output-{}", std::process::id());
        fs::create_dir_all(&output_directory).unwrap();

        let analyzers = [
            "scan", "measure", "config", "git", "graph", "symbols", "tests",
        ];
        for (index, analyzer) in analyzers.into_iter().enumerate() {
            let output = format!("{output_directory}/{index}.json");
            let cli =
                Cli::try_parse_from(["agentskill", "--pretty", "--out", &output, analyzer, &repo])
                    .unwrap();
            dispatch(cli).unwrap();
            assert!(std::path::Path::new(&output).exists());
        }

        for command in ["analyze", "evidence", "validate", "drift"] {
            let output = format!("{output_directory}/{command}.json");
            let cli =
                Cli::try_parse_from(["agentskill", "--out", &output, command, &repo]).unwrap();
            dispatch(cli).unwrap();
            assert!(std::path::Path::new(&output).exists());
        }

        assert!(Cli::try_parse_from(["agentskill", "generate", &repo]).is_err());
        assert!(Cli::try_parse_from(["agentskill", "update", &repo]).is_err());
        fs::remove_dir_all(output_directory).unwrap();
    }

    #[test]
    fn reports_analyzer_errors_as_failed_commands() {
        let cli = Cli::try_parse_from(["agentskill", "scan", "/missing/repository"]).unwrap();
        assert!(dispatch(cli).unwrap());
    }

    #[test]
    fn reports_aggregate_analyzer_errors_as_failed_commands() {
        let cli = Cli::try_parse_from(["agentskill", "analyze", "/missing/repository"]).unwrap();
        assert!(dispatch(cli).unwrap());

        let first = Cli::try_parse_from([
            "agentskill",
            "analyze",
            "/missing/repository",
            "/another/missing/repository",
        ])
        .unwrap();
        assert!(dispatch(first).unwrap());
    }
}
