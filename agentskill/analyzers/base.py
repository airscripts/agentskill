"""Base analyzer interface."""

from abc import ABC, abstractmethod
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Any


@dataclass
class AnalysisResult:
    """Result of analyzing a source file or set of files."""
    naming_patterns: Dict[str, Any]
    error_handling: Dict[str, Any]
    comments: Dict[str, Any]
    spacing: Dict[str, Any]
    imports: Dict[str, Any]
    metrics: Dict[str, float]
    file_count: int = 0

    def to_dict(self) -> Dict:
        return {
            "naming": self.naming_patterns,
            "error_handling": self.error_handling,
            "comments": self.comments,
            "spacing": self.spacing,
            "imports": self.imports,
            "metrics": self.metrics,
            "file_count": self.file_count,
        }


class LanguageAnalyzer(ABC):
    """Abstract base class for language-specific analyzers."""

    def __init__(self, sample_size: int = 30):
        self.sample_size = sample_size

    @abstractmethod
    def analyze_files(self, files: List[Path]) -> AnalysisResult:
        """Analyze a list of source files and return style metrics."""
        pass

    @abstractmethod
    def detect_case_style(self, name: str) -> str:
        """Detect the naming case style of a string."""
        pass

    @abstractmethod
    def get_language_name(self) -> str:
        """Return the language name."""
        pass

    def _read_file_lines(self, filepath: Path, max_lines: int = 1000) -> List[str]:
        """Safely read file and return lines."""
        try:
            content = filepath.read_text(errors='ignore')
            lines = content.split('\n')
            return lines[:max_lines]
        except Exception:
            return []

    def _calculate_avg(self, values: List[float]) -> float:
        """Calculate average, handling empty lists."""
        return sum(values) / len(values) if values else 0.0

    def _get_top_items(self, counts: Dict[str, int], n: int = 3) -> Dict[str, int]:
        """Get top n items by count."""
        return dict(sorted(counts.items(), key=lambda x: -x[1])[:n])
