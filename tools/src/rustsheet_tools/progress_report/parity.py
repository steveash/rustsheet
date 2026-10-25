"""The per-function Excel parity page (T-2041), now part of the progress report (SPEC-0022).

Writes `parity.md`, `parity/functions.md` and three SVG charts under the output directory (the
repository's docs directory; `make progress-report` passes it). The page is generated, never
hand-edited: regenerate it with `make eval && make progress-report`. T-0081 replaces it with the
progress page's `parity` section.

Each in-scope function's scored cells are split into six buckets: bit-exact, within 1 ulp,
within 2-4 ulp, within half a unit in the 15th significant digit, and visibly off (a number
further than that, or a wrong type, error or text; decision 0083, T-0061). The function sits in
the bucket of its worst cell. Its "visibly off" count is the report's blamed count: a cell is held
against the specialised functions it calls, not the glue (IF, SUM, ...) around them. The Phase 1
exit (decision 0114) reads each function's share of cells that are not visibly off, at 99.9%.
"""

from __future__ import annotations

from collections import Counter
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path

from rustsheet_tools.progress_report.markdown import pct, table
from rustsheet_tools.progress_report.model import BUCKETS, EN, LE, PHASE1_BANDS, VISIBLE, Parity
from rustsheet_tools.progress_report.svg import Bar, bar_chart

LABELS = {
    "pass": "Bit-exact",
    "ulp1": "Within 1 ulp",
    "ulp4": f"Within 2{EN}4 ulp",
    "digits15": "Within 15 digits",
    "visible_numeric": "Visibly off: number",
    "visible_other": "Visibly off: type or text",
}


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


REGENERATE = "make eval && make progress-report"


def landing_page(p: Parity, run_date: str, commit: str) -> str:
    """The landing page (parity.md): the snapshot header, takeaway, summary tables, charts and how to read them."""
    charts = "\n\n".join(f"![{c.title}](parity/{c.name})" for c in CHARTS)
    return f"""<!-- Generated by `{REGENERATE}` (tools/src/rustsheet_tools/progress_report/). Do not edit. -->
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

The run date and commit are the eval report's own (`run` in report.json).
"""


def functions_page(p: Parity, run_date: str, commit: str) -> str:
    """The per-function page (parity/functions.md): the full table."""
    return f"""<!-- Generated by `{REGENERATE}` (tools/src/rustsheet_tools/progress_report/). Do not edit. -->
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
