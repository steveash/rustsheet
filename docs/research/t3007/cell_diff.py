"""T-3007: per-cell diff of two `make eval` reports (eval/runs/latest/report.json copied aside before and after a
change). Prints, per function, the cells whose relative error to Excel went down ("better") or up ("worse"), and
writes the rows to rows.pkl next to the second report for follow-up queries.

Usage: python3 docs/research/t3007/cell_diff.py <before/report.json> <after/report.json>
"""
import collections
import json
import pickle
import re
import sys
from pathlib import Path


def load(path):
    report = json.loads(Path(path).read_text())
    return {(c["id"], cell["sheet"], cell["cell"]): cell for c in report["cases"] for cell in c.get("cells", [])}


def rel_err(cell):
    """A cell missing from a report passed (reports list only cells that are not bit-exact)."""
    if cell is None:
        return 0.0
    try:
        e, x = float(cell["expected"]["v"]), float(cell["actual"]["v"])
    except (KeyError, TypeError, ValueError):
        return None
    return abs(e - x) / abs(e) if e else abs(x)


def main():
    before, after = load(sys.argv[1]), load(sys.argv[2])
    rows = []
    for key in set(before) | set(after):
        a, b = before.get(key), after.get(key)
        ea, eb = rel_err(a), rel_err(b)
        if ea is None or eb is None or (a and b and a["actual"] == b["actual"]):
            continue
        formula = (a or b)["formula"]
        m = re.search(r"([A-Z][A-Z0-9.]+)\(", formula or "")
        rows.append((m.group(1) if m else "?", "better" if eb < ea else "worse", key, formula, ea, eb))
    for k, v in sorted(collections.Counter((r[0], r[1]) for r in rows).items()):
        print(k, v)
    Path(sys.argv[2]).with_name("rows.pkl").write_bytes(pickle.dumps(rows))


if __name__ == "__main__":
    main()
