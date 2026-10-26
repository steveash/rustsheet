"""Hand-written SVG bar and line charts for the generated progress page (T-2041, SPEC-0022,
T-0081, T-0071).

GitHub shows a committed SVG through an `<img>` tag, so the chart carries its own `<style>`:
light colors by default and an `@media (prefers-color-scheme: dark)` block for dark mode. Every
shape also has an explicit `fill` attribute, so a renderer that ignores the style still draws a
readable light chart. Each palette is ordered so that adjacent keys stay apart under color-vision
deficiency (checked with the dataviz palette validator in both modes). The bucket palette shows
visibly-off numbers in red and visibly-off types, errors and text in slate; the tier palette shows
below 99% in red and unscored in slate. Legends, value labels and the tables on the page carry
identity beside the color. A line chart's lines also carry an explicit `stroke` and a styled
dark-mode stroke, and a missing value breaks the line instead of being interpolated.
"""

from __future__ import annotations

import math
from dataclasses import dataclass, field
from html import escape
from itertools import pairwise

from rustsheet_tools.progress_report.model import BUCKETS, TIERS


@dataclass(frozen=True)
class Palette:
    """(light, dark) fill per key, and the (light, dark) ink for a value label inside a segment of
    that key: each ink pair clears 4.5:1 (WCAG) on that key's fill in that theme. A line chart
    draws no label inside a mark, so its palette has no ink."""

    fills: dict[str, tuple[str, str]]
    ink: dict[str, tuple[str, str]] = field(default_factory=dict)


# (light, dark) fills from the dataviz reference palette, stepped for each theme's surface.
BLUE = ("#2a78d6", "#3987e5")
AQUA = ("#1baf7a", "#199e70")
YELLOW = ("#eda100", "#c98500")
VIOLET = ("#6250d6", "#9085e9")
MAGENTA = ("#d55181", "#d55181")
RED = ("#e34948", "#e66767")
DEEP_RED = ("#e34948", "#d03f3f")
SLATE = ("#4b5563", "#9ca3af")
# (light, dark) ink for a value label drawn inside a segment: black in both themes, white in the
# light theme only, or white in the dark theme only.
BLACK = ("#000000", "#000000")
WHITE_LIGHT = ("#ffffff", "#000000")
WHITE_DARK = ("#000000", "#ffffff")

# The buckets of model.BUCKETS. Each ink pair clears 4.5:1 (WCAG) on its fill in that theme; the
# worst is black on light blue, 4.76:1.
BUCKET_PALETTE = Palette(
    dict(zip(BUCKETS, (BLUE, AQUA, YELLOW, VIOLET, RED, SLATE), strict=True)),
    dict(zip(BUCKETS, (BLACK, BLACK, BLACK, WHITE_LIGHT, BLACK, WHITE_LIGHT), strict=True)),
)
# The tiers of model.TIERS: validated as a six-hue categorical palette (light: worst adjacent CVD
# Delta E 15.3, normal-vision 20.8; dark: CVD 10.0), with unscored in the neutral slate of "no
# data". Ink: the worst pair is black on light blue, 4.76:1; white on the dark red is 4.70:1.
TIER_PALETTE = Palette(
    dict(zip(TIERS, (BLUE, AQUA, VIOLET, MAGENTA, YELLOW, DEEP_RED, SLATE), strict=True)),
    dict(zip(TIERS, (BLACK, BLACK, WHITE_LIGHT, BLACK, BLACK, WHITE_DARK, WHITE_LIGHT), strict=True)),
)
# (class, light, dark) for the chart chrome; GitHub's own page surfaces and ink.
CHROME = (
    ("bg", "#ffffff", "#0d1117"),
    ("ink", "#1f2328", "#f0f6fc"),
    ("muted", "#59636e", "#9198a1"),
    ("grid", "#d1d9e0", "#3d444d"),
)

WIDTH = 760
PAD = 14  # inner margin around the plot
LABEL_W = 150
END_W = 120
ROW_H = 30
BAR_H = 18
GAP = 2  # surface gap between stacked segments
MIN_INNER_LABEL_W = 30


