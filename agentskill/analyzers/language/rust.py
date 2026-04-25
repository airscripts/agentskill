"""Rust language analyzer."""

import re
from pathlib import Path
from typing import Dict, List, Tuple, Any

from ..base import LanguageAnalyzer, AnalysisResult
from ...constants import (
    CASE_CAMEL, CASE_KEBAB, CASE_MIXED, CASE_PASCAL,
    CASE_SCREAMING_SNAKE, CASE_SNAKE,
    NAME_VAR, NAME_FUNCTION, NAME_TYPE, NAME_CONST,
)


class RustAnalyzer(LanguageAnalyzer):
    """Analyzer for Rust source files."""

    def get_language_name(self) -> str:
        return "rust"

    def detect_case_style(self, name: str) -> str:
        if name.isupper() and '_' in name:
            return CASE_SCREAMING_SNAKE
        if name.islower() and '_' in name:
            return CASE_SNAKE
        if name.islower() and '-' in name:
            return CASE_KEBAB
        if name[0].islower() and '_' not in name and '-' not in name:
            return CASE_CAMEL
        if name[0].isupper() and '_' not in name and '-' not in name:
            return CASE_PASCAL
        return CASE_MIXED

    def analyze_files(self, files: List[Path]) -> AnalysisResult:
        naming_patterns = {
            NAME_VAR: {}, NAME_FUNCTION: {}, NAME_TYPE: {}, NAME_CONST: {}
        }
        error_patterns = {"unwrap": 0, "expect": 0, "?": 0, "panic": 0, "Result": 0}
        comment_metrics = {"lines": 0, "doc": 0, "normal": 0}
        spacing = {"blank_line_counts": []}
        imports = {"std": 0, "crate": 0, "external": 0}
        code_metrics = {"total_lines": 0}

        for filepath in files[:self.sample_size]:
            self._analyze_file(filepath, naming_patterns, error_patterns, comment_metrics, spacing, imports, code_metrics)

        # Build result
        return AnalysisResult(
            naming_patterns=self._format_naming(naming_patterns),
            error_handling=error_patterns,
            comments={
                "doc_style": "///",
                "density": comment_metrics["lines"] / max(code_metrics["total_lines"], 1),
                "doc_comments": comment_metrics["doc"],
                "normal_comments": comment_metrics["normal"],
            },
            spacing={
                "avg_blank_lines": self._calculate_avg(spacing["blank_line_counts"]),
            },
            imports=imports,
            metrics={
                "avg_var_length": 0.0,
                "avg_fn_length": 0.0,
            },
            file_count=len(files),
        )

    def _analyze_file(self, filepath: Path, naming: Dict, errors: Dict, comments: Dict, spacing: Dict, imports: Dict, metrics: Dict):
        lines = self._read_file_lines(filepath)
        if not lines:
            return

        prev_was_code = False
        blank_streak = 0

        for line in lines:
            stripped = line.strip()
            metrics["total_lines"] += 1

            # Track blank lines
            if not stripped:
                blank_streak += 1
                continue

            if prev_was_code and blank_streak > 0:
                spacing["blank_line_counts"].append(blank_streak)
            blank_streak = 0
            prev_was_code = True

            # Comments
            if stripped.startswith('///') or stripped.startswith('//!'):
                comments["doc"] += 1
                comments["lines"] += 1
            elif stripped.startswith('//'):
                comments["normal"] += 1
                comments["lines"] += 1

            # Error patterns
            if 'unwrap()' in line:
                errors["unwrap"] += 1
            if 'expect(' in line:
                errors["expect"] += 1
            if '?' in line and not line.strip().startswith('//'):
                errors["?"] += 1
            if 'panic!' in line:
                errors["panic"] += 1
            if 'Result<' in line:
                errors["Result"] += 1

            # Naming patterns
            self._extract_naming(line, naming)

            # Imports
            if stripped.startswith('use '):
                if 'std::' in line:
                    imports["std"] += 1
                elif 'crate::' in line:
                    imports["crate"] += 1
                else:
                    imports["external"] += 1

    def _extract_naming(self, line: str, naming: Dict):
        if 'let ' in line:
            match = re.search(r'let\s+(?:mut\s+)?(\w+)', line)
            if match:
                style = self.detect_case_style(match.group(1))
                naming[NAME_VAR][style] = naming[NAME_VAR].get(style, 0) + 1

        if 'fn ' in line:
            match = re.search(r'fn\s+(\w+)', line)
            if match:
                style = self.detect_case_style(match.group(1))
                naming[NAME_FUNCTION][style] = naming[NAME_FUNCTION].get(style, 0) + 1

        if 'struct ' in line or 'enum ' in line or 'trait ' in line:
            match = re.search(r'(?:struct|enum|trait)\s+(\w+)', line)
            if match:
                style = self.detect_case_style(match.group(1))
                naming[NAME_TYPE][style] = naming[NAME_TYPE].get(style, 0) + 1

        if 'const ' in line:
            match = re.search(r'const\s+(\w+)', line)
            if match:
                style = self.detect_case_style(match.group(1))
                naming[NAME_CONST][style] = naming[NAME_CONST].get(style, 0) + 1

    def _format_naming(self, naming: Dict) -> Dict:
        """Format naming patterns with dominance info."""
        result = {}
        for category, styles in naming.items():
            if styles:
                dominant = max(styles.items(), key=lambda x: x[1])
                result[category] = {
                    "dominant_case": dominant[0],
                    "counts": styles,
                }
            else:
                result[category] = {"dominant_case": "unknown", "counts": {}}
        return result
