"""`rustsheet-oracle` — compute Excel's answers for eval cases and commit them as sidecars.

  rustsheet-oracle status                         which backends are usable here
  rustsheet-oracle login                          one-time device-code login (Graph, delegated)
  rustsheet-oracle recalc -b BACKEND in.xlsx      recalc one workbook, print formula values as JSON
  rustsheet-oracle fill -b BACKEND --suite S      write <case>.oracle.json for cases lacking one
  rustsheet-oracle calibrate -b BACKEND --suite S compare the oracle with an xlsx suite's cached values

Every workbook is sent with an embedded canary sheet (oracle/canary.py) whose poisoned cached
values must come back recalculated; otherwise that workbook's results are refused.
Results are cached under eval/.cache/oracle/<backend>/<sha256>.xlsx (re-verified on every read)
so re-runs only hit the network for new or changed workbooks. See docs/architecture/oracle.md.
"""

from __future__ import annotations

import argparse
import contextlib
import datetime as dt
import hashlib
import json
import subprocess
import sys
import tempfile
import threading
from pathlib import Path
from typing import Any

from rustsheet_tools import repo_root
from rustsheet_tools.oracle.backends import BACKENDS, OracleError, OracleInfo
from rustsheet_tools.oracle.calibrate import calibrate, to_markdown
from rustsheet_tools.oracle.canary import checked_values, embed_canary
from rustsheet_tools.snapshot import compare_workbooks, formula_values, snapshot, suite_cases
from rustsheet_tools.spec2xlsx import build_xlsx

GOLDEN_BACKENDS = {"excel-graph", "excel-desktop"}


def _backend(name: str) -> Any:
    if name not in BACKENDS:
        sys.exit(f"unknown backend {name}; choose from {sorted(BACKENDS)}")
    return BACKENDS[name]()


def _cache_paths(backend: Any, sha: str) -> tuple[Path, Path]:
    """(recalculated xlsx, metadata) cache files for the input with this sha256."""
    cache_dir = repo_root() / "eval" / ".cache" / "oracle" / backend.name
    cache_dir.mkdir(parents=True, exist_ok=True)
    return cache_dir / f"{sha}.xlsx", cache_dir / f"{sha}.meta.json"


def _recalc_cached(backend: Any, xlsx: bytes) -> tuple[dict[str, dict[str, Any]], OracleInfo, str]:
    """Formula values of `xlsx` as recalculated by `backend`, canary-verified (see canary.py).

    The recalculated workbook stays in the cache at `_cache_paths(backend, sha)[0]`.
    """
    sha = hashlib.sha256(xlsx).hexdigest()
    out, meta = _cache_paths(backend, sha)
    canaried = embed_canary(xlsx)  # first, so refusals also apply to results cached before them
    if out.exists() and meta.exists():
        with contextlib.suppress(OracleError):  # entries without a passing canary are recomputed
            return checked_values(out), OracleInfo(**json.loads(meta.read_text())), sha
    data, info = backend.recalc(canaried)
    out.write_bytes(data)
    try:
        values = checked_values(out)
    except OracleError:
        out.unlink()
        raise
    meta.write_text(json.dumps(info.__dict__))
    return values, info, sha


def cmd_status(_: argparse.Namespace) -> int:
    for name, cls in BACKENDS.items():
        ok, why = cls().available()
        golden = "golden" if name in GOLDEN_BACKENDS else "approximate"
        print(f"{name:<14} {'OK ' if ok else 'n/a'}  [{golden}]  {why}")
    return 0


def cmd_login(_: argparse.Namespace) -> int:
    from rustsheet_tools.oracle.backends import GraphBackend  # noqa: PLC0415

    GraphBackend().login()
    return 0


def cmd_recalc(args: argparse.Namespace) -> int:
    backend = _backend(args.backend)
    raw = args.input.read_bytes()
    if args.input.name.endswith(".case.json"):
        raw = build_xlsx(json.loads(raw))
    values, info, sha = _recalc_cached(backend, raw)
    print(json.dumps({"source": {**info.__dict__, "input_sha256": sha}, "sheets": values}, indent=2))
    return 0


def _skip_reason(case: dict[str, Any], include_cached: bool) -> str | None:
    """Why an xlsx case needs no oracle run: unreadable, nothing to score, or cached values trusted."""
    if case.get("unreadable"):
        return f"rustsheet-eval cannot read it ({case['unreadable'][:120]})"
    if case.get("formula_cells") == 0:
        return "no formula cells"
    if case.get("trusts_cached_values") and not include_cached:
        return "cached values already trusted (Excel-authored); use --include-cached to override"
    return None


