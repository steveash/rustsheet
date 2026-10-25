"""Generate the per-function Excel parity page from an eval report (T-2041).

Reads `eval/runs/latest/report.json` (written by `make eval`) and `eval/catalog/scope.json`, and
writes `parity.md`, `parity/functions.md` and three SVG charts under `--out-dir` (the repository's
docs directory; `make parity-report` passes it). The page is generated, never hand-edited:
regenerate it with `make eval && make parity-report`.

Each in-scope function's scored cells are split into six buckets: bit-exact, within 1 ulp,
within 2-4 ulp, within half a unit in the 15th significant digit, and visibly off (a number
further than that, or a wrong type, error or text; decision 0083, T-0061). The function sits in
the bucket of its worst cell. Its "visibly off" count is the report's blamed count: a cell is held
against the specialised functions it calls, not the glue (IF, SUM, ...) around them. The Phase 1
exit (decision 0114) reads each function's share of cells that are not visibly off, at 99.9%.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from collections import Counter, defaultdict
from collections.abc import Callable
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

from rustsheet_tools import repo_root
from rustsheet_tools.parity_svg import Bar, bar_chart

EN = "\N{EN DASH}"
LE = "\N{LESS-THAN OR EQUAL TO}"
BUCKETS = ("pass", "ulp1", "ulp4", "digits15", "visible_numeric", "visible_other")
VISIBLE = ("visible_numeric", "visible_other")
LABELS = {
    "pass": "Bit-exact",
    "ulp1": "Within 1 ulp",
    "ulp4": f"Within 2{EN}4 ulp",
    "digits15": "Within 15 digits",
    "visible_numeric": "Visibly off: number",
    "visible_other": "Visibly off: type or text",
}
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


class ParityReportError(Exception):
    """The report and catalog do not add up (or are not the shape this generator reads)."""


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
            raise ParityReportError(msg)
        if "digits15" not in closeness:
            msg = f"{name}: no `digits15` bucket; the report predates T-0061, re-run `make eval`"
            raise ParityReportError(msg)
        counts = {b: int(closeness.get(b, 0)) for b in BUCKETS}
        if sum(counts.values()) != scored:
            msg = (
                f"{name}: buckets {counts} sum to {sum(counts.values())}, but the report scores "
                f"{scored} cells (cells {entry['cells']} minus unsupported)"
            )
            raise ParityReportError(msg)
        visible = entry.get("visible", {})
        blamed = int(visible.get("numeric", 0)) + int(visible.get("other", 0))
        cat = catalog[name]
        rows.append(FunctionParity(name, str(cat["category"]), str(cat["milestone"]), counts, blamed))
    rows.sort(key=lambda f: f.name)
    scored_names = {f.name for f in rows}
    return Parity(rows, sorted(in_scope - scored_names), len(in_scope), list(report["suites"]))


# ---------------------------------------------------------------- Markdown


def pct(part: int, whole: int) -> str:
    """`part/whole` as a percentage with two decimals that never rounds up to 100 or down to 0."""
    if whole == 0:
        return "\N{EN DASH}"
    value = 100 * part / whole
    if 0 < part < whole:
        value = min(max(value, 0.01), 99.99)
    return f"{value:.2f}%"


# Markdown table cell text: a pipe would split the cell, `<`/`>` could open raw HTML, a backtick a
# code span, and a newline would end the row.
MD_CELL_ESCAPES = str.maketrans({"|": "\\|", "<": "&lt;", ">": "&gt;", "`": "&#96;", "\n": " ", "\r": " "})


def md_cell(text: str) -> str:
    """`text` made safe to put in one Markdown table cell (see MD_CELL_ESCAPES)."""
    return text.translate(MD_CELL_ESCAPES)


def table(header: list[str], rows: list[list[str]], right_from: int = 1) -> str:
    """A GitHub Markdown table; columns from `right_from` on are right-aligned (numbers)."""
    align = ["---" if i < right_from else "---:" for i in range(len(header))]
    body = [[md_cell(c) for c in line] for line in [header, *rows]]
    lines = [body[0], align, *body[1:]]
    return "\n".join("| " + " | ".join(line) + " |" for line in lines)


def bucket_header() -> list[str]:
    """The bucket labels, in bucket order, as table column headers."""
    return [LABELS[b] for b in BUCKETS]


def summary_tables(p: Parity) -> str:
    """The functions-per-bucket and checked-cells-per-bucket tables, with their headings."""
    fb, cb = p.functions_per_bucket(), p.cells_per_bucket()
    nf, nc = len(p.functions), sum(cb.values())
    funcs = table(
        ["Bucket", "Functions", "Share"],
        [[LABELS[b], f"{fb[b]:,}", pct(fb[b], nf)] for b in BUCKETS] + [["**All**", f"**{nf:,}**", ""]],
    )
    cells = table(
        ["Bucket", "Checked cells", "Share"],
        [[LABELS[b], f"{cb[b]:,}", pct(cb[b], nc)] for b in BUCKETS] + [["**All**", f"**{nc:,}**", ""]],
    )
    return f"### Functions per bucket\n\n{funcs}\n\n### Checked cells per bucket\n\n{cells}\n"


def phase1_table(p: Parity) -> str:
    """Functions per Phase 1 band: the share of their cells within 15 digits (decision 0114)."""
    per, nf = p.functions_per_phase1_band(), len(p.functions)
    rows = [[label, f"{per[label]:,}", pct(per[label], nf)] for label, _ in PHASE1_BANDS]
    return table(["Cells within 15 digits", "Functions", "Share"], [*rows, ["**All**", f"**{nf:,}**", ""]])


def category_table(p: Parity) -> str:
    """Functions per catalog category and worst-cell bucket, with the category's cell totals."""
    rows = []
    for cat, fs in p.categories():
        per = Counter(f.worst for f in fs)
        cells = sum(f.cells for f in fs)
        exact = sum(f.counts["pass"] for f in fs)
        rows.append([cat, str(len(fs)), *(str(per[b]) for b in BUCKETS), f"{cells:,}", pct(exact, cells)])
    return table(["Category", "Functions", *bucket_header(), "Cells", "Cells bit-exact"], rows)


