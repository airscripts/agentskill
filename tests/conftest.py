import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
text = str(ROOT)

if text not in sys.path:
    sys.path.insert(0, text)
