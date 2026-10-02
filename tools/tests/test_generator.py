from __future__ import annotations

import re
from pathlib import Path

from rustsheet_tools import repo_root
from rustsheet_tools.gen.generator import Catalog, generate_case
from rustsheet_tools.gen.numeric_rules import generate_numeric_rules
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
