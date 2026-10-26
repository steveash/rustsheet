"""The progress page's `top-functions` and `evidence` blocks (SPEC-0022 sections 7 and 8, T-0073).

`top-functions` lists the 40 in-scope functions with the highest catalog `prior` (real-world
frequency, eval/catalog/functions.yaml), ties by name, with each one's tier, attributed cells,
within-15-digit share and Excel cells. `evidence` counts the in-scope functions per band of
Excel cells, the median, and how many are covered under decision 0094, with a histogram chart.

The cells, tiers and coverage are the eval report's (`v15_checkpoint.functions[]`, T-0068 fields);
the priors are read with the generator's catalog loader, `Catalog.load`, the one YAML reader of
the catalog. This module only counts, sorts and takes a median.
"""

from __future__ import annotations

import statistics
from pathlib import Path

import yaml

from rustsheet_tools.gen.generator import Catalog
from rustsheet_tools.progress_report.markdown import md_text, pct, table
from rustsheet_tools.progress_report.model import EN, GE, FunctionParity, Parity, ProgressReportError
from rustsheet_tools.progress_report.parity import ASSETS, NEVER_OFF, TIER_LABELS
from rustsheet_tools.progress_report.svg import BLACK, BLUE, Bar, Palette, bar_chart

TOP = 40
# The evidence floor (`EVIDENCE_FLOOR`, crates/rustsheet-eval/src/evidence_floor.rs; T-0056) and
# decision 0094's bars (`CELLS_BAR`, `GENERATED_BAR`, crates/rustsheet-eval/src/v15_checkpoint.rs).
# The report does not write them, and only the band edges and the text here use them: `covered`
# is the report's own verdict.
FLOOR = 20
CELLS_BAR = 400
GENERATED_BAR = 200
# The Excel-cell bands of SPEC-0022 section 8: (label, low, high), low inclusive, high exclusive.
BANDS: tuple[tuple[str, int, int | None], ...] = (
    (f"< {FLOOR}", 0, FLOOR),
    (f"{FLOOR}{EN}199", FLOOR, 200),
    (f"200{EN}{CELLS_BAR - 1}", 200, CELLS_BAR),
    (f"{CELLS_BAR}{EN}999", CELLS_BAR, 1000),
    (f"{GE} 1000", 1000, None),
)
CHART = "evidence.svg"
CHART_TITLE = "Excel cells per function"
# One series: the reference blue, with black value ink (4.76:1 on the light fill; svg.py).
PALETTE = Palette({"cells": BLUE}, {"cells": BLACK})


def load_priors(path: Path) -> dict[str, float | None]:
    """The catalog's `prior` per function name (None when it sets none), read with `Catalog.load`.

    A missing or malformed catalog, or a prior that is not a non-negative number, is a
    `ProgressReportError`."""
    try:
        catalog = Catalog.load(path)
    except (OSError, UnicodeDecodeError, yaml.YAMLError, AttributeError, TypeError) as e:
        msg = f"{path}: {e}"
        raise ProgressReportError(msg) from e
    priors: dict[str, float | None] = {}
    for entry in catalog.functions:
        name, prior = entry.get("name"), entry.get("prior")
        ok = prior is None or (isinstance(prior, int | float) and not isinstance(prior, bool) and prior >= 0)
        if not isinstance(name, str) or not ok:
            msg = f"{path}: function {name!r} has a prior that is not a non-negative number: {prior!r}"
            raise ProgressReportError(msg)
        priors[name] = None if prior is None else float(prior)
    return priors


def band_counts(p: Parity) -> list[tuple[str, int]]:
    """In-scope functions per Excel-cell band (`BANDS`), in band order."""
    return [
        (label, sum(1 for f in p.functions if low <= f.evidence and (high is None or f.evidence < high)))
        for label, low, high in BANDS
    ]


def median_cells(p: Parity) -> float:
    """The median of the in-scope functions' Excel cells; the mean of the middle two for an even
    count, 0 when there are none."""
    return float(statistics.median(f.evidence for f in p.functions)) if p.functions else 0.0


def _number(x: float) -> str:
    """`x` with thousands separators, and one decimal only when it is not whole (a median of two)."""
    return f"{x:,.0f}" if x.is_integer() else f"{x:,.1f}"


