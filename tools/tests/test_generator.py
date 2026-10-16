from __future__ import annotations

import copy
import functools
import io
import json
import re
import zipfile
from collections import Counter
from pathlib import Path

import pytest

from rustsheet_tools import repo_root
from rustsheet_tools.gen.generator import NAME_POOL, Catalog, generate_case
from rustsheet_tools.gen.names_3d import CATEGORIES as NAMES_3D_CATEGORIES
from rustsheet_tools.gen.names_3d import SPAN_SHEETS, generate_names_3d
from rustsheet_tools.gen.numeric_rules import generate_numeric_rules
from rustsheet_tools.gen.operators_refs import CATEGORIES, generate_operators_refs
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


M2_ADDED = [
    "HLOOKUP", "INDEX", "MATCH", "SUMIFS", "COUNTIFS", "ROUNDUP", "ROUNDDOWN", "NOT", "LEFT",
    "RIGHT", "MID", "TRIM", "UPPER", "LOWER", "CONCAT", "YEAR", "MONTH", "DAY",
]  # fmt: skip
VALID_MODES = {"value", "aggregate", "aggregate_a", "range", "criteria", "array", "any", "lambda", "name"}


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
    """Every tag (function name) the generator emits over 40 seeds at `milestone`."""
    seen: set[str] = set()
    for seed in range(40):
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
    # T-1088: every in-scope M5 logical, information and web entry but INFO and SHEETS (`oracle: none`)
    # is generator-ready.
    m5 = {
        f["name"]
        for f in catalog().functions
        if "params" in f and f["milestone"] == "M5" and f["category"] in {"logical", "information", "web"}
    }
    assert len(m5) == 26
    seen = emitted("M5")
    assert not sorted(m5 - seen)


def test_generator_emits_every_m6_date_function() -> None:
    # T-1090: every in-scope M6 date and time entry is generator-ready.
    m6_date = {
        f["name"]
        for f in catalog().functions
        if "params" in f and f["milestone"] == "M6" and f["category"] == "date"
    }
    assert len(m6_date) == 20
    assert not sorted(m6_date - emitted("M6"))


def test_generator_emits_every_m6_lookup_function_but_getpivotdata() -> None:
    # T-1238: every in-scope M6 lookup entry is generator-ready but GETPIVOTDATA (no pivot table to read).
    m6_lookup = {
        f["name"]
        for f in catalog().functions
        if "params" in f and f["milestone"] == "M6" and f["category"] == "lookup"
    }
    assert len(m6_lookup) == 18
    assert "GETPIVOTDATA" not in m6_lookup
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
    assert not any(f.startswith('"') for f in freqs), freqs


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
    made = re.findall(r"MAKEARRAY\([^,()]*,[^,()]*,LAMBDA\(((?:[a-z_]+,)+)", " ".join(formulas))
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
    spec = generate_case(cat, 2, milestone="M5", formulas=400)
    formulas = list(spec["sheets"][1]["cells"].values())
    # Expressions never produce a bare range: only the array branch passes one to SUMPRODUCT.
    assert any(re.search(r"SUMPRODUCT\(Data!\w+:\w+[,)]", f) for f in formulas)


def test_generator_calls_only_entries_with_signatures() -> None:
    # T-0040 catalogs every Excel function, most without params: those must never be generated.
    cat = catalog().scoped("M10")
    assert cat.functions and all("params" in f for f in cat.functions)
    assert generate_case(catalog(), 3, milestone="M10", formulas=40)["sheets"][1]["cells"]


def test_paired_repeat_groups_stay_paired() -> None:
    cat = catalog().scoped("M2")
    countifs = next(f for f in cat.functions if f["name"] == "COUNTIFS")
    assert countifs["params"][0]["repeat"]["group"] == ["criteria_range", "criteria"]
    spec = generate_case(cat, 5, milestone="M2", formulas=300)
    # criteria are literals and ranges contain no commas, so a bare COUNTIFS has 2n-1 commas.
    bare = [f for f in spec["sheets"][1]["cells"].values() if re.fullmatch(r"=COUNTIFS\([^()]*\)", f)]
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
    [("Qty", "Qty", "@Qty"), ("C#", "C'#", "@[C'#]"), ("it's", "it''s", "@[it''s]"), ("[x]", "'[x']", "@['[x']]")]
    + [("x@y", "x'@y", "@[x'@y]"), ("Unit Price", "Unit Price", "@[Unit Price]")],
)
def test_table_columns_are_escaped(header: str, escaped: str, at: str) -> None:
    assert escape_column(header) == escaped
    assert this_row(escaped) == at
