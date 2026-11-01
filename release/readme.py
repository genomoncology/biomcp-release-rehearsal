"""Render the repository README for package indexes."""
import re
from pathlib import Path
from urllib.parse import urljoin

ROOT = Path(__file__).resolve().parents[1]
BASE = "https://github.com/genomoncology/biomcp/blob/main/"


def description():
    text = (ROOT / "README.md").read_text()
    return re.sub(r"(!?\[[^\]]*\]\()([^ )]+)(\))",
                  lambda m: m[1] + urljoin(BASE, m[2]) + m[3], text)
