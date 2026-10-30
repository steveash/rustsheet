#!/usr/bin/env python3
"""Builds the fuzz seed corpora from the repository's own test inputs (T-0021).

Formula targets get one file per formula shape in eval/suites (without the leading `=`), at most
MAX_FORMULA_SEEDS of them; `xlsx_import` gets every workbook under crates/rustsheet-xlsx/tests and eval/suites. Corpora are
generated, never committed, so they track the suites as they grow. Seeds are added to whatever a
corpus directory already holds, so the inputs libFuzzer found on earlier runs are kept.

Usage: fuzz/seed_corpus.py   (from anywhere; writes fuzz/corpus/<target>/ and fuzz/dict/)
"""

from __future__ import annotations

import hashlib
import json
import re
from pathlib import Path

FUZZ = Path(__file__).resolve().parent
ROOT = FUZZ.parent
FORMULA_TARGETS = ("formula_parse", "formula_eval")
# Workbooks above this size slow every fuzz iteration that mutates them without adding coverage.
MAX_SEED_BYTES = 1 << 20
# A function call in a seed formula; its name (with the paren) becomes a dictionary token.
FUNCTION_CALL = re.compile(r"(?<![A-Za-z0-9_.])([A-Z][A-Z0-9.]*\()")
# The suites hold over 800,000 distinct formulas, most of them one generated shape with other
# numbers (probes-numeric). libFuzzer reads every seed once before it fuzzes, at a few hundred
# formula_eval runs a second, so all of them outlasted the nightly job (T-3163). One seed per
# shape (numbers and text literals replaced), capped, keeps the first pass near a minute; the
# fuzzer mutates numbers itself, and the dictionary still lists every function name.
MAX_FORMULA_SEEDS = 20_000
NUMBER = re.compile(r"(?<![A-Za-z0-9_$.])\d+(?:\.\d*)?(?:[eE][+-]?\d+)?")
TEXT = re.compile(r'"(?:[^"]|"")*"')


def write_dictionary(formulas: set[str]) -> Path:
    """Writes dict/formula.dict: the committed syntax tokens plus every function name seen."""
    names = sorted({m for f in formulas for m in FUNCTION_CALL.findall(f)})
    out = FUZZ / "dict" / "formula.dict"
    out.parent.mkdir(exist_ok=True)
    tokens = (FUZZ / "formula.dict").read_text(encoding="utf-8")
    out.write_text(tokens + "".join(f'"{name}"\n' for name in names), encoding="utf-8")
    return out


def case_formulas(case: Path) -> set[str]:
    data = json.loads(case.read_text(encoding="utf-8"))
    found: set[str] = set()
    for sheet in data.get("sheets", []):
        for value in sheet.get("cells", {}).values():
            if isinstance(value, str) and value.startswith("=") and len(value) > 1:
                found.add(value[1:])
    for formula in data.get("names", {}).values():
        if isinstance(formula, str):
            found.add(formula.removeprefix("="))
    return found


def shape(formula: str) -> str:
    return NUMBER.sub("0", TEXT.sub('""', formula))


def formula_seeds(formulas: set[str]) -> list[str]:
    """One formula per shape (the first in sorted order), then the MAX_FORMULA_SEEDS whose
    sha256 sorts first: deterministic, and spread over the suites rather than the first ones."""
    by_shape: dict[str, str] = {}
    for formula in sorted(formulas):
        by_shape.setdefault(shape(formula), formula)
    ranked = sorted(by_shape.values(), key=lambda f: hashlib.sha256(f.encode("utf-8")).hexdigest())
    return ranked[:MAX_FORMULA_SEEDS]


def write_seed(directory: Path, data: bytes) -> None:
    (directory / hashlib.sha256(data).hexdigest()).write_bytes(data)


def main() -> int:
    corpus = FUZZ / "corpus"
    formulas: set[str] = set()
    for case in sorted((ROOT / "eval" / "suites").rglob("*.case.json")):
        formulas |= case_formulas(case)
    seeds = formula_seeds(formulas)
    for target in FORMULA_TARGETS:
        directory = corpus / target
        directory.mkdir(parents=True, exist_ok=True)
        for formula in seeds:
            write_seed(directory, formula.encode("utf-8"))
    dictionary = write_dictionary(formulas)
    directory = corpus / "xlsx_import"
    directory.mkdir(parents=True, exist_ok=True)
    workbooks = [
        *(ROOT / "crates" / "rustsheet-xlsx" / "tests").rglob("*.xlsx"),
        *(ROOT / "eval" / "suites").rglob("*.xlsx"),
    ]
    kept = 0
    for workbook in sorted(workbooks):
        if workbook.stat().st_size <= MAX_SEED_BYTES:
            write_seed(directory, workbook.read_bytes())
            kept += 1
    print(f"{len(seeds)} formula seeds of {len(formulas)} formulas, {kept} workbook seeds in {corpus}; dictionary {dictionary}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
