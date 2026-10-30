from __future__ import annotations

import copy
import functools
import hashlib
import io
import json
import random
import re
import zipfile
from collections import Counter
from pathlib import Path
from typing import Any

import pytest

from rustsheet_tools import repo_root
from rustsheet_tools.gen import database
from rustsheet_tools.gen.common import quoted
from rustsheet_tools.gen.generator import (
    NAME_POOL,
    OUTER_CALL,
    SPILL_BLOCKER,
    SPILL_COLS,
    SPILL_ROWS,
    Catalog,
    Gen,
    col_letter,
    generate_case,
    spill_layout,
)
from rustsheet_tools.gen.names_3d import CATEGORIES as NAMES_3D_CATEGORIES
from rustsheet_tools.gen.names_3d import SPAN_SHEETS, generate_names_3d
from rustsheet_tools.gen.number_formats import BUILTIN, CUSTOM, generate_number_formats
from rustsheet_tools.gen.numeric_fp import CALL, FP_BASES, FP_K, POSITIONS, generate_numeric_fp
from rustsheet_tools.gen.numeric_rules import generate_numeric_rules
from rustsheet_tools.gen.operators_refs import CATEGORIES, generate_operators_refs
from rustsheet_tools.gen.pools import MAX_STRING_LITERAL, POOLS, lambda_arity
from rustsheet_tools.gen.tables import CATEGORIES as TABLES_CATEGORIES
from rustsheet_tools.gen.tables import FORMULA_HEADERS, escape_column, generate_tables, this_row
from rustsheet_tools.spec2xlsx import build_xlsx, parse_a1, prefix_bound_names


@functools.cache
def _loaded_catalog() -> Catalog:
    return Catalog.load(repo_root() / "eval" / "catalog" / "functions.yaml")


def catalog() -> Catalog:
    # Parsing functions.yaml takes about 0.7 s and the coverage tests call this once per seed, which
    # made this file most of CI's python job (12 min). Parse once; a deep copy (about 6 ms) keeps each
    # caller's catalog independent.
    return copy.deepcopy(_loaded_catalog())


def test_generation_is_deterministic() -> None:
    a = generate_case(catalog(), 7, milestone="M2")
    b = generate_case(catalog(), 7, milestone="M2")
    assert a == b


def test_criteria_pool_emits_text_number_and_date_criteria() -> None:
    # T-2717: criteria are quoted text (text-number, tilde escape, boolean text) or a date expression.
    formulas = " ".join(
        " ".join(
            generate_case(catalog(), seed, milestone="M2", functions=["COUNTIF", "SUMIFS"], focus=True)[
                "sheets"
            ][1]["cells"].values()
        )
        for seed in range(3)
    )
    for literal in (
        '"1"',
        '"2.5"',
        '"=2"',
        '"1E1"',
        '"x~*"',
        '"~?"',
        '"TRUE"',
        '"=FALSE"',
        '">"&DATE(2020,1,1)',
        '"<="&DATE(1900,1,1)',
    ):
        assert literal in formulas, literal


def test_milestone_scoping_excludes_later_functions() -> None:
    spec = generate_case(catalog(), 3, milestone="M1", formulas=60)
    formulas = " ".join(spec["sheets"][1]["cells"].values())
    assert "VLOOKUP(" not in formulas
    assert "SUM(" not in formulas


def test_spec_builds_valid_xlsx(tmp_path: Path) -> None:
    spec = generate_case(catalog(), 11, milestone="M2", formulas=10)
    data = build_xlsx(spec)
    assert data[:2] == b"PK"
    (tmp_path / "x.xlsx").write_bytes(data)


def test_parse_a1() -> None:
    assert parse_a1("A1") == (0, 0)
    assert parse_a1("$AB$12") == (11, 27)


def test_numeric_rules_policy() -> None:
    cases = generate_numeric_rules(5)
    assert cases == generate_numeric_rules(5)
    assert cases != generate_numeric_rules(6)
    probes = sum(len(c["check"]) for c in cases)
    assert probes >= 2000
    for c in cases:
        assert set(c["hypotheses"]) == set(c["check"])  # every probe is tagged
        assert all(c["hypotheses"].values())
        assert c["expected"] == {}
        assert build_xlsx(c)[:2] == b"PK"
    assert probes == 2738
    snap = cases[0]
    assert snap["sheets"][1]["cells"]["A1"] == "=Inputs!A1-1"
    assert snap["hypotheses"]["Calc!A1"] == "snap-addsub/top-level-sub/base=1/sign=+1/k=0"
    # Excel: unary minus binds tighter than ^, so `-2^20` would silently add instead of subtract.
    assert not re.search(
        r"-\d+\^", " ".join(f for c in cases for s in c["sheets"] for f in s["cells"].values())
    )


def test_numeric_fp_policy_covers_every_position_with_m2_functions_only() -> None:
    cases = generate_numeric_fp(3)
    assert cases == generate_numeric_fp(3)
    assert cases != generate_numeric_fp(4)  # the seed shuffles the probes' cells
    assert [c["tags"][3] for c in cases] == list(POSITIONS)
    m2 = {f["name"] for f in catalog().scoped("M2").functions}
    probes = 0
    for c in cases:
        assert c["tags"][:2] == ["generated", "M2"]
        assert set(c["hypotheses"]) == set(c["check"]) and c["expected"] == {}
        formulas = [f for s in c["sheets"] for f in s["cells"].values() if isinstance(f, str)]
        assert set(CALL.findall(" ".join(formulas))) <= m2
        assert not re.search(r"-\d+\^", " ".join(formulas))  # -2^20 is +2^20 in Excel
        assert build_xlsx(c)[:2] == b"PK"
        probes += len(c["check"])
    assert probes == 5200
    tags = [t for c in cases for t in c["hypotheses"].values()]
    assert len(set(tags)) == len(tags)  # one cell per position, form, order and pair
    assert FP_BASES == ["1", "2", "0.5", "(2^20)", "1.5"] and list(FP_K) == list(range(10))
    for want in ("root/vlookup/b-x/", "root/hlookup/x-b/", "wrapper/countif/b-x/", "sum/average-zero/b-x/"):
        assert any(t.startswith(f"numeric-fp/{want}") for t in tags), want
    root = cases[0]
    cell = next(
        a for a, t in root["hypotheses"].items() if t == "numeric-fp/root/hlookup/x-b/base=1/sign=-1/k=4"
    )
    hlookup = "HLOOKUP(4,Rows!$A$4:$J$6,{},FALSE)"
    assert root["sheets"][2]["cells"][cell.split("!")[1]] == "=" + hlookup.format(2) + "-" + hlookup.format(3)
    assert root["sheets"][0]["cells"]["B15"] == "=1*(1-4*2^-52)"
    # Lookup keys and the trailing 0 addend are numbers: a text "0" would be skipped by SUM.
    assert root["sheets"][0]["cells"]["A15"] == 15 and root["sheets"][0]["cells"]["F15"] == 0
    assert root["sheets"][1]["cells"]["E4"] == 4


M2_ADDED = [
    "HLOOKUP", "INDEX", "MATCH", "SUMIFS", "COUNTIFS", "ROUNDUP", "ROUNDDOWN", "NOT", "LEFT",
    "RIGHT", "MID", "TRIM", "UPPER", "LOWER", "CONCAT", "YEAR", "MONTH", "DAY",
]  # fmt: skip
VALID_MODES = {"value", "aggregate", "aggregate_a", "range", "criteria", "array", "any", "lambda", "name"}


