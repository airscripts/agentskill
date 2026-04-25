#!/usr/bin/env python3
"""Entry point for agentskill CLI."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent))

from agentskill.cli import main

if __name__ == "__main__":
    main()
