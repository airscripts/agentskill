#!/usr/bin/env python3

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
text = str(ROOT)

if text not in sys.path:
    sys.path.insert(0, text)

from agentskill.main import main

if __name__ == "__main__":
    raise SystemExit(main(["analyze", *sys.argv[1:]]))