def cmd_fill(args: argparse.Namespace) -> int:
    backend = _backend(args.backend)
    if backend.name not in GOLDEN_BACKENDS:
        sys.exit(f"{backend.name} is approximate and must not produce goldens; use `recalc` for triage")
    ok, why = backend.available()
    if not ok:
        sys.exit(f"{backend.name} unavailable: {why}")
    done = failed = skipped = conflicts = 0
    for entry in suite_cases(args.suite)[: args.limit or None]:
        case, sidecar = Path(entry["path"]), Path(entry["sidecar"])
        is_spec = case.name.endswith(".case.json")
        if sidecar.exists() and not args.force:
            continue
        if not is_spec:
            reason = _skip_reason(entry, args.include_cached)
            if reason:
                skipped += 1
                print(f"skip {case.name}: {reason}", file=sys.stderr)
                continue
        raw = case.read_bytes()
        spec = json.loads(raw) if is_spec else None
        xlsx = build_xlsx(spec) if spec else raw
        try:
            values, info, sha = _recalc_cached(backend, xlsx)
            if not spec:
                conflicts += _drop_desktop_conflicts(case, values)
            elif spec.get("invariant"):
                _drop_varying_invariants(case, spec["invariant"], values, _recalc_fresh(backend, xlsx))
        except OracleError as e:
            failed += 1
            print(f"FAIL {case}: {e}", file=sys.stderr)
            continue
        oracle = {
            "format": "rustsheet-oracle/v1",
            "source": {
                "kind": info.kind,
                "application": info.application,
                "version": info.version,
                "produced_at": dt.datetime.now(dt.UTC).isoformat(timespec="seconds"),
                "input_sha256": sha,
            },
            "sheets": values,
        }
        if info.texts:  # displayed text of the cells the case lists in `text` (T-1610)
            oracle["text"] = info.texts
        _drop_undecoded(case, values)
        if spec:
            _report_hand_disagreements(case, spec, values)
        sidecar.parent.mkdir(parents=True, exist_ok=True)
        sidecar.write_text(json.dumps(oracle, indent=1, sort_keys=True) + "\n")
        done += 1
    print(
        f"wrote {done} oracle sidecars ({failed} failures, {skipped} skipped, "
        f"{conflicts} web-vs-desktop conflicts left pending)"
    )
    return 1 if failed else 0


def cmd_calibrate(args: argparse.Namespace) -> int:
    workbooks = [Path(e["path"]) for e in suite_cases(args.suite)]
    if not workbooks or any(p.name.endswith(".case.json") for p in workbooks):
        sys.exit(f"calibrate needs an xlsx suite of Excel-authored workbooks; {args.suite} is not one")
    ok, why = _backend(args.backend).available()
    if not ok:
        sys.exit(f"{args.backend} unavailable: {why}")
    local = threading.local()  # one backend (HTTP session) per worker thread

    def recalculated(path: Path) -> Path:
        if not hasattr(local, "backend"):
            local.backend = _backend(args.backend)
        _, _, sha = _recalc_cached(local.backend, path.read_bytes())
        return _cache_paths(local.backend, sha)[0]

    result = calibrate(workbooks[: args.limit or None], recalculated, compare_workbooks, args.jobs)
    if args.json:
        args.json.write_text(json.dumps(result.disagreements, indent=1) + "\n")
    print(to_markdown(result, args.examples), end="")
    return 0


def _split_cell_key(key: str) -> tuple[str, str]:
    """(sheet, A1) of a case cell key written `Sheet!A1`, `'My Sheet'!$A$1` or similar."""
    sheet, _, a1 = key.rpartition("!")
    return sheet.strip("'").replace("''", "'"), a1.replace("$", "")


def _report_hand_disagreements(case: Path, spec: dict[str, Any], values: dict[str, dict[str, Any]]) -> None:
    for key, hand in spec.get("expected", {}).items():
        sheet, a1 = _split_cell_key(key)
        got = values.get(sheet, {}).get(a1)
        if got is not None and got != hand:
            print(f"NOTE {case.name} {key}: hand={hand} excel={got} (oracle wins)", file=sys.stderr)


def _recalc_fresh(backend: Any, xlsx: bytes) -> dict[str, dict[str, Any]]:
    """Formula values of a second, uncached recalculation of `xlsx`, canary-verified."""
    data, _ = backend.recalc(embed_canary(xlsx))
    with tempfile.TemporaryDirectory() as tmp:
        out = Path(tmp) / "again.xlsx"
        out.write_bytes(data)
        return checked_values(out)


def _drop_varying_invariants(
    case: Path, invariant: list[str], values: dict[str, dict[str, Any]], again: dict[str, dict[str, Any]]
) -> None:
    """Leaves pending every `invariant` cell (T-1283) whose value differs between two recalculations:
    it calls a volatile function, so a single fill cannot show that its value never changes."""
    for key in invariant:
        sheet, a1 = _split_cell_key(key)
        first = values.get(sheet, {}).get(a1)
        if first is not None and first != again.get(sheet, {}).get(a1):
            print(
                f"VARIES {case.name} {key}: listed as invariant but changed between fills (left pending)",
                file=sys.stderr,
            )
            del values[sheet][a1]


