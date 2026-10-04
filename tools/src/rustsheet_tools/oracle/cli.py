"""`rustsheet-oracle` — compute Excel's answers for eval cases and commit them as sidecars.

  rustsheet-oracle status                         which backends are usable here
  rustsheet-oracle login                          one-time device-code login (Graph, delegated)
  rustsheet-oracle recalc -b BACKEND in.xlsx      recalc one workbook, print formula values as JSON
  rustsheet-oracle fill -b BACKEND --suite S      write <case>.oracle.json for cases lacking one

Every workbook is sent with an embedded canary sheet (oracle/canary.py) whose poisoned cached
values must come back recalculated; otherwise that workbook's results are refused.
Results are cached under eval/.cache/oracle/<backend>/<sha256>.xlsx (re-verified on every read)
so re-runs only hit the network for new or changed workbooks. See docs/architecture/oracle.md.
"""

from __future__ import annotations

import argparse
import contextlib
import datetime as dt
import fnmatch
import hashlib
import json
import sys
import tempfile
import tomllib
from pathlib import Path
from typing import Any

from rustsheet_tools import repo_root
from rustsheet_tools.oracle.backends import BACKENDS, OracleError, OracleInfo
from rustsheet_tools.oracle.canary import checked_values, embed_canary
from rustsheet_tools.snapshot import snapshot
from rustsheet_tools.spec2xlsx import build_xlsx

GOLDEN_BACKENDS = {"excel-graph", "excel-desktop"}


def _backend(name: str) -> Any:
    if name not in BACKENDS:
        sys.exit(f"unknown backend {name}; choose from {sorted(BACKENDS)}")
    return BACKENDS[name]()


def _recalc_cached(backend: Any, xlsx: bytes) -> tuple[dict[str, dict[str, Any]], OracleInfo, str]:
    """Formula values of `xlsx` as recalculated by `backend`, canary-verified (see canary.py)."""
    sha = hashlib.sha256(xlsx).hexdigest()
    cache_dir = repo_root() / "eval" / ".cache" / "oracle" / backend.name
    cache_dir.mkdir(parents=True, exist_ok=True)
    out = cache_dir / f"{sha}.xlsx"
    meta = cache_dir / f"{sha}.meta.json"
    if out.exists() and meta.exists():
        with contextlib.suppress(OracleError):  # entries without a passing canary are recomputed
            return checked_values(out), OracleInfo(**json.loads(meta.read_text())), sha
    data, info = backend.recalc(embed_canary(xlsx))
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


def _glob_match(rel: str, pattern: str) -> bool:
    """Approximates the harness's globset match for the `**/x` and `x` patterns suites use."""
    return fnmatch.fnmatch(rel, pattern) or (
        pattern.startswith("**/") and fnmatch.fnmatch(rel.rsplit("/", 1)[-1], pattern[3:])
    )


def _suite_cases(suite: str) -> tuple[dict[str, Any], list[tuple[Path, Path]]]:
    """(manifest, [(case, sidecar)]) the way `rustsheet-eval` discovers them (suite.rs `cases`)."""
    sdir = repo_root() / "eval" / "suites" / suite
    manifest = tomllib.loads((sdir / "suite.toml").read_text())
    if "fetch" in manifest:  # fetched suites live in the cache, like Suite::root()
        root = repo_root() / "eval" / ".cache" / "suites" / manifest["name"]
    else:
        root = (sdir / manifest.get("root", "cases")).resolve()
    ext = ".xlsx" if manifest["kind"] == "xlsx" else ".case.json"
    include, exclude = manifest.get("include", ["**/*"]), manifest.get("exclude", [])
    out = []
    for case in sorted(root.rglob(f"*{ext}")):
        rel = case.relative_to(root).as_posix()
        if case.name.startswith("~$") or not any(_glob_match(rel, p) for p in include):
            continue
        if any(_glob_match(rel, p) for p in exclude):
            continue
        mirror = sdir / manifest["oracle_dir"] / rel if manifest.get("oracle_dir") else case
        out.append((case, _sidecar(mirror)))
    return manifest, out


def _sidecar(path: Path) -> Path:
    """`foo.xlsx` / `foo.xlsm` / `foo.case.json` → `foo.oracle.json` (suite.rs `oracle_sidecar`)."""
    stem = path.name.removesuffix(".case.json").removesuffix(".xlsx").removesuffix(".xlsm")
    return path.with_name(f"{stem}.oracle.json")


def _skip_reason(manifest: dict[str, Any], case: Path, include_cached: bool) -> str | None:
    """Why an xlsx case needs no oracle run: nothing to score, or cached values already trusted."""
    snap = snapshot(case)
    if not any(c.get("formula") for sheet in snap["sheets"] for c in sheet["cells"].values()):
        return "no formula cells"
    trusted = manifest.get("use_cached_values") and (
        snap.get("excel_authored") or not manifest.get("require_excel_authored", True)
    )
    if trusted and not include_cached:
        return "cached values already trusted (Excel-authored); use --include-cached to override"
    return None


def cmd_fill(args: argparse.Namespace) -> int:
    backend = _backend(args.backend)
    if backend.name not in GOLDEN_BACKENDS:
        sys.exit(f"{backend.name} is approximate and must not produce goldens; use `recalc` for triage")
    ok, why = backend.available()
    if not ok:
        sys.exit(f"{backend.name} unavailable: {why}")
    manifest, cases = _suite_cases(args.suite)
    done = failed = skipped = 0
    for case, sidecar in cases[: args.limit or None]:
        if sidecar.exists() and not args.force:
            continue
        if not case.name.endswith(".case.json"):
            reason = _skip_reason(manifest, case, args.include_cached)
            if reason:
                skipped += 1
                print(f"skip {case.name}: {reason}", file=sys.stderr)
                continue
        raw = case.read_bytes()
        spec = json.loads(raw) if case.name.endswith(".case.json") else None
        xlsx = build_xlsx(spec) if spec else raw
        try:
            values, info, sha = _recalc_cached(backend, xlsx)
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
        if spec:
            _report_hand_disagreements(case, spec, values)
        sidecar.parent.mkdir(parents=True, exist_ok=True)
        sidecar.write_text(json.dumps(oracle, indent=1, sort_keys=True) + "\n")
        done += 1
    print(f"wrote {done} oracle sidecars ({failed} failures, {skipped} skipped)")
    return 1 if failed else 0


def _report_hand_disagreements(case: Path, spec: dict[str, Any], values: dict[str, dict[str, Any]]) -> None:
    for key, hand in spec.get("expected", {}).items():
        sheet, _, a1 = key.rpartition("!")
        sheet = sheet.strip("'").replace("''", "'")
        got = values.get(sheet, {}).get(a1)
        if got is not None and got != hand:
            print(f"NOTE {case.name} {key}: hand={hand} excel={got} (oracle wins)", file=sys.stderr)


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
    args = ap.parse_args()
    with tempfile.TemporaryDirectory():
        sys.exit(args.fn(args))


if __name__ == "__main__":
    main()
