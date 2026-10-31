"""Writing and verifying Gate A batches (SPEC-0029, "Location and files", "Point fields").

A batch is `tables/<FUNCTION>/<batch>.case.json` (rustsheet-case/v1, one cell `Calc!A<n>` per
point, its arguments in `hypotheses` as `gate-a/<stratum>/<arg1>,...`) and
`tables/<FUNCTION>/<batch>.math.json` (rustsheet-math/v1: per cell the 50-digit value or the
domain error, the formula text `f` and the stratum `s`). The oracle sidecar comes from
`rustsheet-oracle fill --suite accuracy`; a batch never changes once written.
"""

from __future__ import annotations

import contextlib
import hashlib
import json
import random
import re
import signal
import sys
import threading
from collections.abc import Iterator, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import mpmath

from rustsheet_tools.accuracy import literal
from rustsheet_tools.accuracy.model import Definition, DomainError, NoValueError
from rustsheet_tools.accuracy.sampler import STRATA, Sampler, math_arg, point_count, seed, stratum_counts
from rustsheet_tools.fitlab.sweep import number_literal, split_args

VERSION = "1"  # bump when a definition or the sampler changes what a batch would contain
DPS = (60, 80)  # the value is the dps-80 one; dps 60 is the convergence check
CONVERGENCE = mpmath.mpf("1e-45")  # relative agreement the two precisions must reach
DIGITS = 50  # significant digits written
TIME_LIMIT = 10.0  # seconds per point (both precisions)
MAX_REDRAWS = 20  # a point whose math overflows is redrawn this many times, then dropped
BATCH_NAME = re.compile(r"[A-Za-z0-9][A-Za-z0-9_-]*")  # a file name, never a path
DOUBLE_MAX = mpmath.mpf(1.7976931348623157e308)


class PointTimeoutError(Exception):
    """A point's math took longer than `TIME_LIMIT`."""


@contextlib.contextmanager
def time_limit(seconds: float) -> Iterator[None]:
    """Raises `PointTimeoutError` in the block after `seconds` (SIGALRM; a no-op off the main
    thread or where the platform has no interval timer)."""
    if not hasattr(signal, "setitimer") or threading.current_thread() is not threading.main_thread():
        yield
        return

    def expire(_signum: int, _frame: object) -> None:
        raise PointTimeoutError

    previous = signal.signal(signal.SIGALRM, expire)
    signal.setitimer(signal.ITIMER_REAL, seconds)
    try:
        yield
    finally:
        signal.setitimer(signal.ITIMER_REAL, 0)
        signal.signal(signal.SIGALRM, previous)


@dataclass(frozen=True)
class MathValue:
    """A point's math: `value` (50 significant digits) or `why` (a domain error)."""

    value: str | None = None
    why: str | None = None

    def to_json(self) -> dict[str, str]:
        """The sidecar cell's value fields."""
        return {"t": "n", "v": self.value} if self.value is not None else {"t": "e", "why": self.why or ""}


class DroppedError(Exception):
    """The point is left out of the batch (not converged, too slow, or no single value)."""


def compute(definition: Definition, args: Sequence[literal.Arg]) -> MathValue:
    """The math value of `args` at dps 80, checked against dps 60 (DroppedError when they differ
    by more than 1e-45 relative, when one is a domain error and the other not, or past the time
    limit). NoValueError (redraw) when the value overflows the double range or has no single
    value."""
    math_args = [math_arg(p, a) for p, a in zip(definition.params, args, strict=True)]
    results: list[mpmath.mpf | DomainError] = []
    try:
        with time_limit(TIME_LIMIT):
            for dps in DPS:
                with mpmath.workdps(dps):
                    try:
                        results.append(+definition.evaluate(math_args))
                    except DomainError as err:
                        results.append(err)
    except PointTimeoutError as err:
        msg = f"over {TIME_LIMIT:g} s"
        raise DroppedError(msg) from err
    except mpmath.libmp.NoConvergence as err:  # e.g. a hypergeometric series at a huge df
        msg = f"mpmath did not converge ({err})"
        raise DroppedError(msg) from err
    except NoValueError:
        raise
    except ValueError as err:  # mpmath's hypercomb gives up near 0 or infinity with a ValueError
        msg = f"mpmath did not converge ({str(err).strip().splitlines()[0]})"
        raise DroppedError(msg) from err
    low, high = results
    if isinstance(high, DomainError) or isinstance(low, DomainError):
        if isinstance(high, DomainError) and isinstance(low, DomainError):
            return MathValue(why=str(high))
        msg = "domain error at one precision only"
        raise DroppedError(msg)
    if not mpmath.isfinite(high) or abs(high) > DOUBLE_MAX:
        msg = "math value beyond the double range"
        raise NoValueError(msg)
    with mpmath.workdps(DPS[-1]):
        if abs(high - low) > CONVERGENCE * abs(high):
            msg = f"not converged: dps 60 and 80 differ by {mpmath.nstr(abs(high - low), 3)}"
            raise DroppedError(msg)
        return MathValue(value=mpmath.nstr(high, DIGITS))


