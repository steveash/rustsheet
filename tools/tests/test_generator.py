from __future__ import annotations

from pathlib import Path

from rustsheet_tools import repo_root
from rustsheet_tools.gen.generator import Catalog, generate_case
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