@dataclass(frozen=True)
class Bar:
    """One chart row: a label and its segments as (bucket key, value), drawn left to right."""

    label: str
    segments: tuple[tuple[str, int], ...]
    end_label: str


def _style(palette: Palette, *, lines: bool = False) -> str:
    """The chart's style: light rules, then the dark ones. `lines` adds a stroke rule per key
    (`.s-<key>`) for a line chart's lines."""
    light = [f".{c}{{fill:{lt}}}" for c, lt, _ in CHROME]
    light += [f".b-{k}{{fill:{lt}}}" for k, (lt, _) in palette.fills.items()]
    light += [f".in-{k}{{fill:{lt}}}" for k, (lt, _) in palette.ink.items()]
    dark = [f".{c}{{fill:{dk}}}" for c, _, dk in CHROME]
    dark += [f".b-{k}{{fill:{dk}}}" for k, (_, dk) in palette.fills.items()]
    dark += [f".in-{k}{{fill:{dk}}}" for k, (_, dk) in palette.ink.items()]
    if lines:
        light += [f".s-{k}{{stroke:{lt}}}" for k, (lt, _) in palette.fills.items()]
        dark += [f".s-{k}{{stroke:{dk}}}" for k, (_, dk) in palette.fills.items()]
    return (
        "<style>text{font-family:system-ui,-apple-system,'Segoe UI',Helvetica,Arial,sans-serif;"
        "font-size:13px}.title{font-size:15px;font-weight:600}.num{font-variant-numeric:tabular-nums}"
        + "".join(light)
        + "@media (prefers-color-scheme: dark){"
        + "".join(dark)
        + "}</style>"
    )


def _fill(cls: str) -> str:
    """The light fill of a chrome class: the explicit fallback when the style is ignored."""
    return next(lt for c, lt, _ in CHROME if c == cls)


def _text(x: float, y: float, body: str, cls: str, fill: str, *, anchor: str = "start") -> str:
    """A text element; `fill` is the light-theme fallback for the classes' styled fill."""
    return (
        f'<text x="{x:.1f}" y="{y:.1f}" class="{cls}" fill="{fill}" text-anchor="{anchor}" '
        f'dominant-baseline="middle">{escape(body)}</text>'
    )


def _open(
    title: str, desc: str, height: int, style: str, *, legend: list[tuple[str, str]] | None, palette: Palette
) -> list[str]:
    """A chart's opening elements: the svg root, title and alt text, style, background, the title
    line and, when given, the legend under it (`legend` is (key, label) pairs)."""
    out = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{WIDTH}" height="{height}" '
        f'viewBox="0 0 {WIDTH} {height}" role="img" aria-labelledby="t d">',
        f'<title id="t">{escape(title)}</title><desc id="d">{escape(desc)}</desc>',
        style,
        f'<rect class="bg" fill="{_fill("bg")}" x="0" y="0" width="{WIDTH}" height="{height}"/>',
        f'<g transform="translate({PAD},{PAD})">',
        _text(0, 16, title, "ink title", _fill("ink")),
    ]
    x = 0.0
    for key, label in legend or []:
        out.append(
            f'<rect class="b-{key}" fill="{palette.fills[key][0]}" x="{x:.1f}" y="38" '
            f'width="12" height="12" rx="2"/>'
        )
        out.append(_text(x + 17, 44, label, "muted", _fill("muted")))
        x += 17 + 7.5 * len(label) + 18
    return out


def nice_step(top: int, ticks: int = 5) -> int:
    """A 1, 2 or 5 times a power of ten step that cuts `top` into about `ticks` intervals."""
    if top <= ticks:
        return 1
    raw = top / ticks
    power = int(10 ** math.floor(math.log10(raw)))
    return next(m * power for m in (1, 2, 5, 10) if m * power >= raw)


