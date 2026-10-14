"""Sweep inputs: the cells of an oracle-filled `.case.json` that a fit scores.

A sweep is a case plus its oracle sidecar (Excel's values), both JSON, located through the
harness's own discovery (`rustsheet-eval cases`, `snapshot.suite_cases`); no xlsx is parsed.
Cells are selected by a regex over the formula text whose first group is the argument list.
Arguments are numeric literals, TRUE/FALSE, string literals, array literals of those (`{1,2;3,4}`,
row-major), `DATE(y,m,d)` with integer literals (a 1900-system serial), references (`A1`, `$Z$1`,
`Data!B2`, `D1:D10`) to constant cells of the case or to formula cells with an oracle value, and
arithmetic over numeric literals (`0.05/12`, `1+26955297717044*2^-52`), which is evaluated with
Excel's operator model (`arithmetic`). A cell whose arguments are anything else (another function
call) is reported as unparsed. A pattern with one group is an argument list split at top-level
commas; a pattern with several groups gives one argument per group that took part in the match.
"""

from __future__ import annotations

import datetime
import json
import math
import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from xlsxwriter.utility import xl_rowcol_to_cell

from rustsheet_tools.fitlab.exact import DOUBLE, DOUBLE_MIN_EXP, X87, exact, round_to_bits
from rustsheet_tools.snapshot import suite_cases
from rustsheet_tools.spec2xlsx import parse_a1

# What a candidate receives for one argument: a number, a bool, a string, an empty cell (None)
# or, for a range, the list of those values in row-major order.
Arg = float | bool | str | None | list[float | bool | str | None]


def numeric_arg(a: Arg) -> float:
    """`a` as a number for a kernel candidate; TypeError for anything else (a bool, text, an
    empty cell or a range), which the scorer counts as that candidate's miss on the cell."""
    if isinstance(a, bool) or not isinstance(a, float):
        msg = f"non-numeric argument {a!r}"
        raise TypeError(msg)
    return a


_UNSIGNED = r"(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?"
_NUMBER = re.compile(rf"[+-]?{_UNSIGNED}")
_REF = re.compile(
    r"(?:(?P<sheet>'(?:[^']|'')+'|[A-Za-z_][\w.]*)!)?(?P<first>\$?[A-Za-z]{1,3}\$?\d+)"
    r"(?:\s*:\s*(?P<last>\$?[A-Za-z]{1,3}\$?\d+))?"
)
_MAX_RANGE_CELLS = 1_000_000
_ARITH = re.compile(rf"\s*(?:(?P<num>{_UNSIGNED})|(?P<op>[-+*/^()]))")
_MAX_EXACT_EXPONENT = 1100  # beyond the double range either way
_MAX_NESTING = 50  # parentheses and unary signs: a bound on the parser's recursion (directive 5)
_DATE = re.compile(r"DATE\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*\)", re.IGNORECASE)
EPOCH = datetime.date(1899, 12, 30)  # serial 61 = 1900-03-01 (Excel's phantom 1900-02-29 is 60)


class UnparsedError(ValueError):
    """An argument the sweep reader does not evaluate (e.g. a nested expression)."""


@dataclass(frozen=True)
class Cell:
    """One scored cell: where it is, its formula, parsed arguments and Excel's value."""

    case: str
    sheet: str
    address: str
    formula: str
    args: tuple[Arg, ...]
    expected: float

    @property
    def key(self) -> str:
        """`Sheet!A1`, the key `rustsheet-eval show` prints."""
        return f"{self.sheet}!{self.address}"


@dataclass
class Sweep:
    """The selected cells of one or more cases, plus what was left out and why."""

    cells: list[Cell] = field(default_factory=list)
    unparsed: list[str] = field(default_factory=list)  # "case Sheet!A1: reason"
    non_numeric: int = 0  # Excel's value is an error, text or bool: nothing for a float to match
    excluded: int = 0  # matched the pattern but the fit's `where` filter left it out