def suites_table(p: Parity) -> str:
    """One row per eval suite: cells by harness verdict (every cell, not only catalog calls), and its
    in-scope visibly-off cells."""
    rows = []
    for s in p.suites:
        v: dict[str, int] = s.get("verdicts", {})
        cells = int(s["cells"])
        passed = int(s["cells_passed"])
        dev = v.get("deviation", 0)
        ulp, close, unsup = v.get("numeric_ulp", 0), v.get("numeric_close", 0), v.get("unsupported", 0)
        off = cells - passed - dev - ulp - close - unsup
        visible = sum(int(s.get("closeness", {}).get(b, 0)) for b in VISIBLE)
        rows.append(
            [
                str(s["name"]),
                f"{int(s['cases']):,}",
                f"{cells:,}",
                pct(passed, cells - dev),
                f"{ulp:,}",
                f"{close:,}",
                f"{off:,}",
                f"{visible:,}",
                f"{dev:,}",
                f"{unsup:,}",
            ]
        )
    header = ["Suite", "Cases", "Cells", "Bit-exact", f"{LE}4 ulp", "Within 1e-9", "Off", "Visibly off"]
    return table([*header, "Deviations", "Unsupported"], rows)


def function_table(p: Parity) -> str:
    """The full per-function table, sorted by name."""
    rows = [
        [
            f.name,
            f.category,
            f.milestone,
            f"{f.cells:,}",
            pct(f.counts["pass"], f.cells),
            *(f"{f.counts[b]:,}" for b in BUCKETS),
            f"{f.visible:,}",
            pct(f.cells - min(f.visible, f.cells), f.cells),
            LABELS[f.worst],
        ]
        for f in p.functions
    ]
    header = [
        "Function",
        "Category",
        "Milestone",
        "Cells",
        "Bit-exact %",
        "Exact",
        "1 ulp",
        f"2{EN}4 ulp",
    ]
    return table(
        [*header, "15 digits", "Off: number", "Off: type/text", "Visibly off", "Within 15 digits %", "Worst"],
        rows,
        right_from=3,
    )


