"""Gate A's governance rules (SPEC-0029, T-3379), used by scripts/check_governance.py.

The reference tables under eval/accuracy/ cannot be weakened without a decision record:

- A committed batch file under `eval/accuracy/tables/` is immutable: a diff that changes,
  deletes or renames one changes a protected path. Adding a new batch's files does not.
- `exempt.tsv` and `math-over-excel.tsv` are protected paths (`PROTECTED_FILES`).
- Every `known-failures.tsv` line names an existing task that is not `done`, and a line added by
  a diff names a point whose batch the same diff adds, so a regression of a committed point can
  never be listed (SPEC-0029 "The check" step 4).

The format of each list is the eval harness's (crates/rustsheet-eval/src/accuracy/lists.rs);
lines it would refuse are left to it.
"""

from __future__ import annotations

from collections.abc import Iterable, Mapping

ACCURACY = "eval/accuracy/"
TABLES = ACCURACY + "tables/"
KNOWN_FAILURES = ACCURACY + "known-failures.tsv"
# Lists whose every change needs a decision record (SPEC-0029: an exemption hides a function from
# the missing-table count, an R3 mark lets a point ignore Excel).
PROTECTED_FILES = (ACCURACY + "exempt.tsv", ACCURACY + "math-over-excel.tsv")
POINT_PREFIX = "accuracy/"


def table_edits(name_status_lines: Iterable[str]) -> list[str]:
    """Committed table files a diff's `git diff --name-status` lines change, delete or rename.

    An added file (`A`) and the destination of a copy (`C`) or rename are new; the source of a
    rename (`R`) is a deletion; every other status (`M`, `D`, `T`, `U`) changes the file.
    """
    edits = []
    for line in name_status_lines:
        fields = line.split("\t")
        if len(fields) < 2 or not fields[0]:
            continue
        status = fields[0][0]
        if status in ("A", "C"):
            continue
        if fields[1].startswith(TABLES):
            edits.append(fields[1])
    return edits


def added_paths(name_status_lines: Iterable[str]) -> set[str]:
    """Paths a diff adds: `A` lines and the destinations of copies and renames."""
    out = set()
    for line in name_status_lines:
        fields = line.split("\t")
        if len(fields) >= 2 and fields[0][:1] in ("A", "C", "R"):
            out.add(fields[-1])
    return out


def known_failure_lines(text: str | None) -> list[tuple[int, str, str]]:
    """(line number, point id, owner) of each data line of a `known-failures.tsv` text."""
    out = []
    for number, line in enumerate((text or "").splitlines(), start=1):
        if not line.strip() or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) >= 2:
            out.append((number, fields[0], fields[1]))
    return out


def batch_case(point_id: str) -> str | None:
    """The case file of `accuracy/<FUNCTION>/<batch>!<Sheet>!<A1>`, or None for another id."""
    case, _, _ = point_id.partition("!")
    if not case.startswith(POINT_PREFIX) or case.count("/") != 2:
        return None
    return f"{TABLES}{case.removeprefix(POINT_PREFIX)}.case.json"


def known_failure_problems(
    before: str | None,
    after: str | None,
    name_status_lines: list[str],
    task_status: Mapping[str, str],
) -> list[str]:
    """What breaks the known-failure rules, one line per offender.

    `before` and `after` are `known-failures.tsv` at the merge base and at HEAD (None when the
    file does not exist); `task_status` maps every task id to its status.
    """
    problems = []
    old = {point for _, point, _ in known_failure_lines(before)}
    added = added_paths(name_status_lines)
    for number, point, owner in known_failure_lines(after):
        where = f"{KNOWN_FAILURES} line {number}: {point}"
        status = task_status.get(owner)
        if status is None:
            problems.append(f"{where}: owner {owner} is not a task")
        elif status == "done":
            problems.append(f"{where}: owner {owner} is done; fix the point or name its open owner")
        if point in old:
            continue
        case = batch_case(point)
        if case is None or case not in added:
            problems.append(
                f"{where}: a new line may name only a point whose batch this diff adds"
                f" ({case or 'not a table point'}); a committed point that fails is a regression"
            )
    return problems