def test_number_formats_policy_lists_ten_thousand_text_cells_and_every_format() -> None:
    specs = generate_number_formats(0)
    assert specs == generate_number_formats(0)
    used = set()
    texts = 0
    for spec in specs:
        grid = spec["sheets"][0]
        assert set(spec["text"]) == {f"F!{a1}" for a1 in grid["cells"]}
        texts += len(spec["text"])
        used |= {grid["number_formats"].get(a1, "General") for a1 in grid["cells"]}
        assert all(f.startswith("=TEXT(") for f in spec["sheets"][1]["cells"].values())
        build_xlsx(spec)
    assert texts >= 10_000
    assert used == set(BUILTIN) | set(CUSTOM)
    assert not any("e+" in code or "e-" in code for code in used)  # excel-graph refuses them


def test_catalog_schema() -> None:
    names = [f["name"] for f in catalog().functions]
    # Only entries with `params` are signatures; the rest just name a function (test_catalog.py).
    for f in (f for f in catalog().functions if "params" in f):
        assert f["returns"] in {"number", "text", "bool", "any"}, f["name"]
        assert isinstance(f["prior"], int | float) and f["milestone"].startswith("M"), f["name"]
        assert f.get("volatile", False) in {True, False}, f["name"]
        for p in f["params"]:
            assert p["mode"] in VALID_MODES, (f["name"], p["name"])
            rep = p.get("repeat")
            if isinstance(rep, dict):
                assert set(rep["group"]) <= {q["name"] for q in f["params"]}, f["name"]
    missing = [n for n in M2_ADDED if n not in names]
    assert not missing


def test_generator_emits_every_m2_function() -> None:
    cat = catalog().scoped("M2")
    seen: set[str] = set()
    for seed in range(60):
        seen.update(generate_case(cat, seed, milestone="M2", formulas=80)["tags"])
    assert not [n for n in M2_ADDED if n not in seen]


def emitted(milestone: str) -> set[str]:
    """Every tag (function name) the generator emits over 60 seeds at `milestone`."""
    seen: set[str] = set()
    for seed in range(60):
        seen.update(generate_case(catalog(), seed, milestone=milestone, formulas=80)["tags"])
    return seen


def test_generator_emits_every_m5_math_function_but_the_volatile_ones() -> None:
    # T-1082: every M5 math signature is generator-ready; RAND/RANDBETWEEN never match an oracle.
    m5_math = {
        f["name"]: f
        for f in catalog().functions
        if "params" in f and f["milestone"] == "M5" and f["category"] == "math"
    }
    volatile = {n for n, f in m5_math.items() if f.get("volatile")}
    assert volatile == {"RAND", "RANDBETWEEN"}
    seen = emitted("M5")
    assert not seen & volatile
    assert not sorted(set(m5_math) - volatile - seen)


def test_generator_emits_every_m5_text_function() -> None:
    # T-1087: every M5 text signature is generator-ready.
    m5_text = {
        f["name"]
        for f in catalog().functions
        if "params" in f and f["milestone"] == "M5" and f["category"] == "text"
    }
    assert len(m5_text) == 33
    seen = emitted("M5")
    assert not sorted(m5_text - seen)


def test_generator_emits_every_m5_logical_information_and_web_function() -> None:
    # T-1088: every in-scope M5 logical, information and web entry (INFO and SHEETS only in their
    # oracle-accepted forms) is generator-ready.
    m5 = {
        f["name"]
        for f in catalog().functions
        if "params" in f and f["milestone"] == "M5" and f["category"] in {"logical", "information", "web"}
    }
    assert len(m5) == 28
    seen = emitted("M5")
    assert not sorted(m5 - seen)


def test_generator_emits_only_oracle_accepted_info_and_sheets_forms() -> None:
    # T-2251: the canary refuses SHEETS() with no argument and INFO("numfile"); the generator never writes them.
    calls: set[str] = set()
    for seed in range(40):
        case = str(generate_case(catalog(), seed, milestone="M5", formulas=80))
        calls.update(re.findall(r"(?:INFO|SHEETS)\([^)]*\)", case))
    assert any(c.startswith("INFO(") for c in calls) and any(c.startswith("SHEETS(") for c in calls)
    for c in calls:
        assert c != "SHEETS()"
        assert "numfile" not in c.lower()


def test_generator_emits_every_m6_date_function() -> None:
    # T-1090: every in-scope M6 date and time entry is generator-ready.
    m6_date = {
        f["name"]
        for f in catalog().functions
        if "params" in f and f["milestone"] == "M6" and f["category"] == "date"
    }
    assert len(m6_date) == 20
    assert not sorted(m6_date - emitted("M6"))


def test_generator_emits_every_m6_lookup_function() -> None:
    # T-1238: every in-scope M6 lookup entry is generator-ready; GETPIVOTDATA reads the case's own pivot (T-2396).
    m6_lookup = {
        f["name"]
        for f in catalog().functions
        if "params" in f and f["milestone"] == "M6" and f["category"] == "lookup"
    }
    assert len(m6_lookup) == 19
    assert "GETPIVOTDATA" in m6_lookup
    assert not sorted(m6_lookup - emitted("M6"))


def test_generator_emits_every_m9_statistical_function() -> None:
    # T-1080: every in-scope M9 statistical entry is generator-ready; modes from probes/stat-argument-modes.
    m9_stat = {
        f["name"]
        for f in catalog().functions
        if "params" in f and f["milestone"] == "M9" and f["category"] == "statistical"
    }
    assert len(m9_stat) == 142
    # 142 functions at priors of 0.05-0.5 next to every earlier milestone need more draws than `emitted`'s 40 seeds.
    seen: set[str] = set()
    for seed in range(120):
        seen.update(generate_case(catalog(), seed, milestone="M9", formulas=80)["tags"])
    assert not sorted(m9_stat - seen)


def test_generator_emits_every_m9_financial_function() -> None:
    # T-1084: every in-scope M9 financial entry is generator-ready; modes from probes/fin-argument-modes.
    m9_fin = {
        f["name"]
        for f in catalog().functions
        if "params" in f and f["milestone"] == "M9" and f["category"] == "financial"
    }
    assert len(m9_fin) == 55
    seen: set[str] = set()
    for seed in range(120):
        seen.update(generate_case(catalog(), seed, milestone="M9", formulas=80)["tags"])
    assert not sorted(m9_fin - seen)


def test_generator_emits_every_m9_engineering_function() -> None:
    # T-1085: every in-scope M9 engineering entry is generator-ready; modes from probes/eng-argument-modes.
    m9_eng = {
        f["name"]
        for f in catalog().functions
        if "params" in f and f["milestone"] == "M9" and f["category"] == "engineering"
    }
    assert len(m9_eng) == 54
    seen: set[str] = set()
    for seed in range(120):
        seen.update(generate_case(catalog(), seed, milestone="M9", formulas=80)["tags"])
    assert not sorted(m9_eng - seen)