def takeaway(p: Parity) -> str:
    """The landing page's one-paragraph summary."""
    fb, cb = p.functions_per_bucket(), p.cells_per_bucket()
    nf, nc = len(p.functions), sum(cb.values())
    near = fb["ulp1"] + fb["ulp4"] + fb["digits15"]
    off = sum(fb[b] for b in VISIBLE)
    blamed = p.blamed()
    return (
        f"Of **{nf:,}** implemented functions with Excel-checked cells, **{fb['pass']:,}** "
        f"({pct(fb['pass'], nf)}) match Excel bit for bit on every checked cell, and **{near:,}** more "
        f"show the same 15 digits on every cell ({fb['ulp1']:,} within 1 ulp, {fb['ulp4']:,} within 2{EN}4 ulp, "
        f"{fb['digits15']:,} within half a unit in the 15th digit). **{off:,}** have at least one cell that is "
        f"visibly off, but {p.near_off():,} of those still match bit for bit on 99% or more of their cells. "
        f"Blaming each visibly-off cell on the specialised function it calls rather than on glue such as IF "
        f"or SUM, **{len(blamed):,} in-scope functions are visibly off** "
        f"({pct(len(blamed), nf)}), over **{p.suite_visible_cells():,}** in-scope cells. Across all "
        f"**{nc:,}** checked cells (a cell calling several functions counts for each), "
        f"{pct(cb['pass'], nc)} are bit-exact and {pct(sum(cb[b] for b in VISIBLE), nc)} are visibly off."
    )


def visible_table(p: Parity) -> str:
    """The functions with visibly-off cells blamed on them, worst first."""
    rows = [[f.name, f.category, f.milestone, f"{f.visible:,}", f"{f.cells:,}"] for f in p.blamed()]
    return table(["Function", "Category", "Milestone", "Visibly off", "Cells"], rows, right_from=3)


def unscored_note(p: Parity) -> str:
    """Which in-scope catalog functions have no scored cell yet."""
    if not p.unscored:
        return f"All {p.in_scope:,} in-scope catalog functions have checked cells."
    names = ", ".join(p.unscored) if len(p.unscored) <= 20 else f"{len(p.unscored)} functions"
    return (
        f"The catalog has {p.in_scope:,} in-scope functions; {len(p.unscored):,} "
        f"{'has' if len(p.unscored) == 1 else 'have'} no scored cell yet ({names})."
    )


REGENERATE = "make eval && make parity-report"