# Same string as UNDECODED_RICH_ERROR in crates/rustsheet-eval/src/xlsx_snapshot.rs.
UNDECODED = {"t": "e", "v": "#UNDECODED!"}


def _drop_undecoded(case: Path, values: dict[str, dict[str, Any]]) -> None:
    """Leaves pending the cells whose rich-value error the harness cannot decode yet: a golden
    must hold the error Excel produced, never a guess (T-0203 review: errorType 10)."""
    for sheet, cells in values.items():
        for a1 in [a for a, v in cells.items() if v == UNDECODED]:
            print(
                f"UNDECODED {case.name} {sheet}!{a1}: rich-value error type unknown (left pending)",
                file=sys.stderr,
            )
            del cells[a1]


def _materially_differ(a: dict[str, Any], b: dict[str, Any]) -> bool:
    """Coarse filter, not scoring: last-digit float noise between Excel builds is not a conflict.

    1e-9 relative sits far above the few-ulp web/desktop differences seen in calibration and far
    below any real disagreement found so far (wrong type, wrong text, wrong count)."""
    if a.get("t") == b.get("t") == "n":
        x, y = float(a["v"]), float(b["v"])
        return abs(x - y) > 1e-9 * max(abs(x), abs(y), 1e-300)
    return a != b


def _drop_desktop_conflicts(case: Path, values: dict[str, dict[str, Any]]) -> int:
    """Removes cells where Excel for the web contradicts the values Microsoft Excel cached in the
    workbook, so they stay pending until an excel-desktop probe decides (T-0019 review: lowercase-e
    TEXT formats and elapsed tokens after `s` give #VALUE!/odd values on the web only).

    Any Microsoft application counts as evidence here, deliberately broader than
    `is_excel_authored` (which rejects AppVersion 12.0 because XlsxWriter fakes it): the
    lowercase-e conflicts come from a real Mac Excel 2008 (12.0) file, and a false conflict only
    leaves a cell pending.

    Also overwrites a web number that differs by less than 1e-9 with the cached value when the
    workbook was saved by a 16.x build (decision 0030 rule 1, T-0142). Returns the dropped count."""
    try:
        snap = snapshot(case)
    except subprocess.CalledProcessError as e:  # never write web values we could not check
        raise OracleError(f"cannot check against cached values: {(e.stderr or '').strip()[:120]}") from e
    if not (snap.get("application") or "").startswith("Microsoft"):
        return 0  # cached by POI, LibreOffice, a generator...: no Excel evidence to conflict with
    # Decision 0030 rules 1/2: only a 16.x build outranks the web; older builds (12.0 = Mac Excel
    # 2008, XlsxWriter's placeholder) keep the web value when the difference is last-digit noise.
    is_desktop = (snap.get("app_version") or "").startswith("16.")
    cached = formula_values(snap)
    dropped = 0
    for sheet, cells in values.items():
        for a1 in list(cells):
            mine = cached.get(sheet, {}).get(a1)
            if mine is None:
                continue
            if is_desktop and cells[a1] != mine and not _materially_differ(cells[a1], mine):
                print(
                    f"DESKTOP {case.name} {sheet}!{a1}: web={cells[a1]} desktop={mine} (desktop value kept)",
                    file=sys.stderr,
                )
                cells[a1] = mine
            elif _materially_differ(cells[a1], mine):
                print(
                    f"CONFLICT {case.name} {sheet}!{a1}: web={cells[a1]} desktop={mine} (left pending)",
                    file=sys.stderr,
                )
                del cells[a1]
                dropped += 1
    return dropped


def main() -> None:
    ap = argparse.ArgumentParser(prog="rustsheet-oracle", description=__doc__.splitlines()[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("status").set_defaults(fn=cmd_status)
    sub.add_parser("login").set_defaults(fn=cmd_login)
    p = sub.add_parser("recalc")
    p.add_argument("-b", "--backend", required=True)
    p.add_argument("input", type=Path)
    p.set_defaults(fn=cmd_recalc)
    p = sub.add_parser("fill")
    p.add_argument("-b", "--backend", required=True)
    p.add_argument("--suite", required=True)
    p.add_argument("--force", action="store_true")
    p.add_argument("--limit", type=int, default=0)
    p.add_argument(
        "--include-cached",
        action="store_true",
        help="also recalc xlsx cases whose cached values the suite already trusts (calibration)",
    )
    p.set_defaults(fn=cmd_fill)
    p = sub.add_parser("calibrate")
    p.add_argument("-b", "--backend", required=True)
    p.add_argument("--suite", required=True)
    p.add_argument("--jobs", type=int, default=4)
    p.add_argument("--limit", type=int, default=0)
    p.add_argument("--examples", type=int, default=50, help="disagreeing cells listed in the report")
    p.add_argument("--json", type=Path, help="also write every disagreeing cell as JSON here")
    p.set_defaults(fn=cmd_calibrate)
    args = ap.parse_args()
    with tempfile.TemporaryDirectory():
        sys.exit(args.fn(args))


if __name__ == "__main__":
    main()
