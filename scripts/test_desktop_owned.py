"""Unit tests for scripts/desktop_owned.py (stdlib only; run by `make hygiene`)."""

from __future__ import annotations

import io
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import desktop_owned  # noqa: E402 (the sys.path line above must run first)
import exit_owners  # noqa: E402

GEN, OTHER = "gen-m9/22656c225ff4f22a.case.json", "gen-m9/43a13a091907b402.case.json"
POI = "poi/FormulaEvalTestData_Copy.xlsx"

# One baseline file: `C` lines name the cases, `F` lines their known failures; `*`,
# `!EngineError` and the `TF` text line are no cells.
BASELINE = f"""# a comment line
C\t{GEN}\t400
C\t{OTHER}\t400
C\t{POI}\t3000
F\t{GEN}\tCalc!A4
F\t{GEN}\tCalc!A7
F\t{GEN}\tCalc!A15
F\t{OTHER}\tCalc!A10
F\t{OTHER}\t*
F\t{POI}\tEverythingTests!J136
F\t{POI}\t!EngineError
TF\t{GEN}\tCalc!A4
"""


class DesktopOwnedTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.baseline = self.root / "baseline"
        self.baseline.mkdir()
        (self.baseline / "all.txt").write_text(BASELINE)
        self.tasks = self.root / "tasks"
        self.n = 0

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def task(self, body: str, milestone: str, status: str = "todo") -> str:
        self.n += 1
        ident = f"T-{9000 + self.n}"
        path = self.tasks / milestone / f"{ident}-x.md"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(f"---\nid: {ident}\nmilestone: {milestone}\nstatus: {status}\n---\n\n{body}\n")
        return ident

    def listed(self) -> list[str]:
        return [f"{c.case}!{c.key}\t{o}" for c, o in desktop_owned.desktop_owned(self.baseline, self.tasks)]

    def test_lists_cells_only_open_m14_tasks_own(self) -> None:
        etsy = self.task(f"- {GEN} (2): Calc!A4, A7\n- 43a13a091907b402: Calc!A10\n", "M14")
        self.task(f"{POI} EverythingTests!J136 (T-1075 item 7)", "M14", status="done")
        # A non-M14 task naming A7 keeps it in the rate (decision 0143: M14 must own it alone).
        self.task(f"{GEN} Calc!A7 is a lift error", "M9")
        self.assertEqual(self.listed(), [f"{GEN}!Calc!A4\t{etsy}", f"{OTHER}!Calc!A10\t{etsy}"])

    def test_a_case_named_alone_owns_its_cells_and_the_first_m14_owner_is_named(self) -> None:
        first = self.task(f"Whole case: {GEN}\n", "M14", status="blocked")
        self.task(f"{GEN} Calc!A15", "M14")
        self.assertEqual(
            self.listed(), [f"{GEN}!Calc!A4\t{first}", f"{GEN}!Calc!A7\t{first}", f"{GEN}!Calc!A15\t{first}"]
        )

    def test_agrees_with_exit_owners_owned_on_every_cell(self) -> None:
        # One definition of ownership (directive 3): a cell is listed exactly when `owned` says an
        # open M14 task owns it and no other open task does.
        self.task(f"{GEN} Calc!A4, A15\n{POI}\n", "M14")
        self.task(f"{GEN}: A15 only", "M2")
        cases, cells = desktop_owned.baseline_items(self.baseline)
        called = exit_owners.case_names(cases)
        m14 = exit_owners.open_task_texts(self.tasks, "M14")
        rest = [t for m, t in exit_owners.open_tasks(self.tasks) if m.get("milestone") != "M14"]
        expected = [
            f"{c.case}!{c.key}"
            for c in cells
            if exit_owners.owned(c, m14, called[c.case]) and not exit_owners.owned(c, rest, called[c.case])
        ]
        self.assertEqual([line.split("\t")[0] for line in self.listed()], expected)
        self.assertEqual(expected, [f"{GEN}!Calc!A4", f"{POI}!EverythingTests!J136"])

    def test_skips_whole_case_items_and_text_lines(self) -> None:
        _cases, cells = desktop_owned.baseline_items(self.baseline)
        self.assertNotIn("*", [c.key for c in cells])
        self.assertNotIn("!EngineError", [c.key for c in cells])
        self.assertEqual(len(cells), 5)

    def test_main_writes_the_list(self) -> None:
        self.task(f"{GEN} Calc!A4", "M14")
        out = self.root / "runs" / "desktop-owned.tsv"
        argv = ["--baseline", str(self.baseline), "--tasks", str(self.tasks), "--out", str(out)]
        with redirect_stdout(io.StringIO()) as printed:
            self.assertEqual(desktop_owned.main(argv), 0)
        self.assertEqual(out.read_text(), f"{GEN}!Calc!A4\tT-9001\n")
        self.assertIn("1 known-failure cells", printed.getvalue())


if __name__ == "__main__":
    unittest.main()