def landing_page(p: Parity, run_date: str, commit: str) -> str:
    """The landing page (parity.md): the snapshot header, takeaway, summary tables, charts and how to read them."""
    charts = "\n\n".join(f"![{c.title}](parity/{c.name})" for c in CHARTS)
    return f"""<!-- Generated by `{REGENERATE}` (tools/src/rustsheet_tools/parity_report.py). Do not edit. -->
# RustSheet Excel parity

Per-function parity with Microsoft 365 Excel, from the eval run of **{run_date}** on `main` at
**`{commit}`**.

{takeaway(p)}

{unscored_note(p)} The full per-function table is in [parity/functions.md](parity/functions.md).

## Summary

{summary_tables(p)}
## Phase 1 (decision 0114)

Phase 1 exits when every in-scope function has at least 99.9% of its cells within 15 significant
digits of Excel's, with the right type, error and text. A function's share counts its cells less
the visibly-off cells blamed on it.

{phase1_table(p)}

## Charts

{charts}

## Visibly off

Functions with visibly-off cells blamed on them, worst first: a number further than half a unit
in the 15th significant digit of Excel's (or 0 against nonzero), or a wrong type, error or text.
A cell is blamed on the functions it calls other than glue (IF, SUM, DATE, INDEX and the like;
`GLUE` in crates/rustsheet-eval/src/closeness.rs), or on its glue when it calls nothing else.

{visible_table(p)}

## By category

Each function counts once, in the bucket of its worst checked cell.

{category_table(p)}

## Suites

The eval suites the counts come from. Suite cells include cells that call no catalog function;
"{LE}4 ulp" is the harness's `numeric_ulp` verdict, and bit-exact leaves out accepted
deviations.

{suites_table(p)}

## How to read this

- **Bit-exact**: the same value and type as Excel. Accepted deviations (decision 0036) count here.
- **Within 1 ulp** / **Within 2{EN}4 ulp**: a number at most 1, or 2 to 4, units in the last place of a
  64-bit double from Excel's.
- **Within 15 digits**: further than 4 ulp, but within half a unit in the 15th significant digit of
  Excel's value, so Excel's 15 displayed digits agree up to rounding (decision 0083).
- **Visibly off: number**: a number further than that, or 0 against nonzero.
- **Visibly off: type or text**: a wrong type, error, text or boolean.
- A function is placed by its **worst** checked cell: one visibly-off cell makes the function
  visibly off. {p.near_off():,} of those still match bit for bit on 99% or more of their cells.
- The **Visibly off** column of the function table counts only the cells blamed on that function.
- A cell that calls several functions counts for each of them, as the eval report's per-function
  counts do (`functions_in` in crates/rustsheet-eval/src/report.rs).
- Only catalog functions with `scope: in` (eval/catalog/scope.json) are counted. Cells the engine
  reports as unsupported are left out.

## Regenerate

```
{REGENERATE}
```

`make parity-report` takes `COMMIT=` and `RUN_DATE=` to name a snapshot; they default to the
checked-out commit and the report file's date.
"""


def functions_page(p: Parity, run_date: str, commit: str) -> str:
    """The per-function page (parity/functions.md): the full table."""
    return f"""<!-- Generated by `{REGENERATE}` (tools/src/rustsheet_tools/parity_report.py). Do not edit. -->
# Excel parity by function

Every in-scope function with checked cells, from the eval run of **{run_date}** on `main` at
**`{commit}`**. Columns are checked cells per bucket; "Worst" is the bucket the function is placed
in. Back to the [parity summary](../parity.md).

{function_table(p)}
"""


def _functions_per_bucket_chart(p: Parity, title: str) -> str:
    fb, nf = p.functions_per_bucket(), len(p.functions)
    bars = [Bar(LABELS[b], ((b, fb[b]),), f"{fb[b]:,} ({pct(fb[b], nf)})") for b in BUCKETS]
    return bar_chart(title, f"{nf} functions, each in the bucket of its worst checked cell.", bars)


def _functions_per_category_chart(p: Parity, title: str) -> str:
    bars = []
    for cat, fs in p.categories():
        per = Counter(f.worst for f in fs)
        bars.append(Bar(cat, tuple((b, per[b]) for b in BUCKETS), f"{len(fs)}"))
    legend = [(b, LABELS[b]) for b in BUCKETS]
    return bar_chart(title, "Functions per catalog category, stacked by worst-cell bucket.", bars, legend)


def _cells_per_bucket_chart(p: Parity, title: str) -> str:
    cb = p.cells_per_bucket()
    nc = sum(cb.values())
    bars = [Bar(LABELS[b], ((b, cb[b]),), f"{cb[b]:,} ({pct(cb[b], nc)})") for b in BUCKETS]
    return bar_chart(title, f"{nc:,} checked cells by closeness to Excel.", bars)


@dataclass(frozen=True)
class Chart:
    """One SVG chart: its file name under parity/, its title and alt text, and its builder."""

    name: str
    title: str
    build: Callable[[Parity, str], str]


