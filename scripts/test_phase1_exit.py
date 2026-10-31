"""Unit tests for scripts/phase1_exit.py (stdlib only; run by `make hygiene`)."""

from __future__ import annotations

import io
import json
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parent))
import phase1_exit  # noqa: E402 (the sys.path line above must run first)

GEN = "gen-m9/0123456789abcdef.case.json"
TSV = "# provenance\nname\tms_category\tmin_args\tsyntax\tpage\tnote\nSUM\tMath\t1\tSUM(x)\thttps://x\t\nN\tInfo\t1\tN(x)\thttps://x\t\nCALL\tAdd-in\t1\tCALL(x)\thttps://x\t\n"
YAML = (
    "functions:\n"
    "  - name: SUM\n    category: math\n    milestone: M2\n    scope: in\n"
    '  - name: "N"\n    category: information\n    milestone: M5\n'
    '  - { name: CALL, category: addin, milestone: M5, scope: out, reason: "add-in" }\n'
)


def report(**over: Any) -> dict[str, Any]:
    """A report every measured criterion of which is met."""
    base: dict[str, Any] = {
        "run": {"commit": "abc12345", "dirty": False, "skipped_suites": []},
        "v15_checkpoint": {
            "phase1_met": True,
            "phase1_line": "Phase 1 (decision 0114): met",
            "stale_exceptions": [],
            "unchecked_exceptions": [],
        },
        "milestone_exit": [{"milestone": m, "exit": True} for m in phase1_exit.PHASE1_MILESTONES],
        "out_of_scope": [{"name": "CALL", "reason": "add-in"}],
        "cases": [],
        "attributed_off": [],
        "unattributed_off": [],
        "probe_milestones": [],
    }
    base.update(over)
    return base


