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
  the double range). Only these skips need mpmath, so the sidecar lists them under `skipped`
  (sheet → A1 → `f` and `why`) and `math --check` (T-3588) can tell, without mpmath, a cell the
  generator dropped from one nobody ran the generator for.
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
from rustsheet_tools.accuracy.model import REGISTRY, Definition, NoValueError, Param
from rustsheet_tools.accuracy.tables import MATH_FORMAT, DroppedError, compute, math_sidecar_bytes
from rustsheet_tools.fitlab.sweep import UnparsedError, Workbook, arithmetic, parse_arg, split_args
from rustsheet_tools.snapshot import suite_cases

_ = defs  # importing the package registers the definitions

NO_VALUE = "no single value"
NOT_CONVERGED = "not converged or too slow"
REGENERATE = "cd tools && uv run rustsheet-accuracy math --suite {suite}"

_CALL = re.compile(r"=\s*([A-Za-z][A-Za-z0-9._]*)\((.*)\)\s*", re.DOTALL)


@dataclass(frozen=True)
class LiteralBook(Workbook):
    """A case's cells without any engine or Excel value: a referenced formula cell is read only
    when its formula is arithmetic over number literals (see the module doc)."""

    # Each formula cell's arithmetic value or why it has none: probe cases refer to the same
    # exact-input cells from many calls, and parsing them once keeps `math --check` cheap (T-3588).
    formulas: dict[str, float | UnparsedError] = field(default_factory=dict, compare=False)

    def value(self, sheet: str, address: str) -> float | bool | str | None:
        """A constant, or the value of an arithmetic-over-literals formula; UnparsedError for any
        other formula cell."""
        raw = self.sheets.get(sheet, {}).get(address)
        if isinstance(raw, str) and raw.startswith("="):
            if raw not in self.formulas:
                try:
                    self.formulas[raw] = arithmetic(raw[1:])
                except UnparsedError as err:
                    self.formulas[raw] = err
            found = self.formulas[raw]
            if isinstance(found, UnparsedError):
                raise found
            return found
        return super().value(sheet, address)


@dataclass
class CaseMath:
    """One case's math cells and why the other candidate cells got none."""

    case: str
    cells: dict[str, dict[str, dict[str, str]]] = field(default_factory=dict)  # sheet → A1 → cell
    skipped: dict[str, int] = field(default_factory=dict)  # reason → count
    # The candidates the generator dropped: sheet → A1 → {"f", "why"}, written to the sidecar.
    dropped: dict[str, dict[str, dict[str, str]]] = field(default_factory=dict)

    def skip(self, reason: str) -> None:
        """Counts one candidate cell left without a math value."""
        self.skipped[reason] = self.skipped.get(reason, 0) + 1

    def drop(self, sheet: str, a1: str, text: str, reason: str) -> None:
        """Counts a candidate the generator dropped and lists it for the sidecar's `skipped`."""
        self.skip(reason)
        self.dropped.setdefault(sheet, {})[a1] = {"f": text, "why": reason}

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


def array_elements(value: Any) -> tuple[float, ...] | str:
    """A range or array argument's numbers in row-major order, or why it does not fit (a scalar, or
    an element that is not a number: the definitions take dense numeric arrays only)."""
    if not isinstance(value, list):
        return "scalar for an array"
    flat: list[Any] = []
    for row in value:
        flat.extend(row if isinstance(row, list) else [row])
    if not flat or any(isinstance(e, (bool, str)) or e is None for e in flat):
        return "non-numeric array element"
    return tuple(float(e) for e in flat)


def fit_arg(param: Param, value: Any) -> literal.Arg | str:
    """One argument as the parameter's kind, or why it does not fit."""
    if param.kind == "array":
        return array_elements(value)
    if isinstance(value, list) or value is None or isinstance(value, str):
        return "non-numeric argument"
    if param.kind == "bool":
        # no evidence yet of how each function reads a number here
        return value if isinstance(value, bool) else "number for a logical"
    if isinstance(value, bool):
        return "logical for a number"
    if param.kind == "int":
        return int(value) if float(value).is_integer() else "non-integer for an integer parameter"
    return float(value)


def fit_args(definition: Definition, values: list[Any]) -> list[literal.Arg] | str:
    """`values` as the definition's arguments, or why they do not fit."""
    if len(values) != len(definition.params):
        return "argument count"
    out: list[literal.Arg] = []
    for param, value in zip(definition.params, values, strict=True):
        arg = fit_arg(param, value)
        if isinstance(arg, str):
            return arg
        out.append(arg)
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


@dataclass(frozen=True)
class Candidate:
    """A scored one-call cell of a function with a definition: its arguments as the definition
    reads them, or why it gets no math value without running the generator."""

    sheet: str
    a1: str
    text: str
    definition: Definition
    args: list[literal.Arg] | str