def bar_chart(
    title: str,
    desc: str,
    bars: list[Bar],
    legend: list[tuple[str, str]] | None = None,
    palette: Palette = BUCKET_PALETTE,
) -> str:
    """A horizontal bar chart, stacked when a bar has several segments, scaled to the largest bar.

    Segment keys are keys of `palette`. `legend` is (key, label) pairs drawn under the title; a
    one-series chart needs none."""
    top = max((sum(v for _, v in b.segments) for b in bars), default=0)
    step = nice_step(top)
    axis_max = max(step, math.ceil(top / step) * step)
    plot_w = WIDTH - 2 * PAD - LABEL_W - END_W
    scale = plot_w / axis_max
    plot_y = 40 + (26 if legend else 0)
    height = plot_y + len(bars) * ROW_H + 30 + PAD
    out = _open(title, desc, height, _style(palette), legend=legend, palette=palette)
    grid_bottom = plot_y + len(bars) * ROW_H
    for tick in range(0, axis_max + 1, step):
        gx = LABEL_W + tick * scale
        out.append(
            f'<rect class="grid" fill="{_fill("grid")}" x="{gx:.1f}" y="{plot_y - 4}" '
            f'width="1" height="{grid_bottom - plot_y + 4}"/>'
        )
        out.append(_text(gx, grid_bottom + 14, f"{tick:,}", "muted num", _fill("muted"), anchor="middle"))
    for i, bar in enumerate(bars):
        y = plot_y + i * ROW_H + (ROW_H - BAR_H) / 2
        mid = y + BAR_H / 2
        out.append(_text(LABEL_W - 10, mid, bar.label, "ink", _fill("ink"), anchor="end"))
        x = float(LABEL_W)
        live = [(k, v) for k, v in bar.segments if v > 0]
        for j, (key, value) in enumerate(live):
            w = value * scale
            # A tiny but nonzero count still shows as a sliver; the value label says how small.
            drawn = max(w - (GAP if j < len(live) - 1 else 0), 1.5)
            out.append(
                f'<rect class="b-{key}" fill="{palette.fills[key][0]}" x="{x:.1f}" y="{y:.1f}" '
                f'width="{drawn:.1f}" height="{BAR_H}" rx="3"><title>{escape(bar.label)}: '
                f"{value:,}</title></rect>"
            )
            if len(live) > 1 and drawn >= MIN_INNER_LABEL_W:
                out.append(
                    _text(
                        x + drawn / 2,
                        mid,
                        f"{value:,}",
                        f"num in-{key}",
                        palette.ink[key][0],
                        anchor="middle",
                    )
                )
            x += max(w, 1.5)
        out.append(_text(x + 8, mid, bar.end_label, "ink num", _fill("ink")))
    out.append("</g></svg>")
    return "\n".join(out) + "\n"


# Line chart geometry: the y-axis labels' width, the plot's height and the end labels' room.
AXIS_W = 70
PLOT_H = 180
MIN_LABEL_GAP = 15  # px between two end labels
MAX_X_LABELS = 6


@dataclass(frozen=True)
class Series:
    """One line: its palette key, its label and one value per x slot (None leaves a gap)."""

    key: str
    label: str
    values: tuple[int | None, ...]


def _runs(values: tuple[int | None, ...]) -> list[list[tuple[int, int]]]:
    """Each unbroken run of values, as (slot index, value) pairs."""
    runs: list[list[tuple[int, int]]] = []
    for i, v in enumerate(values):
        if v is None:
            continue
        if runs and runs[-1][-1][0] == i - 1:
            runs[-1].append((i, v))
        else:
            runs.append([(i, v)])
    return runs


def _spread(ys: list[float], gap: float) -> list[float]:
    """`ys` (in drawing order) moved apart by at least `gap`, keeping their order on the axis."""
    order = sorted(range(len(ys)), key=lambda i: ys[i])
    placed = list(ys)
    for prev, cur in pairwise(order):
        placed[cur] = max(placed[cur], placed[prev] + gap)
    return placed


