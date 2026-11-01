"""The eval suites' manifests (`eval/suites/*/suite.toml`) as one commit has them, for the merge
tooling: which known-failure lists CI's eval gate checks (`merge_ready.Repo.checked_baselines`,
decision 0207) and where each suite's case files live (`visible_lock.case_on`, decision 0217)."""

from __future__ import annotations

import posixpath
import tomllib
from dataclasses import dataclass
from pathlib import Path

from merge_lock import git

SUITES_DIR = "eval/suites"


@dataclass(frozen=True)
class SuiteMeta:
    """One suite's manifest fields the merge tooling reads."""

    name: str  # the suite's `name` (its directory's name when unset), which names its list
    gated: bool  # `gate = true`
    fetched: bool  # has a `[fetch]` table: CI's eval job does not run it
    root: str | None  # the case root's repository path (`root`, default `cases`); None if malformed


def suites_on(root: Path, commit: str) -> list[SuiteMeta]:
    """The suites of `commit`, in directory order; a manifest that is missing, not TOML, or whose
    `name` is not a string is left out."""
    found = []
    listed = git("ls-tree", "--name-only", f"{commit}:{SUITES_DIR}", root=root, check=False)
    for suite in listed.stdout.split() if listed.returncode == 0 else []:
        shown = git("show", f"{commit}:{SUITES_DIR}/{suite}/suite.toml", root=root, check=False)
        if shown.returncode != 0:
            continue
        try:
            meta = tomllib.loads(shown.stdout)
        except tomllib.TOMLDecodeError:
            continue
        name, base = meta.get("name", suite), meta.get("root", "cases")
        if not isinstance(name, str):
            continue
        case_root = posixpath.normpath(f"{SUITES_DIR}/{suite}/{base}") if isinstance(base, str) else None
        found.append(SuiteMeta(name, meta.get("gate") is True, "fetch" in meta, case_root))
    return found
