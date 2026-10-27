"""Unit tests for scripts/exit_owners.py (stdlib only; run by `make hygiene`)."""

from __future__ import annotations

import io
import json
import sys
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import exit_owners  # noqa: E402 (the sys.path line above must run first)

GEN = "gen-m5/1d22a002d4f380b3.case.json"
TRIG = "ironcalc-calc/MATH/TRIGONOMETRIC.xlsx"
PROBE = "probes/index-reference-args.case.json"


def cell(sheet: str, a1: str, formula: str = "=X()") -> dict[str, object]:
    return {"sheet": sheet, "cell": a1, "formula": formula, "verdict": "numeric_ulp"}


def report() -> dict[str, object]:
    return {
        "milestone_exit": [
            {
                "milestone": "M5",
                "probe_row": "M5",
                "gen_visibly_off_cells": [f"{GEN}!Calc!A1", f"{GEN}!Calc!A107"],
            }
        ],
        "cases": [
            {
                "id": GEN,
                "suite": "gen-m5",
                # A2 fails within 15 digits: not visibly off, so not an exit cell (decision 0135).
                "cells": [cell("Calc", "A1"), cell("Calc", "A107", "=CSCH(PI()/4)"), cell("Calc", "A2")],
            },
            {"id": TRIG, "suite": "ironcalc-calc", "cells": [cell("S", "C51"), cell("S", "C55"), cell("S", "C60")]},
            {"id": PROBE, "suite": "probes", "cells": [cell("P", "B2"), cell("P", "B3"), cell("P", "B4")]},
        ],
        "function_scope": {
            "rows": [
                {
                    "milestone": "M5",
                    "failing": [f"{TRIG}!S!C51", f"{TRIG}!S!C55", f"{TRIG}!S!C60"],
                    "excepted": [f"{TRIG}!S!C55"],
                    "visibly_off": [f"{TRIG}!S!C51", f"{TRIG}!S!C55"],
                }
            ]
        },
        "probe_milestones": [
            {
                "milestone": "M5",
                "failing_cases": [[PROBE, 3]],
                "visibly_off_cells": [f"{PROBE}!P!B2", f"{PROBE}!P!B3"],
            }
        ],
    }


class TaskDir(unittest.TestCase):
    """A temporary tasks directory and a helper that writes a task into it."""

    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.tasks = Path(self.tmp.name)
        self.n = 0

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def task(self, body: str, milestone: str = "M5", status: str = "todo") -> None:
        self.n += 1
        d = self.tasks / milestone
        d.mkdir(exist_ok=True)
        (d / f"T-{9000 + self.n}-x.md").write_text(
            f"---\nid: T-{9000 + self.n}\nmilestone: {milestone}\nstatus: {status}\n---\n\n{body}\n"
        )