def test_generator_emits_every_m9_database_function() -> None:
    # T-1086: every in-scope M9 database entry is generator-ready; modes from probes/db-argument-modes.
    m9_db = {
        f["name"]
        for f in catalog().functions
        if "params" in f and f["milestone"] == "M9" and f["category"] == "database"
    }
    assert len(m9_db) == 12
    seen: set[str] = set()
    for seed in range(120):
        seen.update(generate_case(catalog(), seed, milestone="M9", formulas=80)["tags"])
    assert not sorted(m9_db - seen)


def test_number_enums_are_number_literals() -> None:
    # T-1084: a bond's frequency enum is 1, 2 or 4 written as a number; "2" would be a coerced text argument.
    freqs: list[str] = []
    for seed in range(60):
        spec = generate_case(catalog(), seed, milestone="M9", formulas=80)
        for formula in spec["sheets"][1]["cells"].values():
            freqs += re.findall(r"COUP(?:DAYBS|DAYS|DAYSNC|NCD|NUM|PCD)\(\d+,\d+,([^,()]*)", formula)
    assert freqs and {"1", "2", "4"} & set(freqs), freqs
    # The frequency pool's "2" is the one deliberate text frequency (T-2248); `""` is the empty-text draw of
    # every number parameter (T-2866).
    assert not any(f.startswith('"') and f not in ('"2"', '""') for f in freqs), freqs


def m7_formulas() -> list[str]:
    return [
        f
        for seed in range(40)
        for f in generate_case(catalog(), seed, milestone="M7", formulas=80)["sheets"][1]["cells"].values()
    ]


def test_generator_emits_every_m7_function_but_randarray() -> None:
    # T-1083: every in-scope M7 entry is generator-ready; RANDARRAY is volatile.
    m7 = {f["name"]: f for f in catalog().functions if "params" in f and f["milestone"] == "M7"}
    assert len(m7) == 32
    assert {n for n, f in m7.items() if f.get("volatile")} == {"RANDARRAY"}
    seen = emitted("M7")
    assert "RANDARRAY" not in seen
    assert not sorted(set(m7) - {"RANDARRAY"} - seen)


def test_bound_names_are_used_only_in_their_scope() -> None:
    # T-1083: a LET variable or LAMBDA parameter outside its LET/LAMBDA is #NAME?. The case builder
    # prefixes exactly the in-scope uses with `_xlpm.`, so no bare pool name may be left.
    bare = re.compile(r"(?<![\w.$!])(" + "|".join(NAME_POOL) + r")(?![\w(!])")
    formulas = m7_formulas()
    assert any("LET(" in f for f in formulas) and any(")(" in f for f in formulas)
    for f in formulas:
        stored = re.sub(r'"(?:[^"]|"")*"', '""', prefix_bound_names(f))
        assert not bare.search(stored), f


def test_lambdas_get_as_many_parameters_as_values() -> None:
    # T-1083: an invoked LAMBDA gets one value per parameter, MAP's LAMBDA one parameter per array,
    # MAKEARRAY's two; a mismatch is #VALUE!.
    formulas = m7_formulas()
    invoked = [m for f in formulas for m in re.findall(r"LAMBDA\(([^()]*)\)\(([^()]*)\)", f)]
    assert invoked and all(a.count(",") == b.count(",") + 1 for a, b in invoked)
    mapped = [
        m
        for f in formulas
        # Names are lower case; a body that starts with one is left out (it would read as a parameter).
        for m in re.findall(
            r"(?<![A-Z])MAP\(((?:Data![A-Z]\d+:[A-Z]\d+,)+)LAMBDA\(((?:[a-z_]+,)+)(?![a-z_])", f
        )
    ]
    assert mapped and all(a.count(",") == b.count(",") for a, b in mapped)
    # A pooled LAMBDA may mark its last parameter optional (`LAMBDA(w,[y],ISOMITTED(y))`).
    made = re.findall(r"MAKEARRAY\([^,()]*,[^,()]*,LAMBDA\(((?:\[?[a-z_]+\]?,)+)", " ".join(formulas))
    assert made and all(names.count(",") == 2 for names in made), made


def test_full_arrays_span_every_row_and_masks_compare_one_column() -> None:
    # T-1083: GROUPBY's and SORTBY's arrays and FILTER's include must be the same height.
    formulas = " ".join(m7_formulas())
    filters = re.findall(r"FILTER\((Data![A-Z]\d+:[A-Z]\d+),\((Data!([A-Z])\d+:([A-Z])\d+)[<>=]", formulas)
    assert filters
    for array, mask, c0, c1 in filters:
        assert array.startswith("Data!") and array.split(":")[0][-1] == "1" and array.endswith("12")
        assert c0 == c1 and mask.endswith("12")
    groupby = re.findall(r"_xlfn\.GROUPBY\((Data![A-Z]1:[A-Z]\d+),(Data![A-Z]1:[A-Z]\d+)", formulas)
    assert groupby and all(a.endswith("12") and b.endswith("12") for a, b in groupby)
    assert "GROUPBY(" not in formulas.replace("_xlfn.GROUPBY(", "")


def test_vector_params_get_one_column() -> None:
    # T-1238: XMATCH's lookup_array is `vector: true`; a 2-D lookup vector only gives an error.
    arrays: list[str] = []
    for seed in range(40):
        spec = generate_case(catalog(), seed, milestone="M6", formulas=80)
        for formula in spec["sheets"][1]["cells"].values():
            arrays += re.findall(r"XMATCH\([^,()]*,(Data![A-Z]\d+:[A-Z]\d+)", formula)
    assert arrays
    assert all(a[5] == a.split(":")[1][0] for a in arrays), arrays


def test_reference_params_get_one_cell_and_cell_always_gets_one() -> None:
    # T-1088: a literal ISFORMULA or CELL reference makes excel-graph refuse the workbook, a range
    # would spill ISFORMULA into the next formula, and CELL without a reference reads the cell last
    # changed. info_type mostly comes from CELL's enum.
    cell = r"Data!\$?[A-Z]\$?\d+"
    isformula: list[str] = []
    cells: list[str] = []
    for seed in range(40):
        spec = generate_case(catalog(), seed, milestone="M5", formulas=80)
        for formula in spec["sheets"][1]["cells"].values():
            isformula += re.findall(r"ISFORMULA\(([^()]*)\)", formula)
            cells += re.findall(r"(?<![A-Z.])CELL\(([^()]*)\)", formula)
    assert isformula and all(re.fullmatch(cell, a) for a in isformula), isformula
    assert cells and all(re.fullmatch(r".*," + cell, a) for a in cells), cells
    assert any(a.startswith(('"row"', '"col"', '"contents"', '"type"')) for a in cells)


def test_hard_domains_are_never_violated() -> None:
    # MUNIT of a large dimension makes excel-graph refuse the whole workbook (T-1087), so its
    # `hard` domain always gets an in-domain integer.
    calls: list[str] = []
    for seed in range(40):
        spec = generate_case(catalog(), seed, milestone="M5", formulas=80)
        for formula in spec["sheets"][1]["cells"].values():
            calls += re.findall(r"MUNIT\(([^()]*)\)", formula)
    assert calls and all(c in {"1", "2", "3", "4"} for c in calls), calls


