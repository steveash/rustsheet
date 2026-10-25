"""Hand-written SVG bar charts for the generated progress page (T-2041, SPEC-0022, T-0081).

GitHub shows a committed SVG through an `<img>` tag, so the chart carries its own `<style>`:
light colors by default and an `@media (prefers-color-scheme: dark)` block for dark mode. Every
shape also has an explicit `fill` attribute, so a renderer that ignores the style still draws a
readable light chart. Each palette is ordered so that adjacent keys stay apart under color-vision
deficiency (checked with the dataviz palette validator in both modes). The bucket palette shows
visibly-off numbers in red and visibly-off types, errors and text in slate; the tier palette shows
below 99% in red and unscored in slate. Legends, value labels and the tables on the page carry
identity beside the color.
"""

from __future__ import annotations

import math
from dataclasses import dataclass
from html import escape

from rustsheet_tools.progress_report.model import BUCKETS, TIERS


@dataclass(frozen=True)
class Palette:
    """(light, dark) fill per key, and the (light, dark) ink for a value label inside a segment of
    that key: each ink pair clears 4.5:1 (WCAG) on that key's fill in that theme."""

    fills: dict[str, tuple[str, str]]
    ink: dict[str, tuple[str, str]]


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


def _style(palette: Palette) -> str:
    light = [f".{c}{{fill:{lt}}}" for c, lt, _ in CHROME]
    light += [f".b-{k}{{fill:{lt}}}" for k, (lt, _) in palette.fills.items()]
    light += [f".in-{k}{{fill:{lt}}}" for k, (lt, _) in palette.ink.items()]
    dark = [f".{c}{{fill:{dk}}}" for c, _, dk in CHROME]
    dark += [f".b-{k}{{fill:{dk}}}" for k, (_, dk) in palette.fills.items()]
    dark += [f".in-{k}{{fill:{dk}}}" for k, (_, dk) in palette.ink.items()]
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
    out = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{WIDTH}" height="{height}" '
        f'viewBox="0 0 {WIDTH} {height}" role="img" aria-labelledby="t d">',
        f'<title id="t">{escape(title)}</title><desc id="d">{escape(desc)}</desc>',
        _style(palette),
        f'<rect class="bg" fill="{_fill("bg")}" x="0" y="0" width="{WIDTH}" height="{height}"/>',
        f'<g transform="translate({PAD},{PAD})">',
        _text(0, 16, title, "ink title", _fill("ink")),
    ]
    if legend:
        x = 0.0
        for key, label in legend:
            out.append(
                f'<rect class="b-{key}" fill="{palette.fills[key][0]}" x="{x:.1f}" y="38" '
                f'width="12" height="12" rx="2"/>'
            )
            out.append(_text(x + 17, 44, label, "muted", _fill("muted")))
            x += 17 + 7.5 * len(label) + 18
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