CHARTS = (
    Chart("functions-per-bucket.svg", "Functions per bucket", _functions_per_bucket_chart),
    Chart("functions-per-category.svg", "Functions per category, by bucket", _functions_per_category_chart),
    Chart("cells-per-bucket.svg", "Checked cells per bucket", _cells_per_bucket_chart),
)


def charts(p: Parity) -> dict[str, str]:
    """The SVG charts by file name."""
    return {c.name: c.build(p, c.title) for c in CHARTS}


def write(p: Parity, out_dir: Path, run_date: str, commit: str) -> list[Path]:
    """Write the landing page, the per-function page and the charts; return the paths written."""
    sub = out_dir / "parity"
    sub.mkdir(parents=True, exist_ok=True)
    files = {
        out_dir / "parity.md": landing_page(p, run_date, commit),
        sub / "functions.md": functions_page(p, run_date, commit),
        **{sub / name: svg for name, svg in charts(p).items()},
    }
    for path, text in files.items():
        path.write_text(text, encoding="utf-8")
    return list(files)


def git_commit(repo: Path) -> str:
    """The short sha of `repo`'s checked-out commit."""
    out = subprocess.run(
        ["git", "rev-parse", "--short=8", "HEAD"], cwd=repo, capture_output=True, text=True, check=True
    )
    return out.stdout.strip()


COMMIT_SHA = re.compile(r"[0-9a-f]{4,40}")
RUN_DATE = re.compile(r"\d{4}-\d{2}-\d{2}")


def commit_sha(text: str) -> str:
    """argparse type: a lower-case hex commit sha, 4 to 40 digits (it is printed into the page)."""
    if not COMMIT_SHA.fullmatch(text):
        msg = f"not a commit sha (4-40 lower-case hex digits): {text!r}"
        raise argparse.ArgumentTypeError(msg)
    return text


def run_date_arg(text: str) -> str:
    """argparse type: a YYYY-MM-DD date (it is printed into the page)."""
    if not RUN_DATE.fullmatch(text):
        msg = f"not a YYYY-MM-DD date: {text!r}"
        raise argparse.ArgumentTypeError(msg)
    return text


def main(argv: list[str] | None = None) -> None:
    repo = repo_root()
    ap = argparse.ArgumentParser(prog="rustsheet-parity-report", description=__doc__.splitlines()[0])
    ap.add_argument("--report", type=Path, default=repo / "eval" / "runs" / "latest" / "report.json")
    ap.add_argument("--scope", type=Path, default=repo / "eval" / "catalog" / "scope.json")
    ap.add_argument("--out-dir", type=Path, required=True, help="the docs directory to write into")
    ap.add_argument("--commit", type=commit_sha, help="main commit the report ran on (default: git HEAD)")
    ap.add_argument(
        "--run-date", type=run_date_arg, help="date of the eval run, YYYY-MM-DD (default: the report's mtime)"
    )
    args = ap.parse_args(argv)
    try:
        report = json.loads(args.report.read_text(encoding="utf-8"))
        scope = json.loads(args.scope.read_text(encoding="utf-8"))
        parity = compute(report, scope)
    except (OSError, ValueError, KeyError, ParityReportError) as e:
        sys.exit(f"rustsheet-parity-report: {e}")
    run_date = args.run_date or datetime.fromtimestamp(args.report.stat().st_mtime, UTC).date().isoformat()
    commit = args.commit or git_commit(repo)
    for path in write(parity, args.out_dir, run_date, commit):
        print(f"wrote {path}")
    fb, cb = parity.functions_per_bucket(), parity.cells_per_bucket()
    print(f"{len(parity.functions)} functions: " + ", ".join(f"{b} {fb[b]}" for b in BUCKETS))
    print("cells: " + ", ".join(f"{b} {cb[b]:,}" for b in BUCKETS))


if __name__ == "__main__":
    main()