def test_array_params_are_sometimes_ranges() -> None:
    cat = catalog().scoped("M5")
    spec = generate_case(cat, 3, milestone="M5", formulas=400)
    formulas = list(spec["sheets"][1]["cells"].values())
    # Expressions never produce a bare range: only the array branch passes one to SUMPRODUCT.
    assert any(re.search(r"SUMPRODUCT\(Data!\w+:\w+[,)]", f) for f in formulas)


def test_lift_arrays_params_never_get_a_name_or_range() -> None:
    # T-1776: a `lift: arrays` param refuses a multi-cell range, which a LAMBDA parameter
    # (BYROW's row) or a range literal would pass. The scope holds a name the call must not use.
    cat = catalog().scoped("M9")
    edate = next(f for f in cat.functions if f["name"] == "EDATE")
    assert all(p.get("lift") == "arrays" for p in edate["params"])
    gen = Gen(
        rng=random.Random(4),
        catalog=Catalog(operators=cat.operators, functions=[edate]),
        data_rows=10,
        data_cols=4,
        max_depth=3,
    )
    gen.scope = ["row_x"]
    calls = [gen.call("any", 0) for _ in range(300)]
    assert not [c for c in calls if "row_x" in c or ":" in c]


def test_generator_calls_only_entries_with_signatures() -> None:
    # T-0040 catalogs every Excel function, most without params: those must never be generated.
    cat = catalog().scoped("M10")
    assert cat.functions and all("params" in f for f in cat.functions)
    assert generate_case(catalog(), 3, milestone="M10", formulas=40)["sheets"][1]["cells"]


def test_paired_repeat_groups_stay_paired() -> None:
    cat = catalog().scoped("M2")
    countifs = next(f for f in cat.functions if f["name"] == "COUNTIFS")
    assert countifs["params"][0]["repeat"]["group"] == ["criteria_range", "criteria"]
    spec = generate_case(cat, 3, milestone="M2", formulas=300)
    # criteria are literals (a date criterion's DATE(y,1,1) is folded to one token) and ranges contain no commas,
    # so a bare COUNTIFS has 2n-1 commas.
    cells = (re.sub(r"DATE\(\d+,1,1\)", "DATE", f) for f in spec["sheets"][1]["cells"].values())
    bare = [f for f in cells if re.fullmatch(r"=COUNTIFS\([^()]*\)", f)]
    assert bare
    assert all(f.count(",") % 2 == 1 for f in bare)


def test_build_xlsx_pins_the_creation_time() -> None:
    # Without the pin, docProps/core.xml carries the wall clock and the bytes differ per run.
    spec = generate_case(catalog(), 11, milestone="M2", formulas=10)
    first = build_xlsx(spec)
    with zipfile.ZipFile(io.BytesIO(first)) as z:
        assert b"2000-01-01T00:00:00Z" in z.read("docProps/core.xml")
    assert build_xlsx(spec) == first


def _operators_refs_formulas() -> list[tuple[str, str, str]]:
    """(category/form tag, cell, formula) for every probe of the default operators-refs run."""
    return [
        (c["hypotheses"][f"Calc!{cell}"], cell, f)
        for c in generate_operators_refs(0)
        for cell, f in c["sheets"][-1]["cells"].items()
    ]


def _strip_quoted(formula: str) -> str:
    """Drops string literals and quoted sheet names so only formula syntax remains."""
    return re.sub(r"'(?:[^']|'')*'", "S", re.sub(r'"(?:[^"]|"")*"', "T", formula))


# One area operand: `Sheet!`-prefixed cell, cell range (corners may repeat the sheet), whole
# column or whole row. Rows are 1-based; columns are letters.
_REF = r"\$?[A-Z]{1,3}\$?\d+"
_AREA = re.compile(
    rf"(?:\w+|S)!(?:{_REF}(?::(?:(?:\w+|S)!)?{_REF})*|\$?[A-Z]{{1,3}}:\$?[A-Z]{{1,3}}|\$?\d+:\$?\d+)"
)


def _bounds(area: str) -> tuple[int, int, int, int]:
    """(c0, r0, c1, r1) bounding box of an area token; whole rows/columns span the sheet."""
    body = re.sub(r"(?:\w+|S)!", "", area).replace("$", "")
    if re.fullmatch(r"[A-Z]+:[A-Z]+", body):
        a, b = sorted(parse_a1(f"{x}1")[1] for x in body.split(":"))
        return a, 1, b, 1_048_576
    if re.fullmatch(r"\d+:\d+", body):
        a, b = sorted(int(x) for x in body.split(":"))
        return 0, a, 16_383, b
    corners = [parse_a1(x) for x in body.split(":")]
    cols, rows = [c for _, c in corners], [r + 1 for r, _ in corners]
    return min(cols), min(rows), max(cols), max(rows)


def _cells_covered(operands: list[str]) -> int:
    """Cells in the intersection of space-separated area operands (sheets ignored: an upper bound)."""
    c0, r0, c1, r1 = _bounds(operands[0])
    for other in operands[1:]:
        d0, s0, d1, s1 = _bounds(other)
        c0, r0, c1, r1 = max(c0, d0), max(r0, s0), min(c1, d1), min(r1, s1)
    return max(0, c1 - c0 + 1) * max(0, r1 - r0 + 1)


def test_operators_refs_is_deterministic() -> None:
    dump = json.dumps(generate_operators_refs(3), sort_keys=True)
    assert dump == json.dumps(generate_operators_refs(3), sort_keys=True)
    assert dump != json.dumps(generate_operators_refs(4), sort_keys=True)


def test_operators_refs_has_no_function_names() -> None:
    for _, _, f in _operators_refs_formulas():
        assert not re.search(r"[A-Za-z_][A-Za-z0-9_.]*\(", _strip_quoted(f)), f


def test_operators_refs_covers_every_category_with_distinct_probes() -> None:
    probes = _operators_refs_formulas()
    counts = Counter(tag.split("/")[0] for tag, _, _ in probes)
    forms = Counter(tag.split("/")[0] for tag in {t for t, _, _ in probes})
    assert set(counts) == set(CATEGORIES)
    assert min(counts.values()) >= 50
    assert all(forms[c] >= 3 for c in CATEGORIES if c != "mixed"), forms  # mixed is one random-tree form
    # M1 exit: gen-m1 has >= 3,000 oracle cells; a formula repeated in a case is the same probe.
    distinct = sum(len(set(c["sheets"][-1]["cells"].values())) for c in generate_operators_refs(0))
    assert distinct >= 3000
    for c in generate_operators_refs(0):
        assert set(c["hypotheses"]) == set(c["check"])
        assert c["expected"] == {}
        assert build_xlsx(c)[:2] == b"PK"


def test_operators_refs_whole_rows_and_columns_only_in_intersections() -> None:
    seen = 0
    for _, _, f in _operators_refs_formulas():
        bare = _strip_quoted(f)
        for m in _AREA.finditer(bare):
            if not re.search(r"!\$?(?:[A-Z]+:\$?[A-Z]+|\d+:\$?\d+)$", m[0]):
                continue
            seen += 1
            # Directly joined to another area by the space operator, on one side or the other.
            before, after = bare[: m.start()], bare[m.end() :]
            assert re.match(r" (?:\w+|S)!", after) or re.search(r"(?:^|[ (=])\S+ $", before), f
    assert seen > 0