class ExitOwners(TaskDir):
    def listed(self) -> list[str]:
        return [f"{c.case}!{c.key}" for c in exit_owners.unowned(report(), "M5", self.tasks)]

    def test_a_cell_named_by_an_open_task_is_owned(self) -> None:
        self.task("Owns 1d22a002d4f380b3 Calc!A107.")
        self.assertNotIn(f"{GEN}!Calc!A107", self.listed())
        self.assertIn(f"{GEN}!Calc!A1", self.listed())

    def test_done_or_other_milestone_tasks_own_nothing(self) -> None:
        self.task("1d22a002d4f380b3 A107", status="done")
        self.task("1d22a002d4f380b3 A107", milestone="M6")
        self.assertIn(f"{GEN}!Calc!A107", self.listed())

    def test_a_case_named_without_cells_owns_all_of_it(self) -> None:
        self.task("The failing cells of `probes/index-reference-args` are this task's.")
        listed = self.listed()
        self.assertNotIn(f"{PROBE}!P!B2", listed)
        self.assertNotIn(f"{PROBE}!P!B3", listed)

    def test_a_case_named_beside_other_cells_owns_only_those(self) -> None:
        self.task("- `TRIGONOMETRIC.xlsx` SINH_COSH_TANH!D51 and B15 (both pass now)")
        self.assertIn(f"{TRIG}!S!C51", self.listed())

    def test_an_excepted_calc_cell_is_not_listed(self) -> None:
        listed = self.listed()
        self.assertIn(f"{TRIG}!S!C51", listed)
        self.assertNotIn(f"{TRIG}!S!C55", listed)

    def test_only_visibly_off_cells_are_exit_cells(self) -> None:
        listed = self.listed()
        self.assertNotIn(f"{GEN}!Calc!A2", listed)
        self.assertNotIn(f"{TRIG}!S!C60", listed)
        self.assertNotIn(f"{PROBE}!P!B4", listed)

    def test_m2_probe_cells_come_from_the_m0_m2_row(self) -> None:
        r = report()
        r["milestone_exit"] = [{"milestone": "M2", "probe_row": "M0-M2"}]
        r["function_scope"] = {"rows": []}
        r["probe_milestones"] = [
            {"milestone": "M0-M2", "failing_cases": [[PROBE, 3]], "visibly_off_cells": [f"{PROBE}!P!B3"]},
            {"milestone": "M5", "failing_cases": [[PROBE, 3]], "visibly_off_cells": [f"{PROBE}!P!B2"]},
        ]
        listed = [f"{c.case}!{c.key}" for c in exit_owners.unowned(r, "M2", self.tasks)]
        self.assertEqual(listed, [f"{PROBE}!P!B3"])
        self.task("probes/index-reference-args P!B3", milestone="M2")
        self.assertEqual(exit_owners.unowned(r, "M2", self.tasks), [])

    def test_a1_does_not_match_a10(self) -> None:
        self.task("1d22a002d4f380b3 A107 and A10")
        listed = self.listed()
        self.assertIn(f"{GEN}!Calc!A1", listed)
        self.assertNotIn(f"{GEN}!Calc!A107", listed)

    def test_a_stem_must_be_whole(self) -> None:
        self.task("probes/index-reference-args-2")
        self.assertIn(f"{PROBE}!P!B2", self.listed())

    def test_output_and_strict_exit(self) -> None:
        path = self.tasks / "report.json"
        path.write_text(json.dumps(report()))
        args = ["--milestone", "M5", "--report", str(path), "--tasks", str(self.tasks)]
        out = io.StringIO()
        with redirect_stdout(out):
            self.assertEqual(exit_owners.main(args), 0)
            self.assertEqual(exit_owners.main([*args, "--strict"]), 1)
        lines = out.getvalue().splitlines()
        self.assertIn(f"gen-m5\t{GEN}!Calc!A107\t=CSCH(PI()/4)", lines)
        self.assertEqual(lines[5], "5 failing M5 exit cells have no open M5 owner")

    def test_missing_report_or_milestone_fails_clearly(self) -> None:
        err = io.StringIO()
        with redirect_stderr(err):
            self.assertEqual(
                exit_owners.main(["--milestone", "M5", "--report", str(self.tasks / "no.json")]), 2
            )
            path = self.tasks / "report.json"
            path.write_text(json.dumps(report()))
            self.assertEqual(exit_owners.main(["--milestone", "M7", "--report", str(path)]), 2)
            self.assertEqual(exit_owners.main(["--milestone", "M5'", "--report", str(path)]), 2)
        self.assertIn("run `make eval` first", err.getvalue())
        self.assertIn("no milestone_exit row for M7", err.getvalue())


class UnattributedOwners(TaskDir):
    UNATTRIBUTED = {
        "unattributed_off": [
            {
                "case": PROBE,
                "cell": "P!B2",
                "formula": "GAMMA(BETA.DIST(0.5,2,3,TRUE))",
            },
            {"case": GEN, "cell": "Calc!A9", "formula": "IF(AND(A1,B1),1,2)"},
        ]
    }

    def unowned(self) -> list[str]:
        cells = exit_owners.unowned_unattributed(self.UNATTRIBUTED, self.tasks)
        return [f"{c.case}!{c.key}" for c in cells]

    def test_an_unattributed_cell_is_listed_until_an_open_task_of_any_milestone_owns_it(
        self,
    ) -> None:
        self.assertEqual(self.unowned(), [f"{PROBE}!P!B2", f"{GEN}!Calc!A9"])
        self.task("index-reference-args B2", status="done")
        self.assertIn(f"{PROBE}!P!B2", self.unowned())
        self.task("index-reference-args B2", milestone="M9")
        self.assertEqual(self.unowned(), [f"{GEN}!Calc!A9"])

    def test_unattributed_output_and_strict_exit(self) -> None:
        path = self.tasks / "report.json"
        path.write_text(json.dumps(self.UNATTRIBUTED))
        args = ["--unattributed", "--report", str(path), "--tasks", str(self.tasks)]
        out = io.StringIO()
        with redirect_stdout(out):
            self.assertEqual(exit_owners.main(args), 0)
            self.assertEqual(exit_owners.main([*args, "--strict"]), 1)
        lines = out.getvalue().splitlines()
        self.assertEqual(lines[0], f"{PROBE}!P!B2\tGAMMA(BETA.DIST(0.5,2,3,TRUE))")
        count = "2 visibly-off cells attributed to no function have no open owner"
        self.assertEqual(lines[2], count)
        # A report from before T-0082 has no list: nothing is unowned.
        path.write_text("{}")
        with redirect_stdout(io.StringIO()):
            self.assertEqual(exit_owners.main([*args, "--strict"]), 0)


if __name__ == "__main__":
    unittest.main()
