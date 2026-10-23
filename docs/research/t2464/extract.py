"""T-2464: extract every oracle-filled incomplete-beta cell with numeric arguments into cells.tsv.

Run from tools/: uv run python ../docs/research/t2464/extract.py
Columns: fn, case, cell, args (|-joined repr of floats/bools), excel (repr). Only cells whose
formula is exactly one call of the function with arguments the fit lab's reader resolves
(playbook section 8: no nested calls).
"""
from __future__ import annotations

import collections
import json
import re
import subprocess
import sys
from pathlib import Path

from rustsheet_tools.fitlab.sweep import Workbook, load_workbook, select, Sweep
from rustsheet_tools.snapshot import suite_cases

FUNCS = ["BETA.DIST", "BETADIST", "F.DIST", "F.DIST.RT", "FDIST", "T.DIST", "T.DIST.RT", "T.DIST.2T", "TDIST",
         "BINOM.DIST", "BINOMDIST", "NEGBINOM.DIST", "NEGBINOMDIST", "BETA.INV", "BETAINV", "F.INV", "F.INV.RT",
         "FINV", "T.INV", "T.INV.2T", "TINV"]
OUT = Path(__file__).with_name("cells.tsv")


def main() -> None:
    suites = ["probes-numeric", "probes", "gen-m9", "gen-m5", "gen-m6", "gen-m7"]
    rows = []
    counts: collections.Counter[str] = collections.Counter()
    unparsed: collections.Counter[str] = collections.Counter()
    for suite in suites:
        for entry in suite_cases(suite):
            if not entry.get("sidecar") or not Path(entry["sidecar"]).exists():
                continue
            text = Path(entry["path"]).read_text(encoding="utf-8")
            if not any(f in text for f in FUNCS):
                continue
            book = load_workbook(Path(entry["path"]), Path(entry["sidecar"]))
            for fn in FUNCS:
                pat = re.compile(r"^=(?:_xlfn\.)?" + re.escape(fn) + r"\((.*)\)$", re.I)
                sw = Sweep()
                select(entry["id"], book, pat, sw)
                for c in sw.cells:
                    args = []
                    ok = True
                    for a in c.args:
                        v = getattr(a, "value", a)
                        if isinstance(v, bool) or isinstance(v, (int, float)):
                            args.append(repr(v))
                        else:
                            ok = False
                            break
                    if not ok:
                        unparsed[fn] += 1
                        continue
                    rows.append((fn, c.case, c.key, "|".join(args), repr(c.expected)))
                    counts[fn] += 1
                for u in sw.unparsed:
                    unparsed[fn] += 1
    rows = sorted(set(rows))
    with OUT.open("w", encoding="utf-8") as f:
        f.write("fn\tcase\tcell\targs\texcel\n")
        for r in rows:
            f.write("\t".join(r) + "\n")
    print("rows", len(rows), dict(counts))
    print("unparsed", dict(unparsed))


if __name__ == "__main__":
    main()
