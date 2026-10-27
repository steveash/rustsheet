"""The progress page's `parity` block and the all-functions page (SPEC-0022, T-0081).

The `parity` block shows the Phase 1 line of report.json `v15_checkpoint` (decisions 0114 and
0118) with the stricter count as information, the in-scope functions per worst-cell tier, the
tiers by catalog category, the attributed Excel cells by closeness and the suites. It links three
SVG charts under `progress/`. `progress/functions.md` lists every in-scope function.

Every tier, attribution and Phase 1 verdict is the report's (crates/rustsheet-eval/src/
attribution.rs, v15_checkpoint.rs). A cell is attributed to a function when that function is its
formula's only call outside the harness glue list (`GLUE` in closeness.rs); below 20 attributed
cells a function is placed by every Excel cell that calls it. This module only adds up counts.
"""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path

from rustsheet_tools.progress_report.markdown import REGENERATE, pct, table
from rustsheet_tools.progress_report.model import BUCKETS, EN, GE, LE, TIERS, VISIBLE, Parity, Run, Suite
from rustsheet_tools.progress_report.svg import TIER_PALETTE, Bar, bar_chart

ASSETS = "progress"
FUNCTIONS_PAGE = "functions.md"

# Short tier labels for tables, charts and legends, and what each tier means.
TIER_LABELS = {
    "exact": "Exact",
    "ulp4": f"{LE} 4 ulp",
    "digits15": "15 digits",
    "rate999": f"{GE} 99.9%",
    "rate99": f"{GE} 99%",
    "below99": "< 99%",
    "unscored": "Unscored",
}
TIER_MEANINGS = {
    "exact": "every cell bit-exact",
    "ulp4": "every cell within 4 ulp",
    "digits15": "every cell within 15 significant digits",
    "rate999": f"{GE} 99.9% of cells within 15 digits",
    "rate99": f"99{EN}99.9% of cells within 15 digits",
    "below99": "< 99% of cells within 15 digits",
    "unscored": "no scored Excel cell",
}
BUCKET_LABELS = {
    "pass": "Bit-exact",
    "ulp1": "Within 1 ulp",
    "ulp4": f"Within 2{EN}4 ulp",
    "digits15": "Within 15 digits",
    "visible_numeric": "Visibly off: number",
    "visible_other": "Visibly off: type or text",
}
# The tiers whose every counted cell shows Excel's 15 digits.
NEVER_OFF = ("exact", "ulp4", "digits15")


def phase1_lines(p: Parity) -> str:
    """The Phase 1 line (decision 0114, counted as decision 0118 says) and the stricter count."""
    ph = p.phase1
    verdict = "met" if ph.met else "not met"
    return (
        f"**Phase 1 (decision 0114): {verdict}.** **{ph.at_rate:,} of {ph.in_scope:,}** in-scope "
        f"functions are covered with {GE} 99.9% of their attributed Excel cells within 15 digits "
        f"(decision 0118; report.json `v15_checkpoint`).\n\n"
        f"_Information, gates nothing:_ {ph.at_strict_rate:,} of {ph.in_scope:,} functions are "
        f"covered with {GE} 99.9% of every Excel cell that calls them within 15 digits, each "
        f"visibly-off cell blamed on every non-glue function of its formula (decision 0088)."
    )


def tier_table(p: Parity) -> str:
    """Functions per tier (report.json `tiers`), with the fallbacks placed by every Excel cell."""
    n = sum(t.functions for t in p.tiers)
    rows = [
        [
            TIER_LABELS[t.tier],
            TIER_MEANINGS[t.tier],
            f"{t.functions:,}",
            pct(t.functions, n),
            f"{t.fallback:,}",
        ]
        for t in p.tiers
    ]
    total = ["**All**", "", f"**{n:,}**", "", f"{sum(t.fallback for t in p.tiers):,}"]
    return table(["Tier", "Meaning", "Functions", "Share", "Placed by every Excel cell"], [*rows, total], 2)


def tier_summary(p: Parity) -> str:
    """One sentence: the functions never visibly off, and those below 99%."""
    n = sum(t.functions for t in p.tiers)
    never = sum(p.tier_functions(t) for t in NEVER_OFF)
    below = p.tier_functions("below99")
    return (
        f"**{never:,} of {n:,}** in-scope functions ({pct(never, n)}) never differ from Excel in "
        f"the 15 digits Excel shows, on every counted cell; **{below:,}** have fewer than 99% of "
        f"their cells within 15 digits. The Phase 1 line above counts a function once 99.9% of its "
        f"cells match; this stricter figure counts only functions with no cell off at all."
    )


def category_table(p: Parity) -> str:
    """Functions per catalog category and tier."""
    rows = [[cat, f"{per.total():,}", *(f"{per[t]:,}" for t in TIERS)] for cat, per in p.categories()]
    return table(["Category", "Functions", *(TIER_LABELS[t] for t in TIERS)], rows)


