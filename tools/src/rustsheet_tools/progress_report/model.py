"""Input loading and validation for the progress report (SPEC-0022, T-0070).

Reads the eval report and the catalog and checks they are the shape this generator reads; every
failure is a `ProgressReportError` naming the input and, for an old report shape, the task whose
field is missing. Verdicts, buckets and tiers come from the report (crates/rustsheet-eval); this
module only counts and checks them.
"""

from __future__ import annotations

import json
import re
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Any

EN = "\N{EN DASH}"
LE = "\N{LESS-THAN OR EQUAL TO}"
BUCKETS = ("pass", "ulp1", "ulp4", "digits15", "visible_numeric", "visible_other")
VISIBLE = ("visible_numeric", "visible_other")
# The visibly-off functions that still match on at least this share of their cells.
NEAR_SHARE = 0.99
# Phase 1 exit (decision 0114): this share of a function's cells within 15 digits.
PHASE1_SHARE = 0.999
# The Phase 1 bands of the parity page: (label, lowest within-15-digit share), best first.
PHASE1_BANDS = (
    (f"{chr(0x2265)} 99.9% (Phase 1 exit)", PHASE1_SHARE),
    (f"99{EN}99.9%", 0.99),
    (f"90{EN}99%", 0.90),
    ("Below 90%", 0.0),
)


class ProgressReportError(Exception):
    """An input is missing, malformed or not the shape this generator reads."""


@dataclass(frozen=True)
class FunctionParity:
    """One in-scope function's scored cells per bucket."""

    name: str
    category: str
    milestone: str
    counts: dict[str, int]
    visible: int
    """Visibly-off cells blamed on this function (the report's `visible`), not on glue."""

    @property
    def cells(self) -> int:
        """The function's scored cells (unsupported cells excluded)."""
        return sum(self.counts.values())

    @property
    def worst(self) -> str:
        """The bucket of the function's worst scored cell."""
        return next(b for b in reversed(BUCKETS) if self.counts[b])

    @property
    def visibly_off(self) -> bool:
        """Whether the function's worst cell is visibly off."""
        return self.worst in VISIBLE

    @property
    def exact_share(self) -> float:
        """The share of the function's scored cells that are bit-exact."""
        return self.counts["pass"] / self.cells

    @property
    def within15_share(self) -> float:
        """The share of the function's scored cells not visibly off by its blamed count (decision
        0114); blamed cells can come from cells it shares with others, so it never drops below 0."""
        return max(0, self.cells - self.visible) / self.cells

    @property
    def phase1_band(self) -> str:
        """The label of the Phase 1 band its within-15-digit share falls in."""
        return next(label for label, low in PHASE1_BANDS if self.within15_share >= low)


@dataclass(frozen=True)
class Parity:
    """The whole page's data: scored functions, in-scope functions with no scored cell, suites."""

    functions: list[FunctionParity]
    unscored: list[str]
    in_scope: int
    suites: list[dict[str, Any]]

    def functions_per_bucket(self) -> Counter[str]:
        """How many functions have their worst cell in each bucket."""
        return Counter(f.worst for f in self.functions)

    def cells_per_bucket(self) -> Counter[str]:
        """Scored cells per bucket, summed over functions (a cell counts once per function)."""
        total: Counter[str] = Counter()
        for f in self.functions:
            total.update(f.counts)
        return total

    def functions_per_phase1_band(self) -> Counter[str]:
        """How many functions fall in each Phase 1 band (decision 0114)."""
        return Counter(f.phase1_band for f in self.functions)

    def near_off(self) -> int:
        """How many visibly-off functions still match bit for bit on NEAR_SHARE of their cells."""
        return sum(1 for f in self.functions if f.visibly_off and f.exact_share >= NEAR_SHARE)

    def blamed(self) -> list[FunctionParity]:
        """The functions with visibly-off cells blamed on them, worst first."""
        return sorted((f for f in self.functions if f.visible), key=lambda f: (-f.visible, f.name))

    def suite_visible_cells(self) -> int:
        """In-scope visibly-off cells over all suites (what the visible-parity ratchet counts)."""
        return sum(sum(int(s.get("closeness", {}).get(b, 0)) for b in VISIBLE) for s in self.suites)

    def categories(self) -> list[tuple[str, list[FunctionParity]]]:
        """Functions grouped by catalog category, the largest category first."""
        groups: dict[str, list[FunctionParity]] = defaultdict(list)
        for f in self.functions:
            groups[f.category].append(f)
        return sorted(groups.items(), key=lambda kv: (-len(kv[1]), kv[0]))