def test_operators_refs_never_spill() -> None:
    """Every range expression covers at most one cell, so no probe spills into the next."""
    checked = 0
    for tag, _, f in _operators_refs_formulas():
        if tag.startswith("union/"):
            continue  # unions are multi-area by design: Excel returns one value or an error
        bare = _strip_quoted(f)
        # Maximal runs of areas joined by the space (intersection) operator.
        for run in re.finditer(rf"{_AREA.pattern}(?: {_AREA.pattern})*", bare):
            operands = _AREA.findall(run[0])
            assert _cells_covered(operands) <= 1, f
            checked += 1
    assert checked > 1000


def test_operators_refs_is_bounded_and_locale_free() -> None:
    for tag, cell, f in _operators_refs_formulas():
        depth = max_depth = 0
        for ch in _strip_quoted(f):
            depth += {"(": 1, ")": -1}.get(ch, 0)
            max_depth = max(max_depth, depth)
        assert max_depth <= 6, f
        for text in re.findall(r'"((?:[^"]|"")*)"', f):
            assert not re.search(r"[/$,:]", text), f  # no date, currency or grouped-number text
        if tag.startswith("chain/"):
            row = int(cell[1:])
            assert all(int(n) < row for n in re.findall(r"(?<![!\w$])\$?A\$?(\d+)", _strip_quoted(f))), f


def test_names_3d_is_deterministic_and_distinct_per_sheet() -> None:
    dump = json.dumps(generate_names_3d(3), sort_keys=True)
    assert dump == json.dumps(generate_names_3d(3), sort_keys=True)
    assert dump != json.dumps(generate_names_3d(4), sort_keys=True)
    cases = generate_names_3d(0)
    assert {c["tags"][-1] for c in cases} == set(NAMES_3D_CATEGORIES)
    total = 0
    for c in cases:
        assert set(c["hypotheses"]) == set(c["check"])
        assert c["expected"] == {}
        assert [s["name"] for s in c["sheets"]][:7] == SPAN_SHEETS
        for sheet in c["sheets"]:
            probes = [v for a1, v in sheet["cells"].items() if f"{sheet['name']}!{a1}" in c["hypotheses"]]
            assert len(probes) == len(set(probes)), sheet["name"]  # a repeat is the same probe again
            total += len(probes)
        assert build_xlsx(c)[:2] == b"PK"
    # T-1606: gen-m8a has >= 3,000 oracle cells.
    assert total >= 3000


def test_names_3d_spans_and_named_ranges_never_cover_a_probe() -> None:
    """3-D spans include `Calc`, so a span reaching column K would make probes circular."""
    for c in generate_names_3d(0):
        texts = [
            *c["names"].values(),
            *(c["sheets"][4]["cells"][a1.split("!")[1]] for a1 in c["check"] if a1.startswith("Calc!")),
        ]
        for text in texts:
            for col in re.findall(r"![$]?([A-Z]+)[$]?\d", _strip_quoted(str(text))):
                assert len(col) == 1 and col <= "F", text


def test_tables_is_deterministic_and_covers_its_shapes() -> None:
    dump = json.dumps(generate_tables(3), sort_keys=True)
    assert dump == json.dumps(generate_tables(3), sort_keys=True)
    assert dump != json.dumps(generate_tables(4), sort_keys=True)
    cases = generate_tables(0)
    assert {c["tags"][-1] for c in cases} == set(TABLES_CATEGORIES)
    total, widths, rows, totals = 0, set(), set(), set()
    for c in cases:
        assert set(c["hypotheses"]) == set(c["check"])
        assert c["expected"] == {}
        cells = {f"{s['name']}!{a1}": v for s in c["sheets"] for a1, v in s["cells"].items()}
        assert all(str(cells[a1]).startswith("=") for a1 in c["check"])
        total += len(c["check"])
        for t in c["sheets"][0]["tables"]:
            (r1, c1), (r2, c2) = (parse_a1(x) for x in t["ref"].split(":"))  # 0-based
            headers = [cells[f"Data!{chr(65 + c1 + i)}{r1 + 1}"] for i in range(c2 - c1 + 1)]
            data = [h for h in headers if h not in FORMULA_HEADERS]
            widths.add(len(data))
            totals.add(bool(t.get("totals_row")))
            body = range(r1 + 2, r2 + 2 - int(bool(t.get("totals_row"))))
            filled = any(f"Data!{chr(65 + c1 + i)}{r}" in cells for i in range(len(data)) for r in body)
            rows.add(len(body) if filled else 0)
        assert build_xlsx(c)[:2] == b"PK"
    assert widths == {1, 2, 3, 4, 5}
    assert totals == {True, False}
    assert 0 in rows and max(rows) == 20
    # T-1607: gen-m8b has >= 3,000 oracle cells.
    assert total >= 3000


def test_tables_probes_never_read_their_own_formula_column() -> None:
    """`in-table` probes fill the formula columns, so reading one of them would be circular."""
    case = next(c for c in generate_tables(0) if c["tags"][-1] == "in-table")
    for a1 in case["check"]:
        assert "Calc" not in case["sheets"][0]["cells"][a1.split("!")[1]]


@pytest.mark.parametrize(
    ("header", "escaped", "at"),
    [
        ("Qty", "Qty", "@Qty"),
        ("C#", "C'#", "@[C'#]"),
        ("it's", "it''s", "@[it''s]"),
        ("[x]", "'[x']", "@['[x']]"),
        ("x@y", "x'@y", "@[x'@y]"),
        ("Unit Price", "Unit Price", "@[Unit Price]"),
    ],
)
def test_table_columns_are_escaped(header: str, escaped: str, at: str) -> None:
    assert escape_column(header) == escaped
    assert this_row(escaped) == at


def _outer_function(formula: str) -> str:
    return formula.removeprefix("=").split("(", 1)[0]


def test_unfocused_output_is_unchanged() -> None:
    # Digest recorded before --focus (T-2108): the flag must not move any unfocused draw. The catalog has its
    # `pool:` keys (T-2109) removed, which do draw from the rng. Re-recorded for T-2251 (INFO and SHEETS gained
    # signatures), T-2318 (TEXTJOIN's delimiter is an array parameter) and T-2719 (*IFS ranges share a shape,
    # MATCH's lookup_array is a vector) and T-2866 (number and bool parameters draw `""` and errors) and T-2717
    # (the criteria pool grew, so criteria draws moved).
    catalog = copy.deepcopy(_loaded_catalog())
    for f in catalog.functions:
        for p in f.get("params", []):
            p.pop("pool", None)
    body = json.dumps(generate_case(catalog, 7, milestone="M5"), indent=1, sort_keys=True) + "\n"
    assert (
        hashlib.sha256(body.encode()).hexdigest()
        == "25b67c6d3b3e9440ef405deb2ae9f585f5994595174a974b74b617d10668edc3"
    )


def test_focus_cycles_target_milestone_functions_evenly() -> None:
    catalog = _loaded_catalog()
    targets = {
        f["name"]: f.get("spelled", f["name"])
        for f in catalog.scoped("M5").functions
        if f.get("milestone") == "M5"
    }
    assert len(targets) == 133
    spelled = set(targets.values())
    outer: Counter[str] = Counter()
    for i in range(50):
        spec = generate_case(
            catalog, 2 * 1_000_003 + i, milestone="M5", formulas=400, focus=True, case_index=i
        )
        outer.update(_outer_function(f) for f in spec["sheets"][1]["cells"].values())
    assert set(outer) <= spelled
    assert min(outer[name] for name in spelled) >= 100