class Phase1Exit(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        root = Path(self.tmp.name)
        self.tasks = root / "tasks"
        self.tasks.mkdir()
        self.catalog = root / "catalog"
        self.catalog.mkdir()
        (self.catalog / "excel-functions.tsv").write_text(TSV)
        (self.catalog / "functions.yaml").write_text(YAML)
        self.n = 0

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def task(self, body: str, milestone: str = "M9", defer: bool = False) -> None:
        self.n += 1
        d = self.tasks / milestone
        d.mkdir(exist_ok=True)
        extra = "defer: phase-1b\n" if defer else ""
        (d / f"T-{9000 + self.n}-x.md").write_text(
            f"---\nid: T-{9000 + self.n}\nmilestone: {milestone}\nstatus: todo\n{extra}---\n\n{body}\n"
        )

    def run_main(self, rep: dict[str, Any]) -> tuple[int, str]:
        path = Path(self.tmp.name) / "report.json"
        path.write_text(json.dumps(rep))
        out = io.StringIO()
        with redirect_stdout(out):
            code = phase1_exit.main(
                ["--report", str(path), "--tasks", str(self.tasks), "--catalog", str(self.catalog)]
            )
        return code, out.getvalue()

    def failing(self, text: str) -> str:
        return text.strip().splitlines()[-1]

    def test_all_met_exits_0_and_lists_the_manual_items(self) -> None:
        code, text = self.run_main(report())
        self.assertEqual(code, 0, text)
        self.assertIn("catalog\tmet\t3 Excel functions, 2 in scope, 1 out", text)
        self.assertEqual(text.count("\tmanual\t"), len(phase1_exit.MANUAL_ITEMS))
        self.assertTrue(self.failing(text).startswith("Phase 1 exit: met"))

    def test_a_milestone_row_not_met(self) -> None:
        rows = [{"milestone": m, "exit": m != "M7"} for m in phase1_exit.PHASE1_MILESTONES]
        code, text = self.run_main(report(milestone_exit=rows))
        self.assertEqual(code, 1)
        self.assertIn("milestone M7\tnot met", text)
        self.assertEqual(self.failing(text), "Phase 1 exit: not met: milestone M7")

    def test_a_missing_milestone_row_is_not_met(self) -> None:
        rows = [{"milestone": m, "exit": True} for m in phase1_exit.PHASE1_MILESTONES if m != "M10"]
        code, text = self.run_main(report(milestone_exit=rows))
        self.assertEqual(code, 1)
        self.assertIn("milestone M10\tnot met\tno milestone_exit row", text)

    def test_phase1_met_false(self) -> None:
        code, text = self.run_main(
            report(v15_checkpoint={"phase1_met": False, "phase1_line": "not met (1/2)"})
        )
        self.assertEqual(code, 1)
        self.assertEqual(self.failing(text), "Phase 1 exit: not met: function rate")

    def test_a_stale_exception_fails_the_rate_even_when_met(self) -> None:
        v15 = dict(report()["v15_checkpoint"], stale_exceptions=["gen-m9/x!Calc!A1"])
        code, text = self.run_main(report(v15_checkpoint=v15))
        self.assertEqual(code, 1)
        self.assertIn("function rate\tnot met", text)
        self.assertIn("1 stale exception lines", text)

    def test_a_skipped_suite(self) -> None:
        run = {
            "commit": "abc12345",
            "dirty": False,
            "skipped_suites": [{"name": "poi", "reason": "not fetched"}],
        }
        code, text = self.run_main(report(run=run))
        self.assertEqual(code, 1)
        self.assertIn("run\tnot met\tat abc12345: skipped suites: poi", text)

    def test_a_dirty_run(self) -> None:
        code, text = self.run_main(report(run={"commit": "abc12345", "dirty": True, "skipped_suites": []}))
        self.assertEqual(code, 1)
        self.assertIn("dirty tree", text)

    def unowned_off_report(self) -> dict[str, Any]:
        off = {
            "function": "GAMMA",
            "case": GEN,
            "cell": "Calc!A1",
            "formula": "=GAMMA(2)",
            "expected": 1,
            "actual": 2,
        }
        return report(
            attributed_off=[off],
            cases=[{"id": GEN, "suite": "gen-m9", "cells": []}],
        )

    def test_one_unowned_cell(self) -> None:
        code, text = self.run_main(self.unowned_off_report())
        self.assertEqual(code, 1)
        self.assertIn("owners\tnot met\t1 visibly-off cells with no open owner (0 owned only", text)

    def test_an_owned_cell_is_met(self) -> None:
        self.task("gen-m9/0123456789abcdef Calc!A1 is off.")
        code, text = self.run_main(self.unowned_off_report())
        self.assertEqual(code, 0, text)

    def test_a_cell_owned_only_by_a_deferred_task_is_unowned(self) -> None:
        self.task("gen-m9/0123456789abcdef Calc!A1 is off.", defer=True)
        code, text = self.run_main(self.unowned_off_report())
        self.assertEqual(code, 1)
        self.assertIn("1 visibly-off cells with no open owner (1 owned only by a deferred task)", text)

    def test_a_catalog_gap(self) -> None:
        (self.catalog / "excel-functions.tsv").write_text(TSV + "ABS\tMath\t1\tABS(x)\thttps://x\t\n")
        code, text = self.run_main(report())
        self.assertEqual(code, 1)
        self.assertIn("catalog\tnot met\tnot in the catalog: ABS", text)

    def test_a_report_out_of_scope_list_that_differs(self) -> None:
        code, text = self.run_main(report(out_of_scope=[]))
        self.assertEqual(code, 1)
        self.assertIn("catalog\tnot met\treport out_of_scope differs: CALL", text)

    def test_the_real_catalog_parses_completely(self) -> None:
        # The line parser must keep up with functions.yaml's layout: a reformat that hides an entry
        # or its scope shows here, not as a silent `met` (tools/tests/test_catalog.py holds the
        # same set equality through PyYAML).
        line = phase1_exit.catalog_line(phase1_exit.CATALOG)
        self.assertEqual(line.verdict, "met", line.detail)
        scopes = phase1_exit.catalog_scopes(phase1_exit.CATALOG / "functions.yaml")
        self.assertIn("out", scopes.values())

    def test_json_output(self) -> None:
        path = Path(self.tmp.name) / "report.json"
        path.write_text(json.dumps(report()))
        out = io.StringIO()
        args = ["--report", str(path), "--tasks", str(self.tasks), "--catalog", str(self.catalog), "--json"]
        with redirect_stdout(out):
            self.assertEqual(phase1_exit.main(args), 0)
        doc = json.loads(out.getvalue())
        self.assertEqual(doc["overall"], "met")
        self.assertEqual(doc["failing"], [])
        self.assertEqual(doc["lines"][0]["criterion"], "run")


if __name__ == "__main__":
    unittest.main()
