"""Gate A's governance and hygiene rules (SPEC-0029 test plan, T-3379)."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import accuracy_governance as acc  # noqa: E402 (import after sys.path setup)
import check_governance as gov  # noqa: E402

BATCH = "eval/accuracy/tables/T.INV/s1"
NEW = "eval/accuracy/tables/T.INV/r4-T-0002"
STATUS = {"T-0001": "todo", "T-0002": "in-progress", "T-0003": "done"}
HEAD = "# comment\n"


def kf(*lines: str) -> str:
    return HEAD + "".join(f"{line}\n" for line in lines)


def gate(*lines: str, known: list[str] | None = None) -> list[list[str]]:
    return gov.failures(list(lines), lambda _p: ["M2"], flip=False, has_issue=lambda _p: False, known_failures=known or [])


class Tables(unittest.TestCase):
    def test_adding_a_batch_is_free(self) -> None:
        added = [f"A\t{NEW}.case.json", f"A\t{NEW}.math.json", f"A\t{NEW}.oracle.json"]
        self.assertEqual(acc.table_edits(added), [])
        self.assertEqual(gate(*added), [])

    def test_modifying_deleting_or_renaming_a_committed_batch_needs_a_decision_record(self) -> None:
        for line in (
            f"M\t{BATCH}.case.json",
            f"D\t{BATCH}.oracle.json",
            f"T\t{BATCH}.math.json",
            f"R100\t{BATCH}.case.json\t{NEW}.case.json",
        ):
            found = gate(line)
            self.assertEqual(len(found), 1, line)
            self.assertIn("protected paths changed without a decision record", found[0][0])
            self.assertIn(f"  {line.split(chr(9))[1]}", found[0], line)
            self.assertEqual(gate(line, "A\tdocs/decisions/0999-x.md"), [], line)

    def test_a_copy_of_a_batch_is_an_addition(self) -> None:
        self.assertEqual(acc.table_edits([f"C100\t{BATCH}.case.json\t{NEW}.case.json"]), [])


class Lists(unittest.TestCase):
    def test_exempt_and_math_over_excel_are_protected(self) -> None:
        for path in ("eval/accuracy/exempt.tsv", "eval/accuracy/math-over-excel.tsv"):
            self.assertTrue(gov.is_protected(path), path)
            self.assertEqual(len(gate(f"M\t{path}")), 1, path)
            self.assertEqual(gate(f"M\t{path}", "A\tdocs/decisions/0999-x.md"), [], path)

    def test_known_failures_and_new_batches_are_not_protected(self) -> None:
        for path in ("eval/accuracy/known-failures.tsv", f"{NEW}.case.json", "eval/accuracy/suite.toml"):
            self.assertFalse(gov.is_protected(path), path)


class KnownFailures(unittest.TestCase):
    def problems(self, before: str | None, after: str | None, *changed: str) -> list[str]:
        return acc.known_failure_problems(before, after, list(changed), STATUS)

    def test_a_line_for_a_point_of_a_batch_the_diff_adds_passes(self) -> None:
        line = "accuracy/T.INV/r4-T-0002!Calc!A3\tT-0002\tfit does not generalize"
        self.assertEqual(self.problems(None, kf(line), f"A\t{NEW}.case.json"), [])

    def test_a_line_for_a_committed_point_fails(self) -> None:
        line = "accuracy/T.INV/s1!Calc!A3\tT-0001\tregressed"
        found = self.problems(kf(), kf(line), f"A\t{NEW}.case.json")
        self.assertEqual(len(found), 1)
        self.assertIn("line 2: accuracy/T.INV/s1!Calc!A3", found[0])
        self.assertIn("whose batch this diff adds", found[0])

    def test_a_line_naming_no_table_point_fails(self) -> None:
        found = self.problems(None, kf("probes-numeric/x!Calc!A1\tT-0001\tn"))
        self.assertEqual(len(found), 1)
        self.assertIn("not a table point", found[0])

    def test_a_committed_line_may_stay_and_change_owner(self) -> None:
        before = kf("accuracy/T.INV/s1!Calc!A3\tT-0001\tn")
        after = kf("accuracy/T.INV/s1!Calc!A3\tT-0002\tmoved")
        self.assertEqual(self.problems(before, after), [])

    def test_an_owner_that_is_done_or_missing_fails_on_every_line(self) -> None:
        before = kf("accuracy/T.INV/s1!Calc!A3\tT-0003\tn", "accuracy/T.INV/s1!Calc!A4\tT-9999\tn")
        found = self.problems(before, before)
        self.assertEqual(len(found), 2)
        self.assertIn("owner T-0003 is done", found[0])
        self.assertIn("owner T-9999 is not a task", found[1])

    def test_comments_blank_lines_and_a_missing_file_are_empty(self) -> None:
        self.assertEqual(self.problems(None, None), [])
        self.assertEqual(self.problems(None, "# only\n\n"), [])

    def test_problems_fail_the_governance_gate(self) -> None:
        found = gate("M\teval/accuracy/known-failures.tsv", known=["x"])
        self.assertEqual(len(found), 1)
        self.assertIn("Gate A known failures", found[0][0])
        self.assertEqual(found[0][1], "  x")


class Points(unittest.TestCase):
    def test_batch_case_of_a_point(self) -> None:
        self.assertEqual(acc.batch_case("accuracy/T.INV.2T/s1!Calc!A9"), "eval/accuracy/tables/T.INV.2T/s1.case.json")
        self.assertIsNone(acc.batch_case("accuracy/s1!Calc!A9"))
        self.assertIsNone(acc.batch_case("probes/F/s1!Calc!A9"))


if __name__ == "__main__":
    unittest.main()