@dataclass(frozen=True)
class Point:
    """One written point."""

    stratum: str
    args: tuple[literal.Arg, ...]
    math: MathValue


def formula(function: str, args: Sequence[literal.Arg]) -> str:
    """`=T.INV(1.87E-297,3937)`."""
    return f"={function}({','.join(literal.text(a) for a in args)})"


def draw_points(
    definition: Definition,
    batch: str,
    count: int | None = None,
    *,
    stratum: str | None = None,
    log: Any = None,
) -> list[Point]:
    """The batch's points: `count` (default `point_count`) split over the strata, or all from
    `stratum`. Dropped points are logged to `log` (stderr by default) with their arguments."""
    log = log or sys.stderr
    total = count if count is not None else point_count(definition)
    counts = {stratum: total} if stratum is not None else stratum_counts(total)
    sampler = Sampler(definition, random.Random(seed(definition.function, batch)))
    points: list[Point] = []
    for name in STRATA:
        for index in range(counts.get(name, 0)):
            for _ in range(MAX_REDRAWS):
                args = sampler.draw(name, index)
                try:
                    points.append(Point(name, tuple(args), compute(definition, args)))
                except NoValueError:
                    continue  # overflow or several roots: redraw
                except DroppedError as err:
                    print(f"dropped {formula(definition.function, args)} ({name}): {err}", file=log)
                break
            else:
                print(f"dropped a {name} point after {MAX_REDRAWS} redraws", file=log)
    return points


MATH_FORMAT = "rustsheet-math/v1"


def dump_json(data: dict[str, Any]) -> bytes:
    """A case or sidecar file's bytes: indented JSON, UTF-8, one trailing newline."""
    return (json.dumps(data, indent=1, ensure_ascii=False) + "\n").encode("utf-8")


def math_sidecar_bytes(
    case_bytes: bytes, sheets: dict[str, dict[str, dict[str, str]]], seed: int | None = None
) -> bytes:
    """A `rustsheet-math/v1` sidecar for the case file `case_bytes`: the one envelope a table
    batch (with its `seed`) and an eval case's math references (`eval_math`, no seed) share."""
    source: dict[str, Any] = {"generator": "rustsheet-accuracy", "version": VERSION, "dps": DPS[-1]}
    if seed is not None:
        source["seed"] = seed
    source["input_sha256"] = hashlib.sha256(case_bytes).hexdigest()
    return dump_json({"format": MATH_FORMAT, "source": source, "sheets": sheets})


def batch_files(definition: Definition, batch: str, points: Sequence[Point]) -> tuple[bytes, bytes]:
    """The case file's and the math sidecar's bytes."""
    cells: dict[str, str] = {}
    hypotheses: dict[str, str] = {}
    math_cells: dict[str, dict[str, str]] = {}
    for n, point in enumerate(points, start=1):
        address = f"A{n}"
        text = formula(definition.function, point.args)
        cells[address] = text
        hypotheses[f"Calc!{address}"] = (
            f"gate-a/{point.stratum}/{','.join(literal.text(a) for a in point.args)}"
        )
        math_cells[address] = {**point.math.to_json(), "f": text, "s": point.stratum}
    case = {
        "format": "rustsheet-case/v1",
        "description": (
            f"Gate A reference table for {definition.function}, batch {batch} (SPEC-0029): "
            f"generated by rustsheet-accuracy {VERSION}, do not edit"
        ),
        "tags": ["accuracy", "gate-a", definition.function],
        "sheets": [{"name": "Calc", "cells": cells}],
        "check": [f"Calc!{a}" for a in cells],
        "hypotheses": hypotheses,
    }
    case_bytes = dump_json(case)
    return case_bytes, math_sidecar_bytes(case_bytes, {"Calc": math_cells}, seed(definition.function, batch))


