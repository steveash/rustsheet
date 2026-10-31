"""Math references for eval cells (SPEC-0029 "Math references for eval cells", T-3380).

`rustsheet-accuracy math --suite S` writes `<case>.math.json` (rustsheet-math/v1) next to each spec
case's oracle sidecar: for every scored cell (a key of the oracle sidecar or of the case's
`expected`) whose formula is one call `=FUNC(args)` of a function with a Gate A definition, the
50-digit math value of its arguments with its formula text `f`. The runner attaches a value to
the cell's result only while `f` still matches the cell's formula, and R1(c) counts a visibly-off
numeric cell inside Gate A's envelope through it.

A candidate cell gets no math value when:
- an argument reads a formula cell (its value is the engine's or Excel's, not a literal), unless that
  cell's whole formula is arithmetic over number literals (`=(83892212*2^-28+4861903*2^-54)`, an
  exact-input cell), which is read with Excel's operator model as the fit lab reads it;
- an argument is anything else the fit lab's sweep reader leaves unparsed (a nested call), text,
  empty or a range;
- an integer parameter is given a non-integer (the definitions take integers; whether Excel
  truncates is per function, so the cell is skipped rather than guessed), a logical parameter is
  given a number (likewise unevidenced per function), or the call has a
  different number of arguments than the definition (an omitted or extra optional argument);
- the generator would drop the point (dps 60 and 80 disagree, over the time limit, or a value beyond
  the double range).
xlsx cases are skipped: tools never parse xlsx values.
"""

from __future__ import annotations

import json
import re
from collections.abc import Iterable
from concurrent.futures import ProcessPoolExecutor
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from rustsheet_tools.accuracy import defs, literal
from rustsheet_tools.accuracy.model import REGISTRY, Definition, NoValueError
from rustsheet_tools.accuracy.tables import MATH_FORMAT, DroppedError, compute, math_sidecar_bytes
from rustsheet_tools.fitlab.sweep import UnparsedError, Workbook, arithmetic, parse_arg, split_args
from rustsheet_tools.snapshot import suite_cases

_ = defs  # importing the package registers the definitions

_CALL = re.compile(r"=\s*([A-Za-z][A-Za-z0-9._]*)\((.*)\)\s*", re.DOTALL)


@dataclass(frozen=True)
class LiteralBook(Workbook):
    """A case's cells without any engine or Excel value: a referenced formula cell is read only
    when its formula is arithmetic over number literals (see the module doc)."""

    def value(self, sheet: str, address: str) -> float | bool | str | None:
        """A constant, or the value of an arithmetic-over-literals formula; UnparsedError for any
        other formula cell."""
        raw = self.sheets.get(sheet, {}).get(address)
        if isinstance(raw, str) and raw.startswith("="):
            return arithmetic(raw[1:])
        return super().value(sheet, address)


@dataclass
class CaseMath:
    """One case's math cells and why the other candidate cells got none."""

    case: str
    cells: dict[str, dict[str, dict[str, str]]] = field(default_factory=dict)  # sheet → A1 → cell
    skipped: dict[str, int] = field(default_factory=dict)  # reason → count

    def skip(self, reason: str) -> None:
        """Counts one candidate cell left without a math value."""
        self.skipped[reason] = self.skipped.get(reason, 0) + 1

    @property
    def written(self) -> int:
        """The number of cells with a math value or a math domain error."""
        return sum(len(c) for c in self.cells.values())


def math_path(oracle_sidecar: Path) -> Path:
    """`foo.oracle.json` → `foo.math.json`, beside it. Keep in step with the runner's
    `accuracy::math::math_sidecar` (crates/rustsheet-eval/src/accuracy/math.rs), which reads it."""
    name = oracle_sidecar.name.removesuffix(".oracle.json")
    return oracle_sidecar.with_name(f"{name}.math.json")


def single_call(text: str) -> tuple[str, str] | None:
    """(function name upper-cased, argument text) when `text` is one call `=FUNC(args)` whose
    opening parenthesis closes at the end, else None."""
    m = _CALL.fullmatch(text)
    if m is None:
        return None
    inner = m.group(2)
    depth, quoted = 0, False
    for ch in inner:
        if ch == '"':
            quoted = not quoted
        elif quoted:
            continue
        elif ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
            if depth < 0:  # `=F(a)+G(b)`: the first call closes before the end
                return None
    return (m.group(1).upper(), inner) if depth == 0 and not quoted else None


