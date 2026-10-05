from __future__ import annotations

import io
import json
import re
import zipfile
from collections import Counter
from pathlib import Path

from rustsheet_tools import repo_root
from rustsheet_tools.gen.generator import Catalog, generate_case
from rustsheet_tools.gen.numeric_rules import generate_numeric_rules
from rustsheet_tools.gen.operators_refs import CATEGORIES, generate_operators_refs
from rustsheet_tools.spec2xlsx import build_xlsx, parse_a1


def catalog() -> Catalog:
    return Catalog.load(repo_root() / "eval" / "catalog" / "functions.yaml")


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
VALID_MODES = {"value", "aggregate", "aggregate_a", "range", "criteria", "any"}


def test_catalog_schema() -> None:
    names = [f["name"] for f in catalog().functions]
    assert len(names) == len(set(names))
    for f in catalog().functions:
        assert f["returns"] in {"number", "text", "bool", "any"}, f["name"]
        assert isinstance(f["prior"], int | float) and f["milestone"].startswith("M"), f["name"]
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


def test_operators_refs_is_deterministic() -> None:
    dump = json.dumps(generate_operators_refs(3), sort_keys=True)
    assert dump == json.dumps(generate_operators_refs(3), sort_keys=True)
    assert dump != json.dumps(generate_operators_refs(4), sort_keys=True)


def test_operators_refs_has_no_function_names() -> None:
    for _, _, f in _operators_refs_formulas():
        assert not re.search(r"[A-Za-z_][A-Za-z0-9_.]*\(", _strip_quoted(f)), f


def test_operators_refs_covers_every_category() -> None:
    cases = generate_operators_refs(0)
    counts = Counter(tag.split("/")[0] for tag, _, _ in _operators_refs_formulas())
    assert set(counts) == set(CATEGORIES)
    assert min(counts.values()) >= 50
    assert sum(counts.values()) >= 3000  # M1 exit: gen-m1 has >= 3,000 oracle cells
    for c in cases:
        assert set(c["hypotheses"]) == set(c["check"])
        assert c["expected"] == {}
        assert build_xlsx(c)[:2] == b"PK"


def test_operators_refs_whole_rows_and_columns_only_in_intersections() -> None:
    whole = re.compile(r"!\$?(?:[A-Z]+:\$?[A-Z]+|\d+:\$?\d+)(?![\w$])")
    seen = 0
    for _, _, f in _operators_refs_formulas():
        bare = _strip_quoted(f)
        for m in whole.finditer(bare):
            seen += 1
            before, after = bare[: m.start()], bare[m.end() :]
            assert after.startswith(" ") or re.search(r" \S*$", before), f
    assert seen > 0


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