def closeness_table(p: Parity) -> str:
    """Attributed Excel cells per closeness bucket."""
    cells = p.attributed_cells()
    n = sum(cells.values())
    rows = [[BUCKET_LABELS[b], f"{cells[b]:,}", pct(cells[b], n)] for b in BUCKETS]
    return table(["Closeness", "Attributed cells", "Share"], [*rows, ["**All**", f"**{n:,}**", ""]])


def closeness_summary(p: Parity) -> str:
    """One sentence: how many attributed cells, and how many of them are visibly off."""
    cells = p.attributed_cells()
    n = sum(cells.values())
    off = sum(cells[b] for b in VISIBLE)
    return (
        f"**{n:,}** attributed Excel cells: {pct(cells['pass'], n)} bit-exact, "
        f"{pct(n - off, n)} within 15 digits, **{off:,}** ({pct(off, n)}) visibly off."
    )


def _known(s: Suite) -> str:
    if s.known_failures is None:
        return "no baseline"
    text = f"; text {s.text_known_failures:,}" if s.text_known_failures else ""
    return f"{s.known_failures:,}{text}"


def suites_table(p: Parity) -> str:
    """One row per eval suite: cells by harness verdict, in-scope visibly-off cells, workbooks
    (cases with a scored in-scope cell) by their worst cell, timeouts and known failures."""
    rows = []
    for s in p.suites:
        wb = s.workbooks
        rows.append(
            [
                s.name,
                f"{s.cases:,}",
                f"{s.cells:,}",
                pct(min(s.passed, s.scored), s.scored),
                pct(min(s.passed + s.ulp, s.scored), s.scored),
                f"{s.visible:,}",
                f"{wb['scored']:,}",
                f"{wb['all_exact']:,}",
                f"{wb['all_within15']:,}",
                f"{wb['any_visible']:,}",
                f"{s.timeouts:,}",
                _known(s),
            ]
        )
    header = ["Suite", "Cases", "Cells", "Bit-exact", f"{LE} 4 ulp", "Visibly off", "Workbooks"]
    return table(
        [*header, "All exact", "All within 15 digits", "Any visibly off", "Timeouts", "Known failures"], rows
    )


def real_world_line(p: Parity) -> str:
    """The share of real-world calls landing within 15 digits: each function's attributed
    within-15-digit share weighted by its real-world calls. A run without calipers or poi says so
    instead of showing a partial number."""
    corpora = " and ".join(p.real_world_suites)
    missing = [s for s in p.real_world_suites if s not in {x.name for x in p.suites}]
    if missing or not corpora:
        return f"Real-world calls: not measured, {' and '.join(missing) or 'no corpus'} did not run."
    weighted = [f for f in p.functions if f.real_world_calls and f.attributed_cells]
    calls = sum(f.real_world_calls for f in weighted)
    if calls == 0:
        return f"Real-world calls: none of the {corpora} formula cells call an attributed function."
    within = sum(f.real_world_calls * f.attributed_within15 / f.attributed_cells for f in weighted)
    share = f"{100 * within / calls:.2f}%"
    return (
        f"Real-world calls: **{share}** of the {calls:,} calls to attributed in-scope functions in the {corpora} "
        "formula cells land within 15 digits, weighting each function's attributed within-15-digit share by its "
        "calls (report.json `functions[].real_world_calls`)."
    )


def block(p: Parity) -> str:
    """The `parity` block of docs/progress.md."""
    charts = {c.name: f"![{c.title}]({ASSETS}/{c.name})" for c in CHARTS}
    return f"""## Parity

{phase1_lines(p)}

{real_world_line(p)}

Each in-scope function is placed by the **worst** of its counted Excel cells, so one off cell
drops it a tier. A cell counts toward a function only when that function is its formula's only
call outside the glue list (IF, SUM, INDEX and the like); a function with fewer than 20 such
cells is placed by every Excel cell that calls it. "Within 15 digits" means Excel's displayed
value is the same. Every function is listed in [docs/progress/functions.md]({ASSETS}/{FUNCTIONS_PAGE}).

### Functions by worst-cell tier

{tier_summary(p)}

{tier_table(p)}

{charts["tiers.svg"]}

### Tiers by category

{category_table(p)}

{charts["tiers-by-category.svg"]}

### Attributed cells by closeness

{closeness_summary(p)}

{closeness_table(p)}

{charts["cells-by-closeness.svg"]}

### Suites

Every cell each suite scored. Bit-exact and {LE} 4 ulp leave out accepted deviations (decision
0036). Visibly off and the workbook columns count in-scope cells; a workbook is a case with a
scored in-scope cell. Known failures are the `F` lines of eval/baseline/&lt;suite&gt;.txt, with the
displayed-text (`TF`) lines after "text".

{suites_table(p)}"""


