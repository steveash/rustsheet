"""The review panel: which reviewer agents a diff needs (decision 0236).

The one definition is the `review` skill (.claude/skills/review/SKILL.md, step 3);
scripts/test_merge_ready.py fails when this copy differs from it. `merge_ready` (markers, verdict
records, `make mark-ready`) reads the panel from here. Standard library and `ci_changes` only.
"""

from __future__ import annotations

from collections.abc import Iterable, Mapping

import ci_changes

APPROVE = "APPROVE"

# Decision 0236: one `design` reviewer replaced `architecture` and `quality`, and `security`
# reviews every diff but one whose every path is under SECURITY_SKIP_PATHS (and none under
# SECURITY_KEEP_PATHS).
SECURITY = "security"
DESIGN = "design"
ALWAYS_REVIEWERS = (DESIGN,)
SECURITY_SKIP_PATHS = ("docs", "eval/suites")
SECURITY_KEEP_PATHS = ("docs/process",)
EXCEL_FIDELITY = "excel-fidelity"
EXCEL_FIDELITY_PATHS = ("crates/rustsheet-engine", "crates/rustsheet-xlsx", "eval/suites", "eval/baseline")
REVIEWERS = (SECURITY, DESIGN, EXCEL_FIDELITY)
# The panel before decision 0236. Markers and verdict records that name them still parse, and a
# head both approved counts as approved by `design`, so PRs marked before the switch stay ready.
LEGACY_DESIGN = ("architecture", "quality")


def with_legacy_design(verdicts: Mapping[str, str]) -> dict[str, str]:
    """`verdicts` with `design` filled in from the pre-0236 `architecture` and `quality` verdicts:
    APPROVE only when both are APPROVE, and only when `design` has no verdict of its own. The
    legacy names are dropped either way. An `n/a` pair also folds to REQUEST_CHANGES (0236,
    Transition): no diff that needs `design` could have had both legacy reviewers not apply."""
    out = {name: verdict for name, verdict in verdicts.items() if name not in LEGACY_DESIGN}
    legacy = [verdicts.get(name) for name in LEGACY_DESIGN]
    if DESIGN not in out and any(v is not None for v in legacy):
        out[DESIGN] = APPROVE if all(v == APPROVE for v in legacy) else "REQUEST_CHANGES"
    return out


def under(path: str, roots: Iterable[str]) -> bool:
    """Whether `path` is one of `roots` or under one of them."""
    return any(path == root or path.startswith(root + "/") for root in roots)


def needs_security(path: str) -> bool:
    """Whether a change to `path` needs the security reviewer (decision 0236). Docs that code
    reads (`ci_changes.is_code_read`, e.g. docs/excel-semantics/) keep it, as CI treats them as code."""
    return under(path, SECURITY_KEEP_PATHS) or ci_changes.is_code_read(path) or not under(path, SECURITY_SKIP_PATHS)


def task_only(paths: Iterable[str]) -> bool:
    """Whether every path is a task file, a `.md` under `docs/tasks/` (decision 0231)."""
    return all(p.startswith("docs/tasks/") and p.endswith(".md") for p in paths)


def applicable_reviewers(paths: Iterable[str] | None) -> tuple[str, ...]:
    """The reviewers a diff of `paths` needs, in REVIEWERS order; every reviewer when the paths
    are unknown (None) or empty, and none when the diff is task-only (decision 0231)."""
    listed = None if paths is None else list(paths)
    if not listed:
        return REVIEWERS
    if task_only(listed):
        return ()
    needed = set(ALWAYS_REVIEWERS)
    if any(needs_security(p) for p in listed):
        needed.add(SECURITY)
    if any(under(p, EXCEL_FIDELITY_PATHS) for p in listed):
        needed.add(EXCEL_FIDELITY)
    return tuple(name for name in REVIEWERS if name in needed)
