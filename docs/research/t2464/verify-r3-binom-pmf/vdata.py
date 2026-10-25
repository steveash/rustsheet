"""Data sets for the verifier, parsed from the case files by vlib (no use of cells.tsv or the worker's row map).

old   every pmf/pdf cell of eval/suites/*/cases/*.case.json outside the t2513 probes (BINOM, NEGBINOM; BETA.DIST
      only from the t2513 probes, so `old` has no BETA rows)
a, n, hold   the worker's three t2513-r3-binom-pmf probes
v     this verifier's fresh probe(s) t2513-verify-r3-binom-pmf-*
"""

from __future__ import annotations

from pathlib import Path

import vlib as V


def _cases(pattern: str):
    return sorted(V.SUITES.glob(pattern))


def load(name: str):
    if name == "old":
        rows = []
        for p in _cases("*/cases/*.case.json"):
            if p.name.startswith("t2513-"):
                continue
            txt = p.read_text()
            if "BINOM" not in txt:
                continue
            rows += [r for r in V.load_case(p) if r["kern"] != "BETA"]
        return rows
    if name == "v":
        return [
            r
            for p in _cases("probes-numeric/cases/t2513-verify-r3-binom-pmf-*.case.json")
            for r in V.load_case(p)
        ]
    return V.load_case(V.CASES / f"t2513-r3-binom-pmf-{name}.case.json")


def region(r) -> str:
    if r["kern"] == "NEG":
        return "NEG r=1" if r["n"] == 1 else "NEG"
    if r["k"] == 0:
        return f"{r['kern']} k=0"
    if r["k"] == r["n"]:
        return f"{r['kern']} k=n"
    return f"{r['kern']} interior"


def key(r):
    return (r["kern"], r["k"], r["n"], r["p"])


def probe_paths() -> list[Path]:
    return _cases("probes-numeric/cases/t2513-*binom-pmf-*.case.json")