def test_focus_without_target_functions_is_an_error() -> None:
    with pytest.raises(ValueError, match="no generator-ready functions"):
        generate_case(_loaded_catalog(), 1, milestone="M0", focus=True)


def _focus_outer_counts(**kw: Any) -> Counter[str]:
    outer: Counter[str] = Counter()
    # Exactly even within one case: the cycle runs through every target before repeating.
    spec = generate_case(_loaded_catalog(), 3 * 1_000_003, milestone="M5", formulas=400, focus=True, **kw)
    outer.update(_outer_function(f) for f in spec["sheets"][1]["cells"].values())
    return outer


def test_focus_functions_narrows_outer_calls_evenly() -> None:
    names = {
        f["name"]: f.get("spelled", f["name"])
        for f in _loaded_catalog().functions
        if f.get("milestone") == "M5"
    }
    picked = sorted(names)[:3]
    outer = _focus_outer_counts(functions=picked)
    assert set(outer) == {names[n] for n in picked}
    assert max(outer.values()) - min(outer.values()) <= 1


def test_focus_category_narrows_outer_calls_to_that_category() -> None:
    cat = next(f["category"] for f in _loaded_catalog().functions if f.get("milestone") == "M5")
    members = {
        f.get("spelled", f["name"])
        for f in _loaded_catalog().scoped("M5").functions
        if f.get("milestone") == "M5" and f.get("category") == cat
    }
    outer = _focus_outer_counts(category=cat)
    assert set(outer) == members
    assert max(outer.values()) - min(outer.values()) <= 1


@pytest.mark.parametrize(
    ("kw", "match"),
    [
        ({"functions": ["NOSUCHFN"]}, "unknown function"),
        ({"category": "nosuchcat"}, "unknown category"),
        ({"functions": ["XLOOKUP"]}, "outside milestone"),
    ],
)
def test_focus_narrowing_errors(kw: dict[str, Any], match: str) -> None:
    with pytest.raises(ValueError, match=match):
        generate_case(_loaded_catalog(), 1, milestone="M5", focus=True, **kw)


@pytest.mark.parametrize("kw", [{"functions": ["MROUND"]}, {"category": "math"}])
def test_narrowing_without_focus_is_an_error(kw: dict[str, Any]) -> None:
    with pytest.raises(ValueError, match="pass --focus"):
        generate_case(_loaded_catalog(), 1, milestone="M5", **kw)


def test_focus_functions_and_category_with_empty_intersection_is_an_error() -> None:
    m5 = [f for f in _loaded_catalog().functions if f.get("milestone") == "M5"]
    first = m5[0]
    other = next(f["category"] for f in m5 if f["category"] != first["category"])
    with pytest.raises(ValueError, match="no generator-ready"):
        generate_case(
            _loaded_catalog(), 1, milestone="M5", focus=True, functions=[first["name"]], category=other
        )


def test_every_catalog_pool_exists_and_stays_under_the_literal_limit() -> None:
    named = {
        p["pool"]
        for f in _loaded_catalog().functions
        for p in f.get("params", [])
        if isinstance(p, dict) and "pool" in p
    }
    assert named
    assert named <= set(POOLS)
    for entries in POOLS.values():
        for text in entries:
            for literal in re.findall(r"\d+(?:\.\d+)?(?:E[+-]?\d+)?", text):
                assert float(literal) < 1e308, text


def test_pool_param_draws_from_its_pool_about_35_percent_of_the_time() -> None:
    g = Gen(random.Random(5), Catalog([], []), 12, 5, 3)
    pooled = {"name": "x", "mode": "value", "type": "number", "pool": "angle"}
    draws = [g.argument_text(pooled, "value", 0) for _ in range(2000)]
    share = sum(d in POOLS["angle"] for d in draws) / len(draws)
    assert 0.25 <= share <= 0.45
    hard = {**pooled, "domain": {"int": True, "min": 1, "max": 9, "hard": True}}
    assert not any(g.argument_text(hard, "value", 0) in POOLS["angle"] for _ in range(500))


def test_pooled_range_param_draws_array_constants_about_35_percent_of_the_time() -> None:
    # T-2286: LOOKUP's lookup_vector and XMATCH's lookup_array take array constants from `lookup_table`.
    g = Gen(random.Random(3), Catalog([], []), 12, 5, 3)
    pooled = {"name": "lookup_array", "mode": "range", "type": "any", "vector": True, "pool": "lookup_table"}
    draws = [g.argument(pooled, 0) for _ in range(2000)]
    share = sum(d in POOLS["lookup_table"] for d in draws) / len(draws)
    assert 0.25 <= share <= 0.45
    assert all(d in POOLS["lookup_table"] or re.fullmatch(r"Data!([A-Z])\d+:\1\d+", d) for d in draws)


def test_pooled_lambdas_have_the_arity_the_call_needs_and_free_names() -> None:
    # T-2286: MAP's LAMBDA takes one parameter per array, REDUCE's and SCAN's two; an outer name is never reused.
    assert {lambda_arity(t) for t in POOLS["lambda_body"]} == {1, 2}
    for arity in (1, 2, 3):
        g = Gen(random.Random(arity), Catalog([], []), 12, 5, 3)
        g.scope = ["x"]
        p = {"name": "function", "mode": "lambda", "arity": arity, "pool": "lambda_body"}
        draws = [g.pooled_lambda(p, arity) for _ in range(400)]
        pooled = [d for d in draws if d is not None]
        if arity == 3:
            assert not pooled
            continue
        assert 0.25 <= len(pooled) / len(draws) <= 0.45
        for text in pooled:
            assert lambda_arity(text) == arity, text
            params = re.match(r"LAMBDA\(((?:\[?[a-z_0-9]+\]?,)+)", text)
            assert params and "x" not in params.group(1).strip(",[]").replace("[", "").split(","), text
        assert g.scope == ["x"]


def test_pooled_catalog_params_keep_their_domain() -> None:
    # T-2286: a pool never overrides a hard domain, and a soft enum (the REGEX patterns) still draws in-domain
    # values most of the time.
    pooled = [
        p
        for f in _loaded_catalog().functions
        for p in f.get("params", [])
        if isinstance(p, dict) and "pool" in p
    ]
    assert not [p for p in pooled if (p.get("domain") or {}).get("hard")]
    pattern = next(
        p
        for f in _loaded_catalog().functions
        if f["name"] == "REGEXTEST"
        for p in f["params"]
        if p["name"] == "pattern"
    )
    g = Gen(random.Random(11), Catalog([], []), 12, 5, 3)
    draws = [g.argument_text(pattern, "value", 0) for _ in range(2000)]
    enum = {'"' + v + '"' for v in pattern["domain"]["enum"]}
    assert sum(d in enum for d in draws) / len(draws) >= 0.6
    assert any(d in POOLS["regex_pattern"] and d not in enum for d in draws)