def candidates(case_path: Path, sidecar: Path) -> list[Candidate]:
    """The candidate cells of one spec case, in sheet and A1 order (no mpmath runs here)."""
    spec = json.loads(case_path.read_text(encoding="utf-8"))
    oracle = json.loads(sidecar.read_text(encoding="utf-8")) if sidecar.exists() else None
    sheets = {s["name"]: s["cells"] for s in spec["sheets"]}
    book = LiteralBook(sheets=sheets, oracle={})
    out: list[Candidate] = []
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
            out.append(Candidate(sheet, a1, text, definition, f"unparsed ({type(err).__name__})"))
            continue
        out.append(Candidate(sheet, a1, text, definition, fit_args(definition, values)))
    return out


def case_math(case_id: str, case_path: Path, sidecar: Path) -> CaseMath:
    """The math cells of one spec case (see the module doc for what is skipped)."""
    out = CaseMath(case_id)
    for c in candidates(case_path, sidecar):
        if isinstance(c.args, str):
            out.skip(c.args)
            continue
        try:
            value = compute(c.definition, c.args)
        except NoValueError:
            out.drop(c.sheet, c.a1, c.text, NO_VALUE)
            continue
        except DroppedError:
            out.drop(c.sheet, c.a1, c.text, NOT_CONVERGED)
            continue
        out.cells.setdefault(c.sheet, {})[c.a1] = {**value.to_json(), "f": c.text}
    return out


def sidecar_bytes(case_path: Path, math: CaseMath) -> bytes:
    """The math sidecar for `math`, keyed to the case file's hash."""
    sheets = {s: dict(sorted(c.items())) for s, c in sorted(math.cells.items())}
    dropped = {s: dict(sorted(c.items())) for s, c in sorted(math.dropped.items())}
    return math_sidecar_bytes(case_path.read_bytes(), sheets, skipped=dropped)


def _job(entry: dict[str, Any]) -> tuple[dict[str, Any], CaseMath]:
    return entry, case_math(entry["id"], Path(entry["path"]), Path(entry["sidecar"]))


def write_suite(suite: str, workers: int | None = None) -> list[CaseMath]:
    """Writes the math sidecar of every spec case of `suite` that has a math cell, and removes a
    math sidecar this tool wrote for a case that no longer has one. Deterministic: the same case
    files give the same bytes."""
    entries = _spec_entries(suite)
    results: list[CaseMath] = []
    with ProcessPoolExecutor(max_workers=workers) as pool:
        for entry, math in pool.map(_job, entries):
            path = math_path(Path(entry["sidecar"]))
            if math.written or math.dropped:
                path.write_bytes(sidecar_bytes(Path(entry["path"]), math))
            elif path.exists() and _ours(path):
                path.unlink()
            results.append(math)
    return results


def _spec_entries(suite: str) -> list[dict[str, Any]]:
    """The harness's listing of `suite`'s spec cases (tools never parse xlsx values)."""
    return [e for e in suite_cases(suite) if str(e["path"]).endswith(".case.json")]


def check_case(case_id: str, case_path: Path, sidecar: Path) -> list[str]:
    """Why the case's math sidecar is not what `math` would write for its cells, without running
    mpmath: a candidate cell the sidecar neither holds nor lists as skipped by the generator, or a
    sidecar entry whose `f` is not the cell's formula (or whose cell is no candidate any more)."""
    path = math_path(sidecar)
    data: dict[str, Any] = {}
    if path.exists():
        try:
            loaded = json.loads(path.read_text(encoding="utf-8"))
        except ValueError:
            return [f"{case_id}: math sidecar {path.name} is not JSON"]
        data = loaded if isinstance(loaded, dict) else {}
    held: dict[tuple[str, str], str | None] = {}
    for key in ("sheets", "skipped"):
        for sheet, cells in (data.get(key) or {}).items():
            for a1, cell in cells.items():
                held[(sheet, a1)] = cell.get("f") if isinstance(cell, dict) else None
    problems: list[str] = []
    wanted = {(c.sheet, c.a1): c.text for c in candidates(case_path, sidecar) if not isinstance(c.args, str)}
    for (sheet, a1), text in sorted(wanted.items()):
        if (sheet, a1) not in held:
            problems.append(f"{case_id}: {sheet}!{a1} {text} has no math value and is not listed as skipped")
    for (sheet, a1), f in sorted(held.items()):
        if wanted.get((sheet, a1)) != f:
            problems.append(
                f"{case_id}: {sheet}!{a1} math sidecar entry {f} is stale (the cell is {wanted.get((sheet, a1))})"
            )
    return problems


def _check_job(entry: dict[str, Any]) -> list[str]:
    """`check_case` for one listing entry, in a worker process."""
    return check_case(entry["id"], Path(entry["path"]), Path(entry["sidecar"]))


def check_suite(suite: str) -> list[str]:
    """`check_case` over every spec case of `suite`; each problem names its case, and the last
    line the command that regenerates the sidecars."""
    problems: list[str] = []
    with ProcessPoolExecutor() as pool:  # parsing every argument is the cost: one case per process
        for found in pool.map(_check_job, _spec_entries(suite)):
            problems += found
    if problems:
        problems.append(f"regenerate: {REGENERATE.format(suite=suite)}")
    return problems


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