def split_args(text: str) -> list[str]:
    """Splits an argument list at top-level commas (outside parentheses, braces and strings)."""
    parts: list[str] = []
    depth, start, quoted = 0, 0, False
    for i, ch in enumerate(text):
        if ch == '"':
            quoted = not quoted
        elif quoted:
            continue
        elif ch in "({":
            depth += 1
        elif ch in ")}":
            depth -= 1
        elif ch == "," and depth == 0:
            parts.append(text[start:i])
            start = i + 1
    parts.append(text[start:])
    return [p.strip() for p in parts] if text.strip() else []


@dataclass(frozen=True)
class Workbook:
    """A case's sheets: spec cell values and the oracle's values for formula cells."""

    sheets: dict[str, dict[str, Any]]
    oracle: dict[str, dict[str, Any]]

    def value(self, sheet: str, address: str) -> float | bool | str | None:
        """A referenced cell's value: a constant from the case, or Excel's value of a formula."""
        cells = self.sheets.get(sheet)
        if cells is None:
            msg = f"no sheet {sheet!r}"
            raise UnparsedError(msg)
        raw = cells.get(address)
        if isinstance(raw, str) and raw.startswith("="):
            return _oracle_value(self.oracle.get(sheet, {}).get(address), f"{sheet}!{address}")
        if isinstance(raw, dict):
            return _oracle_value(raw, f"{sheet}!{address}")
        if raw is None or isinstance(raw, bool | str):
            return raw
        if isinstance(raw, int | float):
            return float(raw)
        msg = f"unsupported cell value at {sheet}!{address}"
        raise UnparsedError(msg)


def _oracle_value(entry: Any, where: str) -> float | bool | str | None:
    """A `{"t": .., "v": ..}` value (oracle sidecar or typed case cell) as an argument."""
    if not isinstance(entry, dict):
        msg = f"{where} has no oracle value"
        raise UnparsedError(msg)
    tag, value = entry.get("t"), entry.get("v")
    if tag == "n":
        return float(entry["v"])
    if tag == "b":
        return bool(value)
    if tag == "s":
        return str(value)
    msg = f"{where} is an error ({value})"
    raise UnparsedError(msg)


def _sheet_name(raw: str | None, default: str) -> str:
    if raw is None:
        return default
    return raw[1:-1].replace("''", "'") if raw.startswith("'") else raw


def _tokens(text: str) -> list[str]:
    """`text` split into unsigned numbers and the operators `+ - * / ^ ( )`, whitespace dropped;
    UnparsedError at anything else (a function name, a reference, a comma)."""
    out: list[str] = []
    pos, end = 0, len(text.rstrip())
    while pos < end:
        m = _ARITH.match(text, pos)
        if not m:
            msg = f"cannot evaluate argument {text!r}"
            raise UnparsedError(msg)
        out.append(m.group("num") or m.group("op"))
        pos = m.end()
    return out


def _exact_power(base: float, power: float) -> float:
    """`base^power` when it is an integer power whose exact value is a double (`2^-52`). Other
    powers are themselves the subject of fits (numbers.md, "`^` / POWER are not correctly
    rounded"), so a sweep argument does not guess one."""
    if power != int(power) or abs(power) > _MAX_EXACT_EXPONENT or base == 0:
        msg = f"power {base}^{power} in an argument"
        raise UnparsedError(msg)
    value = exact(base) ** int(power)
    if round_to_bits(value, DOUBLE, DOUBLE_MIN_EXP) != value:
        msg = f"power {base}^{power} in an argument is not exact"
        raise UnparsedError(msg)
    try:
        return float(value)
    except OverflowError:
        msg = f"power {base}^{power} in an argument overflows"
        raise UnparsedError(msg) from None


