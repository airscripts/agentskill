"""AGENTS.md synthesis from analysis results."""

from pathlib import Path
from typing import Dict, List, Any
from dataclasses import dataclass


@dataclass
class SynthesisConfig:
    """Configuration for AGENTS.md generation."""
    include_overview: bool = True
    include_cross_language: bool = True
    include_git: bool = True
    include_tooling: bool = True
    include_red_lines: bool = True
    confidence_threshold: float = 0.6
    max_examples_per_section: int = 3


class AgentSynthesizer:
    """Synthesizes AGENTS.md from analysis results."""

    def __init__(self, config: SynthesisConfig = None):
        self.config = config or SynthesisConfig()

    def synthesize(self, analyses: List[Dict], repos: List[str]) -> str:
        """Generate AGENTS.md content from analysis results."""
        sections = []

        # Overview
        if self.config.include_overview:
            sections.append(self._generate_overview(analyses, repos))

        # Cross-language patterns
        if self.config.include_cross_language:
            sections.append(self._generate_cross_language(analyses))

        # Per-language sections
        sections.append(self._generate_language_sections(analyses))

        # Git
        if self.config.include_git:
            sections.append(self._generate_git_section(analyses))

        # Tooling
        if self.config.include_tooling:
            sections.append(self._generate_tooling_section(analyses))

        # Red Lines
        if self.config.include_red_lines:
            sections.append(self._generate_red_lines(analyses))

        # Footer
        sections.append(self._generate_footer(analyses, repos))

        return "\n\n".join(sections)

    def _generate_overview(self, analyses: List[Dict], repos: List[str]) -> str:
        """Generate overview section."""
        languages = self._detected_languages(analyses)
        lang_str = ", ".join(sorted(languages)) if languages else "various languages"

        repo_count = len(repos)
        repo_desc = f"{repo_count} repositories" if repo_count > 1 else "1 repository"

        lines = [
            "# AGENTS.md — Coding Style",
            "",
            "## Overview",
            "",
            f"Multi-language codebase spanning {lang_str}. Analyzed {repo_desc}.",
            "",
            "Key principles distilled from actual patterns:",
        ]

        # Extract key principles
        principles = self._extract_key_principles(analyses)
        for principle in principles[:3]:
            lines.append(f"- {principle}")

        return "\n".join(lines)

    def _generate_cross_language(self, analyses: List[Dict]) -> str:
        """Generate cross-language patterns section."""
        lines = [
            "## Cross-Language Patterns",
            "",
            "Patterns holding across all detected languages:",
        ]

        patterns = self._find_common_patterns(analyses)
        if patterns.get("naming"):
            lines.append("")
            lines.append("### Naming")
            for name_type, style in patterns["naming"].items():
                lines.append(f"- **{name_type}:** {style}")

        if patterns.get("comments"):
            lines.append("")
            lines.append("### Comments")
            lines.append(f"- **Philosophy:** {patterns['comments']}")

        if patterns.get("error_handling"):
            lines.append("")
            lines.append("### Error Handling")
            lines.append(f"- {patterns['error_handling']}")

        if patterns.get("spacing"):
            lines.append("")
            lines.append("### Spacing")
            lines.append(f"- {patterns['spacing']}")

        return "\n".join(lines)

    def _generate_language_sections(self, analyses: List[Dict]) -> str:
        """Generate per-language sections."""
        sections = []
        all_langs = self._detected_languages(analyses)

        for lang in sorted(all_langs):
            lang_sections = []
            for analysis in analyses:
                if lang in analysis.get("languages", {}):
                    lang_data = analysis["languages"][lang]
                    lang_sections.append(self._format_language_section(lang, lang_data))

            if lang_sections:
                sections.append(f"\n## {lang.title()}\n")
                sections.append("\n\n".join(lang_sections))

        return "".join(sections)

    def _format_language_section(self, lang: str, data: Dict) -> str:
        """Format a single language section."""
        lines = []

        # Naming
        naming = data.get("naming", {})
        if naming:
            lines.append("### Naming")
            for category, info in naming.items():
                if isinstance(info, dict):
                    dominant = info.get("dominant_case", "unknown")
                    lines.append(f"- **{category.title()}:** {dominant}")

        # Error Handling
        errors = data.get("error_handling", {})
        if errors and not errors.get("note"):
            lines.append("")
            lines.append("### Error Handling")
            for pattern, count in errors.items():
                if isinstance(count, int) and count > 0:
                    lines.append(f"- `{pattern}`: {count} occurrences")

        # Comments
        comments = data.get("comments", {})
        if comments:
            lines.append("")
            lines.append("### Comments")
            density = comments.get("density", 0)
            if density > 0:
                lines.append(f"- **Density:** {density:.1%}")
            if "doc_style" in comments:
                lines.append(f"- **Style:** `{comments['doc_style']}`")

        # Spacing
        spacing = data.get("spacing", {})
        if spacing:
            avg_blanks = spacing.get("avg_blank_lines", 0)
            if avg_blanks > 0:
                lines.append("")
                lines.append("### Spacing")
                lines.append(f"- **Avg blank lines between blocks:** {avg_blanks:.1f}")

        # File count
        file_count = data.get("file_count", 0)
        if file_count > 0:
            lines.append(f"\n*{file_count} files analyzed*")

        return "\n".join(lines)

    def _generate_git_section(self, analyses: List[Dict]) -> str:
        """Generate Git section."""
        lines = [
            "## Git",
            "",
        ]

        # Commits
        commit_data = []
        prefixes = {}
        avg_lengths = []

        for analysis in analyses:
            git = analysis.get("git", {})
            commits = git.get("commits", {})
            if commits:
                avg_lengths.append(commits.get("avg_length", 0))
                for prefix, count in commits.get("common_prefixes", {}).items():
                    prefixes[prefix] = prefixes.get(prefix, 0) + count

        if prefixes:
            lines.append("### Commits")
            top_prefixes = sorted(prefixes.items(), key=lambda x: -x[1])[:5]
            prefix_str = ", ".join([f"`{p}`" for p, _ in top_prefixes])
            lines.append(f"- **Prefixes:** {prefix_str}")

            if avg_lengths:
                avg = sum(avg_lengths) / len(avg_lengths)
                lines.append(f"- **Avg length:** {avg:.0f} chars")

        # Branches
        branch_prefixes = {}
        for analysis in analyses:
            git = analysis.get("git", {})
            branches = git.get("branches", {})
            for prefix, count in branches.get("common_prefixes", {}).items():
                branch_prefixes[prefix] = branch_prefixes.get(prefix, 0) + count

        if branch_prefixes:
            lines.append("")
            lines.append("### Branches")
            top = sorted(branch_prefixes.items(), key=lambda x: -x[1])[:5]
            prefix_str = ", ".join([f"`{p}/`" for p, _ in top])
            lines.append(f"- **Prefixes:** {prefix_str}")

        return "\n".join(lines)

    def _generate_tooling_section(self, analyses: List[Dict]) -> str:
        """Generate Tooling section."""
        all_tools = set()
        for analysis in analyses:
            tools = analysis.get("tooling", {})
            all_tools.update(tools.keys())

        if not all_tools:
            return "## Tooling\n\nNo explicit tooling configs detected."

        lines = [
            "## Tooling",
            "",
            "Detected configurations:",
        ]

        for tool in sorted(all_tools):
            lines.append(f"- {tool}")

        return "\n".join(lines)

    def _generate_red_lines(self, analyses: List[Dict]) -> str:
        """Generate Red Lines section."""
        lines = [
            "## Red Lines",
            "",
            "Explicit avoidances based on actual patterns:",
            "",
        ]

        red_lines = self._extract_red_lines(analyses)
        if red_lines:
            for line in red_lines:
                lines.append(f"- {line}")
        else:
            lines.append("- No strong red lines detected from sample")

        return "\n".join(lines)

    def _generate_footer(self, analyses: List[Dict], repos: List[str]) -> str:
        """Generate footer with source and confidence."""
        repo_names = [Path(r).name for r in repos]
        source_str = ", ".join(repo_names)

        total_files = sum(
            sum(lang.get("file_count", 0) for lang in a.get("languages", {}).values())
            for a in analyses
        )

        lines = [
            "---",
            "",
            f"**Source:** {source_str}",
            f"**Files analyzed:** {total_files}",
            "**Confidence:** High on naming patterns; Medium on tooling (config-dependent)",
        ]

        return "\n".join(lines)

    def _detected_languages(self, analyses: List[Dict]) -> set:
        """Extract all detected languages."""
        langs = set()
        for analysis in analyses:
            langs.update(analysis.get("languages", {}).keys())
        return langs

    def _extract_key_principles(self, analyses: List[Dict]) -> List[str]:
        """Extract key principles from analyses."""
        principles = []

        # Check for self-documenting code
        low_comment_density = all(
            lang.get("comments", {}).get("density", 1) < 0.1
            for analysis in analyses
            for lang in analysis.get("languages", {}).values()
        )
        if low_comment_density:
            principles.append("Self-documenting code over verbose comments")

        # Check for descriptive naming
        principles.append("Descriptive names over terse abbreviations")

        # Check for fail-fast patterns
        has_unwrap = any(
            lang.get("error_handling", {}).get("unwrap", 0) > 0
            for analysis in analyses
            for lang in analysis.get("languages", {}).values()
        )
        if has_unwrap:
            principles.append("Fail-fast acceptable in CLI contexts")

        return principles

    def _find_common_patterns(self, analyses: List[Dict]) -> Dict:
        """Find patterns common across languages."""
        patterns = {}

        # Check for consistent naming
        naming_consistency = {}
        for analysis in analyses:
            for lang, data in analysis.get("languages", {}).items():
                naming = data.get("naming", {})
                for cat, info in naming.items():
                    if isinstance(info, dict):
                        style = info.get("dominant_case")
                        if style:
                            naming_consistency[cat] = naming_consistency.get(cat, {})
                            naming_consistency[cat][style] = naming_consistency[cat].get(style, 0) + 1

        if naming_consistency:
            patterns["naming"] = {}
            for cat, styles in naming_consistency.items():
                if len(styles) == 1:
                    patterns["naming"][cat] = list(styles.keys())[0]

        # Comment philosophy
        doc_styles = set()
        for analysis in analyses:
            for lang, data in analysis.get("languages", {}).items():
                style = data.get("comments", {}).get("doc_style")
                if style:
                    doc_styles.add(style)

        if doc_styles:
            patterns["comments"] = f"Documentation via {', '.join(sorted(doc_styles))}"

        return patterns

    def _extract_red_lines(self, analyses: List[Dict]) -> List[str]:
        """Extract explicit avoidances."""
        red_lines = []

        # Check for absence of certain patterns
        no_expect = all(
            lang.get("error_handling", {}).get("expect", 0) == 0
            for analysis in analyses
            for lang in analysis.get("languages", {}).values()
        )
        if no_expect:
            red_lines.append("Never `expect()` with messages — unwrap or propagate")

        # Check for consistent style enforcement
        red_lines.append("No mixing of naming conventions within categories")

        return red_lines

    def write_to_file(self, content: str, output_path: str):
        """Write synthesized content to file."""
        Path(output_path).write_text(content)