METHOD = """- **Parity.** Tiers, attribution and the Phase 1 line are the eval report's own (report.json
  `tiers`, `functions[].tier`, `attribution`, `attributed`, `excel`, and `v15_checkpoint`;
  crates/rustsheet-eval/src/attribution.rs and v15_checkpoint.rs, decision 0118). The glue list
  is `GLUE` in crates/rustsheet-eval/src/closeness.rs. corpus-bench values come from LibreOffice,
  not Excel, so its cells are in no function's counts. Known failures are counted from
  eval/baseline/, never edited."""


def function_table(p: Parity) -> str:
    """Every in-scope function, sorted by name."""
    rows = [
        [
            f.name,
            f.category,
            f.milestone,
            TIER_LABELS[f.tier],
            "sole call" if f.attribution == "sole" else "every call",
            f"{f.attributed_cells:,}",
            f"{f.excel_cells:,}",
            pct(f.rate_within15, f.rate_cells),
            "yes" if f.meets_rate else "no",
        ]
        for f in p.functions
    ]
    header = ["Function", "Category", "Milestone", "Tier", "Placed by", "Attributed cells", "Excel cells"]
    return table([*header, "Within 15 digits", "Phase 1 rate met"], rows, right_from=5)


def functions_page(p: Parity, run: Run) -> str:
    """docs/progress/functions.md: every in-scope function."""
    commit = f"`{run.commit}`" if run.commit else "an unknown commit"
    return f"""<!-- Generated by `{REGENERATE}` (tools/src/rustsheet_tools/progress_report/). Do not edit. -->
# Every in-scope function

All {len(p.functions):,} in-scope catalog functions, from the eval run of **{run.date}** at
{commit}. Back to the [progress page](../progress.md#parity).

- **Tier**: the function's worst counted cell (see the progress page).
- **Placed by**: "sole call" counts the Excel cells whose formula calls it as the only function
  outside the glue list; "every call", below 20 such cells, counts every Excel cell that calls it.
- **Attributed cells** / **Excel cells**: its scored sole-call cells, and every scored Excel cell
  that calls it.
- **Within 15 digits**: the share of the counted cells within 15 digits, which the Phase 1 rate
  reads; **Phase 1 rate met**: covered with {GE} 99.9% (report.json `v15_checkpoint`).

{function_table(p)}
"""


def _tiers_chart(p: Parity, title: str) -> str:
    n = sum(t.functions for t in p.tiers)
    bars = [
        Bar(TIER_LABELS[t.tier], ((t.tier, t.functions),), f"{t.functions:,} ({pct(t.functions, n)})")
        for t in p.tiers
    ]
    return bar_chart(
        title,
        f"{n} in-scope functions, each in the tier of its worst counted cell.",
        bars,
        palette=TIER_PALETTE,
    )


def _tiers_by_category_chart(p: Parity, title: str) -> str:
    bars = [Bar(cat, tuple((t, per[t]) for t in TIERS), f"{per.total():,}") for cat, per in p.categories()]
    legend = [(t, TIER_LABELS[t]) for t in TIERS]
    desc = "In-scope functions per catalog category, stacked by worst-cell tier."
    return bar_chart(title, desc, bars, legend, palette=TIER_PALETTE)


def _cells_by_closeness_chart(p: Parity, title: str) -> str:
    cells = p.attributed_cells()
    n = sum(cells.values())
    bars = [Bar(BUCKET_LABELS[b], ((b, cells[b]),), f"{cells[b]:,} ({pct(cells[b], n)})") for b in BUCKETS]
    return bar_chart(title, f"{n:,} attributed Excel cells by closeness to Excel.", bars)


@dataclass(frozen=True)
class Chart:
    """One SVG chart: its file name under progress/, its title and alt text, and its builder."""

    name: str
    title: str
    build: Callable[[Parity, str], str]


CHARTS = (
    Chart("tiers.svg", "Functions by worst-cell tier", _tiers_chart),
    Chart("tiers-by-category.svg", "Tiers by category", _tiers_by_category_chart),
    Chart("cells-by-closeness.svg", "Attributed cells by closeness", _cells_by_closeness_chart),
)


def files(p: Parity, run: Run) -> dict[str, str]:
    """The files under progress/ by name: the all-functions page and the charts."""
    return {FUNCTIONS_PAGE: functions_page(p, run), **{c.name: c.build(p, c.title) for c in CHARTS}}


def write(rendered: dict[str, str], out_dir: Path) -> list[Path]:
    """Write `rendered` (from `files`) under `out_dir`/progress/ byte for byte; return the paths."""
    sub = out_dir / ASSETS
    sub.mkdir(parents=True, exist_ok=True)
    written = []
    for name, text in rendered.items():
        path = sub / name
        path.write_bytes(text.encode("utf-8"))
        written.append(path)
    return written