class _Arithmetic:
    """A recursive-descent parser over `_tokens(text)`, evaluating as it goes (see `arithmetic`)."""

    def __init__(self, text: str) -> None:
        self.text = text
        self.tokens = _tokens(text)
        self.pos = 0

    def fail(self, what: str) -> UnparsedError:
        return UnparsedError(f"{what} in argument {self.text!r}")

    def peek(self) -> str | None:
        return self.tokens[self.pos] if self.pos < len(self.tokens) else None

    def take(self) -> str:
        if self.pos >= len(self.tokens):
            raise self.fail("unexpected end")
        self.pos += 1
        return self.tokens[self.pos - 1]

    def primary(self, depth: int) -> float:
        if depth > _MAX_NESTING:
            raise self.fail("nesting too deep")
        tok = self.take()
        if tok in ("+", "-"):
            value = self.primary(depth + 1)
            return -value if tok == "-" else value
        if tok == "(":
            value = self.additive(depth + 1)
            if self.take() != ")":
                raise self.fail("unbalanced parentheses")
            return value
        if not _NUMBER.fullmatch(tok):
            raise self.fail(f"unexpected {tok!r}")
        literal = float(tok)
        if not math.isfinite(literal):
            raise self.fail(f"non-finite literal {tok}")
        return literal

    def power(self, depth: int) -> float:
        value = self.primary(depth)
        while self.peek() == "^":
            self.take()
            value = _exact_power(value, self.primary(depth))  # Excel's ^ is left-associative
        return value

    def multiplicative(self, depth: int) -> float:
        value = self.power(depth)
        while (op := self.peek()) in ("*", "/"):
            self.take()
            rhs = self.power(depth)
            if op == "/" and rhs == 0:
                raise self.fail("division by zero")
            value = X87.mul(value, rhs) if op == "*" else X87.div(value, rhs)
        return value

    def additive(self, depth: int) -> float:
        value = self.multiplicative(depth)
        while (op := self.peek()) in ("+", "-"):
            self.take()
            rhs = self.multiplicative(depth)
            value = X87.add(value, rhs) if op == "+" else X87.sub(value, rhs)
        return value


def arithmetic(text: str) -> float:
    """Evaluates `+ - * / ^`, unary signs and parentheses over numeric literals with Excel's
    operator model: each of `+ - * /` rounds its exact result to 64 bits, then to a double
    (numbers.md, "Division rounds twice"), and `^` must be exact (`_exact_power`). As in Excel,
    a unary sign binds tighter than `^` (`-2^2` is 4). A non-finite literal, an overflow, or
    nesting deeper than 50 is UnparsedError."""
    parser = _Arithmetic(text)
    try:
        result = parser.additive(0)
    except OverflowError:  # an X87 operation's result is beyond the double range
        raise parser.fail("overflow") from None
    if parser.pos != len(parser.tokens):
        raise parser.fail("trailing input")
    return result


def _literal(text: str) -> float | bool | str | None:
    """A number, TRUE/FALSE or string literal, or None when `text` is none of those."""
    if _NUMBER.fullmatch(text):
        # Excel has no negative zero: `-0` is 0.
        return float(text) + 0.0
    if text.upper() in ("TRUE", "FALSE"):
        return text.upper() == "TRUE"
    if len(text) >= 2 and text[0] == text[-1] == '"' and '"' not in text[1:-1].replace('""', ""):
        return text[1:-1].replace('""', '"')
    return None


def _scalar(text: str) -> float | bool | str:
    """An array item: a literal, else UnparsedError."""
    literal = _literal(text)
    if literal is None:
        msg = f"cannot evaluate literal {text!r}"
        raise UnparsedError(msg)
    return literal


def _array(text: str) -> list[float | bool | str | None]:
    """`{a,b;c,d}`: the literals in row-major order."""
    return [_scalar(item) for item in split_args(text[1:-1].replace(";", ","))]


def date_serial(year: int, month: int, day: int) -> float:
    """DATE(y, m, d) for a real calendar date from 1900-03-01 on (the serial Excel stores)."""
    try:
        when = datetime.date(year, month, day)
    except ValueError:
        msg = f"DATE({year},{month},{day}) is not a calendar date"
        raise UnparsedError(msg) from None
    if when < datetime.date(1900, 3, 1):
        msg = f"DATE({year},{month},{day}) is before 1900-03-01"
        raise UnparsedError(msg)
    return float((when - EPOCH).days)


def serial_date(serial: float) -> datetime.date:
    """The calendar date of a 1900-system serial from 61 (1900-03-01) to 2958465 (9999-12-31);
    UnparsedError for anything else (a fraction is dropped, as Excel's date functions do)."""
    if not (math.isfinite(serial) and 61 <= serial < 2958466):
        msg = f"date serial {serial} is outside 1900-03-01..9999-12-31"
        raise UnparsedError(msg)
    return EPOCH + datetime.timedelta(days=int(serial))