def test_pooled_financial_params_keep_their_soft_domain() -> None:
    # T-2248: a financial param with an int or enum domain (a bond's settlement serials, frequency 1, 2 or 4) still
    # draws in-domain most of the time, and its pool adds the edges outside it.
    g = Gen(random.Random(13), Catalog([], []), 12, 5, 3)
    checked = 0
    for f in _loaded_catalog().functions:
        if f.get("category") != "financial":
            continue
        for p in f.get("params", []):
            dom = p.get("domain") or {}
            if "pool" not in p or not (dom.get("int") or dom.get("enum")):
                continue
            if dom.get("int"):
                inside = {str(v) for v in range(dom.get("min", 0), dom.get("max", 10) + 1)}
            else:
                inside = {str(v) for v in dom["enum"]}
            draws = [g.argument_text(p, "value", 0) for _ in range(400)]
            assert sum(d in inside for d in draws) / len(draws) >= 0.6, (f["name"], p["name"])
            assert any(d in POOLS[p["pool"]] and d not in inside for d in draws), (f["name"], p["name"])
            checked += 1
    assert checked >= 100


def test_text_pools_respect_the_string_literal_limit_and_clock_free_values() -> None:
    for name, pool in POOLS.items():
        for text in pool:
            for literal in re.findall(r'"((?:[^"]|"")*)"', text):
                assert len(literal.replace('""', '"')) <= MAX_STRING_LITERAL, (name, text)
    # A year-less date such as "1/2" reads as a date in the current year, so its value depends on the clock.
    for name in ("numeric_text", "date_text", "time_text"):
        assert not any(re.fullmatch(r'"\d{1,2}[/-]\d{1,2}"', t) for t in POOLS[name]), name


def test_number_formats_pool_is_a_quoted_subset_of_the_policy_codes() -> None:
    assert quoted('0.00 "x"') == '"0.00 ""x"""'
    assert quoted("") == '""'
    codes = {quoted(c) for c in (*BUILTIN, *CUSTOM)}
    assert POOLS["number_formats"]
    assert set(POOLS["number_formats"]) <= codes


def test_spill_keeps_the_formulas_and_gives_each_spilling_one_its_footprint() -> None:
    dynamic = catalog().dynamic_array_names()
    for seed in range(12):
        plain = generate_case(catalog(), seed, milestone="M7", focus=True, formulas=20, case_index=seed)
        spill = generate_case(
            catalog(), seed, milestone="M7", focus=True, formulas=20, case_index=seed, spill=True
        )
        assert spill == generate_case(
            catalog(), seed, milestone="M7", focus=True, formulas=20, case_index=seed, spill=True
        )
        cells = spill["sheets"][1]["cells"]
        formulas = [v for v in cells.values() if v != SPILL_BLOCKER]
        assert formulas == list(plain["sheets"][1]["cells"].values())
        check = set(spill["check"])
        assert len(check) == len(spill["check"])
        claimed: dict[str, str] = {}
        for a1, text in cells.items():
            if text == SPILL_BLOCKER:
                continue
            m = OUTER_CALL.match(text)
            row = int(a1[1:])
            if not (m and m.group(1) in dynamic):
                block = {a1}
            else:
                rows, cols = min(SPILL_ROWS, 12), min(SPILL_COLS, 5)  # generate_case's data sheet: 12 x 5
                block = {f"{col_letter(c)}{r}" for r in range(row, row + rows) for c in range(cols)}
            for cell in block:
                assert cell not in claimed, (cell, a1, claimed.get(cell))
                claimed[cell] = a1
                # every footprint cell is checked, except a blocker (an input)
                assert f"Calc!{cell}" in check or cells.get(cell) == SPILL_BLOCKER
        assert check == {f"Calc!{c}" for c in claimed if cells.get(c) != SPILL_BLOCKER}


def test_spill_layout_places_blockers_just_below_or_right_of_the_anchor() -> None:
    texts = ["=SEQUENCE(3)", "=SUM(1)"] * 400
    layout = spill_layout(texts, {"SEQUENCE"}, rows=4, cols=3, rng=random.Random(1))
    blockers = [a1 for a1, v in layout.cells.items() if v == SPILL_BLOCKER]
    assert len(blockers) == layout.blocked
    # SPILL_BLOCKED_RATE = 5% of 400 spilling probes, give or take
    assert 8 <= layout.blocked <= 35
    for a1 in blockers:
        col, row = a1[0], int(a1[1:])
        anchor = f"A{row - 1}" if col == "A" else f"A{row}"
        assert col in "AB"
        assert layout.cells[anchor] == "=SEQUENCE(3)"
        assert f"Calc!{a1}" not in layout.check
    # each SEQUENCE takes 4 rows, each SUM one
    assert layout.cells["A5"] == "=SUM(1)"
    assert layout.cells["A6"] == "=SEQUENCE(3)"
    assert len(layout.check) == 400 * (4 * 3 + 1) - layout.blocked


def test_spill_is_off_by_default() -> None:
    spec = generate_case(catalog(), 3, milestone="M7", focus=True, formulas=10)
    assert spec["check"] == [f"Calc!A{i}" for i in range(1, 11)]
    assert "spill" not in spec["tags"]


def test_m9_statistical_pools_keep_soft_domains_and_reach_their_edges() -> None:
    # T-2247: a pooled df keeps drawing in-domain integers most of the time, and the pool adds its edges
    # (the 1E10 df cut, values below 1 that truncate) the rest; every M9 statistical pool is used.
    catalog = _loaded_catalog()
    by_name = {f["name"]: f for f in catalog.functions}
    df = next(p for p in by_name["T.DIST"]["params"] if p["name"] == "deg_freedom")
    assert df["pool"] == "shape" and df.get("domain", {}).get("int")
    g = Gen(random.Random(9), Catalog([], []), 12, 5, 3)
    draws = [g.argument_text(df, "value", 0) for _ in range(2000)]
    dom = df["domain"]
    in_domain = [d for d in draws if d.isdigit() and dom.get("min", 0) <= int(d) <= dom.get("max", 10)]
    assert len(in_domain) / len(draws) >= 0.6
    assert any(d in POOLS["shape"] and d not in in_domain for d in draws)
    used = {
        p["pool"]
        for f in catalog.functions
        if f.get("milestone") == "M9" and f.get("category") == "statistical"
        for p in f.get("params", [])
        if "pool" in p
    }
    assert {"probability", "shape", "stat_x", "gamma_x", "flag", "stat_data"} <= used


def test_m9_engineering_and_database_pools_keep_soft_domains_and_reach_their_edges() -> None:
    # T-2249: a pooled engineering or database param with an int or enum domain (Bessel orders 0-5, the IM*
    # inumbers, the D* field 1-4) still draws in-domain most of the time, and its pool adds the edges outside it.
    g = Gen(random.Random(17), Catalog([], []), 12, 5, 3)
    checked, used = 0, set()
    for f in _loaded_catalog().functions:
        if f.get("milestone") != "M9" or f.get("category") not in ("engineering", "database"):
            continue
        for p in f.get("params", []):
            used.add(p.get("pool"))
            dom = p.get("domain") or {}
            if "pool" not in p or not (dom.get("int") or dom.get("enum")):
                continue
            if dom.get("int"):
                inside = {str(v) for v in range(dom.get("min", 0), dom.get("max", 10) + 1)}
            else:
                inside = {str(v) if isinstance(v, int) else quoted(str(v)) for v in dom["enum"]}
            draws = [g.argument_text(p, "value", 0) for _ in range(400)]
            assert sum(d in inside for d in draws) / len(draws) >= 0.6, (f["name"], p["name"])
            assert any(d in POOLS[p["pool"]] and d not in inside for d in draws), (f["name"], p["name"])
            checked += 1
    assert checked >= 60
    assert {
        "complex_text",
        "complex_suffix",
        "bin_digits",
        "oct_digits",
        "hex_digits",
        "dec_number",
        "places",
        "bessel_x",
        "bessel_order",
        "convert_unit",
        "erf_x",
        "bit_number",
        "bit_shift",
        "db_table",
        "db_field",
        "db_criteria",
    } <= used


