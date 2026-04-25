"""Python language analyzer."""

import re
from pathlib import Path
from typing import Dict, List, Any

from ..base import LanguageAnalyzer, AnalysisResult
from ...constants import (
    CASE_CAMEL, CASE_KEBAB, CASE_MIXED, CASE_PASCAL,
    CASE_SCREAMING_SNAKE, CASE_SNAKE,
    NAME_VAR, NAME_FUNCTION, NAME_TYPE,
    PYTHON_VAR_KEYWORDS, PYTHON_COMMENT_STYLE,
)


class PythonAnalyzer(LanguageAnalyzer):
    """Analyzer for Python source files."""

    def get_language_name(self) -> str:
        return "python"

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
            NAME_VAR: {}, NAME_FUNCTION: {}, NAME_TYPE: {}
        }
        error_patterns = {
            "try_except": 0,
            "raise": 0,
            "assert": 0,
            "with_context": 0,
        }
        comment_metrics = {"lines": 0, "docstrings": 0, "normal": 0}
        spacing = {"blank_line_counts": []}
        imports = {"stdlib": 0, "third_party": 0, "local": 0}
        code_metrics = {"total_lines": 0}
        type_metrics = {"annotated": 0, "total_params": 0, "return_annotations": 0, "total_functions": 0}
        import_order = {"stdlib_then_third": 0, "mixed": 0, "total_files": 0}

        for filepath in files[:self.sample_size]:
            self._analyze_file(filepath, naming_patterns, error_patterns, comment_metrics, spacing, imports, code_metrics, type_metrics, import_order)

        type_density = type_metrics["annotated"] / max(type_metrics["total_params"], 1)
        return_density = type_metrics["return_annotations"] / max(type_metrics["total_functions"], 1)

        return AnalysisResult(
            naming_patterns=self._format_naming(naming_patterns),
            error_handling=error_patterns,
            comments={
                "doc_style": '"""',
                "density": comment_metrics["lines"] / max(code_metrics["total_lines"], 1),
                "docstrings": comment_metrics["docstrings"],
                "normal_comments": comment_metrics["normal"],
            },
            spacing={
                "avg_blank_lines": self._calculate_avg(spacing["blank_line_counts"]),
            },
            imports=imports,
            metrics={
                "avg_var_length": 0.0,
                "avg_fn_length": 0.0,
                "type_annotation_density": round(type_density, 2),
                "return_annotation_density": round(return_density, 2),
                "import_order_style": "stdlib_first" if import_order.get("stdlib_then_third", 0) > import_order.get("mixed", 0) else "mixed",
            },
            file_count=len(files),
        )

    def _analyze_file(self, filepath: Path, naming: Dict, errors: Dict, comments: Dict, spacing: Dict, imports: Dict, metrics: Dict, type_metrics: Dict, import_order: Dict):
        lines = self._read_file_lines(filepath)
        if not lines:
            return

        prev_was_code = False
        blank_streak = 0
        in_docstring = False

        for i, line in enumerate(lines):
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

            # Comments and docstrings
            if stripped.startswith('"""') or stripped.startswith("'''"):
                if in_docstring:
                    in_docstring = False
                else:
                    in_docstring = True
                    comments["docstrings"] += 1
                    comments["lines"] += 1
            elif stripped.startswith('#'):
                comments["normal"] += 1
                comments["lines"] += 1
            elif in_docstring:
                comments["lines"] += 1

            # Error handling
            if re.match(r'^(try|except|finally)\s*:', stripped):
                errors["try_except"] += 1
            if re.match(r'^raise\s+', stripped):
                errors["raise"] += 1
            if re.match(r'^assert\s+', stripped):
                errors["assert"] += 1
            if re.match(r'^with\s+', stripped):
                errors["with_context"] += 1

            # Naming
            self._extract_naming(line, naming)

            # Imports
            if re.match(r'^import\s+', stripped):
                imports["stdlib"] += 1
            elif re.match(r'^from\s+', stripped):
                module = re.search(r'from\s+(\S+)', stripped)
                if module:
                    mod = module.group(1).split('.')[0]
                    if mod in ['os', 'sys', 'json', 're', 'pathlib', 'typing', 'collections', 'itertools']:
                        imports["stdlib"] += 1
                    else:
                        imports["third_party"] += 1

            # Type annotations
            if re.match(r'^def\s+', stripped):
                type_metrics["total_functions"] += 1
                params = re.search(r'\((.+?)\)', stripped)
                if params:
                    param_str = params.group(1)
                    type_metrics["total_params"] += param_str.count(',') + 1
                    type_metrics["annotated"] += param_str.count(':')
                if '->' in stripped:
                    type_metrics["return_annotations"] += 1

    def _extract_naming(self, line: str, naming: Dict):
        if re.match(r'^\s*\w+\s*=\s*', line) and not line.strip().startswith('#'):
            match = re.match(r'^\s*(\w+)\s*=', line)
            if match and match.group(1) not in PYTHON_VAR_KEYWORDS:
                style = self.detect_case_style(match.group(1))
                naming[NAME_VAR][style] = naming[NAME_VAR].get(style, 0) + 1

        if re.match(r'^def\s+\w+', line):
            match = re.search(r'def\s+(\w+)', line)
            if match:
                style = self.detect_case_style(match.group(1))
                naming[NAME_FUNCTION][style] = naming[NAME_FUNCTION].get(style, 0) + 1

        if re.match(r'^class\s+\w+', line):
            match = re.search(r'class\s+(\w+)', line)
            if match:
                style = self.detect_case_style(match.group(1))
                naming[NAME_TYPE][style] = naming[NAME_TYPE].get(style, 0) + 1

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
