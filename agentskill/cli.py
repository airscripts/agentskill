"""CLI entry point for agentskill."""

import argparse
import json
import sys
from pathlib import Path

from .constants import SAMPLE_SIZE_SMALL, SAMPLE_SIZE_MEDIUM, JSON_INDENT
from .extractors.git import analyze_git_commits, analyze_branches
from .extractors.filesystem import scan_source_files, detect_tooling, is_git_repo, get_project_metadata
from .analyzers.language.rust import RustAnalyzer
from .analyzers.language.python import PythonAnalyzer
from .analyzers.language.generic import GenericAnalyzer
from .synthesis import AgentSynthesizer, SynthesisConfig


def get_analyzer(language: str):
    """Get appropriate analyzer for language."""
    analyzers = {
        "rust": RustAnalyzer,
        "python": PythonAnalyzer,
    }
    if language in analyzers:
        return analyzers[language]()
    return GenericAnalyzer(language)


def analyze_repository(repo_path: str) -> dict:
    """Analyze a single repository."""
    abs_path = str(Path(repo_path).resolve())

    if not is_git_repo(abs_path):
        print(f"Warning: {repo_path} may not be a git repository", file=sys.stderr)

    # Scan for source files
    files_by_lang = scan_source_files(abs_path)

    # Analyze each language
    languages = {}
    for lang, files in files_by_lang.items():
        analyzer = get_analyzer(lang)
        result = analyzer.analyze_files(files)
        languages[lang] = result.to_dict()

    # Git analysis
    git_data = {
        "commits": analyze_git_commits(abs_path),
        "branches": analyze_branches(abs_path),
    }

    # Tooling detection
    tooling = detect_tooling(abs_path)

    # Project metadata
    metadata = get_project_metadata(abs_path)

    return {
        "path": abs_path,
        "languages": languages,
        "git": git_data,
        "tooling": tooling,
        "metadata": metadata,
    }


def generate_agents_md(analyses: list, repos: list, config: SynthesisConfig = None) -> str:
    """Generate AGENTS.md from analyses."""
    synthesizer = AgentSynthesizer(config)
    return synthesizer.synthesize(analyses, repos)


def main():
    parser = argparse.ArgumentParser(
        description="Generate AGENTS.md from your actual coding style",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  agentskill ~/projects/myapp
  agentskill ~/projects/repo1 ~/projects/repo2 -o AGENTS.md
  agentskill ~/projects --json -o report.json
        """
    )
    parser.add_argument(
        "repos",
        nargs="+",
        help="Paths to repositories to analyze"
    )
    parser.add_argument(
        "-o", "--output",
        help="Output file (default: stdout)"
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output raw JSON analysis instead of AGENTS.md"
    )
    parser.add_argument(
        "--skip-git",
        action="store_true",
        help="Skip git analysis"
    )
    parser.add_argument(
        "--skip-tooling",
        action="store_true",
        help="Skip tooling detection"
    )

    args = parser.parse_args()

    # Validate repos
    valid_repos = []
    for repo in args.repos:
        if Path(repo).is_dir():
            valid_repos.append(repo)
        else:
            print(f"Error: Not a directory: {repo}", file=sys.stderr)

    if not valid_repos:
        print("Error: No valid repositories to analyze", file=sys.stderr)
        sys.exit(1)

    # Analyze each repo
    analyses = []
    for repo in valid_repos:
        print(f"Analyzing {repo}...", file=sys.stderr)
        analysis = analyze_repository(repo)
        analyses.append(analysis)

    # Build report
    if args.json:
        output = json.dumps({
            "repos": valid_repos,
            "analyses": analyses,
        }, indent=JSON_INDENT)
    else:
        # Generate AGENTS.md
        config = SynthesisConfig(
            include_git=not args.skip_git,
            include_tooling=not args.skip_tooling,
        )
        output = generate_agents_md(analyses, valid_repos, config)

    # Output
    if args.output:
        Path(args.output).write_text(output)
        print(f"Output written to {args.output}", file=sys.stderr)
    else:
        print(output)


if __name__ == "__main__":
    main()