def top_functions(p: Parity, n: int = TOP) -> list[FunctionParity]:
    """The `n` in-scope functions with the highest prior, ties by name; a function without a prior
    comes after every function with one."""
    return sorted(p.functions, key=lambda f: (f.prior is None, -(f.prior or 0.0), f.name))[:n]


def top_table(top: list[FunctionParity]) -> str:
    """One row per function of `top`, in order."""
    rows = [
        [
            f"{i:,}",
            f.name,
            f.category,
            TIER_LABELS[f.tier],
            "none" if f.prior is None else f"{f.prior:g}",
            f"{f.attributed_cells:,}",
            pct(f.rate_within15, f.rate_cells),
            f"{f.evidence:,}",
        ]
        for i, f in enumerate(top, 1)
    ]
    header = ["#", "Function", "Category", "Tier", "Prior", "Attributed cells", "Within 15 digits"]
    return table([*header, "Excel cells"], rows, right_from=4)


def top_block(p: Parity) -> str:
    """The `top-functions` block of docs/progress.md."""
    top = top_functions(p)
    never = sum(1 for f in top if f.tier in NEVER_OFF)
    at_rate = sum(1 for f in top if f.meets_rate)
    return f"""## The {TOP} most-used functions

The {len(top)} in-scope functions with the highest real-world frequency `prior` in
eval/catalog/functions.yaml, ties by name. These are the bulk of what everyday workbooks call.
**{never} of {len(top)}** never differ from Excel in the 15 digits Excel shows, on every counted
cell; **{at_rate}** meet the Phase 1 rate ({GE} 99.9% within 15 digits, and covered). The columns
are those of [docs/progress/functions.md]({ASSETS}/functions.md); Excel cells counts every
Excel-evidence cell that calls the function, accepted deviations included.

{top_table(top)}"""


def _short_list(fs: list[FunctionParity], limit: int = 20) -> str:
    named = ", ".join(f"{md_text(f.name)} ({f.evidence:,})" for f in fs[:limit])
    more = f", and {len(fs) - limit:,} more" if len(fs) > limit else ""
    return f"{named}{more}" if fs else "none"


def evidence_table(p: Parity) -> str:
    """In-scope functions per Excel-cell band, with their share."""
    n = len(p.functions)
    rows = [[label, f"{k:,}", pct(k, n)] for label, k in band_counts(p)]
    return table(["Excel cells", "Functions", "Share"], [*rows, ["**All**", f"**{n:,}**", ""]])


def evidence_block(p: Parity) -> str:
    """The `evidence` block of docs/progress.md."""
    n = len(p.functions)
    covered = sum(1 for f in p.functions if f.covered)
    thin = sorted((f for f in p.functions if f.evidence < CELLS_BAR), key=lambda f: (f.evidence, f.name))
    below_floor = sum(1 for f in thin if f.evidence < FLOOR)
    return f"""## Evidence depth

Excel cells are the scored cells whose expected value came from Excel itself (oracle values or
Excel-saved cached values) and whose formula calls the function (report.json
`v15_checkpoint.functions[].cells`). The evidence floor is {FLOOR} cells; the fifteen-digit
checkpoint (decision 0094) wants {CELLS_BAR}, {GENERATED_BAR} of them in a generated suite with
the function as the outer call.

The median in-scope function has **{_number(median_cells(p))}** Excel cells. **{covered:,} of
{n:,}** ({pct(covered, n)}) are covered under decision 0094 (the report's `covered`: the two
bars, or {FLOOR} cells for a function the catalog exempts). **{len(thin):,}** have fewer than
{CELLS_BAR} Excel cells, {below_floor:,} of them fewer than {FLOOR}: {_short_list(thin)}.

{evidence_table(p)}

![{CHART_TITLE}]({ASSETS}/{CHART})"""


def _chart(p: Parity) -> str:
    n = len(p.functions)
    bars = [Bar(label, (("cells", k),), f"{k:,} ({pct(k, n)})") for label, k in band_counts(p)]
    desc = f"{n:,} in-scope functions by their number of Excel-evidence cells."
    return bar_chart(CHART_TITLE, desc, bars, palette=PALETTE)


def blocks(p: Parity) -> dict[str, str]:
    """The `top-functions` and `evidence` blocks by name."""
    return {"top-functions": top_block(p), "evidence": evidence_block(p)}


def files(p: Parity) -> dict[str, str]:
    """The files under progress/ by name: the evidence histogram."""
    return {CHART: _chart(p)}
