"""Policy `number-formats`: the displayed text of number formats over an edge-value pool (suite gen-m10).

Each case is one sheet `F`: a grid of edge values (rows) under number formats (columns), every cell
listed in the case's `text` so the oracle records Excel's displayed text (decision 0068), plus a
`TEXT(value, format)` cross-check column per format on sheet `T` (value cells). The formats cover
every built-in format and custom ones with 1-4 sections, conditions, colors, literal text and
escapes, `@`, elapsed time and fractional seconds, fractions, scientific, thousands separators and
scaling commas, and percent. Cases rotate through a seeded shuffle of the format pool, so a run of
`CASES` cases applies every format to many values. Expectations stay empty: run
`rustsheet-oracle fill` (T-1611).
"""

from __future__ import annotations

import random
from typing import Any

from xlsxwriter.utility import xl_rowcol_to_cell

CASES = 20
FORMATS_PER_CASE = 26
VALUES_PER_CASE = 20
TEXT_CHECKS_PER_CASE = 26

# The codes of Excel's built-in formats (ids 0-49 in the en-US locale).
BUILTIN = [
    "General",
    "0",
    "0.00",
    "#,##0",
    "#,##0.00",
    "0%",
    "0.00%",
    "0.00E+00",
    "# ?/?",
    "# ??/??",
    "m/d/yyyy",
    "d-mmm-yy",
    "d-mmm",
    "mmm-yy",
    "h:mm AM/PM",
    "h:mm:ss AM/PM",
    "h:mm",
    "h:mm:ss",
    "m/d/yyyy h:mm",
    "#,##0 ;(#,##0)",
    "#,##0 ;[Red](#,##0)",
    "#,##0.00;(#,##0.00)",
    "#,##0.00;[Red](#,##0.00)",
    "mm:ss",
    "[h]:mm:ss",
    "mm:ss.0",
    "##0.0E+0",
    "@",
    '_("$"* #,##0_);_("$"* \\(#,##0\\);_("$"* "-"_);_(@_)',
    '_(* #,##0_);_(* \\(#,##0\\);_(* "-"_);_(@_)',
    '_("$"* #,##0.00_);_("$"* \\(#,##0.00\\);_("$"* "-"??_);_(@_)',
    '_(* #,##0.00_);_(* \\(#,##0.00\\);_(* "-"??_);_(@_)',
]

CUSTOM = [
    # sections and conditions
    "0.00;-0.00",
    "0.00;(0.00)",
    '0.00;-0.00;"zero"',
    '0;-0;"zero";"text: "@',
    '0.0;[Red]-0.0;0.0;"<"@">"',
    "[<100]0.0;[>=1000]#,##0;0",
    '[<0]"neg";[>0]"pos";"zero"',
    "[>=1]0.00;0.000",
    "[<=-1]0;[<1]0.0000;0.0",
    '[=0]"nil";General',
    # colors
    "[Blue]0.00;[Red]-0.00",
    "[Green]#,##0;[Magenta](#,##0);[Cyan]0",
    "[Color10]0.0",
    # literal text and escapes
    '"Total: "0.00',
    '0.00" units"',
    "\\$0.00",
    "0\\%",
    "#,##0\\ \\k",
    '"("0")"',
    "0.0 \\x \\y",
    '\\"0\\"',
    "0.00_)",
    "0.00*-",
    "*.0.00",
    # text section
    "@",
    '@" suffix"',
    '"pre "@',
    ';;;"hidden"@',
    ";;;",
    # general variants
    "General",
    '"x"General',
    "General;-General",
    "[Red]General",
    # numbers, thousands and scaling commas
    "0",
    "0.0",
    "0.000000",
    "0.000000000000000",
    "#",
    "#.##",
    "#.#0",
    "0.##",
    "000.000",
    "00000",
    "?.??",
    "??0.0?",
    "#,##0",
    "#,##0.000",
    "#,",
    "#,##0,",
    '#,##0,,"M"',
    "0.0,,",
    "#,##0,,,",
    "###,###",
    "0,000.0",
    # percent
    "0%",
    "0.0%",
    "0.000%",
    "#%",
    "0%%",
    "%0",
    # scientific
    "0E+0",
    "0.0E+00",
    "0.000E-00",
    "##0.00E+00",
    "#E+0",
    # Not "0.00e+00": Excel for the web refuses a workbook whose format has a lowercase exponent
    # (docs/excel-semantics/number-formats.md).
    "00.0E+0",
    # fractions
    "# ?/?",
    "# ??/??",
    "# ???/???",
    "?/?",
    "??/??",
    "# ?/2",
    "# ?/4",
    "# ??/16",
    "# ?/10",
    "0 ?/?",
    "# #/#",
    "?/100",
    # dates
    "yyyy-mm-dd",
    "dd/mm/yy",
    "d mmmm yyyy",
    "dddd",
    "ddd, mmm d",
    "mmmmm",
    "m/d",
    "yy",
    "e",
    "yyyy-mm-dd hh:mm:ss",
    "[$-409]mmmm d, yyyy",
    # times, elapsed and fractional seconds
    "hh:mm:ss",
    "h:mm:ss.00",
    "h:mm:ss.000",
    "ss.0",
    "[h]:mm",
    "[hh]:mm:ss",
    "[m]:ss",
    "[mm]:ss.0",
    "[s]",
    "[ss].00",
    "h AM/PM",
    "h:mm A/P",
    "hh:mm am/pm",
    "mm:ss.00",
    "[h]:mm:ss.0",
    "d \\d\\a\\y\\s h:mm",
    # currency and locale tags
    "[$$-409]#,##0.00",
    "[$€-x-euro2] #,##0.00",
    "[$¥-411]#,##0",
    "$#,##0.00;($#,##0.00)",
    "#,##0.00 [$USD]",
]