def parse_arg(text: str, sheet: str, book: Workbook) -> Arg:
    """One argument's value (see the module doc for what is supported)."""
    if len(text) >= 2 and text[0] == "{" and text[-1] == "}":
        return _array(text)
    if d := _DATE.fullmatch(text):
        return date_serial(int(d.group(1)), int(d.group(2)), int(d.group(3)))
    if (literal := _literal(text)) is not None:
        return literal
    if m := _REF.fullmatch(text):
        name = _sheet_name(m.group("sheet"), sheet)
        (r1, c1), (r2, c2) = parse_a1(m.group("first")), parse_a1(m.group("last") or m.group("first"))
        rows, cols = range(min(r1, r2), max(r1, r2) + 1), range(min(c1, c2), max(c1, c2) + 1)
        if len(rows) * len(cols) > _MAX_RANGE_CELLS:
            msg = f"range {text} is too large"
            raise UnparsedError(msg)
        values = [book.value(name, xl_rowcol_to_cell(r, c)) for r in rows for c in cols]
        return values if m.group("last") else values[0]
    return arithmetic(text)


def case_entry(case_id: str) -> dict[str, Any]:
    """The harness's listing entry for `suite/name` (with or without `.case.json`)."""
    suite, _, name = case_id.partition("/")
    wanted = f"{suite}/{name}" if name.endswith(".case.json") else f"{suite}/{name}.case.json"
    for entry in suite_cases(suite):
        if entry["id"] == wanted:
            return entry
    msg = f"case {wanted} not found in suite {suite}"
    raise FileNotFoundError(msg)


def load_workbook(case_path: Path, sidecar: Path) -> Workbook:
    """Reads a case and its oracle sidecar (JSON; see docs/architecture/oracle.md)."""
    spec = json.loads(case_path.read_text(encoding="utf-8"))
    if not sidecar.exists():
        msg = f"{case_path.name} has no oracle sidecar yet ({sidecar}); fill it first"
        raise FileNotFoundError(msg)
    oracle = json.loads(sidecar.read_text(encoding="utf-8"))
    return Workbook(sheets={s["name"]: s["cells"] for s in spec["sheets"]}, oracle=oracle["sheets"])


def _arg_texts(m: re.Match[str]) -> list[str]:
    """One group: the argument list, split at top-level commas. Several groups: one argument per
    group that matched (so alternatives in a pattern can leave some out)."""
    groups = m.groups()
    if len(groups) <= 1:
        return split_args(groups[0]) if groups and groups[0] is not None else []
    return [g.strip() for g in groups if g is not None]


def select(case_id: str, book: Workbook, pattern: re.Pattern[str], sweep: Sweep, where: Any = None) -> None:
    """Adds `book`'s cells whose formula matches `pattern` to `sweep`. `where(cell)`, when given,
    keeps only the cells it returns true for."""
    for sheet, cells in book.sheets.items():
        for address, raw in cells.items():
            if not (isinstance(raw, str) and (m := pattern.match(raw))):
                continue
            entry = book.oracle.get(sheet, {}).get(address)
            if not isinstance(entry, dict) or entry.get("t") != "n":
                sweep.non_numeric += 1
                continue
            try:
                args = tuple(parse_arg(a, sheet, book) for a in _arg_texts(m))
            except UnparsedError as e:
                sweep.unparsed.append(f"{case_id} {sheet}!{address}: {e}")
                continue
            cell = Cell(case_id, sheet, address, raw, args, float(entry["v"]))
            if where is not None and not where(cell):
                sweep.excluded += 1
                continue
            sweep.cells.append(cell)


def load_sweep(case_ids: list[str], pattern: str, where: Any = None) -> Sweep:
    """The matching cells of every case in `case_ids` (`suite/name`)."""
    regex = re.compile(pattern)
    sweep = Sweep()
    for case_id in case_ids:
        entry = case_entry(case_id)
        book = load_workbook(Path(entry["path"]), Path(entry["sidecar"]))
        select(entry["id"], book, regex, sweep, where)
    return sweep
