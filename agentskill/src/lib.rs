use clap::{Args, Parser, Subcommand};

use agentskill_core::output::write_value;

#[derive(Parser)]
#[command(
    name = "agentskill",
    version,
    about = "Analyze repositories and synthesize AGENTS.md"
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
    #[command(about = "Generate AGENTS.md markdown from repository analysis")]
    Generate(GenerateArgs),
    #[command(about = "Update or create AGENTS.md")]
    Update(UpdateArgs),
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
    #[arg(long = "reference", action = clap::ArgAction::Append)]
    references: Vec<String>,
}

#[derive(Args)]
struct GenerateArgs {
    repo: String,
    #[arg(long = "reference", action = clap::ArgAction::Append)]
    references: Vec<String>,
    #[arg(long)]
    interactive: bool,
    #[arg(long, default_value = "concise")]
    profile: String,
    #[arg(long, default_value = "single")]
    layout: String,
}

#[derive(Args)]
struct UpdateArgs {
    repo: String,
    #[arg(long = "section", action = clap::ArgAction::Append)]
    sections: Vec<String>,
    #[arg(long = "exclude-section", action = clap::ArgAction::Append)]
    excluded_sections: Vec<String>,
    #[arg(long)]
    force: bool,
    #[arg(long, default_value = "concise")]
    profile: String,
    #[arg(long, default_value = "single")]
    layout: String,
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
            agentskill_core::reference::load_reference_documents(&args.references)?;
            write_value(
                &agentskill_analyzers::run_many(&args.repos, args.lang.as_deref()),
                pretty,
                out,
            )
            .map(|()| false)
        }
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
        Commands::Generate(args) => {
            if pretty {
                return Err(agentskill_core::AgentskillError::InvalidArgument(
                    "generate does not support --pretty".into(),
                ));
            }

            let answers = if args.interactive {
                agentskill_generation::collect_interactive_answers(&args.repo, &args.references)?
            } else {
                std::collections::BTreeMap::new()
            };
            agentskill_generation::generate_with_answers(
                &args.repo,
                out,
                &args.references,
                args.interactive,
                &args.profile,
                &args.layout,
                &answers,
            )
            .map(|()| false)
        }
        Commands::Update(args) => {
            if pretty {
                return Err(agentskill_core::AgentskillError::InvalidArgument(
                    "update does not support --pretty".into(),
                ));
            }
            agentskill_generation::update(
                &args.repo,
                out,
                &args.sections,
                &args.excluded_sections,
                args.force,
                &args.profile,
                &args.layout,
            )
            .map(|()| false)
        }
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

#[cfg(test)]
mod unit_tests {
    use std::fs;

    use super::{Cli, dispatch};
    use clap::Parser;
    use tempfile::tempdir;

    fn example() -> String {
        format!(
            "{}/../agentskill-skill/examples/rust",
            env!("CARGO_MANIFEST_DIR")
        )
    }

    #[test]
    fn dispatches_all_analyzers_and_document_commands() {
        let directory = tempdir().unwrap();

        let repo = directory.path().to_string_lossy().into_owned();
        fs::write(directory.path().join("main.rs"), "fn main() {}\n").unwrap();

        let analyzers = [
            "scan", "measure", "config", "git", "graph", "symbols", "tests",
        ];
        let output_directory = format!("target/agentskill-test-output-{}", std::process::id());
        fs::create_dir_all(&output_directory).unwrap();

        for (index, analyzer) in analyzers.into_iter().enumerate() {
            let output = format!("{output_directory}/{index}.json");
            let cli =
                Cli::try_parse_from(["agentskill", "--pretty", "--out", &output, analyzer, &repo])
                    .unwrap();
            dispatch(cli).unwrap();

            assert!(std::path::Path::new(&output).exists());
        }

        let analyze_output = format!("{output_directory}/analyze.json");
        dispatch(
            Cli::try_parse_from([
                "agentskill",
                "--out",
                &analyze_output,
                "analyze",
                &example(),
            ])
            .unwrap(),
        )
        .unwrap();

        let generated = directory.path().join("AGENTS.md");
        let generated = generated.to_string_lossy().into_owned();
        dispatch(
            Cli::try_parse_from(["agentskill", "--out", &generated, "generate", &repo]).unwrap(),
        )
        .unwrap();
        dispatch(Cli::try_parse_from(["agentskill", "update", &repo, "--force"]).unwrap()).unwrap();
        fs::remove_dir_all(output_directory).unwrap();
    }

    #[test]
    fn rejects_pretty_document_commands() {
        let directory = tempdir().unwrap();
        fs::write(directory.path().join("main.rs"), "fn main() {}\n").unwrap();

        let repo = directory.path().to_string_lossy().into_owned();
        let generate = Cli::try_parse_from(["agentskill", "--pretty", "generate", &repo]);

        assert!(dispatch(generate.unwrap()).is_err());
        let update = Cli::try_parse_from(["agentskill", "--pretty", "update", &repo]);

        assert!(dispatch(update.unwrap()).is_err());
    }

    #[test]
    fn reports_analyzer_errors_as_failed_commands() {
        let cli = Cli::try_parse_from(["agentskill", "scan", "/missing/repository"]).unwrap();

        assert!(dispatch(cli).unwrap());
    }
}