# Edge values: (spec cell input, TEXT() argument). Numbers that need exact doubles are formulas.
VALUES: list[tuple[Any, str]] = [
    (0, "0"),
    (-0.5, "-0.5"),
    (0.5, "0.5"),
    (1, "1"),
    (-1, "-1"),
    (1.5, "1.5"),
    (-1.5, "-1.5"),
    (2.5, "2.5"),
    (-2.5, "-2.5"),
    (12.345, "12.345"),
    (-1234.5678, "-1234.5678"),
    (99.5, "99.5"),
    (999.9999, "999.9999"),
    (1000, "1000"),
    (123456.789, "123456.789"),
    (-123456.789, "-123456.789"),
    (1234567890, "1234567890"),
    (1e-10, "1E-10"),
    (1e-5, "1E-5"),
    (0.001, "0.001"),
    (0.0049, "0.0049"),
    (0.125, "0.125"),
    (0.9999999999, "0.9999999999"),
    (1.0000000001, "1.0000000001"),
    (1e15, "1E+15"),
    (123456789012345, "123456789012345"),
    (-1e15, "-1E+15"),
    ("=1/3", "1/3"),
    ("=2/3", "2/3"),
    ("=-1/7", "-1/7"),
    ("=PI()", "PI()"),
    (60, "60"),
    (61, "61"),
    (45000.75, "45000.75"),
    (45658.999988426, "45658.999988426"),
    (0.000011574, "0.000011574"),
    (1.75, "1.75"),
    (36.5, "36.5"),
    (2958465.99999, "2958465.99999"),
    (-45000, "-45000"),
    ("text", '"text"'),
    ("", '""'),
    (True, "TRUE"),
    (False, "FALSE"),
]


def generate_number_formats(seed: int) -> list[dict[str, Any]]:
    """`CASES` seeded cases; together they apply every format of the pool (built-in first)."""
    rng = random.Random(seed)
    builtin = BUILTIN[:]
    custom = [code for code in dict.fromkeys(CUSTOM) if code not in BUILTIN]
    rng.shuffle(builtin)
    rng.shuffle(custom)
    pool = builtin + custom
    specs = []
    for i in range(CASES):
        start = i * FORMATS_PER_CASE
        formats = [pool[(start + k) % len(pool)] for k in range(FORMATS_PER_CASE)]
        values = rng.sample(VALUES, VALUES_PER_CASE)
        specs.append(_case(i, seed, formats, values, rng))
    return specs


def _case(
    index: int, seed: int, formats: list[str], values: list[tuple[Any, str]], rng: random.Random
) -> dict[str, Any]:
    """One case: the `values` x `formats` grid on sheet F, TEXT() checks on sheet T."""
    cells: dict[str, Any] = {}
    number_formats: dict[str, str] = {}
    text: list[str] = []
    for row, (value, _) in enumerate(values):
        for col, code in enumerate(formats):
            a1 = xl_rowcol_to_cell(row, col)
            cells[a1] = value
            if code != "General":
                number_formats[a1] = code
            text.append(f"F!{a1}")
    checks: dict[str, Any] = {}
    for row in range(TEXT_CHECKS_PER_CASE):
        code = formats[row % len(formats)]
        _, argument = rng.choice(values)
        quoted = code.replace('"', '""')
        checks[xl_rowcol_to_cell(row, 0)] = f'=TEXT({argument},"{quoted}")'
    return {
        "format": "rustsheet-case/v1",
        "description": (
            f"Displayed text of {len(formats)} number formats over {len(values)} edge values, and TEXT() "
            f"with the same codes (rustsheet-gen --policy number-formats, seed {seed}, case {index}; T-1611)."
        ),
        "tags": ["M10", "number-format", "displayed-text", "generated"],
        "sheets": [
            {"name": "F", "cells": cells, "number_formats": number_formats},
            {"name": "T", "cells": checks},
        ],
        "text": text,
    }