def compute(report: dict[str, Any], scope: dict[str, Any]) -> Parity:
    """Bucket every in-scope function of `report` (see the module docstring).

    The buckets are the report's own per-function `closeness` counts (crates/rustsheet-eval/src/
    closeness.rs, T-2036), so this generator never re-derives a verdict.
    """
    catalog = {str(f["name"]): f for f in scope["functions"]}
    in_scope = {n for n, f in catalog.items() if f.get("scope", "in") == "in"}
    rows: list[FunctionParity] = []
    for entry in report["functions"]:
        name = str(entry["name"])
        if name not in in_scope:
            continue
        verdicts: dict[str, int] = entry.get("verdicts", {})
        scored = int(entry["cells"]) - verdicts.get("unsupported", 0)
        if scored <= 0:
            continue
        closeness = entry.get("closeness")
        if not isinstance(closeness, dict):
            msg = f"{name}: no `closeness` counts; the report predates T-2036, re-run `make eval`"
            raise ProgressReportError(msg)
        if "digits15" not in closeness:
            msg = f"{name}: no `digits15` bucket; the report predates T-0061, re-run `make eval`"
            raise ProgressReportError(msg)
        counts = {b: int(closeness.get(b, 0)) for b in BUCKETS}
        if sum(counts.values()) != scored:
            msg = (
                f"{name}: buckets {counts} sum to {sum(counts.values())}, but the report scores "
                f"{scored} cells (cells {entry['cells']} minus unsupported)"
            )
            raise ProgressReportError(msg)
        visible = entry.get("visible", {})
        blamed = int(visible.get("numeric", 0)) + int(visible.get("other", 0))
        cat = catalog[name]
        rows.append(FunctionParity(name, str(cat["category"]), str(cat["milestone"]), counts, blamed))
    rows.sort(key=lambda f: f.name)
    scored_names = {f.name for f in rows}
    return Parity(rows, sorted(in_scope - scored_names), len(in_scope), list(report["suites"]))


def load_object(path: Path) -> dict[str, Any]:
    """`path` parsed as a JSON object; a missing or malformed file is a `ProgressReportError`."""
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as e:
        msg = f"{path}: {e}"
        raise ProgressReportError(msg) from e
    if not isinstance(data, dict):
        msg = f"{path}: not a JSON object"
        raise ProgressReportError(msg)
    return data


# The run's commit and date are printed into the page, so they must be what `run_meta.rs` writes.
COMMIT_SHA = re.compile(r"[0-9a-f]{4,40}")
RUN_DATE = re.compile(r"\d{4}-\d{2}-\d{2}")


@dataclass(frozen=True)
class SkippedSuite:
    """A suite under eval/suites that the run did not score, and why (`run.skipped_suites`)."""

    name: str
    reason: str


@dataclass(frozen=True)
class Run:
    """The eval run the page shows: report.json `run` (T-0080) and the suites it scored."""

    date: str
    commit: str | None
    """`git rev-parse --short=8 HEAD` when the report was made; None when git failed."""
    suites: tuple[str, ...]
    skipped: tuple[SkippedSuite, ...]


def run_of(report: dict[str, Any]) -> Run:
    """The report's `run` record, checked (crates/rustsheet-eval/src/run_meta.rs)."""
    run = report.get("run")
    if not isinstance(run, dict):
        msg = "report.json has no `run` record; it predates T-0080, re-run `make eval`"
        raise ProgressReportError(msg)
    date, commit, skipped = run.get("date"), run.get("commit"), run.get("skipped_suites")
    if not isinstance(date, str) or not RUN_DATE.fullmatch(date):
        msg = f"report.json `run.date` is not a YYYY-MM-DD date: {date!r}"
        raise ProgressReportError(msg)
    if commit is not None and (not isinstance(commit, str) or not COMMIT_SHA.fullmatch(commit)):
        msg = f"report.json `run.commit` is not a commit sha (4-40 lower-case hex digits): {commit!r}"
        raise ProgressReportError(msg)
    if not isinstance(skipped, list) or not all(
        isinstance(s, dict) and isinstance(s.get("name"), str) and isinstance(s.get("reason"), str)
        for s in skipped
    ):
        msg = "report.json `run.skipped_suites` is not a list of {name, reason}"
        raise ProgressReportError(msg)
    suites = tuple(str(s["name"]) for s in report["suites"])
    return Run(date, commit, suites, tuple(SkippedSuite(s["name"], s["reason"]) for s in skipped))
