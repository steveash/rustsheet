"""Unit tests for scripts/new_case_owners.py (stdlib only; run by `make hygiene`)."""

from __future__ import annotations

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import exit_owners  # noqa: E402 (the sys.path line above must run first)
import new_case_owners  # noqa: E402
from test_exit_owners import TaskDir  # noqa: E402

NEW = "probes/new-probe.case.json"
OLD = "probes/old-probe.case.json"


def off(case: str, a1: str) -> dict[str, object]:
    return {
        "function": "SUM",
        "case": case,
        "cell": f"P!{a1}",
        "formula": "=SUM()",
        "expected": "1",
        "actual": "2",
    }


def report() -> dict[str, object]:
    cells = [off(NEW, a) for a in ("B1", "B2", "B3")] + [off(OLD, "B1")]
    return {"cases": [{"id": NEW, "cells": []}, {"id": OLD, "cells": []}], "attributed_off": cells}


class UnownedNew(TaskDir):
    def listed(self) -> list[str]:
        names = exit_owners.case_names([NEW, OLD])
        return [
            f"{o.cell.case}!{o.cell.key}"
            for o in new_case_owners.unowned_new(report(), self.tasks, {NEW}, names)
        ]

    def test_a_new_case_with_no_owner_fails(self) -> None:
        self.assertEqual(self.listed(), [f"{NEW}!P!B1", f"{NEW}!P!B2", f"{NEW}!P!B3"])

    def test_a_task_naming_the_case_owns_it(self) -> None:
        self.task("probes/new-probe")
        self.assertEqual(self.listed(), [])

    def test_a_colon_span_owns_its_cells(self) -> None:
        self.task("probes/new-probe P!B1:B3")
        self.assertEqual(self.listed(), [])

    def test_a_hyphen_span_leaves_its_middle_cell(self) -> None:
        self.task("probes/new-probe B1-B3")
        self.assertEqual(self.listed(), [f"{NEW}!P!B2"])

    def test_a_deferred_owner_passes_and_a_done_one_does_not(self) -> None:
        self.task("probes/new-probe", status="done")
        self.assertEqual(len(self.listed()), 3)
        self.n += 1
        (self.tasks / "M5" / "T-9999-d.md").write_text(
            "---\nid: T-9999\nmilestone: M5\nstatus: todo\ndefer: phase-1b\n---\n\nprobes/new-probe\n"
        )
        self.assertEqual(self.listed(), [])

    def test_an_old_case_is_not_checked(self) -> None:
        self.assertNotIn(f"{OLD}!P!B1", self.listed())


def git(root: Path, *args: str) -> None:
    subprocess.run(["git", *args], cwd=root, check=True, capture_output=True)


class NewCases(unittest.TestCase):
    """`changed_paths` and `new_cases` on a scratch repository."""

    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        suites = self.root / "eval" / "suites"
        (suites / "probes" / "cases").mkdir(parents=True)
        (suites / "probes" / "suite.toml").write_text('name = "probes"\nroot = "cases"\n')
        # No `root`: the case root defaults to `cases`, and the id takes the manifest's `name`.
        (suites / "gen" / "cases").mkdir(parents=True)
        (suites / "gen" / "suite.toml").write_text('name = "gen-x"\n')
        self.write("gen/cases/h1.case.json")
        (suites / "poi").mkdir()
        (suites / "poi" / "suite.toml").write_text('name = "poi"\nfetch = "x"\n')
        self.write("probes/cases/old.case.json")
        self.write("probes/cases/old.oracle.json")
        self.write("probes/cases/kept.case.json")
        git(self.root, "init", "-q", "-b", "main")
        git(self.root, "add", "-A")
        git(self.root, "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "base")
        git(self.root, "branch", "base")

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def write(self, rel: str, text: str = "{}") -> None:
        path = self.root / "eval" / "suites" / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)

    def found(self) -> dict[str, set[str]]:
        return new_case_owners.new_cases(self.root, new_case_owners.changed_paths(self.root, "base"))

    def test_no_change_is_no_new_case(self) -> None:
        self.assertEqual(self.found(), {})

    def test_an_added_case_and_a_refilled_sidecar_are_new(self) -> None:
        self.write("probes/cases/sub/added.case.json")
        self.write("probes/cases/old.oracle.json", '{"refilled": 1}')
        self.assertEqual(self.found(), {"probes": {"probes/sub/added.case.json", "probes/old.case.json"}})

    def test_a_vendored_suite_and_a_deleted_case_are_not(self) -> None:
        self.write("poi/x.xlsx")
        (self.root / "eval" / "suites" / "probes" / "cases" / "kept.case.json").unlink()
        self.assertEqual(self.found(), {})

    def test_a_default_root_case_is_named_by_the_suite_name(self) -> None:
        self.write("gen/cases/h2.case.json")
        self.assertEqual(self.found(), {"gen-x": {"gen-x/h2.case.json"}})

    def test_all_case_ids_lists_the_local_suites_only(self) -> None:
        self.write("poi/x.xlsx")
        self.assertEqual(
            new_case_owners.all_case_ids(self.root),
            ["gen-x/h1.case.json", "probes/kept.case.json", "probes/old.case.json"],
        )

    def test_a_sidecar_without_its_case_names_none(self) -> None:
        self.write("probes/cases/orphan.oracle.json")
        self.assertEqual(self.found(), {})

    def test_a_missing_base_is_an_error(self) -> None:
        with self.assertRaises(new_case_owners.GateError):
            new_case_owners.changed_paths(self.root, "no-such-ref")


if __name__ == "__main__":
    unittest.main()