def write_batch(
    tables: Path, definition: Definition, batch: str, points: Sequence[Point]
) -> tuple[Path, Path]:
    """Writes the batch under `tables/<FUNCTION>/`; refuses to overwrite one (a committed batch
    never changes: new points are a new batch)."""
    if not BATCH_NAME.fullmatch(batch):
        msg = f"batch name {batch!r} must match {BATCH_NAME.pattern} (s1, r4-T-1234, e-T-1234)"
        raise ValueError(msg)
    folder = tables / definition.function
    case_path, math_path = folder / f"{batch}.case.json", folder / f"{batch}.math.json"
    for path in (case_path, math_path):
        if path.exists():
            msg = f"{path} exists: a batch is never rewritten, generate a new batch name"
            raise FileExistsError(msg)
    case_bytes, math_bytes = batch_files(definition, batch, points)
    folder.mkdir(parents=True, exist_ok=True)
    case_path.write_bytes(case_bytes)
    math_path.write_bytes(math_bytes)
    return case_path, math_path


def parse_formula(definition: Definition, text: str) -> list[literal.Arg]:
    """The arguments of a table formula `=FUNC(lit,...)`, read as Excel reads them."""
    prefix = f"={definition.function}("
    if not (text.startswith(prefix) and text.endswith(")")):
        msg = f"{text!r} is not one call of {definition.function}"
        raise ValueError(msg)
    parts = split_args(text[len(prefix) : -1])
    if len(parts) != len(definition.params):
        msg = f"{text!r}: {len(parts)} arguments, the definition has {len(definition.params)}"
        raise ValueError(msg)
    args: list[literal.Arg] = []
    for param, part in zip(definition.params, parts, strict=True):
        token = part.strip()
        if param.kind == "bool":
            args.append(token.upper() == "TRUE")
        elif param.kind == "int":
            args.append(int(number_literal(token)))
        else:
            args.append(number_literal(token))
    return args


def verify_batch(definition: Definition, case_path: Path, math_path: Path) -> list[str]:
    """Recomputes every committed math value of a batch; one line per difference (a changed
    value, a changed domain verdict, a stale formula or a case whose hash moved)."""
    problems: list[str] = []
    case_bytes = case_path.read_bytes()
    sidecar = json.loads(math_path.read_text(encoding="utf-8"))
    if sidecar.get("source", {}).get("input_sha256") != hashlib.sha256(case_bytes).hexdigest():
        problems.append(f"{case_path}: input_sha256 differs from the case file")
    cells = json.loads(case_bytes)["sheets"][0]["cells"]
    for address, committed in sidecar["sheets"]["Calc"].items():
        where = f"{math_path.parent.name}/{math_path.name} Calc!{address}"
        if cells.get(address) != committed.get("f"):
            problems.append(
                f"{where}: math formula {committed.get('f')!r} is not the cell's {cells.get(address)!r}"
            )
            continue
        try:
            fresh = compute(definition, parse_formula(definition, committed["f"]))
        except (DroppedError, NoValueError, ValueError) as err:
            problems.append(f"{where}: {committed['f']} no longer computes ({err})")
            continue
        if committed.get("t") == "e" or fresh.value is None:
            if committed.get("t") != "e" or fresh.value is not None:
                problems.append(
                    f"{where}: {committed['f']} committed {committed} recomputed {fresh.to_json()}"
                )
            continue
        with mpmath.workdps(DPS[-1]):
            old, new = mpmath.mpf(committed["v"]), mpmath.mpf(fresh.value)
            if abs(old - new) > CONVERGENCE * abs(new):
                problems.append(
                    f"{where}: {committed['f']} committed {committed['v']} recomputed {fresh.value}"
                )
    return problems