def fit_args(definition: Definition, values: list[Any]) -> list[literal.Arg] | str:
    """`values` as the definition's arguments, or why they do not fit."""
    if len(values) != len(definition.params):
        return "argument count"
    out: list[literal.Arg] = []
    for param, value in zip(definition.params, values, strict=True):
        if isinstance(value, list) or value is None or isinstance(value, str):
            return "non-numeric argument"
        if param.kind == "bool":
            if not isinstance(value, bool):  # no evidence yet of how each function reads a number here
                return "number for a logical"
            out.append(value)
        elif isinstance(value, bool):
            return "logical for a number"
        elif param.kind == "int":
            if not float(value).is_integer():
                return "non-integer for an integer parameter"
            out.append(int(value))
        else:
            out.append(float(value))
    return out


def scored_cells(spec: dict[str, Any], oracle: dict[str, Any] | None) -> Iterable[tuple[str, str]]:
    """(sheet, A1) of every scored cell: the oracle sidecar's cells and the case's `expected`."""
    seen: set[tuple[str, str]] = set()
    for sheet, cells in ((oracle or {}).get("sheets") or {}).items():
        for a1 in cells:
            seen.add((sheet, a1.upper()))
    for key in spec.get("expected") or {}:
        sheet, sep, a1 = key.rpartition("!")
        if sep:
            seen.add((sheet, a1.upper()))
    return sorted(seen)


def case_math(case_id: str, case_path: Path, sidecar: Path) -> CaseMath:
    """The math cells of one spec case (see the module doc for what is skipped)."""
    spec = json.loads(case_path.read_text(encoding="utf-8"))
    oracle = json.loads(sidecar.read_text(encoding="utf-8")) if sidecar.exists() else None
    sheets = {s["name"]: s["cells"] for s in spec["sheets"]}
    book = LiteralBook(sheets=sheets, oracle={})
    out = CaseMath(case_id)
    for sheet, a1 in scored_cells(spec, oracle):
        text = sheets.get(sheet, {}).get(a1)
        if not isinstance(text, str) or (call := single_call(text)) is None:
            continue
        definition = REGISTRY.get(call[0])
        if definition is None:
            continue
        try:
            values = [parse_arg(a, sheet, book) for a in split_args(call[1])]
        except (UnparsedError, ValueError) as err:
            out.skip(f"unparsed ({type(err).__name__})")
            continue
        args = fit_args(definition, values)
        if isinstance(args, str):
            out.skip(args)
            continue
        try:
            value = compute(definition, args)
        except NoValueError:
            out.skip("no single value")
            continue
        except DroppedError:
            out.skip("not converged or too slow")
            continue
        out.cells.setdefault(sheet, {})[a1] = {**value.to_json(), "f": text}
    return out


def sidecar_bytes(case_path: Path, math: CaseMath) -> bytes:
    """The math sidecar for `math`, keyed to the case file's hash."""
    sheets = {s: dict(sorted(c.items())) for s, c in sorted(math.cells.items())}
    return math_sidecar_bytes(case_path.read_bytes(), sheets)


def _job(entry: dict[str, Any]) -> tuple[dict[str, Any], CaseMath]:
    return entry, case_math(entry["id"], Path(entry["path"]), Path(entry["sidecar"]))


def write_suite(suite: str, workers: int | None = None) -> list[CaseMath]:
    """Writes the math sidecar of every spec case of `suite` that has a math cell, and removes a
    math sidecar this tool wrote for a case that no longer has one. Deterministic: the same case
    files give the same bytes."""
    entries = [e for e in suite_cases(suite) if str(e["path"]).endswith(".case.json")]
    results: list[CaseMath] = []
    with ProcessPoolExecutor(max_workers=workers) as pool:
        for entry, math in pool.map(_job, entries):
            path = math_path(Path(entry["sidecar"]))
            if math.written:
                path.write_bytes(sidecar_bytes(Path(entry["path"]), math))
            elif path.exists() and _ours(path):
                path.unlink()
            results.append(math)
    return results


def _ours(path: Path) -> bool:
    """Whether `path` is a math sidecar this tool wrote (never delete anything else)."""
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return False
    source = data.get("source") if isinstance(data, dict) else None
    return (
        isinstance(source, dict)
        and data.get("format") == MATH_FORMAT
        and source.get("generator") == "rustsheet-accuracy"
    )