def y_axis(values: list[int]) -> tuple[int, int, int]:
    """(lowest tick, highest tick, step) of a line chart's y axis over `values`. It starts at 0,
    unless every value sits in a band narrower than half the largest: then it starts one step
    below the band, so a change of a few functions in hundreds is still visible."""
    hi = max(values, default=0)
    lo = min(values, default=0)
    if lo == 0 or 2 * (hi - lo) >= hi:
        step = nice_step(hi)
        return 0, max(step, math.ceil(hi / step) * step), step
    step = nice_step(max(hi - lo, hi // 20, 1))
    low = max(0, (lo // step - 1) * step)
    return low, max(low + step, math.ceil(hi / step) * step), step


def line_chart(title: str, desc: str, x_labels: list[str], series: list[Series], palette: Palette) -> str:
    """A line chart over the slots `x_labels` (dates, oldest first), from 0 to the largest value.

    A slot whose value is None is a gap: the line stops before it and starts again after it, and a
    point with no neighbour is drawn as a dot. The last point of each series is a dot with its
    value beside it. More than one series gets a legend; the title names a single one. The y axis
    is `y_axis`'s, so its lowest tick label says where it starts."""
    slots = len(x_labels)
    axis_min, axis_max, step = y_axis([v for s in series for v in s.values if v is not None])
    legend = [(s.key, s.label) for s in series] if len(series) > 1 else None
    plot_y = 40 + (26 if legend else 0)
    plot_w = WIDTH - 2 * PAD - AXIS_W - END_W
    height = plot_y + PLOT_H + 30 + PAD
    out = _open(title, desc, height, _style(palette, lines=True), legend=legend, palette=palette)

    def x_at(i: int) -> float:
        return AXIS_W + (plot_w / 2 if slots == 1 else i * plot_w / (slots - 1))

    def y_at(v: int) -> float:
        return plot_y + PLOT_H - (v - axis_min) * PLOT_H / (axis_max - axis_min)

    for tick in range(axis_min, axis_max + 1, step):
        gy = y_at(tick)
        out.append(
            f'<rect class="grid" fill="{_fill("grid")}" x="{AXIS_W}" y="{gy:.1f}" width="{plot_w}" height="1"/>'
        )
        out.append(_text(AXIS_W - 8, gy, f"{tick:,}", "muted num", _fill("muted"), anchor="end"))
    every = max(1, math.ceil(slots / MAX_X_LABELS))
    shown = sorted({*range(0, slots, every), slots - 1})
    if len(shown) > 1 and shown[-1] - shown[-2] < every:
        shown.pop(-2)  # keep the last date's label clear of the one before it
    for i in shown:
        out.append(
            _text(x_at(i), plot_y + PLOT_H + 16, x_labels[i], "muted num", _fill("muted"), anchor="middle")
        )
    ends: list[tuple[int, int]] = []
    for s in series:
        light = palette.fills[s.key][0]
        runs = _runs(s.values)
        for run in runs:
            if len(run) > 1:
                points = " ".join(f"{x_at(i):.1f},{y_at(v):.1f}" for i, v in run)
                out.append(
                    f'<polyline class="s-{s.key}" stroke="{light}" fill="none" stroke-width="2" '
                    f'stroke-linejoin="round" points="{points}"><title>{escape(s.label)}</title></polyline>'
                )
        last = runs[-1][-1] if runs else None
        # A dot for each point with no neighbour, and for the last point.
        dots = [run[0] for run in runs if len(run) == 1]
        if last is not None and last not in dots:
            dots.append(last)
        for i, value in dots:
            out.append(
                f'<circle class="b-{s.key}" fill="{light}" cx="{x_at(i):.1f}" cy="{y_at(value):.1f}" '
                f'r="3.5"><title>{escape(s.label)}, {escape(x_labels[i])}: {value:,}</title></circle>'
            )
        if last is not None:
            ends.append(last)
    placed = _spread([y_at(v) for _, v in ends], MIN_LABEL_GAP)
    for (i, value), y in zip(ends, placed, strict=True):
        out.append(_text(x_at(i) + 10, y, f"{value:,}", "ink num", _fill("ink")))
    out.append("</g></svg>")
    return "\n".join(out) + "\n"