def test_database_calls_get_the_db_sheet_and_mostly_query_it() -> None:
    # T-2249: the D* database and criteria pools point into sheet Db (gen/database.py) at pool_rate 0.8, so most
    # calls pair a Db table with Db criteria, and a case gets the sheet only when a formula refers to it.
    catalog = _loaded_catalog()
    spec = generate_case(catalog, 11, milestone="M9", formulas=200, focus=True, category="database")
    sheets = {s["name"]: s["cells"] for s in spec["sheets"]}
    assert sheets["Db"] == database.cells()
    calls = list(sheets["Calc"].values())
    paired = [
        c
        for c in calls
        if re.match(r"=D[A-Z]+\(Db!\$?A\$?\d+:\$?[A-D]\$?\d+,.*,Db!\$?[F-S]\$?\d+(:\$?[F-S]\$?\d+)?\)$", c)
    ]
    assert 0.5 <= len(paired) / len(calls) <= 0.8
    assert all(re.fullmatch(r"Db![$A-Z0-9:]+", e) for e in POOLS["db_table"] + POOLS["db_criteria"])
    assert "Db" not in {s["name"] for s in generate_case(catalog, 7, milestone="M5")["sheets"]}


def test_db_sheet_keeps_criteria_text_as_text() -> None:
    # A spec reads text starting with "=" as a formula: "=apple" is typed text, the computed criterion a formula.
    cells = database.cells()
    assert cells["I2"] == {"t": "s", "v": "=apple"}
    assert cells["O2"] == "=B2>3"
    assert cells["F2"] == ">2"
    assert "K2" not in cells and "A6" not in cells


def test_m9_count_and_ets_pools_reach_their_edges() -> None:
    # T-2438: integer counts keep their small-integer domain most of the time and add the edges (a truncating
    # fraction, 2^31, 1E15, text); the FORECAST.ETS options and arrays draw from pools, and the timeline and
    # values pools are long and equal in count so a valid call reaches the algorithm.
    catalog = _loaded_catalog()
    by_name = {f["name"]: f for f in catalog.functions}
    trials = next(p for p in by_name["BINOM.INV"]["params"] if p["name"] == "trials")
    assert trials["pool"] == "count" and trials.get("domain", {}).get("int")
    g = Gen(random.Random(11), Catalog([], []), 12, 5, 3)
    draws = [g.argument_text(trials, "value", 0) for _ in range(2000)]
    assert sum(d.isdigit() and int(d) <= 10 for d in draws) / len(draws) >= 0.6
    assert any(d in POOLS["count"] and not d.isdigit() for d in draws)
    ets = {p["name"]: p["pool"] for p in by_name["FORECAST.ETS.CONFINT"]["params"] if "pool" in p}
    assert ets == {
        "target_date": "ets_target",
        "values": "ets_values",
        "timeline": "ets_timeline",
        "confidence_level": "ets_confidence",
        "seasonality": "ets_seasonality",
        "data_completion": "ets_completion",
        "aggregation": "ets_aggregation",
    }
    assert any(p.get("pool") == "ets_statistic" for p in by_name["FORECAST.ETS.STAT"]["params"])
    for pool in ("ets_timeline", "ets_values"):
        assert len(POOLS[pool][0].split(";")) == 24
    assert all(p.startswith("{") and p.endswith("}") for p in POOLS["ets_timeline"] + POOLS["ets_values"])
    used = {p["pool"] for f in catalog.functions for p in f.get("params", []) if "pool" in p}
    assert {"beta_bound", "fisherinv_y", "forecast_x", "lognorm_mean"} <= used


def test_ifs_ranges_share_a_shape_and_match_looks_in_one_dimension() -> None:
    # T-2719: SUMIFS/COUNTIFS ranges of one call agree in shape (bar a ~1 in 10 mismatch) and MATCH's
    # lookup_array is mostly a single row or column.
    def dims(ref: str) -> tuple[int, int]:
        (c0, r0), (c1, r1) = (re.findall(r"([A-Z])(\d+)", ref)[i] for i in (0, 1))
        return int(r1) - int(r0) + 1, ord(c1) - ord(c0) + 1

    calls: list[list[str]] = []
    matches: list[tuple[int, int]] = []
    for seed in range(40):
        spec = generate_case(catalog(), seed, milestone="M2", formulas=120)
        for formula in spec["sheets"][1]["cells"].values():
            for m in re.finditer(r"(?:SUMIFS|COUNTIFS)\(((?:[^()]|\([^()]*\))*)\)", formula):
                calls.append(re.findall(r"Data![A-Z]\d+:[A-Z]\d+", m.group(1)))
            matches += [dims(a) for a in re.findall(r"MATCH\([^,()]*,(Data![A-Z]\d+:[A-Z]\d+)", formula)]
    multi = [c for c in calls if len(c) > 1]
    assert len(multi) > 50 and matches
    agree = sum(len({dims(r) for r in c}) == 1 for c in multi)
    assert 0.8 <= agree / len(multi) < 1.0
    one_d = sum(1 in d for d in matches)
    assert 0.7 <= one_d / len(matches) < 1.0


def test_fixed_height_ranges_and_probes_stay_inside_their_range() -> None:
    # T-2719: `range_ref(height=)` is exact, and `cell_in` lands inside the probed range.
    gen = Gen(random.Random(1), catalog(), data_rows=12, data_cols=5, max_depth=2)
    for _ in range(50):
        ref = gen.range_ref(width=2, height=3)
        m = re.fullmatch(r"Data!([A-E])(\d+):([A-E])(\d+)", ref)
        assert m and int(m[4]) - int(m[2]) == 2 and ord(m[3]) - ord(m[1]) == 1
        cell = gen.cell_in("Data!B2:D4")
        assert re.fullmatch(r"Data![B-D][2-4]", cell), cell
    assert re.fullmatch(r"Data!\$?[A-E]\$?\d+", gen.cell_in("not a range"))


def test_number_and_bool_value_params_draw_empty_text_and_errors() -> None:
    """T-2866: a number or bool `value` param gets `""` and an error literal; a `hard` domain never does."""
    g = Gen(random.Random(7), Catalog([], []), 12, 5, 3)
    for typ in ("number", "bool"):
        drawn = {g.scalar_text({"type": typ}, 0) for _ in range(2000)}
        assert '""' in drawn, typ
        assert drawn & set(POOLS["errors"]), typ
    hard = {"type": "number", "domain": {"int": True, "min": 1, "max": 5, "hard": True}}
    assert all(g.scalar_text(hard, 0) in {"1", "2", "3", "4", "5"} for _ in range(2000))
