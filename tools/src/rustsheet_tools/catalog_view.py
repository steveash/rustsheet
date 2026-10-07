"""Write eval/catalog/scope.json, the harness's view of the function catalog (T-0051).

The eval harness (crates/rustsheet-eval) has no YAML reader, so it reads this generated JSON
instead of eval/catalog/functions.yaml (decision 0040). tools/tests/test_catalog_view.py fails
when the committed view drifts from the YAML; regenerate it with `uv run rustsheet-catalog-view`.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

from rustsheet_tools.gen.generator import Catalog

REPO = Path(__file__).resolve().parents[3]
CATALOG = REPO / "eval" / "catalog" / "functions.yaml"
VIEW = CATALOG.with_name("scope.json")
FIELDS = ("milestone", "scope", "reason", "oracle")


def render(catalog: Path = CATALOG) -> str:
    """The view of `catalog` as JSON text: one entry per function, sorted by name.

    `scope` is always present (an entry without it is in scope); `reason` and `oracle` only
    when the catalog sets them."""
    rows: list[str] = []
    for entry in sorted(Catalog.load(catalog).functions, key=lambda f: str(f["name"])):
        row: dict[str, Any] = {"name": entry["name"]}
        row.update({k: entry[k] for k in FIELDS if k in entry})
        row.setdefault("scope", "in")
        rows.append(json.dumps(row, ensure_ascii=False))
    # One function per line keeps diffs of the committed view readable.
    header = {
        "source": "eval/catalog/functions.yaml",
        "generated_by": "cd tools && uv run rustsheet-catalog-view",
    }
    head = ",\n".join(f" {json.dumps(k)}: {json.dumps(v)}" for k, v in header.items())
    return "{\n" + head + ',\n "functions": [\n  ' + ",\n  ".join(rows) + "\n ]\n}\n"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="fail if the view is stale")
    args = parser.parse_args()
    text = render()
    if args.check:
        if VIEW.read_text() != text:
            raise SystemExit(f"{VIEW} is stale; run `cd tools && uv run rustsheet-catalog-view`")
        return
    VIEW.write_text(text)
    print(f"wrote {VIEW}")


if __name__ == "__main__":
    main()
